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
