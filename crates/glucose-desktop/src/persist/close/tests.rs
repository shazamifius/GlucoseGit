//! SAVE-3 — fermer ne perd jamais de travail (R-48), et la question qui le demande se dessine
//! (POPUP-1).

use super::*;
use crate::interactions::question::tests::repondre;
use crate::persist::read_project_file;
use glucose_core::types::Annotation;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("glucose-tests-close");
    std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
    dir.join(name)
}

/// **La croix, par le vrai chemin** : rend `true` si la fenêtre a le droit de partir. Un
/// document propre n'attend aucune réponse ; une question posée en attend une, et la fenêtre
/// reste.
pub(crate) fn fermer(app: &mut GlucoseApp) -> bool {
    app.fermer_puis(Apres::Quitter);
    app.ui.fermer_la_fenetre
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

#[test]
fn test_save_3_a_clean_document_closes_without_asking() {
    let mut app = GlucoseApp::new();
    assert!(!app.is_dirty());
    assert!(fermer(&mut app), "un document propre se ferme sans un mot");
    assert!(app.ui.question.is_none(), "sans question");
}

/// **La croix sur un travail modifié pose la question, et la fenêtre attend** : rien ne part
/// avant la réponse. « Annuler » la garde ; « Ne pas enregistrer » la laisse partir sans rien
/// écrire — par les vrais boutons.
#[test]
fn test_save_3_la_croix_attend_la_reponse_et_annuler_garde_tout() {
    let mut app = dirty_app();
    assert!(!fermer(&mut app), "la fenêtre attend la réponse");
    assert!(
        matches!(app.ui.question, Some((_, Suite::Fermer(Apres::Quitter)))),
        "la question est posée"
    );
    repondre(&mut app, Reponse::Annuler);
    assert!(!app.ui.fermer_la_fenetre, "Annuler ne ferme pas");
    assert!(app.is_dirty(), "et ne touche pas au document");

    assert!(!fermer(&mut app));
    repondre(&mut app, Reponse::Non);
    assert!(app.ui.fermer_la_fenetre, "Ne pas enregistrer ferme");
    assert!(app.project_path.is_none(), "et n'écrit rien");
}

/// **« Enregistrer » écrit, puis ferme** — par le vrai bouton.
#[test]
fn test_save_3_enregistrer_ecrit_puis_ferme() {
    let path = scratch("fermeture-par-la-question.glucose");
    let _ = std::fs::remove_file(&path);
    let mut app = dirty_app();
    app.project_path = Some(path.clone());
    assert!(!fermer(&mut app), "modifié et jamais écrit : la question");
    repondre(&mut app, Reponse::Oui);
    assert!(app.ui.fermer_la_fenetre, "enregistré, la fenêtre part");
    assert!(
        path.exists(),
        "le fichier existe avant que la fenêtre ne parte"
    );
    std::fs::remove_file(&path).expect("nettoyage");
}

#[test]
fn test_save_3_cancel_keeps_the_window_open_and_the_work_intact() {
    let mut app = dirty_app();
    assert!(
        !app.close_with(CloseChoice::Cancel),
        "Annuler ne doit pas fermer"
    );
    assert!(app.is_dirty(), "Annuler ne touche pas au document");
}

#[test]
fn test_save_3_discard_closes_and_writes_nothing() {
    let mut app = dirty_app();
    assert!(
        app.close_with(CloseChoice::Discard),
        "Ne pas enregistrer doit fermer"
    );
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

    assert!(
        app.close_with(CloseChoice::Save),
        "un enregistrement reussi autorise la fermeture"
    );
    assert!(
        !app.is_dirty(),
        "le document doit etre propre apres l'enregistrement"
    );
    assert!(
        path.exists(),
        "le fichier doit exister avant que la fenetre ne parte"
    );

    let reloaded = read_project_file(&path).expect("relecture");
    assert_eq!(
        reloaded.project, app.store.project,
        "le document ecrit differe"
    );
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
    assert!(
        app.is_dirty(),
        "rien n'a ete ecrit : le document reste modifie"
    );
    let toast = app
        .ui
        .current_toast
        .as_ref()
        .expect("l'echec doit etre explique");
    assert!(
        toast.message.contains("Enregistrement impossible"),
        "toast : {}",
        toast.message
    );
}

/// **Chaque réponse dit ce qu'elle fait**, sur son bouton : « Enregistrer » d'abord — la
/// réponse qui ne perd rien, celle d'Entrée —, « Annuler » pour Échap.
#[test]
fn test_save_3_chaque_reponse_dit_ce_qu_elle_fait() {
    let q = question_du_travail("carnet");
    assert!(q.texte.contains("carnet"));
    let libelles: Vec<&str> = q.choix.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(libelles, ["Enregistrer", "Ne pas enregistrer", "Annuler"]);
    assert_eq!(
        crate::ui::question::reponse_en_evidence(&q),
        Some(Reponse::Oui)
    );
    assert_eq!(
        crate::ui::question::echappatoire(&q),
        Some(Reponse::Annuler)
    );
}
