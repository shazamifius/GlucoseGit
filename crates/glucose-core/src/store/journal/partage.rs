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

/// **Ce qu'un geste partagé a fait chez celui qui le reçoit.**
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Application {
    /// **La transaction réalisée**, avec les rangs de l'instant : le journal la reçoit comme les
    /// autres (JRN-2 reste vrai, l'histoire du disque se rejoue dans l'ordre), et la défaire rend
    /// exactement l'état d'avant — la valeur d'un autre comprise, en cas de conflit.
    pub realisee: Transaction,
    pub rencontres: Vec<Rencontre>,
}

impl GestePartage {
    /// **Applique le geste par le nom** de ce qu'il touche.
    pub fn appliquer(&self, projet: &mut Project) -> Application {
        let mut a = Application::default();
        for edition in &self.editions {
            let realisee = match edition {
                EditionPartagee::Telle(e) => appliquer_par_nom(e, projet, &mut a.rencontres),
                EditionPartagee::Deplacement(d) => {
                    let realise = projet
                        .boards
                        .iter_mut()
                        .find(|b| b.id == d.board)
                        .map(|t| d.realiser(t));
                    // Une seule façon de déplacer : la loi du journal, aux rangs de l'instant.
                    realise.filter(|e| e.apply(projet))
                }
            };
            a.realisee.edits.extend(realisee);
        }
        a
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

/// Une case réalisée, et ce qu'elle a rencontré.
type Realisee<T> = (Option<Slot<T>>, Option<Rencontre>);

impl<T: Nomme + Clone + PartialEq> Slot<T> {
    /// **Applique cette case par le nom** (PARTAGE-1) ; rend la case réalisée, à son vrai rang.
    fn appliquer_par_nom(&self, liste: &mut Vec<T>) -> Realisee<T> {
        match (&self.before, &self.after) {
            (None, Some(v)) => match trouver(liste, v.nom(), self.index) {
                // Déjà là : un autre l'a posé sous le même nom ; le dernier gagne.
                Some(i) => {
                    let ancien = std::mem::replace(&mut liste[i], (**v).clone());
                    let conflit = ancien != **v;
                    let r = conflit.then(|| Rencontre::Conflit(v.nom().to_string()));
                    (conflit.then(|| Slot::changed(i, ancien, (**v).clone())), r)
                }
                None => {
                    let i = self.index.min(liste.len());
                    liste.insert(i, (**v).clone());
                    (Some(Slot::inserted(i, (**v).clone())), None)
                }
            },
            (Some(avant), apres) => {
                let Some(i) = trouver(liste, avant.nom(), self.index) else {
                    return (None, Some(Rencontre::Absent(avant.nom().to_string())));
                };
                let conflit = liste[i] != **avant;
                let r = conflit.then(|| Rencontre::Conflit(avant.nom().to_string()));
                let realisee = match apres {
                    Some(v) => Slot::changed(
                        i,
                        std::mem::replace(&mut liste[i], (**v).clone()),
                        (**v).clone(),
                    ),
                    None => Slot::removed(i, liste.remove(i)),
                };
                (Some(realisee), r)
            }
            (None, None) => (None, None),
        }
    }
}

/// Un bloc remplacé : l'avant comparé à ce qui est là ; le bloc réalisé part de ce qui était là.
fn remplacer<T: Clone + PartialEq>(
    cible: &mut T,
    whole: &Whole<T>,
    nom: &str,
    rencontres: &mut Vec<Rencontre>,
) -> Whole<T> {
    if *cible != whole.before {
        rencontres.push(Rencontre::Conflit(nom.to_string()));
    }
    Whole::new(
        std::mem::replace(cible, whole.after.clone()),
        whole.after.clone(),
    )
}

/// Garde ce qu'une case a rencontré ; rend la case réalisée.
fn garder<T>(
    rencontres: &mut Vec<Rencontre>,
    (realisee, rencontre): Realisee<T>,
) -> Option<Slot<T>> {
    rencontres.extend(rencontre);
    realisee
}

/// **Une édition appliquée par le nom** ; rend l'édition réalisée, s'il en est une.
fn appliquer_par_nom(
    edit: &Edit,
    projet: &mut Project,
    rencontres: &mut Vec<Rencontre>,
) -> Option<Edit> {
    match edit {
        Edit::Board { slot } => garder(rencontres, slot.appliquer_par_nom(&mut projet.boards))
            .map(|slot| Edit::Board { slot }),
        Edit::Domain { slot } => garder(rencontres, slot.appliquer_par_nom(&mut projet.domains))
            .map(|slot| Edit::Domain { slot }),
        Edit::Preset { slot } => garder(rencontres, slot.appliquer_par_nom(&mut projet.presets))
            .map(|slot| Edit::Preset { slot }),
        Edit::ProjectName { whole } => Some(Edit::ProjectName {
            whole: remplacer(&mut projet.name, whole, "le nom du document", rencontres),
        }),
        Edit::BoardOrder { whole } => {
            let ids = |p: &Project| p.boards.iter().map(|b| b.id.clone()).collect::<Vec<_>>();
            let avant = ids(projet);
            ordonner(&mut projet.boards, &whole.after);
            Some(Edit::BoardOrder {
                whole: Whole::new(avant, ids(projet)),
            })
        }
        // L'onglet regardé ne part pas ; un déplacement part nommé.
        Edit::ActiveBoard { .. } | Edit::Translation { .. } => None,
        _ => {
            let id = edit.board_id().unwrap_or_default().to_string();
            let Some(tableau) = projet.boards.iter_mut().find(|b| b.id == id) else {
                rencontres.push(Rencontre::Absent(id));
                return None;
            };
            appliquer_au_tableau(edit, tableau, rencontres)
        }
    }
}

fn appliquer_au_tableau(
    edit: &Edit,
    t: &mut Board,
    rencontres: &mut Vec<Rencontre>,
) -> Option<Edit> {
    let board = t.id.clone();
    match edit {
        Edit::Image { slot, .. } => garder(rencontres, slot.appliquer_par_nom(&mut t.images))
            .map(|slot| Edit::Image { board, slot }),
        Edit::Annotation { slot, .. } => {
            garder(rencontres, slot.appliquer_par_nom(&mut t.annotations))
                .map(|slot| Edit::Annotation { board, slot })
        }
        Edit::Folder { slot, .. } => garder(rencontres, slot.appliquer_par_nom(&mut t.folders))
            .map(|slot| Edit::Folder { board, slot }),
        Edit::Panel { slot, .. } => garder(rencontres, slot.appliquer_par_nom(&mut t.panels))
            .map(|slot| Edit::Panel { board, slot }),
        Edit::Storyboard { slot, .. } => {
            garder(rencontres, slot.appliquer_par_nom(&mut t.storyboards))
                .map(|slot| Edit::Storyboard { board, slot })
        }
        Edit::Zones { whole, .. } => {
            let whole = remplacer(&mut t.zones, whole, &board, rencontres);
            Some(Edit::Zones { board, whole })
        }
        Edit::BoardName { whole, .. } => {
            let whole = remplacer(&mut t.name, whole, &board, rencontres);
            Some(Edit::BoardName { board, whole })
        }
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
    /// Le déplacement réalisé : chaque élément retrouvé par son nom, à son rang de l'instant ; un
    /// absent est passé.
    fn realiser(&self, t: &Board) -> Edit {
        let rangs = |noms: &[String], ids: &mut dyn Iterator<Item = &str>| -> Vec<u32> {
            ids.enumerate()
                .filter(|(_, id)| noms.iter().any(|n| n == id))
                .map(|(i, _)| i as u32)
                .collect()
        };
        let images = rangs(&self.images, &mut t.images.iter().map(|i| i.id.as_str()));
        let annotations = rangs(
            &self.annotations,
            &mut t.annotations.iter().map(Annotation::id),
        );
        let folders = rangs(&self.folders, &mut t.folders.iter().map(|f| f.id.as_str()));
        let bouts: Vec<(u32, Bouts)> = t
            .annotations
            .iter()
            .enumerate()
            .filter_map(|(i, a)| {
                let (_, q) = self.bouts.iter().find(|(n, _)| n == a.id())?;
                Some((i as u32, *q))
            })
            .collect();
        Edit::Translation {
            board: t.id.clone(),
            delta: self.delta,
            images,
            annotations,
            folders,
            bouts,
        }
    }
}

#[cfg(test)]
#[path = "partage_tests.rs"]
mod tests;
