//! SEQUENCE-1 (fiche 68) : toute case nomme un storyboard de son tableau, et tout storyboard a
//! au moins une case — ramené par une seule règle à la lecture, tenu par chaque geste.

use super::*;
use crate::store::Store;

fn case(id: &str, sb: &str) -> StoryboardPanel {
    StoryboardPanel::new(id, sb, 0, (0.0, 0.0), (10.0, 10.0))
}

/// **Une case orpheline va au storyboard d'avant, né s'il le faut ; un storyboard vide part.**
/// Une case qui nomme un storyboard présent ne bouge pas.
#[test]
fn test_sequence_1_rattacher() {
    let gardee = Storyboard::neuf("seq", Grille::default());
    let vide = Storyboard::neuf("vide", Grille::default());
    let mut storyboards = vec![gardee.clone(), vide];
    let mut cases = [case("a", "seq"), case("b", "absent"), case("c", "")];
    rattacher(&mut cases, &mut storyboards);
    let noms: Vec<&str> = cases.iter().map(|c| c.storyboard.as_str()).collect();
    assert_eq!(noms, ["seq", STORYBOARD_D_AVANT, STORYBOARD_D_AVANT]);
    assert_eq!(
        storyboards,
        [
            gardee,
            Storyboard::neuf(STORYBOARD_D_AVANT, Grille::default())
        ],
        "le vide est parti, celui d'avant est né à la grille de Tauri"
    );
    // Une seconde fois, rien ne change : la règle est stable.
    let (avant_cases, avant_sb) = (cases.clone(), storyboards.clone());
    rattacher(&mut cases, &mut storyboards);
    assert_eq!(
        (cases.to_vec(), storyboards),
        (avant_cases.to_vec(), avant_sb)
    );
}

/// SEQUENCE-1 sur un tableau du magasin.
fn tient(store: &Store, board: &str) -> bool {
    let b = store
        .project
        .boards
        .iter()
        .find(|b| b.id == board)
        .expect("le tableau");
    b.panels
        .iter()
        .all(|c| b.storyboards.iter().any(|s| s.id == c.storyboard))
        && b.storyboards
            .iter()
            .all(|s| b.panels.iter().any(|c| c.storyboard == s.id))
}

/// **Chaque geste tient SEQUENCE-1**, à l'aller et au retour : poser deux storyboards, retirer
/// toutes les cases de l'un — il part avec la dernière —, en ajouter, effacer l'autre ; puis
/// tout défaire, et tout refaire.
#[test]
fn test_sequence_1_chaque_geste_la_tient() {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let un = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 0.0))
        .expect("posé");
    let deux = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 900.0))
        .expect("posé");
    assert!(tient(&store, &board));
    let mut gestes = 2;
    for c in store
        .cases(&board, &un)
        .iter()
        .map(|c| c.id.clone())
        .collect::<Vec<_>>()
    {
        store.retirer_la_case(&board, &c);
        gestes += 1;
        assert!(tient(&store, &board));
    }
    assert!(
        store.storyboard(&board, &un).is_none(),
        "parti avec sa dernière case"
    );
    store.ajouter_une_case(&board, &deux);
    store.effacer_le_storyboard(&board, &deux);
    gestes += 2;
    assert!(tient(&store, &board) && store.storyboards(&board).is_empty());
    for _ in 0..gestes {
        assert!(store.undo());
        assert!(tient(&store, &board), "en défaisant");
    }
    assert!(store.storyboards(&board).is_empty(), "tout défait");
    for _ in 0..gestes {
        assert!(store.redo());
        assert!(tient(&store, &board), "en refaisant");
    }
}

/// **Les champs sont ceux de chaque storyboard** : retirer un champ de l'un retire son texte
/// de ses cases, et des siennes seulement — l'autre garde les siens.
#[test]
fn test_retirer_un_champ_ne_touche_pas_l_autre_storyboard() {
    use crate::store::storyboard::Texte;
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let un = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 0.0))
        .expect("posé");
    let deux = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 900.0))
        .expect("posé");
    let c1 = store.cases(&board, &un)[0].id.clone();
    let c2 = store.cases(&board, &deux)[0].id.clone();
    for c in [&c1, &c2] {
        store.ecrire_dans_la_case(&board, c, Texte::Champ(0), "réplique");
        store.ecrire_dans_la_case(&board, c, Texte::Champ(1), "action");
    }
    store.retirer_le_champ((&board, &un), 0);
    let texte = |store: &Store, sb: &str| store.cases(&board, sb)[0].textes.clone();
    assert_eq!(texte(&store, &un), ["action"], "le sien glisse d'un rang");
    assert_eq!(
        texte(&store, &deux),
        ["réplique", "action"],
        "l'autre garde les siens"
    );
    assert_eq!(store.storyboard(&board, &deux).expect("là").champs.len(), 3);
}

/// **Un storyboard neuf ne reprend jamais le nom d'un storyboard déjà là** (ID-1) : un
/// document relu porte les noms que la fabrique donnerait ensuite — `storyboard-1` à
/// `storyboard-200`, chacun avec une case qui n'a pas de numéro —, et le neuf naît au-delà.
/// Sans les storyboards dans le compte, il reprendrait l'un d'eux et le rangerait à sa place.
#[test]
fn test_un_storyboard_neuf_ne_reprend_pas_un_nom_deja_la() {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let b = store
        .project
        .boards
        .iter_mut()
        .find(|b| b.id == board)
        .expect("là");
    for n in 1..=200 {
        let nom = format!("storyboard-{n}");
        b.storyboards
            .push(Storyboard::neuf(&nom, Grille::default()));
        b.panels.push(case("case-sans-numero", &nom));
    }
    store.resync_next_id();
    let neuf = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 900.0))
        .expect("posé");
    assert_eq!(store.storyboards(&board).len(), 201, "un de plus");
    assert_eq!(store.cases(&board, &neuf).len(), 8, "le sien, et lui seul");
}

/// **La copie d'un storyboard pour ses planches** (P2.7, fiche 68) : ses cases, ce qui est posé
/// dedans — une image, un texte par leur centre —, rien d'autre : ni l'autre storyboard ni ses
/// images, ni une image posée à côté. Vu sur l'aperçu : une planche montrait l'autre storyboard.
#[test]
fn test_la_copie_d_un_storyboard_ne_garde_que_lui() {
    use crate::types::{Annotation, BoardImage};
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let un = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 0.0))
        .expect("posé");
    let deux = store
        .poser_le_storyboard(&board, Grille::default(), (0.0, 600.0))
        .expect("posé");
    let centre = |store: &Store, sb: &str| {
        let c = store.cases(&board, sb)[0].clone();
        (c.x + c.width / 2.0, c.y + c.height / 2.0)
    };
    let (a, b) = (centre(&store, &un), centre(&store, &deux));
    for (id, (x, y)) in [("dans-un", a), ("dans-deux", b), ("a-cote", (5000.0, 0.0))] {
        store.add_image(&board, BoardImage::new(id, x, y, 50.0, 50.0));
    }
    let texte = Annotation::text("texte", a.0 - 20.0, a.1 - 10.0, "une note");
    store
        .active_board_mut()
        .expect("là")
        .annotations
        .push(texte);
    let copie = store.copie_du_storyboard(&un);
    let t = copie.active_board().expect("le tableau");
    assert!(t.panels.iter().all(|c| c.storyboard == un) && t.panels.len() == 8);
    assert_eq!(t.storyboards.len(), 1);
    let images: Vec<&str> = t.images.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(images, ["dans-un"]);
    assert_eq!(t.annotations.len(), 1, "le texte posé dans sa case");
    assert_eq!(
        store.storyboards(&board).len(),
        2,
        "le document n'est pas touché"
    );
}
