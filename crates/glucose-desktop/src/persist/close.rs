//! Fermer la fenêtre sans perdre le travail (R-48, INVARIANT SAVE-3).
//!
//! Un document propre se ferme sans un mot. Un document modifié pose la question, et la
//! réponse passe par [`GlucoseApp::close_with`], qui relit l'état « modifié » **après**
//! l'enregistrement : tant que le document l'est encore, rien n'a été écrit, donc la
//! fenêtre reste ouverte. Fermer après un échec d'écriture serait une version pire du
//! défaut qu'on répare.
//!
//! L'état « modifié » n'est pas réinventé ici : c'est `store.version != saved_version`,
//! lu par `is_dirty()` (SAVE-2). Deux mécanismes finiraient par diverger.

use crate::app::GlucoseApp;

/// Ce que l'utilisateur répond quand on ferme une fenêtre au document modifié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseChoice {
    /// Enregistrer, puis fermer — et **ne pas** fermer si l'enregistrement échoue.
    Save,
    /// Fermer en abandonnant les modifications.
    Discard,
    /// Ne pas fermer.
    Cancel,
}

/// Le texte de la question, séparé du dialogue pour être relisible en test.
///
/// Le sens de chaque bouton est écrit ici parce qu'il ne peut pas l'être sur le bouton
/// lui-même (voir [`ask_unsaved_changes`]) : l'utilisateur doit pouvoir répondre sans
/// deviner ce que « Oui » enregistre (standard § 6.5).
fn unsaved_changes_question(label: &str) -> String {
    format!(
        "« {label} » contient des modifications non enregistrées.\n\n\
         Oui — enregistrer puis fermer\n\
         Non — fermer sans enregistrer\n\
         Annuler — revenir à Glucose"
    )
}

/// Pose la question des modifications non enregistrées et traduit la réponse.
///
/// Les libellés restent `Oui / Non / Annuler` et leur sens est écrit dans le corps du
/// message : renommer les boutons demanderait la variante `common-controls-v6` de `rfd`,
/// donc une modification des dépendances — et sans elle, Windows retombe silencieusement
/// sur `MessageBoxW`, qui ignore les libellés personnalisés. Un bouton dont le texte
/// disparaît selon la plate-forme serait pire qu'un bouton standard expliqué.
fn ask_unsaved_changes(label: &str) -> CloseChoice {
    let answer = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title("Modifications non enregistrées")
        .set_description(unsaved_changes_question(label))
        .set_buttons(rfd::MessageButtons::YesNoCancel)
        .show();
    match answer {
        rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok => CloseChoice::Save,
        rfd::MessageDialogResult::No => CloseChoice::Discard,
        rfd::MessageDialogResult::Cancel => CloseChoice::Cancel,
        // Une réponse personnalisée n'est possible qu'avec `common-controls-v6` ; en son
        // absence elle n'arrive jamais. Le seul choix sûr reste de ne rien perdre.
        rfd::MessageDialogResult::Custom(_) => CloseChoice::Cancel,
    }
}

impl GlucoseApp {
    /// Réponse à la croix de fermeture : `true` si la fenêtre a le droit de se fermer.
    ///
    /// INVARIANT SAVE-3 — la fermeture ne perd jamais de travail (R-48). Un document propre
    /// se ferme sans un mot ; un document modifié pose la question, et la réponse est
    /// traitée par [`GlucoseApp::close_with`], qui refuse de fermer sur un échec.
    pub fn request_close(&mut self) -> bool {
        if !self.is_dirty() {
            return true;
        }
        self.close_with(ask_unsaved_changes(&self.document_label()))
    }

    /// Applique une réponse à la question des modifications non enregistrées.
    ///
    /// Séparée de [`GlucoseApp::request_close`] pour que la règle — et surtout le refus de
    /// fermer après un enregistrement raté — soit vérifiable sans ouvrir de dialogue.
    pub fn close_with(&mut self, choice: CloseChoice) -> bool {
        match choice {
            CloseChoice::Cancel => false,
            CloseChoice::Discard => true,
            CloseChoice::Save => {
                self.save_project();
                // `save_project` a déjà dit pourquoi si l'écriture a échoué (toast). La
                // seule question qui reste est celle du document lui-même : tant qu'il est
                // modifié, rien n'a été écrit, et fermer perdrait exactement ce que R-48
                // décrit. On relit donc l'état « modifié », jamais le retour du dialogue.
                !self.is_dirty()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persist::read_project_file;
    use glucose_core::types::Annotation;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("glucose-tests-close");
        std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
        dir.join(name)
    }

    /// Salit le document d'une note, comme le ferait l'utilisateur.
    fn dirty_app() -> GlucoseApp {
        let mut app = GlucoseApp::new();
        let board_id = app.store.project.active_board_id.clone();
        app.store.add_annotation(
            &board_id,
            Annotation::Text {
                id: "travail-en-cours".into(),
                x: 0.0,
                y: 0.0,
                width: Some(200.0),
                height: Some(48.0),
                text: "ce texte ne doit pas disparaitre".into(),
                font_size: Some(14.0),
                color: None,
                cursor_pos: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            },
        );
        assert!(app.is_dirty());
        app
    }

    // ── SAVE-3 — fermer ne perd jamais de travail (R-48) ────────────────────

    #[test]
    fn test_save_3_a_clean_document_closes_without_asking() {
        // `request_close` est le vrai point d'entree : sur un document propre il ne doit
        // ouvrir AUCUN dialogue, sinon ce test bloquerait la suite.
        let mut app = GlucoseApp::new();
        assert!(!app.is_dirty());
        assert!(app.request_close(), "un document propre se ferme sans un mot");
    }

    #[test]
    fn test_save_3_cancel_keeps_the_window_open_and_the_work_intact() {
        let mut app = dirty_app();
        assert!(!app.close_with(CloseChoice::Cancel), "Annuler ne doit pas fermer");
        assert!(app.is_dirty(), "Annuler ne touche pas au document");
    }

    #[test]
    fn test_save_3_discard_closes_and_writes_nothing() {
        let mut app = dirty_app();
        assert!(app.close_with(CloseChoice::Discard), "Ne pas enregistrer doit fermer");
        assert!(
            app.project_path.is_none(),
            "abandonner ne doit creer aucun fichier"
        );
    }

    #[test]
    fn test_save_3_save_then_close_writes_the_file_first() {
        let path = scratch("fermeture-avec-enregistrement.glucose");
        let _ = std::fs::remove_file(&path);
        let mut app = dirty_app();
        app.project_path = Some(path.clone());

        assert!(app.close_with(CloseChoice::Save), "un enregistrement reussi autorise la fermeture");
        assert!(!app.is_dirty(), "le document doit etre propre apres l'enregistrement");
        assert!(path.exists(), "le fichier doit exister avant que la fenetre ne parte");

        let reloaded = read_project_file(&path).expect("relecture");
        assert_eq!(reloaded.project, app.store.project, "le document ecrit differe");
        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_save_3_a_failed_save_does_not_close_the_window() {
        // Le bug repare serait pire que l'original : fermer APRES avoir echoue a ecrire.
        let mut app = dirty_app();
        app.project_path = Some(PathBuf::from("C:/nulle-part/dossier-absent/projet.glucose"));

        assert!(
            !app.close_with(CloseChoice::Save),
            "un enregistrement rate doit garder la fenetre ouverte"
        );
        assert!(app.is_dirty(), "rien n'a ete ecrit : le document reste modifie");
        let toast = app.ui.current_toast.as_ref().expect("l'echec doit etre explique");
        assert!(toast.message.contains("Enregistrement impossible"), "toast : {}", toast.message);
    }

    #[test]
    fn test_save_3_the_question_says_what_each_button_does() {
        // Les libelles restent Oui/Non/Annuler (cf. `ask_unsaved_changes`) : leur sens doit
        // donc etre lisible dans le corps du message, sinon l'utilisateur devine.
        let q = unsaved_changes_question("carnet");
        assert!(q.contains("carnet"));
        assert!(q.contains("Oui — enregistrer puis fermer"));
        assert!(q.contains("Non — fermer sans enregistrer"));
        assert!(q.contains("Annuler — revenir à Glucose"));
    }
}
