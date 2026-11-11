//! **RETOUR-2 — `Alt+Z` est le `Ctrl+Z` de la caméra**, par les vrais chemins : la molette et la
//! main passent par l'élan, et c'est l'image qui dit quand la vue bouge.
//!
//! **Le temps se joue** : un geste et le suivant sont séparés, ou non, par plus qu'un double-clic
//! (`DBLCLICK_MS`). Les images d'une épreuve ne durent rien ; [`attendre`] fait passer le temps de
//! l'horloge des clics, celle que la loi lit (fiche 103 § 2).

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use glucose_core::hit_priority::pick_consts::DBLCLICK_MS;
use glucose_core::types::Viewport;
use winit::event::MouseScrollDelta;

/// Une application au tableau vide, l'accueil éteint, la vue posée en un lieu connu, et une image
/// au repos pour que la vue précédente soit celle-là.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    app.store.set_viewport(&tableau, depart());
    image(&mut app);
    app
}

fn depart() -> Viewport {
    Viewport {
        x: 40.0,
        y: 30.0,
        scale: 1.0,
    }
}

/// Une image comme la boucle la joue, une période d'écran à 240 Hz après la précédente.
fn image(app: &mut GlucoseApp) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    app.horloge.presentee(t + periode, true);
    app.appliquer_l_elan(800, 600);
}

/// **Le temps passe** de `ms` millisecondes sur l'horloge des clics, sans aucune image : la main
/// s'arrête, l'œil regarde.
fn attendre(app: &mut GlucoseApp, ms: i64) {
    let ms = u64::try_from(ms).expect("une durée positive");
    app.click_epoch = app
        .click_epoch
        .checked_sub(std::time::Duration::from_millis(ms))
        .expect("une horloge assez ancienne");
}

/// Plus qu'un double-clic : deux gestes séparés de ce temps en sont deux.
const PAUSE: i64 = DBLCLICK_MS * 2;

/// Des images jusqu'à ce que la vue s'arrête, et une de plus, immobile.
fn jusqu_au_repos(app: &mut GlucoseApp) {
    for _ in 0..2_000 {
        image(app);
        if !app.elan.bouge() && !app.vol.en_cours() {
            image(app);
            return;
        }
    }
    panic!("la vue ne s'arrête pas");
}

/// Un cran de molette, l'image qui le montre, puis deux images immobiles — ce que le vrai
/// Glucose dessine entre deux crans d'une même main (les textures qui reviennent nettes).
fn cran(app: &mut GlucoseApp) {
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
    image(app);
    image(app);
    image(app);
}

/// La main déplace la vue (le bouton du milieu, ou un doigt seul), et l'image le montre.
fn glisser(app: &mut GlucoseApp, dx: f64) {
    app.is_panning = true;
    app.handle_pan_move(dx, 12.0);
    image(app);
}

fn alt_z(app: &mut GlucoseApp) -> bool {
    app.executer(Commande::VuePrecedente)
}

/// **Un tour de molette se défait** : `Alt+Z` rend la vue d'avant, au bit près.
#[test]
fn test_retour_2_un_tour_de_molette_se_defait() {
    let mut app = app();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    assert_ne!(app.store.viewport(), depart(), "la molette a zoomé");
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
}

/// **Une rafale ne fait qu'un pas**, même quand des images immobiles séparent ses crans — le cas
/// que le pilote a montré : cinq crans faisaient cinq pas (fiche 103 § 2).
#[test]
fn test_retour_2_une_rafale_ne_fait_qu_un_pas_malgre_les_images_immobiles() {
    let mut app = app();
    for _ in 0..5 {
        cran(&mut app);
        attendre(&mut app, DBLCLICK_MS / 4);
    }
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}

/// **Deux gestes séparés de plus qu'un double-clic, deux pas** : la molette, l'œil qui regarde,
/// puis la main qui déplace la vue. `Alt+Maj+Z` repart.
#[test]
fn test_retour_2_deux_gestes_separes_deux_pas() {
    let mut app = app();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    let apres_la_molette = app.store.viewport();
    attendre(&mut app, PAUSE);
    glisser(&mut app, 30.0);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert_ne!(
        app.store.viewport(),
        apres_la_molette,
        "la main a déplacé la vue"
    );
    attendre(&mut app, PAUSE);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), apres_la_molette, "d'abord le glisser");
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart(), "puis la molette");
    assert!(app.executer(Commande::VueSuivante));
    assert_eq!(app.store.viewport(), apres_la_molette, "Alt+Maj+Z repart");
}

/// **Un glisser tenu est un seul geste**, même quand la main s'arrête plus qu'un double-clic
/// avant de repartir : tant qu'elle tient la vue, le pas reste ouvert.
#[test]
fn test_retour_2_un_glisser_tenu_qui_s_arrete_reste_un_pas() {
    let mut app = app();
    glisser(&mut app, 30.0);
    jusqu_au_repos(&mut app);
    // La main tient, immobile : le vrai Glucose ne dessine aucune image pendant ce temps.
    attendre(&mut app, PAUSE);
    glisser(&mut app, 25.0);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}

/// **Tenir sans bouger ne laisse aucun lieu** : un appui sur le vide, longtemps tenu, puis lâché
/// — la vue n'a pas bougé, il n'y a nulle part où revenir.
#[test]
fn test_retour_2_tenir_sans_bouger_ne_laisse_aucun_lieu() {
    let mut app = app();
    app.is_panning = true;
    image(&mut app);
    attendre(&mut app, PAUSE);
    image(&mut app);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert!(!alt_z(&mut app), "rien où revenir");
}

/// **Un retour ferme le pas** : ce que la main fait aussitôt après `Alt+Z` est un geste neuf, qui
/// se défait seul.
#[test]
fn test_retour_2_un_retour_ferme_le_pas() {
    let mut app = app();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    attendre(&mut app, PAUSE);
    glisser(&mut app, 30.0);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    let revenu = app.store.viewport();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(
        app.store.viewport(),
        revenu,
        "le cran d'après le retour, seul"
    );
}

/// **Un vol ne se note pas deux fois** : son départ est déjà retenu (RETOUR-1).
#[test]
fn test_retour_2_un_vol_ne_se_note_pas_deux_fois() {
    let mut app = app();
    app.vol.voler_vers(Viewport {
        x: 900.0,
        y: 30.0,
        scale: 1.0,
    });
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}

/// **Lâcher puis reprendre plus tard, deux pas** : la main qui a lâché la vue ne garde plus le pas
/// ouvert, même si elle la reprend.
#[test]
fn test_retour_2_lacher_puis_reprendre_plus_tard_fait_deux_pas() {
    let mut app = app();
    glisser(&mut app, 30.0);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    let apres = app.store.viewport();
    attendre(&mut app, PAUSE);
    glisser(&mut app, 25.0);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), apres, "le second glisser");
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart(), "puis le premier");
}

/// Un doigt posé, glissé ou levé sur l'écran, par la vraie entrée des doigts.
fn doigt(app: &mut GlucoseApp, id: u64, phase: winit::event::TouchPhase, (x, y): (f64, f64)) {
    app.toucher(&winit::event::Touch {
        device_id: winit::event::DeviceId::dummy(),
        phase,
        location: winit::dpi::PhysicalPosition::new(x, y),
        force: None,
        id,
    });
}

/// Une image par la vraie boucle de la caméra : les doigts et le pavé y avancent (`bouger_la_camera`).
fn image_des_doigts(app: &mut GlucoseApp) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    app.horloge.presentee(t + periode, true);
    app.bouger_la_camera(800, 600);
}

/// **Deux doigts posés tiennent la vue** : un pincement qui s'arrête plus qu'un double-clic, les
/// doigts toujours posés, puis reprend, est un seul pas.
#[test]
fn test_retour_2_deux_doigts_poses_tiennent_le_pas() {
    use winit::event::TouchPhase::{Ended, Moved, Started};
    let mut app = app();
    doigt(&mut app, 1, Started, (300.0, 300.0));
    doigt(&mut app, 2, Started, (500.0, 300.0));
    doigt(&mut app, 2, Moved, (540.0, 300.0));
    for _ in 0..4 {
        image_des_doigts(&mut app);
    }
    assert_ne!(app.store.viewport(), depart(), "les doigts ont zoomé");
    attendre(&mut app, PAUSE);
    doigt(&mut app, 2, Moved, (580.0, 300.0));
    for _ in 0..4 {
        image_des_doigts(&mut app);
    }
    doigt(&mut app, 2, Ended, (580.0, 300.0));
    doigt(&mut app, 1, Ended, (300.0, 300.0));
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}

/// Le pavé joué : un geste en cours ou non, et ce qu'il rendra à l'image suivante.
#[derive(Clone, Default)]
struct PaveJoue(std::rc::Rc<std::cell::RefCell<(bool, Vec<crate::interactions::pave::Mouvement>)>>);

impl crate::interactions::pave::Pave for PaveJoue {
    fn en_geste(&self) -> bool {
        self.0.borrow().0
    }
    fn avancer(&mut self) -> Vec<crate::interactions::pave::Mouvement> {
        std::mem::take(&mut self.0.borrow_mut().1)
    }
}

/// **Le pavé en geste tient la vue** : les doigts posés sur le pavé, immobiles plus qu'un
/// double-clic, gardent le pas ouvert ; levés, un geste plus tard en ouvre un autre.
#[test]
fn test_retour_2_le_pave_en_geste_tient_le_pas() {
    use crate::interactions::pave::Mouvement;
    let mut app = app();
    let pave = PaveJoue::default();
    app.toucher.pave = Some(Box::new(pave.clone()));
    let glisse = Mouvement {
        echelle: 1.0,
        decalage: (20.0, 0.0),
    };
    *pave.0.borrow_mut() = (true, vec![glisse]);
    image_des_doigts(&mut app);
    attendre(&mut app, PAUSE);
    pave.0.borrow_mut().1.push(glisse);
    image_des_doigts(&mut app);
    pave.0.borrow_mut().0 = false;
    jusqu_au_repos(&mut app);
    let apres = app.store.viewport();
    attendre(&mut app, PAUSE);
    *pave.0.borrow_mut() = (true, vec![glisse]);
    image_des_doigts(&mut app);
    pave.0.borrow_mut().0 = false;
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), apres, "le geste d'après, seul");
    assert!(alt_z(&mut app));
    assert_eq!(
        app.store.viewport(),
        depart(),
        "les deux glissements tenus, un pas"
    );
}

/// **Un cran et un vol dans la même image, deux lieux, dans l'ordre** : `Alt+Z` rend d'abord la
/// vue d'où le vol est parti — après le cran —, puis celle d'avant le cran.
#[test]
fn test_retour_2_un_cran_et_un_vol_dans_la_meme_image() {
    let mut app = app();
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
    app.vol.voler_vers(Viewport {
        x: 900.0,
        y: 30.0,
        scale: 1.0,
    });
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    let la = app.store.viewport();
    assert!(
        (la.scale - 1.0).abs() > 0.01 && la.x != 900.0,
        "d'abord d'où le vol est parti, le cran fait : {la:?}"
    );
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart(), "puis d'avant le cran");
}
