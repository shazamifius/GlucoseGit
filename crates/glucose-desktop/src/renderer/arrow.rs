//! La flèche : un trait et une pointe, pour l'instant.
//!
//! La fiche 06 § 7 en attend bien plus — trois passes, un halo en dégradé de la teinte de la
//! source à celle de la cible, une pointe en disque, des étiquettes — et c'est le chantier des
//! flèches (fiche 12, 2.B). Ce qui est déjà tenu : la pointe est de la géométrie du monde,
//! elle grandit avec la flèche sans borne (SCALE-1) ; la sélection est une affordance et
//! garde son épaisseur écran.

use super::pass::{Clip, Pass, SELECTION_RING};
use crate::canvas::world_to_screen;
use tiny_skia::{LineCap, Paint, PathBuilder, PixmapMut, Stroke, StrokeDash, Transform};

/// Longueur des barbes de la pointe d'une flèche, en unités monde.
pub(super) const ARROW_HEAD: f32 = 12.0;
/// Ouverture des barbes, en radians.
const ARROW_ANGLE: f32 = 0.45;
/// Épaisseur du trait d'une flèche, en unités monde.
const ARROW_STROKE: f32 = 1.8;
/// Marge de sécurité du test de visibilité d'une flèche, en pixels écran.
const ARROW_MARGIN: f32 = 16.0;
/// **Le motif d'un lien trans-domaine** : un tiret, un vide, en unités monde (fiche 03 § 11.6).
///
/// C'est celui de la référence — `strokeDasharray="6 4"` — pour un trait de deux pixels, à peu
/// près le nôtre. En unités monde, comme le trait lui-même (SCALE-1) : le motif reste
/// proportionné à l'épaisseur qu'il découpe, à tout zoom.
const TIRET: [f32; 2] = [6.0, 4.0];

/// Un tronçon de flèche. `tirets` porte, pour un lien trans-domaine, la longueur écran déjà
/// parcourue sur la polyligne : le motif reprend là où le tronçon précédent l'a laissé.
pub(super) fn draw_arrow(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (from, to): ((f64, f64), (f64, f64)),
    // Ce tronçon porte-t-il la pointe ? Une polyligne n'en a qu'une, à son dernier segment.
    (selected, tip): (bool, bool),
    tirets: Option<f32>,
) {
    let (ax, ay) = world_to_screen(from.0, from.1, &ctx.vp);
    let (bx, by) = world_to_screen(to.0, to.1, &ctx.vp);
    let (x1, y1, x2, y2) = (ax as f32, ay as f32, bx as f32, by as f32);

    let left = x1.min(x2) - ARROW_MARGIN;
    let top = y1.min(y2) - ARROW_MARGIN;
    let span_x = (x1 - x2).abs() + ARROW_MARGIN * 2.0;
    let span_y = (y1 - y2).abs() + ARROW_MARGIN * 2.0;
    if arrow_is_off_screen(ctx.clip, (left, top, span_x, span_y)) {
        return;
    }

    let mut tige = PathBuilder::new();
    tige.move_to(x1, y1);
    tige.line_to(x2, y2);

    // La pointe est de la géométrie du monde : elle grandit avec la flèche, sans borne.
    // Une polyligne n'en porte qu'une, à son dernier tronçon : les coudes sont des passages,
    // pas des arrivées.
    //
    // Sous des tirets, elle se trace à part et toujours pleine : des barbes en pointillés ne
    // se liraient plus comme une pointe. Sinon, elle reste dans le MÊME tracé que la tige :
    // deux traits anticrénelés qui se rejoignent poseraient deux fois leur jonction, et la
    // scène témoin l'a vu au pixel près.
    let mut separee = PathBuilder::new();
    if tip {
        let pb = if tirets.is_some() {
            &mut separee
        } else {
            &mut tige
        };
        tracer_la_pointe(pb, ctx.scale.world(ARROW_HEAD), (x1, y1), (x2, y2));
    }

    let (paint, stroke) = crayon(ctx, selected);
    if let Some(pointe) = separee.finish() {
        pixmap.stroke_path(&pointe, &paint, &stroke, Transform::identity(), None);
    }
    let Some(tige) = tige.finish() else {
        return;
    };
    let motif = tirets.and_then(|parcouru| {
        StrokeDash::new(
            TIRET.iter().map(|&l| ctx.scale.world(l)).collect(),
            parcouru,
        )
    });
    let stroke = Stroke {
        dash: motif,
        ..stroke
    };
    pixmap.stroke_path(&tige, &paint, &stroke, Transform::identity(), None);
}

/// Les deux barbes d'une pointe posée en `(x2, y2)`, longues de `longueur` pixels, dans l'axe
/// qui vient de `(x1, y1)`.
fn tracer_la_pointe(
    pb: &mut PathBuilder,
    longueur: f32,
    (x1, y1): (f32, f32),
    (x2, y2): (f32, f32),
) {
    let angle = (y2 - y1).atan2(x2 - x1);
    for side in [-ARROW_ANGLE, ARROW_ANGLE] {
        pb.move_to(x2, y2);
        pb.line_to(
            x2 - longueur * (angle + side).cos(),
            y2 - longueur * (angle + side).sin(),
        );
    }
}

/// La couleur et le trait d'une flèche : ceux de la sélection, qui gardent une épaisseur écran
/// (une affordance), ou les siens, à l'échelle du monde (SCALE-1).
fn crayon(ctx: &Pass, selected: bool) -> (Paint<'static>, Stroke) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(if selected {
        ctx.theme.selection_frame
    } else {
        ctx.theme.arrow_default
    });
    let stroke = Stroke {
        width: if selected {
            ctx.scale.screen(SELECTION_RING)
        } else {
            ctx.scale.world(ARROW_STROKE)
        },
        line_cap: LineCap::Round,
        ..Default::default()
    };
    (paint, stroke)
}

/// Une flèche n'a pas de boîte : son test de visibilité porte sur l'enveloppe de ses deux
/// extrémités, et ne peut donc pas réutiliser le rejet « trop petite » de [`Clip`].
fn arrow_is_off_screen(clip: Clip, bounds: (f32, f32, f32, f32)) -> bool {
    let (x, y, w, h) = bounds;
    x + w < 0.0 || x > clip.width || y + h < clip.top || y > clip.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::scale::WorldScale;

    #[test]
    fn test_an_arrow_head_follows_the_zoom_without_a_bound() {
        // Avant R-45 la pointe etait `clamp(6, 16)` : elle cessait de grandir a 1,33 et de
        // retrecir a 0,5. Le rapport pointe/longueur doit rester constant.
        let mut previous: Option<f32> = None;
        for zoom in [0.1_f64, 0.5, 1.0, 4.0, 40.0] {
            let s = WorldScale::new(zoom);
            let ratio = s.world(ARROW_HEAD) / s.world(100.0);
            if let Some(p) = previous {
                assert!((p - ratio).abs() < 1e-6, "zoom {zoom} : {ratio} != {p}");
            }
            previous = Some(ratio);
        }
    }
}
