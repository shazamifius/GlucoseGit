//! Presets et métadonnées de projet. Les zones d'un tableau vivent dans [`super::zones`].
//!
//! Les domaines vivent dans [`super::domains`] : ils portent leurs propres invariants
//! (cascade de suppression, ordre des assignations, validation des pondérations) et méritaient
//! un module à eux.

use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::types::Preset;

impl Store {
    /// **Site migré vers le journal.**
    pub fn add_preset(&mut self, preset: Preset) {
        let index = self.project.presets.len();
        self.project.presets.push(preset.clone());
        self.record_edit(Edit::Preset {
            slot: Slot::inserted(index, preset),
        });
    }

    /// **Site migré vers le journal.**
    pub fn update_preset(&mut self, id: &str, name: impl Into<String>) {
        let Some(i) = self.project.presets.iter().position(|p| p.id == id) else {
            return;
        };
        let before = self.project.presets[i].clone();
        self.project.presets[i].name = name.into();
        let after = self.project.presets[i].clone();
        self.record_edit(Edit::Preset {
            slot: Slot::changed(i, before, after),
        });
    }

    /// **Site migré vers le journal.**
    pub fn remove_preset(&mut self, id: &str) {
        let Some(i) = self.project.presets.iter().position(|p| p.id == id) else {
            return;
        };
        let removed = self.project.presets.remove(i);
        self.record_edit(Edit::Preset {
            slot: Slot::removed(i, removed),
        });
    }

    /// **Site migré vers le journal.**
    pub fn set_project_name(&mut self, name: impl Into<String>) {
        let before = self.project.name.clone();
        self.project.name = name.into();
        let after = self.project.name.clone();
        self.record_edit(Edit::ProjectName {
            whole: Whole::new(before, after),
        });
    }
}
