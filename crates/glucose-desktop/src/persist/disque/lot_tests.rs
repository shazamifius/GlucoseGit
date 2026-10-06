//! **Copier, couper, coller des nœuds, d'une fenêtre à l'autre** (fiche 51 § 2), de bout en
//! bout : de vraies applications, de vrais documents sur le disque, de vraies images scellées.
//!
//! Deux applications d'un même fil d'épreuve partagent son presse-papiers à soi
//! ([`crate::interactions::presse_papiers`]) : c'est exactement deux fenêtres de Glucose qui
//! partagent celui du système.

use super::tests::{application, dossier, image_suivante, noter, png};
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, Project};

/// Un document : une membrane qui possède une photo, une note dehors, et une flèche de la note
/// à la photo. Rend les octets de la photo, dont le fichier d'origine a disparu.
fn scene(app: &mut GlucoseApp, d: &std::path::Path) -> Vec<u8> {
    let b = app.store.project.active_board_id.clone();
    if let Some(board) = app.store.active_board_mut() {
        board.annotations.clear();
        board.images.clear();
    }
    app.store
        .add_annotation(&b, Annotation::membrane("m", 0.0, 0.0, 400.0, 300.0));
    let photo = png(d, "photo.png", 90);
    app.place_image_file(&b, &photo, (50.0, 50.0))
        .expect("la photo");
    noter(app, "note", "Une note qui part avec");
    let image = app.store.project.boards[0].images[0].id.clone();
    let mut fleche = Annotation::arrow("f", 0.0, 0.0, 60.0, 60.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut fleche
    {
        *source_id = Some("note".into());
        *target_id = Some(image);
    }
    app.store.add_annotation(&b, fleche);
    image_suivante(app);
    // L'original disparaît : ce que l'autre fenêtre montrera ne peut venir que du lot.
    let octets = std::fs::read(&photo).expect("ses octets");
    std::fs::remove_file(&photo).expect("l'original disparaît");
    octets
}

fn tout_selectionner(app: &mut GlucoseApp) {
    app.store.clear_selection();
    app.store
        .set_selected_annotation_ids(vec!["m".into(), "note".into()]);
}

/// Le lot que la sélection emporte, ses identifiants remplacés par leur rang et ses places
/// rapportées à celle de la membrane : deux lots identiques à leurs noms et à leur place près
/// donnent le même texte. Le collage pose le lot au curseur ; ce sont les places **relatives**
/// qui doivent revenir.
fn sans_les_noms(app: &GlucoseApp) -> String {
    let b = app.store.project.active_board_id.clone();
    let mut lot: Project = app.store.extraire_la_selection(&b).expect("un lot");
    let board = &mut lot.boards[0];
    let (ox, oy) = (board.annotations[0].x(), board.annotations[0].y());
    for a in &mut board.annotations {
        a.translate(-ox, -oy);
    }
    for i in &mut board.images {
        i.x -= ox;
        i.y -= oy;
    }
    let board = &lot.boards[0];
    let ids: Vec<String> = board
        .images
        .iter()
        .map(|i| i.id.clone())
        .chain(board.annotations.iter().map(|a| a.id().to_string()))
        .collect();
    let mut texte = format!("{:?}", board.annotations);
    texte.push_str(&format!(
        "{:?}",
        board
            .images
            .iter()
            .map(|i| (i.x, i.y, i.width, i.height, i.membrane_id.clone()))
            .collect::<Vec<_>>()
    ));
    for (rang, id) in ids.iter().enumerate() {
        texte = texte.replace(&format!("\"{id}\""), &format!("\"#{rang}\""));
    }
    texte
}

/// **Une sélection mêlée copiée dans une fenêtre revient identique dans une autre**, sous des
/// noms neufs : la membrane, la photo qu'elle possède, la note, la flèche qui les relie — et
/// les octets de la photo, que l'autre document scelle à son tour.
#[test]
fn test_lot_une_selection_melee_passe_d_une_fenetre_a_l_autre() {
    let (da, db) = (dossier("lot-a"), dossier("lot-b"));
    let mut a = application(&da);
    a.save_to(da.join("a.glucose"));
    let octets = scene(&mut a, &da);
    tout_selectionner(&mut a);
    let attendu = sans_les_noms(&a);
    a.copy_selection(false);
    a.suivre_les_echanges(true);

    let mut b = application(&db);
    b.save_to(db.join("b.glucose"));
    if let Some(board) = b.store.active_board_mut() {
        board.annotations.clear();
    }
    b.paste_from_clipboard();
    b.suivre_les_echanges(true);
    assert_eq!(
        sans_les_noms(&b),
        attendu,
        "le même lot, sous des noms neufs"
    );

    let cle = b.store.project.boards[0].images[0]
        .src
        .clone()
        .expect("une clé");
    assert_eq!(b.disque.objets.lire(&cle), Some(octets), "les octets mêmes");
    image_suivante(&mut b);
    assert!(
        b.disque.objets.est_scellee(&cle),
        "scellés dans l'autre document"
    );
}

/// **Couper puis coller ne perd rien**, et chaque geste se défait d'un coup.
#[test]
fn test_lot_couper_puis_coller_ne_perd_rien() {
    let d = dossier("lot-couper");
    let mut app = application(&d);
    app.save_to(d.join("c.glucose"));
    scene(&mut app, &d);
    tout_selectionner(&mut app);
    let attendu = sans_les_noms(&app);
    let avant = app.store.project.clone();
    app.copy_selection(true);
    app.suivre_les_echanges(true);
    let board = &app.store.project.boards[0];
    assert!(
        board.images.is_empty() && board.annotations.is_empty(),
        "coupé"
    );

    app.paste_from_clipboard();
    app.suivre_les_echanges(true);
    assert_eq!(sans_les_noms(&app), attendu, "tout revient");
    assert!(app.store.undo(), "le collage se défait");
    assert!(app.store.undo(), "la coupe se défait");
    assert_eq!(
        app.store.project, avant,
        "en deux gestes, tout est comme avant"
    );
}

/// **`Ctrl+V` tapé avant que la copie soit prête attend son tour** : il colle le lot qu'on vient
/// de copier, jamais ce que le presse-papiers portait avant.
#[test]
fn test_lot_coller_avant_la_fin_de_la_copie_attend_son_tour() {
    let d = dossier("lot-attendre");
    let mut app = application(&d);
    app.save_to(d.join("w.glucose"));
    scene(&mut app, &d);
    crate::interactions::presse_papiers::ouvrir()
        .and_then(|mut p| p.ecrire("l'ancien contenu".into()))
        .expect("écrit");
    tout_selectionner(&mut app);
    let notes = app.store.project.boards[0].annotations.len();
    app.copy_selection(false);
    app.paste_from_clipboard();
    app.suivre_les_echanges(true);
    let board = &app.store.project.boards[0];
    assert_eq!(board.annotations.len(), notes * 2, "le lot, collé une fois");
    assert!(
        !board
            .annotations
            .iter()
            .any(|a| a.own_text().as_deref() == Some("l'ancien contenu")),
        "et rien de l'ancien contenu"
    );
}

/// **Un échec du presse-papiers ne retire rien** : couper sans avoir copié perdrait la sélection.
#[test]
fn test_lot_une_coupe_qui_echoue_ne_retire_rien() {
    let d = dossier("lot-echec");
    let mut app = application(&d);
    scene(&mut app, &d);
    tout_selectionner(&mut app);
    let avant = app.store.project.clone();
    app.copy_selection(true);
    app.finir_la_copie_pour_l_epreuve(None);
    assert_eq!(app.store.project, avant, "rien n'est retiré");
    assert!(app
        .ui
        .toast_message()
        .is_some_and(|m| m.contains("n'a pas abouti")));
}
