//! **ZEN-1 — le mode zen** (`F11`, fiche 115) : celui de Glucose Tauri (`App.tsx`) — plus aucune
//! interface, le canevas seul ; un « ZEN · F11 » discret en haut à droite le referme d'un clic,
//! comme `F11`, `Échap`, la palette et le menu du vide.
//!
//! * **La fenêtre ne change pas** : agrandie et sans cadre, une fenêtre ressemble à un jeu en plein
//!   écran, et l'application NVIDIA lui a appliqué RTX HDR — toutes ses couleurs changées (son
//!   essai du 7/10, `interactions/reference.rs`). Tauri ne la touchait pas non plus.
//! * **Le menu du clic droit et les messages restent**, comme en mode référence : au doigt, sans
//!   touche `F11` ni `Échap`, l'indicateur et le menu sont les sorties.
//! * **La mise en page est une fonction pure** ([`placer`]) : le dessin et la main lisent la même
//!   (loi L4).

use super::UiState;
use crate::theme::{Theme, CIBLE_DU_DOIGT};
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::PixmapMut;

/// Le libellé de Tauri, avec la touche de Glucose.
const LIBELLE: &str = "ZEN · F11";
/// Les mesures de Tauri : un corps de 10, à 8 du haut et 12 de la droite.
const CORPS: f32 = 10.0;
const HAUT: f32 = 8.0;
const DROITE: f32 = 12.0;

/// Un rectangle `(x, y, largeur, hauteur)`, en points de l'écran.
pub type Boite = (f32, f32, f32, f32);

/// **L'indicateur posé** : où s'écrit son libellé, et la boîte qui le prend — le libellé, élargi
/// jusqu'à une cible de doigt (48 points) autour de lui.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Indicateur {
    pub libelle: (f32, f32),
    pub prise: Boite,
}

/// **Pose l'indicateur** en haut à droite d'un écran de `(largeur, …)`, sous ce que le système y
/// garde (la barre d'état d'un téléphone, une encoche).
pub fn placer(ui: &UiState, typo: &Typography, (largeur, _): (f32, f32)) -> Indicateur {
    let s = ui.scale();
    let (l, h) = typo.measure_text(LIBELLE, CORPS * s, Face::Regular);
    let (x, y) = (
        largeur - ui.marges.droite - DROITE * s - l,
        ui.marges.haut + HAUT * s,
    );
    let (pl, ph) = ((CIBLE_DU_DOIGT * s).max(l), (CIBLE_DU_DOIGT * s).max(h));
    let centre = (x + l / 2.0, y + h / 2.0);
    // La prise s'arrête aux bords de ce que le système laisse : au-delà, un toucher ne vient pas.
    let gauche = (centre.0 - pl / 2.0).min(largeur - ui.marges.droite - pl);
    let haut = (centre.1 - ph / 2.0).max(ui.marges.haut);
    Indicateur {
        libelle: (x, y),
        prise: (gauche, haut, pl, ph),
    }
}

/// **Ce point est-il sur l'indicateur ?**
pub fn sous_la_main(i: &Indicateur, (px, py): (f32, f32)) -> bool {
    let (x, y, l, h) = i.prise;
    px >= x && px < x + l && py >= y && py < y + h
}

/// **Dessine l'indicateur** : un libellé discret, plus clair sous la main.
pub fn dessiner(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    (typo, theme): (&Typography, &Theme),
    (ecran, main): ((f32, f32), (f32, f32)),
) {
    let i = placer(ui, typo, ecran);
    let color = if sous_la_main(&i, main) {
        theme.text_primary
    } else {
        theme.text_muted
    };
    let style = TextStyle {
        size: CORPS * ui.scale(),
        color,
        face: Face::Regular,
    };
    typo.draw_text(pixmap, LIBELLE, i.libelle.0, i.libelle.1, style);
}

#[cfg(test)]
mod tests;
