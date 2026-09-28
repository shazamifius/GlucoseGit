//! **FONDRE-1 — dans un geste, deux éditions successives d'une même case n'en font qu'une.**
//!
//! # Ce que le journal écrivait
//!
//! Un geste continu écrit dans le document à chaque mouvement de la main : redimensionner une
//! carte pendant cinq secondes, c'est plus d'un millier d'éditions de la **même** case, chacune
//! portant une copie complète du nœud avant et après — texte compris. Toutes entraient dans la
//! pile d'annulation et dans l'histoire du fichier, pour dire ce que deux valeurs suffisent à
//! dire : la carte d'avant le geste, et celle d'après. C'est une part de *« elle enregistre
//! beaucoup de choses inutiles »* (registre de Tauri, 12).
//!
//! # La règle, et pourquoi elle est exacte
//!
//! Deux éditions **adjacentes** de la même liste, à la même case, se fondent quand la seconde
//! part exactement de là où la première arrive — son « avant » est l'« après » de l'autre. La
//! fusion garde l'avant de la première et l'après de la seconde. Ces éditions portent des
//! **valeurs**, pas des pas : la composée est exacte au bit près, quelle que soit la paire —
//! changer puis changer, insérer puis changer (une insertion), changer puis retirer (un
//! retrait), retirer puis insérer à la même place (un changement). Insérer puis retirer ne
//! laisse rien, et disparaît ; ce qui la précédait devient alors voisin de ce qui la suivait,
//! et peut se fondre à son tour.
//!
//! Adjacentes seulement : entre deux éditions d'une même case, une insertion ailleurs dans la
//! liste décalerait les rangs, et la fusion désignerait une autre case (JRN-2). Et jamais une
//! translation : elle porte un **pas**, et additionner des pas en virgule flottante ne redonne
//! pas la position qu'ils ont laissée — le glisser s'en charge autrement (GLISSER-1).

use super::edit::Edit;
use super::{Slot, Transaction, Whole};

impl<T: PartialEq> Slot<T> {
    /// `self` puis `suite`, en une seule édition — si `suite` part de là où `self` arrive, à la
    /// même case. Sinon, les deux, rendues telles quelles.
    fn suivie_de(self, suite: Self) -> Result<Self, (Self, Self)> {
        if self.index == suite.index && self.after == suite.before {
            Ok(Self {
                index: self.index,
                before: self.before,
                after: suite.after,
            })
        } else {
            Err((self, suite))
        }
    }
}

impl<T: PartialEq> Whole<T> {
    /// `self` puis `suite`, en une seule édition — si `suite` part de là où `self` arrive.
    fn suivie_de(self, suite: Self) -> Result<Self, (Self, Self)> {
        if self.after == suite.before {
            Ok(Self {
                before: self.before,
                after: suite.after,
            })
        } else {
            Err((self, suite))
        }
    }
}

/// Ce que deviennent deux éditions voisines : une seule, ou les deux telles quelles.
enum Suite {
    Fondue(Edit),
    Distinctes(Edit, Edit),
}

/// Fond deux éditions d'une liste portée par un tableau, si c'est le même tableau.
macro_rules! sur_un_tableau {
    ($variante:ident, $champ:ident, $b1:ident, $s1:ident, $b2:ident, $s2:ident) => {
        if $b1 != $b2 {
            Suite::Distinctes(
                Edit::$variante {
                    board: $b1,
                    $champ: $s1,
                },
                Edit::$variante {
                    board: $b2,
                    $champ: $s2,
                },
            )
        } else {
            match $s1.suivie_de($s2) {
                Ok($champ) => Suite::Fondue(Edit::$variante { board: $b1, $champ }),
                Err((a, b)) => Suite::Distinctes(
                    Edit::$variante {
                        board: $b1,
                        $champ: a,
                    },
                    Edit::$variante {
                        board: $b2,
                        $champ: b,
                    },
                ),
            }
        }
    };
}

/// Fond deux éditions d'une liste portée par le projet.
macro_rules! sur_le_projet {
    ($variante:ident, $champ:ident, $s1:ident, $s2:ident) => {
        match $s1.suivie_de($s2) {
            Ok($champ) => Suite::Fondue(Edit::$variante { $champ }),
            Err((a, b)) => {
                Suite::Distinctes(Edit::$variante { $champ: a }, Edit::$variante { $champ: b })
            }
        }
    };
}

impl Edit {
    /// `self` puis `suite`, en une seule édition quand la règle de FONDRE-1 le permet.
    fn suivie_de(self, suite: Edit) -> Suite {
        use Edit as E;
        match (self, suite) {
            (E::Image { board: a, slot: s }, E::Image { board: b, slot: t }) => {
                sur_un_tableau!(Image, slot, a, s, b, t)
            }
            (E::Annotation { board: a, slot: s }, E::Annotation { board: b, slot: t }) => {
                sur_un_tableau!(Annotation, slot, a, s, b, t)
            }
            (E::Folder { board: a, slot: s }, E::Folder { board: b, slot: t }) => {
                sur_un_tableau!(Folder, slot, a, s, b, t)
            }
            (E::Panel { board: a, slot: s }, E::Panel { board: b, slot: t }) => {
                sur_un_tableau!(Panel, slot, a, s, b, t)
            }
            (E::Zones { board: a, whole: s }, E::Zones { board: b, whole: t }) => {
                sur_un_tableau!(Zones, whole, a, s, b, t)
            }
            (E::BoardName { board: a, whole: s }, E::BoardName { board: b, whole: t }) => {
                sur_un_tableau!(BoardName, whole, a, s, b, t)
            }
            (E::Board { slot: s }, E::Board { slot: t }) => sur_le_projet!(Board, slot, s, t),
            (E::Domain { slot: s }, E::Domain { slot: t }) => sur_le_projet!(Domain, slot, s, t),
            (E::Preset { slot: s }, E::Preset { slot: t }) => sur_le_projet!(Preset, slot, s, t),
            (E::ProjectName { whole: s }, E::ProjectName { whole: t }) => {
                sur_le_projet!(ProjectName, whole, s, t)
            }
            (E::ActiveBoard { whole: s }, E::ActiveBoard { whole: t }) => {
                sur_le_projet!(ActiveBoard, whole, s, t)
            }
            (E::BoardOrder { whole: s }, E::BoardOrder { whole: t }) => {
                sur_le_projet!(BoardOrder, whole, s, t)
            }
            // Deux listes différentes, ou une translation : rien ne se fond.
            (a, b) => Suite::Distinctes(a, b),
        }
    }
}

impl Transaction {
    /// **Fond les éditions successives d'une même case** (FONDRE-1) : le geste dit la même
    /// chose, en ne gardant que ce qui la dit.
    pub(super) fn fondre(&mut self) {
        let mut fondues: Vec<Edit> = Vec::with_capacity(self.edits.len());
        for edit in self.edits.drain(..) {
            let Some(precedente) = fondues.pop() else {
                fondues.push(edit);
                continue;
            };
            match precedente.suivie_de(edit) {
                // Ce qui ne change rien disparaît, et laisse voisines celles qui l'entouraient.
                Suite::Fondue(fondue) if fondue.is_noop() => {}
                Suite::Fondue(fondue) => fondues.push(fondue),
                Suite::Distinctes(precedente, edit) => {
                    fondues.push(precedente);
                    fondues.push(edit);
                }
            }
        }
        self.edits = fondues;
    }
}

#[cfg(test)]
#[path = "fondre_tests.rs"]
mod tests;
