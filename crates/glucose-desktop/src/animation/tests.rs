//! Les lois de l'animateur. Le temps y passe vraiment — c'est le seul endroit où il le faut —
//! mais jamais plus que la durée d'une animation courte.

use super::*;
use glucose_core::types::CanvasFolder;
use std::thread::sleep;
use std::time::Duration;

const ECRAN: ScreenSize = ScreenSize { width: 1440.0, height: 900.0 };

/// Un store avec un dossier posé loin de l'origine, pour que le cadrage ait un vrai chemin à
/// parcourir.
fn store_avec_dossier() -> Store {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let mut f = CanvasFolder::new("fold-1", "Recherches", String::new());
    f.x = 4_000.0;
    f.y = 3_000.0;
    f.width = 320.0;
    f.height = 240.0;
    store.create_folder(&board, f);
    store
}

fn viewport(store: &Store, board: &str) -> Viewport {
    store
        .project
        .boards
        .iter()
        .find(|b| b.id == board)
        .map(|b| b.viewport)
        .unwrap_or_default()
}

/// Une animation d'une durée nulle finit à la première image, et exécute ce qui la suit.
/// C'est le cas qui prouve que l'action différée n'est pas perdue.
#[test]
fn test_un_vol_instantane_execute_quand_meme_sa_suite() {
    let mut store = store_avec_dossier();
    let board = store.project.active_board_id.clone();
    let mut a = Animator::new();
    a.fly_to(
        &store,
        &board,
        Viewport { x: 10.0, y: 20.0, scale: 2.0 },
        0,
        Pending::EnterFolder("fold-1".into()),
    );
    assert!(a.is_running());

    assert_eq!(a.tick(&mut store), Some(0), "la première image est la dernière");
    assert!(!a.is_running());
    assert_eq!(viewport(&store, &board), Viewport { x: 10.0, y: 20.0, scale: 2.0 });
    assert_eq!(store.folder_path().len(), 2, "et l'on est bien entré");
}

/// **Pendant le vol, on est encore dans le tableau parent.** La bascule n'a lieu qu'à
/// l'arrivée : c'est ce qui donne la sensation d'entrer plutôt que d'être transporté.
#[test]
fn test_pendant_le_vol_on_est_encore_dans_le_tableau_parent() {
    let mut store = store_avec_dossier();
    let board = store.project.active_board_id.clone();
    let mut a = Animator::new();
    assert!(fly_into_folder(&store, &mut a, "fold-1", ECRAN));

    // Une image au tout début du vol.
    let reste = a.tick(&mut store).expect("le vol court");
    assert!(reste > 0, "il reste du chemin");
    assert_eq!(store.project.active_board_id, board, "toujours dans le parent");
    assert_eq!(store.folder_path().len(), 1);
    assert!(a.is_running());

    // La caméra, elle, a déjà bougé vers le dossier.
    let vp = viewport(&store, &board);
    assert_ne!(vp, Viewport::default(), "la caméra a commencé à plonger");
}

/// Le vol arrive exactement sur le cadrage visé, et y entre. Le temps passe pour de bon ici —
/// un vol de cent millisecondes, pas les quatre cents de la fiche.
#[test]
fn test_le_vol_arrive_sur_son_cadrage_et_entre() {
    let mut store = store_avec_dossier();
    let board = store.project.active_board_id.clone();
    let cible = fit_viewport(
        Rect::new(4_000.0, 3_000.0, 320.0, 240.0),
        ECRAN,
        focus_consts::FIT_PADDING,
    );
    let mut a = Animator::new();
    a.fly_to(&store, &board, cible, 100, Pending::EnterFolder("fold-1".into()));

    let mut images = 0;
    loop {
        let Some(reste) = a.tick(&mut store) else { break };
        images += 1;
        if reste == 0 {
            break;
        }
        sleep(Duration::from_millis(10));
        assert!(images < 200, "le vol doit finir");
    }
    assert!(images > 1, "un vol de 100 ms dure plus d'une image");
    assert_eq!(viewport(&store, &board), cible.normalized(), "arrivée exacte");
    assert_eq!(store.folder_path().len(), 2, "et l'on est entré");
    assert!(!a.is_running());
}

/// **L'échelle s'interpole géométriquement.** À mi-course entre 1 et 100, une interpolation
/// linéaire donnerait 50 ; la bonne donne 10 — la racine du produit. Sans cela, la plongée
/// couvrirait presque tout son chemin dans la première moitié du temps et s'arrêterait net.
#[test]
fn test_l_echelle_s_interpole_geometriquement() {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    store.set_viewport(&board, Viewport { x: 0.0, y: 0.0, scale: 1.0 });
    let mut a = Animator::new();
    // Une courbe linéaire isole l'effet de l'échelle : avec l'amorti, le temps ne serait pas
    // à mi-course au milieu.
    a.flight = Some(Flight {
        from: Viewport { x: 0.0, y: 0.0, scale: 1.0 },
        to: Viewport { x: 0.0, y: 0.0, scale: 100.0 },
        start: Instant::now() - Duration::from_millis(50),
        duration_ms: 100,
        curve: Curve::Linear,
        then: Pending::Nothing,
        board: board.clone(),
    });
    a.tick(&mut store);
    let vu = viewport(&store, &board).scale;
    assert!((vu - 10.0).abs() < 0.5, "à mi-course : {vu} au lieu de 10");
    assert!(vu < 50.0, "et surtout pas 50, qui serait l'interpolation linéaire");
}

/// Un second vol part de **là où la caméra est**, pas de là où le premier avait commencé :
/// deux gestes à la suite ne doivent pas faire sauter l'image.
#[test]
fn test_un_second_vol_part_de_la_ou_la_camera_est() {
    let mut store = store_avec_dossier();
    let board = store.project.active_board_id.clone();
    let mut a = Animator::new();
    a.fly_to(&store, &board, Viewport { x: 1_000.0, y: 0.0, scale: 1.0 }, 400, Pending::Nothing);
    sleep(Duration::from_millis(30));
    a.tick(&mut store);
    let en_route = viewport(&store, &board);
    assert_ne!(en_route.x, 0.0, "la caméra a bougé");

    a.fly_to(&store, &board, Viewport { x: 0.0, y: 0.0, scale: 1.0 }, 400, Pending::Nothing);
    a.tick(&mut store);
    let apres = viewport(&store, &board);
    // Le nouveau vol repart d'où l'on est : au premier instant, la caméra n'a pas sauté.
    assert!(
        (apres.x - en_route.x).abs() < en_route.x.abs().max(1.0),
        "saut de {} à {}",
        en_route.x,
        apres.x
    );
}

/// **Abandonner un vol n'exécute pas sa suite.** Une plongée qu'on interrompt ne doit pas
/// ouvrir le dossier à l'arrivée.
#[test]
fn test_abandonner_un_vol_n_ouvre_rien() {
    let mut store = store_avec_dossier();
    let mut a = Animator::new();
    assert!(fly_into_folder(&store, &mut a, "fold-1", ECRAN));
    a.tick(&mut store);

    assert!(a.cancel());
    assert!(!a.cancel(), "abandonner deux fois ne fait rien");
    assert!(!a.is_running());
    assert_eq!(a.tick(&mut store), None);
    assert_eq!(store.folder_path().len(), 1, "le dossier n'a pas été ouvert");
}

/// Un vol dont le tableau change en route s'arrête : il n'a plus de sujet.
#[test]
fn test_un_vol_dont_le_tableau_change_s_arrete() {
    let mut store = store_avec_dossier();
    let mut a = Animator::new();
    assert!(fly_into_folder(&store, &mut a, "fold-1", ECRAN));
    a.tick(&mut store);

    // Quelqu'un d'autre change de tableau pendant le vol.
    store.try_enter_folder("fold-1").expect("y entrer");
    assert_eq!(a.tick(&mut store), None, "le vol s'est arrêté");
    assert!(!a.is_running());
}

/// Voler vers un dossier qui n'existe pas échoue proprement, sans lancer d'animation.
#[test]
fn test_voler_vers_un_dossier_inexistant_echoue_sans_animer() {
    let store = store_avec_dossier();
    let mut a = Animator::new();
    assert!(!fly_into_folder(&store, &mut a, "jamais-vu", ECRAN));
    assert!(!a.is_running());
}

/// **La remontée est l'inverse de la plongée** : on remonte d'abord, la caméra est posée
/// serrée sur le dossier quitté, puis elle revient au cadrage que le parent avait.
#[test]
fn test_la_remontee_repart_du_dossier_quitte() {
    let mut store = store_avec_dossier();
    let parent = store.project.active_board_id.clone();
    let cadrage_parent = Viewport { x: -120.0, y: 45.0, scale: 0.75 };
    store.set_viewport(&parent, cadrage_parent);
    store.try_enter_folder("fold-1").expect("entrer");
    assert_eq!(store.folder_path().len(), 2);

    let mut a = Animator::new();
    assert!(fly_out_to_depth(&mut store, &mut a, 0, ECRAN));
    assert_eq!(store.project.active_board_id, parent, "on est remonté tout de suite");
    assert!(a.is_running(), "et la caméra, elle, a du chemin à faire");

    // Au départ du vol, la caméra est serrée sur le dossier, pas au cadrage du parent.
    let depart = viewport(&store, &parent);
    assert_ne!(depart, cadrage_parent);
    assert!(depart.scale > cadrage_parent.scale, "serrée, donc plus zoomée");

    // Et elle y revient.
    a.flight.as_mut().expect("le vol").duration_ms = 0;
    a.tick(&mut store);
    assert_eq!(viewport(&store, &parent), cadrage_parent.normalized());
}

/// Remonter là où l'on est déjà ne fait rien, et n'anime rien.
#[test]
fn test_remonter_la_ou_on_est_n_anime_rien() {
    let mut store = store_avec_dossier();
    let mut a = Animator::new();
    assert!(!fly_out_to_depth(&mut store, &mut a, 0, ECRAN));
    assert!(!a.is_running());
}

/// Sans animation en cours, l'animateur ne demande jamais de frame.
#[test]
fn test_sans_animation_rien_n_est_demande() {
    let mut store = Store::new("Projet");
    let mut a = Animator::new();
    assert!(!a.is_running());
    assert_eq!(a.tick(&mut store), None);
}
