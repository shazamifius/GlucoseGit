//! **MEMB-1 — une membrane possède ce qu'on y dépose**, éprouvé par le vrai `Store`.
//!
//! Chaque épreuve pilote le document par ses gestes publics — poser, sélectionner,
//! déplacer, lâcher, supprimer, annuler — et lit l'appartenance que le journal a écrite.

use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};

fn store() -> (Store, String) {
    let mut s = Store::new("appartenance");
    let b = s.project.active_board_id.clone();
    if let Some(board) = s.active_board_mut() {
        board.annotations.clear();
        board.images.clear();
    }
    (s, b)
}

fn membrane(s: &mut Store, b: &str, id: &str, (x, y, w, h): (f64, f64, f64, f64)) {
    s.add_annotation(b, Annotation::membrane(id, x, y, w, h));
}

fn image(s: &mut Store, b: &str, id: &str, x: f64, y: f64) {
    s.add_image(b, BoardImage::new(id, x, y, 40.0, 30.0));
}

fn parent(s: &Store, b: &str, id: &str) -> Option<String> {
    let board = s
        .project
        .boards
        .iter()
        .find(|x| x.id == b)
        .expect("le tableau");
    board
        .images
        .iter()
        .find(|i| i.id == id)
        .map(|i| i.membrane_id.clone())
        .or_else(|| {
            board
                .annotations
                .iter()
                .find(|a| a.id() == id)
                .map(|a| a.membrane_id().map(str::to_string))
        })
        .expect("l'élément existe")
}

fn position(s: &Store, b: &str, id: &str) -> (f64, f64) {
    let board = s
        .project
        .boards
        .iter()
        .find(|x| x.id == b)
        .expect("le tableau");
    board
        .images
        .iter()
        .find(|i| i.id == id)
        .map(|i| (i.x, i.y))
        .or_else(|| {
            board
                .annotations
                .iter()
                .find(|a| a.id() == id)
                .map(|a| (a.x(), a.y()))
        })
        .expect("l'élément existe")
}

fn seul(s: &mut Store, id: &str, est_une_image: bool) {
    s.clear_selection();
    if est_une_image {
        s.select_image(id.to_string(), false);
    } else {
        s.select_annotation(id.to_string(), false);
    }
}

/// Posé dans une membrane, un élément lui appartient dès sa naissance ; posé dehors, il est
/// libre. Et c'est la plus petite membrane qui le contient qui le prend.
#[test]
fn test_la_naissance_donne_la_plus_petite_membrane() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "grande", (0.0, 0.0, 800.0, 600.0));
    membrane(&mut s, &b, "petite", (100.0, 100.0, 200.0, 200.0));
    image(&mut s, &b, "dans-la-petite", 150.0, 150.0);
    image(&mut s, &b, "dans-la-grande", 500.0, 400.0);
    image(&mut s, &b, "dehors", 2000.0, 2000.0);
    assert_eq!(parent(&s, &b, "petite").as_deref(), Some("grande"));
    assert_eq!(parent(&s, &b, "dans-la-petite").as_deref(), Some("petite"));
    assert_eq!(parent(&s, &b, "dans-la-grande").as_deref(), Some("grande"));
    assert_eq!(parent(&s, &b, "dehors"), None);
    assert_eq!(parent(&s, &b, "grande"), None);
}

/// **Déplacer une membrane emporte son contenu**, et celui de ses membranes — et une flèche
/// accrochée à ce contenu suit. Ce qui n'en fait pas partie ne bouge pas. Un `Ctrl+Z` rend
/// tout.
#[test]
fn test_deplacer_une_membrane_emporte_tout_son_contenu() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "grande", (0.0, 0.0, 800.0, 600.0));
    membrane(&mut s, &b, "petite", (100.0, 100.0, 200.0, 200.0));
    image(&mut s, &b, "au-fond", 150.0, 150.0);
    image(&mut s, &b, "voisine", 2000.0, 2000.0);
    let mut fleche = Annotation::arrow("fleche", 170.0, 165.0, 2020.0, 2015.0);
    if let Annotation::Arrow { source_id, .. } = &mut fleche {
        *source_id = Some("au-fond".into());
    }
    s.add_annotation(&b, fleche);
    let avant = s.project.clone();

    seul(&mut s, "grande", false);
    s.move_selected(&b, 30.0, -20.0);
    assert_eq!(position(&s, &b, "grande"), (30.0, -20.0));
    assert_eq!(position(&s, &b, "petite"), (130.0, 80.0));
    assert_eq!(position(&s, &b, "au-fond"), (180.0, 130.0));
    assert_eq!(position(&s, &b, "voisine"), (2000.0, 2000.0));
    assert_eq!(
        position(&s, &b, "fleche"),
        (200.0, 145.0),
        "la flèche suit le bout accroché"
    );

    assert!(s.undo());
    assert!(s.project == avant, "un seul Ctrl+Z rend tout");
}

/// **Le dépôt** : lâcher un élément hors de sa membrane le libère, le lâcher dans une autre
/// l'y range — dans le même geste que le déplacement, qu'un seul `Ctrl+Z` défait.
#[test]
fn test_le_depot_change_l_appartenance_dans_le_meme_geste() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "a", (0.0, 0.0, 300.0, 300.0));
    membrane(&mut s, &b, "b", (1000.0, 0.0, 300.0, 300.0));
    image(&mut s, &b, "photo", 50.0, 50.0);
    assert_eq!(parent(&s, &b, "photo").as_deref(), Some("a"));
    let avant = s.project.clone();

    seul(&mut s, "photo", true);
    s.begin_live_edit();
    s.move_selected(&b, 1000.0, 0.0);
    s.rattacher_la_selection(&b);
    s.end_live_edit();
    assert_eq!(parent(&s, &b, "photo").as_deref(), Some("b"));

    s.begin_live_edit();
    s.move_selected(&b, 0.0, 5000.0);
    s.rattacher_la_selection(&b);
    s.end_live_edit();
    assert_eq!(
        parent(&s, &b, "photo"),
        None,
        "lâchée dehors, elle est libre"
    );

    assert!(s.undo());
    assert_eq!(parent(&s, &b, "photo").as_deref(), Some("b"));
    assert!(s.undo());
    assert!(
        s.project == avant,
        "position et appartenance reviennent ensemble"
    );
}

/// Une membrane ne devient jamais membre de sa propre descendance : le centre de la grande
/// tombe dans la petite qu'elle contient, et rien ne change.
#[test]
fn test_une_membrane_ne_se_range_pas_dans_sa_descendance() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "grande", (0.0, 0.0, 400.0, 400.0));
    membrane(&mut s, &b, "petite", (150.0, 150.0, 100.0, 100.0));
    assert_eq!(parent(&s, &b, "petite").as_deref(), Some("grande"));
    seul(&mut s, "grande", false);
    s.rattacher_la_selection(&b);
    assert_eq!(parent(&s, &b, "grande"), None);
    assert_eq!(parent(&s, &b, "petite").as_deref(), Some("grande"));
}

/// **Supprimer une membrane libère son contenu** vers la membrane qui la contenait : le
/// cadre part, rien de ce qu'il portait. `Ctrl+Z` rend le cadre et son contenu.
#[test]
fn test_supprimer_une_membrane_libere_son_contenu() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "grande", (0.0, 0.0, 800.0, 600.0));
    membrane(&mut s, &b, "petite", (100.0, 100.0, 200.0, 200.0));
    image(&mut s, &b, "photo", 150.0, 150.0);
    let avant = s.project.clone();

    s.remove_annotations(&b, &["petite"]);
    assert_eq!(parent(&s, &b, "photo").as_deref(), Some("grande"));
    s.remove_annotations(&b, &["grande"]);
    assert_eq!(parent(&s, &b, "photo"), None);

    assert!(s.undo());
    assert!(s.undo());
    assert!(s.project == avant);
}

/// Les deux membranes partent ensemble : le contenu remonte jusqu'à la première qui reste —
/// ici aucune.
#[test]
fn test_supprimer_des_membranes_imbriquees_ensemble() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "grande", (0.0, 0.0, 800.0, 600.0));
    membrane(&mut s, &b, "petite", (100.0, 100.0, 200.0, 200.0));
    image(&mut s, &b, "photo", 150.0, 150.0);
    s.remove_annotations(&b, &["petite", "grande"]);
    assert_eq!(parent(&s, &b, "photo"), None);
}

/// L'ordre des écritures d'un dépôt ne doit rien au hasard d'un ensemble : le journal — et
/// l'histoire du document qui le garde — sont les mêmes d'une exécution à l'autre.
#[test]
fn test_un_depot_ecrit_toujours_dans_le_meme_ordre() {
    use glucose_core::store::journal::Edit;
    let (mut s, b) = store();
    membrane(&mut s, &b, "m", (0.0, 0.0, 1000.0, 1000.0));
    let ids: Vec<String> = (0..8).map(|k| format!("p{k}")).collect();
    for (k, id) in ids.iter().enumerate() {
        image(&mut s, &b, id, 50.0 + 100.0 * k as f64, 50.0);
    }
    s.clear_selection();
    for id in &ids {
        s.select_image(id.clone(), true);
    }
    s.journal.prendre_les_ecrits();
    s.begin_live_edit();
    s.move_selected(&b, 0.0, 5000.0);
    s.rattacher_la_selection(&b);
    s.end_live_edit();
    let ecrits = s.journal.prendre_les_ecrits();
    let changees: Vec<String> = ecrits
        .iter()
        .flat_map(|t| &t.edits)
        .filter_map(|e| match e {
            Edit::Image { slot, .. } => slot
                .after
                .as_ref()
                .filter(|a| a.membrane_id.is_none())
                .map(|a| a.id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        changees, ids,
        "les huit libérées, dans l'ordre de leurs noms"
    );
}
