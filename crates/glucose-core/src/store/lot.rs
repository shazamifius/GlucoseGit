//! **Le lot** : ce que `Ctrl+C` emporte de la sélection, et ce que `Ctrl+V` en pose (fiche 51
//! § 2).
//!
//! # Un lot est un document
//!
//! Ce que la sélection emporte s'écrit comme un [`Project`] d'un seul tableau : ses nœuds tels
//! quels, coordonnées comprises, et les domaines qu'ils portent. Le bureau l'enveloppe ensuite
//! dans un vrai `.glucose` ([`crate::persist::encode`]), images comprises. Il n'y a donc pas de
//! « format du presse-papiers » à maintenir à côté du format du fichier : c'est le même, avec
//! ses sommes de contrôle, et ce qui sait relire un document sait relire un lot.
//!
//! # Ce que la sélection emporte
//!
//! Ce qu'un glisser emporterait ([`Emport`]) : la sélection, et tout ce que ses membranes
//! possèdent. Plus **les flèches qui relient deux nœuds emportés**, même non sélectionnées — sans
//! elles, copier deux nœuds reliés en ferait deux nœuds sans lien. Une flèche sélectionnée dont
//! un bout reste dehors part aussi : ce bout devient libre, à la place qu'il avait.
//!
//! # Coller, c'est importer un document dans le tableau
//!
//! Chaque nœud reçoit un identifiant neuf et chaque référence suit, par le renommage de l'import
//! (BOARDS-2) : une référence vers ce que le lot ne porte pas **disparaît**, au lieu de viser par
//! hasard un nœud d'ici qui porterait le même nom. Deux exceptions, et elles ont une raison :
//!
//! * **un lot de ce document** garde ses liens vers les tableaux et les domaines qui existent
//!   encore ici — un portail vers un autre onglet, une assignation de domaine ;
//! * **un lot venu d'ailleurs** retrouve un domaine d'ici quand il lui est identique en tout,
//!   et apporte le sien sinon.
//!
//! Le lot est posé **centré sur le curseur**, ses places relatives intactes, en un seul geste :
//! un `Ctrl+Z` le retire en entier. Ce qui tombe dans une membrane lui appartient, comme
//! n'importe quel nœud qui naît là (MEMB-1).

use super::importer::{suivre_dans_le_tableau, Renommage};
use super::journal::{Edit, Slot};
use super::membranes::Emport;
use super::Store;
use crate::geometry::Rect;
use crate::membrane_space::{items_of_board, reconcile_membership, resolve_items, ResolveOptions};
use crate::types::{Annotation, Board, Project};
use std::collections::HashSet;

/// D'où vient le lot qu'on colle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// De ce document même : ses tableaux et ses domaines sont ceux d'ici.
    CeDocument,
    /// D'un autre document — une autre fenêtre, une autre session.
    Ailleurs,
}

impl Store {
    /// **Le lot que la sélection emporte** sur ce tableau, ou `None` si elle n'emporte rien.
    pub fn extraire_la_selection(&self, board_id: &str) -> Option<Project> {
        let board = self.project.boards.iter().find(|b| b.id == board_id)?;
        let emport = Emport::de(
            board,
            &self.selected_image_ids,
            &self.selected_annotation_ids,
        );
        let mut contenu = Board::new(board.id.clone(), board.name.clone());
        contenu.images = board
            .images
            .iter()
            .filter(|i| emport.images.contains(&i.id))
            .cloned()
            .collect();
        contenu.annotations = board
            .annotations
            .iter()
            .filter(|a| emport.annotations.contains(a.id()) || relie_deux_du_lot(a, &emport))
            .cloned()
            .collect();
        if contenu.images.is_empty() && contenu.annotations.is_empty() {
            return None;
        }
        let portes: HashSet<&str> = contenu
            .images
            .iter()
            .flat_map(|i| &i.domains)
            .chain(contenu.annotations.iter().flat_map(Annotation::domains))
            .map(|d| d.domain_id.as_str())
            .collect();
        let mut lot = Project::new(self.project.name.clone());
        lot.domains = self
            .project
            .domains
            .iter()
            .filter(|d| portes.contains(d.id.as_str()))
            .cloned()
            .collect();
        lot.active_board_id = contenu.id.clone();
        lot.boards = vec![contenu];
        Some(lot)
    }

    /// **Pose ce lot sur ce tableau**, centré sur `centre`, en un seul geste, et rend les
    /// identifiants posés — qui deviennent la sélection.
    pub fn coller_un_lot(
        &mut self,
        board_id: &str,
        mut lot: Project,
        centre: (f64, f64),
        provenance: Provenance,
    ) -> Vec<String> {
        if !self.project.boards.iter().any(|b| b.id == board_id) {
            return Vec::new();
        }
        let Some(mut contenu) = lot.boards.drain(..).next() else {
            return Vec::new();
        };
        // Un lot ne porte que des nœuds : un dossier y mènerait à un tableau qu'il n'apporte pas.
        contenu.folders.clear();
        contenu.panels.clear();
        let mut edits = Vec::new();
        let mut r = Renommage::default();
        if provenance == Provenance::CeDocument {
            for b in &self.project.boards {
                r.0.insert(b.id.clone(), b.id.clone());
            }
        }
        for d in std::mem::take(&mut lot.domains) {
            self.accueillir_le_domaine(&mut r, d, provenance, &mut edits);
        }
        self.nommer_le_contenu(&mut r, &mut contenu);
        suivre_dans_le_tableau(&r, &mut contenu);
        if let Some(boite) = boite_du_contenu(&contenu) {
            let c = boite.center();
            let (dx, dy) = (centre.0 - c.x, centre.1 - c.y);
            for i in &mut contenu.images {
                i.x += dx;
                i.y += dy;
            }
            for a in &mut contenu.annotations {
                a.translate(dx, dy);
            }
        }
        let poses = self.poser_le_contenu(board_id, contenu, &mut edits);
        self.record_as_one_gesture(edits);
        poses
    }

    /// Le domaine du lot devient un domaine d'ici : le même s'il existe déjà, un neuf sinon.
    fn accueillir_le_domaine(
        &mut self,
        r: &mut Renommage,
        mut d: crate::types::Domain,
        provenance: Provenance,
        edits: &mut Vec<Edit>,
    ) {
        let ici = self.project.domains.iter().find(|ici| match provenance {
            Provenance::CeDocument => ici.id == d.id,
            Provenance::Ailleurs => **ici == d,
        });
        if let Some(ici) = ici {
            r.0.insert(d.id.clone(), ici.id.clone());
            return;
        }
        r.nommer(self, "domain", &mut d.id);
        edits.push(Edit::Domain {
            slot: Slot::inserted(self.project.domains.len(), d.clone()),
        });
        self.project.domains.push(d);
    }

    /// Ajoute le contenu au tableau, donne une membrane d'accueil à ce qui n'en a pas, et
    /// consigne chaque nœud posé.
    fn poser_le_contenu(
        &mut self,
        board_id: &str,
        contenu: Board,
        edits: &mut Vec<Edit>,
    ) -> Vec<String> {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let (premiere_image, premiere_annotation) = (b.images.len(), b.annotations.len());
        let racines: Vec<String> = contenu
            .images
            .iter()
            .filter(|i| i.membrane_id.is_none())
            .map(|i| i.id.clone())
            .chain(
                contenu
                    .annotations
                    .iter()
                    .filter(|a| a.membrane_id().is_none() && !matches!(a, Annotation::Arrow { .. }))
                    .map(|a| a.id().to_string()),
            )
            .collect();
        b.images.extend(contenu.images);
        b.annotations.extend(contenu.annotations);
        // MEMB-1 : ce qui tombe dans une membrane lui appartient, comme à sa naissance.
        if b.annotations
            .iter()
            .any(|a| matches!(a, Annotation::Membrane { .. }))
        {
            let items = items_of_board(b);
            let vus = resolve_items(&items, ResolveOptions::default());
            for change in reconcile_membership(&items, &vus, &racines) {
                if let Some(i) = b.images[premiere_image..]
                    .iter_mut()
                    .find(|i| i.id == change.id)
                {
                    i.membrane_id = change.membrane_id;
                } else if let Some(a) = b.annotations[premiere_annotation..]
                    .iter_mut()
                    .find(|a| a.id() == change.id)
                {
                    a.set_membrane_id(change.membrane_id);
                }
            }
        }
        let mut images = Vec::new();
        for (index, img) in b.images.iter().enumerate().skip(premiere_image) {
            images.push(img.id.clone());
            edits.push(Edit::Image {
                board: board_id.to_string(),
                slot: Slot::inserted(index, img.clone()),
            });
        }
        let mut annotations = Vec::new();
        for (index, ann) in b.annotations.iter().enumerate().skip(premiere_annotation) {
            annotations.push(ann.id().to_string());
            edits.push(Edit::Annotation {
                board: board_id.to_string(),
                slot: Slot::inserted(index, ann.clone()),
            });
        }
        // Ce qu'on vient de poser devient la sélection : c'est ce qu'on déplace ensuite.
        self.clear_selection();
        self.selected_image_ids = images.clone();
        self.selected_annotation_ids = annotations.clone();
        let mut poses = images;
        poses.extend(annotations);
        poses
    }
}

/// Ce qu'un lot porte, pour qui le prépare : les identifiants de ses images et de ses
/// annotations — ce que couper retirera —, et les clés de ses images, sans doublon — les
/// octets à joindre.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Inventaire {
    pub images: Vec<String>,
    pub annotations: Vec<String>,
    pub cles: Vec<String>,
}

/// L'inventaire d'un lot que [`Store::extraire_la_selection`] a rendu.
pub fn inventaire(lot: &Project) -> Inventaire {
    let mut inv = Inventaire::default();
    for b in &lot.boards {
        inv.images.extend(b.images.iter().map(|i| i.id.clone()));
        inv.annotations
            .extend(b.annotations.iter().map(|a| a.id().to_string()));
        inv.cles
            .extend(b.images.iter().filter_map(|i| i.src.clone()));
    }
    inv.cles.sort();
    inv.cles.dedup();
    inv
}

/// Une flèche dont les deux bouts sont emportés : elle relie deux nœuds du lot.
fn relie_deux_du_lot(a: &Annotation, e: &Emport) -> bool {
    let Annotation::Arrow {
        source_id: Some(s),
        target_id: Some(t),
        ..
    } = a
    else {
        return false;
    };
    e.contient(s) && e.contient(t)
}

/// La boîte de tout ce que le contenu porte, flèches comprises.
fn boite_du_contenu(b: &Board) -> Option<Rect> {
    b.images
        .iter()
        .map(|i| i.rect())
        .chain(b.annotations.iter().map(Annotation::bounds))
        .reduce(|a, b| a.union(b))
}

#[cfg(test)]
mod tests;
