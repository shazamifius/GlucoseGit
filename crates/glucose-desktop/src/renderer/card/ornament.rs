//! Ce qu'un genre de bloc pose **autour** de son texte.
//!
//! La puce d'une liste, le numéro d'un élément, la barre d'une citation, le trait d'un `---`,
//! la plaque d'un bloc de code, la formule d'un `$$…$$` : six marques qui ne sont pas du
//! texte, et qu'aucun moteur de texte ne saurait poser.
//!
//! Toutes obéissent à la même règle, et c'est pour la dire une fois qu'elles sont réunies :
//! un ornement **remplace un signe que le repos efface**, donc il n'apparaît qu'au repos —
//! pendant l'édition, c'est le `- `, le `1. `, le `> ` qu'on voit et qu'on corrige (MODE-1).
//! La seule exception est la plaque d'un bloc de code, qui n'est pas un signe mais une
//! matière : elle reste pendant qu'on écrit dedans.

use super::{CardLayout, TextCard, BULLET_BASELINE};
use crate::params::Pen;
use crate::renderer::pass::Pass;
use crate::renderer::richtext::{ink_of, mode_of, place_d_une_formule, VisualLine};
use crate::renderer::vectoriel::Vectoriel;
use crate::typography::{Face, TextStyle};
use glucose_core::text::BlockKind;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

/// L'opacité d'un ornement teinté — celle de la puce, que la citation et le trait
/// reprennent : trois marques de même nature n'ont pas trois transparences.
const ORNAMENT_ALPHA: u8 = 200;

/// Ce qu'un genre de bloc pose **autour** de son texte : la puce d'une liste, le numéro d'un
/// élément, la barre d'une citation, le trait d'un `---`, la formule d'un `$$…$$`.
///
/// Rend `true` quand l'ornement est tout ce que la ligne avait à montrer — un trait n'a pas de
/// texte, et une formule au repos ne montre pas sa source.
pub(super) fn draw_ornament(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (left, text_x, y): (f32, f32, f32),
    layout: &CardLayout,
    line: &VisualLine,
    card: &TextCard,
) -> bool {
    match line.kind {
        BlockKind::Bullet if line.first => draw_bullet(pixmap, (left, y), layout, card.tint),
        BlockKind::Ordered(n) if line.first => {
            draw_ordinal(ctx, pixmap, (left, y), layout, n, card.tint);
        }
        BlockKind::Quote => draw_quote_bar(pixmap, (left, y), layout, card.tint),
        BlockKind::Rule | BlockKind::TableRule => {
            draw_rule(pixmap, (left, y), layout, card.tint);
            return true;
        }
        BlockKind::Math { .. } => {
            if line.first {
                draw_formula(ctx, pixmap, (text_x, y), layout, line, card);
            }
            return true;
        }
        _ => {}
    }
    false
}

/// Le numéro d'un élément de liste, **cadré à droite** sur le bord du texte.
///
/// C'est l'alignement de tout ce qui numérote depuis le plomb : les numéros s'empilent par
/// leur point, pas par leur premier chiffre, et un « 10. » déborde vers la marge intérieure
/// — qui a la place — au lieu de pousser le texte. Le blanc qui l'en sépare est une espace de
/// la police courante : une longueur de plus n'aurait rien dit que celle-ci ne dise déjà.
fn draw_ordinal(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    n: u32,
    tint: (u8, u8, u8),
) {
    let mut buf = [0u8; ORDINAL_MAX];
    let (label, (x, y), style) = numero(ctx, at, layout, (n, tint), &mut buf);
    ctx.typography.draw_text(pixmap, label, x, y, style);
}

/// **Le numéro d'un élément**, son coin et son style : là où [`draw_ordinal`] le pose, et le
/// dessin vectoriel avec lui.
fn numero<'b>(
    ctx: &Pass,
    at: (f32, f32),
    layout: &CardLayout,
    (n, tint): (u32, (u8, u8, u8)),
    buf: &'b mut [u8; ORDINAL_MAX],
) -> (&'b str, (f32, f32), TextStyle) {
    let label = ordinal_label(n, buf);
    let (w, _) = ctx
        .typography
        .measure_text(label, layout.font, Face::Regular);
    let gap = ctx.typography.advance(' ', layout.font, Face::Regular);
    let style = TextStyle {
        size: layout.font,
        color: encre_d_ornement(tint),
        face: Face::Regular,
    };
    (label, (at.0 + layout.indent - w - gap, at.1), style)
}

/// L'encre des ornements teintés.
fn encre_d_ornement(tint: (u8, u8, u8)) -> Color {
    Color::from_rgba8(tint.0, tint.1, tint.2, ORNAMENT_ALPHA)
}

/// Dix chiffres d'un `u32`, et le point qui les suit.
const ORDINAL_MAX: usize = 11;

/// « 12. » écrit dans un tampon de pile.
///
/// Un élément de liste par image et par carte visible ne mérite pas une allocation, et le
/// nombre de chiffres d'un `u32` est connu à la compilation.
fn ordinal_label(n: u32, buf: &mut [u8; ORDINAL_MAX]) -> &str {
    buf[ORDINAL_MAX - 1] = b'.';
    let mut i = ORDINAL_MAX - 1;
    let mut rest = n;
    loop {
        i -= 1;
        buf[i] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    std::str::from_utf8(&buf[i..]).expect("des chiffres ASCII et un point")
}

/// La barre d'une citation : ce que le `>` devient quand il s'efface.
///
/// Son épaisseur est le double du filet de la carte — celui d'un `---`, la seule épaisseur de
/// trait que cette carte connaisse déjà, donc la seule qui ne soit pas un nombre de plus.
fn draw_quote_bar(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    fill(
        pixmap,
        barre_de_citation(at, layout),
        encre_d_ornement(tint),
    );
}

/// La boîte de la barre d'une citation.
fn barre_de_citation(at: (f32, f32), layout: &CardLayout) -> (f32, f32, f32, f32) {
    (at.0, at.1, layout.border * 2.0, layout.line_height)
}

/// Le trait d'un `---`, à mi-hauteur de sa ligne et sur toute la largeur du texte.
fn draw_rule(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    fill(pixmap, filet(at, layout), encre_d_ornement(tint));
}

/// La boîte du trait d'un `---`.
fn filet(at: (f32, f32), layout: &CardLayout) -> (f32, f32, f32, f32) {
    let width = layout.width - layout.pad_x * 2.0;
    (
        at.0,
        at.1 + (layout.line_height - layout.border) / 2.0,
        width,
        layout.border,
    )
}

/// Le fond d'une ligne de bloc de code : la plaque d'un `` `code` `` étendue à toute la
/// largeur du texte, parce que c'est elle qui fait qu'un bloc se lit comme un bloc.
pub(super) fn draw_code_plate(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
) {
    fill(pixmap, plaque_de_code(at, layout), ctx.theme.code_bg);
}

/// La boîte de la plaque d'une ligne de code.
pub(super) fn plaque_de_code(at: (f32, f32), layout: &CardLayout) -> (f32, f32, f32, f32) {
    (
        at.0,
        at.1,
        layout.width - layout.pad_x * 2.0,
        layout.line_height,
    )
}

/// Un rectangle plein, quand il y en a un à remplir.
///
/// Filets, barres de citation et plaques de code sont tous alignés sur les axes : ils
/// passent donc par [`fill_crisp`] (SCALE-3), qui les pose sur la grille de pixels. C'est
/// là que la barre d'une citation faisait paniquer le rastériseur.
fn fill(pixmap: &mut PixmapMut, (x, y, l, h): (f32, f32, f32, f32), color: Color) {
    let Some(rect) = Rect::from_xywh(x, y, l, h) else {
        return;
    };
    crate::renderer::scale::fill_crisp(pixmap, rect, color);
}

/// Une formule qui occupe tout un paragraphe, posée sous ce qu'elle monte.
fn draw_formula(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    line: &VisualLine,
    card: &TextCard,
) {
    let Some(f) = poser_la_formule(ctx, at, layout, line, card) else {
        return;
    };
    if ctx.math.draw(pixmap, f.corps, f.mode, f.plume, f.encre) {
        return;
    }
    // Une formule fausse montre sa source, en rouge : l'erreur se voit là où elle est, pas
    // dans une console.
    let (x, y) = f.erreur.0;
    ctx.typography
        .draw_text(pixmap, f.erreur.1, x, y, f.erreur.2);
}

/// **Une formule posée** : sa source, son mode, sa plume, son encre — et, si elle est fausse,
/// ce qui la remplace (sa source en rouge, à sa place). Le tracé et le SVG la lisent tous deux.
struct FormulePosee<'a> {
    corps: &'a str,
    mode: glucose_math::Mode,
    plume: Pen,
    encre: Color,
    erreur: ((f32, f32), &'a str, TextStyle),
}

/// **Où et comment la formule d'une ligne se pose** — rien si la ligne n'en porte pas.
fn poser_la_formule<'a>(
    ctx: &Pass,
    at: (f32, f32),
    layout: &CardLayout,
    line: &VisualLine,
    card: &TextCard<'a>,
) -> Option<FormulePosee<'a>> {
    let text = &card.body[line.start..line.end];
    // Le genre de la ligne dit déjà si la formule veut le mode bloc ; seules ses bornes se
    // relisent, par la même fonction du noyau qui les a reconnues.
    let BlockKind::Math { display } = line.kind else {
        return None;
    };
    let (corps, _) = glucose_core::text::block::formula(text)?;
    let (corps, mode) = (&text[corps], mode_of(display));
    // Une formule qu'un passage désigne brille entière, à la teinte de l'éclat (PASSAGE-2).
    let encre = match card.eclaires.filter(|e| e.touchent(line.start, line.end)) {
        Some(e) => tiny_skia::Color::from_rgba8(e.teinte.0, e.teinte.1, e.teinte.2, 255),
        None => ink_of(line.kind, ctx.theme),
    };
    // La ligne de base se pose **sous ce que la formule monte**, dans les lignes qu'elle
    // réserve et avec son interligne — la même place que la mise en page lui a donnée.
    let (_, ligne_de_base) = place_d_une_formule(ctx.math, corps, mode);
    let plume = Pen {
        x: at.0,
        y: at.1 + ligne_de_base * layout.font,
        font_size: layout.font,
    };
    let rouge = TextStyle {
        size: layout.font,
        color: ctx.theme.danger,
        face: Face::Regular,
    };
    Some(FormulePosee {
        corps,
        mode,
        plume,
        encre,
        erreur: (at, text, rouge),
    })
}

fn draw_bullet(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(encre_d_ornement(tint));
    let mut pb = PathBuilder::new();
    let (cx, cy) = centre_de_la_puce(at, layout);
    pb.push_circle(cx, cy, layout.bullet);
    if let Some(path) = pb.finish() {
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// Le centre de la puce d'une liste.
fn centre_de_la_puce(at: (f32, f32), layout: &CardLayout) -> (f32, f32) {
    (
        at.0 + layout.bullet_offset,
        at.1 + layout.font * BULLET_BASELINE,
    )
}

// ── Le dessin vectoriel (P5 fiche 69, P8 fiche 73) ─────────────────────────────

/// **Les ornements d'une ligne, en dessin vectoriel** : ceux de [`draw_ornament`], aux mêmes
/// places — les mêmes fonctions les placent —, à la même encre ; la barre, le filet et la
/// plaque sur la grille des pixels, comme le canevas les pose.
pub(super) fn ornement_en_vecteurs<V: Vectoriel>(
    ctx: &Pass,
    v: &mut V,
    (left, text_x, y): (f32, f32, f32),
    layout: &CardLayout,
    (line, card): (&VisualLine, &TextCard),
) -> bool {
    let encre = encre_d_ornement(card.tint);
    match line.kind {
        BlockKind::Bullet if line.first => {
            v.disque(centre_de_la_puce((left, y), layout), layout.bullet, encre);
        }
        BlockKind::Ordered(n) if line.first => {
            let mut buf = [0u8; ORDINAL_MAX];
            let (label, at, style) = numero(ctx, (left, y), layout, (n, card.tint), &mut buf);
            v.texte(ctx.typography, at, label, style);
        }
        BlockKind::Quote => v.rectangle_net(barre_de_citation((left, y), layout), encre),
        BlockKind::Rule | BlockKind::TableRule => {
            v.rectangle_net(filet((left, y), layout), encre);
            return true;
        }
        BlockKind::Math { .. } => {
            let posee = line
                .first
                .then(|| poser_la_formule(ctx, (text_x, y), layout, line, card));
            if let Some(f) = posee.flatten() {
                if !ctx.math.en_vecteurs(v, f.corps, f.mode, f.plume, f.encre) {
                    let (at, source, rouge) = f.erreur;
                    v.texte(ctx.typography, at, source, rouge);
                }
            }
            return true;
        }
        _ => {}
    }
    false
}
