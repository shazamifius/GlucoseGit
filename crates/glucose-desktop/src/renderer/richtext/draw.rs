//! Le tracé d'un texte riche : les fragments d'une ligne, leurs ornements, et le curseur.
//!
//! [`super`] dit **où** chaque fragment tombe ; ce module le pose. La séparation compte : la
//! mise en page se teste sans pixmap, le tracé se vérifie par capture, et l'un peut changer
//! sans l'autre.
//!
//! Les ornements du Markdown — le fond d'un `` `code` ``, la barre d'un `~~barré~~` — sont
//! des **fractions du corps**, jamais des longueurs à eux. Le corps est déjà à l'échelle du
//! zoom (SCALE-1), donc ils le suivent sans transformation propre : une longueur qui se dit
//! en multiples d'une autre n'a pas à exister séparément.

use super::hit::offset_to_x;
use super::{fragment_style, Fragment, Ink, TextLayout, VisualLine};
use crate::renderer::pass::Pass;
use crate::renderer::push_rounded_rect;
use crate::typography::Face;
use glucose_core::text::{BlockKind, Selection};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

/// Marge horizontale du fond d'un `` `code` `` : `4px` sur les `14px` de la référence.
const CODE_PAD_X: f32 = 4.0 / 14.0;
/// Marge verticale du même fond : `1px` sur `14px`.
const CODE_PAD_Y: f32 = 1.0 / 14.0;
/// Rayon de ses coins : `4px` sur `14px`.
const CODE_RADIUS: f32 = 4.0 / 14.0;
/// Épaisseur de la barre d'un `~~barré~~`, en multiples du corps.
const STRIKE_WIDTH: f32 = 1.0 / 14.0;

/// Pose les fragments d'une ligne depuis `at`, et rend l'abscisse où la plume s'arrête.
pub(crate) fn draw_line(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    (layout, line): (&TextLayout, &VisualLine),
    (font, ink): (f32, Ink),
    source: &str,
) -> f32 {
    let mut x = at.0;
    // Dans un bloc de code, le fond est déjà posé sous la ligne entière : le redessiner
    // fragment par fragment l'assombrirait par endroits, là où il doit être d'un seul ton.
    let plate = line.kind == BlockKind::Code;
    for fragment in layout.fragments_of(line) {
        // Un taquet replace la plume : c'est ce qui aligne les colonnes d'un tableau.
        if fragment.tab >= 0.0 {
            x = at.0 + fragment.tab * font;
        }
        x = draw_fragment(ctx, pixmap, (x, at.1), fragment, (font, ink), source, plate);
    }
    x
}

/// **L'encre seule d'une ligne** : ses glyphes, soulignés ou barrés, sans le fond de son code
/// — ce qu'un passage éclairé repeint à sa teinte par-dessus le texte (FLECHE-4). Le fond est
/// déjà sous le texte ; le reposer l'assombrirait.
pub(crate) fn draw_line_ink(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    (layout, line): (&TextLayout, &VisualLine),
    (font, ink): (f32, Ink),
    source: &str,
) {
    let mut x = at.0;
    for fragment in layout.fragments_of(line) {
        if fragment.tab >= 0.0 {
            x = at.0 + fragment.tab * font;
        }
        x = draw_fragment(ctx, pixmap, (x, at.1), fragment, (font, ink), source, true);
    }
}

/// Un fragment : son fond s'il est du code, son texte, sa barre s'il est barré.
fn draw_fragment(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    fragment: &Fragment,
    (font, ink): (f32, Ink),
    source: &str,
    plate: bool,
) -> f32 {
    let slice = &source[fragment.start..fragment.end];
    let style = fragment_style(fragment, font, ink);
    if fragment.emphasis.code() && !plate {
        let (w, _) = ctx.typography.measure_text(slice, font, style.face);
        draw_code_background(ctx, pixmap, at, w, (font, style.face));
    }
    let end_x = ctx.typography.draw_text(pixmap, slice, at.0, at.1, style);
    if fragment.emphasis.link() {
        draw_underline(
            ctx,
            pixmap,
            (at.0, end_x),
            at.1,
            (font, style.face),
            style.color,
        );
    }
    if fragment.emphasis.strike() {
        draw_strikethrough(
            ctx,
            pixmap,
            (at.0, end_x),
            at.1,
            (font, style.face),
            style.color,
        );
    }
    end_x
}

/// Le surlignage d'une sélection sur **une** ligne, posé avant le texte qu'il souligne.
///
/// La ligne qui ne porte rien de la sélection ne dessine rien ; celles du milieu d'une
/// sélection multi-lignes sont couvertes de bout en bout, sans que l'appelant ait à distinguer
/// les cas — [`offset_to_x`] borne d'elle-même un offset hors de la ligne.
///
/// Une ligne dont le saut de ligne est pris dans la sélection se prolonge d'une espace :
/// c'est ce qui montre qu'on a bien sélectionné la fin du paragraphe et pas seulement son
/// dernier mot, et c'est ce que fait tout éditeur.
pub(crate) fn draw_line_selection(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    (layout, line): (&TextLayout, &VisualLine),
    (font, line_height): (f32, f32),
    source: &str,
    selection: Selection,
) {
    let (start, end) = selection.range();
    if selection.is_empty() || end <= line.start || start > line.end {
        return;
    }
    let from = offset_to_x(
        ctx.typography,
        layout,
        line,
        source,
        start.max(line.start),
        font,
    );
    let mut to = offset_to_x(
        ctx.typography,
        layout,
        line,
        source,
        end.min(line.end),
        font,
    );
    if end > line.end {
        to += ctx.typography.advance(' ', font, Face::Regular);
    }
    let Some(rect) = Rect::from_xywh(at.0 + from, at.1, (to - from).max(0.0), line_height) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(ctx.theme.text_selection);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

/// Le fond d'un `` `code` `` : la valeur de la référence, dite en multiples du corps.
///
/// Il épouse **la boîte de la police**, montante et descendante comprises, et non la hauteur
/// de la ligne : c'est ce que fait un navigateur pour un élément en ligne, et c'est la seule
/// hauteur qui ne dépende pas de l'interligne choisi autour. Prise sur la ligne, la plaque
/// dépassait le texte par le bas d'un tiers de corps.
fn draw_code_background(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    width: f32,
    (font, face): (f32, Face),
) {
    let (pad_x, pad_y) = (font * CODE_PAD_X, font * CODE_PAD_Y);
    let Some(lm) = ctx.typography.font(face).horizontal_line_metrics(font) else {
        return;
    };
    let baseline = at.1 + font;
    let mut pb = PathBuilder::new();
    push_rounded_rect(
        &mut pb,
        at.0 - pad_x,
        baseline - lm.ascent - pad_y,
        width + pad_x * 2.0,
        lm.ascent - lm.descent + pad_y * 2.0,
        font * CODE_RADIUS,
    );
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(ctx.theme.code_bg);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

/// Le trait d'un lien, posé **sous la ligne de base**, à la profondeur que la police
/// indique pour un soulignement.
///
/// Lue dans la fonte et non devinée : chaque visage a la sienne, et une fraction du corps
/// ferait passer le trait à travers les jambages d'un `p` dans l'un et trop bas dans l'autre.
fn draw_underline(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (from, to): (f32, f32),
    top: f32,
    (font, face): (f32, Face),
    color: Color,
) {
    let metrics = ctx.typography.font(face).horizontal_line_metrics(font);
    let creux = metrics.map_or(font * STRIKE_WIDTH * 2.0, |m| -m.descent / 2.0);
    let thickness = (font * STRIKE_WIDTH).max(1.0);
    let Some(rect) = Rect::from_xywh(from, top + font + creux, to - from, thickness) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

/// La barre d'un `~~barré~~`, à mi-hauteur d'œil.
///
/// La hauteur d'œil est **lue dans la police**, jamais devinée : elle diffère d'un visage à
/// l'autre, et une fraction arbitraire du corps ferait passer la barre au-dessus des
/// minuscules dans l'un et en travers des jambages dans l'autre.
fn draw_strikethrough(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (from, to): (f32, f32),
    top: f32,
    (font, face): (f32, Face),
    color: Color,
) {
    let x_height = ctx.typography.font(face).metrics('x', font).height as f32;
    let baseline = top + font;
    let thickness = (font * STRIKE_WIDTH).max(1.0);
    let Some(rect) = Rect::from_xywh(from, baseline - x_height / 2.0, to - from, thickness) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}
