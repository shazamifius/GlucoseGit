//! **Une carte de texte en dessin vectoriel** (P5 fiche 69, P8 fiche 73) : le contenu de
//! [`super::draw_card_contenu`] — la même mise en page, posée par [`super::poser`], dans le même
//! ordre : la brume, le fond des passages qui brillent, puis chaque ligne — sa plaque, son
//! ornement, le surlignage de la sélection, son texte.
//!
//! L'export SVG l'écrit ; de près, la carte graphique la dessine (fiche 73). Ce qui la distingue
//! du canevas n'est que la façon de poser une lettre : depuis son contour, à sa place exacte.
//!
//! Sa lueur n'est pas ici : comme au canevas, toutes les lueurs passent avant toutes les cartes
//! (LUEUR-3, `halo/svg.rs`, fiche 70).

use super::habit::{boite_de_la_carte, voile_de_la_carte};
use super::ornament::{ornement_en_vecteurs, plaque_de_code};
use super::{encre_de_la_ligne, poser, CardLayout, TextCard};
use crate::renderer::pass::Pass;
use crate::renderer::richtext::draw::{boite_de_la_selection, ligne_en_vecteurs};
use crate::renderer::richtext::{font_of, indent_of, TextLayout, VisualLine};
use crate::renderer::vectoriel::Vectoriel;
use glucose_core::text::{BlockKind, Selection};
use tiny_skia::Color;

/// **Dessine cette carte** : sa brume, puis, au-dessus du seuil de détail (SCALE-2), son texte.
pub(in crate::renderer) fn carte_en_vecteurs<V: Vectoriel>(ctx: &Pass, card: TextCard, v: &mut V) {
    let Some((text, layout, at)) = poser(ctx, &card) else {
        return;
    };
    // La brume : la forme même que le canevas remplit, de la couleur qu'il compose.
    let [r, g, b] = voile_de_la_carte(&card);
    let voile = Color::from_rgba(r, g, b, 1.0).unwrap_or(Color::BLACK);
    let boite = boite_de_la_carte(at, (layout.width, layout.height));
    v.rectangle(boite, layout.radius, voile);
    if ctx.scale.draws_detail() {
        corps_en_vecteurs(ctx, v, at, &layout, &text, &card);
    }
}

/// Le corps : le fond des passages qui brillent, puis les lignes.
fn corps_en_vecteurs<V: Vectoriel>(
    ctx: &Pass,
    v: &mut V,
    at: (f32, f32),
    layout: &CardLayout,
    text: &TextLayout,
    card: &TextCard,
) {
    let selection = card.editing.map(|s| s.selection).unwrap_or_default();
    if let Some(e) = card.eclaires.filter(|_| card.editing.is_none()) {
        let passages = crate::renderer::passages::troncons(
            (ctx.typography, ctx.math),
            (text, card.body, card.size.0),
            e.plages,
        );
        let pose = crate::renderer::passages::Pose {
            origine: card.origin,
            vp: ctx.vp,
            echelle: ctx.scale,
        };
        crate::renderer::passages::cadres_en_vecteurs(v, pose, &passages, (e.teinte, e.style));
    }
    let mut y = at.1 + layout.pad_y;
    for line in &text.lines {
        ligne_en_vecteurs_de_carte(ctx, v, (at.0, y), layout, (text, line), (card, selection));
        y += layout.line_height;
    }
}

/// Une ligne : sa plaque, son ornement — qui remplace un signe au repos (MODE-1) —, le
/// surlignage de la sélection, son texte.
fn ligne_en_vecteurs_de_carte<V: Vectoriel>(
    ctx: &Pass,
    v: &mut V,
    (left, y): (f32, f32),
    layout: &CardLayout,
    (text, line): (&TextLayout, &VisualLine),
    (card, selection): (&TextCard, Selection),
) {
    let gauche = left + layout.pad_x;
    let debut = gauche + indent_of(line.kind, layout.indent);
    if line.kind == BlockKind::Code {
        v.rectangle_net(plaque_de_code((gauche, y), layout), ctx.theme.code_bg);
    }
    if card.editing.is_none()
        && ornement_en_vecteurs(ctx, v, (gauche, debut, y), layout, (line, card))
    {
        return;
    }
    let font = font_of(line.kind, layout.font);
    if let Some(boite) = boite_de_la_selection(
        ctx,
        (debut, y),
        (text, line),
        (font, layout.line_height),
        (card.body, selection),
    ) {
        v.rectangle(boite, 0.0, ctx.theme.text_selection);
    }
    let encre = encre_de_la_ligne(ctx, line, card);
    ligne_en_vecteurs(ctx, v, (debut, y), (text, line), (font, encre), card.body);
}
