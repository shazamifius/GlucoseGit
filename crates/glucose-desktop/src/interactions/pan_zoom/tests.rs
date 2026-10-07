//! NAV-2 — les cas que `navigation.test.ts` tient côté Tauri, tenus ici aussi.

use super::*;
use winit::dpi::PhysicalPosition;

fn lignes(x: f32, y: f32) -> MouseScrollDelta {
    MouseScrollDelta::LineDelta(x, y)
}

fn pixels(x: f64, y: f64) -> MouseScrollDelta {
    MouseScrollDelta::PixelDelta(PhysicalPosition::new(x, y))
}

/// `Ctrl` + défilement : c'est un zoom, quel que soit le reste.
#[test]
fn test_nav_2_ctrl_is_a_zoom() {
    assert!(matches!(
        geste(lignes(0.0, 1.0), true, false, false),
        Geste::Zoom(_)
    ));
    assert!(matches!(
        geste(pixels(3.0, -7.0), true, false, false),
        Geste::Zoom(_)
    ));
}

/// Le pincement zoome **même sans `Ctrl` au clavier** : c'est le systeme qui le marque, et
/// c'était tout le défaut — le geste arrivait nu, donc il déplaçait la vue.
#[test]
fn test_nav_2_un_pincement_zoome_sans_ctrl_clavier() {
    assert!(matches!(
        geste(lignes(0.0, 0.07), false, true, false),
        Geste::Zoom(_)
    ));
    assert!(matches!(
        geste(lignes(0.0, -0.07), false, true, false),
        Geste::Zoom(_)
    ));
}

/// **La nature du geste décide du gain, la touche ne décide que du sens.**
///
/// Windows encode le déclic d'une molette et la course d'un doigt dans la même grandeur
/// numérique, alors que le premier est quantifié et le second continu : un pavé en envoie des
/// dizaines d'unités par seconde. Deux gains, donc — mais **deux**, pas trois.
///
/// Le code d'avant en distinguait trois, et se trompait sur le troisième : `Ctrl` + glissement
/// à deux doigts tombait dans la branche du cran de souris et zoomait deux fois trop vite. Ce
/// sont pourtant les mêmes doigts sur le même pavé.
///
/// Le test vérifie les **rapports qui doivent tenir**, jamais les valeurs : celles-ci sont du
/// ressenti, elles se jugent à la main et doivent pouvoir bouger sans casser une preuve.
#[test]
fn test_nav_2_le_gain_suit_le_geste_et_non_la_touche() {
    // Un demi-cran : pas un nombre entier de lignes, donc jamais un déclic de molette.
    let Geste::Zoom(pince) = geste(lignes(0.0, 0.5), false, true, false) else {
        panic!("un pincement zoome");
    };
    let Geste::Zoom(ctrl_et_doigts) = geste(lignes(0.0, 0.5), true, false, false) else {
        panic!("ctrl et deux doigts zooment");
    };
    assert!(
        (pince - ctrl_et_doigts).abs() < 1e-12,
        "{pince} contre {ctrl_et_doigts} : les memes doigts sur le meme pave, seule la touche change"
    );

    // Le déclic a son propre gain — une secousse par encoche, pas une course. Lequel pèse plus
    // par unité n'est plus une loi : le doigt était lissé puis prolongé par sa glissade, qui
    // doublait son amplitude ; depuis que tout zoom est direct (fiche 52 § 4), les deux valeurs
    // se jugent chacune à la main. Ce qui reste structurel : ce sont deux gains.
    let Geste::Zoom(cran) = geste(lignes(0.0, 1.0), false, false, false) else {
        panic!("un cran de souris zoome");
    };
    assert!(
        (cran - 2.0 * pince).abs() > 1e-9,
        "{cran} octave(s) par cran contre {pince} par demi-unite de doigt : un seul gain"
    );

    // Et `Ctrl` sur une vraie molette ne la transforme pas en doigt.
    let Geste::Zoom(cran_avec_ctrl) = geste(lignes(0.0, 1.0), true, false, false) else {
        panic!("ctrl et molette zooment");
    };
    assert!(
        (cran - cran_avec_ctrl).abs() < 1e-12,
        "{cran} contre {cran_avec_ctrl} : la touche ne change pas la nature du geste"
    );
}

/// Un cran de souris — vertical pur, nombre entier de lignes — zoome.
#[test]
fn test_nav_2_a_mouse_notch_zooms() {
    assert!(matches!(
        geste(lignes(0.0, 1.0), false, false, false),
        Geste::Zoom(_)
    ));
    assert!(matches!(
        geste(lignes(0.0, -1.0), false, false, false),
        Geste::Zoom(_)
    ));
    assert!(matches!(
        geste(lignes(0.0, 3.0), false, false, false),
        Geste::Zoom(_)
    ));
}

/// Deux doigts sur le pavé tactile déplacent la vue — **y compris vers le haut et le bas**.
///
/// C'est le cas que l'ancienne version rendait inatteignable : sous Windows, un pavé tactile
/// passe par `LineDelta` comme une souris, avec des fractions de ligne.
#[test]
fn test_nav_2_two_fingers_pan_in_every_direction() {
    assert!(matches!(
        geste(lignes(0.0, 0.42), false, false, false),
        Geste::Pan(_, _)
    ));
    assert!(matches!(
        geste(lignes(0.0, -0.13), false, false, false),
        Geste::Pan(_, _)
    ));
    assert!(matches!(
        geste(lignes(0.7, 0.0), false, false, false),
        Geste::Pan(_, _)
    ));
    assert!(matches!(
        geste(pixels(0.0, 24.0), false, false, false),
        Geste::Pan(_, _)
    ));
}

/// Une composante horizontale dénonce un pavé tactile, même si le vertical tombe juste.
#[test]
fn test_nav_2_a_whole_line_with_sideways_motion_is_still_a_pan() {
    assert!(matches!(
        geste(lignes(0.5, 1.0), false, false, false),
        Geste::Pan(_, _)
    ));
}

/// Un événement vide ne fait rien plutôt que de zoomer par ×1.
#[test]
fn test_nav_2_an_empty_event_moves_nothing() {
    assert_eq!(
        geste(lignes(0.0, 0.0), false, false, false),
        Geste::Pan(0.0, 0.0)
    );
}

/// Le sens : molette vers l'avant agrandit, vers soi réduit.
#[test]
fn test_nav_2_forward_grows_and_backward_shrinks() {
    let Geste::Zoom(avant) = geste(lignes(0.0, 1.0), false, false, false) else {
        panic!("un cran doit zoomer");
    };
    let Geste::Zoom(arriere) = geste(lignes(0.0, -1.0), false, false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!(avant > 0.0, "vers l'avant, on agrandit : {avant} octave(s)");
    assert!(arriere < 0.0, "vers soi, on réduit : {arriere} octave(s)");
    // **Ce que l'octave fait gagner** : deux crans valent la somme de deux crans, et non le
    // carré d'un facteur. Le zoom devient additif, donc indépendant du découpage des
    // événements reçus -- et l'élan peut les accumuler sans rien trahir.
    let Geste::Zoom(deux) = geste(lignes(0.0, 2.0), false, false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!((deux - 2.0 * avant).abs() < 1e-12);
}

/// **Huit crans doublent.** C'est ce que l'octave permet de dire, et de vérifier.
#[test]
fn test_nav_2_huit_crans_doublent_exactement() {
    let Geste::Zoom(un) = geste(lignes(0.0, 1.0), false, false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!(
        (8.0 * un - 1.0).abs() < 1e-12,
        "huit crans doivent faire une octave pleine, ils font {}",
        8.0 * un
    );
}

/// Deux doigts vers le bas font descendre le contenu — le signe de winit est celui du monde.
#[test]
fn test_nav_2_the_content_follows_the_fingers() {
    let Geste::Pan(dx, dy) = geste(lignes(0.0, 0.5), false, false, false) else {
        panic!("un glissement doit paner");
    };
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.5 * PAN_LIGNE_PX);
}

/// **Une source ne change pas au milieu d'un geste**, et c'est ce qui ferme le dernier cas
/// ambigu : un pavé tactile qui tombe par hasard sur un nombre entier de lignes.
///
/// Mesuré en conditions réelles : six défilements sur huit cent quatre-vingt-neuf, chacun
/// coûtant un saut de zoom d'un huitième d'octave au milieu d'un glissement. Rare, mais
/// parfaitement visible — et la chronique le signalait à chaque session.
#[test]
fn un_pave_qui_tombe_sur_un_entier_ne_devient_pas_une_molette() {
    // Le delta ambigu : vertical pur, nombre entier de lignes. Hors contexte, c'est un cran.
    let ambigu = lignes(0.0, 1.0);
    let Geste::Zoom(comme_un_cran) = geste(ambigu, false, false, false) else {
        panic!("sans contexte, l'entier se lit comme un cran");
    };

    // Le même delta, alors que le geste en cours vient déjà d'un doigt : c'est un pan.
    assert!(
        matches!(geste(ambigu, false, false, true), Geste::Pan(..)),
        "le geste vient d'un pave : cet entier est une course de doigt"
    );
    assert!(comme_un_cran > 0.0);
}

/// Ce qu'une molette **ne peut pas** produire : une fraction de ligne, ou un mouvement
/// latéral. C'est ce qui dénonce un pavé sans le moindre doute, et sans aucun seuil.
#[test]
fn seul_un_delta_impossible_a_la_molette_denonce_un_pave() {
    assert!(source_continue(lignes(0.0, 0.42)), "une fraction de ligne");
    assert!(source_continue(lignes(0.7, 0.0)), "un mouvement lateral");
    assert!(source_continue(lignes(0.5, 1.0)), "les deux a la fois");
    assert!(
        source_continue(pixels(0.0, 24.0)),
        "des pixels, jamais des lignes"
    );

    // Un cran de molette, lui, reste indiscernable — et c'est exact : il l'est vraiment.
    assert!(!source_continue(lignes(0.0, 1.0)));
    assert!(!source_continue(lignes(0.0, -3.0)));
}

/// NAV-4 — **à la souris, la vue est là où la main l'a mise, à l'image suivante** (fiche 51
/// § 1). Joué dans l'application, par les deux portes réelles : un cran de molette, puis un
/// glisser au bouton du milieu. À l'envers, un défilement de pavé n'est montré qu'en partie
/// à la même image : la voie du doigt, elle, n'a pas changé.
#[test]
fn test_nav_4_la_souris_se_montre_a_l_image_suivante_et_rien_apres() {
    const ECRAN: (u32, u32) = (1280, 720);
    let mut app = crate::app::GlucoseApp::new();
    let avant = app.store.viewport().scale;
    app.handle_mouse_wheel(lignes(0.0, 1.0));
    app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    let apres = app.store.viewport().scale;
    assert!(
        (apres / avant - OCTAVES_PAR_CRAN.exp2()).abs() < 1e-12,
        "tout le cran dans l'image suivante : x{}",
        apres / avant
    );
    for _ in 0..200 {
        app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    }
    assert_eq!(app.store.viewport().scale, apres, "et rien ne glisse après");

    let x = app.store.viewport().x;
    for _ in 0..7 {
        app.handle_pan_move(3.0, 0.0);
    }
    app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    assert!(
        (app.store.viewport().x - x - 21.0).abs() < 1e-9,
        "le glisser suit au pixel"
    );
    for _ in 0..200 {
        app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    }
    assert!(
        (app.store.viewport().x - x - 21.0).abs() < 1e-9,
        "le lâcher arrête net"
    );

    let y = app.store.viewport().y;
    app.handle_mouse_wheel(pixels(0.0, 30.0));
    app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    let montre = (app.store.viewport().y - y).abs();
    assert!(
        montre < 30.0,
        "à l'envers, le pavé reste lissé : {montre} px sur 30"
    );
}

/// **Le zoom au doigt passe par la conduite** (fiche 52 § 9) : joué dans l'application, `Ctrl`
/// et un demi-cran de pavé ne se montrent pas d'un bloc — il reste une dette, que la conduite
/// rembourse au fil des images. À l'envers, la molette se montre tout entière à l'image
/// suivante (NAV-4) : c'était le « strate par strate » du pincement. Que le zoom au doigt ne
/// glisse pas après le lâcher, `elan::tests::un_pincement_ne_glisse_pas` le garde.
#[test]
fn test_le_zoom_au_doigt_passe_par_la_conduite() {
    const ECRAN: (u32, u32) = (1280, 720);
    let mut app = crate::app::GlucoseApp::new();
    let avant = app.store.viewport().scale;
    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    app.handle_mouse_wheel(lignes(0.0, 0.5));
    app.modifiers = winit::keyboard::ModifiersState::empty();
    app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    let voulu = (0.5 * OCTAVES_PAR_UNITE_DE_DOIGT).exp2();
    let montre = app.store.viewport().scale / avant;
    assert!(
        montre < voulu - 1e-6 && app.elan.en_cours(),
        "le pincement s'est montre d'un bloc : x{montre} sur x{voulu}"
    );
}
