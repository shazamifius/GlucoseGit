//! Mutations réversibles portant sur les images, et gestes portant sur la sélection entière
//! (déplacement, duplication, suppression).

use super::Store;
use crate::types::{Annotation, BoardImage};
use std::collections::HashSet;

/// Sélection figée au début d'un geste : évite de reconstruire les `HashSet` à chaque
/// événement `CursorMoved` dans les fonctions qui les consomment plusieurs fois (R-22).
struct SelectionSets {
    images: HashSet<String>,
    annotations: HashSet<String>,
    folder: Option<String>,
}

impl Store {
    fn selection_sets(&self) -> SelectionSets {
        SelectionSets {
            images: self.selected_image_ids.iter().cloned().collect(),
            annotations: self.selected_annotation_ids.iter().cloned().collect(),
            folder: self.selected_folder_id.clone(),
        }
    }

    pub fn add_image(&mut self, board_id: &str, img: BoardImage) {
        self.push_undo();
        let id = img.id.clone();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.images.push(img);
        }
        self.select_image(id, false);
    }

    pub fn update_image<F: FnOnce(&mut BoardImage)>(&mut self, board_id: &str, id: &str, f: F) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(img) = b.images.iter_mut().find(|i| i.id == id) {
                f(img);
            }
        }
    }

    /// Ferme l'ensemble des identifiants à supprimer par la cascade des miroirs :
    /// supprimer une image supprime toutes celles qui la reflètent, transitivement.
    fn close_image_mirror_cascade(&self, to_remove: &mut HashSet<String>) {
        let mut grew = true;
        while grew {
            grew = false;
            for b in &self.project.boards {
                for img in &b.images {
                    if let Some(ref m) = img.mirror_of {
                        if to_remove.contains(m) && !to_remove.contains(&img.id) {
                            to_remove.insert(img.id.clone());
                            grew = true;
                        }
                    }
                }
            }
        }
    }

    /// Retire les flèches dont une extrémité pointe vers un nœud supprimé.
    fn drop_orphan_arrows(annotations: &mut Vec<Annotation>, removed: &HashSet<String>) {
        annotations.retain(|a| match a {
            Annotation::Arrow { source_id, target_id, .. } => {
                let src_orphan = source_id.as_ref().is_some_and(|s| removed.contains(s));
                let tgt_orphan = target_id.as_ref().is_some_and(|t| removed.contains(t));
                !src_orphan && !tgt_orphan
            }
            _ => true,
        });
    }

    pub fn remove_images(&mut self, _board_id: &str, ids: &[&str]) {
        self.push_undo();
        let mut to_remove: HashSet<String> = ids.iter().map(|s| s.to_string()).collect();
        self.close_image_mirror_cascade(&mut to_remove);

        for b in &mut self.project.boards {
            b.images.retain(|img| !to_remove.contains(&img.id));
            Self::drop_orphan_arrows(&mut b.annotations, &to_remove);
        }
        self.selected_image_ids.clear();
    }

    pub fn move_selected(&mut self, board_id: &str, dx: f64, dy: f64) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.push_undo();
        let sel = self.selection_sets();

        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        for img in &mut b.images {
            if sel.images.contains(&img.id) && !img.locked {
                img.x += dx;
                img.y += dy;
            }
        }
        for ann in &mut b.annotations {
            if sel.annotations.contains(ann.id()) {
                translate_annotation(ann, dx, dy);
            }
        }
        if let Some(fid) = sel.folder.as_deref() {
            for f in &mut b.folders {
                if f.id == fid {
                    f.x += dx;
                    f.y += dy;
                }
            }
        }
        drag_connected_arrows(&mut b.annotations, &sel, dx, dy);
    }

    pub fn duplicate_selected(&mut self, board_id: &str) {
        if self.selected_image_ids.is_empty() && self.selected_annotation_ids.is_empty() {
            return;
        }
        self.push_undo();

        let (mut new_imgs, mut new_anns) = self.clone_selection(board_id);
        const OFFSET: f64 = 20.0;

        for img in &mut new_imgs {
            img.id = self.generate_id("img");
            img.x += OFFSET;
            img.y += OFFSET;
        }
        for ann in &mut new_anns {
            let prefix = if matches!(ann, Annotation::Arrow { .. }) { "arrow" } else { "ann" };
            let fresh = self.generate_id(prefix);
            set_annotation_id(ann, fresh);
            translate_annotation(ann, OFFSET, OFFSET);
        }

        self.selected_image_ids = new_imgs.iter().map(|i| i.id.clone()).collect();
        self.selected_annotation_ids = new_anns.iter().map(|a| a.id().to_string()).collect();

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.images.extend(new_imgs);
            b.annotations.extend(new_anns);
        }
    }

    /// Copies brutes des éléments sélectionnés, avant réattribution d'identifiants.
    fn clone_selection(&self, board_id: &str) -> (Vec<BoardImage>, Vec<Annotation>) {
        let mut imgs = Vec::new();
        let mut anns = Vec::new();
        if let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) {
            for id in &self.selected_image_ids {
                if let Some(img) = b.images.iter().find(|i| &i.id == id) {
                    imgs.push(img.clone());
                }
            }
            for id in &self.selected_annotation_ids {
                if let Some(ann) = b.annotations.iter().find(|a| a.id() == id) {
                    anns.push(ann.clone());
                }
            }
        }
        (imgs, anns)
    }

    pub fn delete_selected(&mut self, board_id: &str) {
        let imgs = self.selected_image_ids.clone();
        let anns = self.selected_annotation_ids.clone();
        let f_opt = self.selected_folder_id.clone();

        if !imgs.is_empty() {
            let refs: Vec<&str> = imgs.iter().map(|s| s.as_str()).collect();
            self.remove_images(board_id, &refs);
        }
        if !anns.is_empty() {
            let refs: Vec<&str> = anns.iter().map(|s| s.as_str()).collect();
            self.remove_annotations(board_id, &refs);
        }
        if let Some(ref fid) = f_opt {
            self.remove_folders(board_id, &[fid]);
        }
    }
}

/// Translation d'une annotation, extrémités et waypoints compris.
pub(super) fn translate_annotation(ann: &mut Annotation, dx: f64, dy: f64) {
    match ann {
        Annotation::Text { x, y, .. }
        | Annotation::Sticky { x, y, .. }
        | Annotation::Membrane { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Annotation::Arrow { x, y, x2, y2, waypoints, .. } => {
            *x += dx;
            *y += dy;
            *x2 += dx;
            *y2 += dy;
            for wp in waypoints {
                wp.x += dx;
                wp.y += dy;
            }
        }
    }
}

fn set_annotation_id(ann: &mut Annotation, new_id: String) {
    match ann {
        Annotation::Text { id, .. }
        | Annotation::Sticky { id, .. }
        | Annotation::Membrane { id, .. }
        | Annotation::Arrow { id, .. } => *id = new_id,
    }
}

/// R-12 — une flèche non sélectionnée suit l'extrémité dont le nœud, lui, bouge.
fn drag_connected_arrows(annotations: &mut [Annotation], sel: &SelectionSets, dx: f64, dy: f64) {
    for ann in annotations.iter_mut() {
        if sel.annotations.contains(ann.id()) {
            continue;
        }
        let Annotation::Arrow { source_id, target_id, x, y, x2, y2, .. } = ann else {
            continue;
        };
        let moved = |id: &Option<String>| {
            id.as_ref().is_some_and(|s| sel.annotations.contains(s) || sel.images.contains(s))
        };
        if moved(source_id) {
            *x += dx;
            *y += dy;
        }
        if moved(target_id) {
            *x2 += dx;
            *y2 += dy;
        }
    }
}
