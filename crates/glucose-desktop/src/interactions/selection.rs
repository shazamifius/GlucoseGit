//! Gestion de la sélection d'éléments et de la boîte élastique (Marquee).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::hit_priority::{collect_candidates_indexed, PickCandidate, PickInput};
use glucose_core::types::Annotation;

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

    /// Applique la sélection élastique sur tous les éléments contenus ou intersectés.
    pub fn finish_selection_box(&mut self) {
        if let Some((x1, y1, x2, y2)) = self.selection_box.take() {
            let sx_min = x1.min(x2);
            let sx_max = x1.max(x2);
            let sy_min = y1.min(y2);
            let sy_max = y1.max(y2);

            if (sx_max - sx_min) > 3.0 || (sy_max - sy_min) > 3.0 {
                let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                let (wx1, wy1) = screen_to_world(sx_min, sy_min, &vp);
                let (wx2, wy2) = screen_to_world(sx_max, sy_max, &vp);
                let box_left = wx1.min(wx2);
                let box_right = wx1.max(wx2);
                let box_top = wy1.min(wy2);
                let box_bottom = wy1.max(wy2);

                let mut hits_imgs = Vec::new();
                let mut hits_anns = Vec::new();

                if let Some(b) = self.store.active_board() {
                    for img in &b.images {
                        let il = img.x - img.width / 2.0;
                        let ir = img.x + img.width / 2.0;
                        let it = img.y - img.height / 2.0;
                        let ib = img.y + img.height / 2.0;
                        if ir >= box_left && il <= box_right && ib >= box_top && it <= box_bottom {
                            hits_imgs.push(img.id.clone());
                        }
                    }
                    for ann in &b.annotations {
                        let (al, ar, at, ab) = match ann {
                            Annotation::Arrow { x, y, x2, y2, .. } => {
                                (x.min(*x2), x.max(*x2), y.min(*y2), y.max(*y2))
                            }
                            Annotation::Text { x, y, width, height, .. } => {
                                let w = width.unwrap_or(240.0);
                                let h = height.unwrap_or(48.0);
                                (*x, *x + w, *y, *y + h)
                            }
                            Annotation::Sticky { x, y, width, height, .. } => {
                                let w = width.unwrap_or(180.0);
                                let h = height.unwrap_or(130.0);
                                (*x, *x + w, *y, *y + h)
                            }
                            Annotation::Membrane { x, y, width, height, .. } => {
                                (*x, *x + *width, *y, *y + *height)
                            }
                        };
                        if ar >= box_left && al <= box_right && ab >= box_top && at <= box_bottom {
                            hits_anns.push(ann.id().to_string());
                        }
                    }
                }

                if !self.modifiers.shift_key() {
                    self.store.clear_selection();
                }
                for id in hits_imgs {
                    self.store.select_image(id, true);
                }
                for id in hits_anns {
                    self.store.select_annotation(id, true);
                }
            }
        }
    }

    /// Résout l'élément sous le clic à l'aide de hit_priority.
    #[allow(dead_code)]
    pub fn pick_candidate_at(&self, wx: f64, wy: f64) -> Option<PickCandidate> {
        let board = self.store.active_board()?;
        let vp = board.viewport;
        let input = PickInput {
            wx,
            wy,
            scale: vp.scale,
            images: &board.images,
            annotations: &board.annotations,
            folders: &board.folders,
            selected_image_ids: &self.store.selected_image_ids,
            selected_annotation_ids: &self.store.selected_annotation_ids,
            selected_folder_id: self.store.selected_folder_id.as_deref(),
            arrow_id: None,
            dom_hint: None,
        };
        let candidates = collect_candidates_indexed(&input, &self.renderer.spatial_hash);

        candidates.into_iter().next()
    }
}
