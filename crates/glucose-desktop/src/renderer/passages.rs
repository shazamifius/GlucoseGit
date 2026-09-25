//! **Faire briller un passage d'une carte** (FLECHE-4) — ce qu'une flèche survolée y désigne,
//! ou ce que l'éditeur d'ancres y sélectionne.
//!
//! # Ce que Tauri faisait
//!
//! Au survol d'une flèche ancrée, le passage visé s'enveloppait d'un `<mark>` à la couleur de
//! sa carte (`HtmlAnnotationLayer.tsx`) : un fond à 20 %, un liseré à 45 % décalé de deux
//! pixels, une lueur floutée de 16 et étalée de 6 à 25 %, des coins de 3. Ce sont ces nombres,
//! en unités du monde : la carte de Tauri est mise à l'échelle avec tout ce qu'elle porte.
//!
//! Son défaut était ailleurs, et c'est lui que ce module ne reproduit pas : le passage est
//! désigné par ses **octets** ([`glucose_core::text_anchors`]), jamais par son mot. Deux
//! « bonjours » ne s'allument pas ensemble parce qu'on en a désigné un.
//!
//! # Où il est dessiné
//!
//! Dans la couche qui passe au-dessus des cartes, sur les deux voies : un passage éclairé
//! brille sur ce qu'on lit, sans rien changer à la carte elle-même — sa texture reste la même,
//! et rien n'est à refaire quand la souris quitte la flèche.

use super::card::{card_text_layout, text_box, TEXT_ORIGIN};
use super::halo::{draw_halo, HaloBox};
use super::hue::SymbioticHueCache;
use super::math::MathRenderer;
use super::richtext::hit::offset_to_x;
use super::richtext::{font_of, indent_of, TextMode};
use super::scale::WorldScale;
use super::PaintKit;
use crate::canvas::world_to_screen;
use crate::params::{Eclairage, ViewPass};
use crate::typography::Typography;
use glucose_core::quadtree::{noeud_au_rang, Noeud};
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Transform};

/// Le fond du passage, sur 255 (`color-mix(… 20 %)`).
const FOND: u8 = 51;
/// Son liseré (`45 %`), son épaisseur et son décalage, en unités du monde.
const LISERE: u8 = 115;
const EPAISSEUR_DU_LISERE: f32 = 1.5;
const DECALAGE_DU_LISERE: f32 = 2.0;
/// Sa marge autour du texte (`padding: 1px 3px`) et ses coins (`border-radius: 3px`).
const MARGE: (f32, f32) = (3.0, 1.0);
const COINS: f32 = 3.0;
/// Sa lueur (`box-shadow: 0 0 16px 6px`, `25 %`) : un flou de 16 est un écart-type de 8.
const ECART_TYPE: f32 = 8.0;
const ETALEMENT: f32 = 6.0;
const LUEUR: u8 = 64;

/// **Les rectangles d'un passage** dans une carte de texte : un par ligne qu'il touche,
/// `(x, y, largeur, hauteur)` en unités du monde depuis le coin de la carte.
///
/// La mise en page est celle de la carte au repos, et la même règle que le surlignage d'une
/// sélection : une ligne qui ne porte rien du passage ne rend rien.
pub(crate) fn rectangles(
    (typographie, math): (&Typography, &MathRenderer),
    texte: &str,
    largeur: f32,
    plages: &[(usize, usize)],
) -> Vec<(f32, f32, f32, f32)> {
    let mise_en_page = card_text_layout(typographie, math, texte, largeur, TextMode::Rendered);
    let boite = text_box(largeur);
    let mut rects = Vec::new();
    for (rang, ligne) in mise_en_page.lines.iter().enumerate() {
        let corps = font_of(ligne.kind, boite.body);
        let gauche = TEXT_ORIGIN.0 + indent_of(ligne.kind, boite.bullet_indent);
        for &(debut, fin) in plages {
            if fin <= ligne.start || debut > ligne.end {
                continue;
            }
            let x = |o: usize| offset_to_x(typographie, &mise_en_page, ligne, texte, o, corps);
            let (a, b) = (x(debut.max(ligne.start)), x(fin.min(ligne.end)));
            if b > a {
                rects.push((
                    gauche + a,
                    TEXT_ORIGIN.1 + rang as f32 * boite.line_height,
                    b - a,
                    boite.line_height,
                ));
            }
        }
    }
    rects
}

/// **Fait briller les passages demandés**, chacun à la couleur de sa carte — ou à la sienne,
/// s'il en porte une.
pub(super) fn eclairer(
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (store, eclairages, pass): (&Store, &[Eclairage], ViewPass<'_>),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    for eclairage in eclairages {
        let par_l_index = pass
            .index
            .rang_de(&eclairage.carte)
            .and_then(|r| noeud_au_rang(board, r));
        let Some(Noeud::Annotation(carte)) = par_l_index else {
            continue;
        };
        let Annotation::Text { x, y, text, .. } = carte else {
            continue;
        };
        let Some((largeur, _)) = carte.size() else {
            continue;
        };
        let teinte = eclairage
            .teinte
            .unwrap_or_else(|| hue_cache.get_or_compute(carte, pass.index, board).1);
        let rects = rectangles(
            (kit.typography, kit.math),
            text,
            largeur as f32,
            &eclairage.plages,
        );
        for rect in rects {
            peindre_un_passage(pixmap, (*x, *y), rect, teinte, &pass.vp);
        }
    }
}

/// Un rectangle de passage : sa lueur, son fond, son liseré.
fn peindre_un_passage(
    pixmap: &mut PixmapMut,
    carte: (f64, f64),
    (rx, ry, rw, rh): (f32, f32, f32, f32),
    (r, g, b): (u8, u8, u8),
    vp: &glucose_core::types::Viewport,
) {
    let s = WorldScale::new(vp.scale);
    let (sx, sy) = world_to_screen(carte.0 + f64::from(rx), carte.1 + f64::from(ry), vp);
    let (gauche, haut) = (sx as f32 - s.world(MARGE.0), sy as f32 - s.world(MARGE.1));
    let (largeur, hauteur) = (s.world(rw + 2.0 * MARGE.0), s.world(rh + 2.0 * MARGE.1));
    let etale = s.world(ETALEMENT);
    draw_halo(
        pixmap,
        HaloBox {
            left: gauche - etale,
            top: haut - etale,
            right: gauche + largeur + etale,
            bottom: haut + hauteur + etale,
            sigma: s.world(ECART_TYPE),
        },
        (r, g, b),
        LUEUR,
    );
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, gauche, haut, largeur, hauteur, s.world(COINS));
    if let Some(fond) = pb.finish() {
        paint.set_color(Color::from_rgba8(r, g, b, FOND));
        pixmap.fill_path(
            &fond,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    // Le liseré est un **anneau rempli** — deux contours et la règle pair-impair —, pas un
    // trait : dézoomé, il devient plus fin qu'un pixel, et c'est sur ces traits que le
    // rastériseur a déjà planté trois fois dans ce projet (SCALE-3).
    let mut pb = PathBuilder::new();
    let demi = s.world(EPAISSEUR_DU_LISERE) / 2.0;
    for ecart in [
        s.world(DECALAGE_DU_LISERE) + demi,
        s.world(DECALAGE_DU_LISERE) - demi,
    ] {
        crate::renderer::push_rounded_rect(
            &mut pb,
            gauche - ecart,
            haut - ecart,
            largeur + 2.0 * ecart,
            hauteur + 2.0 * ecart,
            s.world(COINS) + ecart,
        );
    }
    if let Some(lisere) = pb.finish() {
        paint.set_color(Color::from_rgba8(r, g, b, LISERE));
        pixmap.fill_path(
            &lisere,
            &paint,
            tiny_skia::FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests;
