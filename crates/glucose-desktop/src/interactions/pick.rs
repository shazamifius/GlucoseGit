//! Le clic sur le canevas : ce qu'il vise, et le geste qu'il commence.
//!
//! Dans l'ordre : le pan (espace tenu, ou l'outil main), un outil de création, une poignée
//! de redimensionnement, puis **l'arbitre de clic** (PICK-1, `hit_priority`) qui dit quel nœud
//! est sous le curseur — et selon qu'il s'agit d'un premier clic ou d'un double, un glisser
//! commence, ou le nœud s'ouvre.

use crate::animation::fly_into_folder;
use crate::app::{GlucoseApp, LastClickInfo};
use crate::canvas::screen_to_world;
use crate::interactions::chrome::screen_size;
use crate::params::ScreenFrame;
use crate::ui::ActiveTool;
use glucose_core::hit_priority::{
    advance_on_release, pick_at_down, PickCandidate, PickOptions, PickOwner,
};
use glucose_core::types::Annotation;

/// Au-delà de ce déplacement écran entre deux clics, ce n'est plus un double-clic.
pub(crate) const DOUBLE_CLICK_SLOP_PX: f64 = 8.0;

impl GlucoseApp {
    /// Le clic a traversé toutes les couches de l'interface : il est pour le canevas.
    pub fn click_canvas(&mut self, screen: ScreenFrame) {
        if self.ui.active_tool == ActiveTool::Pan {
            self.is_panning = true;
            return;
        }

        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        // Les poignées de coude passent avant le redimensionnement : elles appartiennent à
        // un objet déjà sélectionné, comme lui, et une flèche n'a pas de poignée de taille.
        if self.place_with_tool(wx, wy)
            || self.begin_arrow_bend(wx, wy)
            || self.begin_resize_at(wx, wy)
        {
            return;
        }
        self.click_node_at(wx, wy, screen);
    }

    /// L'arbitre de clic désigne le nœud sous le curseur ; un double-clic l'ouvre, un simple le
    /// sélectionne et commence un glisser. Dans le vide, la sélection élastique commence.
    fn click_node_at(&mut self, wx: f64, wy: f64, screen: ScreenFrame) {
        // L'index doit refléter le document AVANT qu'on l'interroge : sans cela, un nœud créé
        // au clic précédent serait introuvable jusqu'à la frame suivante.
        self.renderer.sync_spatial_index(&self.store);

        // `Ctrl`+clic sur un lien l'ouvre, avant tout le reste : c'est le seul geste qui
        // vise le **contenu** d'une carte sans vouloir la prendre.
        if self.click_link_at(self.mouse_pos) {
            return;
        }

        let Some(top) = self.pick_for_click(wx, wy) else {
            self.commit_editing();
            self.store.clear_selection();
            self.start_selection_box(self.mouse_pos.0, self.mouse_pos.1);
            return;
        };

        let count = self.click_count_on(&top);
        self.last_click = Some(LastClickInfo {
            at_ms: self.now_ms(),
            pos: self.mouse_pos,
            id: top.id.clone(),
            count,
        });

        // Une carte déjà ouverte à la saisie reçoit le clic **dans son texte** : c'est là que
        // le curseur se pose, qu'un mot se prend et qu'une sélection s'étend au `Maj`. Le nœud,
        // lui, ne se resélectionne pas et ne se met pas à glisser — on écrit dedans.
        if self
            .editing_session
            .as_ref()
            .is_some_and(|s| s.ann_id == top.id)
            && self.click_text_at(self.mouse_pos, count, self.modifiers.shift_key())
        {
            self.pick_cycle = None;
            return;
        }

        if count > 1 && self.open_node(&top, screen) {
            // La fiche 07 § 3.3 le dit : un double-clic réinitialise le cycle. On vient
            // d'ouvrir ce qu'on visait, on ne cherche plus ce qu'il y a dessous.
            self.pick_cycle = None;
            return;
        }

        // Cliquer ailleurs valide la saisie en cours.
        if self
            .editing_session
            .as_ref()
            .is_some_and(|s| s.ann_id != top.id)
        {
            self.commit_editing();
        }
        self.select_node(&top);
        self.init_item_drag(wx, wy);
    }

    /// Le nœud que ce clic désigne — **cycle de profondeur compris** (PICK-1, fiche 07 § 3.3).
    ///
    /// Le premier clic prend le sommet de la pile. Un re-clic au même endroit, moins de
    /// 2,5 s plus tard et après la fenêtre du double-clic, vise le même nœud : c'est au
    /// **relâchement** que le cycle descend d'un cran ([`Self::advance_pick_cycle`]), afin de
    /// ne jamais gêner un glisser.
    fn pick_for_click(&mut self, wx: f64, wy: f64) -> Option<PickCandidate> {
        let candidates = self.pick_candidates_at(wx, wy);
        let (picked, cycle) = pick_at_down(
            &candidates,
            self.pick_cycle.as_ref(),
            self.mouse_pos.0,
            self.mouse_pos.1,
            self.now_ms(),
            PickOptions {
                alt: self.modifiers.alt_key(),
                // Une sélection additive n'est pas une exploration de pile : on ajoute ce
                // qu'on voit, on ne cherche pas ce qu'il y a derrière.
                multi: self.modifiers.shift_key(),
            },
        );
        self.pick_cycle = cycle;
        picked
    }

    /// Le relâchement d'un clic qui n'a rien déplacé fait descendre le cycle d'un cran.
    ///
    /// Rien ne se passe au premier clic d'une série : `advance_on_release` ne descend que si
    /// le clic précédent avait armé la répétition. Le noyau s'arrête de lui-même sur un nœud
    /// `terminal` — un texte, une note éditable —, qui est le fond de la pile.
    pub fn advance_pick_cycle(&mut self) {
        if self.pick_cycle.is_none() {
            return;
        }
        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let candidates = self.pick_candidates_at(wx, wy);
        let (picked, cycle) =
            advance_on_release(&candidates, self.pick_cycle.as_ref(), self.now_ms());
        self.pick_cycle = cycle;
        if let Some(next) = picked {
            self.select_node(&next);
            self.mark_dirty();
        }
    }

    /// Les millisecondes écoulées depuis l'origine du temps des clics.
    pub fn now_ms(&self) -> i64 {
        i64::try_from(self.click_epoch.elapsed().as_millis()).unwrap_or(i64::MAX)
    }

    /// Le rang de ce clic dans une série rapprochée : 1 s'il ouvre la série, 2 pour un
    /// double-clic, 3 pour un triple.
    ///
    /// Même nœud, moins de `DBLCLICK_MS` depuis le précédent, et le curseur n'a presque pas
    /// bougé — les trois mêmes conditions qu'avant, mais **comptées** au lieu d'être réduites
    /// à un booléen : c'est ce qui permet au triple-clic d'exister.
    fn click_count_on(&self, top: &PickCandidate) -> u32 {
        self.click_count_at(&top.id)
    }

    /// Ce qu'un double-clic ouvre : un dossier, en y plongeant ; une annotation à texte, en
    /// édition. Rend `false` si le nœud n'a rien à ouvrir.
    fn open_node(&mut self, top: &PickCandidate, screen: ScreenFrame) -> bool {
        if top.owner == PickOwner::Folder {
            // La caméra plonge, et la bascule attend l'arrivée. Si le dossier a disparu
            // entre-temps, on entre sans cérémonie plutôt que de ne rien faire.
            let flown = fly_into_folder(
                &self.store,
                &mut self.animator,
                &top.id,
                screen_size(screen),
            );
            if !flown {
                drop(self.store.try_enter_folder(&top.id));
            }
            return true;
        }
        // Un nœud né d'un fichier mène à ce fichier — c'est ce que son lanceur promet, et
        // c'est plus utile que d'éditer un nom de fichier sur place.
        if self.reveal_launcher_target(&top.id) {
            return true;
        }
        let Some(text) = self.editable_text_of(&top.id) else {
            return false;
        };
        // Une **étiquette de flèche** s'ouvre le curseur à la fin, et non sur le mot visé :
        // viser un mot demande la mise en page d'une carte, et une flèche n'a pas de boîte.
        // Le geste reste juste : on double-clique une flèche pour écrire ce qu'elle dit,
        // presque toujours sur une étiquette encore vide.
        if top.owner == PickOwner::Arrow {
            self.start_text_edit(top.id.clone(), text);
            return true;
        }
        // Ouvrir une carte au double-clic sélectionne le mot visé : c'est ce que fait un
        // traitement de texte, et c'est ce qui permet de remplacer un mot d'un seul geste.
        let selection = self.selection_opening_at(&top.id, &text, self.mouse_pos);
        self.start_text_edit_at(top.id.clone(), text, selection);
        true
    }

    /// Le texte qu'une annotation offre à l'édition — `None` si l'identifiant est inconnu.
    pub(crate) fn editable_text_of(&self, id: &str) -> Option<String> {
        let board = self.store.active_board()?;
        let ann = board.annotations.iter().find(|a| a.id() == id)?;
        match ann {
            Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => Some(text.clone()),
            Annotation::Membrane { text, .. } | Annotation::Arrow { text, .. } => {
                Some(text.clone().unwrap_or_default())
            }
        }
    }

    pub(crate) fn select_node(&mut self, top: &PickCandidate) {
        let additive = self.modifiers.shift_key();
        // **SEL-MULTI-1 -- presser sur un element deja selectionne ne reduit pas la
        // selection.**
        //
        // `select_image(id, false)` commence par `clear_selection()`. Selectionner douze
        // images puis presser sur l'une d'elles pour les deplacer ramenait donc la selection
        // a UNE avant meme que le glissement ne demarre : `init_item_drag` n'en voyait plus
        // qu'une, et on ne pouvait deplacer les images qu'une par une.
        //
        // La reduction n'est pas supprimee, elle est **differee** : un clic simple sur un
        // membre de la selection doit toujours ramener la selection a lui seul, sinon on ne
        // pourrait plus en sortir sans passer par le vide. Les deux gestes commencent par la
        // meme pression et ne se distinguent qu'a la fin, donc c'est le relachement qui
        // tranche (voir `reduire_la_selection_si_demandee`).
        if !additive && self.est_deja_selectionne(top) {
            self.reduire_a_la_relache = Some(top.id.clone());
            return;
        }
        self.reduire_a_la_relache = None;
        match top.owner {
            PickOwner::Image => self.store.select_image(top.id.clone(), additive),
            PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                self.store.select_annotation(top.id.clone(), additive);
            }
            PickOwner::Folder => self.store.select_folder(top.id.clone()),
        }
    }

    /// Ce noeud fait-il deja partie de la selection ?
    fn est_deja_selectionne(&self, top: &PickCandidate) -> bool {
        match top.owner {
            PickOwner::Image => self.store.selected_image_ids.contains(&top.id),
            PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                self.store.selected_annotation_ids.contains(&top.id)
            }
            PickOwner::Folder => self.store.selected_folder_id.as_ref() == Some(&top.id),
        }
    }

    /// **Ramene la selection au seul element presse**, quand le geste n'etait qu'un clic.
    ///
    /// Appelee au relachement, et seulement si rien n'a bouge : un glissement garde la
    /// selection entiere, un clic la reduit. Le cycle de profondeur a deja eu sa chance juste
    /// avant, et s'il a designe un autre noeud il a efface cette demande en le selectionnant.
    pub(crate) fn reduire_la_selection_si_demandee(&mut self) {
        let Some(id) = self.reduire_a_la_relache.take() else {
            return;
        };
        if self.store.selected_image_ids.contains(&id) {
            self.store.clear_selection();
            self.store.select_image(id, false);
        } else if self.store.selected_annotation_ids.contains(&id) {
            self.store.clear_selection();
            self.store.select_annotation(id, false);
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
