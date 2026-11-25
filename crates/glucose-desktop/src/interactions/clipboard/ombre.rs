//! **L'ombre des nœuds qu'on emporte** (Q10, fiche 155) : un glisser vers une autre fenêtre ne
//! montrait que le curseur. Les nœuds tenus se dessinent désormais sous la main — l'export en
//! image de la sélection (EXPORT-PNG-1, fiche 62), sur un fond transparent, à la taille où on
//! les voit, plus petits s'ils sont grands, et translucides.
//!
//! Rien n'est choisi pour la forme : c'est l'export, avec sa marge (ce que le contenu peint
//! au-delà de ses boîtes) ; et le point sous la main est celui où elle tenait les nœuds.

use crate::app::GlucoseApp;
use crate::plateforme::Ombre;
use crate::ui::UiState;
use glucose_core::types::Viewport;

/// **Le plus grand côté de l'ombre, en points de l'écran** : celui de Safari (WebCore,
/// `DragController::maxDragImageSize`, 400 × 400 sous macOS ; 200 ailleurs) — une ombre plus
/// grande cacherait la fenêtre où l'on va la lâcher.
const COTE: f64 = 400.0;

/// **Son opacité** : celle du fantôme d'une fenêtre qu'on glisse sous ChromeOS
/// (`kDragPhantomMaxOpacity`) — on voit au travers où on la pose.
const OPACITE: f32 = 0.8;

impl GlucoseApp {
    /// **Où la main tient les nœuds**, en unités du monde depuis leur coin — lu pendant qu'ils
    /// sont encore sous elle, avant qu'ils ne reviennent à leur place.
    pub(super) fn prise_des_noeuds(&self, board: &str, ecran: (f64, f64)) -> Option<(f64, f64)> {
        let region = self.store.selection_bounds(board)?;
        let vue = self.store.viewport();
        let (x, y) = crate::canvas::screen_to_world(ecran.0, ecran.1, &vue);
        Some((x - region.left, y - region.top))
    }

    /// **L'ombre des nœuds choisis**, la main à `prise` de leur coin ; `facteur`, l'échelle de
    /// l'écran (des pixels par point).
    pub(super) fn ombre_des_noeuds(
        &self,
        board: &str,
        prise: (f64, f64),
        facteur: f64,
    ) -> Option<Ombre> {
        let region = self.store.selection_bounds(board)?;
        let densite = facteur.ceil().max(1.0);
        // Un point de l'ombre, un point de l'écran : la taille où on les voit.
        let mut echelle = self.store.viewport().scale / facteur * densite;
        let (cote, plafond) = (region.width.max(region.height), COTE * densite);
        let unite = crate::renderer::export::debord(1.0);
        if (cote + 2.0 * unite) * echelle > plafond {
            // La marge croît comme l'échelle (la lueur s'étale et se floute d'autant) : le
            // contenu et ses deux marges tiennent ensemble dans le plafond.
            echelle = plafond / (cote + 2.0 * unite);
        }
        let scene = self.store.copie_a_exporter(true);
        let marge = if scene.a_des_annotations() {
            crate::renderer::export::debord(echelle)
        } else {
            0.0
        };
        let d = densite as u32;
        // Un multiple de la densité : Wayland l'exige d'un tampon à cette échelle.
        let cote = |l: f64| {
            let pixels = (l * echelle + 2.0 * marge).ceil().clamp(1.0, plafond) as u32;
            pixels.div_ceil(d) * d
        };
        let (largeur, hauteur) = (cote(region.width), cote(region.height));
        let vue = Viewport {
            scale: echelle,
            x: marge - region.left * echelle,
            y: marge - region.top * echelle,
        };
        let mut pixmap = tiny_skia::Pixmap::new(largeur, hauteur)?;
        let mut moteur = crate::persist::export::png::moteur(self.peintre_de_l_export());
        moteur.theme.bg_canvas = tiny_skia::Color::TRANSPARENT;
        moteur.rendre_pour_l_export(&mut pixmap.as_mut(), &scene, &mut UiState::new(), vue);
        let point = |p: f64| (p * echelle + marge).round() as i32;
        Some(Ombre {
            rgba: translucide(pixmap.data()),
            largeur,
            hauteur,
            point: (point(prise.0), point(prise.1)),
            densite: d,
        })
    }
}

/// Des pixels prémultipliés, à l'opacité de l'ombre : chaque canal d'autant.
fn translucide(pixels: &[u8]) -> Vec<u8> {
    pixels
        .iter()
        .map(|o| (f32::from(*o) * OPACITE).round() as u8)
        .collect()
}

#[cfg(test)]
mod tests;
