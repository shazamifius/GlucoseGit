//! DATE-1 par les vraies entrées : `Maj+T`, le champ tapé au clavier, Entrée, une ère proposée,
//! « Retirer la date », `Ctrl+Z`.

use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::ui::question::{Reponse, Suite};
use glucose_core::timeline::format_anchor;
use glucose_core::types::{Annotation, BoardImage, TemporalAnchor};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

const ECRAN: (u32, u32) = (1200, 800);

/// Un texte et une image, choisis ; un post-it à côté, pas choisi.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("a", 0.0, 0.0, "la Révolution"));
    app.store
        .add_annotation(&board, Annotation::sticky("p", 0.0, 400.0, "un post-it"));
    app.store
        .add_image(&board, BoardImage::new("i", 600.0, 0.0, 100.0, 100.0));
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.store.set_selected_image_ids(vec!["i".into()]);
    app.store.journal.clear();
    app
}

fn maj_t(app: &mut GlucoseApp) {
    app.modifiers = ModifiersState::SHIFT;
    app.handle_shortcut_input(&Key::Character(SmolStr::new("T")), ElementState::Pressed);
    app.modifiers = ModifiersState::empty();
}

/// Tape `texte` dans le champ de la question, lettre par lettre, une image entre chacune.
fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        let s = c.to_string();
        assert!(app.prendre_la_touche(&Key::Character(SmolStr::new(&s)), true, Some(&s)));
        app.une_image_sans_fenetre(ECRAN);
    }
}

fn entree(app: &mut GlucoseApp) {
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Enter), true, None));
    app.une_image_sans_fenetre(ECRAN);
}

fn date(app: &GlucoseApp, id: &str) -> Option<TemporalAnchor> {
    let board = &app.store.project.active_board_id;
    match app.store.project.annotation(board, id) {
        Some(a) => a.temporal_anchor().cloned(),
        None => app.store.image(board, id)?.temporal_anchor.clone(),
    }
}

fn libelles(app: &GlucoseApp) -> Vec<String> {
    let (q, _) = app.ui.question.as_ref().expect("la question");
    q.choix.iter().map(|(l, _)| l.clone()).collect()
}

/// **`Maj+T`, « 1789-1799 », Entrée** : le texte et l'image choisis sont datés, pas le post-it ;
/// un seul `Ctrl+Z` les rend sans date.
#[test]
fn test_date_1_maj_t_une_plage_tapee_puis_entree() {
    let mut app = application();
    maj_t(&mut app);
    let (q, suite) = app.ui.question.as_ref().expect("la question");
    assert_eq!(q.titre, "Dater ces 2 nœuds");
    assert!(matches!(suite, Suite::Dater { .. }));
    taper(&mut app, "1789-1799");
    entree(&mut app);
    assert!(app.ui.question.is_none(), "la question est partie");
    let attendue = TemporalAnchor {
        start: 1789,
        end: 1799,
        label: None,
    };
    assert_eq!(date(&app, "a"), Some(attendue.clone()));
    assert_eq!(date(&app, "i"), Some(attendue));
    assert_eq!(date(&app, "p"), None, "pas choisi");
    app.annuler();
    assert_eq!(date(&app, "a"), None);
    assert_eq!(date(&app, "i"), None, "un seul geste");
}

/// **Une date illisible se redemande**, avec ce qu'on avait tapé et ce qui n'allait pas ; rien
/// n'est daté.
#[test]
fn test_date_1_une_date_illisible_se_redemande() {
    let mut app = application();
    maj_t(&mut app);
    taper(&mut app, "hier");
    entree(&mut app);
    let (q, _) = app.ui.question.as_ref().expect("redemandée");
    assert_eq!(q.champ.as_ref().map(|c| c.texte.as_str()), Some("hier"));
    assert!(q.texte.contains("« hier »"), "{}", q.texte);
    assert_eq!(date(&app, "a"), None);
}

/// **Les ères répondent à ce qu'on tape** — sans accents : « moyen age » trouve le Moyen Âge ;
/// en choisir une la pose avec son nom. Puis, datés, la question propose « Retirer la date »,
/// et son champ porte la date.
#[test]
fn test_date_1_une_ere_proposee_puis_retirer() {
    let mut app = application();
    maj_t(&mut app);
    assert!(!libelles(&app).iter().any(|l| l == "Retirer la date"));
    taper(&mut app, "renais");
    let renaissance = libelles(&app)
        .iter()
        .position(|l| l.starts_with("Renaissance"))
        .expect("la Renaissance proposée");
    let (q, _) = app.ui.question.as_ref().unwrap();
    let r = q.choix[renaissance].1;
    repondre(&mut app, r);
    let posee = date(&app, "a").expect("datée");
    assert_eq!(posee.label.as_deref(), Some("Renaissance"));
    assert_eq!(format_anchor(&posee), "Renaissance");
    maj_t(&mut app);
    let (q, _) = app.ui.question.as_ref().unwrap();
    assert_eq!(
        q.champ.as_ref().map(|c| c.texte.as_str()),
        Some("Renaissance")
    );
    assert!(libelles(&app).iter().any(|l| l == "Retirer la date"));
    repondre(&mut app, Reponse::Non);
    assert_eq!(date(&app, "a"), None, "retirée");
    assert_eq!(date(&app, "i"), None);
    maj_t(&mut app);
    taper(&mut app, "moyen age");
    assert!(
        libelles(&app).iter().any(|l| l.contains("Moyen Âge")),
        "{:?}",
        libelles(&app)
    );
}

/// **Sans rien de choisi, `Maj+T` ne demande rien** (APPLIQUE-1) ; la palette ne propose pas
/// « Dater… ».
#[test]
fn test_date_1_sans_choix_rien() {
    let mut app = application();
    app.store.clear_selection();
    maj_t(&mut app);
    assert!(app.ui.question.is_none());
    assert!(!app.s_applique(crate::commandes::Commande::Dater));
}
