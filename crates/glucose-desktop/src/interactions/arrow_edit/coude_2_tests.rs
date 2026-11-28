//! **COUDE-2 — une flèche pliée contourne entre ses coudes** (fiche 191) : un coude est un point
//! de passage, pas une ligne droite. Le 27/11, une flèche qu'il avait pliée zigzaguait au milieu
//! des textes de sa bulle.

use super::contour_tests::{application, chemin, coudes, traverse};
use glucose_core::arrow::{Bout, Itineraire};

/// La carte « o » de [`application`] : (250, -40), 100 × 140.
const O: (f64, f64, f64, f64) = (250.0, -40.0, 100.0, 140.0);

/// Un coude hors de la carte « o », d'où le tronçon droit vers la source la traverserait.
const COUDE: (f64, f64) = (450.0, 60.0);

fn pliee() -> crate::app::GlucoseApp {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.insert_arrow_bend(&board, "f", 0, COUDE);
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// **Entre ses coudes, elle contourne** : elle passe par le coude posé, et aucun tronçon ne
/// traverse la carte.
#[test]
fn test_coude_2_une_fleche_pliee_contourne_entre_ses_coudes() {
    let app = pliee();
    let c = chemin(&app);
    assert!(c.contains(&COUDE), "elle passe par son coude : {c:?}");
    assert!(c.len() > 3, "un détour avant le coude : {c:?}");
    for paire in c.windows(2) {
        assert!(
            !traverse(paire[0], paire[1], O),
            "un tronçon traverse la carte : {c:?}"
        );
    }
}

/// **Glisser une étape du détour fige tout le tracé** : ses coudes et ses détours deviennent
/// des coudes, l'étape tenue sous la main, le coude posé à sa place ; un seul Ctrl+Z rend le
/// seul coude posé.
#[test]
fn test_coude_2_glisser_une_etape_fige_tout_le_trace() {
    let mut app = pliee();
    let c = chemin(&app);
    let interieur = c[1..c.len() - 1].to_vec();
    let rang = interieur
        .iter()
        .position(|p| *p != COUDE)
        .expect("une étape");
    let etape = interieur[rang];
    assert!(
        app.begin_arrow_bend(etape.0, etape.1),
        "la poignée est prise"
    );
    app.update_bend(etape.0, etape.1 - 30.0);
    app.finish_bend();
    let mut attendu = interieur.clone();
    attendu[rang] = (etape.0, etape.1 - 30.0);
    assert_eq!(
        coudes(&app),
        attendu,
        "tout le tracé figé, l'étape déplacée"
    );
    app.store.undo();
    assert_eq!(coudes(&app), vec![COUDE], "un seul Ctrl+Z");
}

/// **À court de temps, un coude qu'on glisse se montre sous la main** : la mémoire rend le
/// chemin d'avant s'il a les mêmes coudes, sinon ses coudes, tout droit — jamais un chemin qui
/// passerait par la place d'avant du coude.
#[test]
fn test_coude_2_a_court_de_temps_ses_coudes_d_abord() {
    let app = application();
    let it = &app.renderer.itineraires;
    it.accorder_la_tranche(Some(std::time::Duration::ZERO));
    it.ouvrir_la_tranche();
    let bout = |x| Bout {
        point: (x, 0.0),
        boite: None,
    };
    let cle = |coudes| Itineraire {
        fleche: "z",
        depart: bout(0.0),
        arrivee: bout(500.0),
        courbe: false,
        coudes,
    };
    let avant = [(200.0, 100.0)];
    let detour = vec![(100.0, 50.0), (200.0, 100.0)];
    assert_eq!(it.retenu(&cle(&avant), &mut || detour.clone()), detour);
    let rien = &mut || panic!("plus de temps : rien ne se cherche");
    assert_eq!(it.retenu(&cle(&avant), rien), detour, "le chemin d'avant");
    let glisse = [(300.0, -100.0)];
    assert_eq!(
        it.retenu(&cle(&glisse), rien),
        glisse.to_vec(),
        "ses coudes"
    );
}
