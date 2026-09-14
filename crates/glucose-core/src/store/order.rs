//! L'ordre d'empilement : ce qui passe devant, ce qui passe derrière.
//!
//! # Ce que l'ordre est, ici
//!
//! Glucose n'a pas de profondeur explicite : un nœud est devant un autre parce qu'il vient
//! **après lui dans sa liste**, et le dessin suit cet ordre. Monter une image au premier plan,
//! c'est donc la déplacer à la fin de `board.images` — ni plus, ni moins.
//!
//! Conséquence à dire plutôt qu'à cacher : l'ordre vaut **à l'intérieur d'une couche**. Les
//! membranes se dessinent sous les images, qui se dessinent sous les annotations ; aucune
//! image ne passera jamais devant une carte de texte, quel que soit le geste. Une profondeur
//! commune aux trois demanderait un rang porté par chaque nœud et un tri à chaque image —
//! ce que la fondation en arène rendra possible, et qui n'a pas sa place dans un `Vec`.
//!
//! # ORDER-1 — l'ordre relatif de la sélection est conservé
//!
//! Monter trois nœuds au premier plan les met devant tous les autres **sans les mélanger
//! entre eux** : celui qui était devant ses deux voisins l'est encore. C'est ce qu'on attend
//! d'un « mettre au premier plan », et c'est ce que le test `test_order_1_*` tient.
//!
//! # Le coût, et pourquoi le journal n'a pas eu besoin d'une variante de plus
//!
//! Déplacer un élément dans une liste, c'est le retirer puis le réinsérer. Le journal sait
//! déjà dire les deux ([`Slot::removed`], [`Slot::inserted`]), et défaire une transaction
//! rejoue ses éditions **à l'envers** : l'insertion s'annule en retrait, le retrait en
//! insertion, et la liste retrouve exactement sa forme. Une entrée pèse donc deux fois
//! l'élément déplacé, jamais la taille du document (JRN-1).

use super::journal::{Edit, Slot};
use super::Store;

/// Où porter la sélection dans sa pile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackMove {
    /// Devant tout le reste de sa couche.
    Front,
    /// Derrière tout le reste de sa couche.
    Back,
}

impl Store {
    /// Porte la sélection au premier ou au dernier plan de sa couche.
    ///
    /// Rend le nombre de nœuds déplacés — `0` quand rien n'est sélectionné, ou quand tout
    /// l'est déjà : un geste sans effet ne doit pas encombrer l'historique.
    pub fn move_selection_in_stack(&mut self, board_id: &str, mv: StackMove) -> usize {
        let images: Vec<String> = self.selected_image_ids.clone();
        let annotations: Vec<String> = self.selected_annotation_ids.clone();
        if images.is_empty() && annotations.is_empty() {
            return 0;
        }

        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return 0;
        };
        let bid = b.id.clone();
        let mut edits = Vec::new();

        let img_idx = selected_indices(&b.images, |i| &i.id, &images);
        for slot in restack(&mut b.images, &img_idx, mv) {
            edits.push(Edit::Image {
                board: bid.clone(),
                slot,
            });
        }

        let ann_idx = selected_indices(&b.annotations, |a| a.id(), &annotations);
        for slot in restack(&mut b.annotations, &ann_idx, mv) {
            edits.push(Edit::Annotation {
                board: bid.clone(),
                slot,
            });
        }

        // Deux éditions par nœud déplacé : le retrait et l'insertion.
        let moved = edits.len() / 2;
        if moved > 0 {
            self.record_as_one_gesture(edits);
        }
        moved
    }
}

/// Les rangs des éléments de `list` dont l'identité est dans `wanted`, croissants.
fn selected_indices<T>(list: &[T], id_of: impl Fn(&T) -> &str, wanted: &[String]) -> Vec<usize> {
    list.iter()
        .enumerate()
        .filter(|(_, item)| wanted.iter().any(|w| w == id_of(item)))
        .map(|(i, _)| i)
        .collect()
}

/// Déplace les rangs `idx` d'une liste vers l'une de ses extrémités, et dit ce qu'il a fait.
///
/// Le calcul des rangs successifs n'a pas besoin de rechercher quoi que ce soit. Vers
/// l'avant, on retire du plus petit rang au plus grand : après `j` retraits, tous portaient
/// un rang inférieur, donc le `j`-ième élément est retombé à `idx[j] - j`. Vers l'arrière, un
/// retrait et une insertion se compensent devant les rangs suivants, qui ne bougent pas du
/// tout. Dans les deux cas les éléments sont repris dans l'ordre, ce qui conserve leur ordre
/// relatif (ORDER-1).
fn restack<T: Clone>(list: &mut Vec<T>, idx: &[usize], mv: StackMove) -> Vec<Slot<T>> {
    // Une sélection déjà rassemblée contre le bord visé n'a nulle part où aller.
    if idx.is_empty() || already_at_edge(idx, list.len(), mv) {
        return Vec::new();
    }
    let mut slots = Vec::with_capacity(idx.len() * 2);
    for (j, &i) in idx.iter().enumerate() {
        let from = match mv {
            StackMove::Front => i - j,
            StackMove::Back => i,
        };
        let value = list.remove(from);
        slots.push(Slot::removed(from, value.clone()));

        let to = match mv {
            StackMove::Front => list.len(),
            StackMove::Back => j,
        };
        list.insert(to, value.clone());
        slots.push(Slot::inserted(to, value));
    }
    slots
}

/// La sélection occupe-t-elle déjà, d'un bloc, l'extrémité visée ?
fn already_at_edge(idx: &[usize], len: usize, mv: StackMove) -> bool {
    let first = idx[0];
    let contigu = idx.iter().enumerate().all(|(k, &i)| i == first + k);
    contigu
        && match mv {
            StackMove::Front => first + idx.len() == len,
            StackMove::Back => first == 0,
        }
}

#[cfg(test)]
mod tests;
