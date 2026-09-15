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

use crate::geometry::distance_to_segment;
use crate::types::Annotation;

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

/// La distance d'un point du monde à une flèche, ou `None` si ce n'en est pas une.
///
/// C'est la plus courte distance à l'un de ses segments : une polyligne n'est pas plus loin
/// que son tronçon le plus proche.
pub fn distance_to(arrow: &Annotation, point: (f64, f64)) -> Option<f64> {
    let points = path(arrow)?;
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
pub fn at(annotations: &[Annotation], point: (f64, f64), scale: f64) -> Option<(&Annotation, f64)> {
    // La bande garde une épaisseur **écran** : une flèche ne devient pas plus dure à viser
    // parce qu'on s'est éloigné. C'est la même exception que les poignées (SCALE-1).
    let portee = BAND_PX / 2.0 / scale.max(1e-6);
    annotations
        .iter()
        .filter_map(|a| distance_to(a, point).map(|d| (a, d)))
        .filter(|(_, d)| *d <= portee)
        .reduce(|meilleur, courant| {
            if courant.1 <= meilleur.1 {
                courant
            } else {
                meilleur
            }
        })
}

#[cfg(test)]
mod tests;
