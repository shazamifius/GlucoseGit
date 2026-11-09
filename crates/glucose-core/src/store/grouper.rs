//! **GROUPER-1 — grouper la sélection dans une membrane neuve** (fiche 98).
//!
//! La spécification d'origine l'annonçait (« Grouper dans une nouvelle membrane », au menu du clic
//! droit) ; Glucose Tauri ne l'a jamais écrit, et le menu de Glucose Rust le taisait faute de
//! geste. C'est pourtant le geste qui fait une région : choisir des nœuds, les entourer, nommer.
//!
//! # La loi
//!
//! * **Le cadre** est celui qu'un agent obtient en posant « dans » une membrane (MEMB-7) : la boîte
//!   de ce qui est choisi, un écart autour, et la place d'un titre au-dessus — une seule loi
//!   ([`Store::cadre_autour`]).
//! * **Les racines de la sélection** entrent dans la membrane neuve ; ce qu'une membrane choisie
//!   contient la suit sans changer d'appartenance (on emballe un pays avec ses villes).
//! * **La membrane neuve naît dans la plus proche membrane qui contient toutes les racines** — leur
//!   ancêtre commun —, qui grandit s'il le faut, ses parentes avec elle (MEMB-7). Sans ancêtre
//!   commun, elle est à la racine du tableau.
//! * Elle est choisie, et rien d'autre ne bouge. L'appelant en fait **un** geste.

use super::Store;
use crate::geometry::Rect;
use crate::membrane_space::MembershipChange;
use crate::types::{Annotation, Board};

impl Store {
    /// **Le cadre d'une membrane qui contient `boite`** : un écart de chaque côté et dessous, la
    /// place d'un titre au-dessus. La loi de MEMB-7 et de GROUPER-1.
    pub(super) fn cadre_autour(boite: Rect) -> Rect {
        let (ecart, titre) = (super::agents::ECART, super::agents::TITRE_DE_MEMBRANE);
        Rect::new(
            boite.left - ecart,
            boite.top - titre,
            boite.width + 2.0 * ecart,
            boite.height + titre + ecart,
        )
    }

    /// **Groupe la sélection dans une membrane neuve** ; rend son identifiant, ou rien si rien de
    /// ce qui est choisi n'a de boîte (une flèche seule, un dossier, rien).
    pub fn grouper_dans_une_membrane(&mut self) -> Option<String> {
        let board_id = self.project.active_board_id.clone();
        let board = self.active_board()?;
        let choisis: Vec<String> = self
            .selected_image_ids
            .iter()
            .chain(&self.selected_annotation_ids)
            .cloned()
            .collect();
        let racines = racines(board, &choisis);
        let boite = racines
            .iter()
            .filter_map(|id| boite_de(board, id))
            .reduce(|a, b| a.union(b))?;
        let parent = ancetre_commun(board, &racines);
        let cadre = Self::cadre_autour(boite);
        let id = self.generate_id("membrane");
        let membrane = Annotation::membrane(&id, cadre.left, cadre.top, cadre.width, cadre.height);
        self.add_annotation(&board_id, membrane);
        let mut changements = vec![MembershipChange {
            id: id.clone(),
            membrane_id: parent.clone(),
        }];
        changements.extend(racines.into_iter().map(|r| MembershipChange {
            id: r,
            membrane_id: Some(id.clone()),
        }));
        self.poser_les_appartenances(&board_id, &changements);
        if let Some(p) = parent {
            // Une membrane qui grandit fait grandir les siennes (MEMB-7).
            let _ = self.agrandir_la_membrane_pour(&board_id, &p, cadre);
        }
        Some(id)
    }
}

/// La membrane d'un nœud de ce tableau, s'il en a une.
fn membrane_de<'a>(board: &'a Board, id: &str) -> Option<&'a str> {
    if let Some(i) = board.images.iter().find(|i| i.id == id) {
        return i.membrane_id.as_deref();
    }
    board
        .annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(Annotation::membrane_id)
}

/// La chaîne des membranes qui contiennent `id`, de la plus proche à la plus lointaine. Une
/// appartenance en boucle (un document abîmé) s'arrête au premier retour.
fn ancetres<'a>(board: &'a Board, id: &str) -> Vec<&'a str> {
    let mut chaine: Vec<&'a str> = Vec::new();
    let mut courant = membrane_de(board, id);
    while let Some(m) = courant.filter(|m| !chaine.contains(m) && *m != id) {
        chaine.push(m);
        courant = membrane_de(board, m);
    }
    chaine
}

/// **Les racines de la sélection** : ce qui est choisi sans qu'une de ses membranes le soit.
fn racines(board: &Board, choisis: &[String]) -> Vec<String> {
    choisis
        .iter()
        .filter(|id| {
            !ancetres(board, id)
                .iter()
                .any(|m| choisis.iter().any(|c| c == m))
        })
        .cloned()
        .collect()
}

/// **La plus proche membrane qui contient toutes les racines**, ou aucune.
fn ancetre_commun(board: &Board, racines: &[String]) -> Option<String> {
    let (premiere, autres) = racines.split_first()?;
    ancetres(board, premiere)
        .into_iter()
        .find(|m| autres.iter().all(|r| ancetres(board, r).contains(m)))
        .map(str::to_string)
}

/// La boîte d'un nœud de ce tableau — une flèche n'en a pas.
fn boite_de(board: &Board, id: &str) -> Option<Rect> {
    if let Some(i) = board.images.iter().find(|i| i.id == id) {
        return Some(i.rect());
    }
    board
        .annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(Annotation::rect)
}

#[cfg(test)]
mod tests;
