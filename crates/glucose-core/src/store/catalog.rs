//! Presets, zones de tableau et métadonnées de projet.
//!
//! Les domaines vivent dans [`super::domains`] : ils portent leurs propres invariants
//! (cascade de suppression, ordre des assignations, validation des pondérations) et méritaient
//! un module à eux.

use super::Store;
use crate::types::{BoardZone, Preset};

impl Store {
    pub fn add_preset(&mut self, preset: Preset) {
        self.push_undo();
        self.project.presets.push(preset);
    }

    pub fn update_preset(&mut self, id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(p) = self.project.presets.iter_mut().find(|p| p.id == id) {
            p.name = name.into();
        }
    }

    pub fn remove_preset(&mut self, id: &str) {
        self.push_undo();
        self.project.presets.retain(|p| p.id != id);
    }

    pub fn apply_preset_to_board(&mut self, board_id: &str, preset_id: Option<&str>) {
        self.push_undo();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        b.zones.clear();
        let Some(pid) = preset_id else {
            return;
        };
        let Some(preset) = self.project.presets.iter().find(|p| p.id == pid) else {
            return;
        };
        for (i, slot) in preset.slots.iter().enumerate() {
            b.zones.push(BoardZone::new(&slot.id, (i as f64) * 250.0, 0.0, 240.0, 180.0));
        }
    }

    pub fn set_project_name(&mut self, name: impl Into<String>) {
        self.push_undo();
        self.project.name = name.into();
    }

    pub fn set_board_zones(&mut self, board_id: &str, zones: Vec<BoardZone>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.zones = zones;
        }
    }
}
