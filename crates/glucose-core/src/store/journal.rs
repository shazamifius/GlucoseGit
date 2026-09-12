//! Journal d'éditions — l'undo coûte la taille de la modification, jamais celle du document.
//!
//! # Pourquoi ce module existe
//!
//! Le mécanisme d'origine (`push_undo`, aujourd'hui disparu) empilait un clone complet du
//! [`Project`] avant chaque mutation, depuis 34 points d'appel. Mesuré au banc `bench_store`,
//! allocateur compteur à l'appui :
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

use crate::types::{
    Annotation, Board, BoardImage, BoardZone, CanvasFolder, Domain, Preset, Project,
    StoryboardPanel,
};

/// Profondeur maximale de la pile d'annulation, en **nombre de gestes** (fiche 09 § 2.1).
///
/// Ce chiffre borne l'histoire, pas les octets : un geste pèse ce qu'il touche (JRN-1), donc
/// une pile pleine pèse la somme des gestes et **jamais la taille du document**. C'est la
/// garantie que le mécanisme par clichés ne pouvait pas tenir — 200 clichés d'un document de
/// 10⁶ nœuds réclamaient 95 Go — et c'est elle que vérifie
/// `test_une_pile_pleine_ne_pese_pas_la_taille_du_document`.
///
/// Y ajouter un second plafond, en octets celui-là, serait une constante arbitraire de plus,
/// et ferait mentir la première : la profondeur annoncée à l'utilisateur cesserait d'être
/// 200. Pour remonter au-delà, la fiche prévoit les jalons durables sur disque.
pub const UNDO_DEPTH: usize = 200;

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

    /// Vrai si l'avant et l'après sont identiques : la case n'a pas changé.
    fn is_noop(&self) -> bool
    where
        T: PartialEq,
    {
        self.before == self.after
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

/// Une valeur remplacée d'un bloc : un champ scalaire, ou une liste entière.
///
/// Même forme que [`Slot`] — un avant, un après, et défaire c'est échanger les deux — mais
/// sans index, parce qu'il n'y a qu'une seule case. Renommer un board ne doit pas coûter le
/// clone du board ; c'est ce que cette forme évite.
#[derive(Debug, Clone, PartialEq)]
pub struct Whole<T> {
    pub before: T,
    pub after: T,
}

impl<T> Whole<T> {
    pub fn new(before: T, after: T) -> Self {
        Self { before, after }
    }

    fn flip(&mut self) {
        std::mem::swap(&mut self.before, &mut self.after);
    }

    fn is_noop(&self) -> bool
    where
        T: PartialEq,
    {
        self.before == self.after
    }
}

/// Une édition élémentaire, rattachée à la liste qu'elle modifie.
///
/// Chaque variante nomme une liste du modèle. Le tableau visé est retrouvé au moment
/// d'appliquer, ce qui évite de stocker une référence et garde l'entrée sérialisable.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    // ── Listes portées par un board ──────────────────────────────────────
    Image { board: String, slot: Slot<BoardImage> },
    Annotation { board: String, slot: Slot<Annotation> },
    Folder { board: String, slot: Slot<CanvasFolder> },
    Panel { board: String, slot: Slot<StoryboardPanel> },
    /// Les zones sont posées en bloc par un preset : la liste entière est la modification.
    Zones { board: String, whole: Whole<Vec<BoardZone>> },
    BoardName { board: String, whole: Whole<String> },

    // ── Listes portées par le projet ─────────────────────────────────────
    /// Supprimer un board emporte tout son contenu : l'entrée est lourde, et c'est conforme
    /// à JRN-1 — la modification *est* de cette taille.
    Board { slot: Slot<Board> },
    Domain { slot: Slot<Domain> },
    Preset { slot: Slot<Preset> },
    ProjectName { whole: Whole<String> },
    ActiveBoard { whole: Whole<String> },
}

impl Edit {
    /// Identifiant du board porteur de la liste éditée, s'il y en a un.
    ///
    /// Les éditions qui portent sur le projet lui-même — ses boards, ses domaines, ses
    /// presets, son nom — n'en ont pas.
    fn board_id(&self) -> Option<&str> {
        match self {
            Self::Image { board, .. }
            | Self::Annotation { board, .. }
            | Self::Folder { board, .. }
            | Self::Panel { board, .. }
            | Self::Zones { board, .. }
            | Self::BoardName { board, .. } => Some(board),
            Self::Board { .. }
            | Self::Domain { .. }
            | Self::Preset { .. }
            | Self::ProjectName { .. }
            | Self::ActiveBoard { .. } => None,
        }
    }

    fn flip(&mut self) {
        match self {
            Self::Image { slot, .. } => slot.flip(),
            Self::Annotation { slot, .. } => slot.flip(),
            Self::Folder { slot, .. } => slot.flip(),
            Self::Panel { slot, .. } => slot.flip(),
            Self::Board { slot } => slot.flip(),
            Self::Domain { slot } => slot.flip(),
            Self::Preset { slot } => slot.flip(),
            Self::Zones { whole, .. } => whole.flip(),
            Self::BoardName { whole, .. } => whole.flip(),
            Self::ProjectName { whole } => whole.flip(),
            Self::ActiveBoard { whole } => whole.flip(),
        }
    }

    /// Écrit l'état `after` dans le projet. Rend `false` si la cible est introuvable (JRN-2).
    fn apply(&self, project: &mut Project) -> bool {
        // Ce qui porte sur le projet lui-même n'a pas de board à retrouver.
        match self {
            Self::Board { slot } => return slot.apply(&mut project.boards),
            Self::Domain { slot } => return slot.apply(&mut project.domains),
            Self::Preset { slot } => return slot.apply(&mut project.presets),
            Self::ProjectName { whole } => {
                project.name = whole.after.clone();
                return true;
            }
            Self::ActiveBoard { whole } => {
                project.active_board_id = whole.after.clone();
                return true;
            }
            _ => {}
        }

        let Some(board_id) = self.board_id() else {
            return false;
        };
        let Some(board) = project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        match self {
            Self::Image { slot, .. } => slot.apply(&mut board.images),
            Self::Annotation { slot, .. } => slot.apply(&mut board.annotations),
            Self::Folder { slot, .. } => slot.apply(&mut board.folders),
            Self::Panel { slot, .. } => slot.apply(&mut board.panels),
            Self::Zones { whole, .. } => {
                board.zones = whole.after.clone();
                true
            }
            Self::BoardName { whole, .. } => {
                board.name = whole.after.clone();
                true
            }
            // Traitées plus haut : `board_id()` les a déjà écartées en rendant `None`.
            _ => false,
        }
    }

    /// Vrai si l'édition ne change rien : même contenu avant et après.
    ///
    /// Un site qui clone, laisse une fermeture travailler et compare n'a pas à savoir si
    /// elle a travaillé pour rien — c'est le journal qui tranche, une fois pour tous.
    fn is_noop(&self) -> bool {
        match self {
            Self::Image { slot, .. } => slot.is_noop(),
            Self::Annotation { slot, .. } => slot.is_noop(),
            Self::Folder { slot, .. } => slot.is_noop(),
            Self::Panel { slot, .. } => slot.is_noop(),
            Self::Board { slot } => slot.is_noop(),
            Self::Domain { slot } => slot.is_noop(),
            Self::Preset { slot } => slot.is_noop(),
            Self::Zones { whole, .. } => whole.is_noop(),
            Self::BoardName { whole, .. } => whole.is_noop(),
            Self::ProjectName { whole } => whole.is_noop(),
            Self::ActiveBoard { whole } => whole.is_noop(),
        }
    }

    /// Nombre d'octets de modèle portés par cette entrée — la grandeur `k` de la loi JRN-1.
    /// C'est ce que mesurent le banc et les tests de coût.
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
            Self::Board { slot } => w(slot),
            Self::Domain { slot } => w(slot),
            Self::Preset { slot } => w(slot),
            Self::Zones { whole, .. } => {
                (whole.before.len() + whole.after.len()) * std::mem::size_of::<BoardZone>()
            }
            Self::BoardName { whole, .. } => whole.before.len() + whole.after.len(),
            Self::ProjectName { whole } => whole.before.len() + whole.after.len(),
            Self::ActiveBoard { whole } => whole.before.len() + whole.after.len(),
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

/// La pile d'annulation : deux piles de gestes, et un geste en cours.
///
/// Rien ici ne grandit avec le document : une pile pleine pèse la somme des gestes qu'elle
/// contient, pas la taille du canvas.
#[derive(Debug, Clone, Default)]
pub struct Journal {
    done: Vec<Transaction>,
    undone: Vec<Transaction>,
    open: Option<Transaction>,
    /// Profondeur maximale, en nombre de gestes (`LIMITS.UNDO_DEPTH`).
    pub max_depth: usize,
}

impl Journal {
    pub fn new(max_depth: usize) -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            open: None,
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

    /// Poids total du journal, en octets de modèle. Borné par la somme des gestes, jamais par
    /// `n` — c'est la propriété que les snapshots ne pouvaient pas tenir.
    pub fn weight(&self) -> usize {
        self.done
            .iter()
            .chain(&self.undone)
            .map(Transaction::weight)
            .sum()
    }

    /// Vide tout. Appelé quand une édition a contourné le journal (chargement d'un projet,
    /// compaction) : mieux vaut perdre l'historique que le rendre faux (JRN-2).
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
        self.open = None;
    }

    /// Ouvre une transaction. Sans appel explicite, chaque édition forme son propre geste.
    ///
    /// **Ouvrir ne réécrit rien.** C'est l'écriture qui abandonne l'histoire alternative — au
    /// `commit`, jamais ici. Un geste qui s'ouvre et se referme sans avoir rien produit (un
    /// clic qui sélectionne sans déplacer, un glisser abandonné par Échap) laisse donc le
    /// document et la pile de rétablissement exactement comme il les a trouvés.
    ///
    /// Ce n'était pas le cas tant qu'ouvrir coûtait un cliché du document : il fallait alors
    /// que l'interface n'ouvre qu'au premier pixel réellement parcouru, seuil compris, et
    /// l'ouverture valait édition. L'ouverture ne coûtant plus rien, le seuil n'a plus
    /// d'objet et la règle se simplifie : *sélectionner n'est pas éditer*.
    pub fn begin(&mut self) {
        if self.open.is_none() {
            self.open = Some(Transaction::default());
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Enregistre une édition : dans la transaction ouverte, ou seule dans la sienne.
    ///
    /// **Une édition qui ne change rien n'est pas une édition.** Elle n'entre pas au journal,
    /// donc ne consomme ni niveau d'annulation ni histoire alternative : valider un texte
    /// sans l'avoir modifié, ou un geste revenu à son point de départ, ne laisse rien.
    pub fn record(&mut self, edit: Edit) {
        if edit.is_noop() {
            return;
        }
        match &mut self.open {
            Some(tx) => tx.push(edit),
            None => {
                let mut tx = Transaction::default();
                tx.push(edit);
                self.commit(tx);
            }
        }
    }

    /// Ferme le geste ouvert. Une transaction vide — un clic qui n'a rien bougé — ne laisse
    /// aucune trace : rien à annuler.
    pub fn end(&mut self) {
        if let Some(tx) = self.open.take() {
            if !tx.is_empty() {
                self.commit(tx);
            }
        }
    }

    /// Abandonne la transaction ouverte en défaisant ce qu'elle a déjà écrit.
    /// Rend `false` hors transaction.
    pub fn cancel(&mut self, project: &mut Project) -> bool {
        let Some(mut tx) = self.open.take() else {
            return false;
        };
        if tx.is_empty() {
            return true;
        }
        tx.invert();
        if !tx.apply(project) {
            self.clear();
        }
        true
    }

    fn commit(&mut self, tx: Transaction) {
        self.done.push(tx);
        if self.done.len() > self.max_depth {
            self.done.remove(0);
        }
        self.undone.clear();
    }

    /// Défait le dernier geste. Rend `false` s'il n'y a rien à défaire.
    pub fn undo(&mut self, project: &mut Project) -> bool {
        self.step(project, true)
    }

    /// Refait le dernier geste défait. Rend `false` s'il n'y a rien à refaire.
    pub fn redo(&mut self, project: &mut Project) -> bool {
        self.step(project, false)
    }

    /// Défaire et refaire sont le même mouvement, entre deux piles échangées.
    ///
    /// Une transaction se défait en échangeant l'avant et l'après de chacune de ses
    /// éditions ([`Slot::flip`]) : l'opération est **sa propre inverse**, ce qui est la
    /// raison pour laquelle `undo` et `redo` n'ont pas besoin de code distinct.
    fn step(&mut self, project: &mut Project, backward: bool) -> bool {
        // Pendant un geste ouvert, la pile ne décrit plus le document : ce que le geste a
        // déjà écrit n'y figure pas encore. Défaire ou refaire appliquerait des index
        // calculés sur un autre état — JRN-2 rompu, et le journal vidé pour rien. Le clavier
        // attend donc le relâchement.
        if self.open.is_some() {
            return false;
        }
        let (from, to) = if backward {
            (&mut self.done, &mut self.undone)
        } else {
            (&mut self.undone, &mut self.done)
        };
        let Some(mut tx) = from.pop() else {
            return false;
        };
        tx.invert();
        if !tx.apply(project) {
            // JRN-2 rompu : le document n'est plus celui qu'on croyait. On vide plutôt que de
            // laisser une pile qui ment.
            self.clear();
            return false;
        }
        to.push(tx);
        true
    }
}

#[cfg(test)]
mod tests;
