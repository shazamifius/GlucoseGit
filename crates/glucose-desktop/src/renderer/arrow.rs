//! La flèche : un trait et une pointe, pour l'instant.
//!
//! La fiche 06 § 7 en attend bien plus — trois passes, un halo en dégradé de la teinte de la
//! source à celle de la cible, une pointe en disque, des étiquettes — et c'est le chantier des
//! flèches (fiche 12, 2.B). Ce qui est déjà tenu : la pointe est de la géométrie du monde,
//! elle grandit avec la flèche sans borne (SCALE-1) ; la sélection est une affordance et
//! garde son épaisseur écran.

use super::card::{Clip, Pass, SELECTION_RING};
use crate::canvas::world_to_screen;
use tiny_skia::{LineCap, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Longueur des barbes de la pointe d'une flèche, en unités monde.
pub(super) const ARROW_HEAD: f32 = 12.0;
/// Ouverture des barbes, en radians.
const ARROW_ANGLE: f32 = 0.45;
/// Épaisseur du trait d'une flèche, en unités monde.
const ARROW_STROKE: f32 = 1.8;
/// Marge de sécurité du test de visibilité d'une flèche, en pixels écran.
const ARROW_MARGIN: f32 = 16.0;

pub(super) fn draw_arrow(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    from: (f64, f64),
    to: (f64, f64),
    selected: bool,
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

    let mut pb = PathBuilder::new();
    pb.move_to(x1, y1);
    pb.line_to(x2, y2);

    // La pointe est de la géométrie du monde : elle grandit avec la flèche, sans borne.
    let head = ctx.scale.world(ARROW_HEAD);
    let angle = (y2 - y1).atan2(x2 - x1);
    for side in [-ARROW_ANGLE, ARROW_ANGLE] {
        pb.move_to(x2, y2);
        pb.line_to(
            x2 - head * (angle + side).cos(),
            y2 - head * (angle + side).sin(),
        );
    }

    let Some(path) = pb.finish() else {
        return;
    };
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
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
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
