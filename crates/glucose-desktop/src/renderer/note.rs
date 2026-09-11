//! Les deux autres annotations : le pense-bête et la flèche.
//!
//! Elles suivent la même règle que la carte (SCALE-1) : leur géométrie est décrite en
//! unités monde, puis [`StickyLayout::scaled`] — ou, pour la flèche, sa seule longueur de
//! pointe — reçoit l'unique mise à l'échelle. Avant R-45, le pense-bête bornait sa police à
//! `clamp(8, 20)`, son rayon à `clamp(2, 6)` et sa marge haute à `clamp(4, 10)`, pendant
//! que sa marge gauche et le pas de son badge d'opérateur ne suivaient pas le zoom **du
//! tout** : trois seuils et deux constantes écran pour une seule forme.

use super::card::{Clip, Pass, SELECTION_RING};
use super::scale::WorldScale;
use super::{parse_hex_color, push_rounded_rect, TextEditSession};
use crate::canvas::world_to_screen;
use crate::typography::{TextStyle, Typography};
use glucose_core::types::{Annotation, StickyOperator};
use tiny_skia::{Color, LineCap, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Largeur par défaut d'un pense-bête, en unités monde.
const STICKY_WIDTH: f64 = 160.0;
/// Hauteur par défaut d'un pense-bête, en unités monde.
const STICKY_HEIGHT: f64 = 120.0;
/// Corps de texte d'un pense-bête.
const STICKY_FONT: f32 = 12.0;
/// Interligne, en multiples du corps.
const STICKY_LINE_FACTOR: f32 = 1.3;
/// Marge intérieure, sur les quatre côtés.
const STICKY_PAD: f32 = 10.0;
/// Rayon des coins.
const STICKY_RADIUS: f32 = 6.0;
/// Corps du badge d'opérateur (`ET`, `OU`, `MAIS`, `PARCE QUE`).
const STICKY_OPERATOR_FONT: f32 = 11.0;
/// Hauteur consommée par le badge d'opérateur.
const STICKY_OPERATOR_ADVANCE: f32 = 16.0;
/// Épaisseur du cadre au repos.
const STICKY_BORDER: f32 = 1.0;

/// Longueur des barbes de la pointe d'une flèche, en unités monde.
const ARROW_HEAD: f32 = 12.0;
/// Ouverture des barbes, en radians.
const ARROW_ANGLE: f32 = 0.45;
/// Épaisseur du trait d'une flèche, en unités monde.
const ARROW_STROKE: f32 = 1.8;
/// Marge de sécurité du test de visibilité d'une flèche, en pixels écran.
const ARROW_MARGIN: f32 = 16.0;

/// Mise en page d'un pense-bête. **En unités monde tant que `scaled` n'a pas été appelée.**
#[derive(Clone, Copy, Debug, PartialEq)]
struct StickyLayout {
    width: f32,
    height: f32,
    font: f32,
    line_height: f32,
    pad: f32,
    radius: f32,
    operator_font: f32,
    operator_advance: f32,
    border: f32,
}

impl StickyLayout {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            font: STICKY_FONT,
            line_height: STICKY_FONT * STICKY_LINE_FACTOR,
            pad: STICKY_PAD,
            radius: STICKY_RADIUS,
            operator_font: STICKY_OPERATOR_FONT,
            operator_advance: STICKY_OPERATOR_ADVANCE,
            border: STICKY_BORDER,
        }
    }

    /// L'UNIQUE transformation d'échelle du pense-bête (SCALE-1).
    fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            font: s.world(self.font),
            line_height: s.world(self.line_height),
            pad: s.world(self.pad),
            radius: s.world(self.radius),
            operator_font: s.world(self.operator_font),
            operator_advance: s.world(self.operator_advance),
            border: s.world(self.border),
        }
    }
}

pub(super) fn draw_sticky(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    ann: &Annotation,
    selected: bool,
    editing: Option<&TextEditSession>,
) {
    let Annotation::Sticky { x, y, width, height, text, color, bg_color, operator, .. } = ann else {
        return;
    };
    let layout = StickyLayout::new(
        width.unwrap_or(STICKY_WIDTH) as f32,
        height.unwrap_or(STICKY_HEIGHT) as f32,
    )
    .scaled(ctx.scale);

    let (wx, wy) = world_to_screen(*x, *y, &ctx.vp);
    let (sx, sy) = (wx as f32, wy as f32);
    if ctx.clip.rejects(sx, sy, layout.width, layout.height) {
        return;
    }

    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, sx, sy, layout.width, layout.height, layout.radius);
    let Some(path) = pb.finish() else {
        return;
    };

    let (bg_r, bg_g, bg_b) = bg_color
        .as_deref()
        .map(|c| parse_hex_color(c, 254, 240, 138))
        .unwrap_or((254, 240, 138));
    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(Color::from_rgba8(bg_r, bg_g, bg_b, 245));
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

    let highlighted = selected || editing.is_some();
    let mut border = Paint { anti_alias: true, ..Default::default() };
    border.set_color(if highlighted {
        Color::from_rgba8(56, 189, 248, 255)
    } else {
        Color::from_rgba8(202, 138, 4, 180)
    });
    let stroke = Stroke {
        // Même partage que la carte : le cadre appartient au monde, l'anneau de sélection
        // est une affordance et garde sa taille écran (exception SCALE-1).
        width: if highlighted { ctx.scale.screen(SELECTION_RING) } else { layout.border },
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, Transform::identity(), None);

    // SCALE-2 — l'unique niveau de détail : sous le seuil, le pense-bête s'arrête là.
    if !ctx.scale.draws_detail() {
        return;
    }
    let content = editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str());
    let ink = color.as_deref().map(|c| parse_hex_color(c, 28, 25, 23)).unwrap_or((28, 25, 23));
    draw_sticky_text(ctx.typography, pixmap, (sx, sy), &layout, StickyText { content, ink, operator });
    if editing.map(|s| (s.blink_timer.elapsed().as_millis() / 500) % 2 == 0).unwrap_or(false) {
        draw_sticky_cursor(ctx, pixmap, (sx, sy), &layout, content);
    }
}

/// Ce qu'un pense-bête a à écrire.
struct StickyText<'a> {
    content: &'a str,
    ink: (u8, u8, u8),
    operator: &'a Option<StickyOperator>,
}

fn draw_sticky_text(
    typography: &Typography,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    body: StickyText<'_>,
) {
    let mut cur_y = at.1 + layout.pad;
    if let Some(op) = body.operator {
        let label = match op {
            StickyOperator::And => "ET",
            StickyOperator::Or => "OU",
            StickyOperator::But => "MAIS",
            StickyOperator::Because => "PARCE QUE",
        };
        let style = TextStyle {
            size: layout.operator_font,
            color: Color::from_rgba8(161, 98, 7, 255),
            bold: true,
        };
        typography.draw_text(pixmap, label, at.0 + layout.pad, cur_y, style);
        cur_y += layout.operator_advance;
    }

    let style = TextStyle {
        size: layout.font,
        color: Color::from_rgba8(body.ink.0, body.ink.1, body.ink.2, 255),
        bold: false,
    };
    for line in body.content.lines() {
        typography.draw_text(pixmap, line, at.0 + layout.pad, cur_y, style);
        cur_y += layout.line_height;
    }
}

fn draw_sticky_cursor(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    content: &str,
) {
    let (measured, _) = ctx.typography.measure_text(content, layout.font, false);
    let cx = (at.0 + layout.pad + measured).min(at.0 + layout.width - layout.pad * 0.6);
    let cy = at.1 + layout.pad + layout.operator_advance;
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(28, 25, 23, 255));
    if let Some(rect) = Rect::from_xywh(cx, cy, ctx.scale.screen(SELECTION_RING), layout.font * 1.2) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

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
        pb.line_to(x2 - head * (angle + side).cos(), y2 - head * (angle + side).sin());
    }

    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(if selected {
        Color::from_rgba8(56, 189, 248, 255)
    } else {
        Color::from_rgba8(148, 163, 184, 220)
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

    #[test]
    fn test_scale_1_a_sticky_is_self_similar_at_every_zoom() {
        let world = StickyLayout::new(160.0, 120.0);
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
            let screen = world.scaled(WorldScale::new(zoom));
            for (on_screen, in_world) in [
                (screen.font, world.font),
                (screen.pad, world.pad),
                (screen.radius, world.radius),
                (screen.operator_font, world.operator_font),
                (screen.operator_advance, world.operator_advance),
                (screen.border, world.border),
            ] {
                let expected = in_world / world.width;
                let observed = on_screen / screen.width;
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : rapport {observed} au lieu de {expected}"
                );
            }
        }
    }

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
