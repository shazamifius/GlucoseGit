//! **LIEN-2 — suivre un lien du canevas : une flèche, jusqu'à son autre bout** (fiche 102).
//!
//! Sa parole du 9/11 : *« un clic sur un lien : un focus sur ce à quoi il est rattaché »*. Le clic
//! seul choisit déjà une flèche — pour la plier, la colorer, la supprimer (ARROW-1 à 3) — : le
//! voler ôterait l'édition. **`Ctrl`+clic** est le geste qui, dans Glucose, suit déjà un lien (une
//! adresse dans un texte, [`super::links`]) : sur une flèche, il suit le lien. Au doigt, sans `Ctrl`,
//! « Suivre le lien » au menu d'une flèche choisie ; à la palette, `lien.suivre`.
//!
//! # Vers quel bout
//!
//! **Celui où l'on n'est pas** : le plus loin du clic — ou, depuis le menu et la palette, du centre
//! de l'écran. On lit un texte, on suit l'un de ses liens, on arrive à ce qu'il désigne. La vue y
//! vole à sa taille de lecture (`F`), et le vol note d'où il part : `Alt+Z` ramène (RETOUR-1). C'est
//! le « Go » de Bring & Go et le bout de Link Sliding (Moscovich et al., CHI 2009) : la vue voyage,
//! la carte ne bouge pas. Une flèche qui ne touche aucun nœud se cadre elle-même.

use crate::app::GlucoseApp;
use glucose_core::geometry::Rect;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **`Ctrl`+clic sur une flèche la suit** ; rend vrai si le clic a été pris.
    pub(crate) fn suivre_la_fleche_cliquee(&mut self, (wx, wy): (f64, f64)) -> bool {
        if !self.modifiers.control_key() {
            return false;
        }
        // Ce que le clic viserait : un nœud qui n'est pas une flèche ne se suit pas, et le clic
        // reste au choix.
        let Some(vise) = self.pick_candidate_at(wx, wy) else {
            return false;
        };
        self.suivre_le_lien(&vise.id, (wx, wy))
    }

    /// **La commande « Suivre le lien »** : la flèche choisie, seule, depuis le centre de l'écran.
    /// Rend faux si la sélection n'est pas une flèche seule — l'accord passe à la suivante.
    pub(crate) fn suivre_le_lien_choisi(&mut self) -> bool {
        let fleches = self.store.selected_arrows();
        if fleches.len() != 1
            || self.store.selected_annotation_ids.len() != 1
            || !self.store.selected_image_ids.is_empty()
        {
            return false;
        }
        let (l, h) = self.taille_de_la_fenetre();
        let vp = self.store.viewport();
        let centre = crate::canvas::screen_to_world(f64::from(l) / 2.0, f64::from(h) / 2.0, &vp);
        let id = fleches[0].id().to_string();
        self.suivre_le_lien(&id, centre)
    }

    /// Vole vers le bout de cette flèche le plus loin de `depuis` (un point du monde).
    fn suivre_le_lien(&mut self, fleche: &str, depuis: (f64, f64)) -> bool {
        let Some(board) = self.store.active_board() else {
            return false;
        };
        let Some(Annotation::Arrow {
            source_id,
            target_id,
            ..
        }) = board.annotations.iter().find(|a| a.id() == fleche)
        else {
            return false;
        };
        let loin = |boite: &Rect| {
            let (cx, cy) = (
                boite.left + boite.width / 2.0,
                boite.top + boite.height / 2.0,
            );
            (cx - depuis.0).hypot(cy - depuis.1)
        };
        let bout = [source_id, target_id]
            .into_iter()
            .flatten()
            .filter_map(|id| Some((id.clone(), glucose_core::arrow::emprise(board, id)?)))
            .max_by(|a, b| loin(&a.1).total_cmp(&loin(&b.1)))
            .map(|(id, _)| id);
        let vise = bout.unwrap_or_else(|| fleche.to_string());
        self.voler_vers_le_noeud(&vise)
    }
}

#[cfg(test)]
#[path = "suivre/tests.rs"]
mod tests;
