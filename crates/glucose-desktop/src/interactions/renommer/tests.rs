//! RENOMMER-1 par les vrais gestes : `F2`, la question et son champ, la frappe, Entrée, `Échap`.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use crate::ui::context_menu::MenuAction;
use glucose_core::types::CanvasFolder;
use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey};

/// Un dossier « Dossier », choisi.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let board = app.store.project.active_board_id.clone();
    app.store
        .create_folder(&board, CanvasFolder::new("d", "Dossier", ""));
    app.store.select_folder("d".into());
    app
}

fn nom(app: &GlucoseApp) -> String {
    app.store.active_board().expect("un tableau").folders[0]
        .name
        .clone()
}

fn f2(app: &mut GlucoseApp) {
    app.handle_shortcut_input(&Key::Named(NamedKey::F2), ElementState::Pressed);
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        let t = c.to_string();
        assert!(app.prendre_la_touche(&Key::Character(t.clone().into()), true, Some(&t)));
    }
}

fn touche(app: &mut GlucoseApp, n: NamedKey) {
    assert!(app.prendre_la_touche(&Key::Named(n), true, None));
}

/// **`F2` ouvre la question, le nom choisi tout entier** : ce qu'on tape le remplace, Entrée
/// renomme, et un seul `Ctrl+Z` rend l'ancien nom.
#[test]
fn test_renommer_1_f2_tape_entree() {
    let mut app = app();
    f2(&mut app);
    let champ = app.ui.question.as_ref().and_then(|(q, _)| q.champ.clone());
    assert_eq!(
        champ.map(|c| c.texte),
        Some("Dossier".into()),
        "le nom dans le champ"
    );
    taper(&mut app, "Croquis");
    touche(&mut app, NamedKey::Enter);
    assert!(app.ui.question.is_none(), "la question se referme");
    assert_eq!(
        nom(&app),
        "Croquis",
        "le nom tapé remplace l'ancien, choisi tout entier"
    );
    assert!(app.store.undo());
    assert_eq!(nom(&app), "Dossier");
}

/// **`Échap` ne renomme rien** ; sans dossier choisi, rien ne s'ouvre ; le menu du dossier et la
/// palette mènent au même geste.
#[test]
fn test_renommer_1_echap_sans_dossier_le_menu_et_la_palette() {
    let mut app = app();
    f2(&mut app);
    taper(&mut app, "Autre");
    touche(&mut app, NamedKey::Escape);
    assert!(app.ui.question.is_none());
    assert_eq!(nom(&app), "Dossier", "Échap ne renomme rien");
    assert_eq!(
        MenuAction::RenommerLeDossier.commande(),
        Some(Commande::RenommerLeDossier)
    );
    assert!(app.s_applique(Commande::RenommerLeDossier));
    app.store.selected_folder_id = None;
    assert!(
        !app.s_applique(Commande::RenommerLeDossier),
        "rien à renommer"
    );
    f2(&mut app);
    assert!(
        app.ui.question.is_none(),
        "sans dossier choisi, F2 n'ouvre rien"
    );
}
