//! **Le geste partagé** (fiche 116 § 8) — ce qu'un geste devient quand il part aux autres
//! participants d'un canevas, et comment il s'applique chez eux.
//!
//! # Par le nom, le rang n'étant qu'un indice
//!
//! Dans le journal, une édition désigne une case de liste par son rang (`Slot::index`, JRN-2) :
//! juste tant que tout se rejoue dans le même ordre. Le co-working rompt cet ordre — un geste
//! s'applique tout de suite chez celui qui le fait, puis se rejoue derrière ceux que la salle a
//! numérotés avant lui (le « rebase »), ou au retour d'une session hors ligne. Son rang vise alors
//! le mauvais nœud.
//!
//! Or chaque `Slot` porte **l'élément entier** avant et après, et chaque élément son identifiant :
//! le geste connaît déjà le nom de ce qu'il touche. Partager ne demande donc ni nouveau journal ni
//! nouveau format de disque, mais **une seconde façon d'appliquer** : trouver l'élément par son
//! nom (le rang d'origine d'abord, presque toujours juste — puis la liste), insérer au rang
//! d'origine s'il tient. Deux exceptions :
//!
//! * **le déplacement** (`Edit::Translation`) ne garde que des rangs, pour déplacer des centaines
//!   de milliers de nœuds en quelques millisecondes (JRN-4) : il est nommé au moment où il part ;
//! * **l'onglet regardé** (`Edit::ActiveBoard`) est un geste du journal, mais chacun regarde le
//!   sien : il ne part pas.
//!
//! # PARTAGE-1 — le dernier gagne, et le conflit se dit
//!
//! L'avant que le geste garde (pour `Ctrl+Z`) est comparé à ce qui est là : s'il diffère, quelqu'un
//! est passé entre-temps — c'est un **conflit**, rendu à l'appelant (la Time Machine le montrera),
//! et le geste s'applique quand même : le dernier numéro de la salle gagne (Figma). Un élément
//! qu'un autre a retiré est **absent** : sa modification ne le ressuscite pas.

use super::{Bouts, Edit, Slot, Transaction, Whole};
use crate::types::{
    Annotation, Board, BoardImage, CanvasFolder, Domain, Preset, Project, Storyboard,
    StoryboardPanel,
};

/// **Ce qui a un nom** dans une liste du modèle : son identifiant.
pub trait Nomme {
    fn nom(&self) -> &str;
}

macro_rules! nomme_par_son_id {
    ($($t:ty),*) => {
        $(impl Nomme for $t {
            fn nom(&self) -> &str {
                &self.id
            }
        })*
    };
}
nomme_par_son_id!(
    BoardImage,
    CanvasFolder,
    StoryboardPanel,
    Storyboard,
    Board,
    Domain,
    Preset
);

impl Nomme for Annotation {
    fn nom(&self) -> &str {
        self.id()
    }
}

/// **Ce qu'un geste a rencontré** en s'appliquant chez un autre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rencontre {
    /// Quelqu'un avait changé cet élément entre-temps : le geste l'a emporté (le dernier gagne).
    Conflit(String),
    /// Cet élément n'est plus là : un autre l'a retiré, la modification ne le fait pas revenir.
    Absent(String),
}

/// **Une édition telle qu'elle part.**
#[derive(Debug, Clone, PartialEq)]
pub enum EditionPartagee {
    /// Toute édition qui porte déjà ses noms.
    Telle(Edit),
    /// Un déplacement, ses rangs devenus des noms.
    Deplacement(Deplacement),
}

/// **Un déplacement nommé** : le vecteur, et ce qu'il emporte, par le nom.
#[derive(Debug, Clone, PartialEq)]
pub struct Deplacement {
    pub board: String,
    pub delta: (f64, f64),
    pub images: Vec<String>,
    pub annotations: Vec<String>,
    /// Les flèches dont un seul bout suit, et lequel.
    pub bouts: Vec<(String, Bouts)>,
    pub folders: Vec<String>,
}

/// **Un geste tel qu'il part aux autres**, et tel qu'ils l'appliquent.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GestePartage {
    pub editions: Vec<EditionPartagee>,
}

impl Transaction {
    /// **Le geste à partager**, nommé dans l'état du projet qui le suit — celui où ses rangs sont
    /// justes. L'onglet regardé reste à soi.
    pub fn a_partager(&self, projet: &Project) -> GestePartage {
        let editions = self
            .edits
            .iter()
            .filter(|e| !matches!(e, Edit::ActiveBoard { .. }))
            .map(|e| nommer(e, projet))
            .collect();
        GestePartage { editions }
    }
}

fn nommer(edit: &Edit, projet: &Project) -> EditionPartagee {
    let Edit::Translation {
        board,
        delta,
        images,
        annotations,
        folders,
        bouts,
    } = edit
    else {
        return EditionPartagee::Telle(edit.clone());
    };
    let tableau = projet.boards.iter().find(|b| b.id == *board);
    let images = noms_de(tableau.map(|t| t.images.as_slice()), images);
    let annotations = noms_de(tableau.map(|t| t.annotations.as_slice()), annotations);
    let folders = noms_de(tableau.map(|t| t.folders.as_slice()), folders);
    let bouts = bouts
        .iter()
        .filter_map(|&(i, q)| {
            let a = tableau?.annotations.get(i as usize)?;
            Some((a.nom().to_string(), q))
        })
        .collect();
    EditionPartagee::Deplacement(Deplacement {
        board: board.clone(),
        delta: *delta,
        images,
        annotations,
        bouts,
        folders,
    })
}

fn noms_de<T: Nomme>(liste: Option<&[T]>, rangs: &[u32]) -> Vec<String> {
    rangs
        .iter()
        .filter_map(|&i| liste?.get(i as usize).map(|e| e.nom().to_string()))
        .collect()
}

impl GestePartage {
    /// **Applique le geste par le nom** de ce qu'il touche ; rend ce qu'il a rencontré.
    pub fn appliquer(&self, projet: &mut Project) -> Vec<Rencontre> {
        let mut rencontres = Vec::new();
        for edition in &self.editions {
            match edition {
                EditionPartagee::Telle(e) => appliquer_par_nom(e, projet, &mut rencontres),
                EditionPartagee::Deplacement(d) => {
                    if let Some(t) = projet.boards.iter_mut().find(|b| b.id == d.board) {
                        d.appliquer(t);
                    }
                }
            }
        }
        rencontres
    }

    /// **L'inverse** : défaire ce geste, par le nom — chez celui qui l'a fait, avant de rejouer
    /// ceux que la salle a numérotés avant lui.
    pub fn inverse(&self) -> GestePartage {
        let editions = self
            .editions
            .iter()
            .rev()
            .map(|e| match e {
                EditionPartagee::Telle(e) => {
                    let mut e = e.clone();
                    e.flip();
                    EditionPartagee::Telle(e)
                }
                // L'inverse d'un déplacement est le déplacement opposé (JRN-4).
                EditionPartagee::Deplacement(d) => EditionPartagee::Deplacement(Deplacement {
                    delta: (-d.delta.0, -d.delta.1),
                    ..d.clone()
                }),
            })
            .collect();
        GestePartage { editions }
    }
}

/// Le rang d'un élément de ce nom : celui d'origine s'il le porte encore — presque toujours —,
/// sinon celui où il est.
fn trouver<T: Nomme>(liste: &[T], nom: &str, indice: usize) -> Option<usize> {
    if liste.get(indice).is_some_and(|e| e.nom() == nom) {
        return Some(indice);
    }
    liste.iter().position(|e| e.nom() == nom)
}

impl<T: Nomme + Clone + PartialEq> Slot<T> {
    /// **Applique cette case par le nom** (PARTAGE-1).
    fn appliquer_par_nom(&self, liste: &mut Vec<T>) -> Option<Rencontre> {
        match (&self.before, &self.after) {
            (None, Some(v)) => match trouver(liste, v.nom(), self.index) {
                // Déjà là : un autre l'a posé sous le même nom ; le dernier gagne.
                Some(i) => {
                    let conflit = liste[i] != **v;
                    liste[i] = (**v).clone();
                    conflit.then(|| Rencontre::Conflit(v.nom().to_string()))
                }
                None => {
                    liste.insert(self.index.min(liste.len()), (**v).clone());
                    None
                }
            },
            (Some(avant), apres) => {
                let Some(i) = trouver(liste, avant.nom(), self.index) else {
                    return Some(Rencontre::Absent(avant.nom().to_string()));
                };
                let conflit = liste[i] != **avant;
                match apres {
                    Some(v) => liste[i] = (**v).clone(),
                    None => {
                        liste.remove(i);
                    }
                }
                conflit.then(|| Rencontre::Conflit(avant.nom().to_string()))
            }
            (None, None) => None,
        }
    }
}

/// Un bloc remplacé : l'avant comparé à ce qui est là.
fn remplacer<T: Clone + PartialEq>(
    cible: &mut T,
    whole: &Whole<T>,
    nom: &str,
) -> Option<Rencontre> {
    let conflit = *cible != whole.before;
    *cible = whole.after.clone();
    conflit.then(|| Rencontre::Conflit(nom.to_string()))
}

fn appliquer_par_nom(edit: &Edit, projet: &mut Project, rencontres: &mut Vec<Rencontre>) {
    let r = match edit {
        Edit::Board { slot } => slot.appliquer_par_nom(&mut projet.boards),
        Edit::Domain { slot } => slot.appliquer_par_nom(&mut projet.domains),
        Edit::Preset { slot } => slot.appliquer_par_nom(&mut projet.presets),
        Edit::ProjectName { whole } => remplacer(&mut projet.name, whole, "le nom du document"),
        Edit::BoardOrder { whole } => {
            ordonner(&mut projet.boards, &whole.after);
            None
        }
        // L'onglet regardé ne part pas ; un déplacement part nommé.
        Edit::ActiveBoard { .. } | Edit::Translation { .. } => None,
        _ => {
            let Some(tableau) = edit
                .board_id()
                .and_then(|id| projet.boards.iter_mut().find(|b| b.id == id))
            else {
                let id = edit.board_id().unwrap_or_default().to_string();
                rencontres.push(Rencontre::Absent(id));
                return;
            };
            appliquer_au_tableau(edit, tableau)
        }
    };
    rencontres.extend(r);
}

fn appliquer_au_tableau(edit: &Edit, t: &mut Board) -> Option<Rencontre> {
    match edit {
        Edit::Image { slot, .. } => slot.appliquer_par_nom(&mut t.images),
        Edit::Annotation { slot, .. } => slot.appliquer_par_nom(&mut t.annotations),
        Edit::Folder { slot, .. } => slot.appliquer_par_nom(&mut t.folders),
        Edit::Panel { slot, .. } => slot.appliquer_par_nom(&mut t.panels),
        Edit::Storyboard { slot, .. } => slot.appliquer_par_nom(&mut t.storyboards),
        Edit::Zones { whole, .. } => remplacer(&mut t.zones, whole, &t.id.clone()),
        Edit::BoardName { whole, .. } => remplacer(&mut t.name, whole, &t.id.clone()),
        _ => None,
    }
}

/// L'ordre des tableaux : ceux que l'ordre nomme d'abord, dans son ordre ; ceux qu'un autre a
/// ajoutés entre-temps ensuite, dans le leur.
fn ordonner(tableaux: &mut [Board], ordre: &[String]) {
    let rang = |b: &Board| {
        ordre
            .iter()
            .position(|id| *id == b.id)
            .unwrap_or(usize::MAX)
    };
    tableaux.sort_by_key(rang);
}

impl Deplacement {
    /// Chaque élément retrouvé par son nom ; un absent est passé.
    fn appliquer(&self, t: &mut Board) {
        let (dx, dy) = self.delta;
        for img in t.images.iter_mut().filter(|i| self.images.contains(&i.id)) {
            img.x += dx;
            img.y += dy;
        }
        for a in &mut t.annotations {
            if self.annotations.iter().any(|n| n == a.id()) {
                a.translate(dx, dy);
            } else if let Some((_, q)) = self.bouts.iter().find(|(n, _)| n == a.id()) {
                if let Annotation::Arrow { x, y, x2, y2, .. } = a {
                    if q.origine {
                        *x += dx;
                        *y += dy;
                    }
                    if q.cible {
                        *x2 += dx;
                        *y2 += dy;
                    }
                }
            }
        }
        for f in t
            .folders
            .iter_mut()
            .filter(|f| self.folders.contains(&f.id))
        {
            f.x += dx;
            f.y += dy;
        }
    }
}

#[cfg(test)]
#[path = "partage_tests.rs"]
mod tests;
