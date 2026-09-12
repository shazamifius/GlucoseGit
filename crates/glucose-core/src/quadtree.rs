//! Index spatial accéléré pour le culling du viewport (O(cellules_du_viewport)).
//! 100% Rust Standard Library (0 dépendance).
//!
//! # Invariant SPAT-1 — l'index est incrémental, jamais reconstruit par frame
//!
//! `index_board()` est appelé par le renderer dès que `store.version` change, c'est-à-dire
//! **à chaque événement `CursorMoved` pendant un drag** (R-39). Il ne doit donc PAS vider la
//! grille : il fait une synchronisation par marquage/balayage (`stamp`) qui ne touche aux
//! cellules que pour les nœuds dont la *plage de cellules* a réellement changé. Déplacer une
//! carte de 3 px à l'intérieur de sa cellule coûte 0 écriture de grille.
//!
//! # Invariant SPAT-2 — aucune `String` allouée par entrée ni par requête
//!
//! La grille stocke des `NodeIdx` (`u32`). L'identifiant textuel est alloué **une seule fois**
//! par nœud, partagé entre la table `id -> index` et la table `index -> id` via `Rc<str>`
//! (le moteur est strictement mono-thread, cf. R-40). Les requêtes rendent des `&str`
//! empruntés à l'index : zéro allocation par frame.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Index entier stable d'un nœud dans l'index spatial.
type NodeIdx = u32;

/// Plage de cellules de la grille couverte par une boîte englobante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellRange {
    min_ci: i32,
    min_cj: i32,
    max_ci: i32,
    max_cj: i32,
}

impl CellRange {
    fn cells(&self) -> impl Iterator<Item = (i32, i32)> {
        let (min_cj, max_cj) = (self.min_cj, self.max_cj);
        (self.min_ci..=self.max_ci).flat_map(move |ci| (min_cj..=max_cj).map(move |cj| (ci, cj)))
    }
}

/// Emplacement d'un nœud indexé. `stamp` sert au balayage de `index_board`.
#[derive(Debug, Clone, Copy)]
struct Slot {
    range: CellRange,
    stamp: u64,
    alive: bool,
}

pub struct SpatialHash {
    cell_size: f64,
    grid: HashMap<(i32, i32), Vec<NodeIdx>>,
    /// index -> identifiant (partagé avec `index_of` : une seule allocation par nœud).
    names: Vec<Rc<str>>,
    /// identifiant -> index.
    index_of: HashMap<Rc<str>, NodeIdx>,
    slots: Vec<Slot>,
    free: Vec<NodeIdx>,
    /// Ordre des nœuds tel que le board les a présentés à la passe précédente. Sert de cache
    /// de résolution `id -> index` : tant que le board ne change pas d'ordre — le cas de
    /// TOUTES les frames d'un drag — la synchronisation ne hache plus aucune chaîne.
    order: Vec<NodeIdx>,
    stamp: u64,
    /// Nombre de reconstructions totales (`clear` / `build`). Instrumentation SPAT-1.
    rebuilds: u64,
    /// Nombre d'écritures dans une cellule de la grille. Instrumentation SPAT-1.
    cell_writes: u64,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size: if cell_size > 0.0 { cell_size } else { 2000.0 },
            grid: HashMap::new(),
            names: Vec::new(),
            index_of: HashMap::new(),
            slots: Vec::new(),
            free: Vec::new(),
            order: Vec::new(),
            stamp: 0,
            rebuilds: 0,
            cell_writes: 0,
        }
    }

    // ── Instrumentation (loi L2 / R-39, lue par les tests et le HUD) ────────

    /// Nombre de reconstructions intégrales depuis la création.
    /// Doit rester constant pendant un drag (SPAT-1).
    pub fn rebuild_count(&self) -> u64 {
        self.rebuilds
    }

    /// Nombre d'écritures de cellule depuis la création : mesure directe du coût d'indexation.
    pub fn cell_write_count(&self) -> u64 {
        self.cell_writes
    }

    /// Nombre de nœuds actuellement indexés.
    pub fn len(&self) -> usize {
        self.index_of.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index_of.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.index_of.contains_key(id)
    }

    // ── Primitives de grille ────────────────────────────────────────────────

    fn range_of(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> CellRange {
        CellRange {
            min_ci: (min_x / self.cell_size).floor() as i32,
            min_cj: (min_y / self.cell_size).floor() as i32,
            max_ci: (max_x / self.cell_size).floor() as i32,
            max_cj: (max_y / self.cell_size).floor() as i32,
        }
    }

    fn attach(&mut self, idx: NodeIdx, range: CellRange) {
        for cell in range.cells() {
            self.grid.entry(cell).or_default().push(idx);
            self.cell_writes += 1;
        }
    }

    fn detach(&mut self, idx: NodeIdx, range: CellRange) {
        for cell in range.cells() {
            if let Some(bucket) = self.grid.get_mut(&cell) {
                bucket.retain(|&n| n != idx);
                self.cell_writes += 1;
                if bucket.is_empty() {
                    self.grid.remove(&cell);
                }
            }
        }
    }

    fn alloc_slot(&mut self, id: &str, range: CellRange) -> NodeIdx {
        let name: Rc<str> = Rc::from(id);
        let slot = Slot {
            range,
            stamp: self.stamp,
            alive: true,
        };
        let idx = match self.free.pop() {
            Some(reused) => {
                self.names[reused as usize] = Rc::clone(&name);
                self.slots[reused as usize] = slot;
                reused
            }
            None => {
                self.names.push(Rc::clone(&name));
                self.slots.push(slot);
                (self.slots.len() - 1) as NodeIdx
            }
        };
        self.index_of.insert(name, idx);
        idx
    }

    fn free_slot(&mut self, idx: NodeIdx) {
        let range = self.slots[idx as usize].range;
        self.detach(idx, range);
        self.slots[idx as usize].alive = false;
        self.free.push(idx);
    }

    // ── API incrémentale (R-39) ─────────────────────────────────────────────

    /// Insère ou met à jour un nœud. Si sa plage de cellules est inchangée, aucune écriture
    /// de grille n'a lieu (SPAT-1).
    pub fn insert(&mut self, id: &str, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        let range = self.range_of(min_x, min_y, max_x, max_y);
        self.upsert(id, range);
    }

    /// Alias historique de [`SpatialHash::insert`] (conservé : appelants existants).
    pub fn insert_bbox(&mut self, id: &str, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.insert(id, min_x, min_y, max_x, max_y);
    }

    /// Met à jour la position d'un nœud. Rend `false` si l'identifiant était inconnu — le nœud
    /// est alors inséré (`update` est un upsert, comme [`SpatialHash::insert`]).
    pub fn update(&mut self, id: &str, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> bool {
        let known = self.contains(id);
        self.insert(id, min_x, min_y, max_x, max_y);
        known
    }

    /// Retire un nœud de l'index. Rend `false` si l'identifiant était inconnu.
    pub fn remove(&mut self, id: &str) -> bool {
        let Some(idx) = self.index_of.remove(id) else {
            return false;
        };
        self.free_slot(idx);
        true
    }

    fn upsert(&mut self, id: &str, range: CellRange) -> NodeIdx {
        match self.index_of.get(id).copied() {
            Some(idx) => {
                self.touch(idx, range);
                idx
            }
            None => {
                let idx = self.alloc_slot(id, range);
                self.attach(idx, range);
                idx
            }
        }
    }

    /// Rafraîchit un nœud déjà résolu. Aucune écriture de grille s'il n'a pas changé de
    /// cellule — c'est le cas de tous les petits déplacements (SPAT-1).
    fn touch(&mut self, idx: NodeIdx, range: CellRange) {
        let previous = self.slots[idx as usize].range;
        self.slots[idx as usize].stamp = self.stamp;
        if previous == range {
            return;
        }
        self.detach(idx, previous);
        self.slots[idx as usize].range = range;
        self.attach(idx, range);
    }

    // ── Boîtes englobantes du modèle ────────────────────────────────────────

    /// Boîte d'une image : ancrage au CENTRE (convention historique du culling).
    /// Boîte d'un dossier : ancrage HAUT-GAUCHE, comme une membrane.
    fn folder_bbox(f: &crate::types::CanvasFolder) -> (f64, f64, f64, f64) {
        (f.x, f.y, f.x + f.width, f.y + f.height)
    }

    fn image_bbox(img: &crate::types::BoardImage) -> (f64, f64, f64, f64) {
        let hw = img.width / 2.0;
        let hh = img.height / 2.0;
        (img.x - hw, img.y - hh, img.x + hw, img.y + hh)
    }

    /// Boîte d'une annotation : ancrage HAUT-GAUCHE (loi L7), sauf flèche (enveloppe des deux
    /// extrémités).
    fn annotation_bbox(ann: &crate::types::Annotation) -> (f64, f64, f64, f64) {
        use crate::types::Annotation as A;
        match ann {
            A::Text {
                x,
                y,
                width,
                height,
                ..
            } => (
                *x,
                *y,
                *x + width.unwrap_or(240.0),
                *y + height.unwrap_or(48.0),
            ),
            A::Sticky {
                x,
                y,
                width,
                height,
                ..
            } => (
                *x,
                *y,
                *x + width.unwrap_or(160.0),
                *y + height.unwrap_or(120.0),
            ),
            A::Membrane {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, *x + *width, *y + *height),
            A::Arrow { x, y, x2, y2, .. } => (x.min(*x2), y.min(*y2), x.max(*x2), y.max(*y2)),
        }
    }

    pub fn insert_image(&mut self, img: &crate::types::BoardImage) {
        let (a, b, c, d) = Self::image_bbox(img);
        self.insert(&img.id, a, b, c, d);
    }

    pub fn insert_annotation(&mut self, ann: &crate::types::Annotation) {
        let (a, b, c, d) = Self::annotation_bbox(ann);
        self.insert(ann.id(), a, b, c, d);
    }

    /// Synchronise l'index sur l'état d'un board **sans reconstruction** (SPAT-1).
    ///
    /// Marquage : chaque nœud présent reçoit l'estampille courante. Balayage : tout nœud resté
    /// à une estampille antérieure a disparu du board et est retiré.
    pub fn index_board(&mut self, board: &crate::types::Board) {
        self.stamp += 1;
        let mut k = 0usize;
        for img in &board.images {
            let (a, b, c, d) = Self::image_bbox(img);
            let range = self.range_of(a, b, c, d);
            self.sync_at(&img.id, range, &mut k);
        }
        for ann in &board.annotations {
            let (a, b, c, d) = Self::annotation_bbox(ann);
            let range = self.range_of(a, b, c, d);
            self.sync_at(ann.id(), range, &mut k);
        }
        // Les dossiers sont des nœuds du canevas comme les autres. Sans eux ici, un dossier
        // seul sur un tableau n'était **pas cliquable du tout** : `collect_candidates_indexed`
        // rend une liste vide quand l'index ne voit rien à proximité, et il ne voyait jamais
        // un dossier.
        for f in &board.folders {
            let (a, b, c, d) = Self::folder_bbox(f);
            let range = self.range_of(a, b, c, d);
            self.sync_at(&f.id, range, &mut k);
        }
        self.order.truncate(k);

        // Un nœud a disparu si, et seulement si, le board en a présenté moins que l'index n'en
        // contient : les identifiants d'un board sont uniques, donc `k` compte des nœuds
        // distincts. Sans écart, le balayage O(n) est inutile.
        if k != self.index_of.len() {
            self.sweep();
        }
    }

    /// Synchronise le k-ième nœud de la passe.
    ///
    /// Chemin rapide : le board présente ses nœuds dans le même ordre qu'à la passe
    /// précédente — le cas de toutes les frames d'un drag — donc l'index est déjà connu et
    /// une comparaison de chaînes suffit à le confirmer. Aucun hachage, aucune allocation.
    fn sync_at(&mut self, id: &str, range: CellRange, k: &mut usize) {
        let cached = self
            .order
            .get(*k)
            .copied()
            .filter(|&idx| self.slots[idx as usize].alive && &*self.names[idx as usize] == id);
        let idx = match cached {
            Some(idx) => {
                self.touch(idx, range);
                idx
            }
            None => self.upsert(id, range),
        };
        match self.order.get_mut(*k) {
            Some(slot) => *slot = idx,
            None => self.order.push(idx),
        }
        *k += 1;
    }

    /// Retire les nœuds absents du dernier `index_board`.
    fn sweep(&mut self) {
        let current = self.stamp;
        let stale: Vec<NodeIdx> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive && s.stamp != current)
            .map(|(i, _)| i as NodeIdx)
            .collect();
        for idx in stale {
            let name = Rc::clone(&self.names[idx as usize]);
            self.index_of.remove(&name);
            self.free_slot(idx);
        }
    }

    /// Reconstruction intégrale à partir d'une liste (id, x, y, largeur, hauteur) centrée.
    /// Réservée à l'indexation initiale : comptée par [`SpatialHash::rebuild_count`].
    pub fn build<'a, I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (&'a str, f64, f64, f64, f64)>,
    {
        self.clear();
        for (id, x, y, width, height) in items {
            let (hw, hh) = (width / 2.0, height / 2.0);
            self.insert(id, x - hw, y - hh, x + hw, y + hh);
        }
    }

    // ── Requêtes ────────────────────────────────────────────────────────────

    /// Requête de visibilité sans allocation d'identifiant (chemin du renderer et du picking).
    pub fn query_rect_refs(
        &self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        margin: f64,
    ) -> HashSet<&str> {
        let range = self.range_of(
            min_x - margin,
            min_y - margin,
            max_x + margin,
            max_y + margin,
        );
        let mut out = HashSet::new();
        for cell in range.cells() {
            if let Some(bucket) = self.grid.get(&cell) {
                for &idx in bucket {
                    out.insert(&*self.names[idx as usize]);
                }
            }
        }
        out
    }

    /// Variante possédée. Alloue une `String` par élément visible : préférer
    /// [`SpatialHash::query_rect_refs`] dans toute boucle de rendu ou de picking.
    pub fn query_rect(
        &self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        margin: f64,
    ) -> HashSet<String> {
        self.query_rect_refs(min_x, min_y, max_x, max_y, margin)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn query_ids(&self, x: f64, y: f64, w: f64, h: f64, margin: f64) -> HashSet<String> {
        self.query_rect(x, y, x + w, y + h, margin)
    }

    /// Vide intégralement l'index. Comptée comme une reconstruction (SPAT-1).
    pub fn clear(&mut self) {
        self.grid.clear();
        self.names.clear();
        self.index_of.clear();
        self.slots.clear();
        self.free.clear();
        self.order.clear();
        self.rebuilds += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_query() {
        let mut sh = SpatialHash::new(1000.0);
        let items = [
            ("img1", 500.0, 500.0, 100.0, 100.0),
            ("img2", 2500.0, 2500.0, 100.0, 100.0),
        ];
        sh.build(items);

        let visible = sh.query_ids(0.0, 0.0, 1000.0, 1000.0, 0.0);
        assert!(visible.contains("img1"));
        assert!(!visible.contains("img2"));
    }

    #[test]
    fn test_insert_remove_update_incremental() {
        let mut sh = SpatialHash::new(1000.0);
        sh.insert("a", 0.0, 0.0, 10.0, 10.0);
        assert!(sh.contains("a"));
        assert_eq!(sh.len(), 1);

        assert!(sh.update("a", 5000.0, 5000.0, 5010.0, 5010.0));
        assert!(sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0).is_empty());
        assert!(sh
            .query_rect_refs(4900.0, 4900.0, 5100.0, 5100.0, 0.0)
            .contains("a"));

        assert!(sh.remove("a"));
        assert!(!sh.remove("a"));
        assert!(sh.is_empty());
        assert!(sh
            .query_rect_refs(4900.0, 4900.0, 5100.0, 5100.0, 0.0)
            .is_empty());
    }

    #[test]
    fn test_reindex_same_id_does_not_duplicate() {
        let mut sh = SpatialHash::new(1000.0);
        for _ in 0..10 {
            sh.insert("a", 0.0, 0.0, 10.0, 10.0);
        }
        assert_eq!(sh.len(), 1);
        assert_eq!(sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0).len(), 1);
        // Une seule écriture de cellule pour 10 insertions identiques (SPAT-1).
        assert_eq!(sh.cell_write_count(), 1);
    }

    #[test]
    fn test_free_slot_is_reused_after_remove() {
        let mut sh = SpatialHash::new(1000.0);
        sh.insert("a", 0.0, 0.0, 10.0, 10.0);
        sh.remove("a");
        sh.insert("b", 0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            sh.slots.len(),
            1,
            "l'emplacement libéré doit être réutilisé"
        );
        let hit = sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(hit.contains("b"));
        assert!(!hit.contains("a"));
    }
}
