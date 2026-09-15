//! Ce que `Ctrl+C` emporte — la part qui se teste sans presse-papiers.
//!
//! L'écriture elle-même appartient au système : aucune machine d'intégration n'a de
//! presse-papiers, et un test qui en dépendrait serait vert ici et rouge ailleurs. La
//! **décision** — quoi copier, et sous quelle forme — est pure, et c'est elle qui portait la
//! faute : `Ctrl+C` hors saisie n'existait pas du tout.

use super::*;
use crate::interactions::resize::tests::text_card;
use glucose_core::types::BoardImage;

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app
}

fn pose_carte(app: &mut GlucoseApp, id: &str, texte: &str) {
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, text_card(id, 0.0, 0.0, 240.0, texte));
}

/// Rien de sélectionné : rien à copier, et surtout pas un presse-papiers vidé.
#[test]
fn test_an_empty_selection_copies_nothing() {
    let app = app();
    assert_eq!(app.store.selection_as_text(), None);
}

/// Une carte sélectionnée rend son texte — le geste que la table des raccourcis oubliait.
#[test]
fn test_a_selected_card_gives_its_text() {
    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    assert_eq!(app.store.selection_as_text().as_deref(), Some("bonjour"));
}

/// Plusieurs cartes se séparent par une ligne vide : le collage les relira comme deux blocs
/// et non comme une phrase coupée en deux.
#[test]
fn test_several_cards_are_separated_by_a_blank_line() {
    let mut app = app();
    pose_carte(&mut app, "a", "premier");
    pose_carte(&mut app, "b", "second");
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    assert_eq!(
        app.store.selection_as_text().as_deref(),
        Some("premier\n\nsecond")
    );
}

/// Une image rend le chemin de son fichier, c'est-à-dire exactement ce que le collage sait
/// relire pour la réimporter. Copier et coller se referment l'un sur l'autre.
#[test]
fn test_an_image_gives_the_path_that_pasting_knows_how_to_read() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("i", 0.0, 0.0, 100.0, 80.0);
    img.src = Some("C:/photos/chat.png".into());
    app.store.add_image(&board, img);
    app.store.set_selected_image_ids(vec!["i".into()]);
    assert_eq!(
        app.store.selection_as_text().as_deref(),
        Some("C:/photos/chat.png")
    );
}

/// Une carte vide ne rend pas une ligne vide : elle ne rend rien.
#[test]
fn test_a_blank_card_contributes_nothing() {
    let mut app = app();
    pose_carte(&mut app, "a", "   ");
    pose_carte(&mut app, "b", "vrai texte");
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    assert_eq!(app.store.selection_as_text().as_deref(), Some("vrai texte"));
}

/// `Ctrl+C` hors saisie atteint bien la copie : c'est la ligne qui manquait à la table des
/// raccourcis, et sans ce test rien ne dirait qu'elle y est revenue.
#[test]
fn test_ctrl_c_reaches_the_copy_outside_a_text_session() {
    use winit::event::ElementState;
    use winit::keyboard::{Key, ModifiersState};

    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("c".into()), ElementState::Pressed);
    // L'accusé doit dire **copié**, pas n'importe quoi : un toast quelconque serait vert
    // même quand l'écriture échoue, et ce test-là mentirait exactement comme celui de la
    // touche Entrée mentait.
    let message = app
        .ui
        .current_toast
        .as_ref()
        .map(|t| t.message.clone())
        .unwrap_or_default();
    assert!(
        message.contains("copié"),
        "Ctrl+C doit dire ce qu'il a copié, pas \"{message}\""
    );
}

/// `Ctrl+X` emporte la sélection **et** la retire du document.
#[test]
fn test_ctrl_x_also_removes_what_it_took() {
    use winit::event::ElementState;
    use winit::keyboard::{Key, ModifiersState};

    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("x".into()), ElementState::Pressed);
    let reste = app
        .store
        .active_board()
        .map(|b| b.annotations.len())
        .unwrap_or(0);
    assert_eq!(reste, 0, "couper doit aussi retirer");
}
