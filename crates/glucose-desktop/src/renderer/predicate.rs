//! Le sigle d'un prédicat de flèche : ce que la relation **veut dire**.
//!
//! Six relations nommées — précurseur, contradiction, héritage, inspiration, dépendance,
//! illustration — qu'une pastille porte au milieu de la flèche. C'est l'un des deux outils
//! que la charte désigne pour naviguer dans l'immense, l'autre étant la couleur des
//! domaines : sur un canevas de millions de nœuds, ce n'est pas la géométrie qui permet de
//! s'orienter, c'est le **sens** des liens.
//!
//! # PRED-1 — les sigles sont **dessinés**, et ce n'est pas une préférence
//!
//! Glucose Tauri écrit ses six sigles en caractères Unicode : `→ ✗ ⊂ ✦ ⊕ ◎`. Mesuré sur
//! les polices embarquées (`typography::coverage`), **quatre des six sont absents d'Inter** :
//! `⊂`, `✦`, `⊕` et `◎`. Un glyphe absent ne se dessine pas — il fait un carré vide, ou
//! rien. Le portage littéral aurait donc livré deux sigles sur six.
//!
//! Restait à choisir entre trois voies : remplacer les quatre manquants par des caractères
//! couverts — ce qui revient à choisir un sens au hasard de ce qu'une fonte contient —,
//! embarquer une police de symboles pour six formes, ou les **tracer**. Six figures
//! géométriques ne justifient pas une police de plus (charte, exigence 1), et le tracé est
//! par surcroît plus net : un sigle de neuf pixels rastérisé depuis une fonte est flou,
//! alors qu'un chemin vectoriel reste franc à toute échelle (R-46).
//!
//! Chaque sigle est donc défini dans un **carré unité** centré sur l'origine, puis mis à
//! l'échelle de la pastille. La forme ne dépend d'aucune fonte, d'aucune taille, et se
//! teste sans écran.

use super::pass::Pass;
use crate::canvas::world_to_screen;
use glucose_core::types::{Annotation, ArrowPredicate, Board};
use tiny_skia::{Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Rayon de la pastille d'un prédicat, en pixels **écran** (Glucose Tauri : `r={10}`).
///
/// Une longueur écran, comme l'étiquette : un prédicat est une **légende**, et c'est en
/// prenant du recul sur un graphe qu'on a le plus besoin de lire ses relations.
pub(super) const BADGE_RADIUS_PX: f32 = 10.0;

/// Épaisseur du cercle de la pastille, en pixels écran (Glucose Tauri : `strokeWidth={1.5}`).
const BADGE_STROKE_PX: f32 = 1.5;

/// Épaisseur du trait d'un sigle, en pixels écran.
const SIGIL_STROKE_PX: f32 = 1.4;

/// Demi-côté du carré où chaque sigle est dessiné, en fraction du rayon de la pastille.
///
/// Le sigle occupe donc un carré de `2 × 0,52` rayon — assez pour se lire, assez peu pour
/// que le cercle reste un cadre et non un étau.
const SIGIL_EXTENT: f32 = 0.52;

/// Le tracé d'un sigle, dans un carré unité centré sur l'origine.
///
/// Chaque forme est une suite de sous-chemins ; un sous-chemin est une polyligne, et un
/// sous-chemin d'un seul point est un **cercle** de rayon donné par son troisième terme.
/// C'est la description la plus courte qui couvre les six, et elle se lit.
enum Stroke2D {
    /// Une polyligne ouverte, en coordonnées unité.
    Line(&'static [(f32, f32)]),
    /// Un cercle centré sur l'origine, de ce rayon en unité.
    Circle(f32),
}

/// Les traits qui composent le sigle d'un prédicat (PRED-1).
fn sigil_of(predicate: ArrowPredicate) -> &'static [Stroke2D] {
    match predicate {
        // Une flèche vers la droite : ce qui précède mène à ce qui suit.
        ArrowPredicate::EstPrecurseur => &[
            Stroke2D::Line(&[(-1.0, 0.0), (1.0, 0.0)]),
            Stroke2D::Line(&[(0.35, -0.55), (1.0, 0.0), (0.35, 0.55)]),
        ],
        // Une croix : ce qui s'oppose.
        ArrowPredicate::Contredit => &[
            Stroke2D::Line(&[(-0.8, -0.8), (0.8, 0.8)]),
            Stroke2D::Line(&[(0.8, -0.8), (-0.8, 0.8)]),
        ],
        // Un arc ouvert à droite — l'inclusion `⊂` : ce qui est contenu dans ce qui précède.
        ArrowPredicate::HeriteDe => &[
            Stroke2D::Line(&[
                (0.7, -0.85),
                (-0.1, -0.7),
                (-0.6, -0.25),
                (-0.6, 0.25),
                (-0.1, 0.7),
                (0.7, 0.85),
            ]),
            Stroke2D::Line(&[(-0.6, 1.0), (0.85, 1.0)]),
        ],
        // Une étoile à quatre branches : l'étincelle.
        ArrowPredicate::Inspire => &[
            Stroke2D::Line(&[(0.0, -1.0), (0.0, 1.0)]),
            Stroke2D::Line(&[(-1.0, 0.0), (1.0, 0.0)]),
            Stroke2D::Line(&[(-0.45, -0.45), (0.45, 0.45)]),
            Stroke2D::Line(&[(0.45, -0.45), (-0.45, 0.45)]),
        ],
        // Une croix cerclée — le `⊕` : ce qui s'ajoute à, donc ce dont on dépend.
        ArrowPredicate::DependDe => &[
            Stroke2D::Circle(0.95),
            Stroke2D::Line(&[(-0.55, 0.0), (0.55, 0.0)]),
            Stroke2D::Line(&[(0.0, -0.55), (0.0, 0.55)]),
        ],
        // Un cercle pointé — le `◎` : ce qui montre, donc ce qui illustre.
        ArrowPredicate::Illustre => &[Stroke2D::Circle(0.95), Stroke2D::Circle(0.35)],
    }
}

/// Pose la pastille d'un prédicat sur le tracé d'une flèche.
///
/// `lowered` la descend pour laisser la place à l'étiquette, qui vise le même point.
pub(super) fn draw_arrow_predicate(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    board: &Board,
    arrow: &Annotation,
    lowered: bool,
) {
    let Annotation::Arrow {
        predicate: Some(predicate),
        ..
    } = arrow
    else {
        return;
    };
    let Some((wx, wy)) = glucose_core::arrow::label_anchor_in(arrow, board) else {
        return;
    };
    let (sx, sy) = world_to_screen(wx, wy, &ctx.vp);
    let radius = ctx.scale.screen(BADGE_RADIUS_PX);
    let cx = sx as f32;
    let cy = sy as f32
        + if lowered {
            ctx.scale.screen(super::arrow_label::LABEL_LIFT_PX)
        } else {
            0.0
        };
    let color = ctx.theme.predicate_color(*predicate);

    // Le fond d'abord : la pastille masque le trait de la flèche, pour que le sigle se lise
    // sur un aplat et non sur ce qu'elle traverse.
    let mut disque = PathBuilder::new();
    disque.push_circle(cx, cy, radius);
    if let Some(disque) = disque.finish() {
        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        paint.set_color(ctx.theme.arrow_badge_bg);
        pixmap.fill_path(
            &disque,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        paint.set_color(color);
        pixmap.stroke_path(
            &disque,
            &paint,
            &Stroke {
                width: ctx.scale.screen(BADGE_STROKE_PX),
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
    }

    draw_sigil(pixmap, *predicate, (cx, cy), radius * SIGIL_EXTENT, color);
}

/// Trace le sigle d'un prédicat, centré en `(cx, cy)` et inscrit dans un carré de `extent`.
pub(super) fn draw_sigil(
    pixmap: &mut PixmapMut,
    predicate: ArrowPredicate,
    (cx, cy): (f32, f32),
    extent: f32,
    color: tiny_skia::Color,
) {
    if !extent.is_finite() || extent <= 0.0 {
        return;
    }
    let mut path = PathBuilder::new();
    for trait_ in sigil_of(predicate) {
        match trait_ {
            Stroke2D::Line(points) => {
                let Some(((px, py), reste)) = points.split_first() else {
                    continue;
                };
                path.move_to(cx + px * extent, cy + py * extent);
                for (px, py) in reste {
                    path.line_to(cx + px * extent, cy + py * extent);
                }
            }
            Stroke2D::Circle(r) => path.push_circle(cx, cy, r * extent),
        }
    }
    let Some(path) = path.finish() else { return };

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color(color);
    pixmap.stroke_path(
        &path,
        &paint,
        &Stroke {
            width: SIGIL_STROKE_PX,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..Stroke::default()
        },
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
