//! **BANDE-2 — la couche du dessous ne garde rien de l'image d'avant**, et ne porte que ce
//! qu'elle écrit.
//!
//! Le défaut que ces épreuves attrapent ne se voit dans aucune image isolée : il faut
//! **deux** images. Si la seconde n'efface pas les lignes que la première avait écrites, le
//! titre d'une membrane laisse une traînée derrière lui dès que la vue glisse — et le relevé,
//! qui balaie ce que le tampon porte, enverrait la traînée à la carte avec le reste.

use super::*;
use crate::renderer::{Confie, Regard};
use glucose_core::types::{Annotation, Viewport};

const TAILLE: (u32, u32) = (800, 600);

/// Une application sans fenêtre, une membrane titrée au milieu de l'écran.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let mut m = Annotation::membrane("m", 100.0, 150.0, 600.0, 300.0);
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some("Recherche".to_string());
    }
    app.store.add_annotation(&board, m);
    app.store.clear_selection();
    app
}

fn regarder(app: &mut GlucoseApp, y: f64) {
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 0.0,
            y,
            scale: 1.0,
        },
    );
}

/// Une image de la voie graphique, à partir de l'image d'avant : rend le dessous tel que le
/// tampon le porte, et ce que le processeur confie à la carte.
fn une_image(app: &mut GlucoseApp, dessous: &mut Pixmap, precedente: &Confie) -> Confie {
    let guides = app.active_guides.clone();
    let overlay = SceneOverlay::sans_rien(&guides);
    let chrome = Chrome {
        ui: &mut app.ui,
        dock_manager: &app.dock_manager,
        dock_cache: &app.dock_cache,
        pointer: Pointer { x: 0.0, y: 0.0 },
        echelle: 1.0,
    };
    let (_, confie) = peindre_par_la_carte(
        (dessous, None),
        &mut app.renderer,
        (&app.store, precedente),
        chrome,
        (overlay, Regard::immobile()),
    );
    confie
}

/// **Deux images de suite laissent le dessous qu'une seule aurait laissé**, au bit près — et
/// il ne porte que les lignes du titre, jamais l'écran entier.
#[test]
fn test_bande_2_le_dessous_ne_garde_rien_de_l_image_d_avant() {
    let mut app = application();
    regarder(&mut app, 0.0);
    let mut dessous = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    let premiere = une_image(&mut app, &mut dessous, &Confie::default());

    assert_eq!(premiere.membranes.len(), 1, "la forme part sur la carte");
    let lignes = premiere.bandes_du_dessous.lignes();
    assert!(
        lignes > 0 && lignes * 8 < TAILLE.1,
        "le dessous ne porte que le titre : {lignes} lignes sur {}",
        TAILLE.1
    );

    // La vue glisse : le titre descend de cent cinquante pixels.
    regarder(&mut app, 150.0);
    let seconde = une_image(&mut app, &mut dessous, &premiere);

    let mut seule = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    let reference = une_image(&mut app, &mut seule, &Confie::default());
    assert!(
        dessous.data() == seule.data(),
        "le dessous garde une trace de l'image d'avant"
    );
    assert_eq!(seconde.bandes_du_dessous, reference.bandes_du_dessous);
    assert_ne!(
        seconde.bandes_du_dessous, premiere.bandes_du_dessous,
        "le titre a bougé : ses bandes aussi"
    );
}

/// **Une image de la voie processeur salit tout le dessous** : c'est le même tampon, qu'elle
/// remplit de l'image entière. Si l'arbitre repasse sur la carte, sa première image devra
/// tout effacer.
#[test]
fn test_bande_2_la_voie_processeur_salit_tout_le_dessous() {
    let mut app = application();
    regarder(&mut app, 0.0);
    app.une_image_sans_fenetre(TAILLE);
    assert_eq!(
        app.confie.bandes_du_dessous,
        crate::present::bandes::Bandes::tout(TAILLE.1)
    );
}

/// **Le liseré du passé part sur la carte** : sur la voie graphique, la couche du dessus ne le
/// porte plus — ses bords gauche et droit touchaient toutes les lignes, et la couche repartait
/// entière à chaque image —, et ce que la carte doit poser le dit. Sur la voie processeur,
/// l'image le porte toujours.
#[test]
fn test_le_lisere_du_passe_part_sur_la_carte() {
    let mut app = application();
    regarder(&mut app, 0.0);
    app.dock_manager.temps.regarde = Some(0);
    let mut dessous = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    let confie = une_image(&mut app, &mut dessous, &Confie::default());
    assert!(confie.lisere.is_some(), "la carte le posera");
    assert!(
        confie.bandes_du_dessus.lignes() < TAILLE.1,
        "le dessus ne touche plus toutes les lignes : {}",
        confie.bandes_du_dessus.lignes()
    );

    app.une_image_sans_fenetre(TAILLE);
    let image = app.pixmap.as_ref().expect("une image");
    let bord = image.pixel(1, TAILLE.1 / 2).expect("dedans");
    assert!(
        bord.red() > 150 && bord.blue() < 100,
        "l'ambre au bord : {bord:?}"
    );
}

/// Aucun panneau ouvert — une application neuve en montre un par défaut —, et pas de message
/// d'accueil, qui s'estompe d'une image à l'autre.
fn sans_panneau(app: &mut GlucoseApp) {
    app.ui.current_toast = None;
    app.dock_manager.top_tabs.clear();
    app.dock_manager.bottom_tabs.clear();
    app.dock_manager.right_tabs.clear();
}

/// Une image de la voie graphique, avec la couche du dessus qu'elle a peinte.
fn une_image_et_son_dessus(app: &mut GlucoseApp) -> (Pixmap, Confie) {
    let guides = app.active_guides.clone();
    let overlay = SceneOverlay::sans_rien(&guides);
    let chrome = Chrome {
        ui: &mut app.ui,
        dock_manager: &app.dock_manager,
        dock_cache: &app.dock_cache,
        pointer: Pointer { x: 0.0, y: 0.0 },
        echelle: 1.0,
    };
    let mut dessous = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    let (dessus, confie) = peindre_par_la_carte(
        (&mut dessous, None),
        &mut app.renderer,
        (&app.store, &Confie::default()),
        chrome,
        (overlay, Regard::immobile()),
    );
    (dessus.expect("un dessus"), confie)
}

/// **PANNEAUX-1 — un panneau part sur la carte, et la couche du dessus ne le porte pas** : la
/// Time Machine, de toute la hauteur, laisse le dessus exactement tel qu'il est sans elle — il
/// ne repart donc plus entier à chaque image —, et la carte reçoit le panneau à poser.
#[test]
fn test_panneaux_1_le_panneau_part_sur_la_carte_et_le_dessus_ne_le_porte_pas() {
    let mut app = application();
    sans_panneau(&mut app);
    regarder(&mut app, 0.0);
    let (sans, rien) = une_image_et_son_dessus(&mut app);
    assert!(rien.panneaux.is_empty());

    app.dock_manager.toggle_tab(crate::dock::TabId::Temps);
    let (avec, confie) = une_image_et_son_dessus(&mut app);
    assert_eq!(confie.panneaux.len(), 1, "la carte pose le panneau");
    let pose = confie.panneaux[0].pose;
    assert!(
        pose.hauteur > TAILLE.1 as f32 / 2.0,
        "de toute la hauteur : {pose:?}"
    );
    assert!(
        avec.data() == sans.data(),
        "la couche du dessus porte le panneau"
    );
    assert_eq!(confie.bandes_du_dessus, rien.bandes_du_dessus);
    let pixels = app
        .dock_cache
        .pixels(&confie.panneaux[0].cle)
        .expect("ses pixels, par sa clé");
    assert_eq!(
        (pixels.width() as f32, pixels.height() as f32),
        (pose.largeur, pose.hauteur),
        "la texture a la taille de sa pose : un texel par pixel"
    );
}

/// **PANNEAUX-1 — un panneau immobile garde sa clé**, donc la carte ne le reçoit qu'une fois ;
/// un panneau qui change en prend une neuve, et l'ancienne ne désigne plus rien.
#[test]
fn test_panneaux_1_un_panneau_immobile_garde_sa_cle() {
    let mut app = application();
    sans_panneau(&mut app);
    regarder(&mut app, 0.0);
    app.dock_manager.toggle_tab(crate::dock::TabId::Temps);
    let (_, premiere) = une_image_et_son_dessus(&mut app);
    regarder(&mut app, 150.0);
    let (_, seconde) = une_image_et_son_dessus(&mut app);
    assert_eq!(
        premiere.panneaux[0].cle, seconde.panneaux[0].cle,
        "la vue a glissé, le panneau n'a pas changé"
    );

    app.dock_manager.temps.regarde = Some(0);
    let (_, troisieme) = une_image_et_son_dessus(&mut app);
    assert_ne!(troisieme.panneaux[0].cle, seconde.panneaux[0].cle);
    assert_eq!(
        troisieme.panneaux[0].identite, seconde.panneaux[0].identite,
        "le même panneau"
    );
    assert!(app.dock_cache.pixels(&seconde.panneaux[0].cle).is_none());
}

/// **En mode référence, la carte ne pose aucun panneau** — ils étaient visibles et morts sur
/// cette voie (son essai du 07/10). À l'envers, hors du mode, le même panneau part sur la carte.
#[test]
fn test_le_mode_reference_ne_confie_aucun_panneau() {
    let mut app = application();
    sans_panneau(&mut app);
    app.dock_manager.toggle_tab(crate::dock::TabId::Temps);
    let (_, hors) = une_image_et_son_dessus(&mut app);
    assert_eq!(
        hors.panneaux.len(),
        1,
        "hors du mode, la carte pose le panneau"
    );
    app.ui.reference = true;
    let (_, dedans) = une_image_et_son_dessus(&mut app);
    assert!(
        dedans.panneaux.is_empty(),
        "le mode reference porte un panneau"
    );
    app.ui.reference = false;
    let (_, apres) = une_image_et_son_dessus(&mut app);
    assert_eq!(
        apres.panneaux.len(),
        1,
        "il revient quand le mode se defait"
    );
}
