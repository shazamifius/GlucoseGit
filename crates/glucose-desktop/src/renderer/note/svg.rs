//! **Un pense-bête en SVG** (P5, fiche 69) : la sœur de [`super::draw_sticky`] — le papier, son
//! ombre et ses lignes, ou la pilule d'un opérateur ; la même mise en page, les mêmes couleurs.

use super::{
    document_color, operator_color, sticky_lines, StickyLayout, PILL_BORDER, PILL_FILL_ALPHA,
    PILL_FONT,
};
use crate::canvas::world_to_screen;
use crate::renderer::pass::Pass;
use crate::renderer::richtext::draw::style_svg;
use crate::renderer::svg::{couleur, n, Svg};
use crate::typography::{Face, TextStyle};
use glucose_core::types::Annotation;

/// **Écrit ce pense-bête**, s'il en est un.
pub(in crate::renderer) fn sticky_en_svg(ctx: &Pass, ann: &Annotation, svg: &mut Svg) {
    let Annotation::Sticky {
        x,
        y,
        text,
        color,
        bg_color,
        operator,
        ..
    } = ann
    else {
        return;
    };
    let Some((w, h)) = ann.size() else {
        return;
    };
    let layout = StickyLayout::new(w as f32, h as f32).scaled(ctx.scale);
    let (wx, wy) = world_to_screen(*x, *y, &ctx.vp);
    let at = (wx as f32, wy as f32);
    if let Some(op) = operator {
        let c = operator_color(*op);
        let (teinte, _) = couleur(c);
        let rayon = layout.height / 2.0;
        let boite = (at.0, at.1, layout.width, layout.height);
        svg.rectangle(boite, rayon, (&teinte, f32::from(PILL_FILL_ALPHA) / 255.0));
        svg.poser(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="none" stroke="{teinte}" stroke-width="{}"/>"#,
            n(at.0), n(at.1), n(layout.width), n(layout.height), n(rayon),
            n(ctx.scale.world(PILL_BORDER))
        ));
        let corps = ctx.scale.world(PILL_FONT);
        let mot = op.label();
        let (lw, lh) = ctx.typography.measure_text(mot, corps, Face::Bold);
        let style = TextStyle {
            size: corps,
            color: c,
            face: Face::Bold,
        };
        let place = (
            at.0 + (layout.width - lw) / 2.0,
            at.1 + (layout.height - lh) / 2.0,
        );
        svg.texte(place, mot, &style_svg(style));
        return;
    }
    let fond = document_color(bg_color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_bg);
    let encre = document_color(color.as_deref()).unwrap_or(ctx.theme.sticky_yellow_text);
    let ombre = couleur(ctx.theme.sticky_shadow);
    let papier = couleur(fond);
    let boite = (at.0, at.1, layout.width, layout.height);
    let decale = (
        at.0,
        at.1 + layout.shadow_offset,
        layout.width,
        layout.height,
    );
    svg.rectangle(decale, layout.radius, (&ombre.0, ombre.1));
    svg.rectangle(boite, layout.radius, (&papier.0, papier.1));
    let style = TextStyle {
        size: layout.font,
        color: encre,
        face: Face::Regular,
    };
    // STICKY-FIT-1 : le papier est fixe, une ligne qui ne tient plus n'est pas écrite.
    let plancher = at.1 + layout.height - layout.pad;
    let mut ligne = at.1 + layout.pad;
    for (debut, fin) in sticky_lines(ctx.typography, text, w as f32) {
        if ligne + layout.line_height > plancher + 1e-3 {
            break;
        }
        svg.texte(
            (at.0 + layout.pad, ligne),
            &text[debut..fin],
            &style_svg(style),
        );
        ligne += layout.line_height;
    }
}
