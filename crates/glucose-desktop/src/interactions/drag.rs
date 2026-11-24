//! Gestion du glisser-déplacer d'éléments avec alignement intelligent (SNAP-1).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::geometry::Rect;
use glucose_core::hit_priority::PickOwner;
use glucose_core::smart_align::{snap_move_sur, union_rect, AlignRect, SnapGuides};
use std::collections::HashSet;

/// **La base d'un glisser**, notée à l'appui : ce que l'aimant regarde — les boîtes de ce qui est
/// emporté —, et ce que chaque pas salit — leur encre, le titre des membranes compris (TITRE-1).
/// Un grand titre bouge avec sa membrane ; l'aimant, lui, aligne des bords.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseDuGlisser {
    pub boite: AlignRect,
    pub encre: Rect,
}

/// **GLISSER-2 — ce qu'on glisse avance de pixels entiers.** Le déplacement du geste, en unités
/// monde, arrondi au pixel entier de l'écran où il se fait (`echelle` : pixels par unité).
///
/// Sous Wayland, la souris donne des positions fractionnaires (l'accélération de libinput, une
/// échelle de 125 ou 150 %) : chaque pas posait ce qu'on glisse à une autre phase sous-pixel, et
/// chaque composant emporté — une membrane et tous ses textes — se rendait de nouveau, à chaque
/// image (fiche 151 : 3,4 ms de textures par image en 4K à 150 %, 0,03 ms par pas entiers). Par
/// pixels entiers, la phase reste celle du départ, et la texture celle qu'on avait. Un demi-pixel
/// de retard au plus sur la main : rien ne se voit, rien ne se refait.
fn au_pixel_entier((dx, dy): (f64, f64), echelle: f64) -> (f64, f64) {
    (
        (dx * echelle).round() / echelle,
        (dy * echelle).round() / echelle,
    )
}

/// **Ce qu'un glisser porte**, de l'appui au lâcher — un seul état, remis à zéro d'un coup.
#[derive(Debug, Clone, Default)]
pub struct Glisser {
    /// Où la main a appuyé, en unités monde.
    pub depart: (f64, f64),
    /// Ce que l'aimant regarde, et ce que chaque pas salit.
    pub base: Option<BaseDuGlisser>,
    /// Ce que le glisser emporte, que l'aimant ne regarde pas (SNAP-4).
    pub exclus: HashSet<String>,
    /// Le déplacement déjà appliqué, aimant compris.
    pub applique: (f64, f64),
    /// **Le dossier sous ce qu'on glisse** : le lâcher l'y range (RANGER-2, fiche 134).
    pub depot: Option<String>,
}

impl GlucoseApp {
    /// Initialise une session de drag pour les éléments sélectionnés.
    pub fn init_item_drag(&mut self, wx: f64, wy: f64) {
        self.is_dragging_item = true;
        self.glisser.depart = (wx, wy);
        self.glisser.applique = (0.0, 0.0);
        self.store.begin_live_edit();

        // Préparer la boîte englobante, et ce que l'aimant ne regarde pas (SNAP-1). Ce qui bouge
        // est ce que la sélection emporte — le contenu de ses membranes compris (MEMB-1) : il se
        // redessine avec elle, et une membrane ne s'aimante pas sur ses propres membres.
        let emport = self
            .store
            .ce_qu_emporte_la_selection(&self.store.project.active_board_id);
        if let Some(board) = self.store.active_board() {
            let mut exclude: HashSet<String> = emport.ids().cloned().collect();
            let mut rects = emport.boites(board);
            let mut encre = emport.encre(board);
            for f in board
                .folders
                .iter()
                .filter(|f| self.store.selected_folder_ids.contains(&f.id))
            {
                exclude.insert(f.id.clone());
                rects.push(f.rect());
                encre = Some(encre.map_or(f.rect(), |e| e.union(f.rect())));
            }
            self.glisser.base = union_rect(&rects)
                .zip(encre)
                .map(|(boite, encre)| BaseDuGlisser { boite, encre });
            self.glisser.exclus = exclude;
        }
    }

    /// Déplace les éléments sélectionnés avec magnétisme en évitant toute dérive de curseur.
    pub fn handle_item_drag_move(&mut self, screen_x: f64, screen_y: f64) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(screen_x, screen_y, &vp);
        let (raw_dx, raw_dy) = au_pixel_entier(
            (wx - self.glisser.depart.0, wy - self.glisser.depart.1),
            vp.scale,
        );

        let mut target_dx = raw_dx;
        let mut target_dy = raw_dy;

        if self.ui.smart_align {
            if let Some(base) = self.glisser.base.map(|b| b.boite) {
                let proposed = AlignRect {
                    left: base.left + raw_dx,
                    top: base.top + raw_dy,
                    width: base.width,
                    height: base.height,
                };
                // Les voisines de la sélection, parmi ce que l'écran montre (SNAP-4).
                let exclus = std::mem::take(&mut self.glisser.exclus);
                let lignes = self.lignes_d_aimant(proposed, &exclus);
                self.glisser.exclus = exclus;
                let snap = snap_move_sur(proposed, &lignes, self.options_d_aimant());
                target_dx += snap.dx;
                target_dy += snap.dy;
                self.active_guides = snap.guides;
            }
        } else {
            self.active_guides = SnapGuides::default();
        }

        let step_dx = target_dx - self.glisser.applique.0;
        let step_dy = target_dy - self.glisser.applique.1;
        self.glisser.applique = (target_dx, target_dy);

        // Chaque position devient son départ plus le déplacement **total** du geste : le geste
        // s'écrira en une seule translation, que le fichier rejoue au bit près (GLISSER-1).
        self.store
            .glisser_la_selection(&active_bid, (target_dx, target_dy));
        self.salir_le_deplacement((target_dx, target_dy), (step_dx, step_dy));
        let depot = self.dossier_sous(screen_x, screen_y);
        if depot != self.glisser.depot {
            self.glisser.depot = depot;
            self.mark_dirty();
        }
    }

    /// **RANGER-2 — le dossier où lâcher range la sélection** : sous le pointeur, celui qu'un
    /// clic y prendrait (le même arbitre), s'il n'est pas emporté par elle.
    fn dossier_sous(&self, screen_x: f64, screen_y: f64) -> Option<String> {
        let (wx, wy) = screen_to_world(screen_x, screen_y, &self.store.viewport());
        self.pick_candidates_at(wx, wy)
            .into_iter()
            .find(|c| {
                c.owner == PickOwner::Folder && !self.store.selected_folder_ids.contains(&c.id)
            })
            .map(|c| c.id)
    }

    /// La boîte du dossier où lâcher rangera ce qu'on glisse, en unités monde (RANGER-2).
    pub(crate) fn boite_du_depot(&self) -> Option<Rect> {
        let id = self.glisser.depot.as_deref()?;
        glucose_core::arrow::node_rect(self.store.active_board()?, id)
    }

    /// **RANGER-2 — ce que la main lâche sur un dossier y entre**, dans le même geste que le
    /// déplacement : un seul `Ctrl+Z` défait les deux. Rien à dire de plus : la boîte du dossier
    /// l'annonçait pendant le glisser, et son badge compte ce qui est entré (le cliquet des toasts).
    /// Refusé — ce que la sélection ne permet pas —, ce qu'on a lâché reste où la main l'a posé.
    fn ranger_la_selection(&mut self, board: &str, dossier: &str) {
        let s = &self.store;
        let choisis: Vec<String> = s
            .selected_annotation_ids
            .iter()
            .chain(s.selected_image_ids.iter())
            .chain(s.selected_folder_ids.iter())
            .cloned()
            .collect();
        let ids: Vec<&str> = choisis.iter().map(String::as_str).collect();
        let _ = self.store.ranger_dans_un_dossier(board, dossier, &ids);
    }

    /// Declare ce qu'un pas de deplacement a sali (A.1) -- le premier geste a le faire.
    ///
    /// # Ce qui change a l'ecran, et rien d'autre
    ///
    /// La selection occupait un rectangle, elle en occupe un autre : seule leur reunion a
    /// besoin d'etre redessinee. Sur trente-six photos, cela fait la difference entre
    /// repeindre l'ecran entier a chaque mouvement de la main et repeindre la carte qu'on
    /// tient.
    ///
    /// # Les deux cas ou l'on ne sait pas, et ou l'on redessine tout
    ///
    /// * **des guides d'alignement sont actifs** : ce sont des traits qui traversent l'ecran
    ///   de part en part, et ils apparaissent et disparaissent d'un pas a l'autre. Leur zone
    ///   est l'ecran ;
    /// * **la base du geste est inconnue** : sans elle, on ne sait pas d'ou la selection
    ///   vient. Le doute vaut `Tout`, c'est la regle de surete.
    fn salir_le_deplacement(&self, cumul: (f64, f64), pas: (f64, f64)) {
        // L'encre, et non la boîte : le grand titre d'une membrane bouge avec elle (TITRE-1).
        let Some(base) = self.glisser.base.map(|b| b.encre) else {
            self.mark_dirty();
            return;
        };
        if self.active_guides.x.is_some() || self.active_guides.y.is_some() {
            self.mark_dirty();
            return;
        }
        let rect = |dx: f64, dy: f64| Rect {
            left: base.left + dx,
            top: base.top + dy,
            width: base.width,
            height: base.height,
        };
        // La ou la selection etait avant ce pas, et la ou elle est maintenant.
        self.salir(rect(cumul.0 - pas.0, cumul.1 - pas.1));
        self.salir(rect(cumul.0, cumul.1));
    }

    /// **Le glisser s'abandonne** : les nœuds reviennent à leur place, et l'annulation n'en
    /// garde rien — ce que fait `Échap`, et ce que fait un glisser qui part vers une autre
    /// fenêtre (fiche 51 § 2).
    pub(crate) fn abandonner_le_glisser(&mut self) {
        if !self.is_dragging_item {
            return;
        }
        self.is_dragging_item = false;
        self.store.cancel_live_edit();
        self.glisser = Glisser::default();
        self.active_guides = SnapGuides::default();
        self.mark_dirty();
    }

    /// Termine la session de drag et nettoie les guides.
    pub fn finish_item_drag(&mut self) {
        if self.is_dragging_item {
            self.is_dragging_item = false;
            // Le dépôt : ce qu'on lâche change peut-être de membrane (MEMB-1) — dans le même
            // geste que le déplacement, qu'un seul `Ctrl+Z` défait.
            if self.glisser.applique != (0.0, 0.0) {
                let board = self.store.project.active_board_id.clone();
                self.store.rattacher_la_selection(&board);
                // Une image lâchée dans une case du storyboard s'y ajuste — dans le même geste,
                // après les pas du glisser : défaire rend l'ajustement, puis le déplacement.
                let images = self.store.selected_image_ids.to_vec();
                self.store.ajuster_dans_les_cases(&board, &images);
                if let Some(dossier) = self.glisser.depot.take() {
                    self.ranger_la_selection(&board, &dossier);
                }
            }
            self.store.end_live_edit();
            self.glisser = Glisser::default();
            self.active_guides = SnapGuides::default();
        }
    }
}

#[cfg(test)]
mod ranger_tests;
#[cfg(test)]
mod tests;
