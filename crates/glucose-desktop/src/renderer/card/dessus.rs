//! **Ce qui se pose au-dessus d'une carte** : ses poignées quand elle est sélectionnée, son
//! curseur quand on l'écrit (COMPOSANT-3).
//!
//! Extrait de [`super`] quand le curseur y a quitté le corps : ce sont les deux choses qui
//! changent sans que le contenu change — un clic sélectionne, un curseur clignote deux fois
//! par seconde. Sur la voie graphique, elles vont dans la couche du dessus et la texture de la
//! carte ne se refait pas ; au processeur, elles se peignent après le contenu, par les mêmes
//! fonctions.

use super::{CardLayout, Pass, TextCard};
use crate::renderer::handles::draw_resize_handles;
use crate::renderer::richtext::hit::offset_to_x;
use crate::renderer::richtext::{font_of, indent_of, ink_of, TextLayout, VisualLine};
use crate::renderer::scale::WorldScale;
use glucose_core::resize::Handle;
use tiny_skia::{Color, PixmapMut, Rect};

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
    if card.selected {
        let screen_box = (at.0, at.1, layout.width, layout.height);
        draw_resize_handles(pixmap, ctx.theme, ctx.scale, screen_box, &Handle::ALL);
    }
    draw_card_caret(ctx, pixmap, at, layout, text, card);
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
