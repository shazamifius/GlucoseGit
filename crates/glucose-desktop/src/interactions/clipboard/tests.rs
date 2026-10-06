//! Ce que `Ctrl+C` emporte, et ce que `Ctrl+V` pose.
//!
//! Sur le presse-papiers **à soi** du fil d'épreuve ([`crate::interactions::presse_papiers`]),
//! jamais celui du système : une épreuve qui l'écrivait remplaçait, à chaque `cargo test`, ce
//! que l'utilisateur venait de copier. La **décision** — quoi copier, et sous quelle forme —
//! portait la faute : `Ctrl+C` hors saisie n'existait pas du tout.

use super::*;
use crate::interactions::resize::tests::text_card;
use glucose_core::types::{Annotation, BoardImage};

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

/// **Une image ne rend aucun texte** : son nom interne ne veut rien dire pour un autre logiciel.
/// Elle voyage dans le lot (fiche 51 § 2).
#[test]
fn test_an_image_gives_no_text() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("i", 0.0, 0.0, 100.0, 80.0);
    img.src = Some("C:/photos/chat.png".into());
    app.store.add_image(&board, img);
    app.store.set_selected_image_ids(vec!["i".into()]);
    assert_eq!(app.store.selection_as_text(), None);
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

/// `Ctrl+C` et `Ctrl+X` hors saisie atteignent la copie — en un seul test, et c'est voulu.
///
/// Le presse-papiers est une ressource **unique du système** : deux tests qui l'écrivent en
/// parallèle se la disputent, l'un échoue, et `Ctrl+X` ne supprime alors rien puisqu'il ne
/// coupe que ce qu'il a réussi à copier. Écrits séparément, ils passaient seuls et tombaient
/// en suite complète — un test qui échoue au hasard ne dit plus rien et abîme la valeur des
/// autres. Un seul test séquentiel supprime la course.
#[test]
fn test_ctrl_c_and_ctrl_x_reach_the_copy_outside_a_text_session() {
    use winit::event::ElementState;
    use winit::keyboard::{Key, ModifiersState};

    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("c".into()), ElementState::Pressed);
    app.suivre_les_echanges(true);
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
    let reste = app
        .store
        .active_board()
        .map(|b| b.annotations.len())
        .unwrap_or(0);
    assert_eq!(reste, 1, "copier ne retire rien");

    // Et couper emporte **et** retire.
    app.handle_shortcut_input(&Key::Character("x".into()), ElementState::Pressed);
    app.suivre_les_echanges(true);
    let reste = app
        .store
        .active_board()
        .map(|b| b.annotations.len())
        .unwrap_or(0);
    assert_eq!(reste, 0, "couper doit aussi retirer");
}

/// **Coller un texte pose une carte qui le porte**, là où est la souris — par la fabrique, qui
/// la mesure — et sans un mot (fiche 51 § 2).
#[test]
fn test_coller_un_texte_pose_une_carte() {
    let mut app = app();
    crate::interactions::presse_papiers::ouvrir()
        .and_then(|mut a| a.ecrire("  bonjour\ndeux lignes \n".into()))
        .expect("écrit");
    let message_d_avant = app.ui.toast_message().map(str::to_string);
    app.paste_from_clipboard();
    let board = app.store.active_board().expect("tableau");
    assert_eq!(board.annotations.len(), 1, "une carte");
    let Annotation::Text { text, .. } = &board.annotations[0] else {
        panic!("une carte de texte");
    };
    assert_eq!(
        text, "bonjour\ndeux lignes",
        "le texte, sans ses blancs de bord"
    );
    // La carte apparaît sous le curseur : cela se regarde, et aucun message n'en parle.
    assert_eq!(
        app.ui.toast_message(),
        message_d_avant.as_deref(),
        "un collage réussi se tait"
    );
}
