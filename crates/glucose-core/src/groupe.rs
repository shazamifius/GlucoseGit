//! **Transformer une sélection entière** — la mettre à l'échelle, la faire tourner — autour d'une
//! origine commune ou de l'origine de chaque nœud (fiche 53 § 10).
//!
//! # Sa demande
//!
//! *« Pour les sélections multiples, un système comme Blender : une origine combinée pour la mise
//! à l'échelle et la rotation, et un autre bouton, origines individuelles, pour mettre à
//! l'échelle toutes les images selon leur origine, et de même pour la rotation. »*
//!
//! Blender appelle cela le **point de pivot** (« Bounding Box Center », « Individual Origins »…),
//! un réglage qui vaut pour toutes les transformations ; PureRef transforme le groupe, et chaque
//! image autour de son centre quand on tient `Maj+Alt`.
//!
//! # Deux transformations, deux origines, une formule
//!
//! Une mise à l'échelle de facteur `f` autour d'une ancre `a` envoie un point `p` en
//! `a + f·(p − a)` ; une rotation d'angle `θ` autour d'un pivot `c` l'envoie en
//! `c + R(θ)·(p − c)`. Avec l'**origine commune**, l'ancre et le pivot sont ceux du groupe, et
//! le centre de chaque nœud suit la formule ; avec les **origines individuelles**, chaque nœud
//! est sa propre ancre — son centre ne bouge pas. Dans les deux cas sa taille est multipliée par
//! `f`, et son angle augmente de `θ` : la même chose arrive au nœud, seul l'endroit change.
//!
//! Tout part de la pose **de départ** du geste (ROT-1) : deux mouvements qui mènent au même
//! point donnent le même résultat, quel que soit le nombre d'événements entre les deux.

use crate::geometry::Rect;
use crate::resize::{resize_rect, Handle, ResizeRule, MIN_IMAGE_SIDE};

/// **Autour de quoi** une sélection se transforme — le point de pivot de Blender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Origine {
    /// Le groupe entier, comme un seul objet : l'ancre ou le pivot du groupe.
    #[default]
    Commune,
    /// Chaque nœud autour de son propre centre.
    Individuelle,
}

impl Origine {
    /// L'autre origine — le bouton qui bascule.
    pub fn autre(self) -> Self {
        match self {
            Self::Commune => Self::Individuelle,
            Self::Individuelle => Self::Commune,
        }
    }
}

/// La pose d'un nœud : son centre, sa taille, son angle (zéro pour ce qui ne tourne pas).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub centre: (f64, f64),
    pub taille: (f64, f64),
    pub rotation: f64,
}

/// Ce que le geste fait au groupe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transformation {
    /// Mettre à l'échelle de ce facteur, autour de cette ancre (celle du groupe).
    Echelle { facteur: f64, ancre: (f64, f64) },
    /// Tourner de cet angle, autour de ce pivot (celui du groupe).
    Rotation { angle: f64, pivot: (f64, f64) },
}

/// **Où va un point** sous cette transformation, autour de l'ancre ou du pivot du groupe.
pub fn deplacer_le_point(t: Transformation, (x, y): (f64, f64)) -> (f64, f64) {
    match t {
        Transformation::Echelle {
            facteur,
            ancre: (ax, ay),
        } => (ax + facteur * (x - ax), ay + facteur * (y - ay)),
        Transformation::Rotation {
            angle,
            pivot: (cx, cy),
        } => {
            let (s, c) = angle.sin_cos();
            let (dx, dy) = (x - cx, y - cy);
            (cx + c * dx - s * dy, cy + s * dx + c * dy)
        }
    }
}

/// **La pose d'un nœud après la transformation du groupe**, selon l'origine.
pub fn transformer(pose: Pose, t: Transformation, origine: Origine) -> Pose {
    let centre = match origine {
        Origine::Commune => deplacer_le_point(t, pose.centre),
        Origine::Individuelle => pose.centre,
    };
    match t {
        Transformation::Echelle { facteur, .. } => Pose {
            centre,
            taille: (pose.taille.0 * facteur, pose.taille.1 * facteur),
            rotation: pose.rotation,
        },
        Transformation::Rotation { angle, .. } => Pose {
            centre,
            taille: pose.taille,
            rotation: pose.rotation + angle,
        },
    }
}

/// **La mise à l'échelle qu'un coin du groupe demande** : le facteur, et l'ancre — le coin
/// opposé, qui ne bouge pas. Le rapport du groupe se garde toujours, comme celui d'une image :
/// étirer un groupe déformerait chaque nœud différemment selon sa place. Le groupe ne descend
/// pas sous la taille minimale d'une image.
pub fn echelle_du_coin(depart: Rect, coin: Handle, delta: (f64, f64)) -> Transformation {
    let regle = ResizeRule {
        min_width: MIN_IMAGE_SIDE,
        min_height: MIN_IMAGE_SIDE,
        keep_aspect: true,
    };
    let tiree = resize_rect(depart, coin, delta, regle);
    let facteur = if depart.width > 0.0 {
        tiree.width / depart.width
    } else {
        1.0
    };
    let (cx, cy) = coin.position_on(depart);
    let ancre = (
        2.0 * depart.left + depart.width - cx,
        2.0 * depart.top + depart.height - cy,
    );
    Transformation::Echelle { facteur, ancre }
}

/// **La rotation qu'un coin du groupe demande**, autour du centre du groupe : l'angle que le
/// pointeur a parcouru depuis la prise (`snap` verrouille sur les huit directions).
pub fn rotation_du_coin(
    depart: Rect,
    prise: (f64, f64),
    pointeur: (f64, f64),
    snap: bool,
) -> Transformation {
    let pivot = (
        depart.left + depart.width / 2.0,
        depart.top + depart.height / 2.0,
    );
    let angle = crate::rotate::rotation_from_drag(pivot, prise, pointeur, 0.0, snap);
    Transformation::Rotation { angle, pivot }
}

#[cfg(test)]
mod tests;
