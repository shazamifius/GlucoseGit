//! Ce que `Ctrl+C` emporte, et ce que `Ctrl+V` pose.
//!
//! Sur le presse-papiers **à soi** du fil d'épreuve ([`crate::interactions::presse_papiers`]),
//! jamais celui du système : une épreuve qui l'écrivait remplaçait, à chaque `cargo test`, ce
//! que l'utilisateur venait de copier. La **décision** — quoi copier, et sous quelle forme —
//! portait la faute : `Ctrl+C` hors saisie n'existait pas du tout.

use super::*;
use crate::interactions::resize::tests::text_card;
use glucose_core::types::{Annotation, BoardImage};

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app
}

fn pose_carte(app: &mut GlucoseApp, id: &str, texte: &str) {
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, text_card(id, 0.0, 0.0, 240.0, texte));
}

/// Rien de sélectionné : rien à copier, et surtout pas un presse-papiers vidé.
#[test]
fn test_an_empty_selection_copies_nothing() {
    let app = app();
    assert_eq!(app.store.selection_as_text(), None);
}

/// Une carte sélectionnée rend son texte — le geste que la table des raccourcis oubliait.
#[test]
fn test_a_selected_card_gives_its_text() {
    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    assert_eq!(app.store.selection_as_text().as_deref(), Some("bonjour"));
}

/// Plusieurs cartes se séparent par une ligne vide : le collage les relira comme deux blocs
/// et non comme une phrase coupée en deux.
#[test]
fn test_several_cards_are_separated_by_a_blank_line() {
    let mut app = app();
    pose_carte(&mut app, "a", "premier");
    pose_carte(&mut app, "b", "second");
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    assert_eq!(
        app.store.selection_as_text().as_deref(),
        Some("premier\n\nsecond")
    );
}

/// **Une image ne rend aucun texte** : son nom interne ne veut rien dire pour un autre logiciel.
/// Elle voyage dans le lot (fiche 51 § 2).
#[test]
fn test_an_image_gives_no_text() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("i", 0.0, 0.0, 100.0, 80.0);
    img.src = Some("C:/photos/chat.png".into());
    app.store.add_image(&board, img);
    app.store.set_selected_image_ids(vec!["i".into()]);
    assert_eq!(app.store.selection_as_text(), None);
}

/// Une carte vide ne rend pas une ligne vide : elle ne rend rien.
#[test]
fn test_a_blank_card_contributes_nothing() {
    let mut app = app();
    pose_carte(&mut app, "a", "   ");
    pose_carte(&mut app, "b", "vrai texte");
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    assert_eq!(app.store.selection_as_text().as_deref(), Some("vrai texte"));
}

/// `Ctrl+C` et `Ctrl+X` hors saisie atteignent la copie — en un seul test, et c'est voulu.
///
/// Le presse-papiers est une ressource **unique du système** : deux tests qui l'écrivent en
/// parallèle se la disputent, l'un échoue, et `Ctrl+X` ne supprime alors rien puisqu'il ne
/// coupe que ce qu'il a réussi à copier. Écrits séparément, ils passaient seuls et tombaient
/// en suite complète — un test qui échoue au hasard ne dit plus rien et abîme la valeur des
/// autres. Un seul test séquentiel supprime la course.
#[test]
fn test_ctrl_c_and_ctrl_x_reach_the_copy_outside_a_text_session() {
    use winit::event::ElementState;
    use winit::keyboard::{Key, ModifiersState};

    let mut app = app();
    pose_carte(&mut app, "a", "bonjour");
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("c".into()), ElementState::Pressed);
    app.suivre_les_echanges(true);
    // L'accusé doit dire **copié**, pas n'importe quoi : un toast quelconque serait vert
    // même quand l'écriture échoue, et ce test-là mentirait exactement comme celui de la
    // touche Entrée mentait.
    let message = app
        .ui
        .current_toast
        .as_ref()
        .map(|t| t.message.clone())
        .unwrap_or_default();
    assert!(
        message.contains("copié"),
        "Ctrl+C doit dire ce qu'il a copié, pas \"{message}\""
    );
    let reste = app
        .store
        .active_board()
        .map(|b| b.annotations.len())
        .unwrap_or(0);
    assert_eq!(reste, 1, "copier ne retire rien");

    // Et couper emporte **et** retire.
    app.handle_shortcut_input(&Key::Character("x".into()), ElementState::Pressed);
    app.suivre_les_echanges(true);
    let reste = app
        .store
        .active_board()
        .map(|b| b.annotations.len())
        .unwrap_or(0);
    assert_eq!(reste, 0, "couper doit aussi retirer");
}

/// **Coller un texte pose une carte qui le porte**, là où est la souris — par la fabrique, qui
/// la mesure — et sans un mot (fiche 51 § 2).
#[test]
fn test_coller_un_texte_pose_une_carte() {
    let mut app = app();
    crate::interactions::presse_papiers::ouvrir()
        .and_then(|mut a| a.ecrire("  bonjour\ndeux lignes \n".into()))
        .expect("écrit");
    let message_d_avant = app.ui.toast_message().map(str::to_string);
    app.paste_from_clipboard();
    let board = app.store.active_board().expect("tableau");
    assert_eq!(board.annotations.len(), 1, "une carte");
    let Annotation::Text { text, .. } = &board.annotations[0] else {
        panic!("une carte de texte");
    };
    assert_eq!(
        text, "bonjour\ndeux lignes",
        "le texte, sans ses blancs de bord"
    );
    // La carte apparaît sous le curseur : cela se regarde, et aucun message n'en parle.
    assert_eq!(
        app.ui.toast_message(),
        message_d_avant.as_deref(),
        "un collage réussi se tait"
    );
}

/// Fiche 172 § 6 — **« Ajouter des images… » s'ouvre aux vidéos là où elles jouent** : son nom
/// (à la palette comme au menu du vide) et son filtre suivent le même drapeau. Les HEIC y sont,
/// et chaque extension aussi en majuscules : `IMG_0001.JPG`, le nom d'un appareil photo, restait
/// caché sous Linux.
#[cfg(not(target_os = "android"))]
#[test]
fn import_1_le_nom_et_le_filtre_suivent_les_videos() {
    use crate::renderer::video::PREVUES_ICI;
    let fichier = filtre_de_l_import();
    let [(nom, extensions)] = fichier.filtres() else {
        panic!("un seul filtre : {:?}", fichier.filtres());
    };
    let a = |e: &str| extensions.iter().any(|x| x == e);
    for e in ["jpg", "JPG", "png", "PNG", "heic", "HEIC", "heif"] {
        assert!(a(e), "{e} : {extensions:?}");
    }
    let uniques: std::collections::HashSet<_> = extensions.iter().collect();
    assert_eq!(
        uniques.len(),
        extensions.len(),
        "aucune extension deux fois"
    );
    assert_eq!(["mp4", "MP4", "mov", "webm"].map(a), [PREVUES_ICI; 4]);
    assert_eq!(nom.contains("vidéos"), PREVUES_ICI, "{nom}");
    assert_eq!(NOM_DE_L_IMPORT.contains("vidéos"), PREVUES_ICI);
    assert_eq!(
        crate::commandes::Commande::ImporterDesImages.nom(),
        NOM_DE_L_IMPORT
    );
}

/// Fiche 172 § 6 — **Coller un chemin** : celui, absolu, d'une image la pose ; tout autre texte
/// se colle tel quel — « Cargo.toml », qui existe pourtant là d'où l'épreuve part, comme le
/// chemin absolu d'un fichier qui n'est pas une image. Avant, les deux se perdaient sous une
/// erreur.
#[test]
fn import_1_coller_un_chemin_pose_l_image_ou_garde_le_texte() {
    let d = crate::epreuve::Dossier::nouveau("coller-un-chemin");
    let photo = crate::persist::disque::tests::png(&d, "photo.png", 40);
    let manifeste = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    assert!(
        Path::new("Cargo.toml").is_file(),
        "l'épreuve part de la caisse"
    );
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.coller_un_texte(&board, (0.0, 0.0), "Cargo.toml");
    app.coller_un_texte(&board, (0.0, 200.0), manifeste);
    app.coller_un_texte(&board, (0.0, 400.0), &format!(" {}\n", photo.display()));
    let b = app.store.active_board().expect("le tableau");
    let textes: Vec<String> = b
        .annotations
        .iter()
        .filter_map(Annotation::own_text)
        .collect();
    assert_eq!(textes, ["Cargo.toml", manifeste]);
    assert_eq!(b.images.len(), 1, "la photo");
}

/// Fiche 172 § 6 — **Une vidéo choisie au dialogue d'import, ou nommée par un chemin collé**,
/// pose la question d'une vidéo déposée (VIDEO-2) ; l'image choisie avec elle se pose, et le
/// compte-rendu ne compte qu'elle. Avant, la vidéo n'était qu'une erreur. Un chemin relatif,
/// lui, reste un texte.
#[cfg(target_os = "linux")]
#[test]
fn import_1_une_video_choisie_ou_collee_pose_sa_question() {
    use crate::ui::question::Reponse;
    if !crate::renderer::video::lisible_ici() {
        return;
    }
    let video = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/donnees/rouge-64x48.mp4");
    let d = crate::epreuve::Dossier::nouveau("importer-une-video");
    let photo = crate::persist::disque::tests::png(&d, "photo.png", 40);
    let mut app = app();
    app.ui.current_toast = None;
    crate::dialogue::epreuve::choisir(Some(vec![photo, PathBuf::from(video)]));
    app.pick_and_import_images();
    app.suivre_les_fichiers_choisis();
    let images = |app: &GlucoseApp| app.store.active_board().expect("tableau").images.clone();
    assert_eq!(images(&app).len(), 1, "la photo, la vidéo attend");
    assert_eq!(app.videos_en_question(), 1);
    assert_eq!(app.ui.toast_message(), Some("1 élément posé"));
    app.repondre_a_la_question(Some(Reponse::Oui));
    assert_eq!(images(&app).iter().filter(|i| i.is_video).count(), 1);

    // Relatif, le chemin dépendrait du dossier d'où Glucose est parti : il reste un texte.
    let board = app.store.project.active_board_id.clone();
    app.coller_un_texte(&board, (0.0, 0.0), "tests/donnees/rouge-64x48.mp4");
    assert_eq!(app.videos_en_question(), 0, "relatif : un texte");
    app.coller_un_texte(&board, (0.0, 0.0), video);
    assert_eq!(app.videos_en_question(), 1, "collée, la même question");
}
