//! **Décoder directement dans la page du niveau natif** (MEMOIRE-3, fiche 66).
//!
//! # Trois copies pleine taille, pour une seule image
//!
//! L'atelier décodait par `image::load_from_memory`, qui rend l'image dans sa couleur
//! d'origine — trois octets par pixel pour une photo JPEG —, puis `to_rgba8` en faisait une
//! seconde copie à quatre octets, que la pyramide recopiait enfin dans ses pages. Pour une photo
//! de cinquante mégapixels, celle d'un moto g54 : 150 et 200 Mo vivants ensemble, puis 200 et
//! 267 — un pic de 467 Mo, et autant de fois que d'ouvriers.
//!
//! # Une seule page
//!
//! Le décodeur écrit sa couleur d'origine au début de la page du niveau natif, `k` octets par
//! pixel ; puis chaque pixel s'étale à quatre octets, prémultiplié, **de la fin vers le début**.
//! Le pixel `i` se lit en `[k·i, k·i + k)` et s'écrit en `[4·i, 4·i + 4)` : tout pixel `j < i`
//! se lit avant `k·(j + 1) ≤ k·i ≤ 4·i`, donc rien de ce qui reste à lire n'est écrasé ; et le
//! pixel lui-même est lu avant d'être écrit. Le pic n'est plus que la pyramide, et ce que le
//! décodeur garde pour travailler.
//!
//! Les couleurs que cette loi ne couvre pas — seize bits, flottants — gardent le chemin général
//! ([`Pyramide::depuis_rgba`]), qui convertit comme avant : les deux chemins donnent les mêmes
//! octets, au bit près, et une épreuve le vérifie.

use super::{Niveau, Pyramide};
use image::{ColorType, ImageDecoder};

impl Pyramide {
    /// Combien d'octets par pixel le décodeur écrit pour cette couleur, si elle s'étale sur
    /// place : gris, gris et alpha, RGB, RGBA — sur huit bits.
    pub fn octets_par_pixel(couleur: ColorType) -> Option<usize> {
        match couleur {
            ColorType::L8 => Some(1),
            ColorType::La8 => Some(2),
            ColorType::Rgb8 => Some(3),
            ColorType::Rgba8 => Some(4),
            _ => None,
        }
    }

    /// **Une image décodée dans la page de son niveau natif**, sans autre copie pleine taille.
    ///
    /// `None` quand le système refuse la place, quand le décodeur échoue, ou quand sa couleur
    /// ne s'étale pas sur place ([`Self::octets_par_pixel`]) — l'appelant le sait d'avance.
    pub fn depuis_le_decodeur(decodeur: impl ImageDecoder) -> Option<Self> {
        let k = Self::octets_par_pixel(decodeur.color_type())?;
        let (largeur, hauteur) = decodeur.dimensions();
        let mut niveau = Niveau::vide(largeur, hauteur)?;
        let page = niveau.octets_mut()?;
        let pixels = largeur as usize * hauteur as usize;
        decodeur.read_image(page.get_mut(..pixels * k)?).ok()?;
        etaler(page, pixels, k);
        Self::batir(niveau)
    }
}

/// **Étale `pixels` pixels de `k` octets en RGBA prémultiplié, sur place, de la fin vers le
/// début** — la loi du module.
fn etaler(page: &mut [u8], pixels: usize, k: usize) {
    let lire = |page: &[u8], s: usize| -> [u8; 4] {
        match k {
            1 => [page[s], page[s], page[s], 255],
            2 => [page[s], page[s], page[s], page[s + 1]],
            3 => [page[s], page[s + 1], page[s + 2], 255],
            _ => [page[s], page[s + 1], page[s + 2], page[s + 3]],
        }
    };
    for i in (0..pixels).rev() {
        let px = lire(page, i * k);
        // La loi de la pyramide, une seule (`premultiplier`), appelée pour un pixel.
        super::pixels::premultiplier(&px, &mut page[i * 4..i * 4 + 4]);
    }
}

#[cfg(test)]
mod tests;
