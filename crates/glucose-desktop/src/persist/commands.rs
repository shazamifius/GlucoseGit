//! Les commandes de fichier vues depuis l'application : état « modifié », titre de fenêtre,
//! dialogues natifs, et les trois raccourcis qui les déclenchent.
//!
//! Le module voisin [`super`] fait l'I/O pure (encoder, écrire atomiquement, relire) sans rien
//! savoir de `GlucoseApp`. Ici on ne fait que l'orchestrer et remonter chaque échec par un
//! toast (standard § 6.4) : un enregistrement raté doit se voir.

use super::import::message_d_import;
use super::objets::Source;
use super::{
    human_size, now_millis, with_glucose_extension, SaveReport, APP_TITLE, DIRTY_MARK, UNTITLED,
};
use crate::app::GlucoseApp;
use crate::error::{DesktopError, DesktopResult};
use glucose_core::persist::histoire::{self, Genre};
use glucose_core::persist::FILE_EXTENSION;
use glucose_core::types::Project;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

// ── Dialogues ───────────────────────────────────────────────────────────────

fn pick_save_path(ancre: crate::dialogue::Ancre<'_>, suggested: &str) -> Option<PathBuf> {
    crate::dialogue::fichier(ancre)
        .filtre("Projet Glucose", &[FILE_EXTENSION])
        .nom(format!("{suggested}.{FILE_EXTENSION}"))
        .enregistrer()
        .map(with_glucose_extension)
}

pub(super) fn pick_open_path(ancre: crate::dialogue::Ancre<'_>) -> Option<PathBuf> {
    crate::dialogue::fichier(ancre)
        .filtre("Projet Glucose", &[FILE_EXTENSION])
        .choisir()
}

// ── Commandes de l'application ──────────────────────────────────────────────

impl GlucoseApp {
    /// Y a-t-il des changements non enregistrés ?
    ///
    /// `store.version` n'avance que sur une vraie mutation du document : `push_undo` la fait
    /// avancer, la navigation non (UNDO-1). Comparer à la version du dernier enregistrement
    /// est donc exactement la question posée, sans compteur supplémentaire à tenir à jour.
    pub fn is_dirty(&self) -> bool {
        self.store.version != self.saved_version
    }

    /// Nom du document affiché : celui du fichier s'il en a un, sinon celui du projet.
    pub fn document_label(&self) -> String {
        self.project_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if self.store.project.name.is_empty() {
                    UNTITLED.to_string()
                } else {
                    self.store.project.name.clone()
                }
            })
    }

    /// Titre complet de la fenêtre, marqueur de modification compris.
    pub fn window_title(&self) -> String {
        let mark = if self.is_dirty() { DIRTY_MARK } else { "" };
        format!("{mark}{} — {APP_TITLE}", self.document_label())
    }

    /// Pose le titre sur la fenêtre s'il a changé. Appelé à chaque frame : le marqueur
    /// apparaît dès la première modification et disparaît dès l'enregistrement, sans qu'aucune
    /// mutation n'ait à y penser.
    pub fn sync_window_title(&mut self) {
        let title = self.window_title();
        if self.window_title_cache == title {
            return;
        }
        if let Some(window) = &self.window {
            window.set_title(&title);
        }
        self.window_title_cache = title;
    }

    /// `Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`, `Ctrl+N`, `Ctrl+I`, `Ctrl+E`. Rend `true` si la touche a été
    /// consommée.
    pub fn handle_file_shortcut(&mut self, key: &str) -> bool {
        if !self.modifiers.control_key() {
            return false;
        }
        match key {
            "s" | "S" => {
                if self.modifiers.shift_key() {
                    self.save_project_as();
                } else {
                    self.save_project();
                }
                true
            }
            "o" | "O" => {
                self.open_project();
                true
            }
            "n" | "N" => {
                self.nouveau_document();
                true
            }
            "i" | "I" => {
                self.pick_and_import_images();
                true
            }
            "e" | "E" => {
                self.export_board();
                true
            }
            // La Time Machine, au raccourci de Glucose Tauri.
            "h" | "H" => {
                self.basculer_la_machine();
                true
            }
            _ => false,
        }
    }

    /// Enregistre, en demandant un chemin si le projet n'en a pas encore.
    pub fn save_project(&mut self) {
        let target = match self.project_path.clone() {
            Some(path) => Some(path),
            None => {
                let suggere = self.document_label();
                self.sous_un_dialogue(|fenetre| pick_save_path(fenetre, &suggere))
            }
        };
        if let Some(path) = target {
            self.save_to(path);
        }
    }

    /// Enregistre sous un nouveau chemin, qui devient celui du projet.
    pub fn save_project_as(&mut self) {
        let suggere = self.document_label();
        if let Some(path) = self.sous_un_dialogue(|fenetre| pick_save_path(fenetre, &suggere)) {
            self.save_to(path);
        }
    }

    /// Ouvre un projet, en remplaçant le document courant — après avoir demandé ce que devient
    /// le travail sans nom qu'on quitte (BROUILLON-1). Le fichier se choisit d'abord : renoncer
    /// au choix ne change rien.
    pub fn open_project(&mut self) {
        if let Some(path) = self.sous_un_dialogue(pick_open_path) {
            if self.laisser_le_document() {
                self.open_from(path);
            }
        }
    }

    /// Le chemin est connu : enregistrer, confirmer ou dire pourquoi ça a échoué.
    pub(crate) fn save_to(&mut self, path: PathBuf) {
        let message = match self.try_save(&path) {
            Ok(report) => {
                self.retenir_le_document(&path);
                self.project_path = Some(path);
                self.saved_version = self.store.version;
                save_message(&report, &self.document_label())
            }
            Err(err) => err.to_string(),
        };
        self.ui.show_toast(message);
        // Ctrl+S pose un jalon : la Time Machine ouverte le montre aussitôt.
        if self.dock_manager.is_open(crate::dock::TabId::Temps) {
            self.lire_l_histoire();
        }
        self.sync_window_title();
        self.mark_dirty();
    }

    /// **Enregistrer ne réécrit plus rien** (HISTOIRE-1). Le document s'écrit déjà, geste
    /// après geste ; enregistrer, c'est :
    ///
    /// * au même endroit — poser un jalon, et attendre que tout soit sur le disque ;
    /// * ailleurs (« Enregistrer sous », ou le premier nom d'un brouillon) — copier le fichier
    ///   tel quel, histoire comprise, et continuer dans la copie ;
    /// * pour un document qui n'a encore aucun fichier — l'écrire là : sa base, ses gestes,
    ///   ses images.
    fn try_save(&mut self, path: &Path) -> DesktopResult<SaveReport> {
        let echec = |reason: String| DesktopError::SaveFailed {
            path: path.display().to_string(),
            reason,
        };
        let instant = now_millis();
        match self.disque.ecriture.as_ref().map(|e| e.chemin == path) {
            Some(true) => {}
            Some(false) => {
                // Ce qui attend s'écrit d'abord dans l'ancien fichier : la copie l'emporte.
                self.consigner();
                self.disque
                    .ecriture
                    .as_mut()
                    .ok_or_else(|| echec("aucun fichier ouvert".to_string()))?
                    .deplacer(path.to_path_buf())
                    .map_err(echec)?;
            }
            None => self
                .naitre(path.to_path_buf(), false, instant)
                .map_err(echec)?,
        }
        self.consigner();
        let ecriture = self
            .disque
            .ecriture
            .as_mut()
            .ok_or_else(|| echec("aucun fichier ouvert".to_string()))?;
        ecriture
            .jalon(&self.store.project, Genre::Enregistrement, "", instant)
            .map_err(echec)?;
        Ok(SaveReport {
            bytes: std::fs::metadata(path)
                .map(|m| m.len() as usize)
                .unwrap_or(0),
            assets: self.store.nombre_d_images(),
            unreadable: self.images_hors_du_document(),
        })
    }

    /// Les images dont les octets ne sont pas encore dans le document, et qu'aucun fichier ne
    /// porte plus : elles s'afficheront vides à la prochaine ouverture. L'utilisateur doit le
    /// savoir (standard § 6.4).
    fn images_hors_du_document(&self) -> usize {
        self.store
            .project
            .toutes_les_images()
            .filter_map(|i| i.src.as_deref())
            .filter(|cle| {
                !self.disque.objets.est_scellee(cle)
                    && !matches!(
                        self.disque.objets.source(cle),
                        Some(Source::Fichier(_)) | Some(Source::Memoire(_))
                    )
                    && !Path::new(cle).is_file()
            })
            .count()
    }

    /// Le chemin est connu : lire, adopter le document ou dire pourquoi ça a échoué.
    pub(crate) fn open_from(&mut self, path: PathBuf) {
        let message = self.ouvrir_et_dire(path);
        self.ui.show_toast(message);
    }

    /// Ouvre, et **rend** ce que l'ouverture a à dire sans le dire : le lancement l'ajoute à
    /// son propre compte rendu — un geste, un compte rendu.
    pub(crate) fn ouvrir_et_dire(&mut self, path: PathBuf) -> String {
        let message = match self.try_open(&path) {
            Ok(Ouverture::Glucose(report)) => {
                self.retenir_le_document(&path);
                self.project_path = Some(path);
                self.saved_version = self.store.version;
                open_message(&self.store.project, &report)
            }
            Ok(Ouverture::Tauri(importe)) => message_d_import(&self.store.project.name, &importe),
            Err(err) => err.to_string(),
        };
        self.sync_window_title();
        self.mark_dirty();
        message
    }

    fn try_open(&mut self, path: &Path) -> DesktopResult<Ouverture> {
        let ouvrir_err = |e: std::io::Error| DesktopError::OpenFailed {
            path: path.display().to_string(),
            reason: e.to_string(),
        };
        let mut f = std::fs::File::open(path).map_err(ouvrir_err)?;
        let mut tete = [0u8; 16];
        let lus = f.read(&mut tete).map_err(ouvrir_err)?;
        if glucose_core::persist::tauri::reconnaitre(&tete[..lus]).is_some() {
            let octets = std::fs::read(path).map_err(ouvrir_err)?;
            return self.importer_de_tauri(path, &octets).map(Ouverture::Tauri);
        }
        // Un document qu'une autre fenêtre de Glucose écrit s'ouvre là-bas, pas ici : deux
        // fenêtres au même titre, dont une qui n'écrit rien, c'est à ne plus savoir laquelle
        // fermer. Celle-ci garde son document.
        let ecrit_ici = self
            .disque
            .ecriture
            .as_ref()
            .is_some_and(|e| super::verrou::meme_fichier(&e.chemin, path));
        if !ecrit_ici && super::verrou::ecrit_ailleurs(path) {
            return Err(DesktopError::DejaOuvertAilleurs {
                nom: path.file_stem().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                ),
            });
        }
        f.seek(SeekFrom::Start(0)).map_err(ouvrir_err)?;
        let ouvert = histoire::ouvrir(&mut std::io::BufReader::new(f))?;
        let adoption = self.adopter_un_ouvert(&ouvert, path.to_path_buf());
        Ok(Ouverture::Glucose(OpenReport {
            repaired: adoption.repares,
            fin_ignoree: ouvert.fin_ignoree,
            mise_de_cote: adoption.mise_de_cote,
            geste_en_echec: ouvert.geste_en_echec.is_some(),
            refus: adoption.refus,
            texte_rendu: adoption.texte_rendu,
        }))
    }
}

/// Ce qu'une ouverture a rendu : un document de Glucose Rust, ou un import de Glucose Tauri.
enum Ouverture {
    Glucose(OpenReport),
    Tauri(super::import::Importe),
}

/// Ce qu'une ouverture a dû rattraper. Tout se dit à l'utilisateur plutôt que de se faire en
/// silence (§ 6.4).
struct OpenReport {
    /// Assignations de domaine réparées.
    repaired: usize,
    /// Octets de fin ignorés : un enregistrement interrompu (un plantage, une coupure).
    fin_ignoree: u64,
    /// Où ces octets ont été mis de côté avant que l'écriture ne les recouvre (FIN-1).
    mise_de_cote: Option<PathBuf>,
    /// Un geste de l'histoire n'a pas pu se rejouer : le document est ouvert dans le dernier
    /// état cohérent.
    geste_en_echec: bool,
    /// Pourquoi le fichier ne s'écrit pas sur place, s'il ne s'écrit pas : les changements
    /// iront dans un brouillon.
    refus: Option<String>,
    /// Le texte qu'on tapait quand Glucose s'est arrêté est revenu.
    texte_rendu: bool,
}

fn save_message(report: &SaveReport, label: &str) -> String {
    let mut msg = format!("« {label} » enregistré — {}", human_size(report.bytes));
    if report.assets > 0 {
        msg.push_str(&format!(", {} image(s) dans le document", report.assets));
    }
    if report.unreadable > 0 {
        msg.push_str(&format!(", {} introuvable(s)", report.unreadable));
    }
    msg
}

fn open_message(project: &Project, report: &OpenReport) -> String {
    let boards = project.boards.len();
    let mut msg = format!("« {} » ouvert — {boards} tableau(x)", project.name);
    if report.repaired > 0 {
        msg.push_str(&format!(
            ", {} nœud(s) réparé(s) : domaine disparu ou pondération aberrante",
            report.repaired
        ));
    }
    if report.fin_ignoree > 0 {
        msg.push_str(&dire_la_fin(
            report.fin_ignoree,
            report.mise_de_cote.as_deref(),
        ));
    }
    if let Some(r) = &report.refus {
        msg.push_str(&format!(
            ", non modifiable sur place ({r}) : tes changements iront dans un brouillon"
        ));
    }
    if report.geste_en_echec {
        msg.push_str(
            ", un geste de l'histoire n'a pas pu se rejouer : ouvert dans le dernier état sûr",
        );
    }
    if report.texte_rendu {
        msg.push_str(", le texte que tu tapais quand Glucose s'est arrêté est revenu : continue");
    }
    msg
}

/// Ce qu'une ouverture dit d'une fin ignorée : sa taille, et où elle a été mise de côté —
/// rien n'est recouvert sans l'avoir été (FIN-1). Sans lieu, le document ne s'écrit pas sur
/// place, et la fin n'a pas été touchée.
pub(super) fn dire_la_fin(octets: u64, cote: Option<&Path>) -> String {
    let taille = human_size(usize::try_from(octets).unwrap_or(usize::MAX));
    match cote {
        Some(p) => format!(
            ", la fin d'un enregistrement interrompu ({taille}) mise de côté dans {}",
            p.display()
        ),
        None => format!(", la fin d'un enregistrement interrompu ({taille}) laissée intacte"),
    }
}

#[cfg(test)]
mod tests;
