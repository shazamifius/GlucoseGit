//! Mutations réversibles portant sur les annotations, leurs miroirs et les panneaux storyboard.

use super::images::translate_annotation;
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, StoryboardPanel};
use std::collections::HashSet;

impl Store {
    pub fn sync_annotation_size(&mut self, board_id: &str, ann_id: &str, width: f64, height: f64) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == ann_id) else {
            return;
        };
        match ann {
            Annotation::Text { width: w, height: h, .. }
            | Annotation::Sticky { width: w, height: h, .. } => {
                *w = Some(width);
                *h = Some(height);
            }
            Annotation::Membrane { width: w, height: h, .. } => {
                *w = width;
                *h = height;
            }
            Annotation::Arrow { .. } => {}
        }
    }

    pub fn add_annotation(&mut self, board_id: &str, ann: Annotation) {
        self.push_undo();
        let id = ann.id().to_string();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.annotations.push(ann);
        }
        self.select_annotation(id, false);
    }

    pub fn update_annotation<F: FnOnce(&mut Annotation)>(&mut self, board_id: &str, id: &str, f: F) {
        self.push_undo();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == id) else {
            return;
        };
        f(ann);

        // Suivi des flèches connectées si la source a bougé (R-12).
        let Some(moved) = b.annotations.iter().find(|a| a.id() == id) else {
            return;
        };
        let (nx, ny) = (moved.x(), moved.y());
        for a in &mut b.annotations {
            if let Annotation::Arrow { source_id, target_id, x, y, x2, y2, .. } = a {
                if source_id.as_deref() == Some(id) {
                    *x = nx;
                    *y = ny;
                }
                if target_id.as_deref() == Some(id) {
                    *x2 = nx;
                    *y2 = ny;
                }
            }
        }
    }

    pub fn remove_annotations(&mut self, _board_id: &str, ids: &[&str]) {
        self.push_undo();
        let id_set: HashSet<&str> = ids.iter().copied().collect();

        for b in &mut self.project.boards {
            b.annotations.retain(|a| !id_set.contains(a.id()));
            b.annotations.retain(|a| match a {
                Annotation::Arrow { source_id, target_id, .. } => {
                    let src_orphan = source_id.as_ref().is_some_and(|s| id_set.contains(s.as_str()));
                    let tgt_orphan = target_id.as_ref().is_some_and(|t| id_set.contains(t.as_str()));
                    !src_orphan && !tgt_orphan
                }
                _ => true,
            });
        }
        self.selected_annotation_ids.clear();
    }

    /// Crée un miroir d'une annotation, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_mirror_annotation(
        &mut self,
        board_id: &str,
        id: &str,
        x: f64,
        y: f64,
    ) -> CoreResult<String> {
        self.push_undo();
        let source = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .ok_or_else(|| CoreError::BoardNotFound(board_id.to_string()))?
            .annotations
            .iter()
            .find(|a| a.id() == id)
            .ok_or_else(|| CoreError::AnnotationNotFound(id.to_string()))?
            .clone();

        let mid = self.generate_id("mirror");
        let mirror = build_mirror(source, &mid, id, x, y);

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.annotations.push(mirror);
        }
        Ok(mid)
    }

    /// Enveloppe silencieuse de [`Store::try_mirror_annotation`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_mirror_annotation`.")]
    pub fn mirror_annotation(&mut self, board_id: &str, id: &str, x: f64, y: f64) -> Option<String> {
        self.try_mirror_annotation(board_id, id, x, y).ok()
    }

    // ── Storyboard Panels ───────────────────────────────────────────────────

    pub fn add_panel(&mut self, board_id: &str, panel: StoryboardPanel) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.panels.push(panel);
        }
    }

    pub fn update_panel(&mut self, board_id: &str, id: &str, description: impl Into<String>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(p) = b.panels.iter_mut().find(|p| p.id == id) {
                p.description = description.into();
            }
        }
    }

    pub fn remove_panel(&mut self, board_id: &str, id: &str) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.panels.retain(|p| p.id != id);
        }
    }
}

/// Copie d'une annotation déposée en (x, y) et reliée à sa source par `mirror_of`.
///
/// Une flèche n'a pas de `mirror_of` : elle est translatée en bloc pour conserver sa forme.
fn build_mirror(source: Annotation, mirror_id: &str, origin_id: &str, x: f64, y: f64) -> Annotation {
    let mut m = source;
    let mut arrow_delta = None;
    match &mut m {
        Annotation::Text { id, x: ax, y: ay, mirror_of, .. }
        | Annotation::Sticky { id, x: ax, y: ay, mirror_of, .. }
        | Annotation::Membrane { id, x: ax, y: ay, mirror_of, .. } => {
            *id = mirror_id.to_string();
            *ax = x;
            *ay = y;
            *mirror_of = Some(origin_id.to_string());
        }
        Annotation::Arrow { id, x: ax, y: ay, .. } => {
            *id = mirror_id.to_string();
            arrow_delta = Some((x - *ax, y - *ay));
        }
    }
    if let Some((dx, dy)) = arrow_delta {
        translate_annotation(&mut m, dx, dy);
    }
    m
}
