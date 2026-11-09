//! **La barre de couleur** : où elle apparaît, ce qui s'y allume, où elle se pose.

use super::*;
use glucose_core::types::{Annotation, BoardImage};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un post-it, un texte, une membrane, une flèche, une photo. Rien de choisi.
fn tableau() -> Store {
    let mut store = Store::new("couleurs");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::sticky("p", 0.0, 0.0, "un post-it"));
    store.add_annotation(&board, Annotation::text("t", 300.0, 0.0, "un texte"));
    store.add_annotation(&board, Annotation::membrane("m", 0.0, 400.0, 300.0, 200.0));
    store.add_annotation(&board, Annotation::arrow("f", 0.0, 900.0, 100.0, 900.0));
    store.add_image(&board, BoardImage::new("i", 600.0, 0.0, 100.0, 100.0));
    store.clear_selection();
    store
}

fn choisir(store: &mut Store, ids: &[&str]) {
    store.clear_selection();
    for id in ids {
        store.select_annotation((*id).to_string(), true);
    }
}

fn barre(store: &Store) -> Option<BarreDeCouleur> {
    layout_barre_de_couleur(store, &Typography::new(), ECRAN, 1.0)
}

/// **Rien de colorable, pas de barre** : ni une flèche (sa couleur est sa relation), ni une
/// photo ; un post-it, un texte, une membrane en ont une.
#[test]
fn test_couleur_1_la_barre_n_existe_que_pour_ce_qui_se_colore() {
    let mut store = tableau();
    assert!(barre(&store).is_none(), "rien de choisi");
    choisir(&mut store, &["f"]);
    assert!(barre(&store).is_none(), "une flèche");
    store.set_selected_image_ids(vec!["i".into()]);
    assert!(barre(&store).is_none(), "une photo");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::sticky("o", 0.0, 700.0, "car"));
    store.poser_l_operateur(
        &board,
        &["o".to_string()],
        Some(glucose_core::types::StickyOperator::Because),
    );
    choisir(&mut store, &["o"]);
    assert!(
        barre(&store).is_none(),
        "un opérateur : sa couleur est son mot"
    );
    for id in ["p", "t", "m"] {
        choisir(&mut store, &[id]);
        let b = barre(&store).expect("une barre");
        assert_eq!(
            b.cases.len(),
            DOMAIN_PALETTE.len() + 1,
            "{id} : huit teintes et par défaut"
        );
    }
}

/// **Une teinte s'allume quand tous la portent** ; « Par défaut » quand aucun n'en a.
#[test]
fn test_couleur_1_la_teinte_commune_s_allume() {
    let mut store = tableau();
    choisir(&mut store, &["p", "m"]);
    let allumees = |store: &Store| -> Vec<Option<u8>> {
        barre(store)
            .expect("une barre")
            .cases
            .iter()
            .filter(|c| c.allume)
            .map(|c| c.teinte)
            .collect()
    };
    assert_eq!(allumees(&store), [None], "aucune couleur : par défaut");
    let board = store.project.active_board_id.clone();
    let ids = ["p".to_string(), "m".to_string()];
    store.colorer(&board, &ids, teinte(2));
    assert_eq!(allumees(&store), [Some(2)], "tous roses");
    store.colorer(&board, &ids[..1], teinte(3));
    assert!(
        allumees(&store).is_empty(),
        "deux teintes : aucune ne s'allume"
    );
}

/// **Elle surmonte la barre d'action — et la barre des flèches quand une flèche est choisie avec
/// un post-it** ; elle tient dans l'écran, ses cases en elle, sans se chevaucher.
#[test]
fn test_couleur_1_elle_se_pose_au_dessus_des_autres_barres() {
    let mut store = tableau();
    choisir(&mut store, &["p"]);
    let b = barre(&store).expect("une barre");
    let action = crate::ui::action_bar::layout_action_bar(
        &store,
        &Typography::new(),
        (ECRAN, 1.0),
        glucose_core::groupe::Origine::Commune,
    )
    .expect("la barre d'action");
    assert!(
        b.rect.1 + b.rect.3 <= action.rect.1,
        "au-dessus de la barre d'action"
    );
    assert!(
        b.rect.0 >= 0.0 && b.rect.0 + b.rect.2 <= ECRAN.0,
        "dans l'écran"
    );
    for paire in b.cases.windows(2) {
        assert!(
            paire[0].rect.0 + paire[0].rect.2 <= paire[1].rect.0,
            "sans chevauchement"
        );
    }
    choisir(&mut store, &["p", "f"]);
    let b = barre(&store).expect("une barre");
    let fleches = crate::ui::options_de_fleche::layout_options_de_fleche(
        &store,
        &Typography::new(),
        ECRAN,
        1.0,
    )
    .expect("la barre des flèches");
    assert!(
        b.rect.1 + b.rect.3 <= fleches.rect.1,
        "au-dessus de la barre des flèches"
    );
}
