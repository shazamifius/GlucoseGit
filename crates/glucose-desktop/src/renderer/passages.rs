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
use super::pass::{Clip, Pass};
use super::richtext::draw::draw_line_ink;
use super::richtext::hit::offset_to_x;
use super::richtext::{font_of, indent_of, mode_of, Ink, TextLayout, TextMode, VisualLine};
use super::scale::WorldScale;
use super::PaintKit;
use crate::canvas::world_to_screen;
use crate::params::{Eclairage, ViewPass};
use crate::typography::{Face, Typography};
use glucose_core::quadtree::{noeud_au_rang, Noeud};
use glucose_core::store::Store;
use glucose_core::text::BlockKind;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapMut, Transform};

/// **Comment un passage brille** — les nombres de Tauri, en unités du monde : la carte de
/// Tauri se met à l'échelle avec tout ce qu'elle porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Surlignage {
    /// Le fond, et le liseré, en opacité de la teinte.
    fond: f32,
    lisere: f32,
    /// L'épaisseur du liseré, et son décalage hors de la boîte (`outline-offset`).
    epaisseur: f32,
    decalage: f32,
    /// La marge autour du texte (`padding`), et les coins.
    marge: (f32, f32),
    coins: f32,
    /// La lueur (`box-shadow`) : son écart-type — la moitié du flou CSS —, son étalement, son
    /// opacité sur 255.
    lueur: (f32, f32, u8),
}

/// Au survol d'une flèche, sur la carte (`HtmlAnnotationLayer.tsx`) : fond à 20 %, liseré de
/// 1,5 à 45 % décalé de 2, marge `1px 3px`, coins de 3, lueur `0 0 16px 6px` à 25 %.
pub(crate) const SUR_LA_CARTE: Surlignage = Surlignage {
    fond: 0.20,
    lisere: 0.45,
    epaisseur: 1.5,
    decalage: 2.0,
    marge: (3.0, 1.0),
    coins: 3.0,
    lueur: (8.0, 6.0, 64),
};

/// Dans la fenêtre de l'éditeur (`ArrowTextEditor.tsx`) : fond à 25 %, liseré de 1 à 40 %
/// décalé de 1, marge `1px 4px`, lueur `0 0 8px` à 15 %.
pub(crate) const DANS_L_EDITEUR: Surlignage = Surlignage {
    fond: 0.25,
    lisere: 0.40,
    epaisseur: 1.0,
    decalage: 1.0,
    marge: (4.0, 1.0),
    coins: 3.0,
    lueur: (4.0, 0.0, 38),
};

/// **La part d'un passage sur une ligne** : sa boîte, en unités du monde depuis le coin de la
/// carte, et la ligne qui la porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Troncon {
    pub rect: (f32, f32, f32, f32),
    pub ligne: usize,
}

/// **Les tronçons d'un passage**, un par ligne qu'il touche — et la mise en page où ils vivent.
///
/// # La boîte épouse le texte, pas la ligne
///
/// Sa hauteur est celle de la **police** — de sa montante à sa descente —, là où la ligne vaut
/// 1,4 corps : la typographie pose la base à `haut + corps`, donc tout ce qui dépasse la police
/// tombe sous le texte. Une boîte haute comme la ligne paraissait décalée vers le bas ; celle-ci,
/// comme le `<mark>` de Tauri, colle au texte. Les métriques sont celles d'Inter, communes à ses
/// quatre graisses.
///
/// # Une formule est un atome
///
/// Au repos, une formule ne se lit pas caractère par caractère : on ne désigne pas la moitié
/// d'une fraction. Un passage qui touche un paragraphe de formule en prend la boîte **dessinée**
/// — sa largeur et sa hauteur rendues —, et non la largeur de sa source, qu'on ne voit pas.
pub(crate) fn troncons(
    (typographie, math): (&Typography, &MathRenderer),
    texte: &str,
    largeur: f32,
    plages: &[(usize, usize)],
) -> (TextLayout, Vec<Troncon>) {
    let mise_en_page = card_text_layout(typographie, math, texte, largeur, TextMode::Rendered);
    let boite = text_box(largeur);
    let mut out = Vec::new();
    for (rang, ligne) in mise_en_page.lines.iter().enumerate() {
        let corps = font_of(ligne.kind, boite.body);
        let gauche = TEXT_ORIGIN.0 + indent_of(ligne.kind, boite.bullet_indent);
        let haut = TEXT_ORIGIN.1 + rang as f32 * boite.line_height;
        for &(debut, fin) in plages {
            if fin <= ligne.start || debut > ligne.end {
                continue;
            }
            if let Some(rect) = boite_d_une_formule(math, texte, ligne, (gauche, haut, corps)) {
                out.push(Troncon { rect, ligne: rang });
                break;
            }
            let x = |o: usize| offset_to_x(typographie, &mise_en_page, ligne, texte, o, corps);
            let (a, b) = (x(debut.max(ligne.start)), x(fin.min(ligne.end)));
            if b > a {
                let (y, h) = cadre_de_la_police(typographie, haut, corps);
                out.push(Troncon {
                    rect: (gauche + a, y, b - a, h),
                    ligne: rang,
                });
            }
        }
    }
    (mise_en_page, out)
}

/// **Les rectangles d'un passage** : ceux de ses tronçons — ce que les épreuves visent.
#[cfg(test)]
pub(crate) fn rectangles(
    outils: (&Typography, &MathRenderer),
    texte: &str,
    largeur: f32,
    plages: &[(usize, usize)],
) -> Vec<(f32, f32, f32, f32)> {
    troncons(outils, texte, largeur, plages)
        .1
        .into_iter()
        .map(|t| t.rect)
        .collect()
}

/// La boîte verticale de la police sur une ligne dont le haut est `haut` : de sa montante à sa
/// descente, autour de la base que la typographie pose à `haut + corps`.
fn cadre_de_la_police(typographie: &Typography, haut: f32, corps: f32) -> (f32, f32) {
    let (montante, descente) = typographie
        .font(Face::Regular)
        .horizontal_line_metrics(corps)
        .map_or((corps, 0.0), |m| (m.ascent, -m.descent));
    (haut + corps - montante, montante + descente)
}

/// La boîte **dessinée** d'une formule, si cette ligne en est la première : là où la carte la
/// pose (`draw_formula`), haute de ce qu'elle monte et descend.
fn boite_d_une_formule(
    math: &MathRenderer,
    texte: &str,
    ligne: &VisualLine,
    (gauche, haut, corps): (f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let BlockKind::Math { display } = ligne.kind else {
        return None;
    };
    let source = &texte[ligne.start..ligne.end];
    let (formule, _) = glucose_core::text::block::formula(source)?;
    let (l, h, d) = math.measure(&source[formule], mode_of(display), corps)?;
    ligne.first.then_some((gauche, haut, l, h + d))
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
        peindre_les_plages(
            kit,
            pixmap,
            (pass.vp, pass.densite),
            ((*x, *y), text, largeur as f32),
            &eclairage.plages,
            (teinte, &SUR_LA_CARTE),
        );
    }
}

/// **Peint les plages d'un texte de carte** posée en `origine`, large de `largeur` unités, vue
/// par `vp` : la lueur, le fond et le liseré de chaque tronçon, puis son texte **repeint à la
/// teinte** — ce qui, chez Tauri, faisait du passage une chose qui brille, et non un rectangle
/// posé sur du blanc.
pub(crate) fn peindre_les_plages(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (vp, densite): (Viewport, f32),
    (origine, texte, largeur): ((f64, f64), &str, f32),
    plages: &[(usize, usize)],
    (teinte, style): ((u8, u8, u8), &Surlignage),
) {
    if plages.is_empty() {
        return;
    }
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale, densite),
        clip: Clip {
            width: pixmap.width() as f32,
            height: pixmap.height() as f32,
            top: 0.0,
        },
    };
    let (mise_en_page, troncons) = troncons((kit.typography, kit.math), texte, largeur, plages);
    for troncon in troncons {
        let ecran = a_l_ecran(&ctx, origine, troncon.rect);
        peindre_la_boite(pixmap, ecran, teinte, (ctx.scale, style));
        repeindre_le_texte(
            &ctx,
            pixmap,
            (origine, texte, &mise_en_page),
            (troncon, ecran),
            teinte,
        );
    }
}

/// La boîte d'un tronçon à l'écran, sans sa marge.
fn a_l_ecran(
    ctx: &Pass,
    origine: (f64, f64),
    (rx, ry, rw, rh): (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let (sx, sy) = world_to_screen(
        origine.0 + f64::from(rx),
        origine.1 + f64::from(ry),
        &ctx.vp,
    );
    let s = ctx.scale;
    (sx as f32, sy as f32, s.world(rw), s.world(rh))
}

/// La lueur, le fond et le liseré d'un tronçon, en `(x, y, l, h)` à l'écran.
fn peindre_la_boite(
    pixmap: &mut PixmapMut,
    (x, y, l, h): (f32, f32, f32, f32),
    (r, g, b): (u8, u8, u8),
    (s, style): (WorldScale, &Surlignage),
) {
    let (gauche, haut) = (x - s.world(style.marge.0), y - s.world(style.marge.1));
    let (largeur, hauteur) = (
        l + s.world(2.0 * style.marge.0),
        h + s.world(2.0 * style.marge.1),
    );
    let (ecart_type, etalement, lueur) = style.lueur;
    let etale = s.world(etalement);
    draw_halo(
        pixmap,
        HaloBox {
            left: gauche - etale,
            top: haut - etale,
            right: gauche + largeur + etale,
            bottom: haut + hauteur + etale,
            sigma: s.world(ecart_type),
            carte: None,
        },
        (r, g, b),
        lueur,
    );
    let opacite = |f: f32| (f * 255.0).round() as u8;
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    let mut pb = PathBuilder::new();
    let coins = s.world(style.coins);
    crate::renderer::push_rounded_rect(&mut pb, gauche, haut, largeur, hauteur, coins);
    if let Some(fond) = pb.finish() {
        paint.set_color(Color::from_rgba8(r, g, b, opacite(style.fond)));
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
    let demi = s.world(style.epaisseur) / 2.0;
    for ecart in [
        s.world(style.decalage) + demi,
        s.world(style.decalage) - demi,
    ] {
        crate::renderer::push_rounded_rect(
            &mut pb,
            gauche - ecart,
            haut - ecart,
            largeur + 2.0 * ecart,
            hauteur + 2.0 * ecart,
            coins + ecart,
        );
    }
    if let Some(lisere) = pb.finish() {
        paint.set_color(Color::from_rgba8(r, g, b, opacite(style.lisere)));
        pixmap.fill_path(
            &lisere,
            &paint,
            tiny_skia::FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }
}

/// **Repeint le texte d'un tronçon à la teinte** : sa ligne entière est redessinée dans une
/// image de la taille du tronçon, qui ne garde que ce qui tombe dedans, puis posée.
///
/// Les glyphes sont ceux de la carte — même mise en page, même corps, même position —, donc
/// l'encre teintée recouvre exactement l'encre blanche.
fn repeindre_le_texte(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (origine, texte, mise_en_page): ((f64, f64), &str, &TextLayout),
    (troncon, (x, y, l, h)): (Troncon, (f32, f32, f32, f32)),
    (r, g, b): (u8, u8, u8),
) {
    let Some(ligne) = mise_en_page.lines.get(troncon.ligne) else {
        return;
    };
    let (ox, oy) = (x.floor() - 1.0, y.floor() - 1.0);
    let (w, hh) = ((l + 3.0).ceil() as u32, (h + 3.0).ceil() as u32);
    let Some(mut image) = Pixmap::new(w.max(1), hh.max(1)) else {
        return;
    };
    let boite = text_box(0.0);
    let haut = TEXT_ORIGIN.1 + troncon.ligne as f32 * boite.line_height;
    let gauche = TEXT_ORIGIN.0 + indent_of(ligne.kind, boite.bullet_indent);
    let (sx, sy) = world_to_screen(
        origine.0 + f64::from(gauche),
        origine.1 + f64::from(haut),
        &ctx.vp,
    );
    let at = (sx as f32 - ox, sy as f32 - oy);
    let corps = ctx.scale.world(font_of(ligne.kind, boite.body));
    let teinte = Color::from_rgba8(r, g, b, 255);
    if let BlockKind::Math { display } = ligne.kind {
        let source = &texte[ligne.start..ligne.end];
        if let Some((formule, _)) = glucose_core::text::block::formula(source) {
            let (formule, mode) = (&source[formule], mode_of(display));
            let monte = ctx
                .math
                .measure(formule, mode, corps)
                .map_or(corps, |m| m.1);
            let plume = crate::params::Pen {
                x: at.0,
                y: at.1 + monte,
                font_size: corps,
            };
            ctx.math
                .draw(&mut image.as_mut(), formule, mode, plume, teinte);
        }
    } else {
        let encre = Ink {
            text: teinte,
            marker: teinte,
            link: teinte,
        };
        draw_line_ink(
            ctx,
            &mut image.as_mut(),
            at,
            (mise_en_page, ligne),
            (corps, encre),
            texte,
        );
    }
    pixmap.draw_pixmap(
        ox as i32,
        oy as i32,
        image.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
