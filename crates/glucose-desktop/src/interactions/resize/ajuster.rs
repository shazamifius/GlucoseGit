//! **TEXT-FIT-1 et TEXT-FIT-2 à l'ouverture d'un document** : chaque carte de texte prend la
//! hauteur de son texte, et celle qui grandit pousse sa membrane. Une normalisation de chargement,
//! hors du journal — sorti de `resize.rs`, qui passait les 600 lignes (fiche 124).

use crate::app::GlucoseApp;
use crate::renderer::card::text_card_fit_height;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// TEXT-FIT-1 à l'ouverture d'un document : chaque carte de texte reçoit la hauteur de
    /// son texte. C'est une normalisation de chargement, pas une action de l'utilisateur —
    /// elle n'entre pas dans la pile d'undo, que `load_project` vient de vider.
    pub fn fit_all_text_cards(&mut self) {
        let (typography, math) = (&self.renderer.typography, &self.renderer.math);
        ajuster_les_cartes(&mut self.store.project.boards, typography, math);
    }
}

/// **Mesure chaque carte de texte de ces tableaux** : sa hauteur écrite est un plancher, la
/// mesure l'élève si le texte en demande plus — et une carte qui grandit pousse sa membrane
/// (TEXT-FIT-2). Hors journal : c'est ce que fait un document qu'on lit, avant qu'il ne devienne
/// un geste (BOARDS-2) ou le document courant.
pub(crate) fn ajuster_les_cartes(
    boards: &mut [glucose_core::types::Board],
    typography: &crate::typography::Typography,
    math: &crate::renderer::math::MathRenderer,
) {
    use glucose_core::store::pousser::pousser_dans_le_tableau;
    for board in boards {
        let mut grandies = Vec::new();
        for ann in &mut board.annotations {
            let Annotation::Text {
                id,
                y,
                width,
                height,
                text,
                ..
            } = ann
            else {
                continue;
            };
            let w = width.unwrap_or(glucose_core::types::DEFAULT_TEXT_CARD_WIDTH);
            *width = Some(w);
            // Au chargement aussi, la hauteur écrite dans le fichier est un plancher :
            // écraser détruirait à l'ouverture ce que l'utilisateur avait tiré en fermant.
            let fit = text_card_fit_height(typography, math, text, w);
            let ecrite = *height;
            *height = Some(ecrite.unwrap_or(0.0).max(fit));
            // Une hauteur écrite que le texte dépasse : une mise en page neuve l'a fait grandir.
            if ecrite.is_some_and(|h| fit > h) {
                grandies.push((id.clone(), *y + fit));
            }
        }
        for (id, bas) in grandies {
            pousser_dans_le_tableau(board, &id, bas);
        }
    }
}

#[cfg(test)]
mod tests;
