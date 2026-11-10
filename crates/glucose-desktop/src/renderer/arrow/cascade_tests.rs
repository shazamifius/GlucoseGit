//! **ITINERAIRE-2 — le rendu n'attend pas les itinéraires** : une image dont la tranche est
//! épuisée ne cherche qu'un itinéraire et montre le détour d'avant pour les autres ; au repos, le
//! réveil paie le reste ; sans tranche, tout se cherche ; un export est entier.

use crate::app::GlucoseApp;
use glucose_core::types::Annotation;
use std::time::Duration;

const FLECHES: usize = 20;

/// Une carte de texte de taille connue.
fn carte(id: &str, (x, y): (f64, f64), (w, h): (f64, f64)) -> Annotation {
    let mut a = Annotation::text(id, x, y, "carte");
    if let Annotation::Text { width, height, .. } = &mut a {
        (*width, *height) = (Some(w), Some(h));
    }
    a
}

/// Vingt flèches de « s » à « t », et « o » sur leur trajet droit : chacune le contourne. Tout
/// est cherché une première fois, sans tranche.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    for a in [
        carte("s", (0.0, 0.0), (100.0, 60.0)),
        carte("t", (600.0, 0.0), (100.0, 60.0)),
        carte("o", (250.0, -40.0), (100.0, 140.0)),
    ] {
        app.store.add_annotation(&board, a);
    }
    for k in 0..FLECHES {
        let mut f = Annotation::arrow(format!("f{k}"), 50.0, 30.0, 650.0, 30.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            (*source_id, *target_id) = (Some("s".into()), Some("t".into()));
        }
        app.store.add_annotation(&board, f);
    }
    app.store.clear_selection();
    app.store.journal.clear();
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// Le chemin d'une flèche tel que le rendu le lit, avec sa mémoire.
fn chemin(app: &GlucoseApp, id: &str) -> Vec<(f64, f64)> {
    let board = app.store.active_board().expect("un tableau");
    let fleche = board
        .annotations
        .iter()
        .find(|a| a.id() == id)
        .expect("la flèche");
    let noeuds = super::NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: Some(app.renderer.contournement()),
    };
    glucose_core::arrow::path_with(fleche, noeuds).expect("un chemin")
}

/// Un nœud sans rapport ajoute une version au document : chaque itinéraire est périmé.
fn une_autre_version(app: &mut GlucoseApp) {
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, carte("loin", (5_000.0, 5_000.0), (100.0, 60.0)));
}

/// **Une image dont la tranche est épuisée ne cherche qu'un itinéraire** — au moins un : la
/// cascade avance —, et montre pour les autres **le détour d'avant**, pas une ligne droite.
#[test]
fn test_itineraire_2_une_image_ne_cherche_que_sa_tranche() {
    let mut app = application();
    let detour = chemin(&app, "f7");
    assert!(detour.len() > 2, "un détour : {detour:?}");
    app.renderer
        .itineraires
        .accorder_la_tranche(Some(Duration::ZERO));
    une_autre_version(&mut app);
    let avant = app.renderer.itineraires.recherches();
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(app.renderer.itineraires.recherches(), avant + 1, "un seul");
    assert!(app.renderer.itineraires.en_retard(), "les autres sont dus");
    let dues: Vec<String> = (0..FLECHES)
        .map(|k| format!("f{k}"))
        .filter(|id| chemin(&app, id) == detour)
        .collect();
    assert_eq!(
        dues.len(),
        FLECHES,
        "toutes montrent le détour, d'avant ou refait"
    );
}

/// **Au repos, le réveil paie le reste sans image**, une tranche à la fois, puis demande une
/// image — une seule ; après quoi plus rien n'est dû, et chaque flèche a été refaite.
#[test]
fn test_itineraire_2_au_repos_le_reveil_paie_le_reste() {
    let mut app = application();
    app.renderer
        .itineraires
        .accorder_la_tranche(Some(Duration::ZERO));
    une_autre_version(&mut app);
    let avant = app.renderer.itineraires.recherches();
    app.une_image_sans_fenetre((1200, 800));
    let mut appels = 0;
    while app.attente_des_itineraires() == Some(false) {
        appels += 1;
        assert!(appels <= FLECHES, "la cascade finit");
    }
    assert!(!app.renderer.itineraires.en_retard(), "plus rien n'est dû");
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant + FLECHES as u64,
        "chaque flèche refaite une fois"
    );
    assert_eq!(app.attente_des_itineraires(), None);
}

/// **Sans tranche, tout se cherche dans l'image** — les épreuves et les bancs restent
/// déterministes ; et rien n'est dû ensuite.
#[test]
fn test_itineraire_2_sans_tranche_tout_se_cherche() {
    let mut app = application();
    une_autre_version(&mut app);
    let avant = app.renderer.itineraires.recherches();
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant + FLECHES as u64
    );
    assert!(!app.renderer.itineraires.en_retard());
}

/// **Un export est entier** : tranche épuisée et mémoire oubliée, il est le même qu'un export
/// qui a tout son temps — chaque flèche y passe par son détour.
#[test]
fn test_itineraire_2_un_export_est_entier() {
    let mut app = application();
    let vue = app.store.viewport();
    let exporter = |app: &mut GlucoseApp| {
        app.renderer
            .rendre_en_svg((&app.store, &mut app.ui), vue, (1200, 800), &mut |_| None)
    };
    let entier = exporter(&mut app);
    app.renderer
        .itineraires
        .accorder_la_tranche(Some(Duration::ZERO));
    app.renderer.itineraires.oublier();
    assert_eq!(exporter(&mut app), entier);
}
