//! Gestion du glisser-déplacer d'éléments avec alignement intelligent (SNAP-1).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::smart_align::{
    collect_align_targets, rect_of_annotation, rect_of_folder, rect_of_image, snap_move,
    union_rect, AlignRect, SnapGuides, SnapOptions,
};
use std::collections::HashSet;

impl GlucoseApp {
    /// Initialise une session de drag pour les éléments sélectionnés.
    pub fn init_item_drag(&mut self, wx: f64, wy: f64) {
        self.is_dragging_item = true;
        self.drag_start_world = (wx, wy);
        self.drag_applied_delta = (0.0, 0.0);
        self.store.begin_live_edit();

        // Préparer la boîte englobante et les cibles d'aimantation (SNAP-1)
        if let Some(board) = self.store.active_board() {
            let mut exclude = HashSet::new();
            let mut rects = Vec::new();
            for id in &self.store.selected_image_ids {
                exclude.insert(id.clone());
                if let Some(img) = board.images.iter().find(|i| &i.id == id) {
                    rects.push(rect_of_image(img));
                }
            }
            for id in &self.store.selected_annotation_ids {
                exclude.insert(id.clone());
                if let Some(ann) = board.annotations.iter().find(|a| a.id() == id) {
                    if let Some(r) = rect_of_annotation(ann) {
                        rects.push(r);
                    }
                }
            }
            if let Some(fid) = &self.store.selected_folder_id {
                exclude.insert(fid.clone());
                if let Some(f) = board.folders.iter().find(|f| &f.id == fid) {
                    rects.push(rect_of_folder(f));
                }
            }
            self.drag_selection_base = union_rect(&rects);
            self.drag_snap_targets = collect_align_targets(board, &exclude);
        }
    }

    /// Déplace les éléments sélectionnés avec magnétisme en évitant toute dérive de curseur.
    pub fn handle_item_drag_move(&mut self, screen_x: f64, screen_y: f64) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
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
        self.mark_dirty();
    }

    /// Termine la session de drag et nettoie les guides.
    pub fn finish_item_drag(&mut self) {
        if self.is_dragging_item {
            self.is_dragging_item = false;
            self.store.end_live_edit();
            self.active_guides = SnapGuides::default();
            self.drag_selection_base = None;
            self.drag_snap_targets.clear();
            self.drag_applied_delta = (0.0, 0.0);
        }
    }
}
