//! **La boîte que l'index donne à chaque nœud** — une image tournée, un dossier, une annotation
//! et son encre ; et une flèche accrochée, **par ses nœuds** (ANCRE-1, fiche 135).
//!
//! Une flèche accrochée se dessine d'un nœud à l'autre ; la copie de ses bouts qu'elle garde
//! (`x`, `y`, `x2`, `y2`) pouvait partir loin d'eux — un glisser qui l'emportait avec un autre
//! texte, un rangement dans un dossier. L'index la rangeait là : elle ne se choisissait plus où
//! on la voyait (l'arbitre de clic ne regarde que ce que l'index trouve près du curseur), et elle
//! pouvait cesser de se dessiner (le culling lit le même index). Sa plage se calcule donc après la
//! passe, des boîtes de ses nœuds et de ses coudes : l'enveloppe de son tracé droit y tient.
//!
//! **Sans hachage tant que rien ne change** (SPAT-3) : chaque flèche retient les nœuds où elle a
//! trouvé ses bouts, relus par leur nom à la passe suivante. **Sans écriture** (SPAT-1) : pendant
//! la passe, une flèche accrochée garde sa plage ; le recalage n'écrit que si elle a changé.

use super::{noeud_au_rang, NodeIdx, SpatialHash};
use crate::geometry::Rect;
use crate::types::{Annotation, Board, BoardImage, CanvasFolder};

/// Aucun nœud retenu pour ce bout.
const AUCUN: NodeIdx = NodeIdx::MAX;

/// Une flèche dont un bout au moins est accroché à un nœud : sa plage dépend de lui.
pub(super) fn accrochee(ann: &Annotation) -> bool {
    matches!(
        ann,
        Annotation::Arrow { source_id, target_id, .. } if source_id.is_some() || target_id.is_some()
    )
}

impl SpatialHash {
    /// Boîte d'un dossier : ancrage HAUT-GAUCHE, comme une membrane.
    pub(super) fn folder_bbox(f: &CanvasFolder) -> (f64, f64, f64, f64) {
        (f.x, f.y, f.x + f.width, f.y + f.height)
    }

    /// Boîte englobante d'une image : tournée comprise — un coin penché qui dépasse de la
    /// boîte droite doit se trouver dans les cellules qu'il couvre.
    pub(super) fn image_bbox(img: &BoardImage) -> (f64, f64, f64, f64) {
        let r = img.bounds();
        (r.left, r.top, r.right(), r.bottom())
    }

    /// Boîte englobante d'une annotation : son encre — points de passage d'une flèche et titre
    /// d'une membrane compris (TITRE-1) : un grand titre au-dessus d'une membrane encore hors de
    /// l'écran se voit, et se dessine. Pour une flèche accrochée, ce n'est que sa plage de
    /// naissance : [`SpatialHash::recaler_les_fleches`] la remplace.
    pub(super) fn annotation_bbox(ann: &Annotation) -> (f64, f64, f64, f64) {
        let r = ann.encre();
        (r.left, r.top, r.right(), r.bottom())
    }

    /// **Recale les flèches accrochées** (`(rang dans les annotations, nœud de l'index)`) sur les
    /// boîtes de leurs nœuds et leurs coudes — après la passe, quand chaque nœud a son rang.
    pub(super) fn recaler_les_fleches(&mut self, board: &Board, accrochees: &[(usize, NodeIdx)]) {
        if self.bouts.len() < self.slots.len() {
            self.bouts.resize(self.slots.len(), [AUCUN; 2]);
        }
        for &(i, idx) in accrochees {
            let Some(Annotation::Arrow {
                x,
                y,
                x2,
                y2,
                waypoints,
                source_id,
                target_id,
                ..
            }) = board.annotations.get(i)
            else {
                continue;
            };
            let point = |x: f64, y: f64| Rect::new(x, y, 0.0, 0.0);
            let depart = (self.boite_du_bout(board, idx, 0, source_id.as_deref()))
                .unwrap_or_else(|| point(*x, *y));
            let arrivee = (self.boite_du_bout(board, idx, 1, target_id.as_deref()))
                .unwrap_or_else(|| point(*x2, *y2));
            let r = (waypoints.iter()).fold(depart.union(arrivee), |r, p| r.union(point(p.x, p.y)));
            let range = self.range_of(r.left, r.top, r.right(), r.bottom());
            self.touch(idx, range);
        }
    }

    /// **La boîte du nœud où ce bout s'accroche** — `None` s'il est libre, ou si son nœud n'est
    /// plus sur le tableau (le dessin le pose alors à sa copie, et l'index aussi).
    fn boite_du_bout(
        &mut self,
        board: &Board,
        fleche: NodeIdx,
        bout: usize,
        id: Option<&str>,
    ) -> Option<Rect> {
        let id = id?;
        let retenu = self.bouts[fleche as usize][bout];
        let noeud = if self.vivant_et_nomme(retenu, id) {
            retenu
        } else {
            self.hachages += 1;
            let trouve = *self.index_of.get(id)?;
            self.bouts[fleche as usize][bout] = trouve;
            trouve
        };
        let ici = noeud_au_rang(board, self.slots[noeud as usize].rang)?;
        (ici.id() == id).then(|| ici.rect()).flatten()
    }

    /// Ce nœud de l'index vit-il encore, sous ce nom ?
    fn vivant_et_nomme(&self, noeud: NodeIdx, id: &str) -> bool {
        (self.slots.get(noeud as usize)).is_some_and(|s| s.alive)
            && &*self.names[noeud as usize] == id
    }
}
