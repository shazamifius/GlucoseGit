//! **Ce qui se pose au-dessus d'une carte** : ses poignées quand elle est sélectionnée, son
//! curseur quand on l'écrit (COMPOSANT-3).
//!
//! Extrait de [`super`] quand le curseur y a quitté le corps : ce sont les deux choses qui
//! changent sans que le contenu change — un clic sélectionne, un curseur clignote deux fois
//! par seconde. Sur la voie graphique, elles vont dans la couche du dessus et la texture de la
//! carte ne se refait pas ; au processeur, elles se peignent après le contenu, par les mêmes
//! fonctions.

use super::{CardLayout, Pass, TextCard};
use crate::params::Pen;
use crate::renderer::handles::draw_resize_handles;
use crate::renderer::pass::SELECTION_RING;
use crate::renderer::richtext::hit::{line_of_offset, offset_to_x};
use crate::renderer::richtext::{font_of, indent_of, ink_of, mode_of, TextLayout, VisualLine};
use crate::renderer::scale::WorldScale;
use crate::theme::Theme;
use glucose_core::resize::Handle;
use glucose_core::text::BlockKind;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Largeur du curseur d'édition.
const CURSOR_WIDTH: f32 = 2.0;
/// Hauteur du curseur d'édition, en multiples du corps.
const CURSOR_HEIGHT: f32 = 1.2;

pub(super) fn dessiner_les_ornements(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    text: &TextLayout,
    card: &TextCard,
) {
    // SCALE-2 : sous le seuil de détail, la carte n'est plus que son cadre — pas de texte, donc
    // ni curseur ni formule à suivre. Les poignées, elles, restent : on redimensionne une carte
    // qu'on ne lit plus. Elles passent sur la pastille (une affordance n'est jamais cachée,
    // ORNEMENTS-1), le curseur sur tout.
    if card.selected || card.editing.is_some() {
        draw_card_ring(ctx, pixmap, at, layout);
    }
    let detail = ctx.scale.draws_detail();
    if detail {
        draw_formula_preview(ctx, pixmap, at, layout, text, card);
    }
    if card.selected {
        let screen_box = (at.0, at.1, layout.width, layout.height);
        draw_resize_handles(pixmap, ctx.theme, ctx.scale, screen_box, &Handle::ALL);
    }
    if detail {
        draw_card_caret(ctx, pixmap, at, layout, text, card);
    }
}

/// **L'anneau qui désigne une carte** sélectionnée ou éditée : une affordance, qui garde sa
/// taille écran (exception SCALE-1).
///
/// Il était dans la texture de la carte (COMPOSANT-4) : sélectionner une carte la refaisait
/// entière — 12,35 ms de `textures` au p99 du geste « sélectionner », dans sa session du
/// 25/09 à 22 h 19. Et c'était le seul trait de `tiny-skia` qu'une texture portait : la même
/// forme translatée d'un nombre entier de pixels n'y donnait pas toujours la même couverture,
/// d'où une tolérance de vingt-six niveaux dans l'accord des deux voies, qui part avec lui.
fn draw_card_ring(ctx: &Pass, pixmap: &mut PixmapMut, at: (f32, f32), layout: &CardLayout) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(
        &mut pb,
        at.0,
        at.1,
        layout.width,
        layout.height,
        layout.radius,
    );
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(ctx.theme.selection_frame);
    let stroke = Stroke {
        width: ctx.scale.screen(SELECTION_RING),
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

/// Le curseur d'édition, sur la ligne qui le porte.
///
/// La phase se **lit** (BLINK-1) : la calculer ici rendrait le dessin dépendant de l'instant
/// où il a lieu, donc non reproductible.
fn draw_card_caret(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    text: &TextLayout,
    card: &TextCard,
) {
    let Some(saisie) = card.editing.filter(|s| s.curseur_visible) else {
        return;
    };
    let cursor = saisie.selection.head;
    // L'ordonnée s'accumule ligne par ligne, exactement comme le corps pose ses lignes : un
    // produit `rang × hauteur` n'aurait pas toujours les mêmes bits, et le curseur se
    // décalerait d'un cheveu de la ligne qu'il édite.
    let mut y = at.1 + layout.pad_y;
    for line in &text.lines {
        if porte_le_curseur(line, cursor) {
            let start_x = at.0 + layout.pad_x + indent_of(line.kind, layout.indent);
            let font = font_of(line.kind, layout.font);
            let dx = offset_to_x(ctx.typography, text, line, card.body, cursor, font);
            let ink = ink_of(line.kind, ctx.theme);
            draw_cursor(pixmap, (start_x + dx, y), layout, ctx.scale, ink);
            return;
        }
        y += layout.line_height;
    }
}

/// **Cette ligne porte-t-elle le curseur ?** La première qui couvre sa position : la fin d'une
/// ligne refluée la garde, le début de la suivante ne la reprend pas.
///
/// Il y avait deux règles de plus, et aucune n'avait de cas : un curseur posé dans le préfixe
/// d'un bloc (`# `, `> `) se rattachait au début du paragraphe — mais une première ligne
/// commence toujours là, préfixe compris ; et la dernière ligne recueillait ce qui dépasse la
/// fin du texte — mais **toute position du texte appartient à une ligne**, en mode source, et
/// c'est éprouvé sur chaque genre de bloc (`test_composant_3_toute_position_du_texte_a_sa_ligne`).
/// Des sabotages qui les retiraient ne faisaient rien tomber.
fn porte_le_curseur(line: &VisualLine, cursor: usize) -> bool {
    line.start <= cursor && cursor <= line.end
}

// ── La prévisualisation d'une formule (fiche 12 § 1.A.3) ──────────────────────

/// Écart entre la carte éditée et sa pastille de prévisualisation, au corps de référence.
const PREVIEW_GAP: f32 = 12.0;
/// Marges intérieures de la pastille, au corps de référence.
const PREVIEW_PAD: f32 = 10.0;
/// Rayon de ses coins, celui des autres surfaces flottantes.
const PREVIEW_RADIUS: f32 = 6.0;

/// **La formule que le curseur est en train d'écrire**, rendue en direct à côté de la carte.
///
/// Pendant l'édition, une ligne de formule montre sa source — c'est elle qu'on corrige, et on
/// n'édite pas une fraction. Le résultat n'apparaîtrait donc qu'en sortant de la carte. Cette
/// pastille le montre pendant la frappe, à hauteur de la ligne éditée : les délimiteurs
/// colorés disent **si** ça compile, celle-ci dit **quoi**. Elle ne s'affiche que là où elle a
/// quelque chose à dire : une ligne de formule, en édition, qui compile.
///
/// **C'est un ornement, pas du contenu** (COMPOSANT-3) : elle suit le curseur et se pose hors
/// de la boîte. Dans le contenu, elle empêchait la carte d'être une texture — son placement lit
/// `clip.width`, qui vaut la texture dans un composant et l'écran dans une passe —, et toute la
/// carte retombait au processeur, à chaque image, le temps qu'un curseur traverse une formule.
/// Au-dessus, `clip.width` vaut l'écran sur les deux voies : elle se pose à droite, et bascule
/// à gauche quand le bord de l'écran approche.
fn draw_formula_preview(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    text: &TextLayout,
    card: &TextCard,
) {
    let Some(session) = card.editing else {
        return;
    };
    let rang = line_of_offset(text, session.selection.head);
    let Some(line) = text.lines.get(rang) else {
        return;
    };
    let BlockKind::Math { display } = line.kind else {
        return;
    };
    let source = super::paragraph_of(card.body, line);
    let Some((corps, _)) = glucose_core::text::block::formula(source) else {
        return;
    };
    let corps = &source[corps];
    let mode = mode_of(display);
    let Some((w, h, d)) = ctx.math.measure(corps, mode, layout.font) else {
        return;
    };

    // La pastille suit le corps du texte, comme la carte : un seul rapport pour ses marges et
    // son écart, des deux côtés. L'écart de gauche restait au corps de référence et se collait
    // d'autant plus à la carte qu'on zoomait.
    let corps_relatif = layout.font / super::BODY_FONT;
    let (pad, ecart) = (PREVIEW_PAD * corps_relatif, PREVIEW_GAP * corps_relatif);
    let (bw, bh) = (w + pad * 2.0, h + d + pad * 2.0);
    let droite = at.0 + layout.width + ecart;
    let x = if droite + bw <= ctx.clip.width {
        droite
    } else {
        (at.0 - bw - ecart).max(0.0)
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
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    paint.set_color(theme.btn_border);
    let filet = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &filet, Transform::identity(), None);
}

#[cfg(test)]
mod tests;

/// Le curseur d'édition, à l'encre de la ligne qu'il édite.
fn draw_cursor(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    scale: WorldScale,
    ink: Color,
) {
    // Le curseur mesure le texte qu'il édite — sa hauteur suit la police — mais son trait
    // est une affordance : il garde sa largeur écran (exception SCALE-1), comme le curseur
    // de n'importe quel éditeur. Mis à l'échelle, il s'effacerait au dézoom.
    let width = scale.screen(CURSOR_WIDTH);
    if let Some(rect) = Rect::from_xywh(at.0, at.1, width, layout.font * CURSOR_HEIGHT) {
        // Sur la grille (SCALE-3) : un curseur d'un pixel posé à une demi-position devient
        // deux demi-traits gris, et il clignote — donc il attire l'œil sur son propre flou.
        crate::renderer::scale::fill_crisp(pixmap, rect, ink);
    }
}
