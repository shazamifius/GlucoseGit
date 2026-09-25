//! **Ce que l'écran montre pendant un geste** (GESTE-1).
//!
//! # Le défaut
//!
//! L'index spatial décrit le document **publié** : sa version n'avance qu'à la fin d'un geste
//! (`Store::end_live_edit`), et c'est voulu — un glisser qui réindexerait le tableau à chaque
//! mouvement paierait sa taille à chaque image. Mais pendant le geste, le document a déjà
//! changé, et l'écran le montrait **tel que l'index le croyait** :
//!
//! * une flèche qu'on tire n'avait **aucun** rang dans l'index : invisible pendant tout le
//!   glisser, elle n'apparaissait qu'au relâchement ;
//! * un nœud qu'on glisse ne se dessinait que tant que son **ancienne** place restait à
//!   l'écran ;
//! * et une flèche ajoutée en fin de liste décalait d'un cran les rangs des dossiers : le
//!   rang périmé du premier dossier désignait la flèche neuve.
//!
//! # La règle
//!
//! Ce qui se voit pendant un geste est **l'union** de ce que l'index connaît, relu dans la
//! numérotation du tableau tel qu'il est, et de ce que le geste a touché. Le journal sait
//! exactement ce qu'il a écrit ; il n'y a rien à deviner.
//!
//! # Le coût suit le geste, jamais le document
//!
//! Un glisser de `k` nœuds sur `m` mouvements écrit `m` translations de `k` indices. Relire le
//! geste entier à chaque image coûterait `m · k` ; le suivi ne lit que les écritures
//! **nouvelles** depuis l'image précédente, et reconnaît le geste à son numéro.
//!
//! # Quand les rangs d'avant ne se relisent plus
//!
//! Poser en fin de liste ou changer en place laisse chaque rang de l'index désigner le même
//! nœud, à un décalage près qui se calcule. Retirer un nœud, en insérer un au milieu, ou
//! réécrire le tableau décale ce que l'index connaît d'une façon qu'aucun décalage ne dit :
//! le suivi le signale, et c'est à l'index de se remettre d'accord.

use super::SpatialHash;
use crate::store::journal::{Edit, Slot};
use crate::types::Board;

/// Les trois listes d'un tableau, dans l'ordre où l'index les numérote.
const IMAGES: usize = 0;
const ANNOTATIONS: usize = 1;
const DOSSIERS: usize = 2;

/// Ce qu'un geste en cours a touché, lu au fil des images (GESTE-1).
#[derive(Debug, Default)]
pub struct SuiviDuGeste {
    /// Le numéro du geste suivi — zéro quand il n'y en a pas.
    numero: u64,
    /// Les longueurs du tableau que l'index connaissait quand le suivi a commencé. Si l'index
    /// se remet d'accord en cours de geste, le suivi repart de zéro.
    avant: [usize; 3],
    /// Combien d'écritures du geste ont déjà été lues.
    lues: usize,
    /// Combien de nœuds le geste a posés en fin de chaque liste.
    poses: [usize; 3],
    /// Ce que le geste a posé ou changé, par liste : des indices triés, sans doublon.
    touches: [Vec<u32>; 3],
    /// Le geste a fait ce que les rangs de l'index ne savent plus dire.
    illisible: bool,
}

impl SuiviDuGeste {
    /// **Complète les rangs que l'index vient de rendre** par ce que le geste en cours a
    /// touché, le tout dans la numérotation du tableau tel qu'il est maintenant.
    ///
    /// `geste` est ce que rend `Journal::en_cours`. Rend `false` quand le geste a fait ce que
    /// les rangs de l'index ne savent plus relire : l'appelant remet alors l'index d'accord
    /// avec le tableau et recommence sa requête — ce que la fin du geste aurait fait.
    pub fn completer(
        &mut self,
        geste: Option<(u64, &[Edit])>,
        board: &Board,
        index: &SpatialHash,
        rangs: &mut Vec<u32>,
    ) -> bool {
        let Some((numero, ecrits)) = geste else {
            *self = Self::default();
            return true;
        };
        if numero != self.numero || index.longueurs() != self.avant || ecrits.len() < self.lues {
            *self = Self {
                numero,
                avant: index.longueurs(),
                ..Self::default()
            };
        }
        for edit in &ecrits[self.lues..] {
            self.lire(edit, board);
        }
        self.lues = ecrits.len();
        if self.illisible {
            return false;
        }
        self.au_present(rangs, board);
        true
    }

    /// **L'index vient de se remettre d'accord avec le tableau, en plein geste** : tout ce que
    /// le geste a écrit jusqu'ici, il le connaît. Le suivi repart de là, et ne relira que la
    /// suite — sans quoi un retrait, relu à chaque image, rendrait le geste illisible à
    /// chaque image et forcerait l'index à tout reparcourir.
    pub fn absorbe(&mut self, geste: Option<(u64, &[Edit])>, index: &SpatialHash) {
        let (numero, lues) = geste.map_or((0, 0), |(n, ecrits)| (n, ecrits.len()));
        *self = Self {
            numero,
            avant: index.longueurs(),
            lues,
            ..Self::default()
        };
    }

    /// Une écriture du geste, portée au compte de ce qu'il a touché.
    fn lire(&mut self, edit: &Edit, board: &Board) {
        match edit {
            Edit::Image { board: b, slot } if *b == board.id => self.case(IMAGES, slot),
            Edit::Annotation { board: b, slot } if *b == board.id => self.case(ANNOTATIONS, slot),
            Edit::Folder { board: b, slot } if *b == board.id => self.case(DOSSIERS, slot),
            Edit::Translation {
                board: b,
                images,
                annotations,
                folders,
                bouts,
                ..
            } if *b == board.id => {
                self.ajouter(IMAGES, images.iter().copied());
                self.ajouter(ANNOTATIONS, annotations.iter().copied());
                self.ajouter(ANNOTATIONS, bouts.iter().map(|(i, _)| *i));
                self.ajouter(DOSSIERS, folders.iter().copied());
            }
            // Le tableau lui-même réécrit ou retiré : plus aucun rang ne tient.
            Edit::Board { slot } => {
                let est_le_sien =
                    |b: &Option<Box<Board>>| b.as_deref().is_some_and(|b| b.id == board.id);
                if est_le_sien(&slot.before) || est_le_sien(&slot.after) {
                    self.illisible = true;
                }
            }
            _ => {}
        }
    }

    /// Une case d'une liste : changée en place, ou posée **exactement** en fin de liste.
    ///
    /// Une pose ailleurs — même après la fin connue de l'index, mais devant une pose du même
    /// geste — décalerait un nœud déjà compté : elle rend le geste illisible, comme un retrait.
    fn case<T>(&mut self, liste: usize, slot: &Slot<T>) {
        match (&slot.before, &slot.after) {
            (_, None) => self.illisible = true,
            (None, Some(_)) => {
                if slot.index == self.avant[liste] + self.poses[liste] {
                    self.poses[liste] += 1;
                    self.ajouter(liste, std::iter::once(slot.index as u32));
                } else {
                    self.illisible = true;
                }
            }
            (Some(_), Some(_)) => self.ajouter(liste, std::iter::once(slot.index as u32)),
        }
    }

    fn ajouter(&mut self, liste: usize, indices: impl Iterator<Item = u32>) {
        let touches = &mut self.touches[liste];
        touches.extend(indices);
        touches.sort_unstable();
        touches.dedup();
    }

    /// Relit les rangs de l'index dans la numérotation du tableau tel qu'il est, puis y joint
    /// ce que le geste a touché.
    ///
    /// Le geste n'ayant fait que poser en fin de liste ou changer en place, chaque liste du
    /// présent commence par celle que l'index a vue : un rang d'avant ne change que du
    /// décalage des listes qui le précèdent.
    fn au_present(&self, rangs: &mut Vec<u32>, board: &Board) {
        let [i_avant, a_avant, _] = self.avant.map(|n| n as u32);
        let (i, a) = (board.images.len() as u32, board.annotations.len() as u32);
        if (i, a) != (i_avant, a_avant) {
            for r in rangs.iter_mut() {
                *r = if *r < i_avant {
                    *r
                } else if *r < i_avant + a_avant {
                    *r - i_avant + i
                } else {
                    *r - i_avant - a_avant + i + a
                };
            }
        }
        let [images, annotations, dossiers] = &self.touches;
        if images.is_empty() && annotations.is_empty() && dossiers.is_empty() {
            return;
        }
        rangs.extend(images.iter().copied());
        rangs.extend(annotations.iter().map(|k| k + i));
        rangs.extend(dossiers.iter().map(|k| k + i + a));
        rangs.sort_unstable();
        rangs.dedup();
    }
}

#[cfg(test)]
#[path = "geste_tests.rs"]
mod tests;
