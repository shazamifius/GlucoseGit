//! **ANCRE-1 — l'enveloppe d'une flèche se lit sur son tracé, jamais sur la copie de ses bouts.**
//!
//! Une flèche garde `(x, y)` et `(x2, y2)` : la place de ses bouts. Pour un bout libre, c'est sa
//! place. Pour un bout **accroché**, ce n'est qu'une copie de la place de son nœud, prise au
//! moment où le bout s'est posé — et le dessin ne la lit pas : il ramène le bout sur le nœud
//! ([`super::path_with`]). Trois gestes laissaient cette copie loin du dessin : choisir la flèche
//! avec un autre texte puis glisser (la translation emportait la copie, le nœud restait) ;
//! ranger un nœud dans un dossier (la flèche vise le dossier, la copie reste là où était le
//! nœud) ; un dossier déplacé, un texte qui grandit. Sa note du 23/11 (`text-kpll-375`) : *« on
//! peut faire bouger une espèce de zone de clic invisible dans le canva, et ensuite faire en
//! sorte que la flèche soit impossible à sélectionner »* — l'index, le cadre du groupe, la
//! sélection élastique et le cadrage lisaient la copie (fiche 135).
//!
//! La loi : **une seule source**. Ce qui a besoin de savoir où est une flèche le demande à son
//! tracé, comme le dessin et le clic (ARROW-1). tldraw fait de même : les bornes d'une flèche
//! liée se déduisent des formes qu'elle relie, et son index re-vérifie les flèches dont les bornes
//! dépendent d'autres formes ; Excalidraw tient une copie à jour (`updateBoundElements`) et en
//! paie une famille de bogues de copies périmées (fiche 135 § 2).

use std::collections::HashMap;

use super::{path_with, Noeuds};
use crate::geometry::Rect;
use crate::types::{Annotation, Board};

/// **L'enveloppe de ce qu'une flèche dessine** : ses bouts ramenés sur les nœuds qu'elle relie,
/// ses coudes — `None` si ce n'est pas une flèche.
pub fn enveloppe_with(arrow: &Annotation, noeuds: impl Noeuds) -> Option<Rect> {
    let points = path_with(arrow, noeuds)?;
    let (mut x0, mut y0) = (f64::INFINITY, f64::INFINITY);
    let (mut x1, mut y1) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (x, y) in points {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    Some(Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// **Ce qu'une annotation montre** : son encre ([`Annotation::encre`]) — et, pour une flèche,
/// l'enveloppe de son tracé ancré.
pub fn encre_vue(ann: &Annotation, noeuds: impl Noeuds) -> Rect {
    enveloppe_with(ann, noeuds).unwrap_or_else(|| ann.encre())
}

/// **La boîte de chaque nœud du tableau, par son nom** — pour qui demande l'enveloppe de
/// toutes ses flèches d'un coup (la sélection élastique, la lecture d'une IA) : un parcours du
/// tableau, au lieu d'un par bout de flèche.
pub fn boites_du_tableau(board: &Board) -> HashMap<&str, Rect> {
    let images = board.images.iter().map(|i| (i.id.as_str(), i.rect()));
    let annotations = (board.annotations.iter()).filter_map(|a| Some((a.id(), a.rect()?)));
    let dossiers = board.folders.iter().map(|f| (f.id.as_str(), f.rect()));
    images.chain(annotations).chain(dossiers).collect()
}

#[cfg(test)]
#[path = "enveloppe_tests.rs"]
mod tests;
