//! Rendu des annotations : la carte de texte, et l'aiguillage vers les autres formes.
//!
//! # CARD-1 — la carte est décrite en unités monde, puis mise à l'échelle une seule fois
//!
//! [`CardLayout`] ne connaît pas le zoom : elle décrit une carte comme si elle était
//! dessinée à l'échelle 1 — sa boîte, sa police, ses marges, son rayon de coin, l'indentation
//! de ses puces et l'épaisseur de son cadre, tous dans la même unité. [`CardLayout::scaled`]
//! applique ensuite **le même facteur à tous ces champs d'un coup** (SCALE-1).
//!
//! C'est la seule façon d'obtenir une carte semblable à elle-même à tous les zooms, et c'est
//! ce que **R-45** reprochait au code précédent : la boîte s'y mettait à l'échelle librement
//! pendant que des `clamp` figeaient son contenu à six seuils différents. Mesuré sur la même
//! carte (`card/proof.rs`), cela donnait cinq cartes pour un seul document :
//!
//! | Zoom | Ce que la carte d'avant montrait |
//! |---:|---|
//! | 0,25 | **aucun texte** (police bornée à 8 px, sous le seuil de rendu) et une boîte 70 % trop haute |
//! | 0,5 | **aucun texte** non plus |
//! | 1 | fidèle — le seul zoom où aucune borne ne se déclenchait |
//! | 2 | texte figé à 24 px : 0,49 de la largeur au lieu de 0,60 |
//! | 4 | texte toujours figé : 0,25 de la largeur |
//!
//! Corollaire important : **la mise en page se calcule avant la mise à l'échelle**. Le
//! découpage en lignes ([`layout_lines`], WRAP-1) et la hauteur nécessaire au contenu
//! (`CardLayout::text_card`) sont dérivés d'une police en unités monde ; calculés après, ils
//! dépendraient d'une police écran et la carte se réorganiserait à chaque palier de zoom.
//!
//! # TEXT-FIT-1 — la hauteur d'une carte de texte suit son texte
//!
//! L'utilisateur choisit la **largeur** d'une carte ; sa hauteur est celle de son texte
//! reflué à cette largeur, ni plus ni moins. Une carte ne tronque jamais son contenu, et
//! n'offre donc pas de poignée verticale ([`Handle::HORIZONTAL`]). Le geste de
//! redimensionnement et la validation d'une saisie écrivent cette hauteur dans le document
//! ([`text_card_fit_height`]), pour que le test de clic, l'index spatial et les poignées
//! voient la même boîte que l'écran. `text_card` garde un `max` de sécurité pour les
//! documents antérieurs à cette règle.

use super::domain::{draw_domain_gauge, DomainTints};
use super::handles::draw_resize_handles;
use super::hue::SymbioticHueCache;
use super::note::{draw_arrow, draw_sticky};
use super::scale::WorldScale;
use super::wrap::wrap_paragraph;
use super::{parse_hex_color, push_rounded_rect, PaintKit, TextEditSession};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::halo::{DEFAULT_TEXT_CARD_HEIGHT, DEFAULT_TEXT_CARD_WIDTH};
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, DomainAssignment, Viewport};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

// ── Mesures d'une carte, en unités monde ────────────────────────────────────

/// Corps de texte d'une carte.
const BODY_FONT: f32 = 14.0;
/// Interligne, en multiples du corps.
const LINE_FACTOR: f32 = 1.35;
/// Grossissement d'un titre `# `.
const H1_FACTOR: f32 = 1.25;
/// Grossissement d'un sous-titre `## `.
const H2_FACTOR: f32 = 1.10;
/// Marge horizontale entre le bord de la carte et son texte.
const PAD_X: f32 = 18.0;
/// Marge verticale entre le bord de la carte et son texte.
const PAD_Y: f32 = 12.0;
/// Rayon des coins de la carte.
const CORNER_RADIUS: f32 = 24.0;
/// Décalage du texte d'une puce `- ` par rapport au reste.
const BULLET_INDENT: f32 = 14.0;
/// Rayon du disque d'une puce.
const BULLET_RADIUS: f32 = 2.2;
/// Décalage du disque d'une puce depuis la marge gauche.
const BULLET_OFFSET: f32 = 3.0;
/// Position de la puce sur la hauteur de la ligne, en multiples du corps.
const BULLET_BASELINE: f32 = 0.45;
/// Épaisseur du cadre d'une carte au repos.
const BORDER: f32 = 1.0;
/// Largeur du curseur d'édition.
const CURSOR_WIDTH: f32 = 2.0;
/// Hauteur du curseur d'édition, en multiples du corps.
const CURSOR_HEIGHT: f32 = 1.2;

/// Épaisseur de l'anneau de sélection, **en pixels écran**.
///
/// Exception SCALE-1 : c'est une affordance, pas du contenu. Mis à l'échelle, il
/// disparaîtrait en dézoomant au moment précis où l'on cherche ce qu'on a sélectionné.
pub(super) const SELECTION_RING: f32 = 2.0;

// ── Ce qu'une passe d'annotations garde constant ────────────────────────────

/// Le bord de l'écran utile. Tout ce qui en sort est écarté avant d'être dessiné (loi L1).
#[derive(Clone, Copy)]
pub(super) struct Clip {
    pub width: f32,
    pub height: f32,
    pub top: f32,
}

impl Clip {
    /// La boîte écran `(x, y, w, h)` est-elle entièrement hors champ ou trop petite ?
    pub(super) fn rejects(self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w < 0.0 || x > self.width || y + h < self.top || y > self.height || (w < 3.0 && h < 3.0)
    }
}

/// Ce qui ne change pas d'une annotation à l'autre pendant une frame.
pub(super) struct Pass<'a> {
    pub typography: &'a Typography,
    /// `domain_id → teinte`, déjà résolue pour cette version du document (DOMAIN-TINT-1).
    pub tints: &'a DomainTints,
    pub theme: &'a Theme,
    pub vp: Viewport,
    pub scale: WorldScale,
    pub clip: Clip,
}

// ── Les lignes d'une carte (WRAP-1) ─────────────────────────────────────────

/// Le genre d'un paragraphe, lu sur son préfixe Markdown minimal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LineKind {
    Heading1,
    Heading2,
    Bullet,
    Body,
}

impl LineKind {
    /// Le genre du paragraphe et la longueur en octets de son préfixe.
    fn of(paragraph: &str) -> (Self, usize) {
        if paragraph.starts_with("# ") {
            (Self::Heading1, 2)
        } else if paragraph.starts_with("## ") {
            (Self::Heading2, 3)
        } else if paragraph.starts_with("- ") || paragraph.starts_with("* ") {
            (Self::Bullet, 2)
        } else {
            (Self::Body, 0)
        }
    }

    /// Les grossissements de titre sont des multiples du corps : ils héritent de l'unique
    /// transformation au lieu de redériver du zoom.
    fn font(self, body: f32) -> f32 {
        match self {
            Self::Heading1 => body * H1_FACTOR,
            Self::Heading2 => body * H2_FACTOR,
            Self::Bullet | Self::Body => body,
        }
    }

    fn bold(self) -> bool {
        matches!(self, Self::Heading1 | Self::Heading2)
    }

    fn indent(self, layout: &CardLayout) -> f32 {
        if self == Self::Bullet {
            layout.indent
        } else {
            0.0
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Heading1 => Color::from_rgba8(255, 255, 255, 255),
            Self::Heading2 => Color::from_rgba8(240, 240, 245, 255),
            Self::Bullet | Self::Body => Color::from_rgba8(220, 225, 235, 255),
        }
    }

    fn style(self, layout: &CardLayout) -> TextStyle {
        TextStyle { size: self.font(layout.font), color: self.color(), bold: self.bold() }
    }
}

/// Une ligne visuelle : une tranche `[start, end)` du corps, et le paragraphe dont elle vient.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VisualLine {
    pub start: usize,
    pub end: usize,
    pub kind: LineKind,
    /// Première ligne de son paragraphe : la seule qui porte la puce.
    pub first: bool,
    /// Début du paragraphe brut, préfixe compris — là où un curseur posé dans le préfixe
    /// se rattache.
    pub paragraph_start: usize,
}

/// WRAP-1 — les lignes visuelles de `body` dans une carte de `width` unités monde.
///
/// Chaque paragraphe (une ligne du texte source) est reflué à la largeur utile de la carte,
/// avec la police et l'indentation de son genre. Tout est en unités monde : le résultat ne
/// dépend pas du zoom.
pub(super) fn layout_lines(typography: &Typography, body: &str, width: f32) -> Vec<VisualLine> {
    let base = CardLayout::text_card(width, 0.0, 1);
    let mut lines = Vec::new();
    let mut offset = 0usize;
    for paragraph in body.split('\n') {
        let (kind, prefix) = LineKind::of(paragraph);
        let text = &paragraph[prefix..];
        let usable = (width - PAD_X * 2.0 - kind.indent(&base)).max(BODY_FONT);
        let font = kind.font(BODY_FONT);
        let advance = |ch: char| typography.get_glyph(ch, font, kind.bold()).metrics.advance_width;
        for (i, (s, e)) in wrap_paragraph(text, usable, advance).into_iter().enumerate() {
            lines.push(VisualLine {
                start: offset + prefix + s,
                end: offset + prefix + e,
                kind,
                first: i == 0,
                paragraph_start: offset,
            });
        }
        offset += paragraph.len() + 1;
    }
    lines
}

/// TEXT-FIT-1 — la hauteur, en unités monde, qu'une carte de `width` doit avoir pour
/// contenir `text` sans le tronquer. C'est ce que le geste de redimensionnement et la
/// validation d'une saisie écrivent dans le document.
pub(crate) fn text_card_fit_height(typography: &Typography, text: &str, width: f64) -> f64 {
    let lines = layout_lines(typography, text, width as f32).len();
    CardLayout::text_card(width as f32, 0.0, lines).height as f64
}

// ── Mise en page d'une carte ────────────────────────────────────────────────

/// Mise en page d'une carte de texte. **En unités monde tant que `scaled` n'a pas été
/// appelée** ; en pixels écran après, et rien d'autre ne dérive du zoom entre les deux.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CardLayout {
    pub width: f32,
    pub height: f32,
    pub font: f32,
    pub line_height: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub indent: f32,
    pub bullet: f32,
    pub bullet_offset: f32,
    pub border: f32,
}

impl CardLayout {
    /// Mise en page d'une carte de `width` × `height` unités monde contenant `lines` lignes.
    ///
    /// La hauteur s'étire pour contenir le texte : une carte ne tronque jamais son contenu.
    pub(super) fn text_card(width: f32, height: f32, lines: usize) -> Self {
        let line_height = BODY_FONT * LINE_FACTOR;
        let needed = PAD_Y * 2.0 + lines.max(1) as f32 * line_height;
        Self {
            width,
            height: height.max(needed),
            font: BODY_FONT,
            line_height,
            pad_x: PAD_X,
            pad_y: PAD_Y,
            radius: CORNER_RADIUS,
            indent: BULLET_INDENT,
            bullet: BULLET_RADIUS,
            bullet_offset: BULLET_OFFSET,
            border: BORDER,
        }
    }

    /// L'UNIQUE transformation d'échelle de la carte (SCALE-1) : tous les champs, le même
    /// facteur, au même instant. Ajouter un champ ici sans le mettre à l'échelle, ou le
    /// borner au passage, c'est ramener R-45.
    pub(super) fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            font: s.world(self.font),
            line_height: s.world(self.line_height),
            pad_x: s.world(self.pad_x),
            pad_y: s.world(self.pad_y),
            radius: s.world(self.radius),
            indent: s.world(self.indent),
            bullet: s.world(self.bullet),
            bullet_offset: s.world(self.bullet_offset),
            border: s.world(self.border),
        }
    }
}

// ── Dessin ──────────────────────────────────────────────────────────────────

/// Dessine les annotations visibles du tableau actif.
pub(super) fn draw_annotations(
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    editing_session: Option<&TextEditSession>,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let ctx = Pass {
        typography: kit.typography,
        tints: kit.tints,
        theme: kit.theme,
        vp: pass.vp,
        scale: WorldScale::new(pass.vp.scale),
        clip: Clip {
            width: pixmap.width() as f32,
            height: pixmap.height() as f32,
            top: pass.header_h,
        },
    };

    for ann in &board.annotations {
        if !pass.visible_ids.contains(ann.id()) {
            continue;
        }
        let selected = store.selected_annotation_ids.iter().any(|s| s == ann.id());
        let editing = editing_session.filter(|s| s.ann_id.as_str() == ann.id());
        match ann {
            Annotation::Text { x, y, width, height, text, color, .. } => {
                let (_, tint) = hue_cache.get_or_compute(ann, &board.annotations);
                let tint = color.as_deref().map(|c| parse_hex_color(c, tint.0, tint.1, tint.2)).unwrap_or(tint);
                let body = editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str());
                let size = (
                    width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH) as f32,
                    height.unwrap_or(DEFAULT_TEXT_CARD_HEIGHT) as f32,
                );
                draw_text_card(&ctx, pixmap, TextCard { origin: (*x, *y), size, body, tint, selected, editing });
                draw_node_gauge(&ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Sticky { x, y, .. } => {
                draw_sticky(&ctx, pixmap, ann, selected, editing);
                draw_node_gauge(&ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Arrow { x, y, x2, y2, .. } => {
                draw_arrow(&ctx, pixmap, (*x, *y), (*x2, *y2), selected);
            }
            _ => {}
        }
    }
}

/// Pose la réglette de domaines d'une annotation au-dessus de son bord haut.
fn draw_node_gauge(ctx: &Pass, pixmap: &mut PixmapMut, origin: (f64, f64), domains: &[DomainAssignment]) {
    let (wx, wy) = world_to_screen(origin.0, origin.1, &ctx.vp);
    draw_domain_gauge(ctx.typography, ctx.tints, pixmap, ctx.scale, (wx as f32, wy as f32), domains);
}

/// Une carte de texte prête à dessiner : sa géométrie **monde** et son contenu.
struct TextCard<'a> {
    origin: (f64, f64),
    size: (f32, f32),
    body: &'a str,
    tint: (u8, u8, u8),
    selected: bool,
    editing: Option<&'a TextEditSession>,
}

fn draw_text_card(ctx: &Pass, pixmap: &mut PixmapMut, card: TextCard) {
    // Le découpage en lignes et la hauteur nécessaire se calculent en unités monde, AVANT
    // l'unique mise à l'échelle (CARD-1, WRAP-1).
    let lines = layout_lines(ctx.typography, card.body, card.size.0);
    let layout = CardLayout::text_card(card.size.0, card.size.1, lines.len()).scaled(ctx.scale);

    let (wx, wy) = world_to_screen(card.origin.0, card.origin.1, &ctx.vp);
    let (sx, sy) = (wx as f32, wy as f32);
    if ctx.clip.rejects(sx, sy, layout.width, layout.height) {
        return;
    }

    draw_card_frame(ctx, pixmap, (sx, sy), &layout, &card);

    // SCALE-2 — l'unique niveau de détail : sous le seuil, la carte s'arrête à son cadre.
    if ctx.scale.draws_detail() {
        draw_card_body(ctx, pixmap, (sx, sy), &layout, &lines, &card);
    }
    if card.selected {
        let screen_box = (sx, sy, layout.width, layout.height);
        draw_resize_handles(pixmap, ctx.theme, ctx.scale, screen_box, &Handle::HORIZONTAL);
    }
}

/// Le fond teinté de la carte et son cadre.
fn draw_card_frame(ctx: &Pass, pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, card: &TextCard) {
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, at.0, at.1, layout.width, layout.height, layout.radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let (r, g, b) = card.tint;

    // Aura douce d'ambiance : #18181B teinté de 12 % de la teinte symbiotique.
    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(Color::from_rgba8(
        ((r as u16 * 12 + 24 * 88) / 100) as u8,
        ((g as u16 * 12 + 24 * 88) / 100) as u8,
        ((b as u16 * 12 + 27 * 88) / 100) as u8,
        248,
    ));
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

    let highlighted = card.selected || card.editing.is_some();
    let mut stroke_paint = Paint { anti_alias: true, ..Default::default() };
    stroke_paint.set_color(match (card.editing.is_some(), card.selected) {
        (true, _) => Color::from_rgba8(56, 189, 248, 255),
        (false, true) => Color::from_rgba8(56, 189, 248, 220),
        (false, false) => Color::from_rgba8(r, g, b, 60),
    });
    let stroke = Stroke {
        // Le cadre au repos appartient à la carte et suit son échelle ; l'anneau de
        // sélection est une affordance et garde sa taille écran (exception SCALE-1).
        width: if highlighted { ctx.scale.screen(SELECTION_RING) } else { layout.border },
        ..Default::default()
    };
    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
}

/// Le texte de la carte, ligne visuelle par ligne visuelle, curseur d'édition compris.
fn draw_card_body(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    lines: &[VisualLine],
    card: &TextCard,
) {
    let show_cursor = card
        .editing
        .map(|s| (s.blink_timer.elapsed().as_millis() / 500) % 2 == 0)
        .unwrap_or(false);
    let cursor_idx = card.editing.map(|s| s.cursor_idx).unwrap_or(0);

    let mut cur_y = at.1 + layout.pad_y;
    let mut cursor_drawn = false;

    for (num, line) in lines.iter().enumerate() {
        let style = line.kind.style(layout);
        if line.first && line.kind == LineKind::Bullet {
            draw_bullet(pixmap, (at.0 + layout.pad_x, cur_y), layout, card.tint);
        }
        let start_x = at.0 + layout.pad_x + line.kind.indent(layout);
        let text = &card.body[line.start..line.end];
        ctx.typography.draw_text(pixmap, text, start_x, cur_y, style);

        // Un curseur posé dans le préfixe (`# `) se rattache au début de sa première ligne ;
        // la dernière ligne recueille tout ce qui dépasse.
        let last = num + 1 == lines.len();
        let from = if line.first { line.paragraph_start } else { line.start };
        let in_line = cursor_idx >= from && (cursor_idx <= line.end || last);
        if show_cursor && !cursor_drawn && in_line {
            let prefix = &card.body[line.start..cursor_idx.clamp(line.start, line.end)];
            let (prefix_w, _) = ctx.typography.measure_text(prefix, style.size, style.bold);
            draw_cursor(pixmap, (start_x + prefix_w, cur_y), layout, ctx.scale);
            cursor_drawn = true;
        }
        cur_y += layout.line_height;
    }
}

fn draw_bullet(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(Color::from_rgba8(tint.0, tint.1, tint.2, 200));
    let mut pb = PathBuilder::new();
    pb.push_circle(at.0 + layout.bullet_offset, at.1 + layout.font * BULLET_BASELINE, layout.bullet);
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

fn draw_cursor(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, scale: WorldScale) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(56, 189, 248, 255));
    // Le curseur mesure le texte qu'il édite — sa hauteur suit la police — mais son trait
    // est une affordance : il garde sa largeur écran (exception SCALE-1), comme le curseur
    // de n'importe quel éditeur. Mis à l'échelle, il s'effacerait au dézoom.
    let width = scale.screen(CURSOR_WIDTH);
    if let Some(rect) = Rect::from_xywh(at.0, at.1, width, layout.font * CURSOR_HEIGHT) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

#[cfg(test)]
mod proof;

#[cfg(test)]
pub mod tests;
