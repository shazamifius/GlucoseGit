//! Sélection courante (images, annotations, dossier). Aucune entrée d'undo (UNDO-1).

use super::Store;

impl Store {
    pub fn clear_selection(&mut self) {
        self.selected_image_ids.clear();
        self.selected_annotation_ids.clear();
        self.selected_folder_id = None;
    }

    pub fn set_selected_image_ids(&mut self, ids: Vec<String>) {
        self.selected_image_ids = ids;
    }

    pub fn set_selected_annotation_ids(&mut self, ids: Vec<String>) {
        self.selected_annotation_ids = ids;
    }

    pub fn select_image(&mut self, id: String, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        if !self.selected_image_ids.contains(&id) {
            self.selected_image_ids.push(id);
        }
    }

    pub fn select_annotation(&mut self, id: String, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        if !self.selected_annotation_ids.contains(&id) {
            self.selected_annotation_ids.push(id);
        }
    }

    pub fn select_folder(&mut self, id: String) {
        self.clear_selection();
        self.selected_folder_id = Some(id);
    }
}
