//! Presets, domaines et métadonnées de projet.

use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{BoardZone, Domain, DomainAssignment, Preset};

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

    pub fn add_domain(&mut self, domain: Domain) {
        self.push_undo();
        self.project.domains.push(domain);
    }

    pub fn update_domain(&mut self, id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(d) = self.project.domains.iter_mut().find(|d| d.id == id) {
            d.name = name.into();
        }
    }

    pub fn remove_domain(&mut self, id: &str) {
        self.push_undo();
        self.project.domains.retain(|d| d.id != id);
    }

    /// Affecte un domaine à un nœud, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_assign_domain_to_node(
        &mut self,
        board_id: &str,
        node_id: &str,
        domain_id: &str,
        weight: f64,
    ) -> CoreResult<()> {
        self.push_undo();
        let board = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == board_id)
            .ok_or_else(|| CoreError::BoardNotFound(board_id.to_string()))?;
        let ann = board
            .annotations
            .iter_mut()
            .find(|a| a.id() == node_id)
            .ok_or_else(|| CoreError::AnnotationNotFound(node_id.to_string()))?;

        let assignments = ann.domains_mut();
        match assignments.iter_mut().find(|d| d.domain_id == domain_id) {
            Some(existing) => existing.weight = weight,
            None => assignments
                .push(DomainAssignment { domain_id: domain_id.to_string(), weight }),
        }
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_assign_domain_to_node`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_assign_domain_to_node`.")]
    pub fn assign_domain_to_node(
        &mut self,
        board_id: &str,
        node_id: &str,
        domain_id: &str,
        weight: f64,
    ) {
        drop(self.try_assign_domain_to_node(board_id, node_id, domain_id, weight));
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
