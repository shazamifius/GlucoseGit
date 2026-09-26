//! **Le texte d'une carte, hors du canevas** (ANCRE-UX) : ce que la fenêtre de l'éditeur du
//! texte lié montre, mis en page par la loi même de la carte.

use super::{draw_card_body, poser, TextCard};
use crate::renderer::pass::Pass;
use crate::renderer::scale::WorldScale;
use tiny_skia::PixmapMut;

/// **Le texte d'une carte, hors du canevas** — le corps seul, sans brume ni anneau : ce que la
/// fenêtre d'ancrage montre (ANCRE-UX).
///
/// Le texte est une carte virtuelle d'origine `(0, 0)`, large de `largeur` unités, posée par
/// `vp` ; il se met en page par la loi même de la carte, donc un octet y désigne ce qu'il
/// désigne dans la carte.
pub(crate) fn peindre_le_texte_seul(
    kit: crate::renderer::PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (texte, largeur, teinte): (&str, f32, (u8, u8, u8)),
    (vp, densite): (glucose_core::types::Viewport, f32),
) {
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale, densite),
        clip: crate::renderer::pass::Clip {
            width: pixmap.width() as f32,
            height: pixmap.height() as f32,
            top: 0.0,
        },
    };
    let card = TextCard {
        origin: (0.0, 0.0),
        size: (largeur, 0.0),
        body: texte,
        tint: teinte,
        fond: super::fond_du_canevas(kit.theme),
        selected: false,
        editing: None,
    };
    let Some((mise_en_page, layout, at)) = poser(&ctx, &card) else {
        return;
    };
    draw_card_body(&ctx, pixmap, at, &layout, &mise_en_page, &card);
}
