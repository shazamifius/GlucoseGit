//! **L'encre d'une lettre** (GRIS-1, fiche 95) : la surface qu'elle couvre — ce qu'un texte montre
//! quand il est trop loin pour que ses lettres se lisent.
//!
//! Vu de loin, un texte n'est plus des lettres : c'est un gris, celui de l'encre de ses lettres
//! répartie sur la place qu'elles occupent. Glucose remplaçait ce texte par le cadre vide de son
//! nœud (SCALE-2) ; sa parole du 8/11 : *« pas de LOD, pas de simplification »*, puis, devant la
//! proposition, *« carrément d'accord pour le gris des lignes »*. Le gris se lit sur la police
//! elle-même, aucune densité n'est choisie.
//!
//! # GRIS-2 — la loi vit dans la typographie du canevas (fiche 96)
//!
//! Une lettre plus petite que [`LETTRE_LISIBLE`] ne porte plus sa forme : tout texte que le canevas
//! écrit à cette taille — le nom d'une zone, d'un dossier, l'étiquette d'une flèche, le mot d'un
//! opérateur, le sigle d'un domaine — se peint du gris de son encre, à la couverture exacte de
//! chaque pixel ([`Typography::griser`], [`peindre_exact`]). Chaque dessin s'en remettait à un seuil de zoom pour se
//! **taire** de loin (SCALE-2) ; il écrit désormais toujours, et la typographie décide comment.
//! L'export SVG, qui s'agrandit sans perte, n'a pas de pixels : il garde ses lettres.

use super::{Face, TextStyle, Typography};
use glucose_core::membrane_forme::{peindre, peindre_un_rectangle, Arrondi, Membrane};
use tiny_skia::{Color, PixmapMut};

/// **Le corps, en pixels, sous lequel une lettre ne se lit plus** : moins que l'épaisseur d'un
/// jambage pour ses traits, une forme qui ne transporte plus que son encre. C'est le seuil de
/// SCALE-2 (le corps d'un nœud de texte, 14, à un cinquième de zoom), dit pour ce qu'il est : une
/// taille de lettre à l'écran — la forme que les interfaces zoomables donnent au « greeking »
/// (Piccolo, GEF). Le seuil de zoom des nœuds s'en déduit.
pub const LETTRE_LISIBLE: f32 = 2.8;

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

    /// **Le gris d'un texte trop petit pour se lire** (GRIS-2) : le rectangle de sa ligne — sa
    /// largeur sans les blancs de fin, un corps de haut, son coin en `(x, y)` comme celui de ses
    /// lettres — et son encre ([`forme_grise`]) ; puis l'abscisse après le texte, comme le dessin
    /// de ses lettres. Le canevas le peint ([`Typography::griser`]), la carte graphique le pose.
    pub fn gris_d_un_texte(
        &self,
        text: &str,
        (x, y): (f32, f32),
        style: TextStyle,
    ) -> (Option<(Arrondi, Color)>, f32) {
        let corps = style.size.max(0.0);
        let lettres = || text.chars().filter(|&c| c != '\n');
        let (largeur, encre) = self.gris(lettres().map(|c| (c, Some(style.face))), corps);
        let boite = (x, y, largeur, corps);
        let forme = forme_grise(boite, (encre, largeur * corps), style.color);
        let fin = x + lettres()
            .map(|c| self.advance(c, corps, style.face))
            .sum::<f32>();
        (forme, fin)
    }

    /// **Un texte trop petit pour se lire, peint de son gris**, à la couverture exacte de chaque
    /// pixel (la loi des membranes). Rend l'abscisse après le texte.
    pub(super) fn griser(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        coin: (f32, f32),
        style: TextStyle,
    ) -> f32 {
        let (forme, fin) = self.gris_d_un_texte(text, coin, style);
        if let Some(forme) = forme {
            peindre_exact(pixmap, forme);
        }
        fin
    }
}

/// **La loi du gris** (GRIS-1, GRIS-2) : le rectangle `(x, y, largeur, hauteur)` de l'écran, de
/// cette couleur à l'opacité de son `encre` sur sa `place` — la surface de ses lettres sur celle
/// qu'elles occupent, à la même échelle. Une ligne d'un nœud, un post-it, un nom, une étiquette :
/// tous la suivent. Rien pour un texte sans encre.
pub fn forme_grise(
    (x, y, largeur, hauteur): (f32, f32, f32, f32),
    (encre, place): (f32, f32),
    couleur: Color,
) -> Option<(Arrondi, Color)> {
    if !(encre > 0.0 && place > 0.0) {
        return None;
    }
    let opacite = (encre / place).min(1.0) * couleur.alpha();
    let c = Color::from_rgba(couleur.red(), couleur.green(), couleur.blue(), opacite)?;
    Some((Arrondi::nouveau(x, y, largeur, hauteur, 0.0), c))
}

/// **Peint une forme pleine vue de loin.** Un rectangle droit — le gris d'une ligne, un filet, une
/// plaque — à la **couverture exacte** de chaque pixel, à toute épaisseur
/// ([`peindre_un_rectangle`]) : un filet d'un dixième de pixel y garde un dixième de son encre, là
/// où la grille des pixels (SCALE-3, nette de près) le ramènerait à un pixel entier, dix fois trop,
/// et la loi des membranes à trois fois en moyenne (fiche 97). Un disque (la puce) par la loi des
/// membranes (MEMB-FORME-1), juste dès qu'il fait un pixel.
pub fn peindre_exact(pixmap: &mut PixmapMut, (forme, c): (Arrondi, Color)) {
    let (l, h) = (pixmap.width(), pixmap.height());
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    let teinte = [c.red(), c.green(), c.blue()];
    if forme.rayon <= 0.0 {
        let boite = (forme.gauche, forme.haut, forme.droite, forme.bas);
        peindre_un_rectangle(boite, (teinte, c.alpha()), pixels, (l, h));
    } else {
        peindre(&Membrane::plein(forme, teinte, c.alpha()), pixels, l, h);
    }
}

#[cfg(test)]
mod tests;
