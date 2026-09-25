//! Création, renommage, rangement et suppression de tableaux.
//!
//! # BOARDS-1 — un tableau vit tant qu'on peut l'atteindre
//!
//! Un **onglet** est un tableau qu'aucun dossier ne contient. Les autres sont le contenu d'un
//! dossier, et un dossier miroir **partage** le tableau de sa source : il recopie son
//! `child_board_id`. Supprimer un miroir retirait donc le contenu de l'original, et supprimer
//! un dossier laissait orphelins les tableaux des dossiers qu'il contenait.
//!
//! Une seule règle répond aux trois, celle d'un ramasse-miettes : après un retrait, on suit
//! les dossiers depuis les onglets qui restent, et ce qu'on n'atteint plus part — dans le même
//! geste, qu'un `Ctrl+Z` rend en entier. Un contenu qu'un miroir tient encore reste.

use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, Board};
use std::collections::HashSet;

/// Les tableaux qu'aucun dossier ne contient : les onglets, dans l'ordre du projet.
pub(crate) fn racines(boards: &[Board]) -> Vec<String> {
    let contenus: HashSet<&str> = boards
        .iter()
        .flat_map(|b| b.folders.iter().map(|f| f.child_board_id.as_str()))
        .collect();
    boards
        .iter()
        .filter(|b| !contenus.contains(b.id.as_str()))
        .map(|b| b.id.clone())
        .collect()
}

/// Ce qu'on atteint depuis ces tableaux en suivant les dossiers, eux compris.
///
/// Un parcours en largeur, et chaque tableau n'est visité qu'une fois : un dossier qui se
/// contiendrait lui-même ne ferait pas tourner la boucle.
fn atteints(boards: &[Board], depuis: &[String]) -> HashSet<String> {
    let mut vus: HashSet<String> = HashSet::new();
    let mut a_voir: Vec<&str> = depuis.iter().map(String::as_str).collect();
    while let Some(id) = a_voir.pop() {
        if !vus.insert(id.to_string()) {
            continue;
        }
        if let Some(b) = boards.iter().find(|b| b.id == id) {
            a_voir.extend(b.folders.iter().map(|f| f.child_board_id.as_str()));
        }
    }
    vus
}

impl Store {
    /// Les onglets : les tableaux racines, dans l'ordre, avec leur nom, et s'ils portent le
    /// tableau actif — lui-même, ou un dossier qu'ils contiennent.
    ///
    /// Ce que la barre d'onglets a besoin de savoir, et rien de plus : elle lisait la
    /// collection du modèle, ce que la règle S interdit au desktop (fiche 12 § 3). Elle
    /// montrait aussi le contenu des dossiers comme des onglets ; Glucose Tauri ne montrait
    /// que les racines (`BoardTabs.tsx`).
    pub fn onglets(&self) -> impl Iterator<Item = (&str, &str, bool)> {
        let racines: HashSet<String> = racines(&self.project.boards).into_iter().collect();
        let actif = self.racine_de(&self.project.active_board_id);
        self.project
            .boards
            .iter()
            .filter(move |b| racines.contains(&b.id))
            .map(move |b| (b.id.as_str(), b.name.as_str(), b.id == actif))
    }

    /// L'onglet qui porte ce tableau : lui-même, ou celui du dossier qui le contient.
    pub fn racine_de(&self, board_id: &str) -> String {
        super::build_folder_stack(&self.project.boards, board_id)
            .first()
            .map_or_else(|| board_id.to_string(), |(parent, _)| parent.clone())
    }

    /// **Range l'onglet `id` juste avant l'onglet `avant`**, ou au bout (BOARDS-1).
    ///
    /// Seul lui bouge : tout le reste garde sa place relative, le contenu des dossiers
    /// compris. Le geste n'écrit que l'ordre ([`Edit::BoardOrder`]).
    pub fn try_move_board(&mut self, id: &str, avant: Option<&str>) -> CoreResult<()> {
        let racines = racines(&self.project.boards);
        for cible in std::iter::once(id).chain(avant) {
            if !racines.iter().any(|r| r == cible) {
                return Err(CoreError::BoardNotFound(cible.to_string()));
            }
        }
        let ordre_avant: Vec<String> = self.project.boards.iter().map(|b| b.id.clone()).collect();
        let mut ordre = ordre_avant.clone();
        ordre.retain(|b| b != id);
        let place = avant.and_then(|a| ordre.iter().position(|b| b == a));
        ordre.insert(place.unwrap_or(ordre.len()), id.to_string());
        if ordre == ordre_avant {
            return Ok(());
        }
        super::journal::edit::ranger(&mut self.project.boards, &ordre);
        self.record_edit(Edit::BoardOrder {
            whole: Whole::new(ordre_avant, ordre),
        });
        Ok(())
    }

    /// **Site migré vers le journal.** Renommer ne clone pas le board : l'entrée ne porte
    /// que les deux noms.
    pub fn rename_board(&mut self, board_id: &str, name: impl Into<String>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let before = b.name.clone();
        b.name = name.into();
        let after = b.name.clone();
        self.record_edit(Edit::BoardName {
            board: board_id.to_string(),
            whole: Whole::new(before, after),
        });
    }

    /// **Site migré vers le journal.**
    pub fn add_board(&mut self, name: impl Into<String>) -> String {
        let id = self.generate_id("board");
        let board = Board::new(&id, name);
        let index = self.project.boards.len();
        self.project.boards.push(board.clone());
        self.record_edit(Edit::Board {
            slot: Slot::inserted(index, board),
        });
        id
    }

    /// **Supprime un onglet**, et ce que ses dossiers contenaient, ou dit pourquoi il ne l'a
    /// pas fait (R-35).
    ///
    /// INVARIANT BOARD-1 — un projet garde toujours au moins un onglet : supprimer le dernier
    /// laisserait l'application sans surface de dessin et sans board actif valide.
    ///
    /// Un seul geste : l'onglet part avec tout ce qu'on n'atteint plus (BOARDS-1), le tableau
    /// actif passe à l'onglet d'avant — celui de gauche, comme Glucose Tauri —, et les flèches
    /// portails qui visaient ce qui part sont neutralisées. Un `Ctrl+Z` remet le tout.
    pub fn try_remove_board(&mut self, id: &str) -> CoreResult<()> {
        let avant = racines(&self.project.boards);
        let Some(rang) = avant.iter().position(|r| r == id) else {
            return Err(if self.project.boards.iter().any(|b| b.id == id) {
                CoreError::InvalidOperation(format!(
                    "'{id}' est le contenu d'un dossier : c'est le dossier qu'il faut supprimer"
                ))
            } else {
                CoreError::BoardNotFound(id.to_string())
            });
        };
        if avant.len() <= 1 {
            return Err(CoreError::InvalidOperation(format!(
                "impossible de supprimer '{}' : c'est le dernier tableau du projet — en créer un autre d'abord",
                id
            )));
        }
        let restants: Vec<String> = avant.iter().filter(|r| *r != id).cloned().collect();
        let mut edits = self.retirer_l_inatteignable(&restants);
        let actif_parti = !self
            .project
            .boards
            .iter()
            .any(|b| b.id == self.project.active_board_id);
        if actif_parti {
            let apres = restants[rang.saturating_sub(1).min(restants.len() - 1)].clone();
            let before = std::mem::replace(&mut self.project.active_board_id, apres.clone());
            edits.push(Edit::ActiveBoard {
                whole: Whole::new(before, apres),
            });
        }
        self.record_as_one_gesture(edits);
        Ok(())
    }

    /// **Retire ce qu'on n'atteint plus depuis ces onglets**, et rend les éditions du geste :
    /// les tableaux, puis les flèches portails qui les visaient.
    ///
    /// Les onglets se donnent : ils se lisent **avant** le retrait. Après, le contenu d'un
    /// dossier retiré n'est plus contenu par personne, et se prendrait pour un onglet.
    pub(super) fn retirer_l_inatteignable(&mut self, onglets: &[String]) -> Vec<Edit> {
        let vivants = atteints(&self.project.boards, onglets);
        let mut edits = Vec::new();
        let mut partis = Vec::new();
        for i in (0..self.project.boards.len()).rev() {
            if vivants.contains(&self.project.boards[i].id) {
                continue;
            }
            let removed = self.project.boards.remove(i);
            partis.push(removed.id.clone());
            edits.push(Edit::Board {
                slot: Slot::removed(i, removed),
            });
        }
        for id in &partis {
            edits.extend(self.clear_portal_arrows_to(id));
        }
        edits
    }

    /// Enveloppe silencieuse de [`Store::try_remove_board`]. Rend `false` en cas d'échec, sans
    /// dire lequel des deux (tableau inexistant / dernier tableau) s'est produit.
    #[deprecated(note = "R-35 : l'échec est muet. Utiliser `try_remove_board`.")]
    pub fn remove_board(&mut self, id: &str) -> bool {
        self.try_remove_board(id).is_ok()
    }

    /// Neutralise les flèches portails qui pointaient vers un tableau supprimé, et rend les
    /// éditions correspondantes.
    ///
    /// Seule une flèche réellement concernée est clonée : le coût suit le nombre de portails
    /// cassés, pas le nombre d'annotations du projet.
    fn clear_portal_arrows_to(&mut self, removed_board_id: &str) -> Vec<Edit> {
        let mut edits = Vec::new();
        for b in &mut self.project.boards {
            for (i, a) in b.annotations.iter_mut().enumerate() {
                let concerned = matches!(
                    a,
                    Annotation::Arrow { target_board_id, .. }
                        if target_board_id.as_deref() == Some(removed_board_id)
                );
                if !concerned {
                    continue;
                }
                let before = a.clone();
                if let Annotation::Arrow {
                    target_board_id, ..
                } = a
                {
                    *target_board_id = None;
                }
                edits.push(Edit::Annotation {
                    board: b.id.clone(),
                    slot: Slot::changed(i, before, a.clone()),
                });
            }
        }
        edits
    }
}
