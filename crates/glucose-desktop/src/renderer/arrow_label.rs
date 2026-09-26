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
use glucose_core::types::Annotation;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Transform};

// # BADGE-1 — l'étiquette et le badge vivent dans le monde
//
// Ils étaient en pixels d'écran, par un choix argumenté — « une légende doit rester lisible
// quand on prend du recul ». Il l'a jugé à l'écran le 26/09 : dézoomé très loin, une pastille
// de relation plus grande qu'un groupe entier d'images, *« et ça, c'est un gros problème »*.
// Chez Tauri, tout le calque des flèches est mis à l'échelle du zoom (`translate(…) scale(…)`) :
// la pastille de rayon 10 et l'étiquette de corps 11 sont des longueurs **du monde** ; seul le
// trait garde son épaisseur à l'écran. C'est la loi de la fiche 05 § 4.4 — une seule
// transformation pour ce qui appartient au monde — et le seuil de détail commun (SCALE-2) les
// retire quand ils deviendraient illisibles, comme le texte d'une carte.

/// Corps de l'étiquette, en unités du monde (Glucose Tauri : `fontSize={11}`).
const LABEL_CORPS: f32 = 11.0;

/// Marge horizontale autour du texte, en unités du monde (Glucose Tauri : `x={-w/2 - 4}`).
const LABEL_PAD_X: f32 = 5.0;

/// Demi-hauteur de la pastille, en unités du monde (Glucose Tauri : `y={-11} height={22}`).
const LABEL_HALF_H: f32 = 11.0;

/// Rayon des coins de la pastille, en unités du monde (Glucose Tauri : `rx={3}`).
const LABEL_RADIUS: f32 = 3.0;

/// Ce dont la barre de saisie s'écarte du bord haut et du bord bas de la pastille.
const CARET_INSET: f32 = 3.0;

/// Décalage vertical de l'étiquette quand un prédicat partage le même point d'ancrage, en
/// unités du monde.
///
/// Les deux se posent au milieu du tracé : sans décalage, ils se superposeraient. Glucose
/// Tauri monte l'étiquette de 14 et descend le badge d'autant.
pub(crate) const LABEL_LIFT: f64 = 14.0;

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

/// **L'étiquette de cette flèche se montre-t-elle ?** — un texte posé, ou celui qu'on y écrit.
/// Le badge descend quand elle se montre ; le dessin et le survol le demandent ici tous deux.
pub(crate) fn montre_une_etiquette(arrow: &Annotation, editing: Option<&TextEditSession>) -> bool {
    shown_text(arrow, editing).is_some()
}

/// Pose l'étiquette d'une flèche sur son tracé, si elle en a une ou qu'on en écrit une.
///
/// `lifted` monte la pastille pour laisser la place au badge de prédicat, qui vise le même
/// point.
///
/// `(milieu, encre)` : le point du tracé où elle se pose, en coordonnées du monde, et la
/// couleur de son texte — la teinte médiane de la flèche, comme chez Tauri (`fill={colMid}`).
pub(super) fn draw_arrow_label(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    ((wx, wy), encre): ((f64, f64), tiny_skia::Color),
    arrow: &Annotation,
    editing: Option<&TextEditSession>,
    lifted: bool,
) {
    let Some((text, caret)) = shown_text(arrow, editing) else {
        return;
    };
    let (typography, theme, scale) = (ctx.typography, ctx.theme, ctx.scale);
    // SCALE-2 : sous le seuil de détail, une étiquette n'a plus rien à dire — sauf celle qu'on
    // est en train d'écrire.
    if !scale.draws_detail() && caret.is_none() {
        return;
    }
    let leve = if lifted { LABEL_LIFT } else { 0.0 };
    let (sx, sy) = world_to_screen(wx, wy - leve, &ctx.vp);
    let (cx, cy) = (sx as f32, sy as f32);

    let size = scale.world(LABEL_CORPS);
    // La largeur **et** la hauteur mesurées, par la fonction qui dessinera les glyphes
    // (LABEL-1). La hauteur sert à centrer le texte dans sa pastille : `draw_text` prend le
    // **haut** de la ligne, et non sa ligne de base.
    let (text_w, text_h) = typography.measure_text(text, size, Face::Regular);
    let pad = scale.world(LABEL_PAD_X);
    let half_h = scale.world(LABEL_HALF_H);
    let half_w = text_w / 2.0 + pad;

    let Some(pastille) = Rect::from_ltrb(cx - half_w, cy - half_h, cx + half_w, cy + half_h) else {
        return;
    };
    fill_rounded(
        pixmap,
        pastille,
        scale.world(LABEL_RADIUS),
        theme.arrow_label_bg,
    );

    typography.draw_text(
        pixmap,
        text,
        cx - text_w / 2.0,
        cy - text_h / 2.0,
        TextStyle {
            size,
            color: encre,
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
    let retrait = ctx.scale.world(CARET_INSET);
    let Some(barre) = Rect::from_ltrb(
        x,
        cy - half_h + retrait,
        x + CARET_WIDTH,
        cy + half_h - retrait,
    ) else {
        return;
    };
    // **SCALE-3**, et pas un `fill_rect` : une barre d'un pixel et demi est un « hairline »
    // pour tiny-skia, qui panique dessus (`hairline_aa.rs`, `assert!(false)`). C'est la
    // troisième fois que ce crash apparaît dans ce projet ; les deux précédentes avaient été
    // corrigées sur place, et c'est précisément pour cela qu'il est revenu. Le cliquet 5 le
    // ferme désormais pour de bon.
    super::scale::fill_crisp(pixmap, barre, ctx.theme.arrow_label_text);
}

/// Un rectangle aux coins arrondis, rempli d'une couleur unie — par le traceur commun, en vrais
/// arcs (ARC-1). Une copie en paraboles survivait ici.
fn fill_rounded(pixmap: &mut PixmapMut, rect: Rect, radius: f32, color: tiny_skia::Color) {
    let mut path = PathBuilder::new();
    crate::renderer::push_rounded_rect(
        &mut path,
        rect.left(),
        rect.top(),
        rect.width(),
        rect.height(),
        radius.max(0.0),
    );
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
