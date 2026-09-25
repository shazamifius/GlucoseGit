//! **Ce que la main fait aux onglets** : les choisir, les renommer, les ranger, les supprimer
//! (BOARDS-1).
//!
//! La géométrie vient de [`crate::ui::onglets`] ; ce module dit ce que chaque geste fait au
//! document. Renommer, ranger et supprimer sont chacun **un** geste du journal : `Ctrl+Z` les
//! défait, la Time Machine les garde.
//!
//! # Pourquoi supprimer ne demande pas de confirmation
//!
//! Glucose Tauri demandait « Supprimer ? Cette action est annulable (Ctrl+Z) » — la question
//! disait elle-même qu'elle n'avait pas lieu d'être. La suppression se défait, et la Time
//! Machine la garde : un message le dit, comme chez Figma ou tldraw, au lieu d'interrompre.

use crate::app::GlucoseApp;
use crate::interactions::text_entry::TextEntry;
use crate::ui::onglets::{cible, CibleOnglet, OngletTenu};
use winit::keyboard::{Key, NamedKey};

impl GlucoseApp {
    /// **Un clic sur un onglet** : le choisir, et le prendre pour le ranger s'il glisse. Le
    /// second clic d'un double-clic l'ouvre au renommage — comme dans Glucose Tauri.
    pub(crate) fn cliquer_un_onglet(&mut self, id: String) {
        let cle = format!("onglet:{id}");
        let rang = self.click_count_at(&cle);
        self.remember_click(cle, rang);
        if rang >= 2 {
            self.commencer_le_renommage(&id);
            return;
        }
        self.store.set_active_board_id(&id);
        self.ui.onglets.tenu = Some(OngletTenu {
            id,
            depuis: (self.mouse_pos.0 as i64, self.mouse_pos.1 as i64),
            glisse: false,
        });
    }

    /// Le clic tombe-t-il dans l'onglet qu'on renomme ?
    pub(crate) fn clic_dans_le_champ_de_l_onglet(&self) -> bool {
        let Some((id, _)) = &self.ui.onglets.renomme else {
            return false;
        };
        let (x, y) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        let onglets = crate::ui::layout_tabs(&self.store, &self.ui, &self.renderer.typography);
        matches!(cible(&onglets, x, y), Some(CibleOnglet::Onglet(c)) if c == *id)
    }

    /// Ouvre cet onglet au renommage, le curseur au bout de son nom.
    pub(crate) fn commencer_le_renommage(&mut self, id: &str) {
        let Some(nom) = self
            .store
            .onglets()
            .find(|(b, _, _)| *b == id)
            .map(|(_, n, _)| n.to_string())
        else {
            return;
        };
        self.ui.onglets.tenu = None;
        self.ui.onglets.renomme = Some((id.to_string(), TextEntry::new(nom)));
        self.mark_dirty();
    }

    /// Une touche pendant qu'un onglet se renomme. Rend `true` si elle est consommée.
    ///
    /// `Entrée` valide, `Échap` renonce ; un raccourci valide ce qui est tapé et descend.
    pub fn frapper_le_nom_de_l_onglet(&mut self, key: &Key) -> bool {
        let Some((_, entree)) = self.ui.onglets.renomme.as_mut() else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Escape) => self.ui.onglets.renomme = None,
            Key::Named(NamedKey::Enter) => self.valider_le_renommage(),
            _ if self.modifiers.control_key() || self.modifiers.alt_key() => {
                self.valider_le_renommage();
                return false;
            }
            autre => {
                if !entree.editer(autre) {
                    return false;
                }
            }
        }
        self.mark_dirty();
        true
    }

    /// **Pose le nom tapé** : un geste, s'il a changé et n'est pas vide. C'est aussi ce que fait
    /// un clic ailleurs — le champ de Glucose Tauri validait en perdant le focus.
    pub(crate) fn valider_le_renommage(&mut self) {
        let Some((id, entree)) = self.ui.onglets.renomme.take() else {
            return;
        };
        let nom = entree.into_text().trim().to_string();
        let actuel = self
            .store
            .onglets()
            .find(|(b, _, _)| *b == id)
            .map(|(_, n, _)| n.to_string());
        if !nom.is_empty() && actuel.is_some_and(|a| a != nom) {
            self.store.rename_board(&id, nom);
        }
        self.mark_dirty();
    }

    /// **L'onglet tenu suit la souris** : au-delà d'un tremblement, il glisse. Rend `true`
    /// quand un onglet est tenu — le mouvement lui appartient.
    pub(crate) fn suivre_l_onglet_tenu(&mut self) -> bool {
        let Some(tenu) = self.ui.onglets.tenu.as_mut() else {
            return false;
        };
        let (x, y) = (tenu.depuis.0 as f64, tenu.depuis.1 as f64);
        let bouge = (self.mouse_pos.0 - x).hypot(self.mouse_pos.1 - y);
        if !tenu.glisse && bouge > super::pick::DOUBLE_CLICK_SLOP_PX {
            tenu.glisse = true;
        }
        self.mark_dirty();
        true
    }

    /// **L'onglet tenu se lâche** : s'il glissait, il se range là où on le lâche. Rend `true`
    /// si un onglet était tenu.
    pub(crate) fn lacher_l_onglet(&mut self) -> bool {
        let Some(tenu) = self.ui.onglets.tenu.take() else {
            return false;
        };
        if tenu.glisse {
            let (x, y) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
            let onglets = crate::ui::layout_tabs(&self.store, &self.ui, &self.renderer.typography);
            let sur = cible(&onglets, x, y);
            self.ranger_l_onglet_sur(&tenu.id, sur);
        }
        self.mark_dirty();
        true
    }

    /// Range l'onglet là où il est lâché : il prend la place de celui qu'il couvre — avant lui
    /// s'il vient de sa droite, après s'il vient de sa gauche —, et va au bout sur le « + ».
    fn ranger_l_onglet_sur(&mut self, id: &str, sur: Option<CibleOnglet>) {
        let cible = match sur {
            Some(CibleOnglet::Onglet(c) | CibleOnglet::Fermer(c)) if c != id => c,
            Some(CibleOnglet::Plus) => {
                drop(self.store.try_move_board(id, None));
                return;
            }
            _ => return,
        };
        let ordre: Vec<String> = self
            .store
            .onglets()
            .map(|(b, _, _)| b.to_string())
            .collect();
        let (Some(de), Some(vers)) = (
            ordre.iter().position(|b| b == id),
            ordre.iter().position(|b| *b == cible),
        ) else {
            return;
        };
        let avant = if de > vers {
            Some(cible.as_str())
        } else {
            ordre.get(vers + 1).map(String::as_str)
        };
        drop(self.store.try_move_board(id, avant));
    }

    /// **Supprime cet onglet**, avec ses dossiers, et dit comment le rendre.
    pub(crate) fn fermer_l_onglet(&mut self, id: &str) {
        let nom = self
            .store
            .onglets()
            .find(|(b, _, _)| *b == id)
            .map(|(_, n, _)| n.to_string())
            .unwrap_or_default();
        let message = match self.store.try_remove_board(id) {
            Ok(()) => format!("« {nom} » supprimé — Ctrl+Z pour le rendre"),
            Err(e) => e.to_string(),
        };
        self.ui.show_toast(message);
        self.mark_dirty();
    }

    /// Un board de plus, nommé d'après le nombre d'onglets — comme Glucose Tauri.
    ///
    /// Sans message : l'onglet neuf paraît et s'allume, et un toast ne dirait rien que l'œil
    /// ne voie déjà.
    pub(crate) fn ajouter_un_onglet(&mut self) {
        let name = format!("Board {}", self.store.onglets().count() + 1);
        let id = self.store.add_board(name);
        self.store.set_active_board_id(id);
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
