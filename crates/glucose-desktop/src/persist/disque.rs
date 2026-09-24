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
//! * **au lancement**, [`GlucoseApp::retrouver_un_brouillon`] rouvre le travail qu'un
//!   plantage a laissé sans nom.
//!
//! Rien ici ne s'exécute dans le constructeur de l'application : les centaines d'épreuves
//! qui en créent une n'écrivent donc jamais de brouillon chez l'utilisateur.

use super::ecriture::{dossier_des_brouillons, nouveau_brouillon, Ecriture};
use super::now_millis;
use super::objets::{Objets, Source};
use crate::app::GlucoseApp;
use glucose_core::persist::histoire;
use glucose_core::types::Project;
use std::path::{Path, PathBuf};
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
    /// Où naissent les brouillons : le dossier de l'application. Un champ, et non une
    /// constante, pour que les épreuves en donnent un à elles — elles ne doivent jamais
    /// laisser un faux brouillon que le vrai lancement suivant rouvrirait.
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
            brouillons: dossier_des_brouillons(),
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
        let rien = transactions.is_empty()
            && self
                .disque
                .ecriture
                .as_ref()
                .is_none_or(|e| !e.a_du_travail());
        if rien {
            self.dire_l_erreur_d_ecriture();
            return;
        }
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
        self.dire_l_erreur_d_ecriture();
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
        match Ecriture::nouvelle(chemin, &depart, brouillon, &self.disque.objets, instant) {
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
    fn dire_l_echec(&mut self, cause: &str) {
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

    /// Oublie le document sans nom qu'on ferme sans l'enregistrer : son brouillon s'efface.
    pub fn abandonner_le_brouillon(&mut self) {
        if let Some(e) = self.disque.ecriture.take() {
            if e.brouillon {
                e.abandonner();
            }
        }
    }

    /// Rouvre le travail qu'un plantage a laissé sans nom — le brouillon le plus récent
    /// qu'aucune autre fenêtre de Glucose ne tient ouvert.
    pub fn retrouver_un_brouillon(&mut self) {
        let Some(chemin) = brouillon_orphelin(&self.disque.brouillons) else {
            return;
        };
        let message = match self.ouvrir_un_brouillon(&chemin) {
            Ok(gestes) => format!(
                "Travail non enregistré retrouvé ({gestes} geste(s)) — Ctrl+S pour lui donner \
                 un nom"
            ),
            Err(e) => format!(
                "Un brouillon n'a pas pu se rouvrir : {e} — il reste dans {}",
                chemin.display()
            ),
        };
        self.ui.show_toast(message);
    }

    fn ouvrir_un_brouillon(&mut self, chemin: &Path) -> Result<usize, String> {
        let f = std::fs::File::open(chemin).map_err(|e| e.to_string())?;
        let ouvert =
            histoire::ouvrir(&mut std::io::BufReader::new(f)).map_err(|e| e.to_string())?;
        self.adopter_un_ouvert(&ouvert, chemin.to_path_buf());
        if let Some(e) = self.disque.ecriture.as_mut() {
            e.brouillon = true;
        }
        self.project_path = None;
        // Du travail sans nom : le document est « modifié » jusqu'à ce qu'il en ait un.
        self.saved_version = self.store.version.wrapping_sub(1);
        Ok(ouvert.gestes.len())
    }

    /// Fait d'un fichier ouvert le document courant : ses images, son état, son écriture.
    ///
    /// Rend les nœuds réparés, et la raison pour laquelle le fichier ne peut pas s'écrire sur
    /// place, s'il y en a une — l'ouverture le dira.
    pub(crate) fn adopter_un_ouvert(
        &mut self,
        ouvert: &histoire::Ouvert,
        chemin: PathBuf,
    ) -> (usize, Option<String>) {
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
        match Ecriture::reprendre(ouvert, chemin, &self.disque.objets) {
            Ok(mut e) => {
                if ouvert.geste_en_echec.is_some() {
                    // Un geste qui ne se rejoue pas : un instantané de l'état lu fera repartir
                    // l'histoire d'ici, et le geste fautif ne sera plus jamais rejoué.
                    e.instantane(&self.store.project);
                }
                self.disque.ecriture = Some(e);
                self.sceller_ce_qui_attend();
                (repares, None)
            }
            Err(err) => {
                // Un fichier qu'on ne peut pas écrire — lecture seule, clé retirée — s'ouvre
                // quand même : ce qu'on y change part dans un brouillon, qui commence à l'état
                // lu. Le fichier, lui, n'est pas touché.
                self.disque.depart = Some(self.store.project.clone());
                (repares, Some(err))
            }
        }
    }
}

/// Le brouillon le plus récent qu'aucun processus ne tient ouvert.
fn brouillon_orphelin(dossier: &Path) -> Option<PathBuf> {
    let mut brouillons: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dossier)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some(glucose_core::persist::FILE_EXTENSION)
        })
        .filter(|p| !tenu_ailleurs(p))
        .filter_map(|p| Some((std::fs::metadata(&p).ok()?.modified().ok()?, p)))
        .collect();
    brouillons.sort();
    brouillons.pop().map(|(_, p)| p)
}

/// Un autre Glucose écrit-il dans ce fichier ? Sous Windows, s'ouvrir sans aucun partage
/// échoue tant qu'un autre le tient.
#[cfg(windows)]
fn tenu_ailleurs(chemin: &Path) -> bool {
    use std::os::windows::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(chemin)
        .is_err()
}

#[cfg(not(windows))]
fn tenu_ailleurs(_chemin: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests;
