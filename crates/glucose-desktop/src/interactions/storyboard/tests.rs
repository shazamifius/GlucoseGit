//! STORYBOARD-1 — les cases à la vraie souris : par leur bande, avec ce qu'elles portent ; le
//! cadre, qui reste au canevas ; la description, la croix, `Suppr`, `Échap`.

use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::ui::question::{Reponse, Suite};
use glucose_core::store::storyboard::{Format, Grille, DESCRIPTION};
use glucose_core::types::{BoardImage, StoryboardPanel, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn grille() -> Grille {
    Grille {
        format: Format::Cinema,
        largeur: 320.0,
        colonnes: 4,
        ecart: 24.0,
    }
}

/// Une application vue à l'échelle 1 depuis l'origine — un point de l'écran est le même point
/// du monde —, et un storyboard de huit cases centré en (720, 450).
fn app() -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    app.store
        .poser_le_storyboard(&board, grille(), (720.0, 450.0));
    app.store.clear_selection();
    app.store.journal.clear();
    (app, board)
}

fn premiere(app: &GlucoseApp, board: &str) -> StoryboardPanel {
    app.store.cases(board)[0].clone()
}

/// Un point de la bande de cette case, sous son cadre.
fn dans_la_bande(c: &StoryboardPanel) -> (f64, f64) {
    (c.x + c.width - 60.0, c.y + c.height + DESCRIPTION / 2.0)
}

/// Appuie en `de`, glisse jusqu'à `a`, relâche.
fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.mouse_pos = de;
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

/// **La bande prend la case, et la case emporte son image** : un seul geste, qu'un seul
/// `Ctrl+Z` défait — l'image revient avec elle.
#[test]
fn test_storyboard_1_la_bande_glisse_la_case_et_son_image() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let centre = (case.x + case.width / 2.0, case.y + case.height / 2.0);
    app.store.add_image(
        &board,
        BoardImage::new("i", centre.0, centre.1, 200.0, 200.0),
    );
    // Ajustée dans sa case, comme toute image qu'on y lâche, et non choisie : les poignées d'un
    // nœud choisi passent avant tout, et celles de ses coins mordent sur la bande.
    app.store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    app.store.clear_selection();
    app.store.journal.clear();
    let de = dans_la_bande(&case);
    glisser(&mut app, de, (de.0 + 60.0, de.1 + 50.0));
    let apres = premiere(&app, &board);
    assert_eq!((apres.x, apres.y), (case.x + 60.0, case.y + 50.0));
    let image = app.store.image(&board, "i").expect("l'image");
    assert_eq!(
        (image.x, image.y),
        (centre.0 + 60.0, centre.1 + 50.0),
        "elle suit"
    );
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    assert!(app.store.undo());
    assert_eq!(premiere(&app, &board).x, case.x);
    assert_eq!(app.store.image(&board, "i").map(|i| i.x), Some(centre.0));
    assert!(!app.store.undo(), "un seul geste");
}

/// **Le cadre reste au canevas** : glisser dans une case vide ne la prend pas et ne la choisit
/// pas — c'est le rectangle de sélection, comme partout.
#[test]
fn test_storyboard_1_le_cadre_reste_au_canevas() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let dedans = (case.x + 40.0, case.y + 40.0);
    glisser(&mut app, dedans, (dedans.0 + 50.0, dedans.1 + 30.0));
    assert_eq!(premiere(&app, &board), case);
    assert_eq!(app.ui.storyboard.choisie, None);
}

/// **Une image lâchée dans une case s'y ajuste** — contenue, centrée — dans le geste même du
/// glisser : un `Ctrl+Z` la rend là d'où elle venait, à sa taille.
#[test]
fn test_storyboard_1_une_image_lachee_dans_une_case_s_y_ajuste() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    app.store
        .add_image(&board, BoardImage::new("i", 1200.0, 820.0, 300.0, 300.0));
    app.store.clear_selection();
    app.store.journal.clear();
    let cible = (case.x + 30.0, case.y + 30.0);
    glisser(&mut app, (1200.0, 820.0), cible);
    let image = app.store.image(&board, "i").expect("l'image").clone();
    assert_eq!(
        (image.x, image.y),
        (case.x + case.width / 2.0, case.y + case.height / 2.0)
    );
    assert_eq!(image.height, case.height, "contenue par la hauteur");
    assert!(app.store.undo());
    let rendue = app.store.image(&board, "i").expect("l'image");
    assert_eq!((rendue.x, rendue.y, rendue.width), (1200.0, 820.0, 300.0));
}

/// **Deux clics sur la bande décrivent la case** : une question, un champ tout sélectionné ;
/// ce qu'on y écrit devient la description, en un geste.
#[test]
fn test_storyboard_1_deux_clics_decrivent_la_case() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let point = dans_la_bande(&case);
    for _ in 0..2 {
        glisser(&mut app, point, point);
    }
    let Some((question, Suite::DecrireLaCase { case: id, .. })) = &mut app.ui.question else {
        panic!("la description se demande");
    };
    assert_eq!(id, &case.id);
    let champ = question.champ.as_mut().expect("un champ");
    champ.texte = "Plan large, la ville au loin".into();
    repondre(&mut app, Reponse::Oui);
    assert_eq!(
        premiere(&app, &board).description,
        "Plan large, la ville au loin"
    );
    assert!(app.store.undo());
    assert_eq!(premiere(&app, &board).description, "");
}

/// **La croix et `Suppr` retirent la case choisie**, et les suivantes se renumérotent.
#[test]
fn test_storyboard_1_la_croix_et_suppr_retirent_la_case() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let point = dans_la_bande(&case);
    glisser(&mut app, point, point);
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    let vp = app.store.viewport();
    let boite = crate::renderer::storyboard::boite_a_l_ecran(&case, &vp);
    let bande = DESCRIPTION as f32 * vp.scale as f32;
    let (cx, cy, cw, ch) = crate::renderer::storyboard::croix(boite, bande, app.ui.scale());
    let croix = (f64::from(cx + cw / 2.0), f64::from(cy + ch / 2.0));
    glisser(&mut app, croix, croix);
    let cases = app.store.cases(&board);
    assert_eq!(cases.len(), 7, "la croix l'a retirée");
    assert_eq!(cases[0].order, 0, "la suivante prend son numéro");

    let seconde = cases[0].clone();
    let point = dans_la_bande(&seconde);
    glisser(&mut app, point, point);
    app.handle_shortcut_input(&Key::Named(NamedKey::Delete), ElementState::Pressed);
    assert_eq!(app.store.cases(&board).len(), 6, "Suppr aussi");
}

/// **`Échap` renonce à la grille armée** ; armée, elle montre une boîte par case.
#[test]
fn test_storyboard_1_echap_renonce_a_la_grille_armee() {
    let (mut app, board) = app();
    app.store.effacer_le_storyboard(&board);
    app.ui.placement.storyboard = Some(grille());
    assert_eq!(app.fantome_du_storyboard().len(), 8);
    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    assert!(app.ui.placement.storyboard.is_none());
    assert!(app.fantome_du_storyboard().is_empty());
}
