//! Les épreuves d'OPACITE-1 dans la roue (fiche 133) : par les vrais gestes — le clic, le glisser,
//! la molette, les touches — sur un dossier choisi, seul ou avec une membrane.

use crate::app::GlucoseApp;
use crate::interactions::roue::tests::{aller, app, couleur, lacher, plat, presser, touche};
use glucose_core::types::{CanvasFolder, COULEUR_DE_NAISSANCE, OPACITE_DE_NAISSANCE};
use winit::keyboard::{ModifiersState, NamedKey};

/// L'application de la roue, un dossier en plus, loin de la membrane ; le dossier choisi — seul,
/// ou avec la membrane (`avec_la_membrane`).
fn app_au_dossier(avec_la_membrane: bool) -> GlucoseApp {
    app_au_dossier_de(COULEUR_DE_NAISSANCE, avec_la_membrane)
}

/// La même, le dossier à cette couleur.
fn app_au_dossier_de(couleur: &str, avec_la_membrane: bool) -> GlucoseApp {
    let mut app = app();
    let tableau = app.store.project.active_board_id.clone();
    let mut d = CanvasFolder::new("d", "Dossier", "");
    d.x = 5000.0;
    d.color = couleur.to_string();
    app.store.create_folder(&tableau, d);
    if !avec_la_membrane {
        app.store.set_selected_annotation_ids(Vec::new());
    }
    app.store.select_folder("d".into(), avec_la_membrane);
    app.store.journal.clear();
    app
}

fn couleur_du_dossier(app: &GlucoseApp) -> String {
    app.store
        .active_board()
        .and_then(|b| b.folders.iter().find(|f| f.id == "d"))
        .map(|f| f.color.clone())
        .unwrap_or_default()
}

/// Le point du curseur de l'opacité à cette hauteur (0 en bas, 1 en haut).
fn sur_l_opacite(app: &GlucoseApp, part: f32) -> (f32, f32) {
    let (x, y, w, h) = plat(app).opacite.expect("le curseur de l'opacité");
    (x + w / 2.0, y + (1.0 - part) * h)
}

/// **Le curseur n'existe qu'avec un dossier choisi** : une membrane seule n'a pas d'opacité à
/// régler (la sienne dit l'appartenance, VISION § 5).
#[test]
fn opacite_1_le_curseur_n_existe_qu_avec_un_dossier() {
    let mut sans = app();
    assert!(sans.ouvrir_la_roue());
    assert_eq!(sans.ui.roue.as_ref().and_then(|r| r.opacite), None);
    assert_eq!(plat(&sans).opacite, None);
    let mut avec = app_au_dossier(false);
    assert!(avec.ouvrir_la_roue());
    let roue = avec.ui.roue.as_ref().expect("ouverte");
    assert_eq!(roue.opacite, Some(OPACITE_DE_NAISSANCE));
    assert!(plat(&avec).opacite.is_some());
    assert!(plat(&avec).rect.2 > plat(&sans).rect.2, "la roue s'élargit");
}

/// **Glisser le curseur règle l'opacité en direct ; valider en fait un seul geste** — un `Ctrl+Z`
/// rend le dossier d'avant ; la teinte ne bouge pas.
#[test]
fn opacite_1_glisser_regle_en_direct_et_valider_fait_un_seul_geste() {
    let mut app = app_au_dossier(false);
    assert!(app.ouvrir_la_roue());
    let haut = sur_l_opacite(&app, 1.0);
    presser(&mut app, haut);
    assert_eq!(
        couleur_du_dossier(&app),
        format!("{COULEUR_DE_NAISSANCE}ff")
    );
    for i in 1..=10 {
        let ici = sur_l_opacite(&app, 1.0 - i as f32 * 0.05);
        aller(&mut app, ici);
    }
    lacher(&mut app);
    let a = app
        .ui
        .roue
        .as_ref()
        .and_then(|r| r.opacite)
        .expect("une opacité");
    assert!((120..=135).contains(&a), "à mi-hauteur : {a}");
    assert_eq!(
        couleur_du_dossier(&app),
        format!("{COULEUR_DE_NAISSANCE}{a:02x}")
    );
    app.valider_la_roue();
    app.annuler();
    assert_eq!(
        couleur_du_dossier(&app),
        COULEUR_DE_NAISSANCE,
        "un seul Ctrl+Z"
    );
}

/// **Régler l'opacité ne colore rien ; choisir une couleur garde l'opacité réglée** — chaque
/// mouvement repart des couleurs d'avant la roue, et repose ce que la main a touché. La membrane,
/// choisie la première, donne à la roue sa couleur d'avant (aucune) : la reposer sans que la main
/// l'ait touchée rendrait le dossier rouge au gris de sa naissance.
#[test]
fn opacite_1_l_opacite_et_la_couleur_se_reposent_chacune() {
    let mut app = app_au_dossier_de("#ff0000", true);
    assert!(app.ouvrir_la_roue());
    let haut = sur_l_opacite(&app, 1.0);
    presser(&mut app, haut);
    lacher(&mut app);
    assert_eq!(couleur(&app), None, "la membrane n'est pas colorée");
    assert_eq!(
        couleur_du_dossier(&app),
        "#ff0000ff",
        "le dossier garde sa teinte"
    );
    let (x, y, d) = plat(&app).disque;
    presser(&mut app, (x + d * 0.9, y + d / 2.0));
    lacher(&mut app);
    let teinte = couleur(&app).expect("la membrane colorée");
    assert_eq!(
        couleur_du_dossier(&app),
        format!("{teinte}ff"),
        "l'opacité reste"
    );
    touche(&mut app, NamedKey::Backspace);
    assert_eq!(
        couleur_du_dossier(&app),
        format!("{COULEUR_DE_NAISSANCE}ff")
    );
    touche(&mut app, NamedKey::Escape);
    assert_eq!(couleur_du_dossier(&app), "#ff0000", "Échap : tout revient");
    assert!(!app.store.can_undo(), "rien à défaire");
}

/// **`Maj` affine l'opacité** : le même mouvement de la main n'en parcourt qu'un dixième.
#[test]
fn opacite_1_maj_affine_d_un_dixieme() {
    let parcours = |mods: ModifiersState| {
        let mut app = app_au_dossier(false);
        assert!(app.ouvrir_la_roue());
        app.modifiers = mods;
        let depart = sur_l_opacite(&app, 0.0);
        presser(&mut app, depart);
        let lue = |app: &GlucoseApp| app.ui.roue.as_ref().and_then(|r| r.opacite).map(i32::from);
        let avant = lue(&app).expect("une opacité");
        aller(&mut app, (depart.0, depart.1 - 100.0));
        lue(&app).expect("une opacité") - avant
    };
    let (plein, fin) = (
        parcours(ModifiersState::empty()),
        parcours(ModifiersState::SHIFT),
    );
    assert!(plein > 50, "{plein}");
    assert!((fin * 10 - plein).abs() <= 10, "{fin} × 10 contre {plein}");
}

/// **La molette sur le curseur change l'opacité d'une différence à peine visible par cran** ;
/// ailleurs sur la roue, elle garde la clarté.
#[test]
fn opacite_1_la_molette_sur_le_curseur_change_l_opacite() {
    let mut app = app_au_dossier(false);
    assert!(app.ouvrir_la_roue());
    let clarte = app.ui.roue.as_ref().expect("ouverte").place.clarte;
    let ici = sur_l_opacite(&app, 0.5);
    aller(&mut app, ici);
    app.handle_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, 2.0));
    let roue = app.ui.roue.as_ref().expect("ouverte");
    let attendue = ((f64::from(OPACITE_DE_NAISSANCE) / 255.0 + 0.04) * 255.0).round() as u8;
    assert_eq!(roue.opacite, Some(attendue));
    assert_eq!(roue.place.clarte, clarte, "la clarté ne bouge pas");
}

/// **Le curseur se dessine : la couleur posée sur le fond du canevas, opaque en haut, le fond seul
/// en bas.** Un dossier à sa couleur de naissance montre son gris.
#[test]
fn opacite_1_le_curseur_montre_la_couleur_sur_le_fond() {
    let mut app = app_au_dossier(false);
    assert!(app.ouvrir_la_roue());
    let (largeur, hauteur) = (1440, 900);
    let mut pixmap = tiny_skia::Pixmap::new(largeur, hauteur).expect("pixmap");
    let e = app.screen_frame(largeur as f32, hauteur as f32);
    let barre = crate::ui::couleurs::layout_barre_de_couleur(
        &app.store,
        &app.renderer.typography,
        app.ui.ecran_visible((e.width, e.height)),
        e.scale,
    )
    .expect("la barre de couleur");
    let theme = crate::theme::Theme::dark();
    let roue = app.ui.roue.as_ref().expect("ouverte");
    let lieu = ((e.width, e.height), e.scale);
    let typo = &app.renderer.typography;
    crate::ui::roue::draw_roue(&mut pixmap.as_mut(), roue, barre.rect, (typo, &theme), lieu);
    let (x, y, w, h) = plat(&app).opacite.expect("le curseur");
    let lire = |py: f32| {
        let p = pixmap
            .pixel((x + w / 2.0) as u32, py as u32)
            .expect("dans l'écran");
        (p.red(), p.green(), p.blue())
    };
    let octet = |c: f32| (c * 255.0).round() as i16;
    let fond = theme.bg_canvas;
    let (hr, hg, hb) = lire(y + 1.0);
    assert!(
        (i16::from(hr) - 136).abs() <= 2 && hr == hg && hg == hb,
        "en haut : {hr},{hg},{hb}"
    );
    let (br, _, _) = lire(y + h - 1.0);
    assert!(
        (i16::from(br) - octet(fond.red())).abs() <= 2,
        "en bas : {br} contre le fond"
    );
}
