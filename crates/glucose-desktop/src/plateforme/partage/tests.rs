//! Ce que garantit le partage vers Glucose (PARTAGE-1).

use super::*;
use std::io::Cursor;

/// Un PNG d'un pixel : de vrais octets d'image.
fn png() -> Vec<u8> {
    let mut octets = Vec::new();
    image::RgbaImage::new(1, 1)
        .write_to(&mut Cursor::new(&mut octets), image::ImageFormat::Png)
        .expect("un pixel s'encode");
    octets
}

/// **Un fichier entier se lit entier**, sous un nom qui ne décide de rien.
#[test]
fn test_a_whole_shared_file_is_read_whole() {
    let recu = lire(Cursor::new(png()), 0, None).expect("le fichier se lit");
    assert_eq!(recu.octets, png());
    assert_eq!(recu.nom, NOM);
}

/// **Un morceau de fichier se lit à sa place** : un fournisseur peut ne donner qu'un morceau
/// d'un fichier plus grand (`AssetFileDescriptor`), et lire depuis le début lirait autre chose.
#[test]
fn test_a_section_is_read_at_its_offset_and_length() {
    let mut fichier = b"ENTETE".to_vec();
    fichier.extend(png());
    fichier.extend(b"QUEUE");
    let recu = lire(Cursor::new(fichier), 6, Some(png().len() as u64)).expect("le morceau");
    assert_eq!(
        recu.octets,
        png(),
        "exactement le morceau, ni l'en-tête ni la queue"
    );
}

/// **Un fichier vide n'est pas une image partagée**, et ne pose pas une tuile vide.
#[test]
fn test_an_empty_shared_file_is_nothing() {
    assert_eq!(lire(Cursor::new(Vec::new()), 0, None), None);
}

/// **Au-delà de la borne, rien** : un fournisseur peut annoncer autant qu'il veut.
#[test]
fn test_a_file_beyond_the_bound_is_refused() {
    struct Infini;
    impl Read for Infini {
        fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
            tampon.fill(7);
            Ok(tampon.len())
        }
    }
    impl Seek for Infini {
        fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
            Ok(0)
        }
    }
    assert_eq!(lire(Infini, 0, None), None);
}

/// **Les adresses d'un texte** : seules, entourées d'une phrase ou de ponctuation, sans
/// doublon, et rien qui ne soit `http` ou `https`.
#[test]
fn test_the_addresses_of_a_shared_text() {
    assert_eq!(adresses("https://pin.it/4xYz"), vec!["https://pin.it/4xYz"]);
    assert_eq!(
        adresses("Regarde cette épingle : https://pin.it/4xYz."),
        vec!["https://pin.it/4xYz"]
    );
    assert_eq!(
        adresses("(https://a.fr/x) « http://b.fr/y » https://a.fr/x"),
        vec!["https://a.fr/x", "http://b.fr/y"]
    );
    assert!(adresses("javascript:alert(1) file:///etc/passwd bonjour").is_empty());
}

/// **Des images se posent**, et le texte qui les accompagne n'est qu'une légende.
#[test]
fn test_images_are_posed_and_their_caption_ignored() {
    let recu = lire(Cursor::new(png()), 0, None).expect("l'image");
    let route = router(Partage {
        recus: vec![recu.clone()],
        illisibles: 1,
        texte: Some("https://pin.it/4xYz".into()),
    });
    assert_eq!(
        route,
        Route::Poser(Moisson {
            recus: vec![recu],
            illisibles: 1,
            ..Moisson::default()
        })
    );
}

/// **Sans image, les adresses vont chercher la leur**, et se posent en liens si elles ne la
/// trouvent pas.
#[test]
fn test_a_link_alone_goes_to_fetch_its_image() {
    let route = router(Partage {
        texte: Some("https://pin.it/4xYz".into()),
        ..Partage::default()
    });
    let liens = vec!["https://pin.it/4xYz".to_string()];
    assert_eq!(
        route,
        Route::Rapatrier(
            liens.clone(),
            Moisson {
                liens,
                ..Moisson::default()
            }
        )
    );
}

/// **Un texte nu le dit** — un partage sans effet visible serait un bouton qui ment.
#[test]
fn test_a_bare_text_says_why_nothing_is_posed() {
    let Route::Poser(moisson) = router(Partage {
        texte: Some("bonjour".into()),
        ..Partage::default()
    }) else {
        panic!("rien à rapatrier");
    };
    assert_eq!(moisson.echec.as_deref(), Some(RIEN));
    // Un partage dont tout est illisible ne dit pas « aucune image » : le compte-rendu
    // dira que le fichier n'a pas pu être posé.
    let Route::Poser(moisson) = router(Partage {
        illisibles: 1,
        ..Partage::default()
    }) else {
        panic!("rien à rapatrier");
    };
    assert_eq!(moisson.echec, None);
    assert!(!moisson.est_vide(), "un illisible se compte");
}

/// **La boîte aux lettres** : un partage arrivé avant la fenêtre l'attend, part quand elle
/// se branche, réveille la boucle ; débranchée, la boîte attend la suivante.
///
/// Une seule épreuve pour toute la boîte : elle est unique dans le processus, et deux
/// épreuves parallèles s'y croiseraient.
#[test]
fn test_the_mailbox_waits_for_the_window() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let image = || Partage {
        recus: vec![lire(Cursor::new(png()), 0, None).expect("l'image")],
        ..Partage::default()
    };
    recevoir(image());
    let (vers, recus) = std::sync::mpsc::channel();
    let reveils = Arc::new(AtomicUsize::new(0));
    let compte = reveils.clone();
    let reveil: Reveil = Arc::new(move || {
        compte.fetch_add(1, Ordering::Relaxed);
    });
    let branchement = brancher(vers, reveil);
    assert_eq!(
        recus.try_iter().count(),
        1,
        "le partage qui attendait est parti"
    );
    assert_eq!(
        reveils.load(Ordering::Relaxed),
        1,
        "et la boucle est réveillée"
    );
    recevoir(image());
    assert_eq!(
        recus.try_iter().count(),
        1,
        "branchée, la boîte livre aussitôt"
    );
    drop(branchement);
    recevoir(image());
    assert_eq!(
        recus.try_iter().count(),
        0,
        "débranchée, rien ne part vers l'ancien pont"
    );
    let (vers, recus) = std::sync::mpsc::channel();
    let _branchement = brancher(vers, Arc::new(|| {}));
    assert_eq!(
        recus.try_iter().count(),
        1,
        "le nouveau pont reçoit ce qui attendait"
    );
}

// ── De bout en bout : un partage, tel que l'application le pose ─────────────────────────

/// Glucose sans document : le canevas vide, l'histoire vide.
fn application() -> crate::app::GlucoseApp {
    let mut app = crate::app::GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Ce qu'un partage devient dans l'application : le chemin du dépôt, par la route qu'il prend.
fn partager(app: &mut crate::app::GlucoseApp, partage: Partage) {
    let Route::Poser(moisson) = router(partage) else {
        panic!("ce partage se pose sans réseau");
    };
    app.recevoir_le_depot(Depot::Pose {
        numero: None,
        moisson,
    });
}

fn message(app: &crate::app::GlucoseApp) -> Option<String> {
    app.ui.current_toast.as_ref().map(|t| t.message.clone())
}

/// **Deux images partagées se posent en un geste**, et l'illisible se compte : un seul
/// `Ctrl+Z` les retire, un seul message le dit.
#[test]
fn test_shared_images_are_one_gesture_and_the_unreadable_is_counted() {
    let mut app = application();
    let image = || lire(Cursor::new(png()), 0, None).expect("l'image");
    partager(
        &mut app,
        Partage {
            recus: vec![image(), image()],
            illisibles: 1,
            texte: None,
        },
    );
    let posees =
        |app: &crate::app::GlucoseApp| app.store.active_board().map_or(0, |b| b.images.len());
    assert_eq!(posees(&app), 2, "les deux images sont sur le canevas");
    assert_eq!(
        message(&app).as_deref(),
        Some("2 éléments posés, 1 illisible")
    );
    app.store.undo();
    assert_eq!(posees(&app), 0, "un seul geste les retire toutes");
}

/// **Un texte nu dit pourquoi rien ne se pose.**
#[test]
fn test_a_bare_shared_text_says_so() {
    let mut app = application();
    partager(
        &mut app,
        Partage {
            texte: Some("bonjour".into()),
            ..Partage::default()
        },
    );
    assert_eq!(message(&app).as_deref(), Some(RIEN));
}

/// **Un seul fichier illisible le dit au singulier.**
#[test]
fn test_a_single_unreadable_file_says_so() {
    let mut app = application();
    partager(
        &mut app,
        Partage {
            illisibles: 1,
            ..Partage::default()
        },
    );
    assert_eq!(
        message(&app).as_deref(),
        Some("Le fichier n'a pas pu être posé")
    );
}

/// **« Ajouter des images… » ouvre le sélecteur branché**, et dit quand il n'y en a pas.
#[test]
fn test_the_photo_picker_opens_when_plugged() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let ouvertures = Arc::new(AtomicUsize::new(0));
    let compte = ouvertures.clone();
    installer_le_selecteur(Box::new(move || {
        compte.fetch_add(1, Ordering::Relaxed);
    }));
    assert!(choisir_des_images(), "un sélecteur est branché");
    assert_eq!(
        ouvertures.load(Ordering::Relaxed),
        1,
        "il s'est ouvert, une fois"
    );
}
