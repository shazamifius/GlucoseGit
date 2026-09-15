//! Ce qu'une flèche **dit** : son étiquette, posée sur son tracé.
//!
//! Glucose Tauri pose un court libellé au milieu de la flèche, dans une pastille sombre
//! (`ArrowSvgLayer.tsx`). Le modèle de Glucose Rust porte le champ `text` d'une flèche
//! depuis le début ; rien ne le lisait, et rien ne l'écrivait.
//!
//! # LABEL-1 — la pastille est **mesurée**, jamais estimée
//!
//! Glucose Tauri calcule sa largeur par `ann.text.length * 6.5`. Sur « W W W » la pastille
//! est trop courte, sur « ililil » elle est trop longue, et sur du texte accentué elle est
//! fausse d'un côté ou de l'autre selon la fonte. C'est le **même** défaut que celui d'une
//! carte collée qui se déclarait vingt fois trop courte : une géométrie devinée là où un
//! moteur de texte sait répondre.
//!
//! Ici, [`Typography::measure_text`] donne la largeur réelle — la même fonction qui dessine
//! ensuite les glyphes, donc la boîte ne peut pas mentir sur ce qu'elle contient.
//!
//! # Une étiquette tient sur **une ligne**, et c'est un choix
//!
//! Elle nomme une relation : « contredit », « d'après Gauss », « depuis 1912 ». Un retour à
//! la ligne y ferait une carte, et une carte se pose à côté de la flèche plutôt que dessus.
//! La saisie accepte donc le texte tel qu'il vient, sans jamais le replier.

use super::pass::Pass;
use crate::canvas::world_to_screen;
use crate::renderer::TextEditSession;
use crate::typography::{Face, TextStyle};
use glucose_core::types::{Annotation, Board};
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Transform};

/// Corps de l'étiquette, en pixels **écran** (Glucose Tauri : `fontSize={11}`).
///
/// Une longueur écran, comme la bande qui désigne une flèche (ARROW-1) : l'étiquette est une
/// **légende**, pas un contenu. Elle doit rester lisible quand on prend du recul pour voir la
/// structure d'un graphe — c'est même le moment où on en a le plus besoin.
const LABEL_SIZE_PX: f32 = 11.0;

/// Marge horizontale autour du texte, en pixels écran (Glucose Tauri : `x={-w/2 - 4}`).
const LABEL_PAD_X: f32 = 5.0;

/// Demi-hauteur de la pastille, en pixels écran (Glucose Tauri : `y={-11} height={22}`).
const LABEL_HALF_H: f32 = 11.0;

/// Rayon des coins de la pastille, en pixels écran (Glucose Tauri : `rx={3}`).
const LABEL_RADIUS: f32 = 3.0;

/// Décalage vertical de l'étiquette quand un prédicat partage le même point d'ancrage.
///
/// Les deux se posent au milieu du tracé : sans décalage, ils se superposeraient. Glucose
/// Tauri monte l'étiquette de 14 et descend le badge d'autant.
pub(super) const LABEL_LIFT_PX: f32 = 14.0;

/// Largeur de la barre de saisie, en pixels écran.
const CARET_WIDTH: f32 = 1.5;

/// Ce qu'une flèche affiche : son texte posé, ou ce qu'on est en train d'y écrire.
fn shown_text<'a>(
    arrow: &'a Annotation,
    editing: Option<&'a TextEditSession>,
) -> Option<(&'a str, Option<usize>)> {
    if let Some(session) = editing {
        // En saisie, la pastille montre le tampon — vide compris : c'est ce qui rend visible
        // qu'on écrit dans une flèche qui n'avait pas encore d'étiquette.
        return Some((session.buffer.as_str(), Some(session.selection.head)));
    }
    match arrow {
        Annotation::Arrow { text: Some(t), .. } if !t.trim().is_empty() => Some((t.as_str(), None)),
        _ => None,
    }
}

/// Pose l'étiquette d'une flèche sur son tracé, si elle en a une ou qu'on en écrit une.
///
/// `lifted` monte la pastille pour laisser la place au badge de prédicat, qui vise le même
/// point.
pub(super) fn draw_arrow_label(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    board: &Board,
    arrow: &Annotation,
    editing: Option<&TextEditSession>,
    lifted: bool,
) {
    let Some((text, caret)) = shown_text(arrow, editing) else {
        return;
    };
    let Some((wx, wy)) = glucose_core::arrow::label_anchor_in(arrow, board) else {
        return;
    };
    let (typography, theme, scale) = (ctx.typography, ctx.theme, ctx.scale);
    let (sx, sy) = world_to_screen(wx, wy, &ctx.vp);
    let cx = sx as f32;
    let cy = sy as f32
        - if lifted {
            scale.screen(LABEL_LIFT_PX)
        } else {
            0.0
        };

    let size = scale.screen(LABEL_SIZE_PX);
    // La largeur **et** la hauteur mesurées, par la fonction qui dessinera les glyphes
    // (LABEL-1). La hauteur sert à centrer le texte dans sa pastille : `draw_text` prend le
    // **haut** de la ligne, et non sa ligne de base.
    let (text_w, text_h) = typography.measure_text(text, size, Face::Regular);
    let pad = scale.screen(LABEL_PAD_X);
    let half_h = scale.screen(LABEL_HALF_H);
    let half_w = text_w / 2.0 + pad;

    let Some(pastille) = Rect::from_ltrb(cx - half_w, cy - half_h, cx + half_w, cy + half_h) else {
        return;
    };
    fill_rounded(
        pixmap,
        pastille,
        scale.screen(LABEL_RADIUS),
        theme.arrow_label_bg,
    );

    typography.draw_text(
        pixmap,
        text,
        cx - text_w / 2.0,
        cy - text_h / 2.0,
        TextStyle {
            size,
            color: theme.arrow_label_text,
            face: Face::Regular,
        },
    );

    if let Some(at) = caret {
        draw_caret(
            ctx,
            pixmap,
            Caret {
                left: cx - text_w / 2.0,
                cy,
                half_h,
                size,
                text,
                at,
            },
        );
    }
}

/// Où poser la barre de saisie : sa pastille, et le rang du curseur dans son texte.
///
/// Une structure plutôt que six paramètres : ils décrivent **un** objet, et les passer à
/// plat laisse deux `f32` voisins s'échanger sans que rien ne le dise.
struct Caret<'a> {
    left: f32,
    cy: f32,
    half_h: f32,
    size: f32,
    text: &'a str,
    at: usize,
}

/// La barre de saisie, à la position du curseur dans le texte.
fn draw_caret(ctx: &Pass<'_>, pixmap: &mut PixmapMut, caret: Caret<'_>) {
    let Caret {
        left,
        cy,
        half_h,
        size,
        text,
        at,
    } = caret;
    // La position du curseur se mesure sur **le début du texte**, avec la même fonction que
    // le dessin : la barre tombe donc exactement entre deux glyphes, jamais à côté.
    let avant = text.get(..at).unwrap_or(text);
    let (offset, _) = ctx.typography.measure_text(avant, size, Face::Regular);
    let x = left + offset;
    let Some(barre) = Rect::from_ltrb(x, cy - half_h + 3.0, x + CARET_WIDTH, cy + half_h - 3.0)
    else {
        return;
    };
    // **SCALE-3**, et pas un `fill_rect` : une barre d'un pixel et demi est un « hairline »
    // pour tiny-skia, qui panique dessus (`hairline_aa.rs`, `assert!(false)`). C'est la
    // troisième fois que ce crash apparaît dans ce projet ; les deux précédentes avaient été
    // corrigées sur place, et c'est précisément pour cela qu'il est revenu. Le cliquet 5 le
    // ferme désormais pour de bon.
    super::scale::fill_crisp(pixmap, barre, ctx.theme.arrow_label_text);
}

/// Un rectangle aux coins arrondis, rempli d'une couleur unie.
fn fill_rounded(pixmap: &mut PixmapMut, rect: Rect, radius: f32, color: tiny_skia::Color) {
    let r = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let (l, t, right, b) = (rect.left(), rect.top(), rect.right(), rect.bottom());
    let mut path = PathBuilder::new();
    path.move_to(l + r, t);
    path.line_to(right - r, t);
    path.quad_to(right, t, right, t + r);
    path.line_to(right, b - r);
    path.quad_to(right, b, right - r, b);
    path.line_to(l + r, b);
    path.quad_to(l, b, l, b - r);
    path.line_to(l, t + r);
    path.quad_to(l, t, l + r, t);
    path.close();
    let Some(path) = path.finish() else { return };

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color(color);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
