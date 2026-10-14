//! La lecture du storyboard à la vraie main (fiche 67) : les durées glissées ou tapées, qui
//! arrivent au document en un geste ; les textes et les champs, par leur question ; « ▶ Lire ».

use crate::app::GlucoseApp;
use crate::dock::champ::{Champ, Saisie};
use crate::dock::{cadre_du_panneau, TabId, WidgetRect};
use crate::ui::question::Suite;
use glucose_core::store::storyboard::{Grille, Texte};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un storyboard de huit cases, la première choisie, le panneau ouvert et peint.
fn app() -> (GlucoseApp, String, Vec<String>) {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let board = app.store.project.active_board_id.clone();
    app.store
        .poser_le_storyboard(&board, Grille::default(), (400.0, 300.0));
    let ids: Vec<String> = app
        .store
        .cases(&board)
        .iter()
        .map(|c| c.id.clone())
        .collect();
    app.ui.storyboard.choisie = Some(ids[0].clone());
    app.dock_manager.toggle_tab(TabId::Storyboard);
    app.store.journal.clear();
    peindre(&mut app);
    (app, board, ids)
}

/// Une image, comme à l'écran : le panneau suit le document.
fn peindre(app: &mut GlucoseApp) {
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
}

fn champ(app: &GlucoseApp, c: Champ) -> (f64, f64) {
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, TabId::Storyboard).expect("ouvert");
    let r = c.rect(cadre, &app.renderer.typography, &app.dock_manager);
    assert!(r.w > 0.0, "{c:?} est dans le panneau");
    centre(r)
}

fn centre(r: WidgetRect) -> (f64, f64) {
    (f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0))
}

fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(de.0, de.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        let t = c.to_string();
        assert!(app.prendre_la_touche(&Key::Character(t.clone().into()), true, Some(&t)));
    }
}

fn entree(app: &mut GlucoseApp) {
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Enter), true, None));
}

/// **Glisser la durée de la case choisie** : elle arrive au document quand on lâche — un seul
/// geste, quel que soit le chemin de la main ; et `Ctrl+Z` la rend au storyboard.
#[test]
fn test_glisser_la_duree_de_la_case_l_ecrit_en_un_geste() {
    let (mut app, board, _) = app();
    let p = champ(&app, Champ::CaseDuree);
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    // Une image entre deux mouvements, comme à l'écran : le panneau suit le document à
    // chacune, et ne doit pas reprendre à la main la valeur qu'elle tient.
    for dx in [20.0, 40.0, 60.0] {
        app.handle_cursor_moved(PhysicalPosition::new(p.0 + dx, p.1));
        peindre(&mut app);
    }
    assert!(
        app.dock_manager.storyboard.duree_case > 2000.0,
        "la main la tient"
    );
    assert_eq!(
        app.store.cases(&board)[0].duree_ms,
        None,
        "tenue, elle attend"
    );
    app.handle_mouse_up(MouseButton::Left);
    let reglee = app.store.cases(&board)[0]
        .duree_ms
        .expect("lâchée, elle est écrite");
    assert!(reglee > 2000, "elle a monté : {reglee}");
    assert_eq!(app.store.undo_depth(), 1, "un geste");
    assert!(app.store.undo());
    assert_eq!(app.store.cases(&board)[0].duree_ms, None);
    peindre(&mut app);
    assert_eq!(
        app.dock_manager.storyboard.duree_case, 2000.0,
        "le champ suit le document"
    );
}

/// **Taper une durée en secondes, la virgule française** : « 2,5 » puis Entrée, et la case dure
/// 2 500 ms.
#[test]
fn test_taper_une_duree_en_secondes() {
    let (mut app, board, _) = app();
    let p = champ(&app, Champ::CaseDuree);
    glisser(&mut app, p, p);
    assert!(matches!(
        app.dock_manager.champ,
        Some(Saisie::Frappe { .. })
    ));
    taper(&mut app, "2,5");
    entree(&mut app);
    assert_eq!(app.store.cases(&board)[0].duree_ms, Some(2500));
}

/// **La durée par case règle toutes celles qui n'ont pas la leur** — pas les autres.
#[test]
fn test_la_duree_par_case_regle_celles_qui_n_ont_pas_la_leur() {
    let (mut app, board, ids) = app();
    app.store.regler_la_duree(&board, &ids[3], Some(700));
    peindre(&mut app);
    let p = champ(&app, Champ::StoryDuree);
    glisser(&mut app, p, p);
    taper(&mut app, "1");
    entree(&mut app);
    let s = app.store.storyboard(&board).expect("posé").clone();
    let cases = app.store.cases(&board);
    assert_eq!(s.duree(cases[0]), 1000);
    assert_eq!(s.duree(cases[3]), 700, "elle garde la sienne");
}

/// **« ↺ » rend la case au storyboard**, par le vrai clic.
#[test]
fn test_rendre_la_duree_par_le_vrai_clic() {
    let (mut app, board, ids) = app();
    app.store.regler_la_duree(&board, &ids[0], Some(900));
    peindre(&mut app);
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, TabId::Storyboard).expect("ouvert");
    let s = &app.dock_manager.storyboard;
    let layout = crate::dock::storyboard::layout_storyboard_panel(cadre, s, &s.contenu);
    let rendre = layout
        .lecture
        .and_then(|l| l.case)
        .and_then(|c| c.rendre)
        .expect("« ↺ »");
    let p = centre(rendre);
    glisser(&mut app, p, p);
    assert_eq!(app.store.cases(&board)[0].duree_ms, None);
}

/// **Écrire un champ de la case** : sa question porte son nom, ce qu'on tape va à son rang.
#[test]
fn test_ecrire_un_champ_par_sa_question() {
    let (mut app, board, ids) = app();
    app.ecrire_la_case(&ids[0], Texte::Champ(1));
    let Some((question, Suite::EcrireLaCase { texte, .. })) = &app.ui.question else {
        panic!("la question se pose");
    };
    assert_eq!(question.titre, "Case 1 — Action");
    assert_eq!(*texte, Texte::Champ(1));
    taper(&mut app, "il court");
    entree(&mut app);
    assert_eq!(app.store.cases(&board)[0].textes, ["", "il court"]);
}

/// **Nommer un champ : renommer, retirer, ajouter** — et un nom vide ne nomme rien.
#[test]
fn test_renommer_retirer_et_ajouter_un_champ() {
    let (mut app, board, ids) = app();
    app.store
        .ecrire_dans_la_case(&board, &ids[0], Texte::Champ(2), "pluie");
    let champs = |app: &GlucoseApp| app.store.storyboard(&board).expect("posé").champs.clone();

    app.nommer_le_champ(Some(0));
    taper(&mut app, "Réplique");
    entree(&mut app);
    assert_eq!(champs(&app), ["Réplique", "Action", "Notes"]);

    app.nommer_le_champ(Some(1));
    app.repondre_a_la_question(Some(crate::ui::question::Reponse::Non));
    assert_eq!(champs(&app), ["Réplique", "Notes"], "retiré");
    assert_eq!(
        app.store.cases(&board)[0].textes,
        ["", "pluie"],
        "les Notes suivent"
    );

    app.nommer_le_champ(None);
    taper(&mut app, "Son");
    entree(&mut app);
    assert_eq!(champs(&app), ["Réplique", "Notes", "Son"]);

    app.nommer_le_champ(None);
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Backspace), true, None));
    entree(&mut app);
    assert_eq!(champs(&app).len(), 3, "un nom vide ne nomme rien");
}

/// **« ▶ Lire », par le vrai clic** : la lecture part de la case choisie, l'interface se cache.
#[test]
fn test_lire_par_le_vrai_clic() {
    let (mut app, _, ids) = app();
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, TabId::Storyboard).expect("ouvert");
    let s = &app.dock_manager.storyboard;
    let layout = crate::dock::storyboard::layout_storyboard_panel(cadre, s, &s.contenu);
    let p = centre(layout.lecture.expect("la lecture").lire);
    glisser(&mut app, p, p);
    let lecture = app.ui.lecture.as_ref().expect("elle lit");
    assert_eq!(app.store.cases(&lecture.board)[lecture.rang].id, ids[0]);
    assert!(app.ui.sans_interface());
}

/// **Tab dans un champ passe au même champ de la case suivante** — pas à sa description.
#[test]
fn test_tab_dans_un_champ_passe_au_meme_champ_de_la_suivante() {
    let (mut app, board, ids) = app();
    app.ecrire_la_case(&ids[0], Texte::Champ(0));
    taper(&mut app, "Halte");
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Tab), true, None));
    let Some((_, Suite::EcrireLaCase { case, texte, .. })) = &app.ui.question else {
        panic!("la suivante se demande");
    };
    assert_eq!((case, *texte), (&ids[1], Texte::Champ(0)));
    assert_eq!(app.store.cases(&board)[0].textes, ["Halte"]);
}
