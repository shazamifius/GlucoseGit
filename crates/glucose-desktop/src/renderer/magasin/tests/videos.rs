//! **Les vidéos en lecture automatique, dans le magasin** (VIDEO-2, fiche 170) : réclamée à
//! l'écran, elle joue seule — une fois sa première image tenue ; une qui ne se lit pas ici ne se
//! relance pas à chaque image.

use super::*;
use glucose_core::types::BoardImage;
use std::time::{Duration, Instant};

/// Une « vidéo » dont l'image fixe se décode (la photo témoin), puis dont le fichier disparaît :
/// son joueur ne trouvera rien à lire, sans dépendre de GStreamer.
fn video_partie(magasin: &mut Magasin, nom: &str) -> BoardImage {
    let dossier = std::env::temp_dir().join("glucose-magasin-tests");
    let chemin = dossier.join(nom);
    std::fs::copy(photo_temoin("video-temoin.png"), &chemin).expect("copie");
    let src = chemin.to_string_lossy().to_string();
    let mut img = BoardImage::new("v", 0.0, 0.0, 160.0, 90.0);
    img.src = Some(src.clone());
    img.is_video = true;
    magasin.reclamer_le_noeud(&img);
    assert!(attendre(magasin, &src), "son image fixe jamais tenue");
    std::fs::remove_file(&chemin).expect("le fichier part");
    img
}

#[test]
fn video_2_la_lecture_auto_joue_des_que_sa_premiere_image_est_tenue() {
    let mut magasin = Magasin::nouveau();
    let mut jamais_vue = BoardImage::new("w", 0.0, 0.0, 160.0, 90.0);
    jamais_vue.src = Some("/nulle/part/glucose-video-2.mp4".into());
    jamais_vue.is_video = true;
    jamais_vue.video.lecture_auto = true;
    magasin.reclamer_le_noeud(&jamais_vue);
    assert!(
        magasin
            .etat_video("/nulle/part/glucose-video-2.mp4")
            .is_none(),
        "sans première image, rien ne se lance"
    );
    let mut img = video_partie(&mut magasin, "video-2-auto.mp4");
    let src = img.src.clone().expect("src");
    magasin.reclamer_le_noeud(&img);
    assert!(
        magasin.etat_video(&src).is_none(),
        "au bouton ▶, elle attend"
    );
    img.video.lecture_auto = true;
    magasin.reclamer_le_noeud(&img);
    assert!(
        magasin.etat_video(&src).is_some(),
        "en lecture automatique, elle joue"
    );
}

#[test]
fn video_2_une_video_qui_ne_se_lit_pas_ici_ne_se_relance_pas() {
    let mut magasin = Magasin::nouveau();
    let mut img = video_partie(&mut magasin, "video-2-muette.mp4");
    img.video.lecture_auto = true;
    let src = img.src.clone().expect("src");
    magasin.reclamer_le_noeud(&img);
    assert!(magasin.etat_video(&src).is_some(), "lancée");
    // Son fil ne trouve rien à lire et finit ; chaque image d'après la réclame encore.
    let depart = Instant::now();
    loop {
        magasin.recolter();
        magasin.reclamer_le_noeud(&img);
        if magasin.etat_video(&src).is_none() {
            break;
        }
        assert!(
            depart.elapsed() < Duration::from_secs(10),
            "relancée sans fin"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    for _ in 0..3 {
        magasin.recolter();
        magasin.reclamer_le_noeud(&img);
        assert!(
            magasin.etat_video(&src).is_none(),
            "muette ici, elle reste arrêtée"
        );
    }
}
