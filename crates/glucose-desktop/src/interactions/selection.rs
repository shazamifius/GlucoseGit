//! Gestion de la sélection d'éléments et de la boîte élastique (Marquee).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::geometry::Rect;
use glucose_core::hit_priority::{collect_candidates_indexed, PickCandidate, PickInput};

/// Déplacement, en pixels **logiques**, au-delà duquel un clic maintenu sur le vide devient
/// une sélection élastique (fiche 07 § 7.3) : en deçà, c'est un clic, qui ne sélectionne rien.
pub const RUBBERBAND_MIN_PX: f64 = 4.0;

impl GlucoseApp {
    /// Démarre le tracé d'un rectangle de sélection élastique.
    pub fn start_selection_box(&mut self, sx: f64, sy: f64) {
        self.selection_box = Some((sx, sy, sx, sy));
    }

    /// Met à jour la position courante du rectangle élastique.
    pub fn update_selection_box(&mut self, sx: f64, sy: f64) {
        if let Some((x1, y1, _, _)) = self.selection_box {
            self.selection_box = Some((x1, y1, sx, sy));
        }
    }

    /// Applique la sélection élastique : tout élément dont la boîte croise le rectangle est
    /// sélectionné — et pour une flèche, la boîte est l'enveloppe de son tracé. **Les dossiers
    /// aussi** (DOSSIER-2, fiche 130) : sa note du 22/11, *« lorsqu'on maintient l'outil
    /// sélectionner, certains nœuds ne sont pas compris dedans, comme les dossiers »*.
    pub fn finish_selection_box(&mut self) {
        let membrane = self.membrane_au_clic.take();
        let Some((x1, y1, x2, y2)) = self.selection_box.take() else {
            return;
        };
        // En deçà du seuil sur les deux axes, c'est un clic, pas un geste : dans le vide d'une
        // membrane, il la choisit (MEMB-SEL-1).
        let seuil = RUBBERBAND_MIN_PX * self.densite();
        if (x2 - x1).abs() <= seuil && (y2 - y1).abs() <= seuil {
            if let Some(top) = membrane {
                self.select_node(&top);
            }
            return;
        }
        let vp = self.store.viewport();
        let (left, top) = screen_to_world(x1.min(x2), y1.min(y2), &vp);
        let (right, bottom) = screen_to_world(x1.max(x2), y1.max(y2), &vp);
        let lasso = Rect::new(left, top, right - left, bottom - top);

        let Some(board) = self.store.active_board() else {
            return;
        };
        let focus = &self.renderer.focus;
        let pris = board.pris_par(lasso, |id| focus.laisse_voir(id));

        if !self.modifiers.shift_key() {
            self.store.clear_selection();
        }
        for id in pris.images {
            self.store.select_image(id, true);
        }
        for id in pris.annotations {
            self.store.select_annotation(id, true);
        }
        for id in pris.dossiers {
            self.store.select_folder(id, true);
        }
    }

    /// **MEMB-SEL-1 — dans le vide d'une membrane qu'on n'a pas choisie, le glisser trace le
    /// rectangle de sélection** (sa note du 24/11, `text-g420-2316` : *« au sein d'une membrane,
    /// si on veut faire un maintien du clic pour effectuer une sélection, c'est impossible car ça
    /// sélectionne la membrane »*). Un clic sans bouger la choisit, au relâchement ; choisie, elle
    /// se glisse par son vide ; son titre et son bord la prennent toujours d'un geste. Aucune durée
    /// à attendre : c'est la sélection qui départage, comme les sections de Figma (choisie, elle se
    /// déplace ; sinon, le rectangle). `Maj` (l'ajouter) et `Alt` (la pile) gardent leur sens ; le
    /// doigt, lui, déplace le canevas dans le vide et garde son geste.
    pub(crate) fn appuyer_dans_une_membrane(&mut self, top: &PickCandidate) -> bool {
        if top.kind != glucose_core::hit_priority::PickKind::MembraneBody
            || self.au_doigt()
            || self.modifiers.shift_key()
            || self.modifiers.alt_key()
            || self.est_deja_selectionne(top)
        {
            return false;
        }
        self.commit_editing();
        self.store.clear_selection();
        self.membrane_au_clic = Some(top.clone());
        self.start_selection_box(self.mouse_pos.0, self.mouse_pos.1);
        true
    }

    /// **Ce que l'arbitre de clic lit** de ce tableau et de la sélection, en ce point du monde.
    fn entree_du_clic<'a>(
        &'a self,
        board: &'a glucose_core::types::Board,
        (wx, wy): (f64, f64),
        noeuds: Option<&'a dyn glucose_core::arrow::Noeuds>,
    ) -> PickInput<'a> {
        PickInput {
            wx,
            wy,
            scale: board.viewport.scale / self.densite(),
            images: &board.images,
            annotations: &board.annotations,
            folders: &board.folders,
            selected_image_ids: &self.store.selected_image_ids,
            selected_annotation_ids: &self.store.selected_annotation_ids,
            selected_folder_ids: &self.store.selected_folder_ids,
            noeuds,
        }
    }

    /// **Le cadre du groupe** quand la sélection compte deux nœuds ou plus — la même boîte que
    /// l'arbitre de clic lit pour ses poignées (fiche 53 § 10).
    pub(crate) fn emprise_du_groupe(&self) -> Option<glucose_core::smart_align::AlignRect> {
        self.store
            .emprise_du_groupe(&self.store.project.active_board_id)
    }

    /// Résout l'élément sous le clic à l'aide de hit_priority.
    pub fn pick_candidate_at(&self, wx: f64, wy: f64) -> Option<PickCandidate> {
        self.pick_candidates_at(wx, wy).into_iter().next()
    }

    /// **Toute** la pile sous `(wx, wy)`, dans l'ordre de l'arbitre.
    ///
    /// [`Self::pick_candidate_at`] n'en garde que le sommet, ce qui suffit à la plupart des
    /// gestes ; le cycle de profondeur, lui, a besoin de ce qu'il y a dessous (PICK-1).
    pub fn pick_candidates_at(&self, wx: f64, wy: f64) -> Vec<PickCandidate> {
        let Some(board) = self.store.active_board() else {
            return Vec::new();
        };
        // Une flèche ancrée à un passage se vise là où le dessin la pose (FLECHE-4).
        let noeuds = crate::renderer::arrow::NoeudsDuRendu {
            board,
            index: Some(&self.renderer.spatial_hash),
            typographie: &self.renderer.typography,
            math: &self.renderer.math,
            contournement: Some(self.renderer.contournement()),
        };
        let input = self.entree_du_clic(board, (wx, wy), Some(&noeuds));
        let mut candidats = collect_candidates_indexed(&input, &self.renderer.spatial_hash);
        // En focus, ce qu'on ne voit pas ne s'attrape pas (MEMB-2).
        candidats.retain(|c| self.renderer.focus.laisse_voir(&c.id));
        candidats
    }
}

#[cfg(test)]
mod tests;
