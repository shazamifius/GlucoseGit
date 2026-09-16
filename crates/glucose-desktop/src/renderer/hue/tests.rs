//! HALO-4 — la teinte se calcule depuis un voisinage demandé à l'index.

use super::*;
use glucose_core::types::Board;

fn card(id: &str, x: f64, y: f64) -> Annotation {
    crate::renderer::card::tests::probe_card(id, x, y)
}

/// Un tableau portant ces cartes, et l'index synchronisé dessus.
fn scene(cartes: Vec<Annotation>) -> (Board, SpatialHash) {
    let mut board = Board::new("b", "B");
    board.annotations = cartes;
    let mut index = SpatialHash::new(1000.0);
    index.index_board(&board);
    (board, index)
}

/// **La teinte ne dépend pas de la générosité de l'appelant.**
///
/// C'est ce qui autorise à remplacer le tableau entier par ce qu'un index spatial rend : le
/// calcul filtre lui-même par distance exacte, donc tout sur-ensemble du rayon donne le même
/// nombre. Si ce test tombe, le passage par l'index a changé des couleurs à l'écran.
#[test]
fn test_le_voisinage_de_lindex_donne_la_meme_teinte_que_le_tableau_entier() {
    let cartes = vec![
        card("T1", 0.0, 0.0),
        card("T2", 500.0, 0.0),
        card("T3", 3_000.0, 0.0),
        card("T4", -800.0, 600.0),
    ];
    let (board, index) = scene(cartes.clone());
    let mut cache = SymbioticHueCache::new();
    cache.suivre(1, "b");

    for carte in &cartes {
        let par_lindex = cache.get_or_compute(carte, &index, &board).0;
        let par_le_tableau = glucose_core::symbiotic_hue::get_symbiotic_hue(carte, &cartes);
        assert_eq!(
            par_lindex,
            par_le_tableau,
            "la teinte de {} doit être la même par les deux chemins",
            carte.id()
        );
    }
}

/// Une version inchangée ne recalcule aucune teinte.
///
/// Sans ce compteur, un cache qui ne cache rien rendrait exactement les mêmes teintes et
/// passerait tous les autres tests — le défaut ne serait visible qu'au chronomètre.
#[test]
fn test_une_version_inchangee_ne_recalcule_rien() {
    let t1 = card("T1", 0.0, 0.0);
    let t2 = card("T2", 500.0, 0.0);
    let (board, index) = scene(vec![t1.clone(), t2.clone()]);
    let mut cache = SymbioticHueCache::new();

    for _ in 0..50 {
        cache.suivre(7, "b");
        let _ = cache.get_or_compute(&t1, &index, &board);
        let _ = cache.get_or_compute(&t2, &index, &board);
    }
    assert_eq!(
        cache.calculs(),
        2,
        "cinquante images, deux teintes calculées"
    );
}

/// Une mutation oublie les teintes : elles sont fonction des positions, qui ont pu changer.
#[test]
fn test_une_mutation_oublie_les_teintes() {
    let t1 = card("T1", 0.0, 0.0);
    let (board, index) = scene(vec![t1.clone()]);
    let mut cache = SymbioticHueCache::new();

    cache.suivre(1, "b");
    let avant = cache.get_or_compute(&t1, &index, &board);
    cache.suivre(2, "b");
    let apres = cache.get_or_compute(&t1, &index, &board);

    assert_eq!(
        cache.calculs(),
        2,
        "la version a changé, la teinte se refait"
    );
    assert_eq!(avant, apres, "rien n'a bougé : la teinte est la même");
}

/// Changer de tableau aussi : les teintes gardées ne sont pas les siennes.
#[test]
fn test_changer_de_tableau_oublie_les_teintes() {
    let t1 = card("T1", 0.0, 0.0);
    let (board, index) = scene(vec![t1.clone()]);
    let mut cache = SymbioticHueCache::new();

    cache.suivre(1, "b");
    let _ = cache.get_or_compute(&t1, &index, &board);
    cache.suivre(1, "autre");
    let _ = cache.get_or_compute(&t1, &index, &board);

    assert_eq!(cache.calculs(), 2);
}

/// Une carte lointaine n'entre pas dans le voisinage — et n'a donc pas à être trouvée pour
/// que la teinte soit juste. C'est le pendant du premier test : le sous-ensemble utile est
/// bien celui du rayon, et pas le document.
#[test]
fn test_une_carte_hors_rayon_ninfluence_pas_la_teinte() {
    let t1 = card("T1", 0.0, 0.0);
    let lointaine = card("loin", 50_000.0, 0.0);

    let (seule, index_seule) = scene(vec![t1.clone()]);
    let (avec, index_avec) = scene(vec![t1.clone(), lointaine]);

    let mut a = SymbioticHueCache::new();
    a.suivre(1, "b");
    let mut b = SymbioticHueCache::new();
    b.suivre(1, "b");

    assert_eq!(
        a.get_or_compute(&t1, &index_seule, &seule),
        b.get_or_compute(&t1, &index_avec, &avec)
    );
}
