//! **La Time Machine**, du côté de l'application (HISTOIRE-3) : regarder un état passé,
//! revenir au présent, restaurer, poser un jalon.
//!
//! # L'aperçu est un bac à sable
//!
//! Regarder le passé met de côté l'état présent **et** son journal d'annulation, puis montre
//! l'état relu du fichier. Tant qu'on regarde, rien ne s'écrit (JRN-5 suspendu : la file de
//! sortie est vidée sans être écrite). Revenir au présent reprend exactement ce qui avait
//! été mis de côté — la caméra seule suit l'utilisateur, qui regarde d'où il veut.
//!
//! # Restaurer est un geste
//!
//! « Restaurer cet état » ne recalcule aucune différence : les gestes qui ont suivi, chacun
//! retourné, forment **un** geste (`histoire::retour_au_geste`). Il s'annule par `Ctrl+Z`, il
//! s'écrit dans l'histoire, et ce qu'il défait y reste : rien ne se perd, pas même ce qu'on
//! vient de quitter.

use crate::app::GlucoseApp;
use crate::dock::temps::{JalonVu, TempsIntent};
use crate::dock::TabId;
use crate::interactions::text_entry::TextEntry;
use crate::persist::now_millis;
use glucose_core::persist::histoire::{self, Genre, Ouvert, Vue};
use glucose_core::store::journal::Journal;
use glucose_core::store::UNDO_DEPTH;
use glucose_core::types::Project;
use winit::keyboard::{Key, NamedKey};

/// Ce qu'un aperçu du passé a mis de côté pour revenir.
pub struct Voyage {
    /// L'index de l'histoire, lu à l'entrée dans le passé.
    pub ouvert: Ouvert,
    present: Project,
    journal: Journal,
}

impl GlucoseApp {
    /// `Ctrl+H` : ouvre ou ferme la Time Machine. La fermer revient au présent.
    pub fn basculer_la_machine(&mut self) {
        if self.dock_manager.is_open(TabId::Temps) {
            self.revenir_au_present();
            self.dock_manager.temps.nom = None;
            self.dock_manager.dismiss_tab(TabId::Temps);
        } else {
            self.lire_l_histoire();
            self.dock_manager.toggle_tab(TabId::Temps);
        }
        self.mark_dirty();
    }

    /// Ce que le panneau demande.
    pub fn agir_dans_le_temps(&mut self, intent: TempsIntent) {
        match intent {
            TempsIntent::Reglette(k) => {
                self.dock_manager.temps.glisse = true;
                self.aller_au_point(k);
            }
            TempsIntent::Voir(k) => self.voir_le_geste(k),
            TempsIntent::Maintenant => self.revenir_au_present(),
            TempsIntent::Restaurer => {
                if let Some(k) = self.dock_manager.temps.regarde {
                    self.restaurer_le_geste(k);
                }
            }
            TempsIntent::RestaurerLeJalon(k) => self.restaurer_le_geste(k),
            TempsIntent::CommencerUnJalon => {
                self.dock_manager.temps.nom = Some(TextEntry::default());
            }
        }
        self.mark_dirty();
    }

    /// Un autre document devient le document courant : la Time Machine, si elle est ouverte,
    /// montre son histoire à lui.
    pub(crate) fn suivre_le_document(&mut self) {
        if self.dock_manager.is_open(TabId::Temps) {
            self.lire_l_histoire();
        }
    }

    /// Le point `k` de la réglette : un état du passé, ou le présent quand c'est le dernier.
    /// Rien ne se relit si c'est déjà celui qu'on regarde.
    fn aller_au_point(&mut self, k: usize) {
        if k >= self.dock_manager.temps.gestes.len() {
            self.revenir_au_present();
        } else if self.dock_manager.temps.regarde != Some(k) {
            self.voir_le_geste(k);
        }
    }

    /// **La réglette tenue suit le curseur** : le passé défile sous la main, sans relâcher.
    /// Hors de la réglette, c'est son bout le plus proche qui compte.
    pub fn glisser_la_reglette(&mut self, x: f32) {
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(largeur, hauteur);
        let Some(k) = crate::dock::point_de_la_reglette(&self.dock_manager, ecran, x) else {
            return;
        };
        self.aller_au_point(k);
        self.mark_dirty();
    }

    /// Relit l'histoire du fichier : ce que la réglette et les jalons montrent. Rien, tant
    /// que le document n'a pas de fichier — il n'a pas encore d'histoire.
    pub fn lire_l_histoire(&mut self) -> Option<Ouvert> {
        self.consigner();
        let t = &mut self.dock_manager.temps;
        t.gestes.clear();
        t.jalons.clear();
        let ecriture = self.disque.ecriture.as_ref()?;
        ecriture.synchroniser().ok()?;
        let fichier = std::fs::File::open(&ecriture.chemin).ok()?;
        let o = histoire::ouvrir(&mut std::io::BufReader::new(fichier)).ok()?;
        let t = &mut self.dock_manager.temps;
        t.gestes = o.gestes.iter().map(|g| g.instant).collect();
        t.jalons = o
            .jalons
            .iter()
            .map(|(apres, j)| JalonVu {
                apres: *apres,
                libelle: j.libelle.clone(),
                nomme: j.genre == Genre::Nomme,
                instant: j.instant,
                date: crate::plateforme::heure::heure_locale(j.instant),
            })
            .collect();
        t.maintenant = now_millis();
        Some(o)
    }

    /// Montre l'état du document après ses `k` premiers gestes.
    pub fn voir_le_geste(&mut self, k: usize) {
        if self.editing_session.is_some() {
            self.commit_editing();
        }
        if self.disque.voyage.is_none() {
            let Some(ouvert) = self.lire_l_histoire() else {
                return;
            };
            self.disque.voyage = Some(Voyage {
                ouvert,
                present: self.store.project.clone(),
                journal: self.store.journal.clone(),
            });
        }
        let Some(passe) = self.etat_au(k) else {
            return;
        };
        let mut passe = passe;
        // On regarde le passé d'où l'on est : la caméra ne saute pas.
        Vue::de(&self.store.project).poser(&mut passe);
        self.store.load_project(passe);
        // Défaire, dans le passé, ne doit pas défaire le présent.
        self.store.journal = Journal::new(UNDO_DEPTH);
        self.store.bump_version();
        self.dock_manager.temps.regarde = Some(k);
    }

    /// L'état après `k` gestes, relu du fichier. `None` si `k` est le présent.
    fn etat_au(&mut self, k: usize) -> Option<Project> {
        let voyage = self.disque.voyage.as_ref()?;
        if k >= voyage.ouvert.gestes.len() {
            self.revenir_au_present();
            return None;
        }
        let chemin = self.disque.ecriture.as_ref()?.chemin.clone();
        let fichier = std::fs::File::open(chemin).ok()?;
        histoire::etat_au_geste(&mut std::io::BufReader::new(fichier), &voyage.ouvert, k).ok()
    }

    /// Revient au présent : l'état et le journal mis de côté, la caméra d'où l'on regardait.
    pub fn revenir_au_present(&mut self) {
        let Some(voyage) = self.disque.voyage.take() else {
            return;
        };
        let mut present = voyage.present;
        Vue::de(&self.store.project).poser(&mut present);
        self.store.load_project(present);
        self.store.journal = voyage.journal;
        // Ce qui s'est fait dans le passé n'est pas de l'histoire.
        self.store.journal.prendre_les_ecrits();
        self.store.bump_version();
        self.dock_manager.temps.regarde = None;
        self.mark_dirty();
    }

    /// Fait de l'état après `k` gestes l'état présent — par un geste annulable.
    pub fn restaurer_le_geste(&mut self, k: usize) {
        self.revenir_au_present();
        let message = match self.retour_au(k) {
            Some(retour) => {
                if self.store.appliquer_comme_un_geste(retour) {
                    format!("État du geste {k} restauré — Ctrl+Z pour revenir à celui d'avant")
                } else {
                    "Cet état ne se restaure pas : un geste de l'histoire ne se défait pas — le \
                     document est inchangé"
                        .to_string()
                }
            }
            None => "L'histoire du document ne se relit pas — le document est inchangé".to_string(),
        };
        self.ui.show_toast(message);
        self.lire_l_histoire();
    }

    fn retour_au(&mut self, k: usize) -> Option<glucose_core::store::journal::Transaction> {
        let ouvert = self.lire_l_histoire()?;
        let chemin = self.disque.ecriture.as_ref()?.chemin.clone();
        let fichier = std::fs::File::open(chemin).ok()?;
        histoire::retour_au_geste(&mut std::io::BufReader::new(fichier), &ouvert, k).ok()
    }

    /// Une touche pendant qu'un nom de jalon se tape. Rend `true` si elle est consommée.
    pub fn frapper_le_nom_du_jalon(&mut self, key: &Key) -> bool {
        let Some(entree) = self.dock_manager.temps.nom.as_mut() else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Escape) => self.dock_manager.temps.nom = None,
            Key::Named(NamedKey::Enter) => self.poser_le_jalon(),
            _ if self.modifiers.control_key() || self.modifiers.alt_key() => {
                // Un raccourci pendant la frappe : la saisie s'abandonne, la touche descend.
                self.dock_manager.temps.nom = None;
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

    /// Pose le jalon nommé, au présent. Un document sans fichier en reçoit un : un jalon
    /// est un point de l'histoire, et l'histoire vit dans le fichier.
    fn poser_le_jalon(&mut self) {
        let Some(entree) = self.dock_manager.temps.nom.take() else {
            return;
        };
        let nom = entree.into_text().trim().to_string();
        if nom.is_empty() {
            return;
        }
        self.revenir_au_present();
        let message = match self.s_assurer_d_un_fichier() {
            Err(e) => format!("Le jalon n'a pas pu se poser : {e}"),
            Ok(()) => match self
                .disque
                .ecriture
                .as_mut()
                .map(|e| e.jalon(&self.store.project, Genre::Nomme, &nom, now_millis()))
            {
                Some(Ok(())) => format!("Jalon « {nom} » posé"),
                Some(Err(e)) => format!("Le jalon n'a pas pu se poser : {e}"),
                None => "Le jalon n'a pas pu se poser : aucun fichier".to_string(),
            },
        };
        self.ui.show_toast(message);
        self.lire_l_histoire();
    }

    /// Les touches de la Time Machine quand elle montre le passé : `Échap` revient au présent,
    /// `←` et `→` avancent geste par geste. Rend `true` si la touche est consommée.
    pub fn touche_du_temps(&mut self, key: &Key) -> bool {
        let Some(k) = self.dock_manager.temps.regarde else {
            return false;
        };
        let n = self.dock_manager.temps.gestes.len();
        match key {
            Key::Named(NamedKey::Escape) => self.revenir_au_present(),
            Key::Named(NamedKey::ArrowLeft) => self.voir_le_geste(k.saturating_sub(1)),
            Key::Named(NamedKey::ArrowRight) if k + 1 >= n => self.revenir_au_present(),
            Key::Named(NamedKey::ArrowRight) => self.voir_le_geste(k + 1),
            _ => return false,
        }
        self.mark_dirty();
        true
    }
}
