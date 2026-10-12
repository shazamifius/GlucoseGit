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

    /// **Combien de nœuds la sélection compte**, dossier compris. Le menu, la barre d'action
    /// et le clavier lisent ce compte-ci : chacun comptait les images et les annotations, et
    /// un dossier choisi n'avait ni « Supprimer » ni barre — au téléphone, sans touche
    /// `Suppr`, il ne s'effaçait pas (DOSSIER-1, fiche 60).
    pub fn compte_de_la_selection(&self) -> usize {
        self.selected_image_ids.len()
            + self.selected_annotation_ids.len()
            + usize::from(self.selected_folder_id.is_some())
    }
}

impl Store {
    /// Ce que la sélection courante vaut comme **texte**, ou `None` si elle n'en porte aucun.
    ///
    /// Vit dans le noyau et non dans l'interface, parce que c'est une question sur le
    /// document : savoir où une annotation range son texte n'est pas l'affaire de celui qui
    /// dessine (règle S, fiche 12 § 3).
    ///
    /// C'est ce qu'un **autre logiciel** colle quand on copie des nœuds : leurs textes. Une image
    /// rendait le chemin de son fichier, puis la clé qui la nomme dans le document — un nom que
    /// personne d'autre ne sait lire. Les images voyagent désormais dans le lot (fiche 51 § 2).
    pub fn selection_as_text(&self) -> Option<String> {
        let board = self.active_board()?;
        let textes: Vec<String> = self
            .selected_annotation_ids
            .iter()
            .filter_map(|id| board.annotations.iter().find(|a| a.id() == id))
            .filter_map(Annotation::own_text)
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

impl Store {
    /// Les flèches que la sélection retient, dans l'ordre du document.
    ///
    /// Une flèche sélectionnée est la seule à montrer ses poignées de coude (ARROW-3) : la
    /// question appartient donc à la sélection, et l'interface n'a pas à parcourir les
    /// annotations pour la poser (règle S).
    pub fn selected_arrows(&self) -> Vec<&crate::types::Annotation> {
        let Some(board) = self.active_board() else {
            return Vec::new();
        };
        board
            .annotations
            .iter()
            .filter(|a| matches!(a, crate::types::Annotation::Arrow { .. }))
            .filter(|a| self.selected_annotation_ids.iter().any(|s| s == a.id()))
            .collect()
    }

    /// **Une copie du document pour un export** (EXPORT-PNG-1) : sans histoire ni sélection,
    /// et, si `la_selection`, réduite sur le tableau actif à ce qui est choisi — images,
    /// annotations, dossier. Elle prend le document en lecture : celui qu'on édite n'est jamais
    /// touché.
    pub fn copie_a_exporter(&self, la_selection: bool) -> Store {
        let mut copie = Store::new(self.project.name.clone());
        copie.project = self.project.clone();
        copie.assets = self.assets.clone();
        if la_selection {
            let choisie = |ids: &[String], id: &str| ids.iter().any(|s| s == id);
            if let Some(b) = copie.active_board_mut() {
                b.images
                    .retain(|i| choisie(&self.selected_image_ids, &i.id));
                b.annotations
                    .retain(|a| choisie(&self.selected_annotation_ids, a.id()));
                b.folders
                    .retain(|f| self.selected_folder_id.as_deref() == Some(f.id.as_str()));
            }
        }
        copie
    }

    /// Le tableau actif porte-t-il au moins une annotation ?
    pub fn a_des_annotations(&self) -> bool {
        self.active_board()
            .is_some_and(|b| !b.annotations.is_empty())
    }
}
