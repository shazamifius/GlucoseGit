//! Interaction de navigation caméra : molette, pavé tactile, pan à la souris.
//!
//! # NAV-2 — un événement de défilement dit **zoom** ou **pan**, jamais les deux
//!
//! La règle est celle de Glucose Tauri (`src/canvas/navigation.ts`), et elle vaut d'être
//! reprise telle quelle : c'est la seule qui rende un pavé tactile utilisable.
//!
//! * pincement — que le pilote livre en `Ctrl` + défilement — : **zoom** ;
//! * cran de molette de souris : **zoom** ;
//! * tout le reste, c'est-à-dire un glissement à deux doigts : **pan**.
//!
//! # Reconnaître un cran de souris sans rien inventer
//!
//! Tauri distingue les deux en testant `wheelDeltaY % 120 == 0` : sous Windows, une souris
//! rend exactement 120 unités par cran, un pavé tactile rend ce que vaut le doigt. Winit
//! livre la **même** grandeur déjà divisée par 120 — donc « multiple de 120 » s'écrit ici
//! « nombre entier ». Aucune constante nouvelle : la même information, mieux dite.
//!
//! Le piège que cela corrige : l'ancienne version décidait sur le **type** winit, `LineDelta`
//! contre `PixelDelta`. Or Windows livre tout en `LineDelta`, pavé tactile compris — alors
//! deux doigts vers le bas zoomaient, et le pan vertical au pavé tactile était littéralement
//! inatteignable.

use crate::app::GlucoseApp;
use winit::event::MouseScrollDelta;

/// Bornes du zoom **au geste** — molette et pincement (fiche 07 § 7.1) : de ×50 dézoomé à
/// ×20 zoomé. Plus étroites que celles du modèle ([`glucose_core::types::Viewport::SCALE_RANGE`]),
/// qu'un signet ou un fichier peuvent atteindre sans que la main y arrive.
pub const WHEEL_SCALE_RANGE: (f64, f64) = (0.02, 20.0);

/// Ce qu'un pixel de défilement fait à l'échelle, quand le geste est un zoom.
///
/// Reprise exacte de Glucose Tauri (`Math.pow(0.999, delta)`). Un cran de souris vaut alors
/// environ +4 %, là où la version précédente en donnait +12 % : c'est cet écart que l'œil
/// lisait comme un saut.
const ZOOM_PAR_PIXEL: f64 = 0.999;

/// Une ligne de défilement, en pixels, pour un **zoom**.
const ZOOM_LIGNE_PX: f64 = 40.0;

/// Une ligne de défilement, en pixels, pour un **pan**.
///
/// Tauri emploie 40 pour le zoom et 16 pour le pan — deux valeurs pour une seule conversion.
/// C'est une incohérence de la référence, et elle est reprise **délibérément** : c'est cette
/// sensation-là qui est validée à la main. Les unifier est une expérience à mener au doigt,
/// pas une correction à faire sur le papier.
const PAN_LIGNE_PX: f64 = 16.0;

/// Ce qu'un événement de défilement demande à la caméra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geste {
    /// Multiplier l'échelle par ce facteur, autour du curseur.
    Zoom(f64),
    /// Déplacer le contenu de tant de pixels écran.
    Pan(f64, f64),
}

/// NAV-2 — ce qu'un événement de défilement veut dire.
///
/// Fonction pure : c'est elle qui porte toute la décision, et elle se teste sans fenêtre.
pub fn geste(delta: MouseScrollDelta, ctrl: bool) -> Geste {
    let (dx, dy, ligne) = deltas(delta);
    if ctrl || cran_de_souris(dx, dy, ligne) {
        let pixels = dy * if ligne { ZOOM_LIGNE_PX } else { 1.0 };
        return Geste::Zoom(ZOOM_PAR_PIXEL.powf(-pixels));
    }
    let px = if ligne { PAN_LIGNE_PX } else { 1.0 };
    Geste::Pan(dx * px, dy * px)
}

/// Les deux composantes d'un défilement, et s'il s'exprime en lignes.
fn deltas(delta: MouseScrollDelta) -> (f64, f64, bool) {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => (f64::from(x), f64::from(y), true),
        MouseScrollDelta::PixelDelta(p) => (p.x, p.y, false),
    }
}

/// **Pourquoi** ce geste a été décidé — pour la trace, jamais pour le comportement.
///
/// La décision elle-même reste dans [`geste`], à un seul endroit. Celle-ci n'en lit que la
/// raison, parce qu'un défilement pris pour un cran de souris est le cas ambigu : s'il abonde
/// pendant qu'on glisse à deux doigts, chacun coûte un saut d'échelle visible — et aucune
/// mesure de durée ne le montrerait.
pub fn pourquoi(
    delta: MouseScrollDelta,
    ctrl: bool,
    pincement: bool,
) -> crate::chronique::navigation::Decision {
    use crate::chronique::navigation::Decision;
    if pincement {
        return Decision::Pincement;
    }
    let (dx, dy, ligne) = deltas(delta);
    if !ctrl && cran_de_souris(dx, dy, ligne) {
        return Decision::CranDeSouris;
    }
    match geste(delta, ctrl) {
        Geste::Zoom(_) => Decision::Zoom,
        Geste::Pan(..) => Decision::Pan,
    }
}

/// Un cran de molette de souris : vertical pur, et d'un nombre **entier** de lignes.
///
/// Un pavé tactile ne remplit ces deux conditions ensemble que par accident, et l'accident
/// coûte une image de zoom au milieu d'un pan — invisible. L'inverse, prendre une souris pour
/// un pavé, coûterait tout le zoom à la molette.
fn cran_de_souris(dx: f64, dy: f64, ligne: bool) -> bool {
    ligne && dx == 0.0 && dy != 0.0 && dy.fract() == 0.0
}

impl GlucoseApp {
    /// Gère les événements de molette et gestes tactiles.
    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (cx, cy) = self.mouse_pos;
        // Le `Ctrl` d'un pincement est virtuel : il vit dans le message du systeme, pas dans
        // l'etat du clavier que `winit` rapporte. Les deux sources disent la meme chose --
        // « ce defilement veut zoomer » -- et se lisent donc ensemble, ici et nulle part
        // ailleurs, pour que la decision elle-meme reste une fonction pure.
        let pincement = super::pincement::zoom_du_systeme();
        let zoom_demande = self.modifiers.control_key() || pincement;
        // NAV-3 : ce que le doigt a demande entre dans la trace, avec l'instant ou il l'a
        // demande. C'est de la qu'on saura si l'ecran suit la main.
        self.chronique
            .navigation
            .evenement(pourquoi(delta, self.modifiers.control_key(), pincement));
        match geste(delta, zoom_demande) {
            Geste::Zoom(facteur) => self.store.zoom(facteur, cx, cy, WHEEL_SCALE_RANGE),
            Geste::Pan(dx, dy) => self.store.pan(dx, dy),
        }
        self.mark_dirty();
    }

    /// Déplacement relatif de la caméra lors d'un pan souris (bouton milieu, droit, ou l'outil
    /// Pan où la barre d'espace fait entrer).
    pub fn handle_pan_move(&mut self, dx: f64, dy: f64) {
        // Protection contre les sauts anormaux du curseur OS
        if dx.hypot(dy) < 300.0 {
            self.store.pan(dx, dy);
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
