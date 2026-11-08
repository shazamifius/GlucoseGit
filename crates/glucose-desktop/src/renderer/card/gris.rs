//! **GRIS-1 — de loin, le gris de chaque ligne** (fiche 95).
//!
//! Sous un cinquième de zoom, une lettre fait moins de trois pixels : elle ne se lit plus. Glucose
//! posait alors le cadre vide du nœud (SCALE-2) — un long article et un nœud vide se
//! ressemblaient. Sa parole du 8/11 : *« pas de LOD, pas de simplification »* ; devant la
//! proposition, *« carrément d'accord pour le gris des lignes »*.
//!
//! Vu de loin, un texte **est** un gris : l'encre de ses lettres répartie sur la place de ses
//! lignes. Chaque ligne se pose donc comme un rectangle de son encre, à l'opacité de son encre
//! sur sa place — sa largeur fois la hauteur d'une ligne : un paragraphe garde sa forme, ses
//! lignes courtes, son retrait, ses titres plus sombres. Ce qui n'est pas une lettre — la plaque
//! d'un bloc de code, la puce, la barre d'une citation, le trait d'un `---`, les passages qui
//! brillent — se pose tel quel, à sa place : rien n'est remplacé ni caché (fiche 96).
//!
//! # Une géométrie, deux peintres
//!
//! Ce qu'un nœud montre de loin se décrit une fois, en formes ([`formes_de_loin`]) : chacune sa
//! géométrie et son encre, dont l'opacité dit déjà la part de sa place que l'encre couvre. Le
//! canevas les peint à la **couverture exacte** de chaque pixel ([`crate::typography::peindre_exact`])
//! — un filet d'un dixième de pixel y garde un dixième de son encre, là où la grille des pixels
//! (SCALE-3, nette de près) le ramènerait à un pixel entier, dix fois trop ; le dessin vectoriel
//! (la carte graphique pour un nœud plus grand que l'écran) les pose tels quels.

use super::{CardLayout, TextCard};
use crate::renderer::pass::Pass;
use crate::renderer::richtext::{indent_of, ink_of, TextLayout};
use crate::renderer::vectoriel::Vectoriel;
use crate::typography::{forme_grise, peindre_exact};
use glucose_core::membrane_forme::Arrondi;
use glucose_core::text::BlockKind;
use tiny_skia::{Color, PixmapMut};

/// **Une forme vue de loin** : sa géométrie à l'écran — un rectangle, arrondi en disque pour une
/// puce —, et son encre.
pub(crate) type FormeDeLoin = (Arrondi, Color);

/// **Un rectangle plein vu de loin** : un filet, une barre, une plaque.
pub(crate) fn plein((x, y, l, h): (f32, f32, f32, f32), c: Color) -> FormeDeLoin {
    (Arrondi::nouveau(x, y, l, h, 0.0), c)
}

/// **Ce qu'un nœud de texte montre de loin**, sous son cadre et sa brume, forme par forme : la
/// plaque d'un bloc de code, les ornements ([`super::ornament::ornement_de_loin`]), le gris de
/// chaque ligne. Les passages qui brillent ont leur propre dessin, sur chaque voie.
pub(super) fn formes_de_loin(
    ctx: &Pass,
    at: (f32, f32),
    (layout, text, card): (&CardLayout, &TextLayout, &TextCard),
    poser: &mut dyn FnMut(FormeDeLoin),
) {
    let zoom = ctx.scale.world(1.0);
    if zoom <= 0.0 || layout.line_height <= 0.0 {
        return;
    }
    let hauteur_monde = layout.line_height / zoom;
    let gauche = at.0 + layout.pad_x;
    let mut y = at.1 + layout.pad_y;
    for line in &text.lines {
        if line.kind == BlockKind::Code {
            let plaque = super::ornament::plaque_de_code((gauche, y), layout);
            poser(plein(plaque, ctx.theme.code_bg));
        }
        let seul = card.editing.is_none()
            && super::ornament::ornement_de_loin(ctx, poser, (gauche, y), layout, line, card);
        if !seul {
            let x = gauche + indent_of(line.kind, layout.indent);
            let boite = (x, y, line.largeur * zoom, layout.line_height);
            let mesure = (line.encre, line.largeur * hauteur_monde);
            if let Some(ligne) = forme_grise(boite, mesure, ink_of(line.kind, ctx.theme)) {
                poser(ligne);
            }
        }
        y += layout.line_height;
    }
}

/// **Le gris de ce nœud, sur le canevas** : les passages qui brillent, puis ses formes.
pub(super) fn draw_card_gris(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    (layout, text, card): (&CardLayout, &TextLayout, &TextCard),
) {
    super::draw_passages(ctx, pixmap, text, card);
    formes_de_loin(ctx, at, (layout, text, card), &mut |f| {
        peindre_exact(pixmap, f)
    });
}

/// **Le gris de ce nœud, en vecteurs** : les mêmes formes, posées telles quelles.
pub(super) fn gris_en_vecteurs<V: Vectoriel>(
    ctx: &Pass,
    v: &mut V,
    at: (f32, f32),
    (layout, text, card): (&CardLayout, &TextLayout, &TextCard),
) {
    formes_de_loin(ctx, at, (layout, text, card), &mut |(f, c)| {
        let boite = (f.gauche, f.haut, f.droite - f.gauche, f.bas - f.haut);
        v.rectangle(boite, f.rayon, c);
    });
}

#[cfg(test)]
mod tests;
