//! FRISE-1 par les vrais gestes : `Maj+R`, l'appui, le glisser, le relâchement, la molette,
//! la croix — sur un tableau de trois nœuds datés et d'un nœud qui ne l'est pas.

use super::*;
use glucose_core::types::{Annotation, TemporalAnchor};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, SmolStr};

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn date(start: i64, end: i64) -> TemporalAnchor {
    TemporalAnchor {
        start,
        end,
        label: None,
    }
}

/// La Révolution, la Grande Guerre, la Lune ; et un texte sans date.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let b = app.store.project.active_board_id.clone();
    if let Some(t) = app.store.active_board_mut() {
        t.annotations.clear();
    }
    for (id, x) in [
        ("rev", 0.0),
        ("guerre", 300.0),
        ("lune", 600.0),
        ("hors", 900.0),
    ] {
        app.store
            .add_annotation(&b, Annotation::text(id, x, 0.0, id));
    }
    for (id, d) in [
        ("rev", date(1789, 1799)),
        ("guerre", date(1914, 1918)),
        ("lune", date(1969, 1969)),
    ] {
        app.store
            .ancrer_dans_le_temps(&b, &[id.to_string()], Some(&d));
    }
    app.store.journal.clear();
    app
}

fn ecran(app: &GlucoseApp) -> ScreenFrame {
    app.screen_frame(ECRAN.0, ECRAN.1)
}

fn placee(app: &GlucoseApp) -> Placee {
    app.frise_placee(ecran(app)).expect("la frise ouverte")
}

fn maj_r(app: &mut GlucoseApp) {
    app.modifiers = ModifiersState::SHIFT;
    app.handle_shortcut_input(&Key::Character(SmolStr::new("R")), ElementState::Pressed);
    app.modifiers = ModifiersState::empty();
}

fn aller(app: &mut GlucoseApp, (x, y): (f32, f32)) {
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x),
        f64::from(y),
    ));
}

fn presser(app: &mut GlucoseApp, ici: (f32, f32)) {
    aller(app, ici);
    app.handle_mouse_down(winit::event::MouseButton::Left, ECRAN.0, ECRAN.1);
}

fn lacher(app: &mut GlucoseApp) {
    app.handle_mouse_up(winit::event::MouseButton::Left);
}

fn vue(app: &GlucoseApp) -> TemporalFilter {
    app.ui.frise.expect("ouverte").vue
}

/// **`Maj+R` l'ouvre sur les dates du document** — de la Révolution à la Lune, et une graduation
/// majeure de part et d'autre —, sans rien estomper : la fenêtre garde toute la vue. **`Maj+R`
/// la ferme**, et plus rien n'est écarté.
#[test]
fn frise_1_maj_r_l_ouvre_sur_les_dates_puis_la_ferme() {
    let mut app = app();
    maj_r(&mut app);
    let v = vue(&app);
    let pas = tick_step(1969 - 1789);
    assert_eq!(
        v,
        TemporalFilter {
            start: 1789 - 5 * pas,
            end: 1969 + 5 * pas
        }
    );
    assert_eq!(app.store.temporal_filter, Some(v), "rien d'estompé");
    assert!(
        app.ui.ecran_visible(ECRAN).1 < ECRAN.1 - placee(&app).rect.3,
        "le bas lui revient"
    );
    maj_r(&mut app);
    assert!(app.ui.frise.is_none());
    assert_eq!(app.store.temporal_filter, None);
    assert_eq!(app.ui.ecran_visible(ECRAN).1, ECRAN.1, "le bas rendu");
}

/// **Sans aucune date, la vue de Tauri.**
#[test]
fn frise_1_sans_date_la_vue_de_tauri() {
    let mut app = GlucoseApp::new();
    if let Some(t) = app.store.active_board_mut() {
        t.annotations.clear();
        t.images.clear();
    }
    assert_eq!(vue_initiale(&app.store), VUE_DE_TAURI);
}

/// **Glisser la poignée du début** jusqu'après la Révolution : la Révolution sort de la fenêtre et
/// s'estompe, la Grande Guerre et le texte sans date restent ; puis **la fenêtre entière**, glissée
/// vers le passé, garde sa largeur. Le relâchement la pose.
#[test]
fn frise_1_glisser_une_poignee_puis_la_fenetre() {
    let mut app = app();
    maj_r(&mut app);
    let p = placee(&app);
    let v = vue(&app);
    let (debut, _) = crate::ui::frise::bords(v, v, p.piste);
    let y = p.piste.1 + p.piste.3 / 2.0;
    presser(&mut app, (debut, y));
    aller(&mut app, (crate::ui::frise::x_de(1850, v, p.piste), y));
    lacher(&mut app);
    let f = app.store.temporal_filter.expect("la fenêtre");
    assert!(
        (1849..=1851).contains(&f.start),
        "le début suit la main : {f:?}"
    );
    assert_eq!(f.end, v.end, "la fin ne bouge pas");
    let presence = |app: &GlucoseApp, id: &str| {
        let b = app.store.project.active_board_id.clone();
        let a = app.store.project.annotation(&b, id).expect("le nœud");
        app.store.presence(a.regard())
    };
    assert!(
        presence(&app, "rev") < 1.0,
        "la Révolution, écartée, s'estompe"
    );
    assert_eq!(presence(&app, "guerre"), 1.0);
    assert_eq!(presence(&app, "hors"), 1.0, "sans date, jamais écarté");
    // La fenêtre entière, de cinquante ans vers le passé.
    let milieu = {
        let (a, b) = crate::ui::frise::bords(f, v, p.piste);
        (a + b) / 2.0
    };
    presser(&mut app, (milieu, y));
    let cinquante = (50.0 / annees(v) * f64::from(p.piste.2)) as f32;
    aller(&mut app, (milieu - cinquante, y));
    lacher(&mut app);
    let g = app.store.temporal_filter.expect("la fenêtre");
    assert_eq!(g.end - g.start, f.end - f.start, "sa largeur gardée");
    assert!(
        (f.start - g.start - 50).abs() <= 1,
        "cinquante ans plus tôt : {g:?}"
    );
    assert!(app.ui.frise.expect("ouverte").tenue.is_none(), "lâchée");
}

/// **La molette sur la frise zoome son échelle** autour de la main — l'année sous elle ne bouge
/// pas — et **le canevas ne bouge pas** ; hors d'elle, la molette va au canevas.
#[test]
fn frise_1_la_molette_zoome_l_echelle_et_pas_le_canevas() {
    let mut app = app();
    maj_r(&mut app);
    let p = placee(&app);
    let avant = vue(&app);
    let camera = app.store.viewport();
    let ici = (p.piste.0 + p.piste.2 * 0.3, p.piste.1 + 4.0);
    let an = an_de(ici.0, avant, p.piste);
    let cran = MouseScrollDelta::LineDelta(0.0, 1.0);
    assert!(app.molette_sur_la_frise(cran, ici, ecran(&app)));
    let apres = vue(&app);
    assert!(
        annees(apres) < annees(avant),
        "zoomée : {avant:?} → {apres:?}"
    );
    assert!(
        (an_de(ici.0, apres, p.piste) - an).abs() <= 1.0,
        "l'année sous la main reste"
    );
    assert_eq!(app.store.viewport(), camera, "le canevas n'a pas bougé");
    let ailleurs = (ECRAN.0 / 2.0, 200.0);
    assert!(
        !app.molette_sur_la_frise(cran, ailleurs, ecran(&app)),
        "hors d'elle"
    );
}

/// **La croix la ferme** ; un appui ailleurs sur elle ne descend pas au canevas — un appui sur le
/// canevas vide déchoisirait ce qui est choisi.
#[test]
fn frise_1_la_croix_la_ferme_et_elle_garde_ses_appuis() {
    let mut app = app();
    maj_r(&mut app);
    app.store.select_annotation("lune".into(), false);
    let p = placee(&app);
    presser(&mut app, (p.titre.0 + 2.0, p.titre.1 + 2.0));
    lacher(&mut app);
    assert!(app.ui.frise.is_some());
    assert_eq!(
        app.store.compte_de_la_selection(),
        1,
        "le choix gardé : l'appui est resté sur elle"
    );
    presser(
        &mut app,
        (p.fermer.0 + p.fermer.2 / 2.0, p.fermer.1 + p.fermer.3 / 2.0),
    );
    lacher(&mut app);
    assert!(app.ui.frise.is_none());
    assert_eq!(app.store.temporal_filter, None);
}

/// **Un zoom ne passe jamais sous une année ni au-delà du modèle**, et une vue au bord s'y appuie.
#[test]
fn frise_1_la_vue_reste_dans_le_modele() {
    let piste = (0.0, 0.0, 1000.0, 40.0);
    let v = TemporalFilter {
        start: 1789,
        end: 1799,
    };
    let au_plus_pres = vue_apres(v, piste, 500.0, (100.0, 0.0));
    assert_eq!(au_plus_pres.end - au_plus_pres.start, 0, "une année");
    let tout = vue_apres(v, piste, 500.0, (-200.0, 0.0));
    assert_eq!(
        (tout.start, tout.end),
        (YEAR_MIN, YEAR_MAX),
        "le modèle entier"
    );
    let poussee = vue_apres(
        TemporalFilter {
            start: 2_000,
            end: 2_900,
        },
        piste,
        0.0,
        (0.0, -1e6),
    );
    assert_eq!(poussee.end, YEAR_MAX, "appuyée au bord");
    assert_eq!(poussee.end - poussee.start, 900, "sans rétrécir");
}

/// **La molette par son vrai chemin** (`handle_mouse_wheel`) : la souris sur la frise, à la taille
/// de la fenêtre, la frise zoome et le canevas ne bouge pas.
#[test]
fn frise_1_la_molette_passe_par_son_vrai_chemin() {
    let mut app = app();
    maj_r(&mut app);
    let (l, h) = app.taille_de_la_fenetre();
    let p = app.frise_placee(app.screen_frame(l, h)).expect("ouverte");
    let avant = vue(&app);
    let camera = app.store.viewport();
    aller(
        &mut app,
        (p.piste.0 + p.piste.2 / 2.0, p.piste.1 + p.piste.3 / 2.0),
    );
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
    assert!(annees(vue(&app)) < annees(avant), "la frise a zoomé");
    assert_eq!(app.store.viewport(), camera, "le canevas n'a pas bougé");
}
