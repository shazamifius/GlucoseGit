//! **Une photo droite** (EXIF-1, fiche 165) : un téléphone écrit souvent ses pixels dans le sens de son
//! capteur, et une balise EXIF qui dit comment les tourner. Le niveau natif se tourne une fois, au
//! décodage, avant que la pyramide se bâtisse : tout l'aval — les deux voies, l'export, le recadrage —
//! voit une photo droite sans rien savoir de l'EXIF.
//!
//! La mémoire (MEMOIRE-3) : une page neuve de la taille native, quand le décodeur a fini — son travail,
//! que [`super::super::atelier`] compte à quatre octets par pixel dans le pic d'admission, est alors
//! rendu ; la page tournée prend sa place, et l'ancienne part aussitôt.

use super::niveau::Niveau;
use image::metadata::Orientation;

impl Niveau {
    /// **Ce niveau, tel que la photo demande d'être vue.** Le sens d'`image` : `Rotate90` tourne d'un
    /// quart dans le sens des aiguilles (l'EXIF 6, un portrait tenu droit), les `Flip…` retournent.
    pub(super) fn oriente(self, orientation: Orientation) -> Option<Self> {
        if orientation == Orientation::NoTransforms {
            return Some(self);
        }
        let (l, h) = (self.largeur as usize, self.hauteur as usize);
        let transpose = matches!(
            orientation,
            Orientation::Rotate90
                | Orientation::Rotate270
                | Orientation::Rotate90FlipH
                | Orientation::Rotate270FlipH
        );
        let (nl, nh) = if transpose {
            (self.hauteur, self.largeur)
        } else {
            (self.largeur, self.hauteur)
        };
        let mut droit = Niveau::vide(nl, nh)?;
        let source = self.vue()?;
        let source = source.data();
        let cible = droit.octets_mut()?;
        for y in 0..h {
            for x in 0..l {
                let (cx, cy) = match orientation {
                    Orientation::FlipHorizontal => (l - 1 - x, y),
                    Orientation::Rotate180 => (l - 1 - x, h - 1 - y),
                    Orientation::FlipVertical => (x, h - 1 - y),
                    Orientation::Rotate90FlipH => (y, x),
                    Orientation::Rotate90 => (h - 1 - y, x),
                    Orientation::Rotate270FlipH => (h - 1 - y, l - 1 - x),
                    Orientation::Rotate270 => (y, l - 1 - x),
                    Orientation::NoTransforms => (x, y),
                };
                let de = 4 * (y * l + x);
                let vers = 4 * (cy * nl as usize + cx);
                cible[vers..vers + 4].copy_from_slice(&source[de..de + 4]);
            }
        }
        Some(droit)
    }
}

#[cfg(test)]
mod tests;
