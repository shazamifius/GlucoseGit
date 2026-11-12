//! **LIEN-3 — la fenêtre des liens, par la vraie main** : `B` l'ouvre ; un clic sur un nœud y
//! mène et `Alt+Z` ramène ; glisser y déplace la vue ; `Maj`+clic la refait autour d'un nœud ;
//! `Échap` et tout autre geste la ferment, sans jamais toucher le document ; le menu.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
use glucose_core::types::{Annotation, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState, NamedKey};

const ECRAN: (f32, f32) = (1200.0, 800.0);

/// « eau » dans la membrane « Chimie », lié à « jeu », très loin dans « Jeux vidéo » ; « seul »
/// n'a pas de lien. La vue sur « eau », à la moitié de la taille réelle.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let b = app.store.project.active_board_id.clone();
    if let Some(board) = app.store.active_board_mut() {
        board.annotations.clear();
    }
    let membrane = |id: &str, x: f64, y: f64, titre: &str| {
        let mut m = Annotation::membrane(id, x, y, 1200.0, 800.0);
        if let Annotation::Membrane { text, .. } = &mut m {
            *text = Some(titre.to_string());
        }
        m
    };
    let texte = |id: &str, x: f64, y: f64, dans: Option<&str>| {
        let mut t = Annotation::text(id, x, y, format!("le texte {id}"));
        if let Annotation::Text { width, height, .. } = &mut t {
            (*width, *height) = (Some(300.0), Some(100.0));
        }
        t.set_membrane_id(dans.map(str::to_string));
        t
    };
    for a in [
        membrane("chimie", 0.0, 0.0, "Chimie"),
        membrane("jeux", 40_000.0, 20_000.0, "Jeux vidéo"),
        texte("eau", 400.0, 300.0, Some("chimie")),
        texte("jeu", 40_400.0, 20_300.0, Some("jeux")),
        texte("seul", -3_000.0, 0.0, None),
    ] {
        app.store.add_annotation(&b, a);
    }
    let mut lien = Annotation::arrow("f", 0.0, 0.0, 1.0, 1.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut lien
    {
        (*source_id, *target_id) = (Some("eau".into()), Some("jeu".into()));
    }
    app.store.add_annotation(&b, lien);
    app.store.clear_selection();
    app.store.set_viewport(&b, depart());
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app
}

fn depart() -> Viewport {
    Viewport {
        x: 300.0,
        y: 200.0,
        scale: 0.5,
    }
}

/// Une touche, par le chemin des raccourcis.
fn touche(app: &mut GlucoseApp, k: Key) {
    if !app.prendre_la_touche(&k, true, None) {
        app.handle_shortcut_input(&k, ElementState::Pressed);
    }
}

fn touche_b(app: &mut GlucoseApp) {
    touche(app, Key::Character("b".into()));
}

fn ouvrir_sur_eau(app: &mut GlucoseApp) {
    app.store.select_annotation("eau".to_string(), false);
    touche_b(app);
    assert!(
        app.store.fenetre_des_liens().is_some(),
        "B ouvre la fenêtre"
    );
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
}

/// Où la fenêtre montre ce nœud du document, à l'écran.
fn a_l_ecran(app: &GlucoseApp, original: &str) -> (f64, f64) {
    let f = app.store.fenetre_des_liens().expect("ouverte");
    let copie = f
        .tableau
        .annotations
        .iter()
        .find(|a| f.original(a.id()) == Some(original) && matches!(a, Annotation::Text { .. }))
        .expect("la copie");
    let b = copie.rect().expect("une boîte");
    let vp = app.store.viewport();
    crate::canvas::world_to_screen(b.left + b.width / 2.0, b.top + b.height / 2.0, &vp)
}

fn appuyer(app: &mut GlucoseApp, ici: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(ici.0, ici.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
}

fn relacher(app: &mut GlucoseApp, ici: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(ici.0, ici.1));
    app.handle_mouse_up(MouseButton::Left);
}

/// Une image comme la boucle la joue, une période d'écran à 240 Hz après la précédente : c'est
/// elle qui pose l'élan de la main et fait avancer un vol.
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

/// **`B` ouvre la fenêtre du nœud choisi** : le tableau qu'on regarde est la fenêtre, « jeu » y
/// est amené près de « eau » ; sur un nœud sans lien, `B` ne fait rien.
#[test]
fn test_lien_3_b_ouvre_la_fenetre_du_noeud_choisi() {
    let mut app = application();
    app.store.select_annotation("seul".to_string(), false);
    touche_b(&mut app);
    assert!(app.store.fenetre_des_liens().is_none(), "sans lien, rien");
    ouvrir_sur_eau(&mut app);
    let (eau, jeu) = (a_l_ecran(&app, "eau"), a_l_ecran(&app, "jeu"));
    assert!(
        (jeu.0 - eau.0).hypot(jeu.1 - eau.1) < 600.0,
        "« jeu » amené près : {jeu:?}"
    );
    assert!(app.store.selected_annotation_ids.is_empty());
}

/// **Un clic sur un nœud amené y mène** — la fenêtre fermée, la vue vole vers sa vraie place ;
/// **`Alt+Z` ramène** là où l'on regardait la fenêtre.
#[test]
fn test_lien_3_un_clic_sur_un_noeud_y_mene() {
    let mut app = application();
    ouvrir_sur_eau(&mut app);
    let jeu = a_l_ecran(&app, "jeu");
    appuyer(&mut app, jeu);
    assert!(
        app.store.fenetre_des_liens().is_some(),
        "l'appui seul n'y va pas"
    );
    relacher(&mut app, jeu);
    assert!(app.store.fenetre_des_liens().is_none(), "fermée");
    arriver(&mut app);
    let vp = app.store.viewport();
    let (l, _) = app.taille_de_la_fenetre();
    let centre = crate::canvas::screen_to_world(f64::from(l) / 2.0, 400.0, &vp);
    assert!(
        (centre.0 - 40_550.0).abs() < 1.0,
        "sur la vraie place de « jeu » : {centre:?}"
    );
    assert!(app.executer(Commande::VuePrecedente));
    assert_eq!(app.store.viewport(), depart(), "Alt+Z ramène");
}

/// **Glisser déplace la vue**, même parti d'un nœud : on n'y déplace rien, et l'appui qui a
/// bougé ne mène nulle part.
#[test]
fn test_lien_3_glisser_deplace_la_vue_sans_aller() {
    let mut app = application();
    ouvrir_sur_eau(&mut app);
    let jeu = a_l_ecran(&app, "jeu");
    let avant = app.store.viewport();
    appuyer(&mut app, jeu);
    relacher(&mut app, (jeu.0 + 120.0, jeu.1 + 40.0));
    image(&mut app);
    assert!(app.store.fenetre_des_liens().is_some(), "toujours ouverte");
    assert!(!app.vol.en_cours(), "aucun vol");
    let apres = app.store.viewport();
    assert_eq!(
        (apres.x - avant.x, apres.y - avant.y),
        (120.0, 40.0),
        "la vue a suivi"
    );
    assert_eq!(
        app.store.project.boards[0].viewport,
        depart(),
        "le document n'a pas bougé"
    );
}

/// **`Maj`+clic la refait autour du nœud** : le Bring & Go qui s'enchaîne — même juste après
/// l'ouverture, sans image entre les deux (l'index des nœuds se refait avant de viser).
#[test]
fn test_lien_3_maj_clic_la_refait_autour_du_noeud() {
    let mut app = application();
    app.store.select_annotation("eau".to_string(), false);
    assert!(app.executer(Commande::MontrerLesLiens));
    let jeu = a_l_ecran(&app, "jeu");
    app.modifiers = ModifiersState::SHIFT;
    appuyer(&mut app, jeu);
    relacher(&mut app, jeu);
    app.modifiers = ModifiersState::empty();
    let f = app.store.fenetre_des_liens().expect("toujours ouverte");
    assert_eq!(f.centre, "jeu");
}

/// **`Échap` la ferme ; tout autre geste la ferme d'abord**, et ne s'exécute que sur le document :
/// supprimer, fenêtre ouverte et une copie choisie, ne supprime rien.
#[test]
fn test_lien_3_echap_et_tout_autre_geste_la_ferment() {
    let mut app = application();
    let document = app.store.project.clone();
    ouvrir_sur_eau(&mut app);
    touche(&mut app, Key::Named(NamedKey::Escape));
    assert!(app.store.fenetre_des_liens().is_none(), "Échap");
    ouvrir_sur_eau(&mut app);
    let copie = a_l_ecran(&app, "jeu");
    app.handle_cursor_moved(PhysicalPosition::new(copie.0, copie.1));
    app.handle_mouse_down(MouseButton::Right, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Right);
    assert_eq!(app.store.compte_de_la_selection(), 1, "la copie choisie");
    app.ui.context_menu_at = None;
    touche(&mut app, Key::Named(NamedKey::Delete));
    assert!(
        app.store.fenetre_des_liens().is_none(),
        "Suppr la ferme d'abord"
    );
    assert_eq!(app.store.project, document, "rien de supprimé");
    ouvrir_sur_eau(&mut app);
    touche(&mut app, Key::Character("t".into()));
    assert!(app.store.fenetre_des_liens().is_none(), "un outil la ferme");
}

/// Les entrées du menu, tel qu'il s'ouvrirait.
fn menu(app: &GlucoseApp) -> Vec<MenuAction> {
    layout_context_menu(
        &app.store,
        (&app.renderer.typography, &app.ui),
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
}

/// **Le menu** : « Montrer ses liens » sur un nœud qui en a, et pas sur un autre ; dans la
/// fenêtre, sur une copie choisie, aller, montrer, fermer — et rien qui change le document ; aller
/// par le menu mène au vrai nœud.
#[test]
fn test_lien_3_le_menu_ouvre_mene_et_ferme() {
    let mut app = application();
    app.store.select_annotation("seul".to_string(), false);
    assert!(
        !menu(&app).contains(&MenuAction::MontrerLesLiens),
        "sans lien"
    );
    app.store.select_annotation("eau".to_string(), false);
    assert!(
        menu(&app).contains(&MenuAction::MontrerLesLiens),
        "avec un lien"
    );
    assert!(app.executer(Commande::MontrerLesLiens));
    assert_eq!(
        menu(&app),
        vec![MenuAction::FermerLaFenetre],
        "rien de choisi"
    );
    let copie = a_l_ecran(&app, "jeu");
    app.handle_cursor_moved(PhysicalPosition::new(copie.0, copie.1));
    app.handle_mouse_down(MouseButton::Right, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Right);
    assert_eq!(
        menu(&app),
        vec![
            MenuAction::AllerAuNoeud,
            MenuAction::MontrerLesLiens,
            MenuAction::FermerLaFenetre
        ]
    );
    assert!(
        app.executer(Commande::MontrerLesLiens),
        "au menu, dans la fenêtre"
    );
    assert_eq!(
        app.store.fenetre_des_liens().expect("ouverte").centre,
        "jeu"
    );
    let copie = a_l_ecran(&app, "jeu");
    app.handle_cursor_moved(PhysicalPosition::new(copie.0, copie.1));
    app.handle_mouse_down(MouseButton::Right, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Right);
    assert!(app.executer(Commande::AllerAuNoeud));
    assert!(app.store.fenetre_des_liens().is_none());
    arriver(&mut app);
    let vp = app.store.viewport();
    let (l, _) = app.taille_de_la_fenetre();
    let centre = crate::canvas::screen_to_world(f64::from(l) / 2.0, 400.0, &vp);
    assert!(
        (centre.0 - 40_550.0).abs() < 1.0,
        "sur « jeu » : {centre:?}"
    );
}

/// **Sur une flèche choisie, `B` montre le bout qu'on regarde** — le plus proche du centre de
/// l'écran ; **sur deux nœuds choisis, rien** : on ne montre les liens que d'un seul.
#[test]
fn test_lien_3_b_sur_une_fleche_et_sur_deux_noeuds() {
    let mut app = application();
    app.store.select_annotation("f".to_string(), false);
    touche_b(&mut app);
    assert_eq!(
        app.store.fenetre_des_liens().expect("ouverte").centre,
        "eau"
    );
    touche(&mut app, Key::Named(NamedKey::Escape));
    app.store.select_annotation("eau".to_string(), false);
    app.store.select_annotation("seul".to_string(), true);
    touche_b(&mut app);
    assert!(app.store.fenetre_des_liens().is_none(), "deux nœuds : rien");
}

/// **Dans la fenêtre, les liens sont pleins** : ils sont ce qu'elle montre, à toute distance — là
/// où, sur la carte, ils pâliraient.
#[test]
fn test_lien_3_dans_la_fenetre_les_liens_sont_pleins() {
    let mut app = application();
    ouvrir_sur_eau(&mut app);
    let board = app.store.active_board().expect("la fenêtre");
    let lien = board
        .annotations
        .iter()
        .find(|a| matches!(a, Annotation::Arrow { .. }))
        .expect("un lien copié");
    assert!(crate::renderer::arrow::en_lumiere(&app.store, None, lien));
    touche(&mut app, Key::Named(NamedKey::Escape));
    let board = app.store.active_board().expect("le tableau");
    let lien = board
        .annotations
        .iter()
        .find(|a| a.id() == "f")
        .expect("le lien");
    assert!(
        !crate::renderer::arrow::en_lumiere(&app.store, None, lien),
        "sur la carte, non"
    );
}
