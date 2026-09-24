//! Gestion du glisser-déplacer d'éléments avec alignement intelligent (SNAP-1).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::smart_align::{
    collect_align_targets, snap_move, union_rect, AlignRect, SnapGuides, SnapOptions,
};
use std::collections::HashSet;

impl GlucoseApp {
    /// Initialise une session de drag pour les éléments sélectionnés.
    pub fn init_item_drag(&mut self, wx: f64, wy: f64) {
        self.is_dragging_item = true;
        self.drag_start_world = (wx, wy);
        self.drag_applied_delta = (0.0, 0.0);
        self.store.begin_live_edit();

        // Préparer la boîte englobante et les cibles d'aimantation (SNAP-1). Ce qui bouge est
        // ce que la sélection emporte — le contenu de ses membranes compris (MEMB-1) : il se
        // redessine avec elle, et une membrane ne s'aimante pas sur ses propres membres.
        let emport = self
            .store
            .ce_qu_emporte_la_selection(&self.store.project.active_board_id);
        if let Some(board) = self.store.active_board() {
            let mut exclude: HashSet<String> = emport.ids().cloned().collect();
            let mut rects = emport.boites(board);
            if let Some(fid) = &self.store.selected_folder_id {
                exclude.insert(fid.clone());
                if let Some(f) = board.folders.iter().find(|f| &f.id == fid) {
                    rects.push(f.rect());
                }
            }
            self.drag_selection_base = union_rect(&rects);
            self.drag_snap_targets = collect_align_targets(board, &exclude);
        }
    }

    /// Déplace les éléments sélectionnés avec magnétisme en évitant toute dérive de curseur.
    pub fn handle_item_drag_move(&mut self, screen_x: f64, screen_y: f64) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(screen_x, screen_y, &vp);
        let raw_dx = wx - self.drag_start_world.0;
        let raw_dy = wy - self.drag_start_world.1;

        let mut target_dx = raw_dx;
        let mut target_dy = raw_dy;

        if self.ui.smart_align {
            if let Some(base) = self.drag_selection_base {
                let proposed = AlignRect {
                    left: base.left + raw_dx,
                    top: base.top + raw_dy,
                    width: base.width,
                    height: base.height,
                };
                let snap = snap_move(
                    proposed,
                    &self.drag_snap_targets,
                    SnapOptions {
                        scale: vp.scale,
                        ..Default::default()
                    },
                );
                target_dx += snap.dx;
                target_dy += snap.dy;
                self.active_guides = snap.guides;
            }
        } else {
            self.active_guides = SnapGuides::default();
        }

        let step_dx = target_dx - self.drag_applied_delta.0;
        let step_dy = target_dy - self.drag_applied_delta.1;
        self.drag_applied_delta = (target_dx, target_dy);

        if step_dx.abs() > 1e-7 || step_dy.abs() > 1e-7 {
            self.store.move_selected(&active_bid, step_dx, step_dy);
        }
        self.salir_le_deplacement((target_dx, target_dy), (step_dx, step_dy));
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
        let Some(base) = self.drag_selection_base else {
            self.mark_dirty();
            return;
        };
        if self.active_guides.x.is_some() || self.active_guides.y.is_some() {
            self.mark_dirty();
            return;
        }
        let rect = |dx: f64, dy: f64| glucose_core::geometry::Rect {
            left: base.left + dx,
            top: base.top + dy,
            width: base.width,
            height: base.height,
        };
        // La ou la selection etait avant ce pas, et la ou elle est maintenant.
        self.salir(rect(cumul.0 - pas.0, cumul.1 - pas.1));
        self.salir(rect(cumul.0, cumul.1));
    }

    /// Termine la session de drag et nettoie les guides.
    pub fn finish_item_drag(&mut self) {
        if self.is_dragging_item {
            self.is_dragging_item = false;
            // Le dépôt : ce qu'on lâche change peut-être de membrane (MEMB-1) — dans le même
            // geste que le déplacement, qu'un seul `Ctrl+Z` défait.
            if self.drag_applied_delta != (0.0, 0.0) {
                let board = self.store.project.active_board_id.clone();
                self.store.rattacher_la_selection(&board);
            }
            self.store.end_live_edit();
            self.active_guides = SnapGuides::default();
            self.drag_selection_base = None;
            self.drag_snap_targets.clear();
            self.drag_applied_delta = (0.0, 0.0);
        }
    }
}

#[cfg(test)]
mod tests;
