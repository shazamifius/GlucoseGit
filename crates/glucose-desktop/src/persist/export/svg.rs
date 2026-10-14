//! **L'export en image vectorielle** (P5, fiche 69) : ce qu'on exporte — la sélection, sinon le
//! tableau, dans le cadre même de l'export en image (EXPORT-PNG-1) —, écrit en SVG par les
//! sœurs vectorielles des dessins ([`crate::renderer::svg`]). Les images portent leurs octets
//! d'origine : le fichier se suffit.

use super::png::{moteur, preparer};
use crate::app::GlucoseApp;
use crate::ui::UiState;
use glucose_core::types::BoardImage;

impl GlucoseApp {
    /// **Le SVG de ce qu'on exporte** — `None` pour un tableau vide.
    pub fn rendre_le_svg(&self) -> Option<String> {
        let scene = preparer(&self.store)?;
        let mut moteur = moteur(self.peintre_de_l_export());
        let mut ui = UiState::new();
        let objets = &self.disque.objets;
        let mut octets = |img: &BoardImage| {
            img.src
                .as_deref()
                .and_then(|cle| objets.lire_en_attendant(cle))
        };
        let taille = (scene.largeur, scene.hauteur);
        let quoi = (&scene.store, &mut ui);
        Some(moteur.rendre_en_svg(quoi, scene.vue, taille, &mut octets))
    }
}

#[cfg(test)]
mod tests;
