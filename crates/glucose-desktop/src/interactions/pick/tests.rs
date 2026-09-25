//! PICK-1 § 3.3 — le cycle de profondeur, joué sur le **vrai** `GlucoseApp`, par le chemin de
//! la souris.
//!
//! Ces tests ne dorment pas. L'origine du temps des clics est un champ de l'application
//! ([`GlucoseApp::click_epoch`]) : les reculer d'une seconde vieillit le clic précédent
//! exactement comme une seconde d'attente, et le résultat ne dépend plus de la charge de la
//! machine.

use super::*;
use crate::canvas::world_to_screen;
use glucose_core::hit_priority::pick_consts;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const SCREEN: (f32, f32) = (1440.0, 900.0);

/// Deux membranes empilées au même endroit, la seconde par-dessus la première.
fn app_with_stack() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app.store.add_annotation(
        &board,
        Annotation::membrane("dessous", 0.0, 0.0, 400.0, 300.0),
    );
    app.store.add_annotation(
        &board,
        Annotation::membrane("dessus", 20.0, 20.0, 360.0, 260.0),
    );
    app.store.clear_selection();
    app
}

/// Le centre commun des deux membranes, en coordonnées écran.
fn centre(app: &GlucoseApp) -> (f64, f64) {
    let vp = app.store.viewport();
    world_to_screen(200.0, 150.0, &vp)
}

fn click_at(app: &mut GlucoseApp, (sx, sy): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// Fait vieillir le dernier clic de `ms`, sans attendre.
fn plus_tard(app: &mut GlucoseApp, ms: u64) {
    app.click_epoch -= Duration::from_millis(ms);
}

fn selection(app: &GlucoseApp) -> Vec<String> {
    app.store.selected_annotation_ids.to_vec()
}

/// L'espacement d'un re-clic : au-delà de la fenêtre du double-clic, en deçà du TTL du cycle.
const RECLIC_MS: u64 = (pick_consts::DBLCLICK_MS as u64 + pick_consts::CYCLE_TTL_MS as u64) / 2;

#[test]
fn test_pick_1_a_slow_re_click_at_the_same_spot_reaches_the_node_underneath() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "le premier clic prend le sommet"
    );

    plus_tard(&mut app, RECLIC_MS);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessous"],
        "le re-clic descend d'un cran"
    );

    // Le cycle est circulaire : au bout de la pile, il remonte au sommet.
    plus_tard(&mut app, RECLIC_MS);
    click_at(&mut app, at);
    assert_eq!(selection(&app), vec!["dessus"], "et il boucle");
}

#[test]
fn test_pick_1_a_double_click_does_not_advance_the_cycle() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    // Deux clics dans la même fenêtre : c'est un double-clic, pas un re-clic.
    plus_tard(&mut app, pick_consts::DBLCLICK_MS as u64 - 50);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "un double-clic ouvre ce qu'il vise, il ne cherche pas dessous"
    );
}

#[test]
fn test_pick_1_the_cycle_expires_and_starts_over_at_the_top() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, pick_consts::CYCLE_TTL_MS as u64 + 100);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "passé {} ms, la pile est oubliée et on repart du sommet",
        pick_consts::CYCLE_TTL_MS
    );
}

#[test]
fn test_pick_1_moving_away_starts_a_new_stack() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, RECLIC_MS);
    // Au-delà du rayon du cycle, ce n'est plus le même endroit.
    let ailleurs = (at.0 + pick_consts::CYCLE_RADIUS_PX + 2.0, at.1);
    click_at(&mut app, ailleurs);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "un clic ailleurs repart du sommet"
    );
}

#[test]
fn test_pick_1_a_drag_never_advances_the_cycle() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, RECLIC_MS);

    // Un re-clic, mais qui **déplace** : on manipulait le nœud, on ne cherchait pas celui du
    // dessous. C'est la raison d'être du « au relâchement » de la fiche 07 § 3.3.
    app.handle_cursor_moved(PhysicalPosition::new(at.0, at.1));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_cursor_moved(PhysicalPosition::new(at.0 + 60.0, at.1 + 40.0));
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "le glisser garde le nœud qu'il déplaçait"
    );
}

/// Les bornes exactes du double-clic, sur la décision pure : l'heure y est une donnée, donc
/// la milliseconde se vérifie sans qu'une machine chargée puisse changer le résultat.
#[test]
fn le_rang_d_un_clic_se_verifie_a_la_milliseconde() {
    use crate::app::LastClickInfo;
    use crate::interactions::arrow_edit::rang_du_clic;

    let precedent = LastClickInfo {
        at_ms: 1_000,
        pos: (100.0, 100.0),
        id: "N1".to_string(),
        count: 1,
    };
    let rang = |ms: i64, pos: (f64, f64), cle: &str| {
        rang_du_clic(Some(&precedent), cle, (pos, DOUBLE_CLICK_SLOP_PX), ms)
    };

    let limite = 1_000 + pick_consts::DBLCLICK_MS;
    assert_eq!(rang(limite - 1, (100.0, 100.0), "N1"), 2, "juste dedans");
    assert_eq!(rang(limite, (100.0, 100.0), "N1"), 1, "la borne exclut");
    assert_eq!(rang(limite + 1, (100.0, 100.0), "N1"), 1, "juste dehors");

    assert_eq!(rang(1_010, (100.0, 100.0), "N2"), 1, "un autre noeud");
    assert_eq!(
        rang(1_010, (999.0, 999.0), "N1"),
        1,
        "le curseur a trop bouge"
    );
    assert_eq!(
        rang_du_clic(None, "N1", ((100.0, 100.0), DOUBLE_CLICK_SLOP_PX), 0),
        1,
        "le premier"
    );
}

/// **DPI-1 — à 150 %, la bande qui désigne une flèche est une fois et demie plus large** : un
/// clic à quinze pixels physiques du trait tombe hors de ses douze pixels logiques à 100 %, et
/// dedans à 150 % — là où ils valent dix-huit. Le noyau reçoit le zoom en pixels logiques ;
/// c'est ce qui rend la cible aussi facile à viser que chez Tauri.
#[test]
fn test_dpi_1_la_bande_d_une_fleche_suit_la_densite() {
    use glucose_core::types::{Annotation, Viewport};
    let touche = |densite: f32| {
        let mut app = GlucoseApp::new();
        let board = app.store.project.active_board_id.clone();
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        app.store
            .add_annotation(&board, Annotation::arrow("f", -200.0, 0.0, 200.0, 0.0));
        app.store.clear_selection();
        app.store.set_viewport(
            &board,
            Viewport {
                x: 400.0,
                y: 300.0,
                scale: 1.0,
            },
        );
        app.ui.scale_factor = densite;
        app.une_image_sans_fenetre((800, 600));
        app.pick_candidate_at(0.0, 15.0).map(|c| c.id)
    };
    assert_eq!(
        touche(1.0),
        None,
        "à 100 %, quinze pixels sont hors de la bande"
    );
    assert_eq!(
        touche(1.5).as_deref(),
        Some("f"),
        "à 150 %, ils sont dedans"
    );
}

/// **PICK-2, son cas exact : une image SOUS une carte** (registre de Tauri, n° 8). Un clic sur
/// la carte prend la carte — pas l'image qu'elle recouvre —, un double-clic l'ouvre, et un
/// re-clic lent descend à l'image.
#[test]
fn test_pick_2_une_image_sous_une_carte_le_clic_prend_la_carte_et_le_reclic_l_image() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app.store.add_image(
        &board,
        glucose_core::types::BoardImage::new("photo", 150.0, 100.0, 300.0, 200.0),
    );
    let mut carte = Annotation::text("carte", 50.0, 50.0, "Un texte sur la photo");
    if let Annotation::Text { width, height, .. } = &mut carte {
        (*width, *height) = (Some(240.0), Some(80.0));
    }
    app.store.add_annotation(&board, carte);
    app.store.clear_selection();
    let vp = app.store.viewport();
    let sur_le_texte = world_to_screen(100.0, 80.0, &vp);

    click_at(&mut app, sur_le_texte);
    assert_eq!(selection(&app), vec!["carte"], "la carte, peinte au-dessus");
    assert!(app.store.selected_image_ids.is_empty(), "et pas la photo");

    plus_tard(&mut app, RECLIC_MS);
    click_at(&mut app, sur_le_texte);
    assert_eq!(
        app.store.selected_image_ids,
        vec!["photo".to_string()],
        "le re-clic lent descend a la photo qu'elle recouvre"
    );

    // Et le double-clic sur la carte l'ouvre, depuis n'importe quel point du cycle : le temps
    // sépare les deux gestes, le texte n'a plus à être le fond de la pile.
    plus_tard(&mut app, pick_consts::CYCLE_TTL_MS as u64 + 100);
    click_at(&mut app, sur_le_texte);
    click_at(&mut app, sur_le_texte);
    assert_eq!(
        app.editing_session.as_ref().map(|s| s.ann_id.as_str()),
        Some("carte"),
        "le double-clic ouvre la carte"
    );
}
