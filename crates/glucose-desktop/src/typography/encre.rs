//! **L'encre d'une lettre** (GRIS-1, fiche 95) : la surface qu'elle couvre — ce qu'un texte montre
//! quand il est trop loin pour que ses lettres se lisent.
//!
//! Vu de loin, un texte n'est plus des lettres : c'est un gris, celui de l'encre de ses lettres
//! répartie sur la place qu'elles occupent. Glucose remplaçait ce texte par le cadre vide de son
//! nœud (SCALE-2) ; sa parole du 8/11 : *« pas de LOD, pas de simplification »*, puis, devant la
//! proposition, *« carrément d'accord pour le gris des lignes »*. Le gris se lit sur la police
//! elle-même, aucune densité n'est choisie.

use super::{Face, Typography};

impl Typography {
    /// **L'encre de `ch` à ce corps**, en pixels carrés : la somme de la couverture de son image,
    /// lue une fois par lettre au plus grand corps qui garde son image (un demi-carré, GRAND-1),
    /// puis mise à l'échelle du carré du corps — une surface croît comme lui.
    pub fn encre(&self, ch: char, size: f32, face: Face) -> f32 {
        let reference = (glucose_core::tuile::COTE / 2) as f32;
        let surface = *self
            .encres
            .borrow_mut()
            .entry((face, ch))
            .or_insert_with(|| {
                let (font, indice) = self.police(ch, face);
                let (_, image) = font.rasterize_indexed(indice, reference);
                image.iter().map(|&c| f32::from(c)).sum::<f32>() / 255.0
            });
        surface * (size / reference).powi(2)
    }

    /// **Ce qu'un morceau de texte montre de loin** : la largeur de ses lettres, sans ses blancs
    /// de fin, et leur encre, au corps `size`. Chaque lettre vient avec son visage — ou rien, pour
    /// un signe qui n'occupe aucune place (MODE-1).
    pub fn gris(
        &self,
        lettres: impl Iterator<Item = (char, Option<Face>)>,
        size: f32,
    ) -> (f32, f32) {
        let (mut avance, mut largeur, mut encre) = (0.0f32, 0.0f32, 0.0f32);
        for (ch, face) in lettres {
            let Some(face) = face else {
                continue;
            };
            avance += self.advance(ch, size, face);
            encre += self.encre(ch, size, face);
            if !ch.is_whitespace() {
                largeur = avance;
            }
        }
        (largeur, encre)
    }

    /// **La densité d'un texte courant** dans ce visage : l'encre de l'alphabet sur la place
    /// qu'il occupe — ses avances fois son corps. Elle ne dépend pas du corps (l'encre croît comme
    /// son carré, la place aussi). C'est le gris d'une formule vue de loin, dont l'encre n'est pas
    /// celle de lettres qu'on sait lire une à une.
    pub fn densite(&self, face: Face) -> f32 {
        let corps = (glucose_core::tuile::COTE / 2) as f32;
        let (encre, place) = ('a'..='z').fold((0.0, 0.0), |(e, p), c| {
            (
                e + self.encre(c, corps, face),
                p + self.advance(c, corps, face) * corps,
            )
        });
        encre / place
    }
}
