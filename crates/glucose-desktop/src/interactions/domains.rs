//! Le panneau DOMAINES branché sur le document.
//!
//! Le panneau produit des [`DomainIntent`] et ne mute rien (standard § 1.7) ; ce module est le
//! seul endroit où une intention devient une commande du noyau. Chaque échec remonte par la
//! barre de toasts, jamais en silence (§ 6.4).
//!
//! Les messages n'y portent **aucun émoji** : l'interface est monochrome et écrite en Inter
//! (`style.md`, FONT-1) — les emojis de Noto Emoji sont pour ce que l'utilisateur écrit
//! (EMOJI-1). Le texte des [`CoreError`](glucose_core::error::CoreError) commence déjà par ce
//! qui a échoué et finit par ce qu'il faut faire (§ 6.5) : il se suffit.
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
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

mod connus;

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
            DomainIntent::Adopter { lignee, weight } => self.adopter_le_domaine(&lignee, weight),
            DomainIntent::MemeSujet(id) => self.reunir_le_domaine(&id),
            DomainIntent::Preciser(id) => self.start_domain_rename(&id),
            DomainIntent::Ou(lignee) => self.ouvrir_le_ou(&lignee),
            DomainIntent::FermerOu => {
                self.dock_manager.domains.ou = None;
                self.dock_manager.domains.decalage = 0;
            }
            DomainIntent::Aller(rang) => self.aller_a_la_place(rang),
        }
        self.mark_dirty();
    }

    /// **Crée un domaine sous le premier nom libre**, « Domaine N », et ouvre aussitôt son nom en
    /// saisie. Libre dans tout ce qu'on connaît, il ne prend le nom de personne (DOM-4). Il naît tout
    /// de suite parce qu'au téléphone, le nom d'un domaine ne se tape pas encore (INVENTAIRE) : un
    /// domaine qui n'existerait qu'une fois nommé ne pourrait jamais y naître.
    fn create_domain(&mut self) {
        let vue = self.vue_du_catalogue();
        let nom = (1..)
            .map(|n| format!("Domaine {n}"))
            .find(|nom| glucose_core::recensement::homonymes(&vue, nom).is_empty())
            .unwrap_or_default();
        let rank = self.store.project.domains.len();
        let (color, icon) = crate::dock::domains::fresh_look(rank);
        let allure = (color.to_string(), icon.to_string());
        let created = self
            .store
            .creer_un_domaine(&nom, allure, crate::persist::now_millis());
        // Créé ou refusé : la même phrase, dite au même endroit (cliquet 2).
        let dit = match created {
            Ok(id) => {
                self.dock_manager.domains.pending_delete = None;
                self.start_domain_rename(&id);
                "Domaine créé — tape son nom, Entrée pour valider".to_string()
            }
            Err(err) => err.to_string(),
        };
        self.ui.show_toast(dit);
    }

    fn start_domain_rename(&mut self, id: &str) {
        let Some(domain) = self.store.domain(id) else {
            self.ui
                .show_toast("Ce domaine n'existe plus — le panneau vient d'être rafraîchi");
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
        let patch = patch.at(crate::persist::now_millis());
        if let Err(err) = self.store.try_update_domain(id, patch) {
            self.ui.show_toast(err.to_string());
        }
    }

    fn delete_domain(&mut self, id: &str) {
        let label = self
            .store
            .domain(id)
            .map_or_else(|| id.to_string(), |d| d.name.clone());
        let dit = match self.store.try_remove_domain(id) {
            Ok(0) => format!("Domaine « {label} » supprimé"),
            Ok(detached) => format!("Domaine « {label} » supprimé — retiré de {detached} nœud(s)"),
            Err(err) => err.to_string(),
        };
        self.ui.show_toast(dit);
        self.dock_manager.domains.pending_delete = None;
        if self
            .dock_manager
            .domains
            .rename
            .as_ref()
            .is_some_and(|r| r.domain_id == id)
        {
            self.dock_manager.domains.rename = None;
        }
    }

    /// Les nœuds sélectionnés du tableau actif — annotations puis images.
    fn selected_nodes(&self) -> Vec<String> {
        let mut nodes = self.store.selected_annotation_ids.to_vec();
        nodes.extend(self.store.selected_image_ids.iter().cloned());
        nodes
    }

    fn assign_domain(&mut self, domain_id: &str, weight: f64) {
        if self.store.domain(domain_id).is_some() {
            self.poser_sur_la_selection(|_| Ok(domain_id.to_string()), weight);
        }
    }

    /// **Pose un domaine sur la sélection, en un geste** (DOM-APP-1) : `domaine` le donne, dans ce
    /// même geste, pour qu'un domaine venu d'un autre fichier entre et se pose d'un seul `Ctrl+Z`.
    fn poser_sur_la_selection(
        &mut self,
        domaine: impl FnOnce(&mut glucose_core::store::Store) -> Result<String, String>,
        weight: f64,
    ) {
        let nodes = self.selected_nodes();
        if nodes.is_empty() {
            return;
        }
        let board = self.store.project.active_board_id.clone();

        // DOM-APP-1 — une transaction, donc une entrée d'annulation pour tout le geste.
        self.store.begin_live_edit();
        let mut done = 0usize;
        let mut failure = None;
        match domaine(&mut self.store) {
            Ok(domain_id) => {
                for node in &nodes {
                    match self
                        .store
                        .try_assign_domain_to_node(&board, node, &domain_id, weight)
                    {
                        Ok(()) => done += 1,
                        Err(err) => failure = Some(err.to_string()),
                    }
                }
            }
            Err(pourquoi) => failure = Some(pourquoi),
        }
        self.store.end_live_edit();

        let percent = (weight * 100.0).round() as i32;
        let dit = match (done, failure) {
            (0, Some(err)) => Some(err),
            (0, None) => None,
            (n, _) => Some(format!("Domaine assigné à {n} nœud(s) — {percent} %")),
        };
        if let Some(dit) = dit {
            self.ui.show_toast(dit);
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
        let mut done = 0usize;
        if !carriers.is_empty() {
            self.store.begin_live_edit();
            for node in &carriers {
                if self
                    .store
                    .try_unassign_domain_from_node(&board, node, domain_id)
                    .is_ok()
                {
                    done += 1;
                }
            }
            self.store.end_live_edit();
        }
        // Retiré, ou rien à retirer : la même phrase, dite au même endroit (cliquet 2).
        let dit = match done {
            0 => "Aucun nœud sélectionné ne porte ce domaine".to_string(),
            n => format!("Domaine retiré de {n} nœud(s)"),
        };
        self.ui.show_toast(dit);
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
            Key::Character(_) if !produces_text => false,
            autre => rename.entry.editer(autre),
        }
    }

    /// **Valide le nom saisi.** Un nom vide est refusé : un domaine sans nom ne se distingue plus de
    /// ses voisins. **Un nom pris aussi** (DOM-4, sa règle) : la saisie reste ouverte, et la question
    /// est posée — il n'y a plus qu'à préciser le mot, ou à choisir le domaine dans la liste.
    pub fn commit_domain_rename(&mut self) {
        let Some(rename) = &self.dock_manager.domains.rename else {
            return;
        };
        let name = rename.entry.text().trim().to_string();
        let domain_id = rename.domain_id.clone();
        // Un nom vide abandonne ; un nom pris laisse la saisie ouverte, pour préciser le mot. Les deux
        // refus se disent au même endroit (cliquet 2).
        let refus = if name.is_empty() {
            self.dock_manager.domains.rename = None;
            Some("Un domaine a besoin d'un nom — renommage abandonné".to_string())
        } else {
            self.question_du_nom(&name, Some(&domain_id))
        };
        if let Some(refus) = refus {
            self.ui.show_toast(refus);
            return;
        }
        self.dock_manager.domains.rename = None;
        self.patch_domain(&domain_id, DomainPatch::new().with_name(name));
    }
}

#[cfg(test)]
mod tests;
