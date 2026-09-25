//! Gestion de la sélection d'éléments et de la boîte élastique (Marquee).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::geometry::Rect;
use glucose_core::hit_priority::{collect_candidates_indexed, PickCandidate, PickInput};

/// Déplacement écran au-delà duquel un clic maintenu sur le vide devient une sélection
/// élastique (fiche 07 § 7.3) : en deçà, c'est un clic, qui ne sélectionne rien.
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
    /// sélectionné — et pour une flèche, la boîte est l'enveloppe de son tracé.
    pub fn finish_selection_box(&mut self) {
        let Some((x1, y1, x2, y2)) = self.selection_box.take() else {
            return;
        };
        // En deçà du seuil sur les deux axes, c'est un clic dans le vide, pas un geste.
        if (x2 - x1).abs() <= RUBBERBAND_MIN_PX && (y2 - y1).abs() <= RUBBERBAND_MIN_PX {
            return;
        }
        let vp = self.store.viewport();
        let (left, top) = screen_to_world(x1.min(x2), y1.min(y2), &vp);
        let (right, bottom) = screen_to_world(x1.max(x2), y1.max(y2), &vp);
        let lasso = Rect::new(left, top, right - left, bottom - top);

        let Some(board) = self.store.active_board() else {
            return;
        };
        let images: Vec<String> = board
            .images
            .iter()
            .filter(|img| lasso.overlaps(img.rect()))
            .filter(|img| self.renderer.focus.laisse_voir(&img.id))
            .map(|img| img.id.clone())
            .collect();
        let annotations: Vec<String> = board
            .annotations
            .iter()
            .filter(|ann| lasso.overlaps(ann.bounds()))
            .filter(|ann| self.renderer.focus.laisse_voir(ann.id()))
            .map(|ann| ann.id().to_string())
            .collect();

        if !self.modifiers.shift_key() {
            self.store.clear_selection();
        }
        for id in images {
            self.store.select_image(id, true);
        }
        for id in annotations {
            self.store.select_annotation(id, true);
        }
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
        let vp = board.viewport;
        // Une flèche ancrée à un passage se vise là où le dessin la pose (FLECHE-4).
        let noeuds = crate::renderer::arrow::NoeudsDuRendu {
            board,
            index: Some(&self.renderer.spatial_hash),
            typographie: &self.renderer.typography,
            math: &self.renderer.math,
        };
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
            dom_hint: None,
            noeuds: Some(&noeuds),
        };
        let mut candidats = collect_candidates_indexed(&input, &self.renderer.spatial_hash);
        // En focus, ce qu'on ne voit pas ne s'attrape pas (MEMB-2).
        candidats.retain(|c| self.renderer.focus.laisse_voir(&c.id));
        candidats
    }
}
