//! Le pense-bête : un papier — ou une pilule, quand il porte un opérateur logique.
//!
//! Il suit la même règle que la carte (SCALE-1) : sa géométrie est décrite en unités monde,
//! puis [`StickyLayout::scaled`] reçoit l'unique mise à l'échelle. Avant R-45, il bornait sa
//! police à `clamp(8, 20)`, son rayon à `clamp(2, 6)` et sa marge haute à `clamp(4, 10)` :
//! trois seuils pour une seule forme.
//!
//! # Les couleurs viennent du thème, et du document
//!
//! Le papier est `theme.sticky_yellow_bg` — `#f5c542`, celui de la référence — sauf si le
//! document en fixe un autre ; l'encre, `theme.sticky_yellow_text`. Le rendu prenait un jaune
//! pastel, un cadre ocre et un anneau de sélection cyan qu'aucune fiche ne connaît (§ 4.6).
//!
//! Ce que la référence fait et que ce rendu ne fait pas encore : le **flou** de l'ombre
//! (`0 4px 6px`) — l'ombre est portée nette, au même décalage — et le halo coloré de la
//! pilule (`0 0 18px`). Un flou est une passe de rastérisation à part entière.
//!
//! # STICKY-FIT-1 — un pense-bête est un papier de taille fixe
//!
//! À l'inverse de la carte (TEXT-FIT-1), le pense-bête se tire librement dans les deux
//! sens : c'est un post-it, l'utilisateur choisit le papier. Son texte reflue à sa largeur
//! (WRAP-1) et **se coupe** en bas quand il ne tient plus — jamais de débordement, jamais
//! de hauteur qui bouge toute seule. Le choix est nommé pour ne pas être un mélange silencieux.

use super::card::{Pass, SELECTION_RING};
use super::handles::draw_resize_handles;
use super::scale::WorldScale;
use super::wrap::wrap_paragraph;
use super::{parse_hex_rgb, push_rounded_rect, TextEditSession};
use crate::canvas::world_to_screen;
use crate::theme::operator_color;
use crate::typography::{TextStyle, Typography};
use glucose_core::resize::Handle;
use glucose_core::types::{Annotation, StickyOperator};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Corps de texte d'un pense-bête (fiche 06 § 5.2 : 13 px).
const STICKY_FONT: f32 = 13.0;
/// Interligne, en multiples du corps.
const STICKY_LINE_FACTOR: f32 = 1.3;
/// Marge intérieure, sur les quatre côtés.
const STICKY_PAD: f32 = 10.0;
/// Rayon des coins (fiche 06 § 5.2 : 2 px).
const STICKY_RADIUS: f32 = 2.0;
/// Décalage vertical de l'ombre portée du papier (référence : `0 4px 6px`).
const STICKY_SHADOW_OFFSET: f32 = 4.0;

/// Bord de la pilule d'un opérateur (fiche 06 § 5.3 : 1,5 px).
const PILL_BORDER: f32 = 1.5;
/// Corps du mot de la pilule, en gras.
const PILL_FONT: f32 = 13.0;
/// Le fond de la pilule : la couleur de l'opérateur à `0x20` — `${color}20` dans la référence.
const PILL_FILL_ALPHA: u8 = 0x20;

/// Mise en page d'un pense-bête. **En unités monde tant que `scaled` n'a pas été appelée.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickyLayout {
    width: f32,
    height: f32,
    font: f32,
    line_height: f32,
    pad: f32,
    radius: f32,
    shadow_offset: f32,
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
            shadow_offset: STICKY_SHADOW_OFFSET,
        }
    }

    /// L'UNIQUE transformation d'échelle du pense-bête (SCALE-1).
    pub fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            font: s.world(self.font),
            line_height: s.world(self.line_height),
            pad: s.world(self.pad),
            radius: s.world(self.radius),
            shadow_offset: s.world(self.shadow_offset),
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
    let Annotation::Sticky {
        x,
        y,
        text,
        color,
        bg_color,
        operator,
        ..
    } = ann
    else {
        return;
    };
    let (w, h) = ann
        .size()
        .expect("Annotation::size ne rend None que pour une flèche");
    let layout = StickyLayout::new(w as f32, h as f32).scaled(ctx.scale);
    let (wx, wy) = world_to_screen(*x, *y, &ctx.vp);
    let at = (wx as f32, wy as f32);
    if ctx.clip.rejects(at.0, at.1, layout.width, layout.height) {
        return;
    }

    let highlighted = selected || editing.is_some();
    match operator {
        Some(op) => draw_pill(ctx, pixmap, at, &layout, *op, highlighted),
        None => {
            let paper = Paper {
                bg: document_color(bg_color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_bg),
                ink: document_color(color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_text),
                text: editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str()),
                world_width: w as f32,
                cursor: editing
                    .filter(|s| (s.blink_timer.elapsed().as_millis() / 500) % 2 == 0)
                    .map(|s| s.cursor_idx),
            };
            draw_paper(ctx, pixmap, at, &layout, &paper, highlighted);
        }
    }

    if selected {
        draw_resize_handles(
            pixmap,
            ctx.theme,
            ctx.scale,
            (at.0, at.1, layout.width, layout.height),
            &Handle::ALL,
        );
    }
}

/// Ce qu'un papier a à montrer.
struct Paper<'a> {
    bg: Color,
    ink: Color,
    text: &'a str,
    /// La largeur du papier en unités monde : le reflux se calcule avant la mise à l'échelle.
    world_width: f32,
    /// Position du curseur d'édition à dessiner, si la session clignote « allumé ».
    cursor: Option<usize>,
}

/// Le papier : son ombre, son fond, son anneau s'il est sélectionné, puis son texte.
fn draw_paper(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    paper: &Paper<'_>,
    highlighted: bool,
) {
    let Some(path) = rounded_path(at, layout.width, layout.height, layout.radius) else {
        return;
    };
    // L'ombre : le même papier, décalé vers le bas, en noir à 0,30 — sans flou pour l'instant.
    if let Some(shadow) = rounded_path(
        (at.0, at.1 + layout.shadow_offset),
        layout.width,
        layout.height,
        layout.radius,
    ) {
        fill(pixmap, &shadow, ctx.theme.sticky_shadow);
    }
    fill(pixmap, &path, paper.bg);
    if highlighted {
        ring(ctx, pixmap, &path);
    }

    // SCALE-2 — l'unique niveau de détail : sous le seuil, le pense-bête s'arrête là.
    if !ctx.scale.draws_detail() {
        return;
    }
    // Le reflux se calcule en unités monde, avant la mise à l'échelle (WRAP-1).
    let lines = sticky_lines(ctx.typography, paper.text, paper.world_width);
    draw_paper_text(ctx, pixmap, at, layout, paper, &lines);
}

/// La pilule d'un opérateur logique (fiche 06 § 5.3) : coins en demi-cercle, fond à sa
/// couleur à 12 %, bord de 1,5 à sa couleur, et son mot centré en gras.
fn draw_pill(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    op: StickyOperator,
    highlighted: bool,
) {
    let color = operator_color(op);
    let radius = layout.height / 2.0;
    let Some(path) = rounded_path(at, layout.width, layout.height, radius) else {
        return;
    };
    let c = color.to_color_u8();
    fill(
        pixmap,
        &path,
        Color::from_rgba8(c.red(), c.green(), c.blue(), PILL_FILL_ALPHA),
    );
    stroke(pixmap, &path, color, ctx.scale.world(PILL_BORDER));
    if highlighted {
        ring(ctx, pixmap, &path);
    }
    if !ctx.scale.draws_detail() {
        return;
    }

    let font = ctx.scale.world(PILL_FONT);
    let label = op.label();
    let (text_w, line_h) = ctx.typography.measure_text(label, font, true);
    let style = TextStyle {
        size: font,
        color,
        bold: true,
    };
    ctx.typography.draw_text(
        pixmap,
        label,
        at.0 + (layout.width - text_w) / 2.0,
        at.1 + (layout.height - line_h) / 2.0,
        style,
    );
}

/// Une couleur que le document fixe, opaque — `None` s'il n'en fixe pas ou si elle est illisible.
fn document_color(hex: Option<&str>) -> Option<Color> {
    let (r, g, b) = parse_hex_rgb(hex?)?;
    Some(Color::from_rgba8(r, g, b, 255))
}

fn rounded_path(at: (f32, f32), w: f32, h: f32, radius: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, at.0, at.1, w, h, radius);
    pb.finish()
}

fn fill(pixmap: &mut PixmapMut, path: &tiny_skia::Path, color: Color) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(color);
    pixmap.fill_path(
        path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

fn stroke(pixmap: &mut PixmapMut, path: &tiny_skia::Path, color: Color, width: f32) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(color);
    let stroke = Stroke {
        width,
        ..Default::default()
    };
    pixmap.stroke_path(path, &paint, &stroke, Transform::identity(), None);
}

/// L'anneau de sélection : une affordance, qui garde sa taille écran (exception SCALE-1).
fn ring(ctx: &Pass, pixmap: &mut PixmapMut, path: &tiny_skia::Path) {
    stroke(
        pixmap,
        path,
        ctx.theme.selection_frame,
        ctx.scale.screen(SELECTION_RING),
    );
}

/// WRAP-1 — les lignes visuelles d'un pense-bête de `width` unités monde, en tranches
/// d'octets de `content`. Un pense-bête n'a pas de Markdown : chaque ligne source est un
/// paragraphe tel quel.
fn sticky_lines(typography: &Typography, content: &str, width: f32) -> Vec<(usize, usize)> {
    let usable = (width - STICKY_PAD * 2.0).max(STICKY_FONT);
    let advance = |ch: char| {
        typography
            .get_glyph(ch, STICKY_FONT, false)
            .metrics
            .advance_width
    };
    let mut lines = Vec::new();
    let mut offset = 0usize;
    for paragraph in content.split('\n') {
        for (s, e) in wrap_paragraph(paragraph, usable, advance) {
            lines.push((offset + s, offset + e));
        }
        offset += paragraph.len() + 1;
    }
    lines
}

fn draw_paper_text(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    paper: &Paper<'_>,
    lines: &[(usize, usize)],
) {
    let typography = ctx.typography;
    let style = TextStyle {
        size: layout.font,
        color: paper.ink,
        bold: false,
    };
    // STICKY-FIT-1 : le papier est fixe, une ligne qui ne tient plus n'est pas dessinée.
    let floor = at.1 + layout.height - layout.pad;
    let mut cur_y = at.1 + layout.pad;
    let mut cursor_drawn = false;
    for (num, &(start, end)) in lines.iter().enumerate() {
        if cur_y + layout.line_height > floor + 1e-3 {
            break;
        }
        typography.draw_text(
            pixmap,
            &paper.text[start..end],
            at.0 + layout.pad,
            cur_y,
            style,
        );
        let last = num + 1 == lines.len();
        if let Some(idx) = paper
            .cursor
            .filter(|&i| !cursor_drawn && i >= start && (i <= end || last))
        {
            let (prefix_w, _) = typography.measure_text(
                &paper.text[start..idx.clamp(start, end)],
                layout.font,
                false,
            );
            draw_cursor(
                ctx,
                pixmap,
                (at.0 + layout.pad + prefix_w, cur_y),
                layout,
                paper.ink,
            );
            cursor_drawn = true;
        }
        cur_y += layout.line_height;
    }
}

/// Le curseur d'édition, à l'encre du texte, épais d'un anneau écran.
fn draw_cursor(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    ink: Color,
) {
    let mut paint = Paint::default();
    paint.set_color(ink);
    if let Some(rect) = Rect::from_xywh(
        at.0,
        at.1,
        ctx.scale.screen(SELECTION_RING),
        layout.font * 1.2,
    ) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
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
                (screen.line_height, world.line_height),
                (screen.shadow_offset, world.shadow_offset),
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

    /// Fiche 06 § 5.2 — 160 × 120 par défaut, coins à 2 px, corps 13 px comme la référence
    /// (`ann.fontSize || 13`) ; § 5.3 — la pilule fait 44 de haut, 80 ou 130 de large.
    #[test]
    fn test_the_sticky_metrics_are_those_of_the_spec() {
        use glucose_core::types::{
            DEFAULT_OPERATOR_HEIGHT, DEFAULT_STICKY_HEIGHT, DEFAULT_STICKY_WIDTH,
        };
        assert_eq!(
            (DEFAULT_STICKY_WIDTH, DEFAULT_STICKY_HEIGHT),
            (160.0, 120.0)
        );
        assert_eq!(STICKY_RADIUS, 2.0);
        assert_eq!(STICKY_FONT, 13.0);
        assert_eq!(DEFAULT_OPERATOR_HEIGHT, 44.0);
        assert_eq!(StickyOperator::And.default_width(), 80.0);
        assert_eq!(StickyOperator::Because.default_width(), 130.0);
        assert_eq!(PILL_BORDER, 1.5);
    }
}
