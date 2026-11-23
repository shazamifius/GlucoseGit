//! **OPACITE-1 — l'opacité d'un dossier, dans la roue** (sa note du 23/11 : *« les dossiers sont
//! très transparents par défaut ; ce serait bien, avec le système de couleur, d'avoir une option
//! pour changer son opacité »* ; fiche 133). Le curseur se glisse comme celui de la clarté (`Maj`
//! affine) ; la molette y change l'opacité d'une différence à peine visible par cran.

use crate::app::GlucoseApp;
use glucose_core::couleur::roue::hex;
use glucose_core::couleur::DIFFERENCE_A_PEINE_VISIBLE;

impl GlucoseApp {
    /// **L'opacité désignée, de 0 à 1, devient celle des dossiers choisis** — dans le geste que la
    /// roue tient ouvert.
    pub(super) fn poser_l_opacite(&mut self, part: f64) {
        // Le curseur n'existe qu'avec un dossier choisi (`placer`) : aucune garde à tenir ici.
        let Some(roue) = self.ui.roue.as_mut() else {
            return;
        };
        roue.opacite = Some((part.clamp(0.0, 1.0) * 255.0).round() as u8);
        roue.opacite_touchee = true;
        self.reposer();
    }

    /// **La molette sur le curseur de l'opacité** : une différence à peine visible par cran. Le
    /// corps posé sur le fond change de clarté de Δα fois l'écart entre sa teinte et le fond —
    /// jamais plus que Δα : un cran ne saute jamais une différence visible.
    pub(super) fn molette_sur_l_opacite(&mut self, crans: f64) {
        let Some(actuelle) = self.ui.roue.as_ref().and_then(|r| r.opacite) else {
            return;
        };
        let part = f64::from(actuelle) / 255.0 + crans * DIFFERENCE_A_PEINE_VISIBLE;
        self.poser_l_opacite(part);
    }

    /// **Ce que la roue a touché, reposé sur les couleurs d'avant elle** : un seul geste, une
    /// écriture par nœud, quelle que soit la longueur du glisser. La couleur et l'opacité se posent
    /// chacune si la main l'a touchée : régler l'opacité ne colore pas les textes choisis avec
    /// les dossiers, et choisir une couleur garde l'opacité qu'on vient de régler.
    pub(super) fn reposer(&mut self) {
        let Some(roue) = self.ui.roue.as_ref() else {
            return;
        };
        let couleur = roue.couleur_touchee.then(|| roue.apres.map(hex));
        let opacite = roue.opacite.filter(|_| roue.opacite_touchee);
        self.store.cancel_live_edit();
        self.store.begin_live_edit();
        if let Some(couleur) = couleur {
            self.store.colorer_la_selection(couleur.as_deref());
        }
        if let Some(opacite) = opacite {
            self.store.opacifier_la_selection(opacite);
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
