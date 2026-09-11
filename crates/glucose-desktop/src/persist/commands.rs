//! Les commandes de fichier vues depuis l'application : état « modifié », titre de fenêtre,
//! dialogues natifs, et les trois raccourcis qui les déclenchent.
//!
//! Le module voisin [`super`] fait l'I/O pure (encoder, écrire atomiquement, relire) sans rien
//! savoir de `GlucoseApp`. Ici on ne fait que l'orchestrer et remonter chaque échec par un
//! toast (standard § 6.4) : un enregistrement raté doit se voir.

use super::{
    human_size, now_millis, read_project_file, with_glucose_extension, write_project_file,
    SaveReport, APP_TITLE, DIRTY_MARK, UNTITLED,
};
use crate::app::GlucoseApp;
use crate::error::DesktopResult;
use crate::persist::assets;
use glucose_core::persist::FILE_EXTENSION;
use glucose_core::types::Project;
use std::path::{Path, PathBuf};

// ── Dialogues ───────────────────────────────────────────────────────────────

fn pick_save_path(suggested: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Projet Glucose", &[FILE_EXTENSION])
        .set_file_name(format!("{suggested}.{FILE_EXTENSION}"))
        .save_file()
        .map(with_glucose_extension)
}

fn pick_open_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Projet Glucose", &[FILE_EXTENSION])
        .pick_file()
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

    /// `Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`, `Ctrl+I`. Rend `true` si la touche a été consommée.
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
            "i" | "I" => {
                self.pick_and_import_images();
                true
            }
            _ => false,
        }
    }

    /// Enregistre, en demandant un chemin si le projet n'en a pas encore.
    pub fn save_project(&mut self) {
        let target = match self.project_path.clone() {
            Some(path) => Some(path),
            None => pick_save_path(&self.document_label()),
        };
        if let Some(path) = target {
            self.save_to(path);
        }
    }

    /// Enregistre sous un nouveau chemin, qui devient celui du projet.
    pub fn save_project_as(&mut self) {
        if let Some(path) = pick_save_path(&self.document_label()) {
            self.save_to(path);
        }
    }

    /// Ouvre un projet, en remplaçant le document courant.
    pub fn open_project(&mut self) {
        if let Some(path) = pick_open_path() {
            self.open_from(path);
        }
    }

    /// Le chemin est connu : encoder, écrire, confirmer ou dire pourquoi ça a échoué.
    fn save_to(&mut self, path: PathBuf) {
        match self.try_save(&path) {
            Ok(report) => {
                self.project_path = Some(path);
                self.saved_version = self.store.version;
                self.ui.show_toast(save_message(&report, &self.document_label()));
            }
            Err(err) => self.ui.show_toast(format!("⚠️ {err}")),
        }
        self.sync_window_title();
        self.mark_dirty();
    }

    fn try_save(&mut self, path: &Path) -> DesktopResult<SaveReport> {
        // Le magasin d'actifs est reconstruit à chaque enregistrement : c'est ce qui rend le
        // ramasse-miettes du § 8 automatique (cf. `assets::collect`).
        let collected = assets::collect(&self.store.project);
        let report = SaveReport {
            bytes: write_project_file(path, &self.store.project, &collected.store, now_millis())?,
            assets: collected.store.len(),
            unreadable: collected.unreadable.len(),
        };
        self.store.assets = collected.store;
        Ok(report)
    }

    /// Le chemin est connu : lire, décoder, adopter le document ou dire pourquoi ça a échoué.
    fn open_from(&mut self, path: PathBuf) {
        match self.try_open(&path) {
            Ok(restored) => {
                self.project_path = Some(path);
                self.saved_version = self.store.version;
                self.ui.show_toast(open_message(&self.store.project, restored));
            }
            Err(err) => self.ui.show_toast(format!("⚠️ {err}")),
        }
        self.sync_window_title();
        self.mark_dirty();
    }

    fn try_open(&mut self, path: &Path) -> DesktopResult<usize> {
        let mut file = read_project_file(path)?;
        let restored = assets::restore(&mut file.project, &file.assets);

        // L'édition en cours porte sur un document qui n'existe plus.
        self.editing_session = None;
        self.selection_box = None;
        self.store.load_project(file.project);
        self.store.assets = file.assets;
        // `load_project` ne fait pas avancer la version ; sans ce coup de pouce, l'index
        // spatial du renderer croirait regarder le document précédent et n'afficherait rien.
        self.store.bump_version();
        Ok(restored)
    }
}

fn save_message(report: &SaveReport, label: &str) -> String {
    let mut msg = format!("💾 « {label} » enregistré — {}", human_size(report.bytes));
    if report.assets > 0 {
        msg.push_str(&format!(", {} image(s) incorporée(s)", report.assets));
    }
    if report.unreadable > 0 {
        msg.push_str(&format!(", {} introuvable(s)", report.unreadable));
    }
    msg
}

fn open_message(project: &Project, restored: usize) -> String {
    let boards = project.boards.len();
    let mut msg = format!("📂 « {} » ouvert — {boards} tableau(x)", project.name);
    if restored > 0 {
        msg.push_str(&format!(", {restored} image(s) restituée(s)"));
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::Annotation;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("glucose-tests-commands");
        std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
        dir.join(name)
    }


    #[test]
    fn test_a_fresh_app_is_clean_and_wears_no_marker() {
        let app = GlucoseApp::new();
        assert!(!app.is_dirty(), "un projet neuf n'a rien a enregistrer");
        assert!(app.project_path.is_none());
        assert!(
            !app.window_title().starts_with(DIRTY_MARK),
            "titre : {}",
            app.window_title()
        );
        assert!(app.window_title().ends_with(APP_TITLE));
    }

    #[test]
    fn test_edit_then_save_then_reopen_through_the_application_itself() {
        // Le scenario complet de R-01, joue sur le vrai `GlucoseApp` : on modifie, le titre
        // porte le marqueur, on enregistre, il disparait, et une AUTRE instance relit le
        // fichier et retrouve le meme document.
        let path = scratch("app-aller-retour.glucose");
        let mut app = GlucoseApp::new();
        let board_id = app.store.project.active_board_id.clone();

        app.store.add_annotation(
            &board_id,
            Annotation::Sticky {
                id: "note-essai".into(),
                x: 120.0,
                y: -64.0,
                width: Some(180.0),
                height: Some(90.0),
                text: "ne doit pas disparaitre".into(),
                font_size: Some(12.0),
                color: None,
                bg_color: Some("#fde047".into()),
                cursor_pos: None,
                operator: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            },
        );
        assert!(app.is_dirty(), "une annotation ajoutee doit salir le document");
        assert!(app.window_title().starts_with(DIRTY_MARK), "titre : {}", app.window_title());

        app.save_to(path.clone());
        assert!(!app.is_dirty(), "l'enregistrement doit effacer le marqueur");
        assert!(!app.window_title().starts_with(DIRTY_MARK), "titre : {}", app.window_title());
        assert_eq!(app.project_path.as_deref(), Some(path.as_path()));
        assert!(
            app.window_title().contains("app-aller-retour"),
            "le titre doit nommer le fichier : {}",
            app.window_title()
        );

        let mut reopened = GlucoseApp::new();
        let version_before = reopened.store.version;
        reopened.open_from(path.clone());
        assert_eq!(
            reopened.store.project, app.store.project,
            "le document rouvert differe de celui qui a ete enregistre"
        );
        assert!(!reopened.is_dirty(), "un document qu'on vient d'ouvrir est propre");
        assert_ne!(
            reopened.store.version, version_before,
            "la version doit avancer, sinon l'index spatial du renderer reste perime"
        );
        assert!(reopened.store.undo_stack.is_empty(), "l'undo du projet precedent doit partir");

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_opening_a_corrupted_file_leaves_the_current_document_alone() {
        let path = scratch("corrompu.glucose");
        std::fs::write(&path, b"ceci n'est pas un projet Glucose").expect("ecriture");

        let mut app = GlucoseApp::new();
        let before = app.store.project.clone();
        app.open_from(path.clone());

        assert_eq!(app.store.project, before, "un fichier illisible ne doit rien remplacer");
        assert!(app.project_path.is_none(), "le chemin ne doit pas etre adopte");
        let toast = app.ui.current_toast.as_ref().expect("un toast doit expliquer l'echec");
        assert!(toast.message.contains("signature"), "toast : {}", toast.message);

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_file_shortcuts_only_fire_with_ctrl() {
        let mut app = GlucoseApp::new();
        // Sans Ctrl, `s`, `o` et `i` appartiennent aux outils, pas au menu Fichier.
        assert!(!app.handle_file_shortcut("s"));
        assert!(!app.handle_file_shortcut("o"));
        assert!(!app.handle_file_shortcut("i"));
        // Avec Ctrl, une lettre etrangere au menu Fichier n'est pas consommee non plus.
        app.modifiers = winit::keyboard::ModifiersState::CONTROL;
        assert!(!app.handle_file_shortcut("q"));
        assert!(!app.handle_file_shortcut("z"));
    }

    #[test]
    fn test_the_untitled_document_is_named_rather_than_left_blank() {
        let mut app = GlucoseApp::new();
        app.store.project.name = String::new();
        assert_eq!(app.document_label(), UNTITLED);
    }


    #[test]
    fn test_save_and_open_messages_stay_honest() {
        let report = SaveReport {
            bytes: 4096,
            assets: 2,
            unreadable: 1,
        };
        let msg = save_message(&report, "carnet");
        assert!(msg.contains("carnet"));
        assert!(msg.contains("4.0 Kio"));
        assert!(msg.contains("2 image(s) incorporée(s)"));
        assert!(msg.contains("1 introuvable(s)"));

        let mut project = Project::new("deux tableaux");
        project.boards.push(glucose_core::types::Board::new("b2", "Annexe"));
        let opened = open_message(&project, 3);
        assert!(opened.contains("2 tableau(x)"));
        assert!(opened.contains("3 image(s) restituée(s)"));
    }
}
