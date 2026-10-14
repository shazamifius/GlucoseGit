//! **Plusieurs storyboards par tableau** (P2, proposition 8, fiche 68), à la vraie main : « +
//! Nouveau » en pose un autre, ‹ et › passent de l'un à l'autre, une case choisie fait le sien
//! courant, et « Mettre à jour », « + Ajouter », la croix et « ▶ Lire » n'agissent que sur lui.

use crate::app::GlucoseApp;
use crate::dock::storyboard::{layout_storyboard_panel, StoryboardPanelLayout};
use crate::dock::{cadre_du_panneau, TabId, WidgetRect};
use glucose_core::store::storyboard::{Grille, DESCRIPTION};
use glucose_core::types::{StoryboardPanel, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un tableau vu à l'échelle 1, deux storyboards de huit cases l'un sous l'autre — le second
/// posé en dernier, donc courant —, le panneau ouvert et peint.
fn app() -> (GlucoseApp, String, [String; 2]) {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
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
    let grille = Grille {
        largeur: 120.0,
        ..Grille::default()
    };
    let premier = app
        .store
        .poser_le_storyboard(&board, grille, (400.0, 100.0))
        .expect("posé");
    let second = app
        .store
        .poser_le_storyboard(&board, grille, (400.0, 500.0))
        .expect("posé");
    app.ui.storyboard.courant = Some(second.clone());
    app.store.clear_selection();
    ouvrir(&mut app);
    (app, board, [premier, second])
}

/// Le panneau ouvert, et une image peinte : il suit le document.
fn ouvrir(app: &mut GlucoseApp) {
    if !app.dock_manager.is_open(TabId::Storyboard) {
        app.dock_manager.toggle_tab(TabId::Storyboard);
    }
    peindre(app);
}

fn peindre(app: &mut GlucoseApp) {
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
}

/// La mise en page du panneau, telle qu'il est peint.
fn panneau(app: &GlucoseApp) -> StoryboardPanelLayout {
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, TabId::Storyboard).expect("ouvert");
    let s = &app.dock_manager.storyboard;
    layout_storyboard_panel(cadre, s, &s.contenu)
}

/// Un clic au point de l'écran, puis une image peinte.
fn cliquer(app: &mut GlucoseApp, (x, y): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
    peindre(app);
}

fn centre(r: WidgetRect) -> (f64, f64) {
    (f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0))
}

fn ids(app: &GlucoseApp, board: &str, sb: &str) -> Vec<String> {
    app.store
        .cases(board, sb)
        .iter()
        .map(|c| c.id.clone())
        .collect()
}

/// **« + Nouveau » pose un autre storyboard, et le panneau le suit** : le neuf devient le
/// courant — « 3 / 3 » —, ses cases à lui ; les deux d'avant ne bougent pas.
#[test]
fn test_p2_8_nouveau_pose_un_autre_storyboard_que_le_panneau_suit() {
    let (mut app, board, [premier, second]) = app();
    // Une case du courant choisie : le neuf la rendra, elle n'est pas à lui.
    let case = app.store.cases(&board, &second)[1].clone();
    cliquer(&mut app, dans_la_bande(&case));
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    let avant = (ids(&app, &board, &premier), ids(&app, &board, &second));
    let nouveau = panneau(&app).nouveau.expect("« + Nouveau »");
    cliquer(&mut app, centre(nouveau));
    assert!(app.ui.placement.storyboard.is_some(), "la grille est armée");
    assert_eq!(app.ui.placement.cible, None, "pour en faire naître un");
    cliquer(&mut app, (40.0, 820.0));
    let tous = app.store.storyboards(&board);
    assert_eq!(tous.len(), 3);
    let neuf = tous[2].id.clone();
    assert_eq!(app.storyboard_courant(), Some(neuf.clone()));
    assert_eq!(
        app.ui.storyboard.choisie, None,
        "la case de l'autre est rendue"
    );
    ouvrir(&mut app);
    let contenu = &app.dock_manager.storyboard.contenu;
    assert_eq!(contenu.sequences, Some((2, 3)), "« 3 / 3 »");
    let montrees: Vec<String> = contenu.cases.iter().map(|c| c.0.clone()).collect();
    assert_eq!(montrees, ids(&app, &board, &neuf));
    assert_eq!(
        (ids(&app, &board, &premier), ids(&app, &board, &second)),
        avant
    );
}

/// **‹ et › passent d'un storyboard à l'autre, en tournant** ; sans eux, un tableau à un seul
/// storyboard ne les montre pas.
#[test]
fn test_p2_8_les_fleches_changent_de_storyboard() {
    let (mut app, board, [premier, second]) = app();
    assert_eq!(app.dock_manager.storyboard.contenu.sequences, Some((1, 2)));
    let (precedent, suivant, _, _) = panneau(&app).sequences.expect("‹ et ›");
    cliquer(&mut app, centre(precedent));
    assert_eq!(app.storyboard_courant(), Some(premier.clone()));
    assert_eq!(app.dock_manager.storyboard.contenu.sequences, Some((0, 2)));
    cliquer(&mut app, centre(suivant));
    cliquer(&mut app, centre(suivant));
    assert_eq!(
        app.storyboard_courant(),
        Some(premier.clone()),
        "après le dernier, le premier"
    );
    app.store.effacer_le_storyboard(&board, &second);
    peindre(&mut app);
    assert_eq!(app.dock_manager.storyboard.contenu.sequences, None);
    assert!(
        panneau(&app).sequences.is_none(),
        "un seul : pas de flèches"
    );
}

/// Un point de la bande de cette case, sous son cadre, à l'écran (échelle 1 depuis l'origine).
fn dans_la_bande(c: &StoryboardPanel) -> (f64, f64) {
    (c.x + c.width - 30.0, c.y + c.height + DESCRIPTION / 2.0)
}

/// **Choisir une case fait son storyboard courant** : le panneau montre ses cases.
#[test]
fn test_p2_8_choisir_une_case_fait_son_storyboard_courant() {
    let (mut app, board, [premier, _]) = app();
    let case = app.store.cases(&board, &premier)[2].clone();
    cliquer(&mut app, dans_la_bande(&case));
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    assert_eq!(app.storyboard_courant(), Some(premier.clone()));
    let contenu = &app.dock_manager.storyboard.contenu;
    assert_eq!(contenu.sequences, Some((0, 2)));
    let montrees: Vec<String> = contenu.cases.iter().map(|c| c.0.clone()).collect();
    assert_eq!(montrees, ids(&app, &board, &premier));
}

/// **« Mettre à jour » ne range que le courant**, là où l'on clique ; l'autre ne bouge pas.
#[test]
fn test_p2_8_mettre_a_jour_ne_range_que_le_courant() {
    let (mut app, board, [premier, second]) = app();
    let autre: Vec<StoryboardPanel> = app
        .store
        .cases(&board, &premier)
        .into_iter()
        .cloned()
        .collect();
    let bouton = panneau(&app).activate_button;
    cliquer(&mut app, centre(bouton));
    assert_eq!(app.ui.placement.cible, Some(second.clone()));
    cliquer(&mut app, (40.0, 820.0));
    assert_eq!(app.store.storyboards(&board).len(), 2, "aucun de plus");
    let premiere = app.store.cases(&board, &second)[0].clone();
    assert_eq!(
        (premiere.x, premiere.y),
        (40.0, 820.0),
        "rangé sous le clic"
    );
    let apres: Vec<StoryboardPanel> = app
        .store
        .cases(&board, &premier)
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(apres, autre, "l'autre ne bouge pas");
}

/// **« + Ajouter » et la croix n'agissent que sur le courant** ; un `Ctrl+Z` rend ce que la
/// croix a retiré.
#[test]
fn test_p2_8_ajouter_et_effacer_n_agissent_que_sur_le_courant() {
    let (mut app, board, [premier, second]) = app();
    let (_, ajouter) = panneau(&app).liste.expect("la liste");
    cliquer(&mut app, centre(ajouter));
    assert_eq!(app.store.cases(&board, &second).len(), 9);
    assert_eq!(app.store.cases(&board, &premier).len(), 8);
    let croix = panneau(&app).effacer.expect("la croix");
    cliquer(&mut app, centre(croix));
    let restants: Vec<String> = app
        .store
        .storyboards(&board)
        .iter()
        .map(|s| s.id.clone())
        .collect();
    assert_eq!(restants, [premier.as_str()]);
    assert_eq!(
        app.storyboard_courant(),
        Some(premier),
        "le courant retombe"
    );
    assert!(app.store.undo());
    assert_eq!(app.store.cases(&board, &second).len(), 9, "rendu");
}

/// **« ▶ Lire » lit le storyboard de la case**, et lui seul : la lecture passe ses cases, puis
/// finit — sans entrer dans l'autre.
#[test]
fn test_p2_8_la_lecture_ne_lit_que_le_storyboard_de_sa_case() {
    let (mut app, board, [premier, _]) = app();
    let case = app.store.cases(&board, &premier)[6].clone();
    app.lire_le_storyboard(Some(&case.id));
    let lecture = app.ui.lecture.as_ref().expect("une lecture");
    assert_eq!(
        (lecture.storyboard.as_str(), lecture.rang),
        (premier.as_str(), 6)
    );
    app.passer_la_case(1);
    assert_eq!(app.ui.lecture.as_ref().map(|l| l.rang), Some(7));
    app.passer_la_case(1);
    assert!(
        app.ui.lecture.is_none(),
        "après sa dernière, la lecture finit"
    );
}
