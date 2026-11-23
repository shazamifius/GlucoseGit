//! **Les flèches d'un texte qu'on écrit partent du bord qu'on voit** (FLECHE-EDITION, sa note du
//! 23/11 au soir : *« toujours et encore le même problème »*). Pendant la saisie, le texte
//! grandit à chaque ligne et le document ne l'apprend qu'à la fin : ses flèches partaient de
//! l'ancien bord et traversaient le texte.

use crate::app::GlucoseApp;
use glucose_core::arrow as noyau;
use glucose_core::types::Annotation;

/// Un texte d'une ligne en `(0, 0)`, relié à un point loin dessous : sa flèche part de son bas.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("carte", 0.0, 0.0, "un"));
    let mut fleche = Annotation::arrow("f", 0.0, 0.0, 120.0, 3000.0);
    if let Annotation::Arrow { source_id, .. } = &mut fleche {
        *source_id = Some("carte".to_string());
    }
    app.store.add_annotation(&board, fleche);
    app.store.clear_selection();
    app.store.journal.clear();
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// L'écart entre le départ de la flèche — tel que le dessin, le clic et les poignées le lisent —
/// et le bas `bas` du texte : une flèche naît un peu hors de son nœud.
fn ecart(app: &GlucoseApp, bas: f64) -> f64 {
    let board = app.store.active_board().expect("un tableau");
    let noeuds = super::NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: Some(app.renderer.contournement()),
    };
    let ann = board.annotations.iter().find(|a| a.id() == "f");
    noyau::path_with(ann.expect("la flèche"), noeuds).expect("un chemin")[0].1 - bas
}

/// Le bas du texte tel que le document le connaît.
fn bas_du_document(app: &GlucoseApp) -> f64 {
    let board = app.store.active_board().expect("un tableau");
    let carte = board.annotations.iter().find(|a| a.id() == "carte");
    carte.and_then(Annotation::rect).expect("la carte").bottom()
}

/// **Huit lignes tapées : la flèche part du nouveau bas du texte**, pas de l'ancien — et, la
/// saisie validée, elle part du même endroit (le document a appris la hauteur).
#[test]
fn test_fleche_edition_la_fleche_suit_le_texte_qui_grandit() {
    let mut app = application();
    let avant = bas_du_document(&app);
    let naissance = ecart(&app, avant);
    app.start_text_edit("carte".to_string(), "un".to_string());
    let session = app.editing_session.as_mut().expect("la saisie");
    session.buffer = "un\ndeux\ntrois\nquatre\ncinq\nsix\nsept\nhuit".to_string();
    app.une_image_sans_fenetre((1200, 800));

    let montree = app
        .renderer
        .boite_vivante
        .clone()
        .expect("la boîte montrée")
        .1;
    assert!(
        montree.bottom() > avant + 100.0,
        "le texte n'a pas grandi à l'écran : l'épreuve ne distinguerait rien"
    );
    let pendant = ecart(&app, montree.bottom());
    assert!(
        (pendant - naissance).abs() < 1.0,
        "pendant la saisie, la flèche part à {pendant} du bas qu'on voit, au lieu de {naissance}"
    );

    app.commit_editing();
    app.une_image_sans_fenetre((1200, 800));
    assert!(app.renderer.boite_vivante.is_none(), "rien hors saisie");
    let bas = bas_du_document(&app);
    assert!(
        (bas - montree.bottom()).abs() < 1.0,
        "validé, le texte n'a pas la hauteur qu'il montrait : {bas} contre {}",
        montree.bottom()
    );
    let apres = ecart(&app, bas);
    assert!(
        (apres - naissance).abs() < 1.0,
        "validée, la flèche part à {apres} du bas, au lieu de {naissance}"
    );
}
