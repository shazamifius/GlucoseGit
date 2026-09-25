//! Épreuves de GESTE-1 : pendant un geste, ce que l'écran doit considérer est l'union de ce
//! que l'index connaît et de ce que le geste a touché — dans la numérotation du présent.

use super::*;
use crate::quadtree::{noeud_au_rang, SpatialHash, Visibles};
use crate::store::Store;
use crate::types::{Annotation, CanvasFolder};

/// Un tableau d'une carte et d'un dossier, indexé : l'état publié qu'un geste va changer.
fn tableau_indexe() -> (Store, SpatialHash) {
    let mut store = Store::new("geste");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("carte", 0.0, 0.0, "a"));
    let mut dossier = CanvasFolder::new("dossier", "D", "enfant");
    dossier.x = 300.0;
    store.project.boards[0].folders.push(dossier);
    let mut index = SpatialHash::new(2000.0);
    index.index_board(store.active_board().expect("un tableau"));
    (store, index)
}

/// Les identifiants que des rangs désignent, dans le tableau tel qu'il est.
fn ids(store: &Store, rangs: &[u32]) -> Vec<String> {
    let board = store.active_board().expect("un tableau");
    rangs
        .iter()
        .filter_map(|&r| noeud_au_rang(board, r).map(|n| n.id().to_string()))
        .collect()
}

/// Ce que l'écran considère : la requête de l'index sur tout le monde, complétée par le geste.
fn a_l_ecran(store: &Store, index: &SpatialHash, suivi: &mut SuiviDuGeste) -> Option<Vec<u32>> {
    let mut rangs = index.query_rect_ranks(-1e6, -1e6, 1e6, 1e6, 0.0);
    let board = store.active_board().expect("un tableau");
    suivi
        .completer(store.journal.en_cours(), board, index, &mut rangs)
        .then_some(rangs)
}

/// **Le défaut qu'il a vu** : une flèche tirée pendant un geste n'a aucun rang dans l'index,
/// et ne se dessinait donc qu'au relâchement.
#[test]
fn test_geste_1_une_fleche_tiree_se_voit_pendant_le_glisser() {
    let (mut store, index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    store.begin_live_edit();
    store.add_annotation(&board, Annotation::arrow("fleche", 0.0, 0.0, 50.0, 50.0));

    let rangs = a_l_ecran(&store, &index, &mut suivi).expect("lisible");
    let vus = ids(&store, &rangs);
    assert!(
        vus.contains(&"fleche".to_string()),
        "la flèche se voit : {vus:?}"
    );
    let annotations: Vec<&str> =
        Visibles::nouvelles(&rangs, store.active_board().expect("tableau"))
            .annotations()
            .map(Annotation::id)
            .collect();
    assert_eq!(annotations, ["carte", "fleche"]);
}

/// Poser une annotation décale le rang de chaque dossier : le rang périmé du premier dossier
/// désignait la flèche neuve, et le dossier disparaissait.
#[test]
fn test_geste_1_les_rangs_de_l_index_se_relisent_au_present() {
    let (mut store, index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    store.begin_live_edit();
    store.add_annotation(&board, Annotation::arrow("fleche", 0.0, 0.0, 50.0, 50.0));

    let rangs = a_l_ecran(&store, &index, &mut suivi).expect("lisible");
    let mut vus = ids(&store, &rangs);
    vus.sort();
    assert_eq!(
        vus,
        ["carte", "dossier", "fleche"],
        "chacun une fois, à sa place"
    );
}

/// Sans geste, rien ne change : la requête de l'index est rendue telle quelle.
#[test]
fn test_geste_1_sans_geste_la_requete_est_intacte() {
    let (store, index) = tableau_indexe();
    let mut suivi = SuiviDuGeste::default();
    let brute = index.query_rect_ranks(-1e6, -1e6, 1e6, 1e6, 0.0);
    assert_eq!(a_l_ecran(&store, &index, &mut suivi), Some(brute));
}

/// Un retrait pendant le geste décale ce que l'index connaît d'une façon qu'aucun décalage ne
/// dit : le suivi le signale au lieu de désigner un nœud pour un autre.
#[test]
fn test_geste_1_un_retrait_rend_le_geste_illisible() {
    let (mut store, index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    store.begin_live_edit();
    store.remove_annotations(&board, &["carte"]);
    assert_eq!(a_l_ecran(&store, &index, &mut suivi), None);
}

/// Le suivi ne relit que ce qui est nouveau : au bout de cent mouvements, il a lu cent
/// écritures une fois chacune — pas cinq mille. Relire la pose de la flèche la ferait passer
/// pour une pose au milieu, et le geste deviendrait illisible dès la seconde image.
#[test]
fn test_geste_1_chaque_ecriture_se_lit_une_fois() {
    let (mut store, index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    store.begin_live_edit();
    store.add_annotation(&board, Annotation::arrow("fleche", 0.0, 0.0, 50.0, 50.0));
    for pas in 0..100 {
        store.update_annotation(&board, "fleche", |a| {
            if let Annotation::Arrow { x2, .. } = a {
                *x2 += 1.0;
            }
        });
        a_l_ecran(&store, &index, &mut suivi).expect("lisible à chaque image");
        assert_eq!(suivi.lues, pas + 2);
    }
}

/// Un nouveau geste repart de zéro : ce que le précédent avait touché ne s'y ajoute pas, et sa
/// première écriture n'est pas prise pour déjà lue — même quand l'index, remis d'accord entre
/// les deux, a gardé ses longueurs.
#[test]
fn test_geste_1_un_nouveau_geste_repart_de_zero() {
    let (mut store, mut index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    let ecrire = |store: &mut Store, id: &str, x: f64| {
        store.update_annotation(&board, id, |a| a.translate(x, 0.0));
    };
    store.begin_live_edit();
    for x in [1.0, 2.0, 3.0] {
        ecrire(&mut store, "carte", x);
    }
    a_l_ecran(&store, &index, &mut suivi).expect("lisible");
    store.end_live_edit();
    index.index_board(store.active_board().expect("tableau"));

    store.begin_live_edit();
    store.add_annotation(&board, Annotation::arrow("fleche", 0.0, 0.0, 50.0, 50.0));
    for x in [1.0, 2.0, 3.0] {
        ecrire(&mut store, "fleche", x);
    }
    a_l_ecran(&store, &index, &mut suivi).expect("lisible");
    assert_eq!(
        suivi.touches,
        [vec![], vec![1], vec![]],
        "seule la flèche, posée par ce geste"
    );
}

/// Une pose ailleurs qu'en fin de liste décale ce que l'index connaît : illisible.
#[test]
fn test_geste_1_une_pose_au_milieu_rend_le_geste_illisible() {
    use crate::store::journal::{Edit, Slot};
    let (store, index) = tableau_indexe();
    let board = store.active_board().expect("tableau");
    let pose = |rang| Edit::Annotation {
        board: board.id.clone(),
        slot: Slot::inserted(rang, Annotation::text("x", 0.0, 0.0, "x")),
    };
    let mut rangs = Vec::new();
    let mut suivi = SuiviDuGeste::default();
    assert!(suivi.completer(Some((1, &[pose(1)])), board, &index, &mut rangs));
    let mut suivi = SuiviDuGeste::default();
    assert!(!suivi.completer(Some((1, &[pose(0)])), board, &index, &mut rangs));
    let mut suivi = SuiviDuGeste::default();
    assert!(
        !suivi.completer(Some((1, &[pose(1), pose(1)])), board, &index, &mut rangs),
        "la seconde passe devant la première"
    );
}

/// Une fois l'index remis d'accord en plein geste, le retrait qu'il connaît désormais ne rend
/// plus le geste illisible : seule la suite se lit.
#[test]
fn test_geste_1_l_index_remis_d_accord_absorbe_le_geste() {
    let (mut store, mut index) = tableau_indexe();
    let board = store.project.active_board_id.clone();
    let mut suivi = SuiviDuGeste::default();
    store.begin_live_edit();
    store.remove_annotations(&board, &["carte"]);
    assert_eq!(a_l_ecran(&store, &index, &mut suivi), None);
    index.index_board(store.active_board().expect("tableau"));
    suivi.absorbe(store.journal.en_cours(), &index);

    store.add_annotation(&board, Annotation::arrow("fleche", 0.0, 0.0, 50.0, 50.0));
    let rangs = a_l_ecran(&store, &index, &mut suivi).expect("lisible");
    let mut vus = ids(&store, &rangs);
    vus.sort();
    assert_eq!(vus, ["dossier", "fleche"]);
}
