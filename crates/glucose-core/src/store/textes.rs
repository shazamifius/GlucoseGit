//! **Écrire le texte d'un nœud** — et faire suivre ce qui s'y rattache (FLECHE-4).

use super::Store;
use crate::text_anchors::{normalize_text_sel, suivre};
use crate::types::{Annotation, TextAnchor, TextSelection};

impl Store {
    /// **Écrit le texte d'un nœud, et fait suivre les ancres des flèches** qui y désignent un
    /// passage — dans le même geste : un `Ctrl+Z` rend le texte et les ancres ensemble.
    ///
    /// Une flèche ancrée au second « bonjours » d'une carte doit encore le désigner quand on
    /// écrit devant lui : ses ancres suivent l'édition comme un curseur suit sa place
    /// ([`crate::text_anchors::suivre`]). Une ancre dont le texte a été effacé disparaît ; la
    /// flèche garde alors son nœud, sans passage.
    ///
    /// Un texte vide retire le texte d'une membrane ou d'une flèche, qui n'en portent que s'il
    /// y a lieu ; une carte et un pense-bête gardent leur chaîne, vide ou non.
    pub fn ecrire_le_texte(&mut self, board_id: &str, id: &str, nouveau: &str) {
        let Some(ancien) = self
            .project
            .annotation(board_id, id)
            .map(|a| a.own_text().unwrap_or_default())
        else {
            return;
        };
        let vide = nouveau.trim().is_empty();
        self.update_annotation(board_id, id, |ann| match ann {
            Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => {
                *text = nouveau.to_string();
            }
            // Les quatre variantes sont couvertes, sans bras fourre-tout : une annotation d'un
            // nouveau genre fera échouer la compilation ici, au lieu de perdre ce qu'on y écrit.
            Annotation::Membrane { text, .. } | Annotation::Arrow { text, .. } => {
                *text = (!vide).then(|| nouveau.to_string());
            }
        });
        if ancien == nouveau {
            return;
        }
        for (fleche, cote_source) in self.fleches_ancrees_a(board_id, id) {
            self.update_annotation(board_id, &fleche, |ann| {
                let Annotation::Arrow {
                    source_text_sel,
                    target_text_sel,
                    ..
                } = ann
                else {
                    return;
                };
                let sel = if cote_source {
                    source_text_sel
                } else {
                    target_text_sel
                };
                let suivies = suivre(&normalize_text_sel(sel.as_ref()), &ancien, nouveau);
                *sel = (!suivies.is_empty()).then_some(TextSelection::Anchors(suivies));
            });
        }
    }

    /// **Pose les passages d'une flèche** — ceux de sa source et de sa cible, choisis dans
    /// l'éditeur d'ancres (FLECHE-4). Une liste vide retire le passage de ce côté : la flèche
    /// garde son nœud, et en part par son milieu.
    pub fn ancrer_la_fleche(
        &mut self,
        board_id: &str,
        fleche: &str,
        (source, cible): (Vec<TextAnchor>, Vec<TextAnchor>),
    ) {
        let passage = |ancres: Vec<TextAnchor>| {
            (!ancres.is_empty()).then_some(TextSelection::Anchors(ancres))
        };
        let (source, cible) = (passage(source), passage(cible));
        self.update_annotation(board_id, fleche, |ann| {
            if let Annotation::Arrow {
                source_text_sel,
                target_text_sel,
                ..
            } = ann
            {
                *source_text_sel = source;
                *target_text_sel = cible;
            }
        });
    }

    /// Les flèches qui désignent un passage du nœud `id`, et de quel côté.
    fn fleches_ancrees_a(&self, board_id: &str, id: &str) -> Vec<(String, bool)> {
        let Some(board) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        board
            .annotations
            .iter()
            .flat_map(|a| match a {
                Annotation::Arrow {
                    id: fleche,
                    source_id,
                    target_id,
                    source_text_sel,
                    target_text_sel,
                    ..
                } => [
                    (source_id.as_deref() == Some(id) && source_text_sel.is_some())
                        .then(|| (fleche.clone(), true)),
                    (target_id.as_deref() == Some(id) && target_text_sel.is_some())
                        .then(|| (fleche.clone(), false)),
                ],
                _ => [None, None],
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests;
