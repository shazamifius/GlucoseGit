//! **MEMB-2 — le mode Focus**, du côté de l'application : « zoomer assez sur une membrane et
//! n'avoir plus qu'elle ».
//!
//! Toute la décision vit dans le noyau ([`glucose_core::membrane_focus`]) : on entre quand une
//! membrane couvre presque tout l'écran et en contient le centre, on sort en dézoomant ou en
//! s'éloignant, et un temps mort empêche d'osciller. Il ne reste ici qu'à la demander quand la
//! vue a bougé, puis à l'appliquer : caler la caméra sur la membrane — par le vol de `F`, une
//! seule géométrie de vol dans tout le programme — et dire au rendu ce qui se voit.
//!
//! Le noyau raisonne sur un écran qui commence en haut du canevas. Le bandeau du haut le
//! recouvre ici : la vue et le cadrage se décalent de sa hauteur, à l'aller et au retour.

use crate::app::GlucoseApp;
use glucose_core::membrane_focus::{
    decider, CarteDesMembranes, FocusAction, FocusState, ScreenSize,
};
use glucose_core::types::Viewport;

/// Le mode Focus : son état, et ce qui a servi à le décider la dernière fois.
#[derive(Debug, Default)]
pub struct Focus {
    pub etat: FocusState,
    /// Le tableau, la vue, la version du document et la taille de l'écran de la dernière
    /// décision : tant qu'aucun ne change, il n'y a rien à redécider.
    vu: Option<(String, Viewport, u64, (u32, u32))>,
    /// Les membranes du tableau telles que la décision les lit, pour un état du document :
    /// pendant un zoom, seule la vue change, et la décision ne parcourt que les membranes.
    carte: Option<((String, u64), CarteDesMembranes)>,
}

impl Focus {
    /// La membrane focalisée, s'il y en a une.
    pub fn membrane(&self) -> Option<&str> {
        self.etat.membrane_id.as_deref()
    }
}

impl GlucoseApp {
    /// Demande au noyau s'il faut entrer en focus, y rester ou en sortir — seulement si la
    /// vue, le tableau ou le document ont changé —, puis dit au rendu ce qui se voit.
    pub(super) fn suivre_le_focus(&mut self, (largeur, hauteur): (u32, u32)) {
        self.decider_le_focus((largeur, hauteur));
        let membrane = self.focus.membrane().map(str::to_string);
        self.renderer
            .regler_le_focus(&self.store, membrane.as_deref());
    }

    fn decider_le_focus(&mut self, (largeur, hauteur): (u32, u32)) {
        let Some(tableau) = self.store.active_board() else {
            return;
        };
        let vu = (
            tableau.id.clone(),
            tableau.viewport,
            self.store.version,
            (largeur, hauteur),
        );
        if self.focus.vu.as_ref() == Some(&vu) {
            return;
        }
        let bandeau = f64::from(self.ui.header_height());
        let ecran = ScreenSize {
            width: f64::from(largeur),
            height: (f64::from(hauteur) - bandeau).max(1.0),
        };
        let vue_du_canevas = Viewport {
            y: tableau.viewport.y - bandeau,
            ..tableau.viewport
        };
        let cle = (tableau.id.clone(), self.store.version);
        let carte = match self.focus.carte.take() {
            Some((c, carte)) if c == cle => carte,
            _ => CarteDesMembranes::du_tableau(tableau),
        };
        let action = decider(
            &carte,
            vue_du_canevas,
            ecran,
            self.focus.etat.clone(),
            crate::persist::now_millis(),
        );
        self.focus.carte = Some((cle, carte));
        self.focus.vu = Some(vu);
        match action {
            FocusAction::Stay(etat) => self.focus.etat = etat,
            FocusAction::Exit(etat) => {
                self.focus.etat = etat;
                self.mark_dirty();
            }
            FocusAction::Enter { state, fit, .. } => {
                self.focus.etat = state;
                self.vol.voler_vers(Viewport {
                    y: fit.y + bandeau,
                    ..fit
                });
                self.mark_dirty();
            }
        }
    }
}

#[cfg(test)]
mod tests;
