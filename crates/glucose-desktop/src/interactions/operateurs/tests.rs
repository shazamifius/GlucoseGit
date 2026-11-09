//! OPERATEUR-1 joué au clavier, sur le vrai `GlucoseApp` : `Alt+1` à `Alt+4`, `Alt+0`.

use super::*;
use glucose_core::types::Annotation;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, SmolStr};

/// Deux post-it et un texte, les trois choisis.
fn app_avec_des_post_it() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::sticky("p", 0.0, 0.0, "il pleut"));
    app.store
        .add_annotation(&board, Annotation::sticky("q", 300.0, 0.0, "je sors"));
    app.store
        .add_annotation(&board, Annotation::text("t", 600.0, 0.0, "un texte"));
    app.store
        .set_selected_annotation_ids(vec!["p".into(), "q".into(), "t".into()]);
    app.store.journal.clear();
    app
}

fn alt(app: &mut GlucoseApp, chiffre: &str) {
    app.modifiers = ModifiersState::ALT;
    app.handle_shortcut_input(
        &Key::Character(SmolStr::new(chiffre)),
        ElementState::Pressed,
    );
    app.modifiers = ModifiersState::empty();
}

fn operateur(app: &GlucoseApp, id: &str) -> Option<StickyOperator> {
    match app
        .store
        .project
        .annotation(&app.store.project.active_board_id, id)?
    {
        Annotation::Sticky { operator, .. } => *operator,
        _ => None,
    }
}

/// **`Alt+1` à `Alt+4` posent ET, OU, MAIS, PARCE QUE sur chaque post-it choisi ; `Alt+0` les
/// rend au papier** — comme Glucose Tauri. Le texte choisi avec eux n'est pas touché.
#[test]
fn operateur_1_alt_et_un_chiffre_posent_l_operateur() {
    let mut app = app_avec_des_post_it();
    for (chiffre, attendu) in [
        ("1", StickyOperator::And),
        ("2", StickyOperator::Or),
        ("3", StickyOperator::But),
        ("4", StickyOperator::Because),
    ] {
        alt(&mut app, chiffre);
        assert_eq!(operateur(&app, "p"), Some(attendu), "Alt+{chiffre}");
        assert_eq!(operateur(&app, "q"), Some(attendu), "toute la sélection");
    }
    assert!(
        matches!(
            app.store
                .project
                .annotation(&app.store.project.active_board_id, "t"),
            Some(Annotation::Text { .. })
        ),
        "le texte reste un texte"
    );
    alt(&mut app, "0");
    assert_eq!(operateur(&app, "p"), None, "Alt+0 : le papier revient");
}

/// **Un seul `Ctrl+Z`** défait l'opérateur posé sur deux post-it.
#[test]
fn operateur_1_un_geste_se_defait_d_un_coup() {
    let mut app = app_avec_des_post_it();
    alt(&mut app, "2");
    assert!(app.store.undo());
    assert_eq!(operateur(&app, "p"), None);
    assert_eq!(operateur(&app, "q"), None);
}

/// **Sans post-it choisi, `Alt+1` ne fait rien** — ni geste, ni document modifié.
#[test]
fn operateur_1_sans_post_it_rien_ne_s_ecrit() {
    let mut app = app_avec_des_post_it();
    app.store.set_selected_annotation_ids(vec!["t".into()]);
    let version = app.store.version;
    alt(&mut app, "1");
    assert!(!app.store.can_undo(), "aucun geste");
    assert_eq!(app.store.version, version, "le document n'a pas changé");
    assert!(
        !app.executer(crate::commandes::Commande::Operateur(Some(
            StickyOperator::And
        ))),
        "la commande dit qu'elle ne s'applique pas : l'accord passe à la suivante"
    );
}
