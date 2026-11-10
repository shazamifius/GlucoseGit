//! **LIEN-2 — `Ctrl`+clic suit une flèche jusqu'à son autre bout**, par la vraie souris ; `Alt+Z`
//! ramène ; au doigt, le menu ; à la palette, la commande.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use glucose_core::types::{Annotation, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::ModifiersState;

const ECRAN: (f32, f32) = (1200.0, 800.0);

/// « a » à l'origine, « b » loin en bas à droite, une flèche de l'un à l'autre ; la vue au
/// dixième, sur les deux.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    for (id, x, y) in [("a", 0.0, 0.0), ("b", 5000.0, 3000.0)] {
        app.store
            .add_annotation(&board, Annotation::text(id, x, y, "un bloc"));
    }
    let mut lien = Annotation::arrow("f", 0.0, 0.0, 5000.0, 3000.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut lien
    {
        *source_id = Some("a".to_string());
        *target_id = Some("b".to_string());
    }
    app.store.add_annotation(&board, lien);
    app.store.clear_selection();
    app.store.set_viewport(&board, depart());
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app
}

fn depart() -> Viewport {
    Viewport {
        x: 100.0,
        y: 100.0,
        scale: 0.1,
    }
}

/// Le point du trait de la flèche à la fraction `t` de son départ, en pixels d'écran.
fn sur_la_fleche(app: &GlucoseApp, t: f64) -> (f64, f64) {
    let board = app.store.active_board().expect("un tableau");
    let ann = board.annotations.iter().find(|a| a.id() == "f").expect("f");
    let noeuds = crate::renderer::arrow::NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: None,
    };
    let chemin = glucose_core::arrow::path_with(ann, noeuds).expect("un chemin");
    let (a, b) = (chemin[0], chemin[chemin.len() - 1]);
    let monde = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    crate::canvas::world_to_screen(monde.0, monde.1, &app.store.viewport())
}

fn cliquer(app: &mut GlucoseApp, ecran: (f64, f64), modificateurs: ModifiersState) {
    app.modifiers = modificateurs;
    app.handle_cursor_moved(PhysicalPosition::new(ecran.0, ecran.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
    app.modifiers = ModifiersState::empty();
}

/// Une image comme la boucle la joue, une période d'écran à 240 Hz après la précédente.
fn image(app: &mut GlucoseApp) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    app.horloge.presentee(t + periode, true);
    let (l, h) = app.taille_de_la_fenetre();
    app.appliquer_l_elan(l as u32, h as u32);
}

/// Le vol jusqu'au bout, image par image.
fn arriver(app: &mut GlucoseApp) {
    for _ in 0..4_000 {
        image(app);
        if !app.vol.en_cours() {
            return;
        }
    }
    panic!("le vol n'arrive pas");
}

/// Ce que la vue montre au centre de l'écran, en coordonnées du monde.
fn au_centre(app: &GlucoseApp) -> (f64, f64) {
    let (l, h) = app.taille_de_la_fenetre();
    let bandeau = f64::from(app.ui.header_height());
    let milieu = (f64::from(h) - bandeau) / 2.0 + bandeau;
    crate::canvas::screen_to_world(f64::from(l) / 2.0, milieu, &app.store.viewport())
}

/// Le centre de la boîte de ce nœud.
fn centre_de(app: &GlucoseApp, id: &str) -> (f64, f64) {
    let board = app.store.active_board().expect("un tableau");
    let b = glucose_core::arrow::emprise(board, id).expect("une boîte");
    (b.left + b.width / 2.0, b.top + b.height / 2.0)
}

fn proche(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).hypot(a.1 - b.1) < 1.0
}

/// **`Ctrl`+clic près de « a » mène à « b »**, à sa taille de lecture ; la flèche n'est pas
/// choisie (le clic est pris par le geste) ; **`Alt+Z` ramène** exactement où l'on était.
#[test]
fn test_lien_2_ctrl_clic_suit_la_fleche_jusqu_a_son_autre_bout() {
    let mut app = application();
    let point = sur_la_fleche(&app, 0.2);
    cliquer(&mut app, point, ModifiersState::CONTROL);
    assert!(app.vol.en_cours(), "la vue vole");
    assert!(
        app.store.selected_annotation_ids.is_empty(),
        "rien de choisi"
    );
    arriver(&mut app);
    assert!(
        proche(au_centre(&app), centre_de(&app, "b")),
        "on est sur « b »"
    );
    assert_eq!(app.store.viewport().scale, 1.0, "à sa taille de lecture");
    assert!(app.executer(Commande::VuePrecedente), "un lieu où revenir");
    assert_eq!(app.store.viewport(), depart(), "Alt+Z ramène");
}

/// **Près de « b », il mène à « a »** : le bout où l'on n'est pas.
#[test]
fn test_lien_2_le_bout_vise_est_celui_ou_l_on_n_est_pas() {
    let mut app = application();
    let point = sur_la_fleche(&app, 0.8);
    cliquer(&mut app, point, ModifiersState::CONTROL);
    arriver(&mut app);
    assert!(
        proche(au_centre(&app), centre_de(&app, "a")),
        "on est sur « a »"
    );
}

/// **Sans `Ctrl`, le clic choisit la flèche, comme avant** : la vue ne bouge pas.
#[test]
fn test_lien_2_sans_ctrl_le_clic_choisit_la_fleche() {
    let mut app = application();
    let point = sur_la_fleche(&app, 0.2);
    cliquer(&mut app, point, ModifiersState::empty());
    assert!(!app.vol.en_cours(), "la vue ne vole pas");
    assert_eq!(
        app.store.selected_annotation_ids.to_vec(),
        vec!["f".to_string()]
    );
}

/// **Au doigt et à la palette** : « Suivre le lien » est au menu d'une flèche choisie seule, et
/// seulement là ; la commande suit la flèche choisie depuis le centre de l'écran.
#[test]
fn test_lien_2_le_menu_et_la_commande_suivent_la_fleche_choisie() {
    use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
    let mut app = application();
    let actions = |app: &GlucoseApp| -> Vec<MenuAction> {
        let clavier = crate::commandes::clavier_de_glucose().0;
        layout_context_menu(
            &app.store,
            (&app.renderer.typography, &clavier),
            ((100.0, 100.0), None),
            ECRAN,
            1.0,
            false,
        )
        .map(|m| {
            m.rows
                .iter()
                .filter_map(|r| match r {
                    MenuRow::Item { action, .. } => Some(*action),
                    MenuRow::Separator { .. } => None,
                })
                .collect()
        })
        .unwrap_or_default()
    };
    assert!(
        !app.executer(Commande::SuivreLeLien),
        "rien de choisi : sans objet"
    );
    app.store.select_annotation("a".to_string(), false);
    assert!(
        !actions(&app).contains(&MenuAction::SuivreLeLien),
        "un texte"
    );
    assert!(
        !app.executer(Commande::SuivreLeLien),
        "un texte : sans objet"
    );
    app.store.select_annotation("f".to_string(), true);
    assert!(
        !actions(&app).contains(&MenuAction::SuivreLeLien),
        "un texte et la flèche"
    );
    assert!(
        !app.executer(Commande::SuivreLeLien),
        "un texte et la flèche : sans objet"
    );
    app.store.select_annotation("f".to_string(), false);
    assert!(
        actions(&app).contains(&MenuAction::SuivreLeLien),
        "la flèche"
    );
    // La vue posée sur « b » : la commande mène à « a », le bout le plus loin du centre.
    let board = app.store.project.active_board_id.clone();
    let (cx, cy) = centre_de(&app, "b");
    let (l, h) = app.taille_de_la_fenetre();
    app.store.set_viewport(
        &board,
        Viewport {
            x: f64::from(l) / 2.0 - cx,
            y: f64::from(h) / 2.0 - cy,
            scale: 1.0,
        },
    );
    assert!(app.executer(Commande::SuivreLeLien));
    arriver(&mut app);
    assert!(
        proche(au_centre(&app), centre_de(&app, "a")),
        "on est sur « a »"
    );
}

/// **Une flèche qui ne touche aucun nœud se cadre elle-même** : il n'y a pas d'autre bout à
/// rejoindre, et le geste ne reste pas sans effet.
#[test]
fn test_lien_2_une_fleche_libre_se_cadre_elle_meme() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &board,
        Annotation::arrow("libre", 0.0, 2000.0, 3000.0, 2000.0),
    );
    app.store.clear_selection();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let point = crate::canvas::world_to_screen(1000.0, 2000.0, &app.store.viewport());
    cliquer(&mut app, point, ModifiersState::CONTROL);
    assert!(app.vol.en_cours(), "la vue vole");
    arriver(&mut app);
    assert!(
        proche(au_centre(&app), centre_de(&app, "libre")),
        "sur la flèche"
    );
}
