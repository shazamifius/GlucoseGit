//! Les épreuves de l'animatique (fiche 67) : la case remplit l'écran, la coupe tombe à sa
//! durée, la fin rend la vue, et la lecture prend toute la main — par les vrais gestes.

use super::{vue_sur_la_case, Temps};
use crate::app::GlucoseApp;
use crate::canvas::world_to_screen;
use crate::interactions::pave::Mouvement;
use glucose_core::store::storyboard::Grille;
use glucose_core::types::Viewport;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};

/// Une application et un storyboard de huit cases ; la vue d'avant, loin d'elles.
fn app() -> (GlucoseApp, String, Vec<String>) {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .poser_le_storyboard(&board, Grille::default(), (100.0, 100.0));
    let vue = Viewport {
        x: -3000.0,
        y: 250.0,
        scale: 0.4,
    };
    app.store.set_viewport(&board, vue);
    let ids = app
        .store
        .cases(&board)
        .iter()
        .map(|c| c.id.clone())
        .collect();
    (app, board, ids)
}

fn rang(app: &GlucoseApp) -> Option<usize> {
    app.ui.lecture.as_ref().map(|l| l.rang)
}

/// Fait comme si la case montrée s'était montrée `ms` de plus.
fn vieillir(app: &mut GlucoseApp, ms: i64) {
    let l = app.ui.lecture.as_mut().expect("une lecture");
    if let Temps::Depuis(t) = &mut l.temps {
        *t -= ms;
    }
}

fn touche(app: &mut GlucoseApp, nommee: NamedKey) {
    assert!(app.prendre_la_touche(&Key::Named(nommee), true, None));
}

fn cliquer(app: &mut GlucoseApp, (x, y): (f32, f32)) {
    app.handle_cursor_moved(PhysicalPosition::new(f64::from(x), f64::from(y)));
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_mouse_down(MouseButton::Left, w, h);
    app.handle_mouse_up(MouseButton::Left);
}

/// **Le temps d'une case** : ce qui reste, la pause qui l'arrête, la reprise là où il en était.
#[test]
fn test_le_temps_d_une_case_se_suspend_et_reprend() {
    let t = Temps::Depuis(1000);
    assert_eq!(t.reste(2000, 1500), Some(1500));
    let pause = t.basculer(1500);
    assert_eq!(pause, Temps::EnPause(500));
    assert_eq!(pause.reste(2000, 9000), None, "en pause, rien ne presse");
    let reprise = pause.basculer(9000);
    assert_eq!(
        reprise.reste(2000, 9000),
        Some(1500),
        "reprise là où elle était"
    );
}

/// **La case remplit l'écran à ses proportions, centrée** : ses bords touchent l'écran dans la
/// dimension qui la limite, et son centre est au centre.
#[test]
fn test_la_case_remplit_l_ecran_a_ses_proportions() {
    let cadre = (100.0, 50.0, 320.0, 180.0);
    for ecran in [(0.0, 0.0, 1440.0, 900.0), (0.0, 24.0, 412.0, 860.0)] {
        let vue = vue_sur_la_case(cadre, ecran);
        let (gx, gy) = world_to_screen(cadre.0, cadre.1, &vue);
        let (dx, dy) = world_to_screen(cadre.0 + cadre.2, cadre.1 + cadre.3, &vue);
        let (cx, cy) = ((gx + dx) / 2.0, (gy + dy) / 2.0);
        assert!((cx - (ecran.0 + ecran.2 / 2.0)).abs() < 1e-9);
        assert!((cy - (ecran.1 + ecran.3 / 2.0)).abs() < 1e-9);
        let pleine = (dx - gx - ecran.2).abs() < 1e-9 || (dy - gy - ecran.3).abs() < 1e-9;
        assert!(
            pleine,
            "une dimension touche les bords de l'écran : {ecran:?}"
        );
        assert!(
            dx - gx <= ecran.2 + 1e-9 && dy - gy <= ecran.3 + 1e-9,
            "elle tient"
        );
    }
}

/// **« ▶ Lire » depuis la case choisie** : l'interface se cache, la vue cadre cette case ; à sa
/// durée, la coupe montre la suivante, qui a la sienne ; après la dernière, la vue d'avant
/// revient, et l'interface.
#[test]
fn test_la_lecture_coupe_a_chaque_duree_puis_rend_la_vue() {
    let (mut app, board, ids) = app();
    let avant = app.store.viewport();
    app.store.regler_la_duree_du_storyboard(&board, 1000);
    app.store.regler_la_duree(&board, &ids[7], Some(300));
    app.lire_le_storyboard(Some(&ids[6]));
    assert_eq!(rang(&app), Some(6));
    assert!(app.ui.sans_interface() && app.ui.header_height() == 0.0);
    let (w, h) = app.taille_de_la_fenetre();
    let septieme = app.store.cases(&board)[6].clone();
    let attendue = vue_sur_la_case(
        (septieme.x, septieme.y, septieme.width, septieme.height),
        (0.0, 0.0, f64::from(w), f64::from(h)),
    );
    assert_eq!(app.store.viewport(), attendue, "la case remplit l'écran");

    let reste = app.attente_de_la_lecture().expect("elle attend sa coupe");
    assert!(
        (990..=1000).contains(&reste),
        "la durée du storyboard : {reste}"
    );
    vieillir(&mut app, 1000);
    assert_eq!(
        app.attente_de_la_lecture(),
        Some(300),
        "la coupe, et la durée de la huitième"
    );
    assert_eq!(rang(&app), Some(7));
    vieillir(&mut app, 300);
    app.attente_de_la_lecture();
    assert!(app.ui.lecture.is_none(), "après la dernière, la fin");
    assert_eq!(app.store.viewport(), avant, "la vue d'avant");
    assert!(!app.ui.sans_interface());
}

/// **Espace suspend** : la boucle dort, sans coupe ; Espace reprend là où elle en était.
#[test]
fn test_espace_suspend_et_reprend() {
    let (mut app, _, _) = app();
    app.lire_le_storyboard(None);
    vieillir(&mut app, 500);
    touche(&mut app, NamedKey::Space);
    assert_eq!(
        app.attente_de_la_lecture(),
        None,
        "en pause, la boucle dort"
    );
    vieillir(&mut app, 60_000);
    touche(&mut app, NamedKey::Space);
    let reste = app.attente_de_la_lecture().expect("reprise");
    assert!(
        (1490..=1500).contains(&reste),
        "là où elle en était : {reste}"
    );
    assert_eq!(rang(&app), Some(0));
}

/// **Les flèches passent, Échap finit** ; avant la première, la lecture reste sur elle.
#[test]
fn test_les_fleches_passent_et_echap_rend_la_vue() {
    let (mut app, _, _) = app();
    let avant = app.store.viewport();
    app.lire_le_storyboard(None);
    touche(&mut app, NamedKey::ArrowLeft);
    assert_eq!(rang(&app), Some(0), "avant la première : elle reste");
    touche(&mut app, NamedKey::ArrowRight);
    touche(&mut app, NamedKey::ArrowRight);
    assert_eq!(rang(&app), Some(2));
    touche(&mut app, NamedKey::Home);
    assert_eq!(rang(&app), Some(0));
    touche(&mut app, NamedKey::Escape);
    assert!(app.ui.lecture.is_none());
    assert_eq!(app.store.viewport(), avant);
}

/// **Au toucher : le tiers gauche revient, le reste avance, la croix finit** — par le vrai
/// clic ; le bouton droit n'ouvre aucun menu.
#[test]
fn test_le_clic_revient_avance_ou_finit() {
    let (mut app, _, _) = app();
    let (w, h) = app.taille_de_la_fenetre();
    app.lire_le_storyboard(None);
    cliquer(&mut app, (w * 0.8, h / 2.0));
    cliquer(&mut app, (w * 0.8, h / 2.0));
    assert_eq!(rang(&app), Some(2));
    cliquer(&mut app, (w * 0.1, h / 2.0));
    assert_eq!(rang(&app), Some(1));
    app.mouse_pos = (f64::from(w / 2.0), f64::from(h / 2.0));
    app.handle_mouse_down(MouseButton::Right, w, h);
    app.handle_mouse_up(MouseButton::Right);
    assert!(
        app.ui.context_menu_at.is_none(),
        "aucun menu sous la lecture"
    );
    let (x, y, cw, ch) = crate::renderer::lecture::croix(w, 0.0, app.ui.scale());
    cliquer(&mut app, (x + cw / 2.0, y + ch / 2.0));
    assert!(app.ui.lecture.is_none(), "la croix finit");
}

/// **La lecture tient la caméra et le clavier** : la molette ne bouge rien, une touche ne
/// devient pas un raccourci — `Suppr` ne retire pas la case choisie.
#[test]
fn test_la_lecture_prend_toute_la_main() {
    let (mut app, board, ids) = app();
    app.ui.storyboard.choisie = Some(ids[0].clone());
    app.lire_le_storyboard(Some(&ids[0]));
    let cadree = app.store.viewport();
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 3.0));
    let (w, h) = app.taille_de_la_fenetre();
    app.bouger_la_camera(w as u32, h as u32);
    assert_eq!(app.store.viewport(), cadree, "la molette ne bouge rien");
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Delete), true, None));
    assert!(app.prendre_la_touche(&Key::Character("t".into()), true, Some("t")));
    assert_eq!(app.store.cases(&board).len(), 8, "Suppr n'a rien retiré");
}

/// **Une fenêtre qui change de taille recadre la case** à l'image suivante.
#[test]
fn test_la_case_suit_la_taille_de_l_ecran() {
    let (mut app, board, _) = app();
    app.lire_le_storyboard(None);
    app.bouger_la_camera(400, 800);
    let premiere = app.store.cases(&board)[0].clone();
    let attendue = vue_sur_la_case(
        (premiere.x, premiere.y, premiere.width, premiere.height),
        (0.0, 0.0, 400.0, 800.0),
    );
    assert_eq!(app.store.viewport(), attendue);
}

/// **Le réveil de la boucle attend la coupe**, sous `Animation` ; en pause, rien ne réveille.
#[test]
fn test_la_boucle_dort_jusqu_a_la_coupe() {
    let (mut app, _, _) = app();
    app.lire_le_storyboard(None);
    let delai = app.prochain_reveil().expect("la coupe réveille");
    assert!(delai <= 2000);
    // Le réveil lui-même fait la coupe : la lecture est l'une de ses attentes.
    vieillir(&mut app, 2000);
    app.prochain_reveil();
    assert_eq!(rang(&app), Some(1), "le réveil a coupé");
    touche(&mut app, NamedKey::Space);
    app.attente_de_la_lecture();
    assert_eq!(app.attente_de_la_lecture(), None);
}

/// La vue qui cadre la première case dans un écran de 400 × 800.
fn sur_la_premiere(app: &GlucoseApp, board: &str) -> Viewport {
    let c = app.store.cases(board)[0].clone();
    vue_sur_la_case((c.x, c.y, c.width, c.height), (0.0, 0.0, 400.0, 800.0))
}

/// **Pendant une lecture, la caméra est la sienne** (fiche 68) : deux doigts qui pincent — au
/// téléphone, ou au pavé — ne la décadrent pas ; et **rien ne reste dû** : Échap rend la vue
/// d'avant, exactement, au lieu de la faire partir de tout ce qu'on a pincé.
#[test]
fn test_les_doigts_ne_decadrent_pas_la_lecture_et_rien_ne_reste_du() {
    let (mut app, board, _) = app();
    let avant = app.store.viewport();
    app.lire_le_storyboard(None);
    app.toucher.attendre(Mouvement {
        echelle: 1.5,
        decalage: (120.0, -40.0),
    });
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), sur_la_premiere(&app, &board));
    touche(&mut app, NamedKey::Escape);
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), avant, "la vue d'avant, et rien de dû");
}

/// **Ce qui était dû avant la lecture ne part pas à sa fin**, ni ce que la molette demande
/// pendant (fiche 68).
#[test]
fn test_ce_qu_on_demande_avant_ou_pendant_ne_part_pas_a_la_fin() {
    let (mut app, board, _) = app();
    let avant = app.store.viewport();
    app.elan.placer_pan(400.0, 120.0);
    app.lire_le_storyboard(None);
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), sur_la_premiere(&app, &board));
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 3.0));
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), sur_la_premiere(&app, &board));
    touche(&mut app, NamedKey::Escape);
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), avant, "la vue d'avant, et rien de dû");
}

/// **Un vol en route ne reprend pas à la fin** (fiche 68) : `F`, un signet, la minimap lancés
/// juste avant « ▶ Lire » — la lecture le pose, et Échap rend la vue d'avant, immobile.
#[test]
fn test_un_vol_en_route_ne_reprend_pas_a_la_fin() {
    let (mut app, _, _) = app();
    let avant = app.store.viewport();
    app.vol.voler_vers(Viewport {
        x: 5000.0,
        y: -800.0,
        scale: 2.0,
    });
    app.lire_le_storyboard(None);
    app.bouger_la_camera(400, 800);
    touche(&mut app, NamedKey::Escape);
    // L'horloge d'une épreuve n'avance pas : un vol vivant ne déplacerait rien ici, mais
    // partirait à la première vraie image. C'est donc son état qui se lit.
    assert!(!app.vol.en_cours(), "le vol est posé");
    app.bouger_la_camera(400, 800);
    assert_eq!(app.store.viewport(), avant, "la vue d'avant, immobile");
}
