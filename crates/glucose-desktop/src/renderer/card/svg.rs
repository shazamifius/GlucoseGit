//! **Une carte de texte en SVG** (P5, fiche 69) : la sœur de [`super::draw_text_card`] — la
//! même mise en page, posée par [`super::poser`], la même brume, les mêmes lignes déjà coupées.
//!
//! Sa lueur, que le rendu peint dans une passe à elle (`halo.rs`), se pose ici sous la carte :
//! l'ombre de boîte de Glucose Tauri, la boîte dilatée puis floutée d'une gaussienne — ce que
//! le SVG dit exactement, coins arrondis compris, là où le rendu les approxime droits pour aller
//! vite (HALO-1).

use super::habit::BRUME;
use super::ornament::ornement_en_svg;
use super::{poser, TextCard};
use crate::renderer::halo::{HALO_ALPHA, HALO_BLUR, HALO_SPREAD};
use crate::renderer::pass::Pass;
use crate::renderer::richtext::draw::ligne_en_svg;
use crate::renderer::richtext::{font_of, indent_of, Ink};
use crate::renderer::svg::{couleur, hex, Svg};
use glucose_core::text::BlockKind;

/// **Écrit cette carte** : sa lueur, sa brume, puis son texte ligne par ligne.
pub(in crate::renderer) fn carte_en_svg(ctx: &Pass, card: TextCard, svg: &mut Svg) {
    let Some((text, layout, at)) = poser(ctx, &card) else {
        return;
    };
    let teinte = hex(card.tint);
    let (ecart, sigma) = (
        ctx.scale.world(HALO_SPREAD),
        ctx.scale.world(HALO_BLUR / 2.0),
    );
    let lueur = (
        at.0 - ecart,
        at.1 - ecart,
        layout.width + 2.0 * ecart,
        layout.height + 2.0 * ecart,
    );
    let opacite = f32::from(HALO_ALPHA) / 255.0;
    svg.lueur(lueur, layout.radius + ecart, (&teinte, opacite), sigma);
    // La brume : la teinte à quelques pour cent sur le fond qu'elle couvre (LUEUR-1).
    let part = f32::from(BRUME) / 255.0;
    let (t, f) = (card.tint, card.fond);
    let melange = |t: u8, f: u8| (f32::from(f) * (1.0 - part) + f32::from(t) * part).round() as u8;
    let voile = hex((melange(t.0, f.0), melange(t.1, f.1), melange(t.2, f.2)));
    let boite = (at.0, at.1, layout.width, layout.height);
    svg.rectangle(boite, layout.radius, (&voile, 1.0));
    let plaque = couleur(ctx.theme.code_bg);
    let mut y = at.1 + layout.pad_y;
    for line in &text.lines {
        let gauche = at.0 + layout.pad_x;
        let debut = gauche + indent_of(line.kind, layout.indent);
        if line.kind == BlockKind::Code {
            let fond = (
                gauche,
                y,
                layout.width - layout.pad_x * 2.0,
                layout.line_height,
            );
            svg.rectangle(fond, 0.0, (&plaque.0, plaque.1));
        }
        if !ornement_en_svg(ctx, svg, (gauche, debut, y), &layout, (line, &card)) {
            let encre = Ink::of(line.kind, ctx.theme);
            let font = font_of(line.kind, layout.font);
            ligne_en_svg(
                ctx,
                svg,
                (debut, y),
                (&text, line),
                (font, encre),
                card.body,
            );
        }
        y += layout.line_height;
    }
}
