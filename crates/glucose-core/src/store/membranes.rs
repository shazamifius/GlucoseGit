//! **MEMB-1 — une membrane possède ce qu'on y dépose** (fiche 36, lot 4.2).
//!
//! # L'appartenance est écrite, pas redéduite
//!
//! Un élément garde ses vraies coordonnées ; ce qui dit qu'il appartient à une membrane est
//! son `membrane_id`. Le redéduire de la géométrie à chaque image serait faux dès qu'une
//! membrane minimisée rapetisse son contenu : elle le perdrait en le rangeant. La géométrie
//! ne fait donc que **proposer** — au dépôt d'un élément qu'on lâche, à la naissance d'un
//! élément qu'on pose —, et c'est [`crate::membrane_space::reconcile_membership`] qui juge :
//! la plus petite membrane qui contient le centre de l'élément tel qu'on le voit, jamais sa
//! propre descendance.
//!
//! # Ce qu'une membrane emporte
//!
//! Déplacer une membrane déplace ses membres, et les membres de ses membres. Glucose Tauri
//! laissait le contenu sur place — la moitié de ce que « posséder » veut dire. Supprimer une
//! membrane **libère** ses membres vers la membrane qui la contenait : le contenu ne se perd
//! jamais avec son cadre.
//!
//! Tout passe par le journal : annuler un dépôt rend la position **et** l'appartenance, en un
//! seul geste, et l'histoire du document les enregistre comme le reste.

use super::journal::{Edit, Slot};
use super::Store;
use crate::geometry::Rect;
use crate::membrane_space::{
    items_of_board, reconcile_membership, resolve_items, MembershipChange, ResolveOptions,
};
use crate::types::{Annotation, Board};
use std::collections::{HashMap, HashSet};

/// Ce qu'un déplacement emporte : la sélection, et le contenu des membranes qu'elle porte.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Emport {
    pub images: HashSet<String>,
    pub annotations: HashSet<String>,
}

impl Emport {
    /// La sélection, et tout ce que ses membranes possèdent — transitivement. Une appartenance
    /// en boucle (un document abîmé) ne fait pas tourner : un élément déjà emporté ne se
    /// revisite pas.
    pub fn de(board: &Board, images: &[String], annotations: &[String]) -> Self {
        let mut e = Self {
            images: images.iter().cloned().collect(),
            annotations: annotations.iter().cloned().collect(),
        };
        let mut a_vider: Vec<String> = board
            .annotations
            .iter()
            .filter(|a| est_une_membrane(a) && e.annotations.contains(a.id()))
            .map(|a| a.id().to_string())
            .collect();
        while let Some(m) = a_vider.pop() {
            for img in &board.images {
                if img.membrane_id.as_deref() == Some(m.as_str()) {
                    e.images.insert(img.id.clone());
                }
            }
            for a in &board.annotations {
                if a.membrane_id() == Some(m.as_str())
                    && e.annotations.insert(a.id().to_string())
                    && est_une_membrane(a)
                {
                    a_vider.push(a.id().to_string());
                }
            }
        }
        e
    }

    pub fn contient(&self, id: &str) -> bool {
        self.images.contains(id) || self.annotations.contains(id)
    }

    /// Les identifiants de tout ce qui est emporté.
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.images.iter().chain(&self.annotations)
    }

    /// Les boîtes de ce qui est emporté, sur ce tableau : ce que la zone à redessiner et
    /// l'aimantation demandent. Une flèche n'a pas de boîte.
    pub fn boites(&self, board: &Board) -> Vec<Rect> {
        let images = board
            .images
            .iter()
            .filter(|i| self.images.contains(&i.id))
            .map(|i| i.rect());
        let annotations = board
            .annotations
            .iter()
            .filter(|a| self.annotations.contains(a.id()))
            .filter_map(Annotation::rect);
        images.chain(annotations).collect()
    }
}

fn est_une_membrane(a: &Annotation) -> bool {
    matches!(a, Annotation::Membrane { .. })
}

/// La membrane qui accueille l'élément `id` déjà posé sur ce tableau, s'il en est une — ce
/// que sa naissance lui donne. Rien à calculer sur un tableau sans membrane.
pub(super) fn membrane_d_accueil(board: &Board, id: &str) -> Option<String> {
    if !board
        .annotations
        .iter()
        .any(|a| est_une_membrane(a) && a.id() != id)
    {
        return None;
    }
    let items = items_of_board(board);
    let vus = resolve_items(&items, ResolveOptions::default());
    reconcile_membership(&items, &vus, &[id.to_string()])
        .into_iter()
        .next()
        .and_then(|c| c.membrane_id)
}

impl Store {
    /// Ce que déplacer la sélection emporte sur ce tableau.
    pub fn ce_qu_emporte_la_selection(&self, board_id: &str) -> Emport {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map(|b| Emport::de(b, &self.selected_image_ids, &self.selected_annotation_ids))
            .unwrap_or_default()
    }

    /// **Le dépôt** : ce qu'on vient de lâcher appartient désormais à la plus petite membrane
    /// qui contient son centre — ou redevient libre. À appeler à la fin d'un geste qui a
    /// déplacé la sélection ; dans un geste encore ouvert, l'appartenance entre dans la même
    /// entrée d'annulation que le déplacement.
    pub fn rattacher_la_selection(&mut self, board_id: &str) {
        let deposes: Vec<String> = self
            .selected_image_ids
            .iter()
            .chain(&self.selected_annotation_ids)
            .cloned()
            .collect();
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return;
        };
        let sans_membrane = !b.annotations.iter().any(est_une_membrane);
        let sans_appartenance = b
            .images
            .iter()
            .map(|i| i.membrane_id.as_deref())
            .chain(b.annotations.iter().map(Annotation::membrane_id))
            .all(|m| m.is_none());
        if deposes.is_empty() || (sans_membrane && sans_appartenance) {
            return;
        }
        let items = items_of_board(b);
        let vus = resolve_items(&items, ResolveOptions::default());
        let mut changements = reconcile_membership(&items, &vus, &deposes);
        // L'ordre des écritures ne doit rien au hasard d'un ensemble : le journal, et
        // l'histoire qui le rejoue, restent les mêmes d'une exécution à l'autre.
        changements.sort_by(|a, b| a.id.cmp(&b.id));
        self.poser_les_appartenances(board_id, &changements);
    }

    /// Écrit ces appartenances, par le journal. Aucune flèche ne bouge : seule l'appartenance
    /// change.
    fn poser_les_appartenances(&mut self, board_id: &str, changements: &[MembershipChange]) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let mut edits = Vec::new();
        for c in changements {
            if let Some(i) = b.images.iter().position(|x| x.id == c.id) {
                let avant = b.images[i].clone();
                b.images[i].membrane_id = c.membrane_id.clone();
                edits.push(Edit::Image {
                    board: board_id.to_string(),
                    slot: Slot::changed(i, avant, b.images[i].clone()),
                });
            } else if let Some(i) = b.annotations.iter().position(|a| a.id() == c.id) {
                let avant = b.annotations[i].clone();
                b.annotations[i].set_membrane_id(c.membrane_id.clone());
                edits.push(Edit::Annotation {
                    board: board_id.to_string(),
                    slot: Slot::changed(i, avant, b.annotations[i].clone()),
                });
            }
        }
        self.record_as_one_gesture(edits);
    }
}

/// Les membres des membranes `partantes` passent à la membrane qui les contenait — la
/// première de leurs ascendantes qui reste. Rend les écritures, à faire **avant** celles qui
/// retirent les membranes : leurs index sont encore ceux du tableau entier.
pub(super) fn liberer_les_membres(board: &mut Board, partantes: &HashSet<&str>) -> Vec<Edit> {
    let parent: HashMap<String, Option<String>> = board
        .annotations
        .iter()
        .filter(|a| est_une_membrane(a) && partantes.contains(a.id()))
        .map(|a| (a.id().to_string(), a.membrane_id().map(str::to_string)))
        .collect();
    if parent.is_empty() {
        return Vec::new();
    }
    let heritiere = |depart: &str| -> Option<String> {
        let mut vus = HashSet::new();
        let mut m = parent.get(depart).cloned().flatten();
        while let Some(courante) = m.clone() {
            if !parent.contains_key(&courante) || !vus.insert(courante.clone()) {
                break;
            }
            m = parent.get(&courante).cloned().flatten();
        }
        m.filter(|x| !parent.contains_key(x))
    };
    let mut edits = Vec::new();
    for (i, img) in board.images.iter_mut().enumerate() {
        let Some(m) = img.membrane_id.clone().filter(|m| parent.contains_key(m)) else {
            continue;
        };
        let avant = img.clone();
        img.membrane_id = heritiere(&m);
        edits.push(Edit::Image {
            board: board.id.clone(),
            slot: Slot::changed(i, avant, img.clone()),
        });
    }
    for (i, a) in board.annotations.iter_mut().enumerate() {
        if partantes.contains(a.id()) {
            continue;
        }
        let Some(m) = a
            .membrane_id()
            .map(str::to_string)
            .filter(|m| parent.contains_key(m))
        else {
            continue;
        };
        let avant = a.clone();
        a.set_membrane_id(heritiere(&m));
        edits.push(Edit::Annotation {
            board: board.id.clone(),
            slot: Slot::changed(i, avant, a.clone()),
        });
    }
    edits
}
