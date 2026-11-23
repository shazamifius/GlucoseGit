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

/// Le geste de l'outil membrane (`place_membrane`, au bureau) : la pose et l'adoption, **un**
/// geste.
fn tracer(s: &mut Store, b: &str, id: &str, cadre: (f64, f64, f64, f64)) {
    s.begin_live_edit();
    membrane(s, b, id, cadre);
    s.adopter_ce_qu_elle_entoure(b, id);
    s.end_live_edit();
}

/// Trois photos de 40 × 30, posées par leur centre, autour du bord droit d'un cadre (0, 0, 320,
/// 300) : l'une dedans, l'une qui déborde (de 290 à 330) mais dont le centre est dedans (310),
/// l'une en partie dedans (de 310 à 350) mais dont le centre est dehors (330). Et une, loin.
fn trois_photos_au_bord(s: &mut Store, b: &str) {
    image(s, b, "dedans", 120.0, 115.0);
    image(s, b, "deborde", 310.0, 115.0);
    image(s, b, "centre-dehors", 330.0, 215.0);
    image(s, b, "loin", 2000.0, 2000.0);
}

/// **MEMB-9 — une membrane tracée prend ce qu'elle entoure, par la loi du dépôt** (fiche 134
/// § 12, sa note du 23/11 au soir) : ce dont le centre est dedans y entre, le reste reste libre ;
/// un seul `Ctrl+Z` défait la pose et l'adoption. **Et l'envers** : la pose seule — l'outil
/// d'avant — ne prenait rien, ses « Groupe » restaient vides.
#[test]
fn test_memb_9_une_membrane_tracee_prend_ce_dont_le_centre_est_dedans() {
    let (mut s, b) = store();
    trois_photos_au_bord(&mut s, &b);
    let avant = s.project.clone();

    tracer(&mut s, &b, "groupe", (0.0, 0.0, 320.0, 300.0));
    assert_eq!(parent(&s, &b, "dedans").as_deref(), Some("groupe"));
    assert_eq!(
        parent(&s, &b, "deborde").as_deref(),
        Some("groupe"),
        "le centre est dedans : la loi du dépôt la range"
    );
    assert_eq!(
        parent(&s, &b, "centre-dehors"),
        None,
        "le centre est dehors"
    );
    assert_eq!(parent(&s, &b, "loin"), None);
    assert_eq!(parent(&s, &b, "groupe"), None, "elle-même à la racine");
    assert!(s.undo(), "un geste");
    assert!(s.project == avant, "un seul Ctrl+Z rend tout");

    // L'envers : la pose seule, la loi d'avant.
    membrane(&mut s, &b, "vide", (0.0, 0.0, 320.0, 300.0));
    assert_eq!(
        parent(&s, &b, "dedans"),
        None,
        "l'outil d'avant ne prenait rien"
    );
}

/// **MEMB-9 dans une membrane** : tracée dans P, la neuve naît dans P et ne prend que les sœurs
/// qu'elle entoure ; une sœur membrane entre avec ce qu'elle contient, qui garde son
/// appartenance (on emballe un pays avec ses villes) ; ce qui reste dehors reste à P.
#[test]
fn test_memb_9_dans_une_membrane_elle_prend_ses_soeurs_et_leur_contenu_suit() {
    let (mut s, b) = store();
    membrane(&mut s, &b, "p", (0.0, 0.0, 1000.0, 800.0));
    image(&mut s, &b, "a", 100.0, 100.0);
    image(&mut s, &b, "b", 700.0, 600.0);
    membrane(&mut s, &b, "soeur", (80.0, 300.0, 200.0, 150.0));
    image(&mut s, &b, "dans-la-soeur", 100.0, 350.0);
    assert_eq!(parent(&s, &b, "soeur").as_deref(), Some("p"));
    assert_eq!(parent(&s, &b, "dans-la-soeur").as_deref(), Some("soeur"));

    tracer(&mut s, &b, "neuve", (50.0, 50.0, 300.0, 450.0));
    assert_eq!(parent(&s, &b, "neuve").as_deref(), Some("p"), "née dans P");
    assert_eq!(parent(&s, &b, "a").as_deref(), Some("neuve"));
    assert_eq!(parent(&s, &b, "soeur").as_deref(), Some("neuve"));
    assert_eq!(
        parent(&s, &b, "dans-la-soeur").as_deref(),
        Some("soeur"),
        "le contenu d'une sœur la suit sans changer d'appartenance"
    );
    assert_eq!(
        parent(&s, &b, "b").as_deref(),
        Some("p"),
        "dehors, il reste à P"
    );
}

/// **MEMB-9 et le dossier, une seule loi** : une membrane et un dossier tracés au même endroit
/// prennent les mêmes nœuds (l'emprise du dossier juge aussi le centre).
#[test]
fn test_memb_9_une_membrane_et_un_dossier_au_meme_endroit_prennent_les_memes() {
    let cadre = (0.0, 0.0, 320.0, 300.0);
    let (mut s, b) = store();
    trois_photos_au_bord(&mut s, &b);
    tracer(&mut s, &b, "groupe", cadre);
    let mut par_la_membrane: Vec<String> = ["dedans", "deborde", "centre-dehors", "loin"]
        .into_iter()
        .filter(|id| parent(&s, &b, id).as_deref() == Some("groupe"))
        .map(str::to_string)
        .collect();
    par_la_membrane.sort();

    let (mut d, bd) = store();
    trois_photos_au_bord(&mut d, &bd);
    let mut dossier = glucose_core::types::CanvasFolder::new("dossier", "Dossier", "");
    (dossier.x, dossier.y, dossier.width, dossier.height) = cadre;
    d.create_folder(&bd, dossier);
    let enfant = d
        .project
        .boards
        .iter()
        .find(|x| x.id == bd)
        .and_then(|x| x.folders.first())
        .map(|f| f.child_board_id.clone())
        .expect("le dossier");
    let mut par_le_dossier: Vec<String> = d
        .project
        .boards
        .iter()
        .find(|x| x.id == enfant)
        .expect("son tableau")
        .images
        .iter()
        .map(|i| i.id.clone())
        .collect();
    par_le_dossier.sort();
    assert_eq!(par_la_membrane, ["deborde", "dedans"]);
    assert_eq!(par_la_membrane, par_le_dossier);
}

/// **MEMB-9 ne touche qu'à ce qu'elle entoure** : une appartenance écrite ailleurs ne se
/// redéduit pas (MEMB-1). Une photo libre posée avant une membrane qui la couvre reste libre
/// quand on trace ailleurs ; une photo d'une autre membrane, dont le centre tombe dans la neuve,
/// reste à la sienne — la neuve ne prend que ses sœurs, et seulement pour elle.
#[test]
fn test_memb_9_elle_ne_prend_que_ses_soeurs_et_seulement_pour_elle() {
    let (mut s, b) = store();
    image(&mut s, &b, "libre", 1500.0, 1500.0);
    membrane(&mut s, &b, "couvre", (1400.0, 1400.0, 300.0, 300.0));
    membrane(&mut s, &b, "ailleurs", (3000.0, 0.0, 100.0, 100.0));
    image(&mut s, &b, "d-ailleurs", 150.0, 150.0);
    if let Some(board) = s.active_board_mut() {
        let photo = board.images.iter_mut().find(|i| i.id == "d-ailleurs");
        photo.expect("la photo").membrane_id = Some("ailleurs".into());
    }
    assert_eq!(
        parent(&s, &b, "libre"),
        None,
        "posée avant, elle reste libre"
    );

    tracer(&mut s, &b, "neuve", (0.0, 0.0, 320.0, 300.0));
    assert_eq!(parent(&s, &b, "libre"), None, "loin d'elle, rien ne change");
    assert_eq!(
        parent(&s, &b, "d-ailleurs").as_deref(),
        Some("ailleurs"),
        "pas une sœur : elle reste à la sienne"
    );
}
