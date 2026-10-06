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
    assert!(
        app.is_dirty(),
        "une annotation ajoutee doit salir le document"
    );
    assert!(
        app.window_title().starts_with(DIRTY_MARK),
        "titre : {}",
        app.window_title()
    );

    app.save_to(path.clone());
    assert!(!app.is_dirty(), "l'enregistrement doit effacer le marqueur");
    assert!(
        !app.window_title().starts_with(DIRTY_MARK),
        "titre : {}",
        app.window_title()
    );
    assert_eq!(app.project_path.as_deref(), Some(path.as_path()));
    assert!(
        app.window_title().contains("app-aller-retour"),
        "le titre doit nommer le fichier : {}",
        app.window_title()
    );

    // Une seule fenêtre écrit un document : celle-ci le ferme avant qu'une autre le rouvre.
    assert!(app.fermer_le_document());
    let mut reopened = GlucoseApp::new();
    let version_before = reopened.store.version;
    reopened.open_from(path.clone());
    assert_eq!(
        reopened.store.project, app.store.project,
        "le document rouvert differe de celui qui a ete enregistre"
    );
    assert!(
        !reopened.is_dirty(),
        "un document qu'on vient d'ouvrir est propre"
    );
    assert_ne!(
        reopened.store.version, version_before,
        "la version doit avancer, sinon l'index spatial du renderer reste perime"
    );
    assert!(
        reopened.store.undo_depth() == 0,
        "l'undo du projet precedent doit partir"
    );

    // Un document ouvert ne s'efface pas (un seul scribe par fichier) : on ferme d'abord.
    drop((app, reopened));
    std::fs::remove_file(&path).expect("nettoyage");
}

#[test]
fn test_opening_a_corrupted_file_leaves_the_current_document_alone() {
    let path = scratch("corrompu.glucose");
    std::fs::write(&path, b"ceci n'est pas un projet Glucose").expect("ecriture");

    let mut app = GlucoseApp::new();
    let before = app.store.project.clone();
    app.open_from(path.clone());

    assert_eq!(
        app.store.project, before,
        "un fichier illisible ne doit rien remplacer"
    );
    assert!(
        app.project_path.is_none(),
        "le chemin ne doit pas etre adopte"
    );
    let toast = app
        .ui
        .current_toast
        .as_ref()
        .expect("un toast doit expliquer l'echec");
    assert!(
        toast.message.contains("signature"),
        "toast : {}",
        toast.message
    );

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
    assert!(msg.contains("2 image(s) dans le document"));
    assert!(msg.contains("1 introuvable(s)"));

    let mut project = Project::new("deux tableaux");
    project
        .boards
        .push(glucose_core::types::Board::new("b2", "Annexe"));
    let sain = OpenReport {
        repaired: 0,
        fin_ignoree: 0,
        mise_de_cote: None,
        geste_en_echec: false,
        refus: None,
        texte_rendu: false,
    };
    let opened = open_message(&project, &sain);
    assert!(opened.contains("2 tableau(x)"));
    for silence in ["réparé", "interrompu", "rejouer", "tapais"] {
        assert!(
            !opened.contains(silence),
            "un document sain ne parle pas de « {silence} » : {opened}"
        );
    }

    let abime = open_message(
        &project,
        &OpenReport {
            repaired: 4,
            fin_ignoree: 12,
            mise_de_cote: Some(PathBuf::from("C:/cote/abime-81-7.fin")),
            geste_en_echec: true,
            refus: Some("lecture seule".into()),
            texte_rendu: true,
        },
    );
    assert!(abime.contains("4 nœud(s) réparé(s)"), "{abime}");
    assert!(abime.contains("interrompu (12 o)"), "{abime}");
    assert!(abime.contains("mise de côté dans"), "{abime}");
    assert!(abime.contains("abime-81-7.fin"), "{abime}");
    assert!(abime.contains("dernier état sûr"), "{abime}");
    assert!(abime.contains("brouillon"), "{abime}");
    assert!(abime.contains("tapais"), "{abime}");
}
