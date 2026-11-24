//! La flèche en double (fiche 149 § 2) : la question au lâcher, et ses deux réponses.

use super::DEJA_RELIES;
use crate::app::GlucoseApp;
use crate::ui::question::Reponse;
use crate::ui::ActiveTool;
use glucose_core::types::Annotation;

/// Deux textes, « A » en (0, 0) et « B » en (600, 0), reliés par « f » de A à B ; et « C »
/// plus bas, que rien ne relie.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    for (id, x, y) in [("A", 0.0, 0.0), ("B", 600.0, 0.0), ("C", 0.0, 600.0)] {
        app.store
            .add_annotation(&board, Annotation::text(id, x, y, "un texte"));
    }
    let mut f = Annotation::arrow("f", 100.0, 20.0, 600.0, 20.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some("A".into());
        *target_id = Some("B".into());
    }
    app.store.add_annotation(&board, f);
    app.store.clear_selection();
    app.store.journal.clear();
    app
}

/// Trace une flèche à l'outil, d'un point du monde à un autre.
fn tracer(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.ui.active_tool = ActiveTool::Arrow;
    assert!(app.place_with_tool(de.0, de.1));
    app.update_draw(a.0, a.1);
    app.finish_draw();
}

fn fleches(app: &GlucoseApp) -> usize {
    app.store.active_board().map_or(0, |b| {
        b.annotations
            .iter()
            .filter(|a| matches!(a, Annotation::Arrow { .. }))
            .count()
    })
}

fn question(app: &GlucoseApp) -> Option<&str> {
    app.ui.question.as_ref().map(|(q, _)| q.titre.as_str())
}

#[test]
fn test_double_les_memes_bouts_posent_la_question_sans_rien_ecrire() {
    let mut app = app();
    tracer(&mut app, (20.0, 20.0), (620.0, 20.0));
    assert_eq!(question(&app), Some(DEJA_RELIES));
    assert_eq!(fleches(&app), 1, "la seconde attend la réponse");
    assert!(!app.store.can_undo(), "rien dans l'histoire");
}

#[test]
fn test_double_dans_l_autre_sens_aussi() {
    let mut app = app();
    tracer(&mut app, (620.0, 20.0), (20.0, 20.0));
    assert_eq!(question(&app), Some(DEJA_RELIES));
}

#[test]
fn test_double_annuler_choisit_l_ancienne() {
    let mut app = app();
    tracer(&mut app, (20.0, 20.0), (620.0, 20.0));
    app.repondre_a_la_question(Some(Reponse::Annuler));
    assert_eq!(fleches(&app), 1);
    let choisies: Vec<&str> = app.store.selected_arrows().iter().map(|a| a.id()).collect();
    assert_eq!(choisies, ["f"]);
}

#[test]
fn test_double_oui_trace_la_seconde_qu_un_ctrl_z_defait() {
    let mut app = app();
    tracer(&mut app, (20.0, 20.0), (620.0, 20.0));
    app.repondre_a_la_question(Some(Reponse::Oui));
    assert_eq!(fleches(&app), 2);
    let seconde = app.store.selected_arrows()[0].clone();
    assert!(
        matches!(&seconde, Annotation::Arrow { source_id: Some(s), target_id: Some(c), .. }
        if s == "A" && c == "B")
    );
    assert!(app.store.undo());
    assert_eq!(fleches(&app), 1, "un seul Ctrl+Z");
}

#[test]
fn test_double_d_autres_bouts_ne_demandent_rien() {
    let mut app = app();
    tracer(&mut app, (20.0, 20.0), (20.0, 620.0));
    assert_eq!(question(&app), None);
    assert_eq!(fleches(&app), 2);
}
