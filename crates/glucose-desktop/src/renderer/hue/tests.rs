//! HALO-4 — la teinte se calcule depuis un voisinage demandé à l'index.

use super::*;
use glucose_core::types::Board;
use std::time::Duration;

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

/// Trois textes voisins, et le cache qui les a déjà vus à la version 1.
fn trois_voisines() -> (Board, SpatialHash) {
    scene(vec![
        card("a", 0.0, 0.0),
        card("b", 300.0, 0.0),
        card("c", 0.0, 300.0),
    ])
}

fn teinte(cache: &mut SymbioticHueCache, board: &Board, index: &SpatialHash, i: usize) -> f64 {
    cache.get_or_compute(&board.annotations[i], index, board).0
}

/// **La tranche épuisée** : la première teinte de l'image se calcule (la cascade avance), les
/// suivantes gardent la leur, et l'image le dit.
#[test]
fn test_teintes_1_une_image_sans_temps_montre_l_ancienne_et_le_dit() {
    let (board, index) = trois_voisines();
    let mut cache = SymbioticHueCache::new();
    cache.suivre(1, &board.id);
    (0..3).for_each(|i| {
        teinte(&mut cache, &board, &index, i);
    });
    assert_eq!(cache.calculs(), 3);
    assert!(!cache.en_retard());

    cache.accorder_la_tranche(Some(Duration::ZERO));
    cache.suivre(2, &board.id);
    (0..3).for_each(|i| {
        teinte(&mut cache, &board, &index, i);
    });
    assert_eq!(
        cache.calculs(),
        4,
        "une seule teinte refaite dans cette image"
    );
    assert!(cache.en_retard(), "et il en reste : une image doit suivre");
}

/// **Elle converge, puis se rendort** : autant d'images que de teintes à refaire, et la dernière
/// ne redemande rien.
#[test]
fn test_teintes_1_la_cascade_finit_et_rend_les_memes_teintes_que_le_calcul_entier() {
    let (board, index) = trois_voisines();
    let mut entier = SymbioticHueCache::new();
    entier.suivre(1, &board.id);
    let attendues: Vec<f64> = (0..3)
        .map(|i| teinte(&mut entier, &board, &index, i))
        .collect();

    let mut cache = SymbioticHueCache::new();
    cache.accorder_la_tranche(Some(Duration::ZERO));
    let mut images = 0;
    let vues = loop {
        images += 1;
        cache.suivre(1, &board.id);
        let vues: Vec<f64> = (0..3)
            .map(|i| teinte(&mut cache, &board, &index, i))
            .collect();
        if !cache.en_retard() {
            break vues;
        }
        assert!(images < 10, "la cascade ne finit pas");
    };
    assert_eq!(images, 3, "une teinte par image, sans tranche");
    assert_eq!(vues, attendues, "et le même résultat, au bit près");
}

/// **Sans tranche**, rien ne change : tout se calcule tout de suite, pour les épreuves et les bancs.
#[test]
fn test_teintes_1_sans_tranche_tout_se_refait_dans_l_image() {
    let (board, index) = trois_voisines();
    let mut cache = SymbioticHueCache::new();
    cache.suivre(1, &board.id);
    (0..3).for_each(|i| {
        teinte(&mut cache, &board, &index, i);
    });
    cache.suivre(2, &board.id);
    (0..3).for_each(|i| {
        teinte(&mut cache, &board, &index, i);
    });
    assert_eq!(cache.calculs(), 6);
    assert!(!cache.en_retard());
}

/// **Un autre tableau oublie tout** : ses identifiants ne disent rien de celui-ci.
#[test]
fn test_teintes_1_un_autre_tableau_oublie_les_teintes() {
    let (board, index) = trois_voisines();
    let mut cache = SymbioticHueCache::new();
    cache.suivre(1, &board.id);
    teinte(&mut cache, &board, &index, 0);
    cache.suivre(1, "un-autre-tableau");
    cache.suivre(1, &board.id);
    teinte(&mut cache, &board, &index, 0);
    assert_eq!(cache.calculs(), 2);
}

/// **Le vrai lancement pose la tranche, et le réveil la suit** : sans lui, la cascade s'arrêterait
/// à la première image, et les teintes resteraient fausses jusqu'au prochain geste.
#[test]
fn test_teintes_1_le_lancement_pose_la_tranche_et_le_reveil_finit_la_cascade() {
    assert!(include_str!("../../demarrage.rs").contains("hue_cache.accorder_la_tranche("));
    assert!(include_str!("../../app/reveil.rs").contains("hue_cache.en_retard()"));
}

/// **REPOS-2 — le cache paie hors de l'image ce qu'elle a dû**, une dette par paiement sans
/// tranche, puis plus rien ; l'image suivante n'a plus rien à calculer, et montre les teintes du
/// calcul entier, au bit près.
#[test]
fn test_repos_2_payer_hors_de_l_image_rend_les_teintes_du_calcul_entier() {
    let (board, index) = trois_voisines();
    let mut entier = SymbioticHueCache::new();
    entier.suivre(1, &board.id);
    let attendues: Vec<f64> = (0..3)
        .map(|i| teinte(&mut entier, &board, &index, i))
        .collect();

    let mut cache = SymbioticHueCache::new();
    cache.accorder_la_tranche(Some(Duration::ZERO));
    cache.suivre(1, &board.id);
    (0..3).for_each(|i| {
        teinte(&mut cache, &board, &index, i);
    });
    assert_eq!(
        cache.rattraper(&index, &board, 1),
        Some(false),
        "une dette payée, une due"
    );
    assert_eq!(
        cache.rattraper(&index, &board, 1),
        Some(true),
        "tout payé : une image"
    );
    assert_eq!(cache.rattraper(&index, &board, 1), None, "puis rien");
    assert_eq!(cache.calculs(), 3, "chaque teinte calculée une fois");

    cache.suivre(1, &board.id);
    let vues: Vec<f64> = (0..3)
        .map(|i| teinte(&mut cache, &board, &index, i))
        .collect();
    assert_eq!(cache.calculs(), 3, "l'image qui suit ne calcule plus rien");
    assert!(!cache.en_retard());
    assert_eq!(vues, attendues, "et montre le calcul entier, au bit près");
}

/// **REPOS-2 — chaque image refait le compte de ce qu'elle doit** : pendant un geste, aucune
/// dette ne se paie entre les images, et deux images ne doivent pas le double.
#[test]
fn test_repos_2_chaque_image_refait_le_compte_de_ses_dettes() {
    let (board, index) = trois_voisines();
    let mut cache = SymbioticHueCache::new();
    cache.accorder_la_tranche(Some(Duration::ZERO));
    for _ in 0..2 {
        cache.suivre(1, &board.id);
        (0..3).for_each(|i| {
            teinte(&mut cache, &board, &index, i);
        });
    }
    assert_eq!(
        cache.dettes(),
        1,
        "la seconde image ne doit que ce qu'elle a montré d'avant"
    );
}
