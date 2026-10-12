//! **Les zones d'un tableau** : poser un gabarit, déplacer, redimensionner, retirer une zone —
//! et l'appartenance d'une image à la sienne (PRESET-1, fiche 62).
//!
//! Chaque geste est **une** entrée du journal : les zones du tableau, avant et après. Un
//! gabarit de dix zones posé d'un coup s'annule d'un coup.
//!
//! # L'appartenance ne s'écrit pas
//!
//! Glucose Tauri prévoyait qu'une image porte sa catégorie (`slotId`) ; aucune ligne ne l'a
//! jamais écrite, et son rangement « par slot » rangeait tout dans une seule colonne. Ici, une
//! image **appartient à la zone où son centre est posé** — comme une membrane possède ce qu'on
//! y dépose. Rien à tenir à jour : déplacer l'image change sa zone, et la géométrie le dit.

use super::journal::{Edit, Whole};
use super::Store;
use crate::gabarits::identifiant;
use crate::types::{BoardImage, BoardZone};

/// La taille d'une zone posée, et l'écart entre deux — ceux de Glucose Tauri
/// (`applyPresetToBoard`).
pub const ZONE: (f64, f64) = (340.0, 700.0);
pub const ECART: f64 = 30.0;

impl Store {
    /// **Pose ce gabarit sur le tableau**, une zone par catégorie, en rangée centrée sur
    /// `centre` ; les zones d'avant partent — un tableau suit un gabarit.
    ///
    /// Le gabarit se dit par son identifiant et ceux de ses catégories : un gabarit intégré
    /// ([`crate::gabarits::INTEGRES`]) comme un gabarit du document (`Project::presets`).
    pub fn poser_un_gabarit(
        &mut self,
        board_id: &str,
        (gabarit, categories): (&str, &[&str]),
        centre: (f64, f64),
    ) {
        let n = categories.len() as f64;
        let largeur = n * ZONE.0 + (n - 1.0).max(0.0) * ECART;
        let (gauche, haut) = (centre.0 - largeur / 2.0, centre.1 - ZONE.1 / 2.0);
        let zones = categories
            .iter()
            .enumerate()
            .map(|(i, cat)| {
                BoardZone::new(
                    identifiant(gabarit, cat),
                    gauche + i as f64 * (ZONE.0 + ECART),
                    haut,
                    ZONE.0,
                    ZONE.1,
                )
            })
            .collect();
        self.remplacer_les_zones(board_id, |z| *z = zones);
    }

    /// **Pose ces zones**, toutes d'un coup, à la place de celles du tableau.
    pub fn poser_les_zones(&mut self, board_id: &str, zones: Vec<BoardZone>) {
        self.remplacer_les_zones(board_id, |z| *z = zones);
    }

    /// **Retire le gabarit** : toutes les zones du tableau. Les images restent où elles sont.
    pub fn retirer_le_gabarit(&mut self, board_id: &str) {
        self.remplacer_les_zones(board_id, Vec::clear);
    }

    /// **Pose cette zone à cette place et à cette taille** — un glisser ou un redimensionnement
    /// qui finit.
    pub fn placer_la_zone(
        &mut self,
        board_id: &str,
        zone: &str,
        (x, y, w, h): (f64, f64, f64, f64),
    ) {
        self.remplacer_les_zones(board_id, |zones| {
            if let Some(z) = zones.iter_mut().find(|z| z.slot_id == zone) {
                (z.x, z.y, z.width, z.height) = (x, y, w, h);
            }
        });
    }

    /// **Retire une zone**, et elle seule.
    pub fn retirer_la_zone(&mut self, board_id: &str, zone: &str) {
        self.remplacer_les_zones(board_id, |zones| zones.retain(|z| z.slot_id != zone));
    }

    /// Les zones de ce tableau.
    pub fn zones(&self, board_id: &str) -> &[BoardZone] {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map_or(&[], |b| b.zones.as_slice())
    }

    /// **La zone d'une image** : celle où son centre est posé — la plus haute, si deux se
    /// recouvrent : la dernière posée se dessine par-dessus.
    pub fn zone_de<'a>(&'a self, board_id: &str, image: &BoardImage) -> Option<&'a BoardZone> {
        self.zones(board_id).iter().rev().find(|z| {
            (z.x..=z.x + z.width).contains(&image.x) && (z.y..=z.y + z.height).contains(&image.y)
        })
    }

    /// Change les zones du tableau, et consigne avant et après en une entrée.
    fn remplacer_les_zones(&mut self, board_id: &str, changer: impl FnOnce(&mut Vec<BoardZone>)) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let avant = b.zones.clone();
        changer(&mut b.zones);
        let apres = b.zones.clone();
        self.record_edit(Edit::Zones {
            board: board_id.to_string(),
            whole: Whole::new(avant, apres),
        });
    }
}

#[cfg(test)]
mod tests;
