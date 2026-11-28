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

use super::handles::draw_resize_handles;
use super::pass::{Pass, SELECTION_RING};
use super::richtext::brut::layout_brut;
use super::richtext::draw::{draw_line, draw_line_selection};
use super::richtext::{Ink, TextLayout};
use super::scale::WorldScale;
use super::{parse_hex_rgb, push_rounded_rect, TextEditSession};
use crate::canvas::world_to_screen;
use crate::theme::operator_color;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::resize::Handle;
use glucose_core::text::Selection;
use glucose_core::types::{Annotation, StickyOperator};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

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

pub(in crate::renderer) mod svg;

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
    (editing, presence): (Option<&TextEditSession>, f32),
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
    // ESTOMPE-1 : ce que la frise chronologique écarte s'estompe — sauf ce qu'on écrit, qui se
    // voit entier. L'anneau et les poignées d'un post-it choisi restent entiers.
    let presence = if editing.is_some() { 1.0 } else { presence };
    let estomper = |c| super::estompe::estomper(c, presence);
    match operator {
        Some(op) => draw_pill(ctx, pixmap, (at, &layout), (*op, presence), highlighted),
        None => {
            let paper = Paper {
                bg: estomper(
                    document_color(bg_color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_bg),
                ),
                ink: estomper(
                    document_color(color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_text),
                ),
                presence,
                text: editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str()),
                world_width: w as f32,
                // La phase se **lit** (BLINK-1), elle ne se recalcule pas dans le dessin.
                cursor: editing
                    .filter(|s| s.curseur_visible)
                    .map(|s| s.selection.head),
                selection: editing.map(|s| s.selection).unwrap_or_default(),
            };
            draw_paper(ctx, pixmap, at, &layout, &paper, highlighted);
        }
    }

    if selected && ctx.poignees {
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
    /// Sa présence (ESTOMPE-1) : son ombre la suit.
    presence: f32,
    text: &'a str,
    /// La largeur du papier en unités monde : le reflux se calcule avant la mise à l'échelle.
    world_width: f32,
    /// Position du curseur d'édition à dessiner, si la session clignote « allumé ».
    cursor: Option<usize>,
    /// Ce qui est sélectionné dans la saisie — vide hors d'elle (fiche 188).
    selection: Selection,
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
        let ombre = super::estompe::estomper(ctx.theme.sticky_shadow, paper.presence);
        fill(pixmap, &shadow, ombre);
    }
    fill(pixmap, &path, paper.bg);
    if highlighted {
        ring(ctx, pixmap, &path, (layout.width, layout.height));
    }

    // Le reflux se calcule en unités monde, avant la mise à l'échelle (WRAP-1).
    let mise = sticky_text_layout(ctx.typography, paper.text, paper.world_width);
    // SCALE-2 — sous le seuil, une lettre ne se lit plus : le gris de chaque ligne (GRIS-1).
    if !ctx.scale.draws_detail() {
        let lines: Vec<(usize, usize)> = mise.lines.iter().map(|l| (l.start, l.end)).collect();
        draw_paper_gris(ctx, pixmap, at, layout, (paper, &lines));
        return;
    }
    draw_paper_text(ctx, pixmap, at, layout, paper, &mise);
}

/// **GRIS-1 — de loin, le gris de chaque ligne du papier** (fiche 95) : la loi des nœuds de texte
/// ([`crate::typography::forme_grise`]), sur les lignes que le papier montre — celles qui
/// tiennent au-dessus de sa marge du bas (STICKY-FIT-1).
fn draw_paper_gris(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    (paper, lines): (&Paper<'_>, &[(usize, usize)]),
) {
    let zoom = ctx.scale.world(1.0);
    if zoom <= 0.0 {
        return;
    }
    let floor = at.1 + layout.height - layout.pad;
    let mut y = at.1 + layout.pad;
    for &(start, end) in lines {
        if y + layout.line_height > floor + 1e-3 {
            break;
        }
        let lettres = paper.text[start..end]
            .chars()
            .map(|c| (c, Some(Face::Regular)));
        let (largeur, encre) = ctx.typography.gris(lettres, STICKY_FONT);
        let boite = (at.0 + layout.pad, y, largeur * zoom, layout.line_height);
        let place = largeur * layout.line_height / zoom;
        if let Some(ligne) = crate::typography::forme_grise(boite, (encre, place), paper.ink) {
            crate::typography::peindre_exact(pixmap, ligne);
        }
        y += layout.line_height;
    }
}

/// La pilule d'un opérateur logique (fiche 06 § 5.3) : coins en demi-cercle, fond à sa
/// couleur à 12 %, bord de 1,5 à sa couleur, et son mot centré en gras.
fn draw_pill(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (at, layout): ((f32, f32), &StickyLayout),
    (op, presence): (StickyOperator, f32),
    highlighted: bool,
) {
    let color = super::estompe::estomper(operator_color(op), presence);
    let radius = layout.height / 2.0;
    let Some(path) = rounded_path(at, layout.width, layout.height, radius) else {
        return;
    };
    let c = color.to_color_u8();
    fill(
        pixmap,
        &path,
        super::estompe::estomper(
            Color::from_rgba8(c.red(), c.green(), c.blue(), PILL_FILL_ALPHA),
            presence,
        ),
    );
    stroke(pixmap, &path, color, ctx.scale.world(PILL_BORDER));
    if highlighted {
        ring(ctx, pixmap, &path, (layout.width, layout.height));
    }
    // Le mot, à toute distance : de loin, le gris de son encre (GRIS-2, fiche 96).
    let font = ctx.scale.world(PILL_FONT);
    let label = op.label();
    let (text_w, line_h) = ctx.typography.measure_text(label, font, Face::Bold);
    let style = TextStyle {
        size: font,
        color,
        face: Face::Bold,
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

/// L'anneau de sélection : une affordance, qui garde sa taille écran (exception SCALE-1) tant
/// que le nœud a la place de la porter, et la suit ensuite, comme ses poignées (POIGNEE-1).
fn ring(ctx: &Pass, pixmap: &mut PixmapMut, path: &tiny_skia::Path, boite: (f32, f32)) {
    let part = crate::renderer::handles::part_des_ornements(ctx.scale, boite);
    stroke(
        pixmap,
        path,
        ctx.theme.selection_frame,
        ctx.scale.screen(SELECTION_RING) * part,
    );
}

/// **La boîte du texte d'un pense-bête** de `width` unités monde : sa marge retirée, son corps,
/// son interligne — ce que le dessin, le clic et le clavier lisent ensemble (fiche 188).
pub(crate) fn sticky_text_box(width: f32) -> crate::renderer::richtext::TextBox {
    crate::renderer::richtext::TextBox {
        usable: (width - STICKY_PAD * 2.0).max(STICKY_FONT),
        body: STICKY_FONT,
        bullet_indent: 0.0,
        line_height: STICKY_FONT * STICKY_LINE_FACTOR,
    }
}

/// Le coin du texte d'un pense-bête, depuis le coin du papier, en unités monde.
pub(crate) const STICKY_TEXT_ORIGIN: (f32, f32) = (STICKY_PAD, STICKY_PAD);

/// WRAP-1 — la mise en page d'un pense-bête de `width` unités monde. Un pense-bête n'a pas de
/// Markdown : chaque ligne source est un paragraphe tel quel ([`layout_brut`]).
pub(crate) fn sticky_text_layout(typography: &Typography, content: &str, width: f32) -> TextLayout {
    layout_brut(typography, content, sticky_text_box(width))
}

/// Ses lignes visuelles, en tranches d'octets de `content`.
fn sticky_lines(typography: &Typography, content: &str, width: f32) -> Vec<(usize, usize)> {
    let mise = sticky_text_layout(typography, content, width);
    mise.lines.iter().map(|l| (l.start, l.end)).collect()
}

fn draw_paper_text(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &StickyLayout,
    paper: &Paper<'_>,
    mise: &TextLayout,
) {
    let encre = Ink {
        text: paper.ink,
        marker: paper.ink,
        link: paper.ink,
        eclat: paper.ink,
    };
    // STICKY-FIT-1 : le papier est fixe, une ligne qui ne tient plus n'est pas dessinée.
    let floor = at.1 + layout.height - layout.pad;
    let gauche = at.0 + layout.pad;
    let mut cur_y = at.1 + layout.pad;
    let mut cursor_drawn = false;
    for (num, line) in mise.lines.iter().enumerate() {
        if cur_y + layout.line_height > floor + 1e-3 {
            break;
        }
        // Le surlignage passe sous le texte (fiche 188), comme dans un nœud de texte.
        let corps = (layout.font, layout.line_height);
        let ligne = (mise, line);
        draw_line_selection(
            ctx,
            pixmap,
            (gauche, cur_y),
            ligne,
            corps,
            paper.text,
            paper.selection,
        );
        draw_line(
            ctx,
            pixmap,
            (gauche, cur_y),
            ligne,
            (layout.font, encre),
            paper.text,
        );
        let (start, end) = (line.start, line.end);
        let last = num + 1 == mise.lines.len();
        if let Some(idx) = paper
            .cursor
            .filter(|&i| !cursor_drawn && i >= start && (i <= end || last))
        {
            let (prefix_w, _) = ctx.typography.measure_text(
                &paper.text[start..idx.clamp(start, end)],
                layout.font,
                Face::Regular,
            );
            crate::renderer::curseur::poser(
                pixmap,
                (gauche + prefix_w, cur_y),
                layout.font,
                ctx.scale,
                paper.ink,
            );
            cursor_drawn = true;
        }
        cur_y += layout.line_height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_1_a_sticky_is_self_similar_at_every_zoom() {
        let world = StickyLayout::new(160.0, 120.0);
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
            let screen = world.scaled(WorldScale::new(zoom, 1.0));
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

    /// Ce que la sélection change au plus sur le bord haut d'un post-it posé en `(40, 40)` de
    /// l'écran, vu à ce zoom.
    fn ecart_de_l_anneau(zoom: f64) -> u8 {
        use crate::renderer::pass::Clip;
        let renderer = crate::renderer::Renderer::new();
        let kit = renderer.kit();
        let ctx = Pass {
            typography: kit.typography,
            math: kit.math,
            mises_en_page: kit.mises_en_page,
            tints: kit.tints,
            theme: kit.theme,
            vp: glucose_core::types::Viewport {
                scale: zoom,
                x: 40.0,
                y: 40.0,
            },
            scale: WorldScale::new(zoom, 1.0),
            clip: Clip {
                width: 300.0,
                height: 300.0,
                top: 0.0,
            },
            poignees: true,
        };
        let note = Annotation::sticky("n", 0.0, 0.0, "");
        let peindre = |choisi: bool| {
            let mut p = tiny_skia::Pixmap::new(300, 300).expect("pixmap");
            draw_sticky(&ctx, &mut p.as_mut(), &note, choisi, (None, 1.0));
            p
        };
        let (avec, sans) = (peindre(true), peindre(false));
        (38..=41)
            .flat_map(|y| (40..60).map(move |x| (y * 300 + x) as usize))
            .map(|i| avec.pixels()[i].red().abs_diff(sans.pixels()[i].red()))
            .max()
            .unwrap_or(0)
    }

    /// **POIGNEE-1 — l'anneau d'un post-it suit sa place à l'écran** : plein de près, une trace
    /// sur un post-it de quelques pixels.
    #[test]
    fn test_poignee_1_l_anneau_d_un_post_it_minuscule_s_estompe() {
        let (pres, loin) = (ecart_de_l_anneau(1.0), ecart_de_l_anneau(0.06));
        assert!(pres >= 100, "l'anneau d'un post-it de pres : {pres}");
        assert!(loin < pres / 3, "de loin il garde {loin} contre {pres}");
    }

    /// **POST-IT-SEL — ce qu'on choisit dans un post-it se voit** (fiche 188) : il ne dessinait
    /// qu'un curseur, et une sélection au clavier (`Maj`+flèches) n'y était pas surlignée.
    #[test]
    fn test_post_it_sel_la_selection_se_surligne() {
        let renderer = crate::renderer::Renderer::new();
        let kit = renderer.kit();
        let ctx = Pass {
            typography: kit.typography,
            math: kit.math,
            mises_en_page: kit.mises_en_page,
            tints: kit.tints,
            theme: kit.theme,
            vp: glucose_core::types::Viewport {
                scale: 1.0,
                x: 0.0,
                y: 0.0,
            },
            scale: WorldScale::new(1.0, 1.0),
            clip: crate::renderer::pass::Clip {
                width: 300.0,
                height: 200.0,
                top: 0.0,
            },
            poignees: false,
        };
        let texte = "le chat dort";
        let note = Annotation::sticky("n", 0.0, 0.0, texte);
        let peindre = |selection: Selection| {
            let session = TextEditSession {
                ann_id: "n".into(),
                buffer: texte.into(),
                selection,
                goal_x: None,
                blink_timer: std::time::Instant::now(),
                curseur_visible: false,
            };
            let mut p = tiny_skia::Pixmap::new(300, 200).expect("pixmap");
            draw_sticky(&ctx, &mut p.as_mut(), &note, false, (Some(&session), 1.0));
            p
        };
        let (avec, sans) = (
            peindre(Selection { anchor: 0, head: 7 }),
            peindre(Selection::at(0)),
        );
        // La bande de la première ligne, sous « le chat » : la marge, puis une ligne de corps.
        let differents = (12..26)
            .flat_map(|y| (12..40).map(move |x| (y * 300 + x) as usize))
            .filter(|&i| avec.pixels()[i] != sans.pixels()[i])
            .count();
        assert!(
            differents > 100,
            "le surlignage couvre « le chat » : {differents} pixels"
        );
    }
}
