//! Ce qu'une **édition** est, et comment elle s'applique (JRN-1, JRN-2, JRN-4).
//!
//! Une entrée de journal nomme la liste qu'elle modifie et porte de quoi la remettre dans
//! l'un ou l'autre état. Le tableau visé est retrouvé au moment d'appliquer, ce qui évite de
//! garder une référence et laisse l'entrée sérialisable.
//!
//! # Deux façons de retenir une modification
//!
//! La plupart des éditions gardent l'**avant** et l'**après** : c'est la seule chose à faire
//! quand rien ne relie les deux. Mais un déplacement, lui, a une forme fermée — et son
//! inverse aussi. [`Edit::Translation`] garde alors la transformation plutôt que ses effets,
//! et c'est ce qui fait passer le déplacement d'une grande sélection de deux secondes à
//! quelques millisecondes.

use super::{Slot, Whole};
use crate::types::{
    Annotation, Board, BoardImage, BoardZone, CanvasFolder, Domain, Preset, Project,
    StoryboardPanel,
};

/// Quelles extrémités d'une flèche ont suivi un déplacement (JRN-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bouts {
    pub origine: bool,
    pub cible: bool,
}

impl Bouts {
    /// Vrai si au moins une extrémité suit — sinon l'entrée n'a pas lieu d'être.
    pub fn suit(self) -> bool {
        self.origine || self.cible
    }
}

/// Une édition élémentaire, rattachée à la liste qu'elle modifie.
///
/// Chaque variante nomme une liste du modèle. Le tableau visé est retrouvé au moment
/// d'appliquer, ce qui évite de stocker une référence et garde l'entrée sérialisable.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    // ── Listes portées par un board ──────────────────────────────────────
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
    /// Les zones sont posées en bloc par un preset : la liste entière est la modification.
    Zones {
        board: String,
        whole: Whole<Vec<BoardZone>>,
    },
    BoardName {
        board: String,
        whole: Whole<String>,
    },

    /// Un **déplacement** de sélection : un vecteur, et ce qu'il a emporté (JRN-4).
    ///
    /// # Pourquoi une variante à part
    ///
    /// Déplacer une sélection passait par une entrée `Annotation` par élément, chacune
    /// portant une copie complète de l'avant **et** de l'après — texte compris. Sur une
    /// sélection de sept cent cinquante mille nœuds, cela faisait un million et demi de
    /// clones, plus autant d'identifiants de tableau recopiés : **2 243 ms**, et deux fois le
    /// document gardé en mémoire pour retenir deux nombres.
    ///
    /// Or une translation est **inversible par elle-même**. L'annuler, c'est appliquer son
    /// opposé : il n'y a rien à mémoriser d'autre que le vecteur et la liste de ce qui l'a
    /// subi. C'est l'exigence 5 de la charte appliquée au journal — quand une transformation
    /// a une forme fermée, on garde la transformation, pas ses effets.
    ///
    /// Les rangs sont ceux du moment du geste, comme ceux de [`Slot`], et sont vérifiés à
    /// l'application (JRN-2).
    Translation {
        board: String,
        /// Le vecteur appliqué, en unités monde.
        delta: (f64, f64),
        /// Les images déplacées en entier.
        images: Vec<u32>,
        /// Les annotations déplacées en entier.
        annotations: Vec<u32>,
        /// Les dossiers déplacés en entier.
        folders: Vec<u32>,
        /// Les flèches dont **une seule extrémité** a suivi, et laquelle.
        ///
        /// Une flèche que la sélection ne contient pas peut voir son origine ou sa cible
        /// traînée, parce que le nœud auquel elle s'accroche, lui, bouge.
        bouts: Vec<(u32, Bouts)>,
    },

    // ── Listes portées par le projet ─────────────────────────────────────
    /// Supprimer un board emporte tout son contenu : l'entrée est lourde, et c'est conforme
    /// à JRN-1 — la modification *est* de cette taille.
    Board {
        slot: Slot<Board>,
    },
    Domain {
        slot: Slot<Domain>,
    },
    Preset {
        slot: Slot<Preset>,
    },
    ProjectName {
        whole: Whole<String>,
    },
    ActiveBoard {
        whole: Whole<String>,
    },
}

impl Edit {
    /// Identifiant du board porteur de la liste éditée, s'il y en a un.
    ///
    /// Les éditions qui portent sur le projet lui-même — ses boards, ses domaines, ses
    /// presets, son nom — n'en ont pas.
    pub(super) fn board_id(&self) -> Option<&str> {
        match self {
            Self::Image { board, .. }
            | Self::Annotation { board, .. }
            | Self::Folder { board, .. }
            | Self::Panel { board, .. }
            | Self::Zones { board, .. }
            | Self::BoardName { board, .. }
            | Self::Translation { board, .. } => Some(board),
            Self::Board { .. }
            | Self::Domain { .. }
            | Self::Preset { .. }
            | Self::ProjectName { .. }
            | Self::ActiveBoard { .. } => None,
        }
    }

    pub(super) fn flip(&mut self) {
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
            // Toute la raison d'être de cette variante : l'inverse d'un déplacement est le
            // déplacement opposé, et il n'y a rien d'autre à retourner.
            Self::Translation { delta, .. } => *delta = (-delta.0, -delta.1),
        }
    }

    /// Écrit l'état `after` dans le projet. Rend `false` si la cible est introuvable (JRN-2).
    pub(super) fn apply(&self, project: &mut Project) -> bool {
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
            Self::Translation {
                delta,
                images,
                annotations,
                folders,
                bouts,
                ..
            } => appliquer_translation(board, *delta, images, annotations, folders, bouts),
            // Traitées plus haut : `board_id()` les a déjà écartées en rendant `None`.
            _ => false,
        }
    }

    /// Vrai si l'édition ne change rien : même contenu avant et après.
    ///
    /// Un site qui clone, laisse une fermeture travailler et compare n'a pas à savoir si
    /// elle a travaillé pour rien — c'est le journal qui tranche, une fois pour tous.
    pub(super) fn is_noop(&self) -> bool {
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
            // Un vecteur nul ne déplace rien, et une translation sans cible non plus.
            Self::Translation {
                delta,
                images,
                annotations,
                folders,
                bouts,
                ..
            } => {
                *delta == (0.0, 0.0)
                    || (images.is_empty()
                        && annotations.is_empty()
                        && folders.is_empty()
                        && bouts.is_empty())
            }
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
            // Le poids d'un déplacement est celui de sa **liste**, pas celui de ce qu'elle
            // désigne : c'est un rang par élément, là où deux copies complètes coûtaient la
            // taille du document. C'est ce que JRN-1 cherche à borner.
            Self::Translation {
                images,
                annotations,
                folders,
                bouts,
                ..
            } => {
                (images.len() + annotations.len() + folders.len()) * std::mem::size_of::<u32>()
                    + bouts.len() * std::mem::size_of::<(u32, Bouts)>()
            }
        }
    }
}

/// Applique un déplacement aux rangs qu'il désigne (JRN-4).
///
/// Chaque rang est vérifié avant d'être suivi : un rang hors bornes signifie qu'une édition a
/// contourné le journal et que la liste a changé sous lui (JRN-2). La fonction rend alors
/// `false` **sans rien avoir écrit** — reprendre à moitié laisserait le document dans un état
/// que ni l'avant ni l'après ne décrit.
fn appliquer_translation(
    board: &mut Board,
    (dx, dy): (f64, f64),
    images: &[u32],
    annotations: &[u32],
    folders: &[u32],
    bouts: &[(u32, Bouts)],
) -> bool {
    let dans = |rangs: &[u32], taille: usize| rangs.iter().all(|&i| (i as usize) < taille);
    if !dans(images, board.images.len())
        || !dans(annotations, board.annotations.len())
        || !dans(folders, board.folders.len())
        || !bouts
            .iter()
            .all(|(i, _)| (*i as usize) < board.annotations.len())
    {
        return false;
    }

    for &i in images {
        let img = &mut board.images[i as usize];
        img.x += dx;
        img.y += dy;
    }
    for &i in annotations {
        board.annotations[i as usize].translate(dx, dy);
    }
    for &i in folders {
        let f = &mut board.folders[i as usize];
        f.x += dx;
        f.y += dy;
    }
    for &(i, quels) in bouts {
        if let Annotation::Arrow { x, y, x2, y2, .. } = &mut board.annotations[i as usize] {
            if quels.origine {
                *x += dx;
                *y += dy;
            }
            if quels.cible {
                *x2 += dx;
                *y2 += dy;
            }
        }
    }
    true
}
