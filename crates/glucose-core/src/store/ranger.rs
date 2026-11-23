//! **RANGER-1 — ranger des nœuds dans un dossier** (fiche 134 ; ses notes du 23/11 au soir :
//! *« archive les bugs une fois qu'ils sont corrigés »*, *« éviter de supprimer, mais plutôt
//! archiver »*). Les nœuds quittent leur tableau pour celui que le dossier ouvre, en un geste que
//! `Ctrl+Z` défait d'un coup. Avant, seul « créer un dossier sur ce qu'il couvre » existait.
//!
//! # Les flèches
//!
//! * Une flèche dont les bouts partent tous (ou dont l'autre bout est libre) **part avec eux**.
//! * **Une flèche dont un seul bout part reste dehors, et vise le dossier** : le dossier tient lieu
//!   de ce qu'il contient. Rien ne pend dans le vide, et le lien garde son sens — « cette note parle
//!   de quelque chose dans l'archive ». tldraw, lui, retire le lien d'une flèche dont un bout change
//!   de page : elle devient libre, son sens perdu (<https://tldraw.dev/sdk-features/bindings>,
//!   consulté le 23/11/2026).
//! * Une flèche qui allait **au dossier** depuis un nœud rangé est arrivée : elle part avec le geste
//!   (`Ctrl+Z` la rend). Ses propres flèches vers « archive » le disaient d'avance.
//!
//! # Une membrane emporte ce qu'elle contient
//!
//! Ses membres (MEMB-1), et les leurs : ranger une membrane, c'est ranger sa région. Un membre dont
//! la membrane reste dehors arrive sans membrane.
//!
//! # Où ils arrivent
//!
//! Là où un agent poserait le prochain nœud de ce tableau (`place_pour_un_agent`) : à droite de
//! tout ce qu'il porte déjà. Une seule loi de placement ; leur disposition entre eux ne change pas.

use super::agents::Refus;
use super::journal::{Edit, Slot};
use super::Store;
use crate::geometry::Rect;
use crate::types::{Annotation, Board, BoardImage, CanvasFolder};
use std::collections::HashSet;

/// **Ce qu'un rangement a fait**, pour le dire à qui l'a demandé.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rangement {
    /// Le tableau que le dossier ouvre, où ils sont arrivés.
    pub ouvre: String,
    /// Les nœuds rangés, flèches emportées comprises.
    pub ranges: usize,
    /// Les flèches restées dehors, qui visent désormais le dossier.
    pub vers_le_dossier: usize,
    /// Les flèches qui allaient au dossier depuis un nœud rangé : arrivées, retirées.
    pub arrivees: usize,
}

/// Le sort d'une flèche du tableau quand `partent` s'en va.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sort {
    Reste,
    Part,
    /// Un seul bout part : il vise le dossier. `true` : la source.
    ViseLeDossier(bool),
    Arrivee,
}

/// **RANGER-1 — la loi des flèches** : voir l'en-tête du module.
fn sort(a: &Annotation, partent: &HashSet<String>, dossier: &str) -> Sort {
    let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = a
    else {
        return Sort::Reste;
    };
    let dedans = |b: &Option<String>| b.as_ref().is_some_and(|i| partent.contains(i));
    let libre_ou_dedans = |b: &Option<String>| b.is_none() || dedans(b);
    let au_dossier = |b: &Option<String>| b.as_deref() == Some(dossier);
    let (s, t) = (dedans(source_id), dedans(target_id));
    if !s && !t {
        // Une flèche libre nommée, ou membre d'une membrane qui part, part avec elle.
        let libre = source_id.is_none() && target_id.is_none();
        return if libre && partent.contains(a.id()) {
            Sort::Part
        } else {
            Sort::Reste
        };
    }
    if libre_ou_dedans(source_id) && libre_ou_dedans(target_id) {
        Sort::Part
    } else if (s && au_dossier(target_id)) || (t && au_dossier(source_id)) {
        Sort::Arrivee
    } else {
        Sort::ViseLeDossier(s)
    }
}

/// La flèche, son bout `source` (ou sa cible) posé sur le dossier : plus de passage désigné.
fn vers_le_dossier(mut a: Annotation, source: bool, dossier: &str) -> Annotation {
    if let Annotation::Arrow {
        source_id,
        target_id,
        source_block_id,
        target_block_id,
        source_text_sel,
        target_text_sel,
        ..
    } = &mut a
    {
        let (id, bloc, passage) = if source {
            (source_id, source_block_id, source_text_sel)
        } else {
            (target_id, target_block_id, target_text_sel)
        };
        *id = Some(dossier.to_string());
        *bloc = None;
        *passage = None;
    }
    a
}

/// **Ce qui part** : les nœuds nommés, et les membres des membranes qui partent, jusqu'au bout.
fn ce_qui_part(b: &Board, dossier: &str, ids: &[&str]) -> Result<HashSet<String>, Refus> {
    let mut partent = HashSet::new();
    for &id in ids {
        if id == dossier {
            return Err(Refus::DansSoiMeme(id.to_string()));
        }
        let existe = b.annotations.iter().any(|a| a.id() == id)
            || b.images.iter().any(|i| i.id == id)
            || b.folders.iter().any(|f| f.id == id);
        if !existe {
            return Err(Refus::NoeudInconnu(id.to_string()));
        }
        partent.insert(id.to_string());
    }
    loop {
        let membres: Vec<String> = b
            .annotations
            .iter()
            .filter_map(|a| Some((a.id(), a.membrane_id()?)))
            .chain(
                b.images
                    .iter()
                    .filter_map(|i| Some((i.id.as_str(), i.membrane_id.as_deref()?))),
            )
            .filter(|(id, m)| partent.contains(*m) && !partent.contains(*id))
            .map(|(id, _)| id.to_string())
            .collect();
        if membres.is_empty() {
            return Ok(partent);
        }
        partent.extend(membres);
    }
}

/// La boîte de ce qui part (sans les flèches, qui suivent leurs bouts), ou `None`.
fn boite_de(b: &Board, partent: &HashSet<String>) -> Option<Rect> {
    let annotations = b
        .annotations
        .iter()
        .filter(|a| partent.contains(a.id()) && !matches!(a, Annotation::Arrow { .. }))
        .map(Annotation::bounds);
    let images = b
        .images
        .iter()
        .filter(|i| partent.contains(&i.id))
        .map(|i| i.bounds());
    let dossiers = b
        .folders
        .iter()
        .filter(|f| partent.contains(&f.id))
        .map(|f| f.rect());
    annotations
        .chain(images)
        .chain(dossiers)
        .reduce(|a, c| a.union(c))
}

impl Store {
    /// **RANGER-1 — range ces nœuds dans le dossier `dossier` de ce tableau**, en un geste. Rend ce
    /// qui a été fait ; refuse un dossier ou un nœud inconnu, et un dossier rangé en lui-même.
    pub fn ranger_dans_un_dossier(
        &mut self,
        board: &str,
        dossier: &str,
        ids: &[&str],
    ) -> Result<Rangement, Refus> {
        let b = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board)
            .ok_or_else(|| Refus::TableauInconnu(board.to_string()))?;
        let dest = b
            .folders
            .iter()
            .find(|f| f.id == dossier)
            .map(|f| f.child_board_id.clone())
            .ok_or_else(|| Refus::PasUnDossier(dossier.to_string()))?;
        let mut partent = ce_qui_part(b, dossier, ids)?;
        let sorts: Vec<Sort> = b
            .annotations
            .iter()
            .map(|a| sort(a, &partent, dossier))
            .collect();
        // Une flèche ne part que si sa loi le dit : nommée, mais ses deux bouts dehors, elle
        // pendrait dans le vide de l'autre côté.
        for (a, s) in b.annotations.iter().zip(&sorts) {
            if matches!(a, Annotation::Arrow { .. }) {
                if *s == Sort::Part {
                    partent.insert(a.id().to_string());
                } else {
                    partent.remove(a.id());
                }
            }
        }
        let (px, py) = self.place_pour_un_agent(&dest, None)?;
        let b = self.project.boards.iter().find(|b| b.id == board);
        let decalage = b
            .and_then(|b| boite_de(b, &partent))
            .map_or((0.0, 0.0), |g| (px - g.left, py - g.top));
        let mut edits = Vec::new();
        let mut fait = Rangement {
            ouvre: dest.clone(),
            ..Rangement::default()
        };
        let bagage = match self.project.boards.iter_mut().find(|b| b.id == board) {
            Some(b) => partir(b, dossier, (&partent, &sorts), &mut edits, &mut fait),
            None => Bagage::default(),
        };
        fait.ranges = bagage.annotations.len() + bagage.images.len() + bagage.dossiers.len();
        if let Some(d) = self.project.boards.iter_mut().find(|b| b.id == dest) {
            arriver(d, bagage, &partent, decalage, &mut edits);
        }
        self.record_as_one_gesture(edits);
        self.selected_annotation_ids
            .retain(|i| !partent.contains(i));
        self.selected_image_ids.retain(|i| !partent.contains(i));
        self.selected_folder_ids.retain(|i| !partent.contains(i));
        Ok(fait)
    }
}

/// Ce qui part, sorti de son tableau, dans l'ordre de ses listes.
#[derive(Default)]
struct Bagage {
    annotations: Vec<Annotation>,
    images: Vec<BoardImage>,
    dossiers: Vec<CanvasFolder>,
}

/// **Le départ**, appliqué à mesure et consigné : les flèches qui visent le dossier d'abord (leurs
/// cases ne bougent pas encore), puis les départs, du dernier au premier — `Ctrl+Z` les remet à
/// leur place (JRN-2).
fn partir(
    b: &mut Board,
    dossier: &str,
    (partent, sorts): (&HashSet<String>, &[Sort]),
    edits: &mut Vec<Edit>,
    fait: &mut Rangement,
) -> Bagage {
    let ici = b.id.clone();
    let mut bagage = Bagage::default();
    for (i, s) in sorts.iter().enumerate() {
        if let Sort::ViseLeDossier(source) = *s {
            let avant = b.annotations[i].clone();
            b.annotations[i] = vers_le_dossier(avant.clone(), source, dossier);
            let slot = Slot::changed(i, avant, b.annotations[i].clone());
            edits.push(Edit::Annotation {
                board: ici.clone(),
                slot,
            });
            fait.vers_le_dossier += 1;
        }
    }
    for i in (0..b.annotations.len()).rev() {
        let arrivee = sorts[i] == Sort::Arrivee;
        if arrivee || partent.contains(b.annotations[i].id()) {
            let a = b.annotations.remove(i);
            let slot = Slot::removed(i, a.clone());
            edits.push(Edit::Annotation {
                board: ici.clone(),
                slot,
            });
            if arrivee {
                fait.arrivees += 1;
            } else {
                bagage.annotations.push(a);
            }
        }
    }
    for i in (0..b.images.len()).rev() {
        if partent.contains(&b.images[i].id) {
            let img = b.images.remove(i);
            let slot = Slot::removed(i, img.clone());
            edits.push(Edit::Image {
                board: ici.clone(),
                slot,
            });
            bagage.images.push(img);
        }
    }
    for i in (0..b.folders.len()).rev() {
        if partent.contains(&b.folders[i].id) {
            let f = b.folders.remove(i);
            let slot = Slot::removed(i, f.clone());
            edits.push(Edit::Folder {
                board: ici.clone(),
                slot,
            });
            bagage.dossiers.push(f);
        }
    }
    bagage
}

/// **L'arrivée**, au bout des listes du tableau du dossier, décalée de `(dx, dy)`, dans l'ordre
/// d'origine. Un membre dont la membrane reste dehors arrive sans membrane.
fn arriver(
    d: &mut Board,
    bagage: Bagage,
    partent: &HashSet<String>,
    (dx, dy): (f64, f64),
    edits: &mut Vec<Edit>,
) {
    let ici = d.id.clone();
    let garde = |m: Option<&str>| m.filter(|m| partent.contains(*m)).map(str::to_string);
    for mut a in bagage.annotations.into_iter().rev() {
        a.translate(dx, dy);
        let m = garde(a.membrane_id());
        a.set_membrane_id(m);
        let slot = Slot::inserted(d.annotations.len(), a.clone());
        edits.push(Edit::Annotation {
            board: ici.clone(),
            slot,
        });
        d.annotations.push(a);
    }
    for mut img in bagage.images.into_iter().rev() {
        img.x += dx;
        img.y += dy;
        img.membrane_id = garde(img.membrane_id.as_deref());
        let slot = Slot::inserted(d.images.len(), img.clone());
        edits.push(Edit::Image {
            board: ici.clone(),
            slot,
        });
        d.images.push(img);
    }
    for mut f in bagage.dossiers.into_iter().rev() {
        f.x += dx;
        f.y += dy;
        let slot = Slot::inserted(d.folders.len(), f.clone());
        edits.push(Edit::Folder {
            board: ici.clone(),
            slot,
        });
        d.folders.push(f);
    }
}

#[cfg(test)]
#[path = "ranger_tests.rs"]
mod tests;
