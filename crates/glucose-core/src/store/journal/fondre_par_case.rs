//! **FONDRE-145 — deux éditions d'une même case se rejoignent par-dessus ce qui ne la touche pas.**
//!
//! Une saisie qui pousse un voisin écrit le texte, le voisin, le texte, le voisin… : jamais deux
//! éditions de la même case côte à côte, et FONDRE-1 n'en fondait aucune. Revenue à son départ,
//! elle laissait un geste qui ne change rien — un `Ctrl+Z` mort.
//!
//! Deux éditions d'une même case peuvent se rejoindre quand rien entre elles ne la touche ni ne
//! décale les rangs : un changement **sur place** d'une autre case, une translation qui ne la
//! déplace pas. Elles commutent — l'ordre entre elles ne change pas le document. Une insertion,
//! un retrait, un tout (l'ordre des tableaux, les zones…) arrêtent tout : les rangs ont pu bouger,
//! la case d'après n'est peut-être plus celle d'avant (JRN-2).
//!
//! **Dans la pile seulement** (`additionner`), deux translations du même ensemble s'additionnent :
//! le fichier le refuse — `(x + a) + b` n'est pas toujours `x + (a + b)`, et il doit rejouer l'écran
//! au bit près (GLISSER-1) —, mais la pile n'est jamais relue ni écrite, et défaire une translation
//! applique déjà son opposé.

use super::super::edit::Edit;
use super::super::{Slot, Transaction};
use super::Suite;
use std::collections::HashMap;

/// Une liste du document que des éditions changent case par case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Liste {
    Image,
    Annotation,
    Folder,
    Panel,
    Storyboard,
    Board,
    Domain,
    Preset,
}

/// Une case : sa liste, son tableau (vide pour une liste du projet), son rang.
type Case = (Liste, String, usize);

impl<T> Slot<T> {
    /// Ni insertion ni retrait : aucun rang de la liste ne bouge.
    fn sur_place(&self) -> bool {
        self.before.is_some() && self.after.is_some()
    }
}

impl Edit {
    /// La case que cette édition change **sur place** — `None` pour une insertion, un retrait,
    /// une translation ou un tout.
    fn case_sur_place(&self) -> Option<Case> {
        use Edit as E;
        let (liste, board, index, sur_place) = match self {
            E::Image { board, slot } => {
                (Liste::Image, board.as_str(), slot.index, slot.sur_place())
            }
            E::Annotation { board, slot } => (
                Liste::Annotation,
                board.as_str(),
                slot.index,
                slot.sur_place(),
            ),
            E::Folder { board, slot } => {
                (Liste::Folder, board.as_str(), slot.index, slot.sur_place())
            }
            E::Panel { board, slot } => {
                (Liste::Panel, board.as_str(), slot.index, slot.sur_place())
            }
            E::Storyboard { board, slot } => (
                Liste::Storyboard,
                board.as_str(),
                slot.index,
                slot.sur_place(),
            ),
            E::Board { slot } => (Liste::Board, "", slot.index, slot.sur_place()),
            E::Domain { slot } => (Liste::Domain, "", slot.index, slot.sur_place()),
            E::Preset { slot } => (Liste::Preset, "", slot.index, slot.sur_place()),
            _ => return None,
        };
        sur_place.then(|| (liste, board.to_string(), index))
    }

    /// Cette translation déplace-t-elle la case (le nœud, ou un bout de la flèche) ?
    fn deplace(&self, (liste, b, rang): &Case) -> bool {
        let Edit::Translation {
            board,
            images,
            annotations,
            folders,
            bouts,
            ..
        } = self
        else {
            return false;
        };
        let Ok(rang) = u32::try_from(*rang) else {
            return false;
        };
        board == b
            && match liste {
                Liste::Image => images.contains(&rang),
                Liste::Annotation => {
                    annotations.contains(&rang) || bouts.iter().any(|t| t.0 == rang)
                }
                Liste::Folder => folders.contains(&rang),
                _ => false,
            }
    }

    /// `self` puis `suite`, deux translations du même ensemble : `self` devient leur somme.
    /// Rend `false`, sans rien changer, si elles ne déplacent pas le même ensemble.
    fn ajouter(&mut self, suite: &Edit) -> bool {
        let (
            Edit::Translation {
                board,
                delta,
                images,
                annotations,
                folders,
                bouts,
            },
            Edit::Translation {
                board: b2,
                delta: d2,
                images: i2,
                annotations: a2,
                folders: f2,
                bouts: t2,
            },
        ) = (self, suite)
        else {
            return false;
        };
        if (&*board, &*images, &*annotations, &*folders, &*bouts) != (b2, i2, a2, f2, t2) {
            return false;
        }
        *delta = (delta.0 + d2.0, delta.1 + d2.1);
        true
    }
}

impl Transaction {
    /// Un passage : chaque édition rejoint la précédente de sa case si rien ne les sépare qui la
    /// touche. Rend `true` s'il a fondu quelque chose.
    pub(super) fn fondre_par_case(&mut self, additionner: bool) -> bool {
        let mut sortie: Vec<Option<Edit>> = Vec::with_capacity(self.edits.len());
        // La dernière édition de chaque case encore joignable, et les translations joignables.
        let mut cases: HashMap<Case, usize> = HashMap::new();
        let mut translations: Vec<usize> = Vec::new();
        let mut fondu = false;
        for edit in self.edits.drain(..) {
            if matches!(edit, Edit::Translation { .. }) {
                cases.retain(|case, _| !edit.deplace(case));
                let suite = if additionner {
                    additionner_a(&mut sortie, &mut translations, edit, &mut fondu)
                } else {
                    Some(edit)
                };
                sortie.extend(suite.map(Some));
                continue;
            }
            let Some(case) = edit.case_sur_place() else {
                cases.clear();
                translations.clear();
                sortie.push(Some(edit));
                continue;
            };
            translations.retain(|&r| sortie[r].as_ref().is_some_and(|t| !t.deplace(&case)));
            let Some(rang) = cases.get(&case).copied() else {
                cases.insert(case, sortie.len());
                sortie.push(Some(edit));
                continue;
            };
            let precedente = sortie[rang].take().expect("une case joignable est pleine");
            match precedente.suivie_de(edit) {
                Suite::Fondue(f) if f.is_noop() => {
                    cases.remove(&case);
                    fondu = true;
                }
                Suite::Fondue(f) => {
                    sortie[rang] = Some(f);
                    fondu = true;
                }
                Suite::Distinctes(p, e) => {
                    sortie[rang] = Some(p);
                    cases.insert(case, sortie.len());
                    sortie.push(Some(e));
                }
            }
        }
        self.edits = sortie.into_iter().flatten().collect();
        fondu
    }
}

/// La pile : `edit`, une translation, s'ajoute à la translation joignable du même ensemble — et
/// les deux disparaissent si leur somme est nulle. Rend ce qui reste à écrire à la suite.
fn additionner_a(
    sortie: &mut [Option<Edit>],
    translations: &mut Vec<usize>,
    edit: Edit,
    fondu: &mut bool,
) -> Option<Edit> {
    for (k, &rang) in translations.iter().enumerate() {
        let Some(premiere) = sortie[rang].as_mut() else {
            continue;
        };
        if premiere.ajouter(&edit) {
            *fondu = true;
            if premiere.is_noop() {
                sortie[rang] = None;
                translations.remove(k);
            }
            return None;
        }
    }
    // Pas encore dans `sortie` : son rang sera le prochain.
    translations.push(sortie.len());
    Some(edit)
}
