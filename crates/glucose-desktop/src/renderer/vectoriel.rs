//! **Le dessin vectoriel d'un nœud** (P8, fiche 73) : une seule description, que l'export SVG
//! écrit et que la carte graphique dessine.
//!
//! # Pourquoi
//!
//! L'export SVG (P5, fiches 69 à 71) décrit déjà un nœud de texte comme le canevas le montre —
//! sa brume, ses lignes, chaque glyphe par sa police et son indice, ses formules —, et cette
//! description a été éprouvée par superposition au PNG. De près, la carte graphique a besoin
//! **exactement** de cela : dessiner chaque lettre depuis son contour (Slug), au lieu de poser
//! une image du nœud que le processeur aurait peinte et qu'un zoom refait. Une seconde
//! description, pour la carte, divergerait de la première à la première retouche : il n'y en a
//! qu'une, et deux destinataires.
//!
//! # Le vocabulaire
//!
//! Celui du SVG, réduit à ce qu'un nœud emploie : un rectangle aux coins arrondis, un anneau
//! (le liseré d'un passage), un disque (la puce), un **groupe** qui porte l'encre de ses
//! glyphes — comme `<g fill>` la donne à ses `<use>` —, un glyphe, une forme de KaTeX. Un texte
//! est un groupe de glyphes, posés à la plume et à l'avance du canevas ([`Vectoriel::texte`]).

mod releve;
pub(crate) use releve::Releve;

use super::svg::{couleur, echapper, hex, n, rayon_tenu, Svg};
use crate::typography::{clamp_font_size, Police, TextStyle, Typography};
use glucose_math::Forme;
use std::fmt::Write as _;
use tiny_skia::Color;

/// **Ce qui reçoit le dessin vectoriel d'un nœud** : le SVG d'un export, ou la carte graphique.
pub(crate) trait Vectoriel {
    /// Le rectangle `(x, y, largeur, hauteur)` aux coins de `rayon`, plein de `couleur`.
    fn rectangle(&mut self, boite: (f32, f32, f32, f32), rayon: f32, couleur: Color);

    /// **Un rectangle net** : aux pixels entiers, sans anticrénelage — la règle de
    /// [`super::scale::fill_crisp`] (SCALE-3), celle des filets, des barres et des plaques.
    fn rectangle_net(&mut self, boite: (f32, f32, f32, f32), couleur: Color);

    /// **Un anneau** : le bord de ce rectangle arrondi repoussé de `decalage`, d'épaisseur
    /// `2 · demi` — le trait d'une membrane, et le liseré d'un passage qui brille. `trait_` vaut
    /// `(decalage, demi)`.
    fn anneau(
        &mut self,
        boite: (f32, f32, f32, f32),
        rayon: f32,
        trait_: (f32, f32),
        couleur: Color,
    );

    /// Le disque de ce centre et de ce rayon.
    fn disque(&mut self, centre: (f32, f32), rayon: f32, couleur: Color);

    /// **Ouvre un groupe** : ses glyphes prennent cette encre ; `mots` dit ce qu'il écrit, aux
    /// machines qui le liront.
    fn groupe(&mut self, mots: &str, encre: Color);

    /// Ferme le groupe ouvert.
    fn fin_du_groupe(&mut self);

    /// **Un glyphe**, à l'encre du groupe ouvert : sa police et son indice, la plume et la ligne
    /// de base, le corps.
    fn glyphe(&mut self, glyphe: (Police, u16), plume: (f32, f32), corps: f32);

    /// **Une forme de KaTeX** dans sa boîte `(x, y, largeur, hauteur)`, rognée par elle ; pleine,
    /// ou tracée à l'épaisseur `trait_px` (en pixels : elle ne s'étire pas avec la boîte).
    fn forme(
        &mut self,
        forme: &Forme,
        boite: (f32, f32, f32, f32),
        trait_px: Option<f32>,
        couleur: Color,
    );

    /// **Un texte sur une ligne**, son coin haut gauche en `at` : chaque caractère dans la police
    /// que la typographie lui choisit (EMOJI-1), la ligne de base à `y + corps`, la plume avancée
    /// de l'avance même du canevas — le texte du canevas, au trait près.
    fn texte(
        &mut self,
        typographie: &Typography,
        (x, y): (f32, f32),
        texte: &str,
        style: TextStyle,
    ) {
        let corps = clamp_font_size(style.size);
        self.groupe(texte, style.color);
        let mut plume = x;
        for ch in texte.chars().filter(|&ch| ch != '\n') {
            let glyphe = typographie.source(ch, style.face);
            self.glyphe(glyphe, (plume, y + corps), corps);
            plume += typographie.advance(ch, corps, style.face);
        }
        self.fin_du_groupe();
    }
}

/// **Les pixels qu'un rectangle net couvre**, par la règle même du canevas
/// ([`super::scale::on_pixel_grid`]) : une seconde règle a déjà décalé d'un pixel chaque zone du
/// SVG (fiche 71).
pub(crate) fn sur_la_grille_des_pixels(
    (x, y, l, h): (f32, f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let r = super::scale::on_pixel_grid(tiny_skia::Rect::from_xywh(x, y, l, h)?)?;
    Some((r.x(), r.y(), r.width(), r.height()))
}

impl Vectoriel for Svg {
    fn rectangle(&mut self, boite: (f32, f32, f32, f32), rayon: f32, c: Color) {
        let (teinte, a) = couleur(c);
        Svg::rectangle(self, boite, rayon, (&teinte, a));
    }

    fn rectangle_net(&mut self, boite: (f32, f32, f32, f32), c: Color) {
        if let Some(r) = sur_la_grille_des_pixels(boite) {
            Vectoriel::rectangle(self, r, 0.0, c);
        }
    }

    fn anneau(
        &mut self,
        (x, y, l, h): (f32, f32, f32, f32),
        rayon: f32,
        (decalage, demi): (f32, f32),
        c: Color,
    ) {
        // Le contour d'un rectangle arrondi repoussé de `decalage` est exact (`Arrondi::dilate`) ;
        // le trait de SVG s'étend de part et d'autre de lui.
        let (bx, by, bl, bh) = (
            x - decalage,
            y - decalage,
            l + 2.0 * decalage,
            h + 2.0 * decalage,
        );
        let (teinte, a) = couleur(c);
        self.poser(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="none" stroke="{teinte}" stroke-opacity="{}" stroke-width="{}"/>"#,
            n(bx),
            n(by),
            n(bl),
            n(bh),
            n(rayon_tenu(rayon + decalage, (bl, bh))),
            n(a),
            n(2.0 * demi)
        ));
    }

    fn disque(&mut self, (cx, cy): (f32, f32), rayon: f32, c: Color) {
        let u = c.to_color_u8();
        self.poser(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}" fill-opacity="{}"/>"#,
            n(cx),
            n(cy),
            n(rayon),
            hex((u.red(), u.green(), u.blue())),
            n(c.alpha())
        ));
    }

    fn groupe(&mut self, mots: &str, encre: Color) {
        let (c, a) = couleur(encre);
        let mut groupe = format!(r#"<g fill="{c}""#);
        if a < 1.0 {
            let _ = write!(groupe, r#" fill-opacity="{}""#, n(a));
        }
        let _ = write!(groupe, r#" aria-label="{}">"#, echapper(mots));
        self.poser(&groupe);
    }

    fn fin_du_groupe(&mut self) {
        self.poser("</g>");
    }

    fn glyphe(&mut self, glyphe: (Police, u16), plume: (f32, f32), corps: f32) {
        Svg::glyphe(self, glyphe, plume, corps);
    }

    fn forme(
        &mut self,
        forme: &Forme,
        boite: (f32, f32, f32, f32),
        trait_px: Option<f32>,
        c: Color,
    ) {
        let (teinte, a) = couleur(c);
        super::math::svg::forme_en_svg(self, forme, boite, ((&teinte, a), trait_px));
    }
}
