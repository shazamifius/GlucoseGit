//! Presets, zones de tableau et métadonnées de projet.
//!
//! Les domaines vivent dans [`super::domains`] : ils portent leurs propres invariants
//! (cascade de suppression, ordre des assignations, validation des pondérations) et méritaient
//! un module à eux.

use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::types::{BoardZone, Preset};

impl Store {
    /// **Site migré vers le journal.**
    pub fn add_preset(&mut self, preset: Preset) {
        let index = self.project.presets.len();
        self.project.presets.push(preset.clone());
        self.record_edit(Edit::Preset { slot: Slot::inserted(index, preset) });
    }

    /// **Site migré vers le journal.**
    pub fn update_preset(&mut self, id: &str, name: impl Into<String>) {
        let Some(i) = self.project.presets.iter().position(|p| p.id == id) else {
            return;
        };
        let before = self.project.presets[i].clone();
        self.project.presets[i].name = name.into();
        let after = self.project.presets[i].clone();
        self.record_edit(Edit::Preset { slot: Slot::changed(i, before, after) });
    }

    /// **Site migré vers le journal.**
    pub fn remove_preset(&mut self, id: &str) {
        let Some(i) = self.project.presets.iter().position(|p| p.id == id) else {
            return;
        };
        let removed = self.project.presets.remove(i);
        self.record_edit(Edit::Preset { slot: Slot::removed(i, removed) });
    }

    /// Pose les zones d'un board d'après un preset. Sans preset, les zones sont effacées.
    ///
    /// **Site migré vers le journal.** Les zones sont remplacées d'un bloc : l'entrée porte
    /// la liste avant et après, ce qui *est* la taille de la modification.
    pub fn apply_preset_to_board(&mut self, board_id: &str, preset_id: Option<&str>) {
        // Les slots sont lus avant d'emprunter le board en écriture.
        let slots: Vec<String> = preset_id
            .and_then(|pid| self.project.presets.iter().find(|p| p.id == pid))
            .map(|p| p.slots.iter().map(|s| s.id.clone()).collect())
            .unwrap_or_default();

        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let before = b.zones.clone();
        b.zones = slots
            .iter()
            .enumerate()
            .map(|(i, id)| BoardZone::new(id, (i as f64) * 250.0, 0.0, 240.0, 180.0))
            .collect();
        let after = b.zones.clone();

        self.record_edit(Edit::Zones {
            board: board_id.to_string(),
            whole: Whole::new(before, after),
        });
    }

    /// **Site migré vers le journal.**
    pub fn set_project_name(&mut self, name: impl Into<String>) {
        let before = self.project.name.clone();
        self.project.name = name.into();
        let after = self.project.name.clone();
        self.record_edit(Edit::ProjectName { whole: Whole::new(before, after) });
    }

    /// **Site migré vers le journal.**
    pub fn set_board_zones(&mut self, board_id: &str, zones: Vec<BoardZone>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let before = std::mem::replace(&mut b.zones, zones);
        let after = b.zones.clone();
        self.record_edit(Edit::Zones {
            board: board_id.to_string(),
            whole: Whole::new(before, after),
        });
    }
}
