//! **La surface** (P5, fiche 70) : un dessin écrit **une fois**, posé sur les pixels du canevas
//! ou écrit en SVG.
//!
//! # Pourquoi, après les sœurs de la fiche 69
//!
//! La fiche 69 a donné à chaque dessin une sœur vectorielle : la même mise en page, mais les
//! appels de dessin écrits deux fois — une couleur, une opacité ou un ordre peuvent diverger
//! sans que la mise en page partagée le voie. Cairo dessine le même code sur une image, un PDF
//! ou un SVG ; Skia a son `SkSVGCanvas`. Ici, un dessin qui n'emploie que ces quatre gestes —
//! remplir un tracé, tracer un contour, remplir un rectangle net, écrire — se rend générique sur
//! une [`Surface`] (le mot de Cairo), et le SVG reçoit **le même objet tracé** que le canevas
//! (`tiny_skia::Path`) : la même géométrie, au point près. Les dessins qui composent leurs
//! pixels à la main (le texte riche, les lueurs en bandes, les photos) gardent leur sœur.
//!
//! La voie des pixels fait exactement les appels d'avant : rien ne change à l'écran, et la
//! généricité se résout à la compilation.

use super::svg::{couleur, n, Svg};
use crate::typography::{TextStyle, Typography};
use std::fmt::Write as _;
use tiny_skia::{Color, LineCap, Paint, Path, PathSegment, PixmapMut, Rect, Stroke, Transform};

/// **Un trait de contour** : sa largeur, ses tirets (trait, vide), son bout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Trait {
    pub largeur: f32,
    pub tirets: Option<(f32, f32)>,
    pub bout: LineCap,
}

impl Trait {
    /// Un trait plein, aux bouts francs — le trait par défaut de `tiny-skia` et de SVG.
    pub fn plein(largeur: f32) -> Self {
        Self {
            largeur,
            tirets: None,
            bout: LineCap::Butt,
        }
    }
}

/// **Ce qui reçoit un dessin** : les pixels du canevas, ou le SVG d'un export.
pub(crate) trait Surface {
    /// Remplit ce tracé, anticrénelé, selon la règle des enroulements.
    fn remplir(&mut self, chemin: &Path, couleur: Color);
    /// Trace le contour de ce tracé.
    fn tracer(&mut self, chemin: &Path, couleur: Color, trait_: Trait);
    /// Remplit ce rectangle aux pixels entiers, sans anticrénelage ([`super::scale::fill_crisp`]).
    fn remplir_net(&mut self, rect: Rect, couleur: Color);
    /// Écrit ce texte, son coin haut gauche en `at`.
    fn ecrire(&mut self, at: (f32, f32), texte: &str, style: TextStyle);
    /// La typographie qui mesure ce qu'on écrit.
    fn typographie(&self) -> &Typography;
}

/// **Les pixels du canevas.**
pub(crate) struct Pixels<'a, 'p> {
    pub pixmap: &'a mut PixmapMut<'p>,
    pub typographie: &'a Typography,
}

fn peinture(couleur: Color) -> Paint<'static> {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(couleur);
    paint
}

impl Surface for Pixels<'_, '_> {
    fn remplir(&mut self, chemin: &Path, couleur: Color) {
        let paint = peinture(couleur);
        let regle = tiny_skia::FillRule::Winding;
        self.pixmap
            .fill_path(chemin, &paint, regle, Transform::identity(), None);
    }

    fn tracer(&mut self, chemin: &Path, couleur: Color, trait_: Trait) {
        let stroke = Stroke {
            width: trait_.largeur,
            dash: trait_
                .tirets
                .and_then(|(t, v)| tiny_skia::StrokeDash::new(vec![t, v], 0.0)),
            line_cap: trait_.bout,
            ..Default::default()
        };
        let paint = peinture(couleur);
        self.pixmap
            .stroke_path(chemin, &paint, &stroke, Transform::identity(), None);
    }

    fn remplir_net(&mut self, rect: Rect, couleur: Color) {
        super::scale::fill_crisp(self.pixmap, rect, couleur);
    }

    fn ecrire(&mut self, (x, y): (f32, f32), texte: &str, style: TextStyle) {
        self.typographie.draw_text(self.pixmap, texte, x, y, style);
    }

    fn typographie(&self) -> &Typography {
        self.typographie
    }
}

/// **Le SVG d'un export.**
pub(crate) struct Vecteurs<'a> {
    pub svg: &'a mut Svg,
    pub typographie: &'a Typography,
}

/// Les commandes d'un tracé, telles que SVG les écrit.
pub(crate) fn en_d(chemin: &Path) -> String {
    let mut d = String::new();
    for s in chemin.segments() {
        let _ = match s {
            PathSegment::MoveTo(p) => write!(d, "M{} {}", n(p.x), n(p.y)),
            PathSegment::LineTo(p) => write!(d, "L{} {}", n(p.x), n(p.y)),
            PathSegment::QuadTo(a, p) => {
                write!(d, "Q{} {} {} {}", n(a.x), n(a.y), n(p.x), n(p.y))
            }
            PathSegment::CubicTo(a, b, p) => write!(
                d,
                "C{} {} {} {} {} {}",
                n(a.x),
                n(a.y),
                n(b.x),
                n(b.y),
                n(p.x),
                n(p.y)
            ),
            PathSegment::Close => write!(d, "Z"),
        };
    }
    d
}

impl Surface for Vecteurs<'_> {
    fn remplir(&mut self, chemin: &Path, c: Color) {
        let (teinte, a) = couleur(c);
        self.svg.poser(&format!(
            r#"<path d="{}" fill="{teinte}" fill-opacity="{}"/>"#,
            en_d(chemin),
            n(a)
        ));
    }

    fn tracer(&mut self, chemin: &Path, c: Color, trait_: Trait) {
        let (teinte, a) = couleur(c);
        let tirets = trait_.tirets.map_or(String::new(), |(t, v)| {
            format!(r#" stroke-dasharray="{} {}""#, n(t), n(v))
        });
        let bout = match trait_.bout {
            LineCap::Butt => "butt",
            LineCap::Round => "round",
            LineCap::Square => "square",
        };
        self.svg.poser(&format!(
            r#"<path d="{}" fill="none" stroke="{teinte}" stroke-opacity="{}" stroke-width="{}" stroke-linecap="{bout}"{tirets}/>"#,
            en_d(chemin),
            n(a),
            n(trait_.largeur)
        ));
    }

    fn remplir_net(&mut self, rect: Rect, c: Color) {
        // Les pixels mêmes que `fill_crisp` couvre, par **sa** règle (`on_pixel_grid`). La
        // fiche 70 en écrivait une seconde — tronquer les bords — qui décalait d'un pixel
        // chaque zone et chaque case du SVG ; la superposition, à un pixel près, ne le voyait
        // pas (fiche 71).
        if let Some(r) = super::scale::on_pixel_grid(rect) {
            let (teinte, a) = couleur(c);
            self.svg
                .rectangle((r.x(), r.y(), r.width(), r.height()), 0.0, (&teinte, a));
        }
    }

    fn ecrire(&mut self, at: (f32, f32), texte: &str, style: TextStyle) {
        self.svg.texte(self.typographie, at, texte, style);
    }

    fn typographie(&self) -> &Typography {
        self.typographie
    }
}
