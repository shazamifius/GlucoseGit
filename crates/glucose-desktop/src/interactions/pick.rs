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
use glucose_core::hit_priority::{pick_consts, PickCandidate, PickOwner};
use glucose_core::types::Annotation;

/// Au-delà de ce déplacement écran entre deux clics, ce n'est plus un double-clic.
const DOUBLE_CLICK_SLOP_PX: f64 = 8.0;

impl GlucoseApp {
    /// Le clic a traversé toutes les couches de l'interface : il est pour le canevas.
    pub fn click_canvas(&mut self, screen: ScreenFrame) {
        if self.space_pressed || self.ui.active_tool == ActiveTool::Pan {
            self.is_panning = true;
            return;
        }

        let vp = self
            .store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or_default();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        if self.place_with_tool(wx, wy) || self.begin_resize_at(wx, wy) {
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

        let Some(top) = self.pick_candidate_at(wx, wy) else {
            self.commit_editing();
            self.store.clear_selection();
            self.start_selection_box(self.mouse_pos.0, self.mouse_pos.1);
            return;
        };

        let count = self.click_count_on(&top);
        self.last_click = Some(LastClickInfo {
            time: std::time::Instant::now(),
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
            return;
        }

        if count > 1 && self.open_node(&top, screen) {
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

    /// Le rang de ce clic dans une série rapprochée : 1 s'il ouvre la série, 2 pour un
    /// double-clic, 3 pour un triple.
    ///
    /// Même nœud, moins de `DBLCLICK_MS` depuis le précédent, et le curseur n'a presque pas
    /// bougé — les trois mêmes conditions qu'avant, mais **comptées** au lieu d'être réduites
    /// à un booléen : c'est ce qui permet au triple-clic d'exister.
    fn click_count_on(&self, top: &PickCandidate) -> u32 {
        let Some(last) = &self.last_click else {
            return 1;
        };
        let elapsed = i64::try_from(last.time.elapsed().as_millis()).unwrap_or(i64::MAX);
        let moved = (last.pos.0 - self.mouse_pos.0).hypot(last.pos.1 - self.mouse_pos.1);
        if last.id == top.id && elapsed < pick_consts::DBLCLICK_MS && moved < DOUBLE_CLICK_SLOP_PX {
            last.count + 1
        } else {
            1
        }
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
        let Some(text) = self.editable_text_of(&top.id) else {
            return false;
        };
        // Ouvrir une carte au double-clic sélectionne le mot visé : c'est ce que fait un
        // traitement de texte, et c'est ce qui permet de remplacer un mot d'un seul geste.
        let selection = self.selection_opening_at(&top.id, &text, self.mouse_pos);
        self.start_text_edit_at(top.id.clone(), text, selection);
        true
    }

    /// Le texte qu'une annotation offre à l'édition — `None` si elle n'en a pas (une flèche).
    fn editable_text_of(&self, id: &str) -> Option<String> {
        let board = self.store.active_board()?;
        let ann = board.annotations.iter().find(|a| a.id() == id)?;
        match ann {
            Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => Some(text.clone()),
            Annotation::Membrane { text, .. } => Some(text.clone().unwrap_or_default()),
            Annotation::Arrow { .. } => None,
        }
    }

    fn select_node(&mut self, top: &PickCandidate) {
        let additive = self.modifiers.shift_key();
        match top.owner {
            PickOwner::Image => self.store.select_image(top.id.clone(), additive),
            PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                self.store.select_annotation(top.id.clone(), additive);
            }
            PickOwner::Folder => self.store.select_folder(top.id.clone()),
        }
    }
}
