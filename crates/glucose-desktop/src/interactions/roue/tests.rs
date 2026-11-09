//! Les épreuves de la roue de couleur (ROUE-1, fiche 99) : par les vrais gestes — le clic, le
//! glisser, la molette, les touches — sur une membrane choisie.

use super::*;
use glucose_core::couleur::roue::point_de;
use glucose_core::types::Annotation;
use winit::keyboard::{Key, ModifiersState, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application au tableau vide, l'accueil éteint, une membrane sans couleur choisie.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&tableau, Annotation::membrane("m", 0.0, 0.0, 320.0, 240.0));
    app.store.select_annotation("m".into(), false);
    // Le décor n'est pas un geste de l'épreuve : sans cela, « rien à défaire » serait déjà faux,
    // et « un geste à défaire » déjà vrai — aveugle.
    app.store.journal.clear();
    app
}

fn ecran(app: &GlucoseApp) -> ScreenFrame {
    app.screen_frame(ECRAN.0, ECRAN.1)
}

fn couleur(app: &GlucoseApp) -> Option<String> {
    app.store.couleurs_choisies()[0].map(str::to_string)
}

/// La roue placée, telle que le clic la lit.
fn plat(app: &GlucoseApp) -> crate::ui::roue::RoueAPlat {
    let e = ecran(app);
    let barre = crate::ui::couleurs::layout_barre_de_couleur(
        &app.store,
        &app.renderer.typography,
        app.ui.ecran_visible((e.width, e.height)),
        e.scale,
    )
    .expect("la barre de couleur");
    placer(barre.rect, (e.width, e.height), e.scale)
}

/// Le point de l'écran d'un point du disque, en fractions de son rayon.
fn sur_le_disque(app: &GlucoseApp, (dx, dy): (f64, f64)) -> (f32, f32) {
    let (x, y, d) = plat(app).disque;
    let r = d / 2.0;
    (x + r + dx as f32 * r, y + r + dy as f32 * r)
}

fn presser_sur_le_disque(app: &mut GlucoseApp, point: (f64, f64)) {
    let ici = sur_le_disque(app, point);
    presser(app, ici);
}

/// Le curseur va là — le vrai chemin de la souris, et du doigt.
fn aller(app: &mut GlucoseApp, (x, y): (f32, f32)) {
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x),
        f64::from(y),
    ));
}

/// Le bouton gauche presse là, par le vrai chemin de la souris.
fn presser(app: &mut GlucoseApp, ici: (f32, f32)) {
    aller(app, ici);
    app.handle_mouse_down(winit::event::MouseButton::Left, ECRAN.0, ECRAN.1);
}

fn lacher(app: &mut GlucoseApp) {
    app.handle_mouse_up(winit::event::MouseButton::Left);
}

/// Une touche, par le chemin de ce qui passe avant tout.
fn touche(app: &mut GlucoseApp, nommee: NamedKey) {
    app.prendre_la_touche(&Key::Named(nommee), true, None);
}

/// **Glisser dans le disque colore ce qui est choisi, en direct ; valider en fait un seul
/// geste** — un `Ctrl+Z` rend la couleur d'avant.
#[test]
fn roue_1_glisser_colore_en_direct_et_valider_fait_un_seul_geste() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    let rose = PlaceDansLaRoue {
        clarte: 0.7,
        teinte: 0.0,
        saturation: 1.0,
    };
    presser_sur_le_disque(&mut app, point_de(rose));
    let posee = couleur(&app).expect("une couleur, en direct");
    for i in 1..=20 {
        let p = sur_le_disque(
            &app,
            point_de(PlaceDansLaRoue {
                teinte: f64::from(i) * 0.05,
                ..rose
            }),
        );
        aller(&mut app, p);
    }
    lacher(&mut app);
    assert!(
        app.ui.roue.as_ref().is_some_and(|r| r.glisse.is_none()),
        "lâchée, ouverte"
    );
    assert_ne!(
        couleur(&app).as_deref(),
        Some(posee.as_str()),
        "le glisser change la couleur"
    );
    app.valider_la_roue();
    assert!(app.ui.roue.is_none(), "valider ferme la roue");
    app.annuler();
    assert_eq!(
        couleur(&app),
        None,
        "un seul Ctrl+Z rend la couleur d'avant"
    );
}

/// **`Échap` annule tout ce que la roue a fait, sans rien laisser à défaire.**
#[test]
fn roue_1_echap_annule_sans_trace() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    presser_sur_le_disque(&mut app, (0.5, 0.0));
    assert!(couleur(&app).is_some());
    lacher(&mut app);
    touche(&mut app, NamedKey::Escape);
    assert_eq!(couleur(&app), None);
    assert!(app.ui.roue.is_none());
    assert!(!app.store.can_undo(), "rien à défaire");
}

/// **`Ctrl` cale la teinte tous les 30°.**
#[test]
fn roue_1_ctrl_cale_la_teinte() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    app.modifiers = ModifiersState::CONTROL;
    presser_sur_le_disque(&mut app, (0.6, -0.25));
    let teinte = app.ui.roue.as_ref().expect("ouverte").place.teinte;
    let douzieme = teinte / (std::f64::consts::PI / 6.0);
    assert!(
        (douzieme - douzieme.round()).abs() < 1e-9,
        "teinte {teinte}"
    );
}

/// **`Maj` affine** : le même mouvement de la main ne déplace la place que d'un dixième.
#[test]
fn roue_1_maj_affine_d_un_dixieme() {
    let deplacement = |mods: ModifiersState| {
        let mut app = app();
        assert!(app.ouvrir_la_roue());
        app.modifiers = mods;
        let depart = sur_le_disque(&app, (0.0, 0.0));
        presser(&mut app, depart);
        aller(&mut app, (depart.0 + 40.0, depart.1));
        app.ui.roue.as_ref().expect("ouverte").place.saturation
    };
    let (plein, fin) = (
        deplacement(ModifiersState::empty()),
        deplacement(ModifiersState::SHIFT),
    );
    assert!(plein > 0.0);
    assert!(
        (fin * 10.0 - plein).abs() < 1e-9,
        "{fin} × 10 contre {plein}"
    );
}

/// **La molette sur la roue change la clarté d'une différence à peine visible par cran**, et le
/// canevas ne bouge pas.
#[test]
fn roue_1_la_molette_change_la_clarte_d_une_difference_a_peine_visible() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    let avant = app.ui.roue.as_ref().expect("ouverte").place.clarte;
    let vue = app.store.viewport();
    let (x, y, d) = plat(&app).disque;
    aller(&mut app, (x + d / 2.0, y + d / 2.0));
    app.handle_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, -2.0));
    let apres = app.ui.roue.as_ref().expect("ouverte").place.clarte;
    assert!(
        (avant - apres - 2.0 * DIFFERENCE_A_PEINE_VISIBLE).abs() < 1e-12,
        "{avant} → {apres}"
    );
    assert_eq!(app.store.viewport(), vue);
}

/// **`Retour arrière` rend « Par défaut »**, la roue restant ouverte.
#[test]
fn roue_1_retour_arriere_rend_par_defaut() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    presser_sur_le_disque(&mut app, (0.5, 0.2));
    assert!(couleur(&app).is_some());
    lacher(&mut app);
    touche(&mut app, NamedKey::Backspace);
    assert_eq!(couleur(&app), None);
    assert!(app.ui.roue.is_some());
}

/// **Un clic hors de la roue la valide**, sans désélectionner : il ne va pas plus loin.
#[test]
fn roue_1_un_clic_dehors_valide_sans_deselectionner() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    presser_sur_le_disque(&mut app, (-0.4, 0.4));
    lacher(&mut app);
    let posee = couleur(&app);
    presser(&mut app, (5.0, 300.0));
    lacher(&mut app);
    assert!(app.ui.roue.is_none());
    assert_eq!(couleur(&app), posee);
    assert!(app.store.selected_annotation_ids.contains("m"));
    assert!(app.store.can_undo(), "validée : un geste à défaire");
}

/// **Sans rien de colorable, la roue ne s'ouvre pas.**
#[test]
fn roue_1_rien_de_colorable_pas_de_roue() {
    let mut app = app();
    app.store.clear_selection();
    assert!(!app.ouvrir_la_roue());
}

/// **La case de la roue, au bout de la barre de couleur, l'ouvre** — au clic comme au doigt.
#[test]
fn roue_1_la_case_de_la_barre_ouvre_la_roue() {
    let mut app = app();
    let e = ecran(&app);
    let barre = crate::ui::couleurs::layout_barre_de_couleur(
        &app.store,
        &app.renderer.typography,
        app.ui.ecran_visible((e.width, e.height)),
        e.scale,
    )
    .expect("la barre");
    let (x, y, w, h) = barre
        .cases
        .iter()
        .find(|c| c.roue)
        .expect("la case de la roue")
        .rect;
    let ici = Pointer {
        x: x + w / 2.0,
        y: y + h / 2.0,
    };
    presser(&mut app, (ici.x, ici.y));
    lacher(&mut app);
    assert!(app.ui.roue.is_some(), "la roue s'ouvre");
    assert_eq!(couleur(&app), None, "l'ouvrir ne colore rien");
}

/// **La roue se dessine** : au centre du disque, le gris de sa clarté ; le curseur va du clair, en
/// haut, au sombre, en bas.
#[test]
fn roue_1_la_roue_se_dessine() {
    let mut app = app();
    assert!(app.ouvrir_la_roue());
    let clarte = app.ui.roue.as_ref().expect("ouverte").place.clarte;
    let roue = plat(&app);
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let image = app.pixmap.as_ref().expect("une image");
    let pixel = |x: f32, y: f32| {
        let p = image.pixels()[y as usize * image.width() as usize + x as usize].demultiply();
        Oklch::des_octets((p.red(), p.green(), p.blue()))
    };
    let (x, y, d) = roue.disque;
    let centre = pixel(x + d / 2.0, y + d / 2.0);
    assert!(
        !centre.a_une_teinte(),
        "le centre est gris : chroma {}",
        centre.c
    );
    assert!(
        (centre.l - clarte).abs() < 0.02,
        "à la clarté de la roue : {} contre {clarte}",
        centre.l
    );
    let (cx, cy, cw, ch) = roue.curseur;
    let (haut, bas) = (
        pixel(cx + cw / 2.0, cy + 2.0),
        pixel(cx + cw / 2.0, cy + ch - 2.0),
    );
    assert!(
        haut.l > 0.9 && bas.l < 0.1,
        "du clair au sombre : {} → {}",
        haut.l,
        bas.l
    );
}
