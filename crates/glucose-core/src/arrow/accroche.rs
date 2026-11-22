//! **Où un bout de flèche s'accroche** (ARROW-2, SNAP-3, ARROW-4) : au nœud le plus proche, s'il
//! est assez près, et sinon nulle part.
//!
//! # ARROW-4 — une membrane se vise par sa frontière (fiche 130)
//!
//! Une membrane est une **région** : elle contient d'autres nœuds, et de l'espace pour en poser.
//! Jugée comme une boîte pleine, elle était à distance nulle de tout point de son intérieur — elle
//! gagnait donc partout dedans, et même sur un texte posé en elle (deux distances nulles, et la
//! membrane vient d'abord dans l'ordre du tableau). Sa parole du 22/11 : *« impossible de placer
//! une flèche en tant qu'utilisateur à l'intérieur d'une membrane, car celle-ci se rattache
//! automatiquement à la membrane »*. Glucose Tauri ne visait jamais une membrane ; on garde la
//! possibilité de la relier, mais par ce qu'on voit d'elle : **sa frontière**. Dedans, on vise ce
//! qu'elle contient, ou le vide.

use crate::geometry::Rect;
use crate::types::{Annotation, Board};

/// La distance, en unités **monde**, à laquelle un bout de flèche s'aimante à un nœud.
///
/// C'est le `HIT_DIST` de Glucose Tauri, et c'est bien une longueur monde et non écran :
/// l'aimantation décrit un voisinage **du document** — « assez près de cette carte pour
/// vouloir la relier » — et non une tolérance de main comme la bande de sélection.
pub const SNAP_DIST: f64 = 120.0;

/// Où un bout de flèche se pose, et à quoi il s'accroche.
#[derive(Debug, Clone, PartialEq)]
pub struct Snap {
    /// Le point retenu : le curseur lui-même, ou le point du bord le plus proche.
    pub point: (f64, f64),
    /// Le nœud auquel ce bout s'accroche, s'il y en a un assez près.
    pub node: Option<String>,
}

impl Snap {
    /// Un bout qui ne s'accroche à rien : le curseur, tel quel.
    pub fn free(point: (f64, f64)) -> Self {
        Self { point, node: None }
    }
}

/// **Comment un nœud se vise** (ARROW-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visee {
    /// Partout dans sa boîte : un texte, un post-it, une photo, un dossier.
    Boite,
    /// Par sa frontière seulement : une membrane, une région qui en contient d'autres.
    Frontiere,
}

impl Visee {
    /// Comment se vise cette annotation.
    pub fn de(a: &Annotation) -> Visee {
        match a {
            Annotation::Membrane { .. } => Visee::Frontiere,
            _ => Visee::Boite,
        }
    }

    /// **Le point visé** de cette boîte, depuis `point` : le point de la boîte le plus proche
    /// (le point lui-même s'il est dedans) ; pour une frontière, le point du bord le plus proche,
    /// dedans comme dehors.
    fn point(self, rect: Rect, (x, y): (f64, f64)) -> (f64, f64) {
        let dans_la_boite = (
            x.clamp(rect.left, rect.right()),
            y.clamp(rect.top, rect.bottom()),
        );
        if self == Visee::Boite || dans_la_boite != (x, y) {
            return dans_la_boite;
        }
        // Dedans, vers le bord le plus proche : à gauche, à droite, en haut ou en bas.
        [
            (rect.left, y),
            (rect.right(), y),
            (x, rect.top),
            (x, rect.bottom()),
        ]
        .into_iter()
        .min_by(|a, b| distance(*a, (x, y)).total_cmp(&distance(*b, (x, y))))
        .unwrap_or(dans_la_boite)
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Le nœud auquel un bout de flèche s'aimante à cet endroit, et où il s'y pose (ARROW-2).
///
/// Chaque nœud est jugé sur la distance au **point de sa boîte le plus proche**, pas à son
/// centre : viser le bord d'une grande carte l'accroche, alors qu'une distance au centre
/// ferait préférer une petite carte lointaine. À l'intérieur d'une boîte, la distance est
/// nulle et le point retenu est le curseur — on pointe où l'on veut dans la carte visée.
/// Une membrane, elle, se juge sur la distance à sa frontière (ARROW-4).
///
/// `exclude` écarte les identifiants qui ne doivent pas être visés : la flèche en cours de
/// tracé, et le nœud dont elle part — sans quoi elle se refermerait sur son origine dès le
/// premier pixel de glisser.
pub fn snap_to_nearest(board: &Board, point: (f64, f64), exclude: &[&str]) -> Snap {
    let boxes = board
        .images
        .iter()
        .map(|img| (img.id.as_str(), img.rect(), Visee::Boite))
        .chain(
            board
                .annotations
                .iter()
                .filter_map(|a| Some((a.id(), a.rect()?, Visee::de(a)))),
        )
        .chain(
            board
                .folders
                .iter()
                .map(|f| (f.id.as_str(), f.rect(), Visee::Boite)),
        );
    snap_parmi(boxes, point, exclude)
}

/// **Le même choix que [`snap_to_nearest`], parmi ces candidats seulement** (SNAP-3) : ce
/// qu'un index spatial rend autour du point, là où le tableau entier se parcourait à chaque
/// mouvement de la souris.
///
/// Tout nœud à moins de [`SNAP_DIST`] a sa boîte dans le carré de ce rayon autour du point :
/// l'index qui rend ce carré rend donc tous ceux qui comptent. Les candidats viennent dans
/// l'ordre du tableau — images, annotations, dossiers — pour qu'à égalité le même l'emporte.
pub fn snap_parmi<'a>(
    candidats: impl IntoIterator<Item = (&'a str, Rect, Visee)>,
    point: (f64, f64),
    exclude: &[&str],
) -> Snap {
    let mut best = Snap::free(point);
    let mut best_dist = SNAP_DIST;
    for (id, rect, visee) in candidats {
        if exclude.contains(&id) {
            continue;
        }
        let vise = visee.point(rect, point);
        let d = distance(vise, point);
        if d < best_dist {
            best_dist = d;
            best = Snap {
                point: vise,
                node: Some(id.to_string()),
            };
        }
    }
    best
}
