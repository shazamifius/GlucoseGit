//! Les épreuves du storyboard (STORYBOARD-1) : la grille, l'ordre de lecture, un geste par
//! changement, et l'image qui s'ajuste dans sa case.

use super::*;
use crate::types::BoardImage;

fn grille() -> Grille {
    Grille {
        format: Format::Cinema,
        largeur: 320.0,
        colonnes: 4,
        ecart: 24.0,
    }
}

fn tableau() -> (Store, String) {
    let store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    (store, board)
}

/// **La grille se centre sur le clic**, case après case dans l'ordre de lecture, au pas de la
/// largeur et de la hauteur — description comprise — plus l'écart.
#[test]
fn test_la_grille_se_centre_sur_le_clic() {
    let g = grille();
    let coins = g.coins(8, (1000.0, 500.0));
    assert_eq!(coins.len(), 8);
    let h = 320.0 * 9.0 / 16.0;
    let (pas_x, pas_y) = (320.0 + 24.0, h + DESCRIPTION + 24.0);
    assert_eq!(coins[1].0 - coins[0].0, pas_x);
    assert_eq!(
        coins[4],
        (coins[0].0, coins[0].1 + pas_y),
        "la deuxième rangée"
    );
    let (gauche, droite) = (coins[0].0, coins[3].0 + 320.0);
    let (haut, bas) = (coins[0].1, coins[7].1 + h + DESCRIPTION);
    assert_eq!(
        ((gauche + droite) / 2.0, (haut + bas) / 2.0),
        (1000.0, 500.0)
    );
}

/// **Poser le storyboard : deux rangées sur un tableau qui n'en a pas, en un geste** ; poser
/// encore range les mêmes cases à la nouvelle grille, sans en créer.
#[test]
fn test_poser_puis_mettre_a_jour() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let premieres: Vec<String> = store.cases(&board).iter().map(|c| c.id.clone()).collect();
    assert_eq!(premieres.len(), 8, "deux rangées de quatre");
    assert!(store.can_undo());
    store.undo();
    assert!(
        store.cases(&board).is_empty(),
        "un seul Ctrl+Z retire toute la grille"
    );
    store.redo();

    let large = Grille {
        largeur: 500.0,
        colonnes: 2,
        ..grille()
    };
    store.poser_le_storyboard(&board, large, (100.0, 100.0));
    let cases = store.cases(&board);
    let ids: Vec<String> = cases.iter().map(|c| c.id.clone()).collect();
    assert_eq!(
        ids, premieres,
        "les mêmes cases, rangées, dans l'ordre de leurs numéros"
    );
    assert!(cases.iter().all(|c| c.width == 500.0));
    assert_eq!(
        cases[2].x, cases[0].x,
        "deux colonnes : la troisième revient à gauche"
    );
}

/// **Retirer une case renumérote les suivantes** : la suite reste continue.
#[test]
fn test_retirer_une_case_garde_la_suite_continue() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let deuxieme = store.cases(&board)[1].id.clone();
    store.retirer_la_case(&board, &deuxieme);
    let ordres: Vec<i32> = store.cases(&board).iter().map(|c| c.order).collect();
    assert_eq!(ordres, (0..7).collect::<Vec<_>>());
    store.undo();
    assert_eq!(store.cases(&board).len(), 8, "un geste");
}

/// **Ajouter une case la pose à la suite, sur la grille de la première.**
#[test]
fn test_ajouter_une_case_a_la_suite() {
    let (mut store, board) = tableau();
    assert!(
        store.ajouter_une_case(&board, grille()).is_none(),
        "sans storyboard, rien"
    );
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let id = store.ajouter_une_case(&board, grille()).expect("une case");
    let cases = store.cases(&board);
    let neuve = cases.last().expect("la dernière");
    assert_eq!((neuve.id.as_str(), neuve.order), (id.as_str(), 8));
    assert_eq!(neuve.x, cases[0].x, "la neuvième ouvre la troisième rangée");
    assert!(neuve.y > cases[4].y);
}

/// **Renuméroter suit l'ordre de lecture** : une case glissée en tête de la première rangée
/// devient la première ; une case un peu plus basse que ses voisines reste dans leur rangée.
#[test]
fn test_renumeroter_suit_l_ordre_de_lecture() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    // La sixième (deuxième rangée) passe à gauche de la première, un peu plus bas.
    let sixieme = cases[5].id.clone();
    store.placer_la_case(&board, &sixieme, (cases[0].x - 400.0, cases[0].y + 30.0));
    // La deuxième descend d'un rien : elle reste dans la première rangée.
    let deuxieme = cases[1].id.clone();
    store.placer_la_case(&board, &deuxieme, (cases[1].x, cases[1].y + 20.0));
    store.renumeroter_les_cases(&board);
    let ordre: Vec<String> = store.cases(&board).iter().map(|c| c.id.clone()).collect();
    assert_eq!(ordre[0], sixieme, "la plus à gauche de la première rangée");
    assert_eq!(ordre[2], deuxieme, "toujours dans la première rangée");
    // L'ancienne loi — l'ordre de création — aurait laissé la sixième sixième.
    let par_creation: Vec<String> = cases.iter().map(|c| c.id.clone()).collect();
    assert_eq!(par_creation[5], sixieme);
}

/// **Un glisser ne consigne que la case qui bouge.**
#[test]
fn test_placer_une_case_ne_consigne_qu_elle() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    let edits = diff_des_cases(&board, &cases, &{
        let mut c = cases.clone();
        c[3].x += 10.0;
        c
    });
    assert_eq!(edits.len(), 1);
}

/// Une image de 400 × 400 posée là.
fn image(id: &str, (x, y): (f64, f64)) -> BoardImage {
    let mut img = BoardImage::new(id, x, y, 400.0, 400.0);
    img.original_width = 400.0;
    img.original_height = 400.0;
    img
}

/// **Une image posée dans une case s'y ajuste** : contenue dans le cadre, centrée, à ses
/// proportions ; une image hors des cases, ou verrouillée, ne bouge pas.
#[test]
fn test_une_image_posee_dans_une_case_s_y_ajuste() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let case = store.cases(&board)[2].clone();
    let dedans = (case.x + 10.0, case.y + 10.0);
    store.add_image(&board, image("dedans", dedans));
    store.add_image(&board, image("dehors", (5000.0, 5000.0)));
    let mut verrou = image("verrou", dedans);
    verrou.locked = true;
    store.add_image(&board, verrou);
    let ids = ["dedans", "dehors", "verrou"].map(String::from);
    store.ajuster_dans_les_cases(&board, &ids);
    let ajustee = store.image(&board, "dedans").expect("l'image");
    assert_eq!(
        (ajustee.x, ajustee.y),
        (case.x + case.width / 2.0, case.y + case.height / 2.0)
    );
    assert_eq!(
        ajustee.height, case.height,
        "un carré dans un 16:9 : contenu par la hauteur"
    );
    assert!(ajustee.width <= case.width);
    assert_eq!(store.image(&board, "dehors").map(|i| i.width), Some(400.0));
    assert_eq!(store.image(&board, "verrou").map(|i| i.width), Some(400.0));
}

/// **Effacer le storyboard retire toutes les cases en un geste**, et laisse les images.
#[test]
fn test_effacer_le_storyboard() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    store.add_image(&board, image("i", (0.0, 0.0)));
    store.effacer_le_storyboard(&board);
    assert!(store.cases(&board).is_empty());
    assert!(store.image(&board, "i").is_some());
    store.undo();
    assert_eq!(store.cases(&board).len(), 8);
}

/// **Glisser une case emporte ses images, en deux éditions** — la case et une translation —
/// quelle que soit la durée du geste ; un seul `Ctrl+Z` rend tout à sa place.
#[test]
fn test_glisser_une_case_emporte_ses_images_en_deux_editions() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (0.0, 0.0));
    let case = store.cases(&board)[0].clone();
    store.add_image(&board, image("i", (case.x + 50.0, case.y + 50.0)));
    store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    let depart = store.image(&board, "i").expect("l'image").clone();
    let images = store.images_de_la_case(&board, &case.id);
    assert_eq!(images, ["i"]);
    store.journal.prendre_les_ecrits();
    store.begin_live_edit();
    for pas in 1..=50 {
        let total = (f64::from(pas) * 3.0, f64::from(pas) * 2.0);
        store.glisser_la_case(&board, (&case.id, (case.x, case.y)), &images, total);
    }
    store.end_live_edit();
    let deplacee = store.cases(&board)[0].clone();
    assert_eq!((deplacee.x, deplacee.y), (case.x + 150.0, case.y + 100.0));
    let suivie = store.image(&board, "i").expect("l'image");
    assert_eq!(
        (suivie.x, suivie.y),
        (depart.x + 150.0, depart.y + 100.0),
        "l'image suit sa case"
    );
    let ecrits = store.journal.prendre_les_ecrits();
    assert_eq!(ecrits.len(), 1, "un geste");
    assert_eq!(
        ecrits[0].edits.len(),
        2,
        "la case et une translation, pas cent éditions"
    );
    store.undo();
    assert_eq!(store.cases(&board)[0].x, case.x);
    assert_eq!(store.image(&board, "i").map(|i| i.x), Some(depart.x));
}
