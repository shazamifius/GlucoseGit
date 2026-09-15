//! Sélection courante (images, annotations, dossier). Aucune entrée d'undo (UNDO-1).

use super::Store;
use crate::types::Annotation;

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

impl Store {
    /// Ce que la sélection courante vaut comme **texte**, ou `None` si elle n'en porte aucun.
    ///
    /// Vit dans le noyau et non dans l'interface, parce que c'est une question sur le
    /// document : savoir où une annotation range son texte n'est pas l'affaire de celui qui
    /// dessine (règle S, fiche 12 § 3).
    ///
    /// Une image rend le **chemin de son fichier** plutôt que rien : c'est précisément ce que
    /// le collage sait relire pour la réimporter. Copier produit ainsi exactement ce que
    /// coller attend — les deux gestes se referment l'un sur l'autre au lieu de se croiser.
    pub fn selection_as_text(&self) -> Option<String> {
        let board = self.active_board()?;
        let textes: Vec<String> = self
            .selected_annotation_ids
            .iter()
            .filter_map(|id| board.annotations.iter().find(|a| a.id() == id))
            .filter_map(Annotation::own_text)
            .chain(
                self.selected_image_ids
                    .iter()
                    .filter_map(|id| board.images.iter().find(|i| &i.id == id))
                    .filter_map(|img| img.src.clone()),
            )
            .filter(|t| !t.trim().is_empty())
            .collect();
        if textes.is_empty() {
            return None;
        }
        // Une ligne vide entre deux nœuds : c'est le séparateur de paragraphe du Markdown,
        // donc celui que le collage relira comme deux blocs et non comme une phrase coupée.
        Some(textes.join("\n\n"))
    }
}
