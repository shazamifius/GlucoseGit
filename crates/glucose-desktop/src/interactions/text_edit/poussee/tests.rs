//! POUSSER-145 lettre à lettre : le document grandit pendant qu'on tape, un seul `Ctrl+Z` au bout.

use crate::app::GlucoseApp;
use crate::interactions::resize::tests::text_card;
use crate::interactions::text_edit::keys::Command;
use glucose_core::geometry::Rect;
use glucose_core::types::Annotation;

const ECART: f64 = 10.0;

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    app
}

fn boite(app: &GlucoseApp, id: &str) -> Rect {
    (app.store.active_board())
        .and_then(|b| b.annotations.iter().find(|a| a.id() == id))
        .and_then(Annotation::rect)
        .expect("une boîte")
}

/// Un texte d'une ligne, ajusté, et un voisin `ECART` dessous ; la saisie ouverte sur le texte.
fn en_saisie() -> GlucoseApp {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, text_card("t", 30.0, 42.0, 240.0, "une ligne"));
    app.fit_text_card_height("t");
    let bas = boite(&app, "t").bottom();
    (app.store).add_annotation(
        &board,
        text_card("dessous", 30.0, bas + ECART, 240.0, "voisin"),
    );
    app.start_text_edit("t".into(), "une ligne".into());
    app
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    app.apply_text_command(Command::Insert(texte.into()), false);
}

fn l_ecart_est_garde(app: &GlucoseApp) -> bool {
    (boite(app, "dessous").top - (boite(app, "t").bottom() + ECART)).abs() < 1e-6
}

/// Le texte grandit dans le document **pendant** qu'on tape, son voisin s'écarte, et toute la
/// saisie — ses pas et sa validation — se défait d'un seul `Ctrl+Z`.
#[test]
fn test_pousser_145_le_voisin_s_ecarte_pendant_qu_on_tape() {
    let mut app = en_saisie();
    let (depart, voisin) = (boite(&app, "t"), boite(&app, "dessous"));
    let entrees = app.store.undo_depth();
    for i in 2..=8 {
        taper(&mut app, &format!("\nligne {i}"));
    }
    assert!(app.editing_session.is_some(), "toujours en saisie");
    assert!(
        boite(&app, "t").height > depart.height,
        "le document a grandi"
    );
    assert!(l_ecart_est_garde(&app), "le voisin s'écarte, l'écart gardé");
    app.commit_editing();
    assert!(l_ecart_est_garde(&app));
    assert_eq!(app.store.undo_depth(), entrees + 1, "une saisie = un geste");
    assert!(app.store.undo());
    assert_eq!(boite(&app, "t"), depart);
    assert_eq!(boite(&app, "dessous"), voisin);
}

/// Effacer fait remonter, pendant la saisie ; revenue à son départ, elle ne laisse rien à défaire.
#[test]
fn test_pousser_145_effacer_fait_remonter_pendant_qu_on_tape() {
    let mut app = en_saisie();
    let (depart, voisin) = (boite(&app, "t"), boite(&app, "dessous"));
    let entrees = app.store.undo_depth();
    taper(&mut app, "\ndeux\ntrois\nquatre");
    assert!(
        boite(&app, "dessous").top > voisin.top,
        "le cas n'est pas exercé"
    );
    app.apply_text_command(Command::SelectAll, false);
    taper(&mut app, "une ligne");
    assert_eq!(boite(&app, "t"), depart, "jamais sous son origine");
    assert_eq!(boite(&app, "dessous"), voisin, "revenu à sa place");
    app.commit_editing();
    assert_eq!(app.store.undo_depth(), entrees, "rien à défaire");
}

/// Ce qu'une autre main déplace pendant la saisie lui reste : le pas suivant ne le ramène pas.
#[test]
fn test_pousser_145_ce_qu_une_autre_main_prend_lui_reste() {
    let mut app = en_saisie();
    taper(&mut app, "\ndeux\ntrois");
    let board = app.store.project.active_board_id.clone();
    let loin = Rect {
        top: 2000.0,
        ..boite(&app, "dessous")
    };
    app.store.begin_live_edit();
    app.store.set_annotation_rect(&board, "dessous", loin);
    app.store.end_live_edit();
    taper(&mut app, "\nquatre");
    app.apply_text_command(Command::SelectAll, false);
    taper(&mut app, "une ligne");
    app.commit_editing();
    assert_eq!(boite(&app, "dessous"), loin);
}

/// Une hauteur tirée à la main est un plancher : écrire dedans ne la change pas.
#[test]
fn test_pousser_145_une_hauteur_tiree_a_la_main_reste_un_plancher() {
    let mut app = en_saisie();
    let board = app.store.project.active_board_id.clone();
    app.commit_editing();
    let haute = Rect {
        height: 400.0,
        ..boite(&app, "t")
    };
    app.store.begin_live_edit();
    app.store.set_annotation_rect(&board, "t", haute);
    app.store.end_live_edit();
    app.start_text_edit("t".into(), "une ligne".into());
    taper(&mut app, "\ndeux");
    assert_eq!(boite(&app, "t"), haute);
}

/// Deux saisies de suite font deux gestes : le numéro d'une saisie ne sert qu'une fois.
#[test]
fn test_pousser_145_deux_saisies_font_deux_gestes() {
    let mut app = en_saisie();
    let entrees = app.store.undo_depth();
    taper(&mut app, "\ndeux");
    app.commit_editing();
    app.start_text_edit("t".into(), "une ligne\ndeux".into());
    taper(&mut app, "\ntrois");
    app.commit_editing();
    assert_eq!(app.store.undo_depth(), entrees + 2);
}

/// Pendant la saisie, le document a la hauteur que le dessin montre — le texte coupé comme on
/// l'édite, balises comprises, et non comme on le lira.
#[test]
fn test_pousser_145_le_document_a_la_hauteur_que_le_dessin_montre() {
    let mut app = en_saisie();
    taper(&mut app, &" **gras**".repeat(40));
    let r = &app.renderer;
    let ann = (app.store.project.annotation("main", "t")).expect("le texte");
    let montree = crate::renderer::card::boite_montree(
        (&r.typography, &r.math, &r.mises_en_page),
        ann,
        app.editing_session.as_ref(),
    );
    assert_eq!(Some(boite(&app, "t")), montree);
    let lue = crate::renderer::card::text_card_fit_height(
        &r.typography,
        &r.math,
        &app.editing_session.as_ref().expect("en saisie").buffer,
        240.0,
    );
    assert!(boite(&app, "t").height > lue, "le cas n'est pas exercé");
}
