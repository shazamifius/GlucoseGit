//! **Une formule en SVG** (P5, fiche 70) : la sœur de [`MathRenderer::draw`] — la même mise en
//! page de [`glucose_math`], posée à la même plume. Ses caractères sont les glyphes mêmes des
//! fontes de KaTeX, en tracés ([`crate::renderer::svg`]) ; ses filets, la boîte que le tracé
//! remplit ([`boite_du_filet`]) ; ses formes étirables, le chemin de KaTeX dans sa boîte, rogné
//! par elle comme le `overflow: hidden` de KaTeX.
//!
//! La fiche 69 écrivait la **source** d'une formule à sa place : le moteur ne livrait que des
//! pixels. Ce qu'il livre en réalité, c'est une géométrie — des glyphes nommés, des filets, des
//! chemins — : le SVG l'écrit telle quelle.

use super::{boite_du_filet, nom_de_fonte, MathRenderer, FONTS};
use crate::params::Pen;
use crate::renderer::svg::{couleur, echapper, n, Svg};
use crate::typography::Police;
use glucose_math::layout::MathItem;
use glucose_math::{Commande, Family, Forme, Mode, Style};
use std::fmt::Write as _;
use tiny_skia::Color;

impl MathRenderer {
    /// **Écrit une formule** dont la ligne de base commence à la plume — `false` si la source
    /// est fausse, comme [`Self::draw`] : l'appelant montre alors la source, en rouge.
    pub fn en_svg(&self, svg: &mut Svg, source: &str, mode: Mode, pen: Pen, color: Color) -> bool {
        let Ok(layout) = self.layout(source, mode) else {
            return false;
        };
        let Pen { x, y, font_size } = pen;
        let (c, a) = couleur(color);
        svg.poser(&format!(
            r#"<g fill="{c}" fill-opacity="{}" aria-label="{}">"#,
            n(a),
            echapper(source)
        ));
        for item in &layout.items {
            match item {
                MathItem::Glyph {
                    text,
                    x: gx,
                    y: gy,
                    size,
                    family,
                    style,
                } => {
                    let plume = Pen {
                        x: x + *gx as f32 * font_size,
                        y: y - *gy as f32 * font_size,
                        font_size: *size as f32 * font_size,
                    };
                    self.glyphes_en_svg(svg, text, (*family, *style), plume);
                }
                MathItem::Rule {
                    x: rx,
                    y: ry,
                    width,
                    height,
                } => {
                    let boite = boite_du_filet((*rx, *ry, *width, *height), pen);
                    svg.rectangle(boite, 0.0, (&c, a));
                }
                MathItem::Path {
                    x: px,
                    y: py,
                    width,
                    height,
                    forme: Some(forme),
                    ..
                } => {
                    let (w, h) = (*width as f32 * font_size, *height as f32 * font_size);
                    let bas = y - *py as f32 * font_size;
                    let boite = (x + *px as f32 * font_size, bas - h, w, h);
                    let trait_px = forme.epaisseur.map(|e| e as f32 * font_size);
                    forme_en_svg(svg, forme, boite, ((&c, a), trait_px));
                }
                MathItem::Path { forme: None, .. } => {}
            }
        }
        svg.poser("</g>");
        true
    }

    /// Les caractères d'un élément, dans la fonte de KaTeX qui les dessine, la plume avancée
    /// comme [`super::dessine_glyphe`] l'avance.
    fn glyphes_en_svg(&self, svg: &mut Svg, texte: &str, (f, s): (Family, Style), pen: Pen) {
        let nom = nom_de_fonte(f, s);
        let Some((&nom, font)) = self.fonts.get_key_value(nom.as_str()) else {
            return;
        };
        let Some(&(_, octets)) = FONTS.iter().find(|(n, _)| *n == nom) else {
            return;
        };
        let Pen {
            x,
            y,
            font_size: taille,
        } = pen;
        if taille < 0.5 {
            return;
        }
        let police = Police { nom, octets };
        let mut plume = x;
        for c in texte.chars() {
            svg.glyphe((police, font.lookup_glyph_index(c)), (plume, y), taille);
            plume += font.metrics(c, taille).advance_width;
        }
    }
}

/// **Une forme de KaTeX dans sa boîte** `(x, y, largeur, hauteur)`, rognée par elle ; remplie,
/// ou tracée à l'épaisseur qu'elle porte (en pixels : elle ne s'étire pas avec la boîte).
fn forme_en_svg(
    svg: &mut Svg,
    forme: &Forme,
    (bx, by, bl, bh): (f32, f32, f32, f32),
    ((teinte, alpha), trait_px): ((&str, f32), Option<f32>),
) {
    let (sx, sy, dx, dy) = forme.transformation((f64::from(bl), f64::from(bh)));
    let point = |x: f64, y: f64| {
        let (px, py) = (f64::from(bx) + dx + x * sx, f64::from(by) + dy + y * sy);
        format!("{} {}", n(px as f32), n(py as f32))
    };
    let mut d = String::new();
    for c in &forme.commandes {
        let _ = match *c {
            Commande::Aller(x, y) => write!(d, "M{}", point(x, y)),
            Commande::Ligne(x, y) => write!(d, "L{}", point(x, y)),
            Commande::Cubique(a, b, c2, e, f, g) => {
                write!(d, "C{} {} {}", point(a, b), point(c2, e), point(f, g))
            }
            Commande::Fermer => write!(d, "Z"),
        };
    }
    let rogne = svg.nom("forme");
    svg.definir(&format!(
        r#"<clipPath id="{rogne}"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath>"#,
        n(bx),
        n(by),
        n(bl),
        n(bh)
    ));
    let peinture = match trait_px {
        Some(l) => format!(
            r#"fill="none" stroke="{teinte}" stroke-opacity="{}" stroke-width="{}""#,
            n(alpha),
            n(l)
        ),
        None => String::new(),
    };
    svg.poser(&format!(
        r#"<path clip-path="url(#{rogne})" d="{d}" {peinture}/>"#
    ));
}
