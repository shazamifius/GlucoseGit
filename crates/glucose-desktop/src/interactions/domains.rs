//! Le panneau DOMAINES branché sur le document.
//!
//! Le panneau produit des [`DomainIntent`] et ne mute rien (standard § 1.7) ; ce module est le
//! seul endroit où une intention devient une commande du noyau. Chaque échec remonte par la
//! barre de toasts, jamais en silence (§ 6.4).
//!
//! Les messages n'y portent **aucun émoji**, contrairement au reste de l'application : la
//! police embarquée (`assets/font.ttf`) couvre 122 points de code, tous latins de base. Un
//! `⚠️` n'y a pas de glyphe, donc `fontdue` rastérise le `.notdef` — un pictogramme invisible
//! n'avertit de rien. Le texte des [`CoreError`](glucose_core::error::CoreError) commence déjà
//! par ce qui a échoué et finit par ce qu'il faut faire (§ 6.5) : il se suffit.
//!
//! # DOM-APP-1 — un geste de l'utilisateur, une entrée d'annulation
//!
//! Assigner un domaine à une sélection de quarante nœuds, c'est **un** geste. Sans précaution,
//! ce serait quarante entrées d'undo et quarante `Ctrl+Z` pour le défaire (§ 3.6). La
//! transaction live du store — `begin_live_edit` / `end_live_edit` — existe exactement pour
//! ça : le premier instantané est pris, les suivants sont absorbés.
//!
//! Corollaire : on ne l'ouvre **qu'après** avoir vérifié qu'il y a quelque chose à faire.
//! Ouvrir une transaction pour n'y rien écrire laisserait une entrée d'annulation pour un
//! geste sans effet, ce que DOM-3 interdit côté noyau et qui ne vaut pas mieux ici.

use crate::app::GlucoseApp;
use crate::dock::domains::{next_color, next_sigil, DomainIntent, DomainRename};
use crate::interactions::text_entry::TextEntry;
use glucose_core::store::DomainPatch;
use glucose_core::types::Domain;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

impl GlucoseApp {
    /// Applique une intention du panneau DOMAINES.
    pub fn apply_domain_intent(&mut self, intent: DomainIntent) {
        match intent {
            DomainIntent::Create => self.create_domain(),
            DomainIntent::StartRename(id) => self.start_domain_rename(&id),
            DomainIntent::CycleColor(id) => self.cycle_domain_color(&id),
            DomainIntent::CycleSigil(id) => self.cycle_domain_sigil(&id),
            DomainIntent::AskDelete(id) => {
                self.dock_manager.domains.rename = None;
                self.dock_manager.domains.pending_delete = Some(id);
            }
            DomainIntent::CancelDelete => self.dock_manager.domains.pending_delete = None,
            DomainIntent::ConfirmDelete(id) => self.delete_domain(&id),
            DomainIntent::Assign { domain_id, weight } => self.assign_domain(&domain_id, weight),
            DomainIntent::Unassign(id) => self.unassign_domain(&id),
        }
        self.mark_dirty();
    }

    /// Crée un domaine et ouvre aussitôt son nom en saisie.
    ///
    /// L'identifiant vient du générateur monotone du store (§ 2.5) — jamais d'un `format!` sur
    /// la longueur d'un `Vec`, qui produisait deux fois le même après une suppression.
    fn create_domain(&mut self) {
        let rank = self.store.project.domains.len();
        let (color, icon) = crate::dock::domains::fresh_look(rank);
        let domain = Domain {
            id: self.store.generate_id("domain"),
            // Un libellé, pas un identifiant : il est là pour être remplacé tout de suite, et
            // c'est pourquoi la saisie s'ouvre dans la foulée.
            name: format!("Domaine {}", rank + 1),
            color: color.to_string(),
            icon: icon.to_string(),
            created_at: crate::persist::now_millis(),
        };
        let id = domain.id.clone();
        match self.store.try_add_domain(domain) {
            Ok(()) => {
                self.dock_manager.domains.pending_delete = None;
                self.start_domain_rename(&id);
                self.ui.show_toast("Domaine créé — tape son nom, Entrée pour valider");
            }
            Err(err) => self.ui.show_toast(err.to_string()),
        }
    }

    fn start_domain_rename(&mut self, id: &str) {
        let Some(domain) = self.store.domain(id) else {
            self.ui.show_toast("Ce domaine n'existe plus — le panneau vient d'être rafraîchi");
            self.dock_manager.domains.reset();
            return;
        };
        self.dock_manager.domains.pending_delete = None;
        self.dock_manager.domains.rename = Some(DomainRename {
            domain_id: id.to_string(),
            entry: TextEntry::new(domain.name.clone()),
        });
    }

    fn cycle_domain_color(&mut self, id: &str) {
        let Some(domain) = self.store.domain(id) else {
            return;
        };
        let patch = DomainPatch::new().with_color(next_color(&domain.color));
        self.patch_domain(id, patch);
    }

    fn cycle_domain_sigil(&mut self, id: &str) {
        let Some(domain) = self.store.domain(id) else {
            return;
        };
        let patch = DomainPatch::new().with_icon(next_sigil(&domain.icon));
        self.patch_domain(id, patch);
    }

    /// Envoie un patch au noyau et dit pourquoi si le noyau refuse.
    fn patch_domain(&mut self, id: &str, patch: DomainPatch) {
        if let Err(err) = self.store.try_update_domain(id, patch) {
            self.ui.show_toast(err.to_string());
        }
    }

    fn delete_domain(&mut self, id: &str) {
        let label = self.store.domain(id).map_or_else(|| id.to_string(), |d| d.name.clone());
        match self.store.try_remove_domain(id) {
            Ok(0) => self.ui.show_toast(format!("Domaine « {label} » supprimé")),
            Ok(detached) => self.ui.show_toast(format!(
                "Domaine « {label} » supprimé — retiré de {detached} nœud(s)"
            )),
            Err(err) => self.ui.show_toast(err.to_string()),
        }
        self.dock_manager.domains.pending_delete = None;
        if self.dock_manager.domains.rename.as_ref().is_some_and(|r| r.domain_id == id) {
            self.dock_manager.domains.rename = None;
        }
    }

    /// Les nœuds sélectionnés du tableau actif — annotations puis images.
    fn selected_nodes(&self) -> Vec<String> {
        let mut nodes = self.store.selected_annotation_ids.clone();
        nodes.extend(self.store.selected_image_ids.iter().cloned());
        nodes
    }

    fn assign_domain(&mut self, domain_id: &str, weight: f64) {
        let nodes = self.selected_nodes();
        if nodes.is_empty() || self.store.domain(domain_id).is_none() {
            return;
        }
        let board = self.store.project.active_board_id.clone();

        // DOM-APP-1 — une transaction, donc une entrée d'annulation pour tout le geste.
        self.store.begin_live_edit();
        let mut done = 0usize;
        let mut failure = None;
        for node in &nodes {
            match self.store.try_assign_domain_to_node(&board, node, domain_id, weight) {
                Ok(()) => done += 1,
                Err(err) => failure = Some(err),
            }
        }
        self.store.end_live_edit();

        let percent = (weight * 100.0).round() as i32;
        match (done, failure) {
            (0, Some(err)) => self.ui.show_toast(err.to_string()),
            (0, None) => {}
            (n, _) => self.ui.show_toast(format!("Domaine assigné à {n} nœud(s) — {percent} %")),
        }
    }

    fn unassign_domain(&mut self, domain_id: &str) {
        let board = self.store.project.active_board_id.clone();
        let carriers: Vec<String> = self
            .selected_nodes()
            .into_iter()
            .filter(|node| {
                self.store
                    .node_domains(&board, node)
                    .is_ok_and(|list| list.iter().any(|a| a.domain_id == domain_id))
            })
            .collect();
        if carriers.is_empty() {
            self.ui.show_toast("Aucun nœud sélectionné ne porte ce domaine");
            return;
        }

        self.store.begin_live_edit();
        let mut done = 0usize;
        for node in &carriers {
            if self.store.try_unassign_domain_from_node(&board, node, domain_id).is_ok() {
                done += 1;
            }
        }
        self.store.end_live_edit();
        self.ui.show_toast(format!("Domaine retiré de {done} nœud(s)"));
    }

    // ── Saisie du nom ───────────────────────────────────────────────────────

    /// Traite une touche pendant qu'un nom de domaine est en cours de frappe.
    ///
    /// Rend `true` si la touche a été consommée. Un accord `Ctrl` ou `Alt` valide d'abord la
    /// saisie puis laisse la touche descendre : sans cela, `Ctrl+S` serait impossible tant
    /// qu'un nom est ouvert, exactement le piège que `text_edit` documente déjà.
    pub fn handle_domain_rename_key(&mut self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        self.handle_domain_rename_input(&event.logical_key)
    }

    /// Le corps de [`GlucoseApp::handle_domain_rename_key`], sans le `KeyEvent` de winit.
    ///
    /// Séparé parce qu'un `KeyEvent` ne se construit pas hors de la boucle d'événements — son
    /// champ `platform_specific` est un type opaque propre à chaque plateforme. Tout ce qui
    /// décide, décide ici, et se teste donc sans fenêtre (§ 7.1).
    pub fn handle_domain_rename_input(&mut self, key: &Key) -> bool {
        if self.dock_manager.domains.rename.is_none() {
            return false;
        }
        if self.modifiers.control_key() || self.modifiers.alt_key() {
            self.commit_domain_rename();
            return false;
        }
        let consumed = self.edit_domain_name(key);
        if consumed {
            self.mark_dirty();
        }
        consumed
    }

    /// Applique une touche au tampon de saisie. Séparée pour que l'aiguillage ci-dessus reste
    /// une suite de décisions, sans corps de `match` (§ 1.7).
    fn edit_domain_name(&mut self, key: &Key) -> bool {
        let produces_text = !self.modifiers.control_key() && !self.modifiers.alt_key();
        let Some(rename) = &mut self.dock_manager.domains.rename else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Escape) => {
                self.dock_manager.domains.rename = None;
                true
            }
            Key::Named(NamedKey::Enter) => {
                self.commit_domain_rename();
                true
            }
            Key::Named(NamedKey::Backspace) => {
                rename.entry.backspace();
                true
            }
            Key::Named(NamedKey::Delete) => {
                rename.entry.delete();
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                rename.entry.move_left();
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                rename.entry.move_right();
                true
            }
            Key::Named(NamedKey::Home) => {
                rename.entry.home();
                true
            }
            Key::Named(NamedKey::End) => {
                rename.entry.end();
                true
            }
            Key::Character(text) if produces_text => {
                rename.entry.insert(text.as_str());
                true
            }
            _ => false,
        }
    }

    /// Valide le nom saisi. Un nom vide est refusé : un domaine sans nom ne se distingue plus
    /// de ses voisins dans le panneau et sa réglette ne se laisse plus identifier.
    pub fn commit_domain_rename(&mut self) {
        let Some(rename) = self.dock_manager.domains.rename.take() else {
            return;
        };
        let name = rename.entry.into_text().trim().to_string();
        if name.is_empty() {
            self.ui.show_toast("Un domaine a besoin d'un nom — renommage abandonné");
            return;
        }
        self.patch_domain(&rename.domain_id, DomainPatch::new().with_name(name));
    }
}

#[cfg(test)]
mod tests;
