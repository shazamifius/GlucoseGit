//! COULEUR-1 par les vraies entrées de l'application : la souris sur la barre, la commande.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use crate::ui::couleurs::{layout_barre_de_couleur, teinte};
use glucose_core::types::Annotation;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un post-it et une membrane, choisis.
fn app_avec_deux_noeuds() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::sticky("p", 100.0, 100.0, "un post-it"));
    app.store.add_annotation(
        &board,
        Annotation::membrane("m", 100.0, 400.0, 300.0, 200.0),
    );
    app.store
        .set_selected_annotation_ids(vec!["p".into(), "m".into()]);
    app.store.journal.clear();
    app
}

fn couleurs(app: &GlucoseApp) -> (Option<String>, Option<String>) {
    let b = app.store.project.active_board_id.clone();
    let p = match app.store.project.annotation(&b, "p") {
        Some(Annotation::Sticky { bg_color, .. }) => bg_color.clone(),
        _ => None,
    };
    let m = match app.store.project.annotation(&b, "m") {
        Some(Annotation::Membrane { color, .. }) => color.clone(),
        _ => None,
    };
    (p, m)
}

fn clic(app: &mut GlucoseApp, (x, y): (f32, f32)) {
    app.handle_cursor_moved(PhysicalPosition::new(f64::from(x), f64::from(y)));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// **Un clic sur une teinte colore tout ce qui est choisi** — le papier du post-it, la teinte
/// de la membrane —, un `Ctrl+Z` le défait d'un coup, et un clic entre deux cases ne
/// désélectionne rien.
#[test]
fn couleur_1_un_clic_colore_la_selection() {
    let mut app = app_avec_deux_noeuds();
    let b = layout_barre_de_couleur(
        &app.store,
        &app.renderer.typography,
        ECRAN,
        app.ui.scale_factor,
    )
    .expect("la barre");
    let rose = b
        .cases
        .iter()
        .find(|c| c.teinte == Some(2))
        .expect("le rose")
        .rect;
    clic(&mut app, (rose.0 + rose.2 / 2.0, rose.1 + rose.3 / 2.0));
    let attendue = teinte(2).map(str::to_string);
    assert_eq!(couleurs(&app), (attendue.clone(), attendue));
    let premiere = b.cases[0].rect;
    clic(&mut app, (premiere.0 + premiere.2 + 1.0, premiere.1 + 1.0));
    assert_eq!(
        app.store.selected_annotation_ids.len(),
        2,
        "toujours choisis"
    );
    assert!(app.store.undo(), "un geste");
    assert_eq!(couleurs(&app), (None, None), "Ctrl+Z les rend");
}

/// **La commande** : `couleur.ambre` colore, `couleur.par-defaut` rend la couleur d'avant ; sans
/// rien de colorable, elle ne s'applique pas — l'accord passe à la suivante.
#[test]
fn couleur_1_la_commande_colore_et_rend_par_defaut() {
    let mut app = app_avec_deux_noeuds();
    let ambre = Commande::de_l_id("couleur.ambre").expect("la commande");
    assert!(app.executer(ambre));
    assert_eq!(couleurs(&app).0.as_deref(), teinte(3));
    let defaut = Commande::de_l_id("couleur.par-defaut").expect("la commande");
    assert!(app.executer(defaut));
    assert_eq!(couleurs(&app), (None, None));
    app.store.clear_selection();
    assert!(
        !app.executer(ambre),
        "rien de choisi : elle ne s'applique pas"
    );
}
