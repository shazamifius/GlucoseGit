//! La rotation d'un nœud : l'angle qu'un glisser lui donne.
//!
//! # ROT-1 — l'angle est celui de la main, pas un incrément
//!
//! Faire tourner, c'est saisir un point du nœud et l'emmener ailleurs autour de son centre.
//! L'angle appliqué est donc **la différence des deux azimuts** — celui du point saisi, celui
//! du curseur — ajoutée à l'angle que le nœud avait déjà. Rien ne s'accumule d'une image à
//! l'autre : recalculer depuis l'origine du geste à chaque mouvement rend le résultat
//! indépendant du nombre d'événements reçus, donc identique quelle que soit la fréquence de
//! la souris.
//!
//! C'est la même discipline que le redimensionnement, qui repart lui aussi de la boîte du
//! début plutôt que d'ajouter des deltas.
//!
//! # La contrainte est dérivée, pas choisie
//!
//! `Maj` verrouille l'angle sur les **huit directions des poignées**. Ce n'est pas un pas de
//! 45° décidé au hasard : c'est le seul jeu de directions que ce nœud connaisse déjà, celui
//! de ses propres prises. Une constante de moins, et un résultat que l'œil reconnaît —
//! chaque cran amène une poignée exactement là où était sa voisine.

/// Le nombre de directions que porte un nœud : ses huit poignées.
///
/// Il vit ici plutôt qu'en face de `Handle::ALL` parce que c'est *sa longueur* qui a un sens
/// géométrique, et un test tient les deux ensemble.
pub const OCTANTS: usize = 8;

/// L'angle, en radians, qu'un glisser de rotation donne au nœud.
///
/// `center` est le centre du nœud, `grabbed` le point du monde saisi au début du geste,
/// `pointer` la position courante, `start` l'angle que le nœud avait alors. `snap` verrouille
/// le résultat sur les huit directions des poignées.
///
/// Un point saisi **au centre même** n'a pas d'azimut : le nœud garde alors son angle, plutôt
/// que d'en prendre un arbitraire.
pub fn rotation_from_drag(
    center: (f64, f64),
    grabbed: (f64, f64),
    pointer: (f64, f64),
    start: f64,
    snap: bool,
) -> f64 {
    let (Some(a0), Some(a1)) = (azimut(center, grabbed), azimut(center, pointer)) else {
        return start;
    };
    let angle = start + a1 - a0;
    if snap {
        let cran = std::f64::consts::TAU / OCTANTS as f64;
        (angle / cran).round() * cran
    } else {
        angle
    }
}

/// Où tombe, dans le monde, un point de la boîte d'un nœud tourné.
///
/// `offset` est sa position **relative au centre**, boîte droite ; le résultat est sa
/// position réelle une fois le nœud tourné de `rotation` autour de ce centre.
///
/// Une seule formule pour tout le monde : l'arbitre de clic place les poignées avec elle, le
/// rendu dessine le cadre avec elle. Les deux ne peuvent donc pas diverger — et c'est
/// exactement la faute qu'ils faisaient, l'un tenant compte de la rotation et l'autre pas.
pub fn place(center: (f64, f64), offset: (f64, f64), rotation: f64) -> (f64, f64) {
    if rotation == 0.0 {
        return (center.0 + offset.0, center.1 + offset.1);
    }
    let (c, s) = (rotation.cos(), rotation.sin());
    (
        center.0 + offset.0 * c - offset.1 * s,
        center.1 + offset.0 * s + offset.1 * c,
    )
}

/// L'azimut d'un point vu du centre — `None` s'il **est** le centre, où aucune direction
/// n'existe.
fn azimut(center: (f64, f64), point: (f64, f64)) -> Option<f64> {
    let (dx, dy) = (point.0 - center.0, point.1 - center.1);
    if dx == 0.0 && dy == 0.0 {
        return None;
    }
    Some(dy.atan2(dx))
}

/// Ramène un angle dans `[-π, π)`.
///
/// Un angle n'a pas de valeur absolue : `3π` et `-π` désignent la même orientation. Sans ce
/// repliement, un nœud qu'on fait tourner longtemps accumule des tours entiers — invisibles à
/// l'écran, mais qui grossissent le nombre écrit sur le disque et rendent deux documents
/// identiques comparables comme différents.
pub fn normalize(angle: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    if !angle.is_finite() {
        return 0.0;
    }
    let reste = (angle + PI).rem_euclid(TAU);
    reste - PI
}

#[cfg(test)]
mod tests;
