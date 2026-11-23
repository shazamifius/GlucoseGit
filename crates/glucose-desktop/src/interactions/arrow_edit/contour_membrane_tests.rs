//! **FLECHE-7 par les vraies pièces de l'application** : ce que contient un bout de la flèche
//! n'est pas un obstacle. Vers une membrane, elle va droit à son bord, sans contourner les nœuds
//! qu'elle porte — sa capture du 23/11 montrait le contraire (fiche 134).

use super::contour_tests::{carte, chemin, traverse};
use crate::app::GlucoseApp;
use glucose_core::arrow::contour::{self, ECART};
use glucose_core::geometry::Rect;
use glucose_core::types::Annotation;

/// La membrane « m », visée par la flèche « f » de la carte « s ».
const MEMBRANE: (f64, f64, f64, f64) = (400.0, -270.0, 600.0, 600.0);
/// Un nœud qu'elle porte, en travers du trajet droit de « s » au centre de « m », sans couvrir ce
/// centre : un obstacle qui contient un bout ne se contournait déjà pas.
const DEDANS: (f64, f64, f64, f64) = (450.0, -20.0, 150.0, 100.0);

/// La carte « s », la membrane « m » qui porte « dedans », la flèche « f » de l'une à l'autre
/// (de « m » à « s » si `depuis_la_membrane`), et ce que `poser` ajoute.
fn scene(poser: &[Annotation], depuis_la_membrane: bool) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
    }
    let ((mx, my, mw, mh), (dx, dy, dw, dh)) = (MEMBRANE, DEDANS);
    let mut fleche = Annotation::arrow("f", 50.0, 30.0, mx + mw / 2.0, my + mh / 2.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut fleche
    {
        let (source, cible) = if depuis_la_membrane {
            ("m", "s")
        } else {
            ("s", "m")
        };
        *source_id = Some(source.into());
        *target_id = Some(cible.into());
    }
    let scene = [
        carte("s", (0.0, 0.0), (100.0, 60.0)),
        Annotation::membrane("m", mx, my, mw, mh),
        carte("dedans", (dx, dy), (dw, dh)),
        fleche,
    ];
    for a in scene.into_iter().chain(poser.iter().cloned()) {
        app.store.add_annotation(&board, a);
    }
    app.store.journal.clear();
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// **Vers une membrane, la flèche va droit à son bord**, et l'ancienne loi — ce qu'elle porte
/// compté comme obstacle — faisait un détour sur les mêmes données.
#[test]
fn test_fleche_7_vers_une_membrane_elle_va_droit_a_son_bord() {
    let c = chemin(&scene(&[], false));
    assert_eq!(c.len(), 2, "aucune étape : {c:?}");
    let (bout, bord) = (c[1], MEMBRANE.0);
    assert!(
        bout.0 <= bord && bout.0 >= bord - 2.0 * ECART,
        "au bord gauche : {bout:?}"
    );
    assert!(
        (bout.1 - 30.0).abs() < 1.0,
        "sur le trajet droit : {bout:?}"
    );
    // La preuve à l'envers : « dedans » compté, le même trajet faisait un détour.
    let (dx, dy, dw, dh) = DEDANS;
    let mut avec_dedans = |_: Rect, sortie: &mut Vec<Rect>| sortie.push(Rect::new(dx, dy, dw, dh));
    let ancien = contour::itineraire((50.0, 30.0), (700.0, 30.0), &mut avec_dedans);
    assert!(!ancien.is_empty(), "l'ancienne loi allait droit aussi");
}

/// **Un nœud à cheval sur le bord se contourne encore** : il n'est pas *dans* la membrane, et la
/// flèche le traverserait avant d'arriver.
#[test]
fn test_fleche_7_un_noeud_a_cheval_sur_le_bord_se_contourne_encore() {
    const BORD: (f64, f64, f64, f64) = (340.0, -10.0, 120.0, 80.0);
    let c = chemin(&scene(
        &[carte("bord", (BORD.0, BORD.1), (BORD.2, BORD.3))],
        false,
    ));
    assert!(c.len() > 2, "un détour : {c:?}");
    for s in c.windows(2) {
        assert!(
            !traverse(s[0], s[1], BORD),
            "{s:?} traverse le nœud du bord"
        );
    }
}

/// **Depuis une membrane aussi** : ce que porte la source n'est pas un obstacle — la flèche part
/// de son bord, droit vers sa cible.
#[test]
fn test_fleche_7_depuis_une_membrane_elle_part_droit_de_son_bord() {
    let c = chemin(&scene(&[], true));
    assert_eq!(c.len(), 2, "aucune étape : {c:?}");
    let depart = c[0];
    assert!(
        depart.0 <= MEMBRANE.0 && depart.0 >= MEMBRANE.0 - 2.0 * ECART,
        "du bord gauche : {depart:?}"
    );
}
