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
    /// Le rang auquel le tableau a présenté ce nœud — ses images, puis ses annotations, puis
    /// ses dossiers (CULL-1).
    ///
    /// C'est ce qui permet au culling de rendre des **positions** plutôt que des noms. Une
    /// passe de rendu peut alors aller droit aux nœuds visibles, au lieu de parcourir le
    /// document entier en demandant de chacun s'il est dans l'ensemble des visibles.
    rang: u32,
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
    /// Ordre des nœuds tel que le board les a présentés à la passe **précédente**. Sert de
    /// cache de résolution `id -> index` : tant que le board ne change pas d'ordre — le cas
    /// de TOUTES les frames d'un drag — la synchronisation ne hache plus aucune chaîne.
    ordre_precedent: Vec<NodeIdx>,
    /// L'ordre en cours de construction. Il est distinct du précédent, et non écrit par-dessus,
    /// parce que le chemin rapide a besoin de relire l'ancien ordre **intact** à des places
    /// déjà dépassées (SPAT-3).
    ordre: Vec<NodeIdx>,
    stamp: u64,
    /// Nombre de reconstructions totales (`clear` / `build`). Instrumentation SPAT-1.
    rebuilds: u64,
    /// Nombre d'écritures dans une cellule de la grille. Instrumentation SPAT-1.
    cell_writes: u64,
    /// Nombre de résolutions par la table de hachage. Instrumentation SPAT-3.
    hachages: u64,
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
            ordre_precedent: Vec::new(),
            ordre: Vec::new(),
            stamp: 0,
            rebuilds: 0,
            cell_writes: 0,
            hachages: 0,
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

    /// Nombre de résolutions passées par la table de hachage (SPAT-3).
    ///
    /// Une insertion au milieu du document doit en coûter **un**, pas un par nœud qui suit.
    /// Sans ce compteur, l'invariant ne se vérifierait qu'au chronomètre.
    pub fn hash_count(&self) -> u64 {
        self.hachages
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
            // Le rang est posé par `sync_at`, juste après, quand la place du nœud est connue.
            rang: 0,
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
        let r = img.rect();
        (r.left, r.top, r.right(), r.bottom())
    }

    /// Boîte englobante d'une annotation : celle du modèle, points de passage d'une flèche
    /// compris.
    fn annotation_bbox(ann: &crate::types::Annotation) -> (f64, f64, f64, f64) {
        let r = ann.bounds();
        (r.left, r.top, r.right(), r.bottom())
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
        self.ordre.clear();
        let mut glissement = 0i64;
        for img in &board.images {
            let (a, b, c, d) = Self::image_bbox(img);
            let range = self.range_of(a, b, c, d);
            self.sync_at(&img.id, range, &mut glissement);
        }
        for ann in &board.annotations {
            let (a, b, c, d) = Self::annotation_bbox(ann);
            let range = self.range_of(a, b, c, d);
            self.sync_at(ann.id(), range, &mut glissement);
        }
        // Les dossiers sont des nœuds du canevas comme les autres. Sans eux ici, un dossier
        // seul sur un tableau n'était **pas cliquable du tout** : `collect_candidates_indexed`
        // rend une liste vide quand l'index ne voit rien à proximité, et il ne voyait jamais
        // un dossier.
        for f in &board.folders {
            let (a, b, c, d) = Self::folder_bbox(f);
            let range = self.range_of(a, b, c, d);
            self.sync_at(&f.id, range, &mut glissement);
        }
        std::mem::swap(&mut self.ordre_precedent, &mut self.ordre);

        // Un nœud a disparu si, et seulement si, le board en a présenté moins que l'index n'en
        // contient : les identifiants d'un board sont uniques, donc la passe compte des nœuds
        // distincts. Sans écart, le balayage O(n) est inutile.
        if self.ordre_precedent.len() != self.index_of.len() {
            self.sweep();
        }
    }

    /// Synchronise le nœud que la passe présente maintenant.
    ///
    /// # Invariant SPAT-3 — une insertion coûte un hachage, pas un par nœud qui suit
    ///
    /// Chemin rapide : le board présente ses nœuds dans le même ordre qu'à la passe
    /// précédente — le cas de toutes les frames d'un drag — donc l'index est déjà connu et
    /// une comparaison de chaînes suffit à le confirmer. Aucun hachage, aucune allocation.
    ///
    /// Reconnaître un nœud à sa **place** avait un défaut que la mesure a mis à nu : insérer
    /// une image décale d'un cran tout ce qui la suit — les annotations, les dossiers — et
    /// chaque reconnaissance échouait alors. Mesuré à un million de nœuds, ajouter **une**
    /// image coûtait 1 192 ms, contre 54 ms pour ajouter une note, qui ne décale rien.
    ///
    /// Or une insertion décale la queue **uniformément**. Le `glissement` est cet écart, et il
    /// n'est qu'un endroit où regarder : la reconnaissance reste une comparaison de noms
    /// exacte, donc aucune justesse ne dépend de lui. Il se découvre tout seul — le premier
    /// nœud qui manque à l'appel passe par le chemin lent, et son ancien rang donne l'écart
    /// que toute la queue partage. Un hachage, puis le chemin rapide reprend.
    fn sync_at(&mut self, id: &str, range: CellRange, glissement: &mut i64) {
        let k = self.ordre.len();
        let idx = match self.retrouver_sans_hacher(id, k, *glissement) {
            Some(idx) => {
                self.touch(idx, range);
                idx
            }
            None => self.resoudre(id, range, k, glissement),
        };
        self.slots[idx as usize].rang = k as u32;
        self.ordre.push(idx);
    }

    /// Le nœud attendu au rang `k`, retrouvé sans hacher son nom — ou rien.
    ///
    /// Deux places sont plausibles : celle que le glissement courant désigne, et la place nue.
    /// Quand le glissement est nul, les deux se confondent et il n'y a qu'une comparaison.
    fn retrouver_sans_hacher(&self, id: &str, k: usize, glissement: i64) -> Option<NodeIdx> {
        let attendue = usize::try_from(k as i64 - glissement).ok();
        attendue
            .and_then(|place| self.nomme(place, id))
            .or_else(|| match attendue {
                Some(place) if place == k => None,
                _ => self.nomme(k, id),
            })
    }

    /// Le nœud vivant qui occupait `place` à la passe précédente, s'il porte bien ce nom.
    fn nomme(&self, place: usize, id: &str) -> Option<NodeIdx> {
        self.ordre_precedent
            .get(place)
            .copied()
            .filter(|&idx| self.slots[idx as usize].alive && &*self.names[idx as usize] == id)
    }

    /// Chemin lent : un hachage. Il apprend au passage de combien la queue a glissé.
    fn resoudre(&mut self, id: &str, range: CellRange, k: usize, glissement: &mut i64) -> NodeIdx {
        self.hachages += 1;
        match self.index_of.get(id).copied() {
            Some(idx) => {
                // Le nœud était déjà indexé : son rang d'avant dit l'écart que l'insertion ou
                // la suppression vient d'imposer à tout ce qui suit.
                *glissement = k as i64 - self.slots[idx as usize].rang as i64;
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
    /// Les **rangs** des nœuds qui tombent dans la fenêtre, triés et sans doublon (CULL-1).
    ///
    /// # Pourquoi des rangs et non des noms
    ///
    /// La version par noms rendait un ensemble de chaînes, et chaque passe de rendu faisait
    /// l'inverse de ce qu'il fallait : elle parcourait **tout** le tableau en demandant de
    /// chaque nœud s'il était dedans. Sur un million de nœuds dont cinq cents visibles, cela
    /// faisait un million de hachages de chaîne par passe — et il y en a quatre. Mesuré :
    /// 67 ms pour déplacer la vue, alors qu'il n'y avait rien à faire.
    ///
    /// Avec des rangs, une passe va droit aux nœuds visibles. Le coût cesse de dépendre de la
    /// taille du document pour ne plus dépendre que de ce qu'on regarde — ce qui est la seule
    /// définition utile du culling.
    ///
    /// Les rangs sont ceux de la présentation : les images du tableau, puis ses annotations,
    /// puis ses dossiers. L'appelant retranche le décalage de sa propre liste.
    pub fn query_rect_ranks(
        &self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        margin: f64,
    ) -> Vec<u32> {
        let mut out = Vec::new();
        self.query_rect_ranks_into(min_x, min_y, max_x, max_y, margin, &mut out);
        out
    }

    /// Comme [`SpatialHash::query_rect_ranks`], en réécrivant le tampon de l'appelant.
    ///
    /// Une requête par carte visible et par image — ce que fait le calcul des teintes — ne
    /// doit pas allouer une fois par carte. Le tampon appartient à celui qui interroge, et sa
    /// capacité se stabilise après quelques images.
    pub fn query_rect_ranks_into(
        &self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        margin: f64,
        out: &mut Vec<u32>,
    ) {
        let range = self.range_of(
            min_x - margin,
            min_y - margin,
            max_x + margin,
            max_y + margin,
        );
        out.clear();
        for cell in range.cells() {
            if let Some(bucket) = self.grid.get(&cell) {
                out.extend(bucket.iter().map(|&i| self.slots[i as usize].rang));
            }
        }
        // Un nœud couvre plusieurs cellules quand il est grand : le même rang revient alors
        // autant de fois. Trier puis dédupliquer coûte moins qu'un ensemble de hachage, et
        // rend au passage l'ordre de parcours séquentiel — donc favorable au cache.
        out.sort_unstable();
        out.dedup();
    }

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
        self.ordre_precedent.clear();
        self.ordre.clear();
        self.rebuilds += 1;
    }
}

mod visibles;
pub use visibles::{tous_les_rangs, Visibles};

#[cfg(test)]
mod tests;
