//! **Ce que le document a sur le disque**, vu depuis l'application.
//!
//! Trois moments, et c'est tout :
//!
//! * **à chaque image**, [`GlucoseApp::consigner`] confie au scribe ce que le journal a
//!   appliqué au document (JRN-5). Un document qui a un nom est alors enregistré — il n'y a
//!   plus d'état « modifié » à rattraper. Un document sans nom écrit dans son **brouillon**,
//!   qui naît au premier geste, dans le dossier de l'application ;
//! * **en changeant de document** ou en fermant, [`GlucoseApp::fermer_le_document`] écrit la
//!   vue et attend que tout soit sur le disque ;
//! * **au lancement**, [`GlucoseApp::retrouver_le_travail`] rouvre le dernier document, ou
//!   ce qu'un plantage a interrompu ([`super::reprise`]).
//!
//! Rien ici ne s'exécute dans le constructeur de l'application : les centaines d'épreuves
//! qui en créent une n'écrivent donc jamais de brouillon chez l'utilisateur.

use super::ecriture::{nouveau_brouillon, Ecriture};
use super::now_millis;
use super::objets::{Objets, Source};
use crate::app::GlucoseApp;
use glucose_core::persist::histoire;
use glucose_core::types::Project;
use std::path::PathBuf;
use std::sync::Arc;

/// L'état « disque » du document ouvert.
pub struct Disque {
    /// Où sont les octets de chaque image — partagé avec l'atelier et le scribe.
    pub objets: Arc<Objets>,
    /// L'histoire en train de s'écrire, dès qu'il existe un fichier.
    pub ecriture: Option<Ecriture>,
    /// L'état de départ d'un document qui n'a encore aucun fichier : la base qu'écrira son
    /// brouillon, ou son premier enregistrement.
    pub depart: Option<Project>,
    /// Des octets d'images à sceller dès qu'un fichier existera — les images en base64 d'un
    /// vieux document Tauri.
    pub a_sceller: Vec<(String, Arc<Vec<u8>>)>,
    /// Le passé qu'on regarde dans la Time Machine, et ce qu'on a mis de côté pour revenir.
    pub voyage: Option<crate::interactions::temps::Voyage>,
    /// Où naissent les brouillons, où se gardent les textes en cours de frappe et le souvenir
    /// du dernier document.
    ///
    /// **Seul le vrai lancement y met le dossier de l'utilisateur**, par
    /// [`GlucoseApp::habiter`] ; toute autre application — les centaines que créent les
    /// épreuves, les bancs, les exemples — habite un dossier temporaire qui n'appartient à
    /// personne.
    pub brouillons: PathBuf,
}

impl Disque {
    pub fn nouveau(depart: Project) -> Self {
        Self {
            objets: Objets::nouveau(),
            ecriture: None,
            depart: Some(depart),
            a_sceller: Vec::new(),
            voyage: None,
            brouillons: crate::app::accueil::dossier_hors_lancement().join("brouillons"),
        }
    }
}

impl GlucoseApp {
    /// **Écrit ce que le journal a appliqué au document** depuis la dernière image.
    pub fn consigner(&mut self) {
        let transactions = self.store.journal.prendre_les_ecrits();
        if self.disque.voyage.is_some() {
            // On regarde le passé : ce qui s'y fait n'est pas de l'histoire (HISTOIRE-3).
            return;
        }
        let a_ecrire = !transactions.is_empty()
            || self
                .disque
                .ecriture
                .as_ref()
                .is_some_and(Ecriture::a_du_travail);
        if a_ecrire {
            self.ecrire_les_gestes(transactions);
        }
        // Après les gestes : une saisie qui vient de se valider s'écrit avant que son texte
        // en cours ne s'oublie.
        self.garder_la_saisie();
        self.dire_l_erreur_d_ecriture();
    }

    fn ecrire_les_gestes(&mut self, transactions: Vec<glucose_core::store::journal::Transaction>) {
        let instant = now_millis();
        if let Err(e) = self.s_assurer_d_un_fichier() {
            self.dire_l_echec(&e);
            return;
        }
        let Some(ecriture) = self.disque.ecriture.as_mut() else {
            return;
        };
        let nombre = transactions.len();
        ecriture.consigner(
            transactions,
            &self.store.project,
            &self.disque.objets,
            instant,
        );
        // La réglette de la Time Machine s'allonge sans relire le fichier.
        self.dock_manager.temps.noter(nombre, instant);
        if !ecriture.brouillon {
            // Un document qui a un nom vient d'être enregistré : c'est l'enregistrement
            // continu de Glucose Tauri, au coût du geste (INVARIANT SAVE-2 inchangé).
            self.saved_version = self.store.version;
        }
    }

    /// Donne un fichier au document qui n'en a pas : un brouillon, dans le dossier de
    /// l'application — le seul dossier qui se crée, parce qu'il appartient à l'application.
    pub(crate) fn s_assurer_d_un_fichier(&mut self) -> Result<(), String> {
        if self.disque.ecriture.is_some() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.disque.brouillons)
            .map_err(|e| format!("dossier des brouillons, {e}"))?;
        let instant = now_millis();
        let brouillon = nouveau_brouillon(&self.disque.brouillons, instant);
        self.naitre(brouillon, true, instant)
            .map_err(|e| format!("brouillon, {e}"))
    }

    /// Crée le fichier d'un document qui n'en avait pas : sa base est l'état de départ, et
    /// ses images s'y scellent aussitôt.
    pub(crate) fn naitre(
        &mut self,
        chemin: PathBuf,
        brouillon: bool,
        instant: i64,
    ) -> Result<(), String> {
        let depart = self
            .disque
            .depart
            .take()
            .unwrap_or_else(|| self.store.project.clone());
        let objets = &self.disque.objets;
        match Ecriture::nouvelle(
            chemin,
            &depart,
            brouillon,
            objets,
            instant,
            &self.disque.brouillons,
        ) {
            Ok(e) => {
                self.disque.ecriture = Some(e);
                self.sceller_ce_qui_attend();
                Ok(())
            }
            Err(err) => {
                self.disque.depart = Some(depart);
                Err(err)
            }
        }
    }

    /// Scelle les images qui vivent encore hors du document.
    pub(crate) fn sceller_ce_qui_attend(&mut self) {
        let Some(e) = self.disque.ecriture.as_mut() else {
            return;
        };
        for (cle, octets) in self.disque.a_sceller.drain(..) {
            e.sceller_des_octets(&cle, octets.as_ref().clone());
        }
        e.sceller_tout(&self.store.project, &self.disque.objets);
    }

    fn dire_l_erreur_d_ecriture(&mut self) {
        if let Some(e) = self
            .disque
            .ecriture
            .as_ref()
            .and_then(Ecriture::prendre_l_erreur)
        {
            self.dire_l_echec(&e);
        }
    }

    /// **La seule voix du disque quand il refuse.** Un brouillon qui ne naît pas, une écriture
    /// refusée, une fermeture qui n'aboutit pas : c'est la même nouvelle pour l'utilisateur,
    /// et la même chose à faire.
    pub(super) fn dire_l_echec(&mut self, cause: &str) {
        self.ui.show_toast(format!(
            "L'enregistrement a échoué : {cause} — le document est intact en mémoire, \
             Ctrl+Maj+S pour l'enregistrer ailleurs"
        ));
    }

    /// Écrit une dernière fois — ce qui reste, et la vue — puis ferme le fichier.
    ///
    /// Un brouillon qui a quelque chose reste sur le disque : c'est du travail sans nom, que
    /// le prochain lancement rouvrira. Rend `false` si le disque a refusé : l'appelant qui
    /// fermait la fenêtre la garde ouverte (SAVE-3).
    pub fn fermer_le_document(&mut self) -> bool {
        self.terminer_les_gestes_en_cours();
        self.consigner();
        let Some(e) = self.disque.ecriture.take() else {
            return true;
        };
        e.vue(&self.store.project);
        match e.synchroniser() {
            Ok(()) => true,
            Err(err) => {
                self.dire_l_echec(&err);
                self.disque.ecriture = Some(e);
                false
            }
        }
    }

    /// **Ce qui est en cours se termine avant qu'on quitte un document** : l'aperçu du passé
    /// revient au présent, la carte en édition se valide — comme un clic ailleurs. Sans cela,
    /// fermer la fenêtre pendant une frappe perdait le texte (le document passait pour
    /// propre), et ouvrir un document pendant un aperçu laissait l'écriture suspendue.
    pub(crate) fn terminer_les_gestes_en_cours(&mut self) {
        self.revenir_au_present();
        self.commit_editing();
    }

    /// Oublie le document sans nom qu'on ferme sans l'enregistrer : son brouillon s'efface.
    pub fn abandonner_le_brouillon(&mut self) {
        if let Some(e) = self.disque.ecriture.take() {
            if e.brouillon {
                e.abandonner();
            }
        }
    }

    /// Fait d'un fichier ouvert le document courant : ses images, son état, son écriture —
    /// et le texte qu'un arrêt y avait laissé en cours de frappe.
    pub(crate) fn adopter_un_ouvert(
        &mut self,
        ouvert: &histoire::Ouvert,
        chemin: PathBuf,
    ) -> Adoption {
        // Le document qu'on quitte s'écrit une dernière fois. S'il n'a pas pu, son écriture
        // s'abandonne ici — le toast l'a dit — plutôt que de retenir le nouveau document.
        if !self.fermer_le_document() {
            self.disque.ecriture = None;
        }
        let objets = &self.disque.objets;
        objets.vider();
        objets.porter(Some(chemin.clone()));
        for (cle, empreinte) in &ouvert.liens {
            if let Some(t) = ouvert.objets.get(empreinte) {
                objets.poser(
                    cle,
                    Source::Tranche {
                        empreinte: *empreinte,
                        offset: t.offset,
                        longueur: t.longueur,
                    },
                );
            }
        }
        self.editing_session = None;
        self.historique_du_texte.oublier();
        self.selection_box = None;
        self.dock_manager.domains.reset();
        let repares = self.store.load_project(ouvert.projet.clone());
        // Ce que le chargement a pu mettre dans la file de sortie n'appartient à personne :
        // l'état qu'on vient de lire est déjà sur le disque.
        self.store.journal.prendre_les_ecrits();
        self.fit_all_text_cards();
        self.store.bump_version();
        self.disque.depart = None;
        self.disque.a_sceller.clear();
        let saisies = self.disque.brouillons.clone();
        match Ecriture::reprendre(ouvert, chemin, &self.disque.objets, &saisies) {
            Ok((mut e, saisie)) => {
                if ouvert.geste_en_echec.is_some() {
                    // Un geste qui ne se rejoue pas : un instantané de l'état lu fera repartir
                    // l'histoire d'ici, et le geste fautif ne sera plus jamais rejoué.
                    e.instantane(&self.store.project);
                }
                self.disque.ecriture = Some(e);
                self.sceller_ce_qui_attend();
                let texte_rendu = saisie.is_some_and(|s| self.rendre_la_saisie(s));
                self.suivre_le_document();
                Adoption {
                    repares,
                    refus: None,
                    texte_rendu,
                }
            }
            Err(err) => {
                // Un fichier qu'on ne peut pas écrire — lecture seule, clé retirée, document
                // tenu par une autre fenêtre — s'ouvre quand même : ce qu'on y change part
                // dans un brouillon, qui commence à l'état lu. Le fichier, lui, n'est pas
                // touché.
                self.disque.depart = Some(self.store.project.clone());
                self.suivre_le_document();
                Adoption {
                    repares,
                    refus: Some(err),
                    texte_rendu: false,
                }
            }
        }
    }
}

/// Ce que l'adoption d'un fichier ouvert a dû faire, pour que l'ouverture le dise.
pub(crate) struct Adoption {
    /// Nœuds réparés au chargement.
    pub repares: usize,
    /// Pourquoi le fichier ne s'écrit pas sur place, s'il ne s'écrit pas.
    pub refus: Option<String>,
    /// Un texte en cours de frappe, laissé par un arrêt, est revenu dans le document.
    pub texte_rendu: bool,
}

#[cfg(test)]
pub(crate) mod tests;
