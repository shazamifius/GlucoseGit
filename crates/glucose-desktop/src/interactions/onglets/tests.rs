//! Les gestes des onglets, par les vraies entrées de l'application : la souris, le clavier.

use crate::app::GlucoseApp;
use crate::ui::onglets::TabButtonLayout;
use glucose_core::types::CanvasFolder;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application à trois onglets : A (actif), B, C.
fn trois_onglets() -> (GlucoseApp, [String; 3]) {
    let mut app = GlucoseApp::new();
    let a = app.store.project.active_board_id.clone();
    app.store.rename_board(&a, "A");
    let b = app.store.add_board("B");
    let c = app.store.add_board("C");
    app.store.set_active_board_id(a.clone());
    app.store.journal.clear();
    (app, [a, b, c])
}

fn onglets(app: &GlucoseApp) -> Vec<TabButtonLayout> {
    crate::ui::layout_tabs(&app.store, &app.ui, &app.renderer.typography)
}

fn centre_de(app: &GlucoseApp, id: &str) -> (f64, f64) {
    let t = onglets(app)
        .into_iter()
        .find(|t| t.board_id == id)
        .expect("l'onglet est là");
    (
        f64::from(t.x + t.width / 3.0),
        f64::from(t.y + t.height / 2.0),
    )
}

fn aller(app: &mut GlucoseApp, p: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
}

fn clic(app: &mut GlucoseApp, p: (f64, f64)) {
    aller(app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        assert!(app.frapper_le_nom_de_l_onglet(&Key::Character(c.to_string().into())));
    }
}

fn noms(app: &GlucoseApp) -> Vec<String> {
    app.store.onglets().map(|(_, n, _)| n.to_string()).collect()
}

/// **Un double-clic ouvre le renommage** ; ce qu'on tape, puis `Entrée`, renomme — un geste,
/// que `Ctrl+Z` défait.
#[test]
fn test_un_double_clic_renomme_l_onglet() {
    let (mut app, [_, b, _]) = trois_onglets();
    let p = centre_de(&app, &b);
    clic(&mut app, p);
    assert!(
        app.ui.onglets.renomme.is_none(),
        "un simple clic choisit, sans renommer"
    );
    assert_eq!(app.store.project.active_board_id, b);
    clic(&mut app, p);
    assert!(
        app.ui.onglets.renomme.is_some(),
        "le second clic ouvre le champ"
    );
    for _ in 0.."B".len() {
        app.frapper_le_nom_de_l_onglet(&Key::Named(NamedKey::Backspace));
    }
    taper(&mut app, "Références");
    assert!(app.frapper_le_nom_de_l_onglet(&Key::Named(NamedKey::Enter)));
    assert_eq!(noms(&app), ["A", "Références", "C"]);
    assert!(app.store.undo(), "un geste");
    assert_eq!(noms(&app), ["A", "B", "C"]);
}

/// **`Échap` renonce ; un clic ailleurs valide** — le champ de Glucose Tauri validait en
/// perdant le focus.
#[test]
fn test_echap_renonce_et_un_clic_ailleurs_valide() {
    let (mut app, [a, b, _]) = trois_onglets();
    app.commencer_le_renommage(&b);
    taper(&mut app, " bis");
    assert!(app.frapper_le_nom_de_l_onglet(&Key::Named(NamedKey::Escape)));
    assert_eq!(noms(&app), ["A", "B", "C"]);

    app.commencer_le_renommage(&b);
    taper(&mut app, " bis");
    let ailleurs = centre_de(&app, &a);
    clic(&mut app, ailleurs);
    assert!(app.ui.onglets.renomme.is_none());
    assert_eq!(noms(&app), ["A", "B bis", "C"]);
}

/// **Glisser un onglet le range** : vers la droite il passe après celui qu'il couvre, vers la
/// gauche avant, et au bout sur le « + ». Un clic sans glisser ne range rien.
#[test]
fn test_glisser_un_onglet_le_range() {
    let (mut app, [a, _, c]) = trois_onglets();
    let glisser = |app: &mut GlucoseApp, de: &str, vers: (f64, f64)| {
        let depart = centre_de(app, de);
        aller(app, depart);
        app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
        aller(app, ((depart.0 + vers.0) / 2.0, depart.1));
        aller(app, vers);
        app.handle_mouse_up(MouseButton::Left);
    };
    let vers_c = centre_de(&app, &c);
    glisser(&mut app, &a, vers_c);
    assert_eq!(noms(&app), ["B", "C", "A"], "A passe après C");
    let premier = app.store.onglets().next().expect("B").0.to_string();
    let vers_b = centre_de(&app, &premier);
    glisser(&mut app, &a, vers_b);
    assert_eq!(noms(&app), ["A", "B", "C"], "A revient devant B");
    let plus = onglets(&app)
        .last()
        .map(|t| (f64::from(t.x + 5.0), f64::from(t.y + 5.0)));
    glisser(&mut app, &a, plus.expect("le +"));
    assert_eq!(noms(&app), ["B", "C", "A"], "sur le +, au bout");
    let sur_c = centre_de(&app, &c);
    clic(&mut app, sur_c);
    assert_eq!(noms(&app), ["B", "C", "A"], "un clic ne range rien");
}

/// **La croix supprime l'onglet** ; `Ctrl+Z` le rend. Le dernier onglet n'a pas de croix.
#[test]
fn test_la_croix_supprime_et_ctrl_z_rend() {
    let (mut app, [_, b, _]) = trois_onglets();
    let (cx, cy, cote) = onglets(&app)
        .into_iter()
        .find(|t| t.board_id == b)
        .and_then(|t| t.fermer)
        .expect("une croix");
    clic(
        &mut app,
        (f64::from(cx + cote / 2.0), f64::from(cy + cote / 2.0)),
    );
    assert_eq!(noms(&app), ["A", "C"]);
    assert!(app.ui.toast_message().is_some_and(|m| m.contains("Ctrl+Z")));
    assert!(app.store.undo());
    assert_eq!(noms(&app), ["A", "B", "C"]);

    let mut seul = GlucoseApp::new();
    assert!(
        onglets(&seul).iter().all(|t| t.fermer.is_none()),
        "le dernier reste"
    );
    let _ = &mut seul;
}

/// **Le clic droit sur un onglet ouvre son menu** : renommer, supprimer, un board de plus.
#[test]
fn test_le_clic_droit_sur_un_onglet_ouvre_son_menu() {
    let (mut app, [_, b, _]) = trois_onglets();
    let p = centre_de(&app, &b);
    aller(&mut app, p);
    app.handle_mouse_down(MouseButton::Right, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Right);
    assert_eq!(app.ui.onglets.menu.as_deref(), Some(b.as_str()));
    let menu = crate::ui::context_menu::layout_context_menu(
        &app.store,
        &app.renderer.typography,
        (
            app.ui.context_menu_at.expect("un menu"),
            app.ui.onglets.menu.as_deref(),
        ),
        ECRAN,
        app.ui.scale_factor,
    )
    .expect("un menu d'onglet");
    let renommer = menu
        .rows
        .iter()
        .find_map(|r| match r {
            crate::ui::context_menu::MenuRow::Item { label, rect, .. } if *label == "Renommer" => {
                Some(*rect)
            }
            _ => None,
        })
        .expect("« Renommer » y est");
    clic(
        &mut app,
        (f64::from(renommer.0 + 5.0), f64::from(renommer.1 + 5.0)),
    );
    assert_eq!(
        app.ui.onglets.renomme.as_ref().map(|(id, _)| id.as_str()),
        Some(b.as_str())
    );
}

/// **Le contenu d'un dossier n'est pas un onglet**, et « + » numérote d'après les onglets.
#[test]
fn test_un_dossier_n_ajoute_pas_d_onglet() {
    let (mut app, [a, _, _]) = trois_onglets();
    app.store
        .create_folder(&a, CanvasFolder::new("f", "Dossier", ""));
    assert_eq!(noms(&app), ["A", "B", "C"]);
    app.ajouter_un_onglet();
    assert_eq!(noms(&app), ["A", "B", "C", "Board 4"]);
}
