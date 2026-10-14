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

/// **FANTOME-1 — la grille part du coin du clic** (fiche 66) : la première case a son coin
/// haut gauche sous le clic, les suivantes case après case dans l'ordre de lecture, au pas de
/// la largeur et de la hauteur — description comprise — plus l'écart. À l'envers : la loi de
/// Tauri, centrée, aurait mis la première case une demi-grille plus haut et plus à gauche.
#[test]
fn test_fantome_1_la_grille_part_du_coin_du_clic() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, grille(), (1000.0, 500.0));
    let coins: Vec<(f64, f64)> = store.cases(&board).iter().map(|c| (c.x, c.y)).collect();
    assert_eq!(coins.len(), 8);
    assert_eq!(coins[0], (1000.0, 500.0), "la première sous le clic");
    let h = 320.0 * 9.0 / 16.0;
    let (pas_x, pas_y) = (320.0 + 24.0, h + DESCRIPTION + 24.0);
    assert_eq!(coins[1].0 - coins[0].0, pas_x);
    assert_eq!(coins[4], (1000.0, 500.0 + pas_y), "la deuxième rangée");
    let (largeur, hauteur) = grille().etendue(8);
    assert_ne!(
        coins[0],
        (1000.0 - largeur / 2.0, 500.0 - hauteur / 2.0),
        "plus centrée"
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

// ── STORYBOARD-2 : le rectangle tracé, la suite rangée (fiche 65) ───────────────────────────

/// Une grille dont le pas tombe juste : des cases de 100 × 100 (le carré), un écart de 20 ;
/// d'une case à sa voisine, 120 en largeur et 160 en hauteur (la bande de 40 comprise).
fn carree() -> Grille {
    Grille {
        format: Format::Carre,
        largeur: 100.0,
        colonnes: 4,
        ecart: 20.0,
    }
}

/// **Un rectangle donne les cases dont l'étendue est la plus proche de la sienne**, ancrées au
/// coin d'où la main est partie : 350 de large donnent trois cases (3 × 100 + 2 × 20 = 340),
/// 420 en donnent quatre (460, plus près que 340) ; tracé vers le haut et la gauche, la
/// grille finit au point de départ.
#[test]
fn test_un_rectangle_donne_les_cases_les_plus_proches_ancrees_au_depart() {
    let (store, board) = tableau();
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (350.0, 330.0)));
    assert_eq!(
        (d.grille.colonnes, d.nombre),
        (3, 6),
        "trois colonnes, deux rangées"
    );
    assert_eq!(d.origine, (0.0, 0.0));
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (420.0, 170.0)));
    assert_eq!(d.grille.colonnes, 4, "460 est plus près de 420 que 340");

    let vers_le_haut = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (-350.0, -330.0)));
    let (l, h) = vers_le_haut.grille.etendue(vers_le_haut.nombre);
    assert_eq!(
        vers_le_haut.origine,
        (-l, -h),
        "la grille finit au point de départ"
    );

    let minuscule = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (5.0, 5.0)));
    assert_eq!(minuscule.nombre, 1, "jamais moins d'une case");
}

/// **Tracer pose le storyboard, et sa grille s'enregistre** : les colonnes du rectangle, pas
/// celles du panneau — « + Ajouter » les suit ensuite. Un `Ctrl+Z` retire cases et grille.
#[test]
fn test_tracer_enregistre_la_grille_que_les_gestes_suivants_suivent() {
    let (mut store, board) = tableau();
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (600.0, 170.0)));
    store.disposer(&board, d);
    assert_eq!(store.cases(&board).len(), 5, "cinq colonnes, une rangée");
    assert_eq!(
        store.grille_du_storyboard(&board).map(|g| g.colonnes),
        Some(5)
    );
    let id = store.ajouter_une_case(&board, carree()).expect("une case");
    let neuve = store
        .cases(&board)
        .into_iter()
        .find(|c| c.id == id)
        .cloned()
        .expect("elle");
    assert_eq!(
        (neuve.x, neuve.y),
        (0.0, 160.0),
        "la sixième ouvre la deuxième rangée de cinq — le panneau en disait quatre"
    );
    store.undo();
    store.undo();
    assert!(store.cases(&board).is_empty());
    assert_eq!(
        store.grille_du_storyboard(&board),
        None,
        "la grille part avec ses cases"
    );
}

/// **Tracer plus petit retire les cases vides de la fin, jamais une case qui porte quelque
/// chose** : la grille s'allonge plutôt. Tracer plus grand en ajoute.
#[test]
fn test_tracer_plus_petit_garde_ce_qui_porte_quelque_chose() {
    let (mut store, board) = tableau();
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (470.0, 330.0)));
    store.disposer(&board, d);
    assert_eq!(store.cases(&board).len(), 8);
    let cinquieme = store.cases(&board)[4].id.clone();
    store.update_panel(&board, &cinquieme, "le héros entre");
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (230.0, 170.0)));
    assert_eq!(d.grille.colonnes, 2);
    assert_eq!(
        d.nombre, 5,
        "deux cases tiendraient ; la cinquième est décrite"
    );
    store.disposer(&board, d);
    let cases = store.cases(&board);
    assert_eq!(cases.len(), 5);
    assert_eq!(cases[4].description, "le héros entre");
    assert_eq!(
        (cases[4].x, cases[4].y),
        (0.0, 320.0),
        "troisième rangée, à gauche"
    );
    let plus_grand = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (470.0, 490.0)));
    store.disposer(&board, plus_grand);
    assert_eq!(store.cases(&board).len(), 12);
}

/// **Une case qui ne porte qu'une image se garde aussi** : l'image compte comme une
/// description.
#[test]
fn test_une_case_qui_porte_une_image_se_garde() {
    let (mut store, board) = tableau();
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (470.0, 330.0)));
    store.disposer(&board, d);
    let septieme = store.cases(&board)[6].clone();
    store.add_image(&board, image("i", (septieme.x + 50.0, septieme.y + 50.0)));
    assert_eq!(store.cases_a_garder(&board), 7);
    let d = store.disposition_du_trace(&board, carree(), ((0.0, 0.0), (110.0, 110.0)));
    assert_eq!(d.nombre, 7);
}

/// **Une image suit sa case** : à taille égale, d'une translation exacte ; à taille changée,
/// la composition de la case est gardée — contenue, centrée.
#[test]
fn test_une_image_suit_sa_case_par_une_similitude() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let case = store.cases(&board)[0].clone();
    store.add_image(&board, image("i", (case.x + 50.0, case.y + 50.0)));
    store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    let avant = store.image(&board, "i").expect("l'image").clone();
    store.poser_le_storyboard(&board, carree(), (1000.0, 0.0));
    let deplacee = store.cases(&board)[0].clone();
    let apres = store.image(&board, "i").expect("l'image").clone();
    assert_eq!(
        (apres.x - avant.x, apres.y - avant.y, apres.width),
        (deplacee.x - case.x, deplacee.y - case.y, avant.width),
        "une translation"
    );
    let double = Grille {
        largeur: 200.0,
        ..carree()
    };
    store.poser_le_storyboard(&board, double, (1000.0, 0.0));
    let grande = store.cases(&board)[0].clone();
    let refaite = store.image(&board, "i").expect("l'image");
    assert_eq!(refaite.width, 200.0, "deux fois plus grande, comme sa case");
    assert_eq!(
        (refaite.x, refaite.y),
        (grande.x + 100.0, grande.y + 100.0),
        "toujours centrée"
    );
}

/// **Retirer une case fait avancer les suivantes d'une place, leurs images avec elles** ; la
/// dernière retirée emporte le storyboard.
#[test]
fn test_retirer_une_case_fait_avancer_les_suivantes() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    store.add_image(&board, image("i", (cases[2].x + 50.0, cases[2].y + 50.0)));
    store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    store.retirer_la_case(&board, &cases[1].id);
    let apres: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    assert_eq!(
        apres[1].id, cases[2].id,
        "la troisième prend la deuxième place"
    );
    assert_eq!((apres[1].x, apres[1].y), (cases[1].x, cases[1].y));
    let image = store.image(&board, "i").expect("l'image");
    assert!(
        apres[1].contient((image.x, image.y)),
        "son image l'a suivie"
    );
    for c in apres {
        store.retirer_la_case(&board, &c.id);
    }
    assert_eq!(store.grille_du_storyboard(&board), None);
}

/// **Déplacer une case à une autre place** fait glisser celles d'entre les deux, renumérote
/// tout dans l'ordre qu'on voit, et chaque image suit sa case — un geste.
#[test]
fn test_deplacer_une_case_reordonne_la_suite() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    store.add_image(&board, image("i", (cases[0].x + 50.0, cases[0].y + 50.0)));
    store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    store.journal.prendre_les_ecrits();
    let portees = store.images_de_la_case(&board, &cases[0].id);
    store.deplacer_la_case(
        &board,
        (&cases[0].id, &portees),
        5,
        (cases[0].x, cases[0].y),
    );
    let apres = store.cases(&board);
    let ordre: Vec<&str> = apres.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ordre[5], cases[0].id, "la première est maintenant sixième");
    assert_eq!(ordre[0], cases[1].id, "la deuxième ouvre la suite");
    assert!(apres.iter().enumerate().all(|(i, c)| c.order == i as i32));
    assert_eq!(
        (apres[5].x, apres[5].y),
        (cases[5].x, cases[5].y),
        "à la sixième place"
    );
    let image = store.image(&board, "i").expect("l'image");
    assert!(
        apres[5].contient((image.x, image.y)),
        "son image l'a suivie"
    );
    assert_eq!(store.journal.prendre_les_ecrits().len(), 1, "un geste");
}

/// **La place sous un point** : sa colonne et sa rangée, bornées à la suite.
#[test]
fn test_la_place_sous_un_point() {
    let d = Disposition {
        grille: carree(),
        origine: (0.0, 0.0),
        nombre: 6,
    };
    assert_eq!(d.rang_sous((130.0, 10.0)), 1);
    assert_eq!(d.rang_sous((10.0, 170.0)), 4, "deuxième rangée");
    assert_eq!(
        d.rang_sous((9000.0, 10.0)),
        3,
        "au-delà à droite : la dernière colonne"
    );
    assert_eq!(
        d.rang_sous((370.0, 900.0)),
        5,
        "au-delà de la suite : la dernière case"
    );
    assert_eq!(d.rang_sous((-50.0, -50.0)), 0);
}

/// **Chaque rangement numérote de un en un, dans l'ordre de la suite** — même quand les
/// numéros d'avant étaient en désordre.
#[test]
fn test_ranger_numerote_dans_l_ordre_de_la_suite() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let b = store
        .project
        .boards
        .iter_mut()
        .find(|b| b.id == board)
        .expect("le tableau");
    for (i, c) in b.panels.iter_mut().enumerate() {
        c.order = 100 - 7 * i as i32;
    }
    let attendu: Vec<String> = store.cases(&board).iter().map(|c| c.id.clone()).collect();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let cases = store.cases(&board);
    assert!(cases.iter().enumerate().all(|(i, c)| c.order == i as i32));
    let ids: Vec<String> = cases.iter().map(|c| c.id.clone()).collect();
    assert_eq!(ids, attendu, "la suite d'avant, renumérotée");
}

/// **Glisser une case puis la lâcher à une autre place, dans un seul geste** : défaire rend
/// chaque case **exactement** à son départ, et refaire exactement à son arrivée — les pas du
/// glisser se sont refermés en leur translation avant le rangement.
///
/// **L'image reste dans sa case, à l'aller comme au retour** (fiche 66). Cette épreuve
/// promettait l'image exacte, et ne le tenait que par la chance des coordonnées : la grille
/// posée par son coin en (0,3 ; 0,7) l'a montrée revenir deux ulps à côté. GLISSER-1 écrit un
/// **pas**, pas une valeur ; défaire soustrait ce que faire a ajouté, et `(x + d) − d` n'est
/// pas toujours `x`. Ce que GLISSER-1 garantit — `Échap` rend les départs exacts, le fichier
/// rejoue l'écran au bit près — tient ; une annulation exacte demanderait au journal les
/// départs eux-mêmes (GLISSER-2, SUITE). Aucune tolérance n'est choisie ici : l'épreuve dit
/// ce qui est exact (les cases) et ce que l'œil voit (l'image dans sa case).
#[test]
fn test_glisser_puis_lacher_une_case_se_defait_exactement() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.3, 0.7));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    store.add_image(&board, image("i", (cases[0].x + 50.0, cases[0].y + 50.0)));
    store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    let portees = store.images_de_la_case(&board, &cases[0].id);
    store.begin_live_edit();
    for pas in 1..=40 {
        let total = (f64::from(pas) * 7.1, f64::from(pas) * 0.3);
        let tenue = (cases[0].id.as_str(), (cases[0].x, cases[0].y));
        store.glisser_la_case(&board, tenue, &portees, total);
    }
    store.deplacer_la_case(
        &board,
        (&cases[0].id, &portees),
        2,
        (cases[0].x, cases[0].y),
    );
    store.end_live_edit();
    let lachee = store.cases(&board)[2].clone();
    assert_eq!(lachee.id, cases[0].id);
    // Lâchée, elle recouvrait la troisième case : son image reste la sienne quand même.
    let image = store.image(&board, "i").expect("l'image");
    assert!(
        lachee.contient((image.x, image.y)),
        "l'image suit la case tenue"
    );
    let les_cases = |store: &Store| -> Vec<StoryboardPanel> {
        store.cases(&board).into_iter().cloned().collect()
    };
    let ou = |store: &Store| {
        let i = store.image(&board, "i").expect("l'image");
        (i.x, i.y)
    };
    let cases_apres = les_cases(&store);
    store.undo();
    assert_eq!(les_cases(&store), cases, "défaire rend chaque case exacte");
    assert!(cases[0].contient(ou(&store)), "et l'image dans sa case");
    store.redo();
    assert_eq!(
        les_cases(&store),
        cases_apres,
        "refaire rend chaque case exacte"
    );
    assert!(
        lachee.contient(ou(&store)),
        "et l'image dans la case lâchée"
    );
}

/// **Une étendue exacte redonne exactement ses cases**, à tous les formats — c'est ce que
/// saisir la poignée au coin demande. La partie entière s'y trompait : sept rangées au format
/// Scope s'étendent sur `7·p − e`, et `(7·p − e + e) / p` vaut 6,999999999999999.
#[test]
fn test_une_etendue_exacte_redonne_ses_cases() {
    for format in Format::TOUS {
        for (largeur, ecart) in [(280.0, 24.0), (277.0, 7.0), (300.0, 16.0), (250.0, 0.0)] {
            let g = Grille {
                format,
                largeur,
                colonnes: 1,
                ecart,
            };
            for n in 1..=60 {
                let (pas_x, pas_y) = g.pas();
                let etendue = (n as f64 * pas_x - ecart, n as f64 * pas_y - ecart);
                assert_eq!(
                    g.tient_dans(etendue),
                    (n, n),
                    "{format:?} {largeur} {ecart} {n}"
                );
            }
        }
    }
    // À l'envers : la partie entière perdait une rangée.
    let scope = Grille {
        format: Format::Scope,
        largeur: 280.0,
        colonnes: 1,
        ecart: 24.0,
    };
    let pas = scope.pas().1;
    assert_eq!(((7.0 * pas - 24.0 + 24.0) / pas).floor(), 6.0);
}

// ── P2, proposition 1 : plusieurs images lâchées remplissent les cases vides (fiche 66) ─────

/// Lâche ces images, chacune à son centre, en un geste — comme le dépôt d'un lot.
fn lacher(store: &mut Store, board: &str, centres: &[(&str, (f64, f64))]) {
    store.begin_live_edit();
    for (id, centre) in centres {
        store.add_image(board, image(id, *centre));
    }
    let ids: Vec<String> = centres.iter().map(|(id, _)| id.to_string()).collect();
    store.ajuster_dans_les_cases(board, &ids);
    store.end_live_edit();
}

/// Le rang, dans la suite, de la case où cette image est posée.
fn rang_de(store: &Store, board: &str, id: &str) -> Option<usize> {
    let i = store.image(board, id)?;
    store
        .cases(board)
        .iter()
        .position(|c| c.contient((i.x, i.y)))
}

/// Le milieu du cadre de la case de ce rang, et trois points en cascade à partir de lui — ce
/// que pose un dépôt de trois fichiers.
fn cascade(store: &Store, board: &str, rang: usize) -> [(f64, f64); 3] {
    let c = store.cases(board)[rang].clone();
    let m = (c.x + c.width / 2.0, c.y + c.height / 2.0);
    [m, (m.0 + 24.0, m.1 + 24.0), (m.0 + 48.0, m.1 + 48.0)]
}

/// **Trois images lâchées sur une case vide en remplissent trois à la suite**, dans l'ordre de
/// lecture — et un seul `Ctrl+Z` défait tout le lâcher. À l'envers, la loi d'avant les aurait
/// empilées toutes les trois dans la première case.
#[test]
fn test_p2_1_un_lot_lache_sur_une_case_remplit_la_suite() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    store.journal.clear();
    let [a, b, c] = cascade(&store, &board, 0);
    lacher(&mut store, &board, &[("a", a), ("b", b), ("c", c)]);
    let rangs: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|id| rang_de(&store, &board, id))
        .collect();
    assert_eq!(rangs, vec![Some(0), Some(1), Some(2)]);
    assert_eq!(store.cases(&board).len(), 8, "aucune case ajoutée");
    store.undo();
    assert!(store.image(&board, "a").is_none(), "un seul geste");
    assert!(!store.can_undo());
}

/// **Une case qui porte déjà une image est sautée** — elle n'est pas vide.
#[test]
fn test_p2_1_une_case_occupee_est_sautee() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let [deja, ..] = cascade(&store, &board, 1);
    lacher(&mut store, &board, &[("deja", deja)]);
    let [a, b, c] = cascade(&store, &board, 0);
    lacher(&mut store, &board, &[("a", a), ("b", b), ("c", c)]);
    let rangs: Vec<_> = ["deja", "a", "b", "c"]
        .iter()
        .map(|id| rang_de(&store, &board, id))
        .collect();
    assert_eq!(rangs, vec![Some(1), Some(0), Some(2), Some(3)]);
}

/// **Quand il ne reste plus de case vide, le storyboard s'allonge**, dans le même geste.
#[test]
fn test_p2_1_le_storyboard_s_allonge_s_il_le_faut() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    store.journal.clear();
    let [a, b, c] = cascade(&store, &board, 6);
    lacher(&mut store, &board, &[("a", a), ("b", b), ("c", c)]);
    assert_eq!(store.cases(&board).len(), 9, "une case de plus");
    let rangs: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|id| rang_de(&store, &board, id))
        .collect();
    assert_eq!(rangs, vec![Some(6), Some(7), Some(8)]);
    store.undo();
    assert_eq!(
        store.cases(&board).len(),
        8,
        "défaire retire aussi la case ajoutée"
    );
    assert!(store.image(&board, "c").is_none());
}

/// **Une image lâchée seule fait ce qu'elle faisait** : dans la case sous elle, même occupée.
#[test]
fn test_p2_1_une_image_seule_va_dans_la_case_sous_elle() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let [deja, ..] = cascade(&store, &board, 1);
    lacher(&mut store, &board, &[("deja", deja)]);
    let [seule, ..] = cascade(&store, &board, 1);
    lacher(&mut store, &board, &[("seule", seule)]);
    assert_eq!(rang_de(&store, &board, "seule"), Some(1));
}

/// **L'ordre de lecture décide, pas l'ordre de la liste** : les mêmes trois images, données à
/// l'envers, remplissent les cases dans l'ordre où on les lit.
#[test]
fn test_p2_1_l_ordre_de_lecture_decide() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let [a, b, c] = cascade(&store, &board, 0);
    lacher(&mut store, &board, &[("c", c), ("b", b), ("a", a)]);
    let rangs: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|id| rang_de(&store, &board, id))
        .collect();
    assert_eq!(rangs, vec![Some(0), Some(1), Some(2)]);
}

/// **Une image du lâcher qui déborde déjà sur la case suivante ne la bloque pas** : elle s'en
/// va à son tour, donc la case est vide pour celle qui la précède dans la lecture.
#[test]
fn test_p2_1_le_lacher_ne_se_bloque_pas_lui_meme() {
    let (mut store, board) = tableau();
    store.poser_le_storyboard(&board, carree(), (0.0, 0.0));
    let cases: Vec<StoryboardPanel> = store.cases(&board).into_iter().cloned().collect();
    let milieu = |c: &StoryboardPanel| (c.x + c.width / 2.0, c.y + c.height / 2.0);
    let (m0, m1) = (milieu(&cases[0]), milieu(&cases[1]));
    // a et b au-dessus de la première case, c déjà au-dessus de la deuxième.
    lacher(
        &mut store,
        &board,
        &[("a", m0), ("b", (m0.0 + 40.0, m0.1)), ("c", m1)],
    );
    let rangs: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|id| rang_de(&store, &board, id))
        .collect();
    assert_eq!(rangs, vec![Some(0), Some(1), Some(2)]);
}
