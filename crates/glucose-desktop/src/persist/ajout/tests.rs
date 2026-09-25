//! **L'ajout d'un document, de bout en bout**, sur de vrais fichiers : ce qui entre, où ses
//! images se lisent, ce qui reste après que le document d'origine a disparu.

use crate::app::GlucoseApp;
use crate::persist::disque::tests::{application, dossier, image_suivante, noter, png, rouvrir};
use glucose_core::types::Annotation;
use std::path::{Path, PathBuf};

/// Un document enregistré — une carte, une image scellée — puis fermé. Rend son chemin, les
/// octets de son image, et sa clé.
fn un_ancien_document(d: &Path) -> (PathBuf, Vec<u8>, String) {
    let chemin = d.join("ancien.glucose");
    let photo = png(d, "photo.png", 90);
    let octets = std::fs::read(&photo).expect("la photo");
    let mut app = application(d);
    noter(&mut app, "c-ancien", "une idée ancienne");
    app.save_to(chemin.clone());
    app.import_image_files(std::slice::from_ref(&photo));
    image_suivante(&mut app);
    assert!(app.fermer_le_document());
    (chemin, octets, photo.to_string_lossy().into_owned())
}

fn onglets(app: &GlucoseApp) -> Vec<String> {
    app.store.onglets().map(|(_, n, _)| n.to_string()).collect()
}

/// L'image et la carte du tableau actif.
fn contenu_actif(app: &GlucoseApp) -> (Option<String>, Vec<String>) {
    let b = app.store.active_board().expect("un tableau actif");
    let cle = b.images.first().and_then(|i| i.src.clone());
    let textes = b
        .annotations
        .iter()
        .filter_map(Annotation::own_text)
        .collect();
    (cle, textes)
}

/// **Un document ajouté entre dans un onglet neuf, et ses images viennent avec lui** — même
/// quand ce document-ci donne déjà leur clé à d'autres octets, et même quand le document
/// d'origine disparaît ensuite.
#[test]
fn test_un_document_ajoute_emporte_ses_images_meme_sous_une_cle_prise() {
    let d = dossier("ajout-cle-prise");
    let (ancien, octets_anciens, cle) = un_ancien_document(&d);
    // Le document courant : la même clé — le même chemin —, d'autres octets.
    let photo = png(&d, "photo.png", 200);
    let octets_courants = std::fs::read(&photo).expect("la photo");
    let courant = d.join("courant.glucose");
    let mut app = application(&d);
    app.save_to(courant.clone());
    app.import_image_files(std::slice::from_ref(&photo));
    image_suivante(&mut app);

    app.ajouter_un_document(&ancien);
    assert_eq!(onglets(&app), ["Canvas Principal", "ancien"]);
    let (nouvelle, textes) = contenu_actif(&app);
    let nouvelle = nouvelle.expect("l'image est venue");
    assert!(
        textes.iter().any(|t| t == "une idée ancienne"),
        "{textes:?}"
    );
    assert_ne!(
        nouvelle, cle,
        "la clé était prise par d'autres octets : renommée"
    );
    assert_eq!(
        app.disque.objets.lire(&nouvelle).as_deref(),
        Some(&octets_anciens[..])
    );

    image_suivante(&mut app);
    assert!(
        app.disque.objets.est_scellee(&nouvelle),
        "copiée dans ce document"
    );
    std::fs::remove_file(&ancien).expect("l'ancien disparaît");
    assert!(app.fermer_le_document());
    let relu = rouvrir(&d, &courant);
    assert_eq!(onglets(&relu), ["Canvas Principal", "ancien"]);
    assert_eq!(
        relu.disque.objets.lire(&nouvelle).as_deref(),
        Some(&octets_anciens[..])
    );
    assert_eq!(
        relu.disque.objets.lire(&cle).as_deref(),
        Some(&octets_courants[..])
    );
}

/// **Une même image n'entre qu'une fois** : sa clé est déjà là avec les mêmes octets, elle
/// se garde. Et l'ajout est un geste, qu'un `Ctrl+Z` retire.
#[test]
fn test_une_meme_image_garde_sa_cle_et_l_ajout_se_defait() {
    let d = dossier("ajout-meme-image");
    let (ancien, _, cle) = un_ancien_document(&d);
    let mut app = application(&d);
    app.save_to(d.join("courant.glucose"));
    app.import_image_files(&[PathBuf::from(&cle)]);
    image_suivante(&mut app);

    app.ajouter_un_document(&ancien);
    assert_eq!(
        contenu_actif(&app).0.as_deref(),
        Some(cle.as_str()),
        "mêmes octets, même clé"
    );
    assert!(app.ui.toast_message().is_some_and(|m| m.contains("ajouté")));
    assert!(app.store.undo(), "un geste");
    assert_eq!(onglets(&app), ["Canvas Principal"]);
}

/// **Le document courant s'ajoute à lui-même** — le lire ne demande pas de l'écrire — et un
/// document introuvable le dit sans rien changer.
#[test]
fn test_le_document_courant_s_ajoute_a_lui_meme_et_un_absent_le_dit() {
    let d = dossier("ajout-soi");
    let courant = d.join("courant.glucose");
    let mut app = application(&d);
    noter(&mut app, "c-1", "moi");
    app.save_to(courant.clone());
    image_suivante(&mut app);
    app.ajouter_un_document(&courant);
    assert_eq!(onglets(&app), ["Canvas Principal", "courant"]);

    let avant = app.store.project.clone();
    app.ajouter_un_document(&d.join("n-existe-pas.glucose"));
    assert!(app.store.project == avant, "rien n'a changé");
    assert!(app
        .ui
        .toast_message()
        .is_some_and(|m| m.contains("n'a pas pu")));
}

/// Un document de Glucose Tauri, dans sa forme JSON : une carte sans hauteur, et une image
/// portée en base64.
const DOCUMENT_TAURI: &str = r##"{
  "version": "1.0.0", "name": "Tauri", "activeBoardId": "b1", "createdAt": 1, "updatedAt": 2,
  "presets": [], "domains": [],
  "boards": [{
    "id": "b1", "name": "Principal", "createdAt": 4, "updatedAt": 5,
    "viewport": {"x": 0, "y": 0, "scale": 1}, "bookmarks": {},
    "panels": [], "zones": [], "folders": [],
    "images": [
      {"id": "i1", "x": 0, "y": 0, "width": 10, "height": 10, "rotation": 0, "locked": false,
       "tags": [], "originalWidth": 10, "originalHeight": 10,
       "src": "data:image/png;base64,iVBORw0KGgo="}
    ],
    "annotations": [
      {"type": "text", "id": "t1", "x": 1, "y": 2, "text": "# un titre\n\net un paragraphe", "fontSize": 14, "width": 240}
    ]
  }]
}"##;

/// **Un document de Glucose Tauri s'ajoute aussi** : ses cartes sont mesurées avant le geste,
/// et son image portée en base64 se scelle dans ce document.
#[test]
fn test_un_document_tauri_s_ajoute_mesure_et_scelle() {
    let d = dossier("ajout-tauri");
    let tauri = d.join("vieux.glucose");
    std::fs::write(&tauri, DOCUMENT_TAURI).expect("le document Tauri");
    let mut app = application(&d);
    app.save_to(d.join("courant.glucose"));
    app.ajouter_un_document(&tauri);
    assert_eq!(onglets(&app), ["Canvas Principal", "vieux"]);
    let b = app.store.active_board().expect("l'onglet ajouté");
    let hauteur = b.annotations.iter().find_map(|a| match a {
        Annotation::Text { height, .. } => *height,
        _ => None,
    });
    assert!(
        hauteur.is_some_and(|h| h > 0.0),
        "la carte est mesurée : {hauteur:?}"
    );
    let cle = b.images[0].src.clone().expect("l'image a une clé");
    image_suivante(&mut app);
    assert!(
        app.disque.objets.est_scellee(&cle),
        "scellée dans ce document"
    );
}

/// **Lâché sur la barre d'onglets, un document s'ajoute** ; lâché sur le canevas, il se pose
/// comme avant — une tuile qui mène au fichier.
#[test]
fn test_un_document_lache_sur_les_onglets_s_ajoute_et_sur_le_canevas_se_pose() {
    let d = dossier("ajout-depot");
    let (ancien, _, _) = un_ancien_document(&d);
    let mut app = application(&d);
    let moisson = crate::plateforme::moisson::Moisson {
        chemins: vec![ancien],
        ..Default::default()
    };
    let y_onglets = f64::from(app.ui.topbar_height() + app.ui.tabs_height() / 2.0);
    app.poser_une_moisson(&moisson, Some((300.0, y_onglets)), None);
    assert_eq!(onglets(&app), ["Canvas Principal", "ancien"]);

    let onglet_ajoute = app.store.project.active_board_id.clone();
    app.poser_une_moisson(&moisson, Some((600.0, 500.0)), None);
    assert_eq!(
        onglets(&app).len(),
        2,
        "sur le canevas, pas d'onglet de plus"
    );
    let posees = app
        .store
        .project
        .boards
        .iter()
        .find(|b| b.id == onglet_ajoute)
        .map_or(0, |b| b.annotations.len());
    assert!(
        posees >= 2,
        "la tuile du document s'est posée : {posees} annotation(s)"
    );
}
