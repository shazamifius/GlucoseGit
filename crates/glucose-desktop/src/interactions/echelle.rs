//! **ECHELLE-1 — l'échelle de l'interface, à son choix** (fiche 121).
//!
//! Sa capture du 17/11 : le texte de l'interface *« très très pixelisé, alors que je possède souvent
//! des écrans bien plus performants »*. Le grain est réglé (GRILLE-1, fiche 120) ; reste la taille :
//! sur un grand écran à 100 %, la bande et les tiroirs sont petits, et le système seul en décide.
//! Comme la « Resolution Scale » de Blender — *« Adjusts the size of fonts and buttons relative to
//! the automatically detected DPI »* —, l'utilisateur **multiplie** l'échelle que le système donne.
//!
//! # Une seule valeur, que tout lit
//!
//! `UiState::scale_factor` est **l'échelle de l'interface** : celle de l'écran, fois ce choix.
//! Tout ce qui dessine la chrome et tout ce qui y reçoit un clic la lisaient déjà : le dessin et le
//! clic restent d'accord par construction (la leçon de BARRE-2, fiche 118 : un clic qui cherchait
//! les boutons ailleurs qu'ils n'étaient dessinés). Ce qui parle au matériel garde l'échelle de
//! l'écran (`GlucoseApp::scale_factor`) : les bords de la fenêtre en mode référence, la vitesse
//! perçue d'un geste, le pincement. Le canevas ne lit ni l'une ni l'autre : son zoom est le sien.
//!
//! # Les crans
//!
//! Ceux du zoom de Chrome, de 50 à 300 % : des pourcentages que chacun connaît déjà. Pas de
//! raccourci par défaut — `Ctrl` `+` et `Ctrl` `-` zooment le canevas dans Figma et dans Miro, et
//! Blender n'en donne pas à son échelle : le menu du vide, à côté du plein écran, et la palette.
//! Le choix se retient d'une session à l'autre, à côté du mode de la fenêtre.

use crate::app::GlucoseApp;
use crate::ui::UiState;

/// Les crans de l'échelle, en pour cent (ceux du zoom de Chrome entre 50 et 300 %).
pub const CRANS: [u16; 13] = [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300];

/// Le nom du souvenir, à côté de celui du mode de la fenêtre.
const SOUVENIR: &str = "echelle-interface";

/// Où va l'échelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vers {
    PlusGrand,
    PlusPetit,
    Cent,
}

impl Vers {
    /// Les trois gestes, dans l'ordre de la palette.
    pub const TOUS: [Vers; 3] = [Vers::PlusGrand, Vers::PlusPetit, Vers::Cent];

    /// La fin de son identifiant, `interface.<id>` : stable, une couche de raccourcis l'écrit.
    pub fn id(self) -> &'static str {
        match self {
            Vers::PlusGrand => "agrandir",
            Vers::PlusPetit => "reduire",
            Vers::Cent => "cent",
        }
    }

    /// Son nom à l'écran.
    pub fn nom(self) -> &'static str {
        match self {
            Vers::PlusGrand => "Agrandir l'interface",
            Vers::PlusPetit => "Réduire l'interface",
            Vers::Cent => "L'interface à 100 %",
        }
    }
}

/// **Le cran où mène ce geste**, ou rien s'il ne changerait pas l'interface : au bout des crans,
/// ou quand l'échelle bornée de l'interface ([`crate::theme::clamp_ui_scale`]) est déjà atteinte.
pub fn cran_vers(ui: &UiState, vers: Vers) -> Option<u16> {
    let ici = CRANS.iter().position(|&c| c == ui.echelle_choisie);
    let (min, max) = (crate::theme::MIN_UI_SCALE, crate::theme::MAX_UI_SCALE);
    let cran = match vers {
        Vers::Cent => Some(100),
        Vers::PlusGrand if ui.scale() < max => match ici {
            Some(i) => CRANS.get(i + 1).copied(),
            None => CRANS.iter().copied().find(|&c| c > ui.echelle_choisie),
        },
        Vers::PlusPetit if ui.scale() > min => match ici {
            Some(i) => i.checked_sub(1).map(|i| CRANS[i]),
            None => CRANS
                .iter()
                .rev()
                .copied()
                .find(|&c| c < ui.echelle_choisie),
        },
        _ => None,
    };
    cran.filter(|&c| c != ui.echelle_choisie)
}

impl GlucoseApp {
    /// **Pose l'échelle de l'interface** : celle de l'écran, fois le choix. Au lancement, quand
    /// l'écran change d'échelle, et à chaque cran.
    pub(crate) fn poser_l_echelle(&mut self) {
        self.ui.scale_factor =
            self.scale_factor as f32 * f32::from(self.ui.echelle_choisie) / 100.0;
        self.mark_dirty();
    }

    /// **Change l'échelle de l'interface d'un cran**, ou la remet à 100 % — et la retient.
    pub(crate) fn changer_l_echelle(&mut self, vers: Vers) {
        let Some(cran) = cran_vers(&self.ui, vers) else {
            return;
        };
        self.ui.echelle_choisie = cran;
        let _ = std::fs::write(self.souvenir_de_la_fenetre(SOUVENIR), cran.to_string());
        self.ui.context_menu_at = None;
        self.poser_l_echelle();
    }

    /// **L'échelle choisie à la session précédente** — 100 % si rien n'est retenu, ou si ce qui
    /// l'est n'est pas un cran (un fichier abîmé ne fait pas une interface géante).
    pub(crate) fn echelle_retenue(&self) -> u16 {
        std::fs::read_to_string(self.souvenir_de_la_fenetre(SOUVENIR))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .filter(|c| CRANS.contains(c))
            .unwrap_or(100)
    }
}

#[cfg(test)]
#[path = "echelle/tests.rs"]
mod tests;
