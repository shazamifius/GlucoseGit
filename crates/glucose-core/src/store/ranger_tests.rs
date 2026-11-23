//! RANGER-1 : ranger des nœuds dans un dossier (fiche 134).

use super::Rangement;
use crate::store::agents::Refus;
use crate::store::Store;
use crate::types::{Annotation, Board, BoardImage, CanvasFolder, TextSelection};

/// Une flèche de `source` à `cible`.
fn fleche(id: &str, source: &str, cible: &str) -> Annotation {
    let mut a = Annotation::arrow(id, 0.0, 0.0, 1.0, 1.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut a
    {
        *source_id = Some(source.into());
        *target_id = Some(cible.into());
    }
    a
}

/// Le dossier « archive », loin à gauche ; une note « n » et sa réponse « r » reliées par « f » ;
/// un texte « t » resté dehors, relié à la note par « g » (à un passage de la note) ; et sa flèche
/// « h » de la note vers l'archive. Rend le magasin, le tableau, celui du dossier.
fn scene() -> (Store, String, String) {
    let mut store = Store::new("ranger");
    let board = store.project.active_board_id.clone();
    let mut d = CanvasFolder::new("archive", "archive", "");
    d.x = -2000.0;
    store.create_folder(&board, d);
    let dest = store.active_board().expect("un tableau").folders[0]
        .child_board_id
        .clone();
    let mut g = fleche("g", "t", "n");
    if let Annotation::Arrow {
        target_text_sel, ..
    } = &mut g
    {
        *target_text_sel = Some(TextSelection::Legacy("un bogue".into()));
    }
    for a in [
        Annotation::text("n", 0.0, 0.0, "un bogue"),
        Annotation::text("r", 400.0, 30.0, "corrigé"),
        Annotation::text("t", 0.0, 600.0, "reste"),
        fleche("f", "n", "r"),
        g,
        fleche("h", "n", "archive"),
    ] {
        store.add_annotation(&board, a);
    }
    store.journal.clear();
    (store, board, dest)
}

fn tableau<'s>(store: &'s Store, id: &str) -> &'s Board {
    store
        .project
        .boards
        .iter()
        .find(|b| b.id == id)
        .expect("le tableau")
}

fn ids(b: &Board) -> Vec<&str> {
    let mut v: Vec<&str> = b.annotations.iter().map(Annotation::id).collect();
    v.extend(b.images.iter().map(|i| i.id.as_str()));
    v.extend(b.folders.iter().map(|f| f.id.as_str()));
    v.sort_unstable();
    v
}

fn noeud<'b>(b: &'b Board, id: &str) -> &'b Annotation {
    b.annotations
        .iter()
        .find(|a| a.id() == id)
        .expect("le nœud")
}

/// **La note et sa réponse partent, avec leur flèche** ; la flèche du texte resté dehors vise le
/// dossier ; sa flèche vers l'archive est arrivée. Un seul `Ctrl+Z` remet tout.
#[test]
fn test_ranger_1_la_note_et_sa_reponse_partent_et_ctrl_z_rend_tout() {
    let (mut store, board, dest) = scene();
    let avant = (
        tableau(&store, &board).clone(),
        tableau(&store, &dest).clone(),
    );
    let fait = store.ranger_dans_un_dossier(&board, "archive", &["n", "r"]);
    let attendu = Rangement {
        ouvre: dest.clone(),
        ranges: 3,
        vers_le_dossier: 1,
        arrivees: 1,
    };
    assert_eq!(fait, Ok(attendu));
    assert_eq!(ids(tableau(&store, &board)), ["archive", "g", "t"]);
    assert_eq!(ids(tableau(&store, &dest)), ["f", "n", "r"]);
    let (n, r) = (
        noeud(tableau(&store, &dest), "n"),
        noeud(tableau(&store, &dest), "r"),
    );
    assert_eq!(
        (r.x() - n.x(), r.y() - n.y()),
        (400.0, 30.0),
        "leur disposition"
    );
    match noeud(tableau(&store, &board), "g") {
        Annotation::Arrow {
            source_id,
            target_id,
            target_text_sel,
            ..
        } => {
            assert_eq!(source_id.as_deref(), Some("t"));
            assert_eq!(
                target_id.as_deref(),
                Some("archive"),
                "elle vise le dossier"
            );
            assert_eq!(
                *target_text_sel, None,
                "plus de passage : la note n'est plus là"
            );
        }
        _ => panic!("une flèche"),
    }
    assert!(store.undo(), "un seul geste");
    assert_eq!(
        (tableau(&store, &board), tableau(&store, &dest)),
        (&avant.0, &avant.1)
    );
    assert!(!store.undo(), "rien d'autre");
    assert!(store.redo());
    assert_eq!(ids(tableau(&store, &dest)), ["f", "n", "r"]);
}

/// **Ils arrivent là où un agent poserait le prochain nœud** de ce tableau : à droite de ce que le
/// dossier porte déjà — une seule loi de placement.
#[test]
fn test_ranger_1_ils_arrivent_a_droite_de_ce_que_le_dossier_porte() {
    let (mut store, board, dest) = scene();
    store.add_annotation(&dest, Annotation::text("deja", 100.0, 50.0, "déjà rangé"));
    let place = store.place_pour_un_agent(&dest, None).expect("une place");
    assert!(place.0 > 100.0, "à droite de ce qu'il porte : {place:?}");
    store
        .ranger_dans_un_dossier(&board, "archive", &["r"])
        .expect("rangé");
    let r = noeud(tableau(&store, &dest), "r");
    assert_eq!((r.x(), r.y()), place);
}

/// **Une membrane emporte ses membres**, et les leurs ; un membre dont la membrane reste dehors
/// arrive sans membrane.
#[test]
fn test_ranger_1_une_membrane_emporte_ses_membres() {
    let (mut store, board, dest) = scene();
    let mut dedans = Annotation::text("dedans", 50.0, 1050.0, "membre");
    dedans.set_membrane_id(Some("m".into()));
    let mut sous = Annotation::membrane("sous", 40.0, 1100.0, 200.0, 100.0);
    sous.set_membrane_id(Some("m".into()));
    let mut profond = Annotation::text("profond", 60.0, 1120.0, "membre d'un membre");
    profond.set_membrane_id(Some("sous".into()));
    let mut photo = BoardImage::new("photo", 300.0, 1100.0, 100.0, 80.0);
    photo.membrane_id = Some("m".into());
    let mut seul = Annotation::text("seul", 900.0, 1050.0, "membre d'une autre");
    seul.set_membrane_id(Some("autre".into()));
    let b = store
        .project
        .boards
        .iter_mut()
        .find(|b| b.id == board)
        .expect("le tableau");
    b.annotations
        .push(Annotation::membrane("m", 0.0, 1000.0, 600.0, 300.0));
    b.annotations.extend([dedans, sous, profond, seul]);
    b.images.push(photo);
    store
        .ranger_dans_un_dossier(&board, "archive", &["m", "seul"])
        .expect("rangé");
    let d = tableau(&store, &dest);
    assert_eq!(ids(d), ["dedans", "m", "photo", "profond", "seul", "sous"]);
    assert_eq!(noeud(d, "profond").membrane_id(), Some("sous"));
    assert_eq!(d.images[0].membrane_id.as_deref(), Some("m"));
    assert_eq!(
        noeud(d, "seul").membrane_id(),
        None,
        "sa membrane est restée dehors"
    );
}

/// **Une flèche nommée dont les deux bouts restent ne part pas** : elle pendrait dans le vide.
#[test]
fn test_ranger_1_une_fleche_dont_les_bouts_restent_ne_part_pas() {
    let (mut store, board, dest) = scene();
    let fait = store.ranger_dans_un_dossier(&board, "archive", &["g"]);
    assert_eq!(fait.map(|f| f.ranges), Ok(0));
    assert!(ids(tableau(&store, &board)).contains(&"g"));
    assert!(ids(tableau(&store, &dest)).is_empty());
}

/// **Un dossier rangé dans un dossier garde ce qu'il contient**.
#[test]
fn test_ranger_1_un_dossier_range_garde_son_contenu() {
    let (mut store, board, dest) = scene();
    let note = Annotation::text("dans-b2", 0.0, 0.0, "au fond");
    let mut b2 = CanvasFolder::new("b2", "b2", "");
    b2.x = 3000.0;
    store.create_folder_with_content(&board, b2, vec![note]);
    let ouvre = tableau(&store, &board).folders[1].child_board_id.clone();
    store
        .ranger_dans_un_dossier(&board, "archive", &["b2"])
        .expect("rangé");
    assert_eq!(ids(tableau(&store, &dest)), ["b2"]);
    assert_eq!(
        ids(tableau(&store, &ouvre)),
        ["dans-b2"],
        "son tableau vit encore"
    );
}

/// **Ce qui est refusé ne touche à rien** : un dossier inconnu, un nœud inconnu, un dossier en
/// lui-même.
#[test]
fn test_ranger_1_les_refus_ne_touchent_a_rien() {
    let (mut store, board, _) = scene();
    let refus = [
        (("n", vec!["r"]), Refus::PasUnDossier("n".into())),
        (
            ("archive", vec!["inconnu"]),
            Refus::NoeudInconnu("inconnu".into()),
        ),
        (
            ("archive", vec!["n", "archive"]),
            Refus::DansSoiMeme("archive".into()),
        ),
    ];
    for ((dossier, liste), attendu) in refus {
        assert_eq!(
            store.ranger_dans_un_dossier(&board, dossier, &liste),
            Err(attendu)
        );
    }
    assert!(!store.undo(), "aucun geste");
}

/// **La sélection perd ce qui part**, et garde le reste.
#[test]
fn test_ranger_1_la_selection_perd_ce_qui_part() {
    let (mut store, board, _) = scene();
    store.set_selected_annotation_ids(vec!["n".into(), "t".into()]);
    store
        .ranger_dans_un_dossier(&board, "archive", &["n"])
        .expect("rangé");
    let choisis: Vec<&String> = store.selected_annotation_ids.iter().collect();
    assert_eq!(choisis, [&"t".to_string()]);
}
