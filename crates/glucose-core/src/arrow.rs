//! La géométrie d'une flèche : le chemin qu'elle suit, et ce qui la désigne sous un curseur.
//!
//! # ARROW-1 — une flèche est une polyligne, et une seule fonction la décrit
//!
//! Elle part de `(x, y)`, passe par ses `waypoints` dans l'ordre, et finit en `(x2, y2)`.
//! Tout ce qui a besoin de la suivre — le test de clic, le dessin, le déplacement d'une
//! extrémité, l'étiquette posée au milieu — lit cette même suite de points. C'est la leçon
//! que la rotation a coûtée : deux endroits qui recalculent la même forme finissent par en
//! dessiner deux différentes.
//!
//! # Pourquoi une flèche n'était pas cliquable
//!
//! Glucose Tauri désigne la flèche sous le curseur par le **DOM** : une bande invisible de
//! 24 pixels d'épaisseur la double, et le navigateur dit laquelle a été touchée
//! (`ArrowSvgLayer.tsx`). L'arbitre de clic reçoit ce nom tout fait, par un champ nommé
//! `arrow_id`.
//!
//! Le portage a gardé le champ mais pas le navigateur : personne ne le remplissait, et
//! **aucune flèche n'était donc jamais sélectionnable**. Elle se voyait, elle ne se prenait
//! pas. Ici, c'est la géométrie qui répond, et le champ disparaît avec le trou.

use crate::arrow_anchor::{arrow_endpoints, ArrowAnchor};
use crate::geometry::{distance_to_segment, Rect};
use crate::types::Annotation;
use crate::types::{Board, CanvasFolder};

/// Épaisseur de la bande qui désigne une flèche, en pixels **écran**.
///
/// C'est la valeur de Glucose Tauri (`strokeWidth={24}` sur un tracé transparent, en
/// `non-scaling-stroke`, donc constante à tout zoom). Un trait de deux pixels ne se vise pas :
/// c'est la bande qui se vise, et elle vaut la même chose qu'une poignée.
pub const BAND_PX: f64 = 24.0;

/// Les points par lesquels une flèche passe, dans l'ordre.
///
/// Rend `None` pour tout ce qui n'est pas une flèche, plutôt qu'une suite vide : « ce nœud
/// n'est pas une flèche » et « cette flèche n'a pas de point » ne sont pas la même chose.
pub fn path(arrow: &Annotation) -> Option<Vec<(f64, f64)>> {
    let Annotation::Arrow {
        x,
        y,
        x2,
        y2,
        waypoints,
        ..
    } = arrow
    else {
        return None;
    };
    let mut points = Vec::with_capacity(waypoints.len() + 2);
    points.push((*x, *y));
    points.extend(waypoints.iter().map(|p| (p.x, p.y)));
    points.push((*x2, *y2));
    Some(points)
}

/// L'ancre d'une extrémité : la boîte du nœud qu'elle désigne, ou le point brut.
///
/// Une flèche qui vise un nœud ne vise pas son centre : elle s'arrête sur son **bord**, du
/// côté d'où elle vient. C'est ce que [`crate::arrow_anchor`] sait faire depuis toujours —
/// cent soixante-dix-sept lignes écrites, testées, et que personne n'appelait.
fn anchor_of(
    resolve: impl Fn(&str) -> Option<Rect>,
    id: Option<&String>,
    fallback: (f64, f64),
) -> ArrowAnchor {
    let Some(rect) = id.and_then(|id| resolve(id)) else {
        return ArrowAnchor::point(fallback.0, fallback.1);
    };
    let centre = rect.center();
    ArrowAnchor::with_box(
        centre.x,
        centre.y,
        rect.left,
        rect.right(),
        rect.top,
        rect.bottom(),
    )
}

/// La boîte d'un nœud du tableau, quelle que soit sa nature.
pub fn node_rect(board: &Board, id: &str) -> Option<Rect> {
    if let Some(img) = board.images.iter().find(|i| i.id == id) {
        return Some(img.rect());
    }
    if let Some(ann) = board.annotations.iter().find(|a| a.id() == id) {
        return ann.rect();
    }
    board
        .folders
        .iter()
        .find(|f| f.id == id)
        .map(CanvasFolder::rect)
}

/// Les points par lesquels une flèche passe **une fois ancrée** à ce qu'elle relie.
///
/// C'est la même suite que [`path`], sauf que les deux bouts sont ramenés sur le périmètre
/// des nœuds visés. Le brancher **ici** le propage partout d'un coup : le test de clic, la
/// sélection élastique et le dessin lisent tous cette fonction, donc aucun d'eux ne peut
/// voir une flèche ailleurs que là où elle est (ARROW-1).
pub fn path_with(
    arrow: &Annotation,
    resolve: impl Fn(&str) -> Option<Rect>,
) -> Option<Vec<(f64, f64)>> {
    let Annotation::Arrow {
        x,
        y,
        x2,
        y2,
        waypoints,
        source_id,
        target_id,
        ..
    } = arrow
    else {
        return None;
    };
    if source_id.is_none() && target_id.is_none() {
        return path(arrow);
    }
    let depart = anchor_of(&resolve, source_id.as_ref(), (*x, *y));
    let arrivee = anchor_of(&resolve, target_id.as_ref(), (*x2, *y2));
    // `Point2D` est le point du **document**, `Point` celui de la géométrie : deux types
    // pour une même notion, dont la fusion dépasse ce chantier. La conversion est ici, à
    // l'unique frontière où les deux se rencontrent.
    let etapes: Vec<crate::geometry::Point> = waypoints
        .iter()
        .map(|p| crate::geometry::Point::new(p.x, p.y))
        .collect();
    let bouts = arrow_endpoints(depart, arrivee, &etapes);
    let mut points = Vec::with_capacity(waypoints.len() + 2);
    points.push((bouts.start.x, bouts.start.y));
    points.extend(waypoints.iter().map(|p| (p.x, p.y)));
    points.push((bouts.end.x, bouts.end.y));
    Some(points)
}

/// La distance d'un point du monde à une flèche, ou `None` si ce n'en est pas une.
///
/// C'est la plus courte distance à l'un de ses segments : une polyligne n'est pas plus loin
/// que son tronçon le plus proche.
pub fn distance_to(arrow: &Annotation, point: (f64, f64)) -> Option<f64> {
    distance_along(&path(arrow)?, point)
}

/// La même distance, à une flèche **ancrée** : c'est là où elle se dessine qu'on la vise.
pub fn distance_to_with(
    arrow: &Annotation,
    resolve: impl Fn(&str) -> Option<Rect>,
    point: (f64, f64),
) -> Option<f64> {
    distance_along(&path_with(arrow, resolve)?, point)
}

/// La plus courte distance d'un point à une polyligne.
fn distance_along(points: &[(f64, f64)], point: (f64, f64)) -> Option<f64> {
    points
        .windows(2)
        .map(|seg| distance_to_segment(point, seg[0], seg[1]))
        .fold(None, |acc: Option<f64>, d| {
            Some(acc.map_or(d, |m| m.min(d)))
        })
}

/// La flèche la plus proche de `point`, si l'une d'elles est à portée de la bande.
///
/// À égalité, la **dernière** l'emporte : c'est celle qui est dessinée au-dessus, donc celle
/// que la main croit viser. Le même arbitrage que pour les nœuds empilés.
pub fn at(
    annotations: &[Annotation],
    resolve: impl Fn(&str) -> Option<Rect> + Copy,
    point: (f64, f64),
    scale: f64,
) -> Option<(&Annotation, f64)> {
    // La bande garde une épaisseur **écran** : une flèche ne devient pas plus dure à viser
    // parce qu'on s'est éloigné. C'est la même exception que les poignées (SCALE-1).
    let portee = BAND_PX / 2.0 / scale.max(1e-6);
    annotations
        .iter()
        .filter_map(|a| distance_to_with(a, resolve, point).map(|d| (a, d)))
        .filter(|(_, d)| *d <= portee)
        .reduce(|meilleur, courant| {
            if courant.1 <= meilleur.1 {
                courant
            } else {
                meilleur
            }
        })
}

/// Le chemin d'une flèche ancrée dans son tableau — le raccourci courant de [`path_with`].
pub fn path_in(arrow: &Annotation, board: &Board) -> Option<Vec<(f64, f64)>> {
    path_with(arrow, |id| node_rect(board, id))
}

#[cfg(test)]
mod tests;
