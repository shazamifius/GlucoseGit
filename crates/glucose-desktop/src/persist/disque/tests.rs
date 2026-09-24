//! **L'histoire, par la vraie application** : chaque épreuve passe par `GlucoseApp` et ses
//! gestes — importer, enregistrer, ouvrir, fermer —, jamais par un état posé à la main.

use crate::app::GlucoseApp;
use glucose_core::hash::{hex_of, sha256};
use glucose_core::types::Annotation;
use std::path::{Path, PathBuf};

/// Un dossier à soi, vidé : les épreuves tournent en parallèle.
fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir()
        .join("glucose-tests-histoire")
        .join(nom);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dossier d'épreuve");
    d
}

/// Une application dont les brouillons naissent dans `d`, jamais chez l'utilisateur.
fn application(d: &Path) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.disque.brouillons = d.join("brouillons");
    app
}

/// Pose une carte **par la fabrique de l'application** : mesurée, comme un clic la pose.
fn noter(app: &mut GlucoseApp, id: &str, texte: &str) {
    let b = app.store.project.active_board_id.clone();
    let carte = crate::interactions::tools::text_card(
        &app.renderer.typography,
        &app.renderer.math,
        id,
        10.0,
        20.0,
        texte,
    );
    app.store.add_annotation(&b, carte);
}

/// Ce que l'image suivante ferait : écrire le document, puis attendre le disque.
fn image_suivante(app: &mut GlucoseApp) {
    app.consigner();
    if let Some(e) = &app.disque.ecriture {
        e.synchroniser().expect("le disque doit suivre");
    }
}

fn png(d: &Path, nom: &str, teinte: u8) -> PathBuf {
    let chemin = d.join(nom);
    let img = image::RgbaImage::from_pixel(8, 6, image::Rgba([teinte, 40, 200, 255]));
    img.save(&chemin).expect("png d'épreuve");
    chemin
}

fn rouvrir(d: &Path, chemin: &Path) -> GlucoseApp {
    let mut autre = application(d);
    autre.open_from(chemin.to_path_buf());
    autre
}

/// **Un document nommé s'enregistre sans `Ctrl+S`** : ce qu'on y change est écrit à l'image
/// suivante, et une autre application le relit.
#[test]
fn test_un_document_nomme_s_enregistre_sans_ctrl_s() {
    let d = dossier("sans-ctrl-s");
    let chemin = d.join("carnet.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    noter(&mut app, "idee", "écrite après l'enregistrement");
    image_suivante(&mut app);
    assert!(
        !app.is_dirty(),
        "un document nommé ne reste pas « modifié »"
    );
    let autre = rouvrir(&d, &chemin);
    assert_eq!(autre.store.project, app.store.project);
}

/// **Une image déposée entre dans le document** et n'a plus besoin de son fichier : c'est ce
/// qui met fin aux images perdues du dossier temporaire (fiche 33 § 5.1).
#[test]
fn test_une_image_deposee_est_scellee_et_survit_a_son_fichier() {
    let d = dossier("scellee");
    let chemin = d.join("photos.glucose");
    let source = png(&d, "depot.png", 90);
    let octets = std::fs::read(&source).unwrap();
    let mut app = application(&d);
    app.save_to(chemin.clone());
    app.import_image_files(std::slice::from_ref(&source));
    image_suivante(&mut app);
    let cle = source.to_string_lossy().into_owned();
    assert!(
        app.disque.objets.est_scellee(&cle),
        "l'image est dans le document"
    );
    std::fs::remove_file(&source).unwrap();

    let autre = rouvrir(&d, &chemin);
    assert_eq!(autre.store.project, app.store.project);
    assert_eq!(
        autre.disque.objets.lire(&cle).as_deref(),
        Some(&octets[..]),
        "les octets se relisent du document, le fichier d'origine n'existant plus"
    );
}

/// Une image dont le fichier paraît après coup — une image collée, que l'atelier écrit —
/// se scelle dès qu'il paraît.
#[test]
fn test_une_image_dont_le_fichier_parait_apres_coup_se_scelle_quand_il_parait() {
    let d = dossier("apres-coup");
    let chemin = d.join("colle.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    let a_venir = d.join("pas-encore.png");
    let b = app.store.project.active_board_id.clone();
    let mut img = glucose_core::types::BoardImage::new("collee", 0.0, 0.0, 8.0, 6.0);
    img.src = Some(a_venir.to_string_lossy().into_owned());
    app.store.add_image(&b, img);
    image_suivante(&mut app);
    let cle = a_venir.to_string_lossy().into_owned();
    assert!(!app.disque.objets.est_scellee(&cle));
    png(&d, "pas-encore.png", 10);
    image_suivante(&mut app);
    assert!(
        app.disque.objets.est_scellee(&cle),
        "scellée dès que son fichier existe"
    );
}

/// **« Enregistrer sous » emporte l'histoire** : la copie continue, l'original ne bouge plus.
#[test]
fn test_enregistrer_sous_emporte_l_histoire_et_laisse_l_original() {
    let d = dossier("enregistrer-sous");
    let (a, b) = (d.join("a.glucose"), d.join("b.glucose"));
    let mut app = application(&d);
    app.save_to(a.clone());
    noter(&mut app, "un", "dans les deux");
    image_suivante(&mut app);
    let etat_de_a = app.store.project.clone();
    app.save_to(b.clone());
    noter(&mut app, "deux", "dans b seulement");
    image_suivante(&mut app);

    assert_eq!(
        rouvrir(&d, &a).store.project,
        etat_de_a,
        "l'original ne bouge plus"
    );
    let copie = rouvrir(&d, &b);
    assert_eq!(copie.store.project, app.store.project);
}

/// **Un document sans nom écrit dans un brouillon**, que le lancement suivant retrouve — mais
/// jamais celui qu'une autre fenêtre tient encore.
#[test]
fn test_un_brouillon_nait_au_premier_geste_et_se_retrouve() {
    let d = dossier("brouillon");
    let mut app = application(&d);
    noter(
        &mut app,
        "sans-nom",
        "du travail qu'un plantage ne doit pas perdre",
    );
    image_suivante(&mut app);
    let brouillons: Vec<_> = std::fs::read_dir(d.join("brouillons")).unwrap().collect();
    assert_eq!(brouillons.len(), 1, "le premier geste ouvre un brouillon");

    let mut voisine = application(&d);
    voisine.retrouver_un_brouillon();
    assert!(
        voisine.store.project != app.store.project,
        "un brouillon tenu par une autre fenêtre ne se vole pas"
    );

    let attendu = app.store.project.clone();
    drop(app);
    let mut relance = application(&d);
    relance.retrouver_un_brouillon();
    assert_eq!(relance.store.project, attendu);
    assert!(relance.is_dirty(), "du travail sans nom est « modifié »");
    assert!(relance.project_path.is_none());

    assert!(relance.close_with(crate::persist::close::CloseChoice::Discard));
    let restants = std::fs::read_dir(d.join("brouillons")).unwrap().count();
    assert_eq!(restants, 0, "« ne pas enregistrer » efface le brouillon");
}

/// **Un document portable de Glucose Tauri** s'importe : ses images s'affichent depuis son
/// dossier `objects/`, puis entrent dans le fichier de Glucose Rust — qui n'a plus besoin de
/// ce dossier.
#[test]
fn test_un_document_tauri_portable_s_importe_et_ses_images_se_scellent() {
    let d = dossier("tauri");
    let portable = d.join("Projet-portable");
    std::fs::create_dir_all(portable.join("objects")).unwrap();
    let pixels = std::fs::read(png(&d, "source.png", 200)).unwrap();
    let nom = format!("{}.png", &hex_of(&sha256(&pixels))[..16]);
    std::fs::write(portable.join("objects").join(&nom), &pixels).unwrap();
    let doc = format!(
        r#"{{"version":"1.0.0","name":"Venu de Tauri","activeBoardId":"b","boards":[{{
            "id":"b","name":"B","viewport":{{"x":0,"y":0,"scale":1}},"annotations":[
              {{"type":"text","id":"t","x":1,"y":2,"text":"bonjour"}}],
            "images":[{{"id":"i","x":5,"y":5,"width":80,"height":60,"rotation":0,"locked":false,
              "tags":[],"originalWidth":8,"originalHeight":6,
              "asset":{{"mode":"link","href":"asset:{nom}"}}}}]}}]}}"#
    );
    let tauri = portable.join("project.glucose");
    std::fs::write(&tauri, doc).unwrap();

    let mut app = application(&d);
    app.open_from(tauri.clone());
    assert_eq!(app.store.project.name, "Venu de Tauri");
    assert!(
        app.project_path.is_none(),
        "un import n'a pas encore de nom"
    );
    assert!(app.is_dirty());
    let cle = app.store.project.boards[0].images[0]
        .src
        .clone()
        .expect("une clé");
    assert_eq!(app.disque.objets.lire(&cle).as_deref(), Some(&pixels[..]));

    let rust = d.join("rust.glucose");
    app.save_to(rust.clone());
    image_suivante(&mut app);
    std::fs::remove_dir_all(portable.join("objects")).unwrap();
    let autre = rouvrir(&d, &rust);
    assert_eq!(autre.store.project, app.store.project);
    assert_eq!(autre.disque.objets.lire(&cle).as_deref(), Some(&pixels[..]));
    assert_eq!(
        std::fs::read(&tauri).unwrap().first(),
        Some(&b'{'),
        "le fichier de Tauri n'est jamais réécrit"
    );
}

/// **Un fichier v2 d'hier** s'ouvre, reçoit l'histoire à sa suite, et passe en version 3.
#[test]
fn test_un_fichier_v2_s_ouvre_recoit_l_histoire_et_passe_en_v3() {
    let d = dossier("v2");
    let chemin = d.join("ancien.glucose");
    let mut projet = glucose_core::types::Project::new("d'hier");
    projet.boards[0]
        .annotations
        .push(Annotation::text("a", 0.0, 0.0, "écrit en v2"));
    let mut octets = glucose_core::persist::encode(&projet, &Default::default(), 0);
    octets[8..10].copy_from_slice(&2u16.to_le_bytes());
    std::fs::write(&chemin, &octets).unwrap();

    let mut app = application(&d);
    app.open_from(chemin.clone());
    noter(&mut app, "b", "écrit après");
    image_suivante(&mut app);
    let relu = std::fs::read(&chemin).unwrap();
    assert_eq!(u16::from_le_bytes([relu[8], relu[9]]), 3);
    assert_eq!(rouvrir(&d, &chemin).store.project, app.store.project);
}

/// **Une fin déchirée** — un plantage au milieu d'une écriture — laisse tout le reste, et se
/// dit à l'ouverture.
#[test]
fn test_une_fin_dechiree_par_un_plantage_laisse_tout_le_reste() {
    let d = dossier("plantage");
    let chemin = d.join("interrompu.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    noter(&mut app, "gardee", "avant le plantage");
    image_suivante(&mut app);
    let attendu = app.store.project.clone();
    drop(app);
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&chemin)
        .unwrap();
    // Quatre kilo-octets de débris : plus que ce que la prochaine entrée recouvrira. Sans
    // troncature, ils resteraient derrière elle, et chaque ouverture dirait « interrompu ».
    let debris: Vec<u8> = (0..4096u32).map(|i| (i * 37 % 251) as u8).collect();
    std::io::Write::write_all(&mut f, &debris).unwrap();
    drop(f);

    let mut autre = rouvrir(&d, &chemin);
    assert_eq!(autre.store.project, attendu);
    let toast = autre.ui.current_toast.as_ref().expect("l'ouverture parle");
    assert!(toast.message.contains("interrompu"), "{}", toast.message);
    noter(&mut autre, "apres", "écrit par-dessus la fin déchirée");
    image_suivante(&mut autre);
    let troisieme = rouvrir(&d, &chemin);
    assert_eq!(troisieme.store.project, autre.store.project);
    let toast = troisieme
        .ui
        .current_toast
        .as_ref()
        .expect("l'ouverture parle");
    assert!(
        !toast.message.contains("interrompu"),
        "les débris ont été retirés en écrivant : {}",
        toast.message
    );
}
