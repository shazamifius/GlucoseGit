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
use crate::renderer::richtext::{ink_of, mode_of, VisualLine};
use crate::theme::Theme;
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
    let label = ordinal_label(n, &mut buf);
    let (w, _) = ctx
        .typography
        .measure_text(label, layout.font, Face::Regular);
    let gap = ctx.typography.advance(' ', layout.font, Face::Regular);
    let style = TextStyle {
        size: layout.font,
        color: Color::from_rgba8(tint.0, tint.1, tint.2, ORNAMENT_ALPHA),
        face: Face::Regular,
    };
    ctx.typography
        .draw_text(pixmap, label, at.0 + layout.indent - w - gap, at.1, style);
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
/// Son épaisseur est le double de celle du cadre de la carte — la seule épaisseur de trait
/// que cette carte connaisse déjà, donc la seule qui ne soit pas un nombre de plus.
fn draw_quote_bar(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    fill(
        pixmap,
        Rect::from_xywh(at.0, at.1, layout.border * 2.0, layout.line_height),
        Color::from_rgba8(tint.0, tint.1, tint.2, ORNAMENT_ALPHA),
    );
}

/// Le trait d'un `---`, à mi-hauteur de sa ligne et sur toute la largeur du texte.
fn draw_rule(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    let width = layout.width - layout.pad_x * 2.0;
    fill(
        pixmap,
        Rect::from_xywh(
            at.0,
            at.1 + (layout.line_height - layout.border) / 2.0,
            width,
            layout.border,
        ),
        Color::from_rgba8(tint.0, tint.1, tint.2, ORNAMENT_ALPHA),
    );
}

/// Le fond d'une ligne de bloc de code : la plaque d'un `` `code` `` étendue à toute la
/// largeur du texte, parce que c'est elle qui fait qu'un bloc se lit comme un bloc.
pub(super) fn draw_code_plate(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
) {
    fill(
        pixmap,
        Rect::from_xywh(
            at.0,
            at.1,
            layout.width - layout.pad_x * 2.0,
            layout.line_height,
        ),
        ctx.theme.code_bg,
    );
}

/// Un rectangle plein, quand il y en a un à remplir.
///
/// Filets, barres de citation et plaques de code sont tous alignés sur les axes : ils
/// passent donc par [`fill_crisp`] (SCALE-3), qui les pose sur la grille de pixels. C'est
/// là que la barre d'une citation faisait paniquer le rastériseur.
fn fill(pixmap: &mut PixmapMut, rect: Option<Rect>, color: Color) {
    let Some(rect) = rect else {
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
    let text = &card.body[line.start..line.end];
    // Le genre de la ligne dit déjà si la formule veut le mode bloc ; seules ses bornes se
    // relisent, par la même fonction du noyau qui les a reconnues.
    let BlockKind::Math { display } = line.kind else {
        return;
    };
    let Some((corps, _)) = glucose_core::text::block::formula(text) else {
        return;
    };
    let (corps, mode) = (&text[corps], mode_of(display));
    let ink = ink_of(line.kind, ctx.theme);
    // La ligne de base se pose **sous ce que la formule monte**. La poser à une hauteur fixe
    // ferait déborder par le haut tout ce qui monte plus qu'un corps de texte — une
    // intégrale, une somme, un exposant d'exposant — et la formule mordrait sur la ligne
    // précédente.
    let au_dessus = ctx
        .math
        .measure(corps, mode, layout.font)
        .map(|(_, h, _)| h)
        .unwrap_or(layout.font);
    let plume = Pen {
        x: at.0,
        y: at.1 + au_dessus,
        font_size: layout.font,
    };
    if ctx.math.draw(pixmap, corps, mode, plume, ink) {
        return;
    }
    // Une formule fausse montre sa source, en rouge : l'erreur se voit là où elle est, pas
    // dans une console.
    ctx.typography.draw_text(
        pixmap,
        text,
        at.0,
        at.1,
        crate::typography::TextStyle {
            size: layout.font,
            color: ctx.theme.danger,
            face: Face::Regular,
        },
    );
}

fn draw_bullet(pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout, tint: (u8, u8, u8)) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(Color::from_rgba8(tint.0, tint.1, tint.2, 200));
    let mut pb = PathBuilder::new();
    pb.push_circle(
        at.0 + layout.bullet_offset,
        at.1 + layout.font * BULLET_BASELINE,
        layout.bullet,
    );
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

// ── La prévisualisation d'une formule (fiche 12 § 1.A.3) ──────────────────────

/// Écart entre la carte éditée et sa pastille de prévisualisation.
const PREVIEW_GAP: f32 = 12.0;
/// Marges intérieures de la pastille.
const PREVIEW_PAD: f32 = 10.0;
/// Rayon de ses coins, celui des autres surfaces flottantes.
const PREVIEW_RADIUS: f32 = 6.0;

/// La formule que le curseur est en train d'écrire, rendue **en direct** à côté de la carte.
///
/// Pendant l'édition, une ligne de formule montre sa source — c'est elle qu'on corrige, et on
/// n'édite pas une fraction. Le résultat n'apparaîtrait donc qu'en sortant de la carte. Cette
/// pastille le montre pendant la frappe, à hauteur de la ligne éditée : les délimiteurs
/// colorés disent **si** ça compile, celle-ci dit **quoi**.
///
/// Elle ne s'affiche que là où elle a quelque chose à dire : une ligne de formule, en
/// édition, qui compile. Une formule fausse n'a pas de résultat à montrer, et ses `$` sont
/// déjà rouges — une pastille vide en plus n'apprendrait rien.
///
/// Elle se pose à droite, et bascule à gauche si elle sortirait de l'écran : le même réflexe
/// que le menu contextuel, pour la même raison.
/// **Cette carte montre-t-elle une previsualisation de formule ?**
///
/// C'est la seule chose qui empeche une carte en saisie d'etre un composant (COMPOSANT-2) :
/// la plaque se pose hors de sa boite et son placement lit `clip.width`, qui vaut l'ecran
/// dans une passe et la texture dans un composant.
///
/// **Le seul endroit qui en decide**, et c'est ce qui compte : `Regime::carte` refuse alors
/// d'en faire une texture, `draw_annotations` la dessine alors entiere. Deux tests separes
/// qui doivent rester d'accord finissent par ne plus l'etre -- ou bien les deux la
/// dessinent, ou bien aucun.
///
/// La mise en page se refait ici, et c'est assume : `bench_texte` la chiffre a 0,55 ms pour
/// soixante-douze cartes, soit huit microsecondes pour celle-ci, et c'est la SEULE carte
/// concernee. Porter une liste d'identites depuis la voie graphique jusqu'ici couterait plus
/// cher en couplage qu'en calcul.
pub(in crate::renderer) fn porte_une_previsualisation(ctx: &Pass, card: &TextCard) -> bool {
    let Some(session) = card.editing else {
        return false;
    };
    let text = super::card_text_layout(
        ctx.typography,
        ctx.math,
        card.body,
        card.size.0,
        card.mode(),
    );
    previsualisation_en_cours(&text, session)
}

/// **Le curseur est-il pose sur une ligne de formule ?**
///
/// La seule condition qui decide qu'une previsualisation VA se poser, et la seule qui se
/// calcule sans mesurer la formule. Deux appelants la lisent : [`draw_formula_preview`], qui
/// la pose, et `Regime::carte`, qui refuse alors de faire de la carte un composant --
/// la plaque sort de la boite de la carte et son placement lit `clip.width`, donc elle ne
/// tiendrait pas dans une texture (COMPOSANT-2).
///
/// **Une seule definition pour les deux**, parce que deux tests qui doivent rester d'accord
/// finissent par ne plus l'etre : le processeur dessinerait la carte que la carte graphique
/// pose deja, ou personne ne la dessinerait.
pub(in crate::renderer) fn previsualisation_en_cours(
    text: &crate::renderer::richtext::TextLayout,
    session: &crate::renderer::TextEditSession,
) -> bool {
    ligne_de_formule(text, session).is_some()
}

/// La ligne de formule sous le curseur, et son mode d'affichage.
fn ligne_de_formule<'a>(
    text: &'a crate::renderer::richtext::TextLayout,
    session: &crate::renderer::TextEditSession,
) -> Option<(&'a VisualLine, bool)> {
    let rang = crate::renderer::richtext::hit::line_of_offset(text, session.selection.head);
    let line = text.lines.get(rang)?;
    match line.kind {
        BlockKind::Math { display } => Some((line, display)),
        _ => None,
    }
}

pub(super) fn draw_formula_preview(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    text: &crate::renderer::richtext::TextLayout,
    card: &TextCard,
) {
    let Some(session) = card.editing else {
        return;
    };
    let Some((line, display)) = ligne_de_formule(text, session) else {
        return;
    };
    let rang = crate::renderer::richtext::hit::line_of_offset(text, session.selection.head);
    let source = super::paragraph_of(card.body, line);
    let Some((corps, _)) = glucose_core::text::block::formula(source) else {
        return;
    };
    let corps = &source[corps];
    let mode = mode_of(display);
    let Some((w, h, d)) = ctx.math.measure(corps, mode, layout.font) else {
        return;
    };

    let pad = PREVIEW_PAD * layout.font / crate::renderer::card::BODY_FONT;
    let (bw, bh) = (w + pad * 2.0, h + d + pad * 2.0);
    let droite = at.0 + layout.width + PREVIEW_GAP * layout.font / crate::renderer::card::BODY_FONT;
    let x = if droite + bw <= ctx.clip.width {
        droite
    } else {
        (at.0 - bw - PREVIEW_GAP).max(0.0)
    };
    // À hauteur de la ligne qu'on écrit : l'œil n'a pas à chercher le lien entre les deux.
    let y = at.1 + layout.pad_y + rang as f32 * layout.line_height;

    plaque(pixmap, (x, y, bw, bh), PREVIEW_RADIUS, ctx.theme);
    let plume = Pen {
        x: x + pad,
        y: y + pad + h,
        font_size: layout.font,
    };
    ctx.math
        .draw(pixmap, corps, mode, plume, ctx.theme.card_body);
}

/// Le fond d'une surface flottante : sa matière et son filet.
fn plaque(pixmap: &mut PixmapMut, (x, y, w, h): (f32, f32, f32, f32), r: f32, theme: &Theme) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, r);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(theme.btn_bg);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
    paint.set_color(theme.btn_border);
    pixmap.stroke_path(
        &path,
        &paint,
        &tiny_skia::Stroke {
            width: 1.0,
            ..Default::default()
        },
        Transform::identity(),
        None,
    );
}
