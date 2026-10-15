//! **Les glyphes en tracés** (P5, fiche 70) : le texte du SVG s'écrit par les contours mêmes
//! des polices embarquées, chacun défini une fois et posé autant de fois qu'il revient.
//!
//! # Pourquoi pas `<text>`
//!
//! Un `<text>` demande sa police au lecteur. Inter n'est installée presque nulle part : le
//! lecteur la remplace, ses lettres n'ont plus les mêmes avances, et des lignes que la mise en
//! page a coupées à la mesure d'Inter débordent de leur nœud — chaque passage en gras laissait
//! un trou (vu dans `resvg`, fiche 70 § 1). Embarquer la police dans une `@font-face`
//! (tldraw, Excalidraw) ne sert que les navigateurs : `resvg`, Inkscape, Illustrator l'ignorent.
//!
//! Un contour ne demande rien à personne. Le glyphe est écrit dans les unités de sa police,
//! défini une fois (`<path>` dans `<defs>`), puis posé par un `<use>` à la place exacte où le
//! canevas le pose — la plume, la ligne de base à `y + corps`, l'avance de [`Typography`] —, à
//! l'échelle `corps / unités par em`. C'est le mode par défaut de matplotlib (`svg.fonttype =
//! 'path'`), et l'option « Outline text » de Figma. Le texte reste lisible par une machine :
//! chaque ligne porte ses mots dans un `aria-label`.
//!
//! Le même mécanisme écrit les formules (les fontes de KaTeX) et les emojis (Noto Emoji) : une
//! police nommée, un indice de glyphe.

use super::{n, Svg};
use crate::renderer::vectoriel::Vectoriel;
use crate::typography::{Police, TextStyle, Typography, DETOURAGE};
use std::fmt::Write as _;

/// Le contour d'un glyphe, écrit en commandes de chemin dans les unités de sa police — l'axe
/// des ordonnées vers le haut, comme la police le dit.
#[derive(Default)]
struct Contour(String);

impl ttf_parser::OutlineBuilder for Contour {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "M{} {}", n(x), n(y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "L{} {}", n(x), n(y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.0, "Q{} {} {} {}", n(x1), n(y1), n(x), n(y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(
            self.0,
            "C{} {} {} {} {} {}",
            n(x1),
            n(y1),
            n(x2),
            n(y2),
            n(x),
            n(y)
        );
    }

    fn close(&mut self) {
        self.0.push('Z');
    }
}

/// Un glyphe défini : son nom dans `<defs>`, et l'inverse des unités par em de sa police.
#[derive(Debug, Clone)]
pub(super) struct Defini {
    nom: String,
    par_unite: f32,
}

impl Svg {
    /// **Pose ce glyphe** à `(x, base)` — la plume et la ligne de base —, à ce corps ; le
    /// définit à sa première venue. Un glyphe sans contour (une espace) ne pose rien.
    pub fn glyphe(&mut self, (police, indice): (Police, u16), (x, base): (f32, f32), corps: f32) {
        let cle = (police.nom, indice);
        let defini = match self.glyphes.get(&cle) {
            Some(connu) => connu.clone(),
            None => {
                let neuf = self.definir_le_glyphe(police, indice);
                self.glyphes.insert(cle, neuf.clone());
                neuf
            }
        };
        let Some(Defini { nom, par_unite }) = defini else {
            return;
        };
        // L'échelle s'écrit entière : arrondie au millième comme une coordonnée, un glyphe de
        // 14 px dans une police de 2 048 unités changerait de taille de quelques pour cent.
        let s = corps * par_unite;
        self.poser(&format!(
            r##"<use xlink:href="#{nom}" transform="matrix({s} 0 0 {} {} {})"/>"##,
            -s,
            n(x),
            n(base)
        ));
    }

    /// Lit le contour du glyphe dans les tables de sa police et l'ajoute aux définitions.
    fn definir_le_glyphe(&mut self, police: Police, indice: u16) -> Option<Defini> {
        let face = ttf_parser::Face::parse(police.octets, 0).ok()?;
        let mut contour = Contour::default();
        face.outline_glyph(ttf_parser::GlyphId(indice), &mut contour)?;
        let nom = self.nom("g");
        self.definir(&format!(r#"<path id="{nom}" d="{}"/>"#, contour.0));
        Some(Defini {
            nom,
            par_unite: 1.0 / f32::from(face.units_per_em()),
        })
    }

    /// **Un texte sur une ligne**, son coin haut gauche en `(x, y)` : le texte de tout dessin
    /// vectoriel ([`Vectoriel::texte`], fiche 73), écrit en un groupe de glyphes.
    pub fn texte(
        &mut self,
        typographie: &Typography,
        at: (f32, f32),
        texte: &str,
        style: TextStyle,
    ) {
        Vectoriel::texte(self, typographie, at, texte, style);
    }

    /// **Un texte détouré** : ses huit copies de la teinte du contour ([`DETOURAGE`]), puis
    /// lui — le titre d'une membrane, lisible sur ce qu'il chevauche, comme au canevas.
    pub fn texte_detoure(
        &mut self,
        typographie: &Typography,
        (x, y): (f32, f32),
        texte: &str,
        (style, contour): (TextStyle, tiny_skia::Color),
    ) {
        let dessous = TextStyle {
            color: contour,
            ..style
        };
        for (dx, dy) in DETOURAGE {
            self.texte(typographie, (x + dx, y + dy), texte, dessous);
        }
        self.texte(typographie, (x, y), texte, style);
    }
}
