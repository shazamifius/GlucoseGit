//! Journal d'éditions — l'undo coûte la taille de la modification, jamais celle du document.
//!
//! # Pourquoi ce module existe
//!
//! Le mécanisme historique ([`Store::push_undo`]) empile un clone complet du [`Project`] avant
//! chaque mutation, depuis 34 points d'appel. Mesuré au banc `bench_store`, allocateur
//! compteur à l'appui :
//!
//! | nœuds | une mutation | pile pleine (200 niveaux) |
//! |------:|-------------:|--------------------------:|
//! | 1 000 | 0,28 ms | 95 Mo |
//! | 100 000 | 38,16 ms | 9,5 Go |
//! | 1 000 000 | 333,49 ms | 95 Go |
//!
//! Déplacer une carte d'un pixel dans un document de 100 000 nœuds coûtait donc près de
//! quatre fois le budget d'une frame entière.
//!
//! # La loi de coût (JRN-1)
//!
//! Pour un geste touchant `k` éléments dans un document de `n` nœuds :
//!
//! ```text
//!     T(n, k) = α + β·k        avec        ∂T/∂n = 0
//! ```
//!
//! Le coût d'une modification est la taille de la modification. Il ne dépend **pas** de la
//! taille du document. C'est la loi L3 de l'architecture cible, rendue exécutable : le banc
//! mesure le rapport `T(10⁶) / T(10³)`, qui valait 1191 et doit tendre vers 1.
//!
//! # Le principe : un emplacement, deux états
//!
//! Insérer, supprimer et modifier ne sont pas trois opérations mais une seule, vue sous trois
//! angles. Un [`Slot`] désigne une case d'une liste et porte ce qu'elle contenait **avant** et
//! ce qu'elle contient **après** :
//!
//! | Geste | `before` | `after` |
//! |---|---|---|
//! | insertion | `None` | `Some(v)` |
//! | suppression | `Some(v)` | `None` |
//! | modification | `Some(a)` | `Some(b)` |
//!
//! Défaire, c'est échanger les deux. Il n'y a donc **ni drapeau, ni variante d'opération, ni
//! branche de code par geste** : la sémantique émerge de la présence des valeurs, et `undo` et
//! `redo` sont la même fonction appelée dans l'autre sens.
//!
//! # Invariants
//!
//! - **JRN-1** — le coût d'une entrée est proportionnel aux éléments touchés, jamais à `n`.
//! - **JRN-2** — les index d'un [`Slot`] sont valides au moment où l'entrée est défaite. La
//!   pile étant strictement LIFO, l'état de la liste au moment du `undo` est exactement celui
//!   qui suivait l'édition enregistrée. Toute édition hors pile (chargement d'un projet,
//!   compaction) **vide le journal** plutôt que de tenter de le réécrire.
//! - **JRN-3** — une transaction est atomique : elle se défait entièrement ou pas du tout, et
//!   ses éditions se défont dans l'ordre inverse de leur enregistrement.

use crate::types::{Annotation, BoardImage, CanvasFolder, Project, StoryboardPanel};

/// Une case de liste, avec son contenu avant et après l'édition.
///
/// `Box` maintient la taille de [`Edit`] petite quelle que soit l'entité : sans lui, l'enum
/// prendrait la taille de sa plus grosse variante (512 octets pour une `Annotation`) dans
/// chaque entrée du journal, y compris pour les gestes qui ne la concernent pas.
#[derive(Debug, Clone, PartialEq)]
pub struct Slot<T> {
    /// Position dans la liste. Voir JRN-2 pour sa validité.
    pub index: usize,
    /// Contenu avant l'édition — `None` si l'élément n'existait pas.
    pub before: Option<Box<T>>,
    /// Contenu après l'édition — `None` si l'élément a disparu.
    pub after: Option<Box<T>>,
}

impl<T> Slot<T> {
    /// Un élément apparaît.
    pub fn inserted(index: usize, value: T) -> Self {
        Self {
            index,
            before: None,
            after: Some(Box::new(value)),
        }
    }

    /// Un élément disparaît.
    pub fn removed(index: usize, value: T) -> Self {
        Self {
            index,
            before: Some(Box::new(value)),
            after: None,
        }
    }

    /// Un élément change.
    pub fn changed(index: usize, before: T, after: T) -> Self {
        Self {
            index,
            before: Some(Box::new(before)),
            after: Some(Box::new(after)),
        }
    }

    /// Échange les deux états. Défaire et refaire sont la même opération.
    fn flip(&mut self) {
        std::mem::swap(&mut self.before, &mut self.after);
    }

    /// Applique l'état `after` à la liste.
    ///
    /// Rend `false` si l'index est hors bornes — signe que JRN-2 a été rompu, c'est-à-dire
    /// qu'une édition a contourné le journal. L'appelant doit alors vider la pile plutôt que
    /// de continuer sur un état incohérent.
    fn apply(&self, list: &mut Vec<T>) -> bool
    where
        T: Clone,
    {
        match (&self.before, &self.after) {
            // Insertion : la case n'existait pas, elle apparaît.
            (None, Some(v)) => {
                if self.index > list.len() {
                    return false;
                }
                list.insert(self.index, (**v).clone());
                true
            }
            // Suppression : la case existait, elle disparaît.
            (Some(_), None) => {
                if self.index >= list.len() {
                    return false;
                }
                list.remove(self.index);
                true
            }
            // Modification : la case reste, son contenu change.
            (Some(_), Some(v)) => match list.get_mut(self.index) {
                Some(slot) => {
                    *slot = (**v).clone();
                    true
                }
                None => false,
            },
            // Ni avant ni après : l'entrée ne décrit rien. Jamais construite par les
            // constructeurs publics ; tolérée sans effet plutôt que de paniquer.
            (None, None) => true,
        }
    }
}

/// Une édition élémentaire, rattachée à la liste qu'elle modifie.
///
/// Chaque variante nomme une liste du modèle. Le tableau visé est retrouvé au moment
/// d'appliquer, ce qui évite de stocker une référence et garde l'entrée sérialisable.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Image {
        board: String,
        slot: Slot<BoardImage>,
    },
    Annotation {
        board: String,
        slot: Slot<Annotation>,
    },
    Folder {
        board: String,
        slot: Slot<CanvasFolder>,
    },
    Panel {
        board: String,
        slot: Slot<StoryboardPanel>,
    },
}

impl Edit {
    /// Identifiant du board porteur de la liste éditée.
    fn board_id(&self) -> &str {
        match self {
            Self::Image { board, .. }
            | Self::Annotation { board, .. }
            | Self::Folder { board, .. }
            | Self::Panel { board, .. } => board,
        }
    }

    fn flip(&mut self) {
        match self {
            Self::Image { slot, .. } => slot.flip(),
            Self::Annotation { slot, .. } => slot.flip(),
            Self::Folder { slot, .. } => slot.flip(),
            Self::Panel { slot, .. } => slot.flip(),
        }
    }

    /// Écrit l'état `after` dans le projet. Rend `false` si la cible est introuvable (JRN-2).
    fn apply(&self, project: &mut Project) -> bool {
        let board_id = self.board_id();
        let Some(board) = project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        match self {
            Self::Image { slot, .. } => slot.apply(&mut board.images),
            Self::Annotation { slot, .. } => slot.apply(&mut board.annotations),
            Self::Folder { slot, .. } => slot.apply(&mut board.folders),
            Self::Panel { slot, .. } => slot.apply(&mut board.panels),
        }
    }

    /// Nombre d'octets de modèle portés par cette entrée — la grandeur `k` de la loi JRN-1.
    /// Sert au plafonnement du journal en mémoire, et aux tests de coût.
    pub fn weight(&self) -> usize {
        fn w<T>(slot: &Slot<T>) -> usize {
            let unit = std::mem::size_of::<T>();
            usize::from(slot.before.is_some()) * unit + usize::from(slot.after.is_some()) * unit
        }
        match self {
            Self::Image { slot, .. } => w(slot),
            Self::Annotation { slot, .. } => w(slot),
            Self::Folder { slot, .. } => w(slot),
            Self::Panel { slot, .. } => w(slot),
        }
    }
}

/// Un geste de l'utilisateur, atomique du point de vue de l'annulation (JRN-3).
///
/// Un glisser de cinq secondes, une frappe de paragraphe ou une suppression multiple sont
/// **une** transaction : un seul Ctrl+Z les défait. C'est la contrepartie exacte de
/// `begin_live_edit` / `end_live_edit`, mais payée au poids du geste et non du document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Transaction {
    pub edits: Vec<Edit>,
}

impl Transaction {
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub fn push(&mut self, edit: Edit) {
        self.edits.push(edit);
    }

    /// Somme des poids — la taille du geste, au sens de JRN-1.
    pub fn weight(&self) -> usize {
        self.edits.iter().map(Edit::weight).sum()
    }

    /// Inverse la transaction sur place : chaque édition est retournée, et leur ordre aussi.
    ///
    /// L'inversion de l'ordre n'est pas cosmétique. Supprimer les éléments 2 puis 5 doit se
    /// défaire en réinsérant 5 puis 2 : dans l'autre sens, la réinsertion de 2 décalerait la
    /// case 5 avant qu'on y touche.
    fn invert(&mut self) {
        self.edits.reverse();
        for edit in &mut self.edits {
            edit.flip();
        }
    }

    /// Applique la transaction au projet. Rend `false` à la première édition impossible.
    fn apply(&self, project: &mut Project) -> bool {
        self.edits.iter().all(|edit| edit.apply(project))
    }
}

/// Une entrée de la pile d'annulation.
///
/// # Pourquoi deux formes coexistent (transitoire)
///
/// Les 34 sites de mutation migrent des snapshots vers le journal **un par un**, pour garder
/// les tests verts à chaque pas. Pendant cette migration, un geste migré produit une
/// [`Transaction`] et un geste non migré un [`Entry::Snapshot`].
///
/// Ces deux formes doivent vivre dans **la même pile**, pas dans deux piles parallèles : avec
/// deux piles, Ctrl+Z ne défait plus le dernier geste mais le dernier geste *de son
/// mécanisme*, et l'ordre chronologique est perdu dès que l'utilisateur alterne entre un
/// geste migré et un geste qui ne l'est pas.
///
/// La variante `Snapshot` disparaît quand le dernier site est migré. Tant qu'elle existe,
/// elle est la raison pour laquelle [`Journal::weight`] peut encore croître avec `n`.
#[derive(Debug, Clone)]
pub enum Entry {
    /// Un geste décrit par ce qu'il a changé — coût proportionnel au geste (JRN-1).
    Transaction(Transaction),
    /// Un geste décrit par l'état complet d'avant — coût proportionnel au document.
    Snapshot(Box<Project>),
}

impl Entry {
    fn weight(&self) -> usize {
        match self {
            Self::Transaction(tx) => tx.weight(),
            // Un snapshot pèse tout le document : c'est exactement ce qu'on élimine.
            Self::Snapshot(p) => p
                .boards
                .iter()
                .map(|b| {
                    b.images.len() * std::mem::size_of::<BoardImage>()
                        + b.annotations.len() * std::mem::size_of::<Annotation>()
                })
                .sum(),
        }
    }
}

/// Ce qu'un pas d'annulation vient de faire — l'appelant en a besoin pour savoir s'il doit
/// rétablir la vue (un snapshot écrase la caméra, une transaction n'y touche pas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Le document a été modifié localement. Caméra, sélection et pile de dossiers intactes.
    Local,
    /// Le document entier a été remplacé. L'appelant doit rétablir la vue (UNDO-1).
    Replaced,
}

/// La pile d'annulation : deux piles d'entrées, et une transaction ouverte.
///
/// Contrairement aux snapshots, rien ici ne grandit avec le document : une pile pleine pèse la
/// somme des gestes qu'elle contient.
#[derive(Debug, Clone, Default)]
pub struct Journal {
    done: Vec<Entry>,
    undone: Vec<Entry>,
    open: Option<Transaction>,
    /// Rang, dans `done`, du snapshot posé à l'ouverture du geste courant.
    open_snapshot: Option<usize>,
    /// Vrai si un site **non migré** a réclamé un snapshot pendant le geste courant.
    snapshot_claimed: bool,
    /// Profondeur maximale, en nombre de gestes (`LIMITS.UNDO_DEPTH`).
    pub max_depth: usize,
}

impl Journal {
    pub fn new(max_depth: usize) -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            open: None,
            open_snapshot: None,
            snapshot_claimed: false,
            max_depth,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.done.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.undone.len()
    }

    /// Efface la pile de rétablissement sans toucher au reste. Sert à `cancel_live_edit` :
    /// un geste abandonné ne doit laisser aucune trace, pas même à refaire.
    pub fn forget_redo(&mut self) {
        self.undone.clear();
    }

    /// Poids total du journal, en octets de modèle. Borné par la somme des gestes, jamais par
    /// `n` — c'est la propriété que les snapshots ne pouvaient pas tenir.
    pub fn weight(&self) -> usize {
        self.done
            .iter()
            .chain(&self.undone)
            .map(Entry::weight)
            .sum()
    }

    /// Nombre d'entrées encore stockées sous forme de snapshot. Sert de **compteur de dette
    /// de migration** : il doit atteindre zéro, et un test le surveille.
    pub fn snapshot_count(&self) -> usize {
        self.done
            .iter()
            .chain(&self.undone)
            .filter(|e| matches!(e, Entry::Snapshot(_)))
            .count()
    }

    /// Empile l'état complet d'avant un geste — le mécanisme historique, pour les sites pas
    /// encore migrés. Ignoré pendant une transaction ouverte, comme l'était `push_undo`.
    pub fn push_snapshot(&mut self, project: &Project) {
        if self.open.is_some() {
            return;
        }
        self.commit(Entry::Snapshot(Box::new(project.clone())));
    }

    /// Vide tout. Appelé quand une édition a contourné le journal (chargement d'un projet,
    /// compaction) : mieux vaut perdre l'historique que le rendre faux (JRN-2).
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
        self.open = None;
        self.open_snapshot = None;
        self.snapshot_claimed = false;
    }

    /// Ouvre une transaction. Sans appel explicite, chaque édition forme son propre geste.
    pub fn begin(&mut self) {
        if self.open.is_none() {
            self.open = Some(Transaction::default());
            self.snapshot_claimed = false;
        }
    }

    /// Ouvre un geste continu : un filet de sécurité, puis la transaction.
    ///
    /// # Pourquoi un filet, et pourquoi il disparaîtra
    ///
    /// Un geste peut encore toucher des sites non migrés, qui ne savent décrire leur
    /// modification que par un état complet d'avant. Le snapshot posé ici les couvre.
    ///
    /// **Un geste ne doit produire qu'une seule entrée** : à la fermeture, l'une des deux
    /// formes est éliminée. Si aucun site non migré n'a réclamé le filet, la transaction
    /// prend sa place ; sinon le filet reste et la transaction est jetée, puisqu'il la
    /// couvre déjà. Le jour où le dernier site sera migré, `snapshot_claimed` restera faux
    /// pour toujours et le filet ne survivra plus jamais — sans qu'aucun drapeau n'ait à
    /// être changé à la main.
    pub fn begin_gesture(&mut self, project: &Project) {
        if self.open.is_some() {
            return;
        }
        self.push_snapshot(project);
        self.open_snapshot = self.done.len().checked_sub(1);
        self.begin();
    }

    /// Signale qu'un site non migré a besoin du filet de sécurité du geste courant.
    pub fn claim_snapshot(&mut self) {
        if self.open.is_some() {
            self.snapshot_claimed = true;
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Enregistre une édition : dans la transaction ouverte, ou seule dans la sienne.
    pub fn record(&mut self, edit: Edit) {
        match &mut self.open {
            Some(tx) => tx.push(edit),
            None => {
                let mut tx = Transaction::default();
                tx.push(edit);
                self.commit(Entry::Transaction(tx));
            }
        }
    }

    /// Ferme le geste ouvert, en ne laissant qu'**une seule** entrée.
    ///
    /// Une transaction vide — un clic qui n'a rien bougé — n'ajoute rien de son côté.
    pub fn end(&mut self) {
        let Some(tx) = self.open.take() else {
            return;
        };
        let filet = self.open_snapshot.take();
        let claimed = std::mem::take(&mut self.snapshot_claimed);

        // Rien de journalisé : le geste est décrit par le filet, s'il y en a un.
        if tx.is_empty() {
            return;
        }
        // Un site non migré s'est exprimé : le filet couvre tout, y compris la transaction.
        if claimed {
            return;
        }
        // Tous les sites touchés étaient migrés : la transaction remplace le filet.
        if let Some(i) = filet {
            if matches!(self.done.get(i), Some(Entry::Snapshot(_))) {
                self.done.remove(i);
            }
        }
        self.commit(Entry::Transaction(tx));
    }

    /// Abandonne la transaction ouverte en défaisant ce qu'elle a déjà écrit.
    /// Rend `false` hors transaction.
    pub fn cancel(&mut self, project: &mut Project) -> bool {
        let Some(mut tx) = self.open.take() else {
            return false;
        };
        self.open_snapshot = None;
        self.snapshot_claimed = false;
        if tx.is_empty() {
            return true;
        }
        tx.invert();
        if !tx.apply(project) {
            self.clear();
        }
        true
    }

    fn commit(&mut self, entry: Entry) {
        self.done.push(entry);
        if self.done.len() > self.max_depth {
            self.done.remove(0);
        }
        self.undone.clear();
    }

    /// Défait le dernier geste. Rend `None` s'il n'y a rien à défaire.
    pub fn undo(&mut self, project: &mut Project) -> Option<Step> {
        self.step(project, true)
    }

    /// Refait le dernier geste défait. Rend `None` s'il n'y a rien à refaire.
    pub fn redo(&mut self, project: &mut Project) -> Option<Step> {
        self.step(project, false)
    }

    /// Défaire et refaire sont le même mouvement, entre deux piles échangées.
    ///
    /// Les deux formes d'entrée se défont par la même idée — échanger l'avant et l'après.
    /// Pour une transaction c'est [`Slot::flip`] ; pour un snapshot c'est `mem::swap` entre le
    /// document et l'entrée. Dans les deux cas l'opération est **sa propre inverse**, ce qui
    /// est la raison pour laquelle `undo` et `redo` n'ont pas besoin de code distinct.
    fn step(&mut self, project: &mut Project, backward: bool) -> Option<Step> {
        let (from, to) = if backward {
            (&mut self.done, &mut self.undone)
        } else {
            (&mut self.undone, &mut self.done)
        };
        match from.pop()? {
            Entry::Transaction(mut tx) => {
                tx.invert();
                if !tx.apply(project) {
                    // JRN-2 rompu : le document n'est plus celui qu'on croyait. On vide plutôt
                    // que de laisser une pile qui ment.
                    self.clear();
                    return None;
                }
                to.push(Entry::Transaction(tx));
                Some(Step::Local)
            }
            Entry::Snapshot(mut snapshot) => {
                std::mem::swap(project, &mut snapshot);
                to.push(Entry::Snapshot(snapshot));
                Some(Step::Replaced)
            }
        }
    }
}

#[cfg(test)]
mod tests;
