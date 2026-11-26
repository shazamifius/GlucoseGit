//! **Les épreuves de chaque lecteur du système** (fiche 169, fiche 175) : les mêmes sous Linux
//! (GStreamer), sous Windows (Media Foundation ; ici, sous Wine : `outils/essai_wine.sh video`) et
//! sur le Mac (AVFoundation ; à la CI seulement),
//! sur une vraie vidéo de téléphone en miniature : H.264 et AAC dans un MP4, 64 × 48, rouge, dix
//! images en une seconde (`tests/donnees/rouge-64x48.mp4`, 3,8 Ko, faite par FFmpeg).
//!
//! **Sur le Mac, hors du harnais de `cargo test`** (fiche 175 § 9) : AVPlayer ne devient « prêt »
//! qu'en passant par la file principale, que Glucose sert par la boucle de sa fenêtre et le harnais
//! jamais. Ces épreuves y sont donc de simples fonctions, que `tests/videos_mac_suite.rs` passe sur
//! un autre fil, la file principale servie ([`TOUTES`]).

use super::joueur::JoueurVideo;
use super::{Fin, LecteurVideo, Lieu, PREPARATION};
use crate::renderer::gif::Toile;
use std::time::{Duration, Instant};

const ROUGE: &[u8] = include_bytes!("../../../tests/donnees/rouge-64x48.mp4");

/// **Le lecteur du système est-il là ?** Sous `nix develop` (où les greffons de GStreamer sont
/// portés) et sous Wine (`outils/essai_wine.sh` pose `GLUCOSE_EXIGER_LES_VIDEOS`), il doit
/// l'être : son absence est une faute. Ailleurs (une machine sans GStreamer, un Windows sans
/// décodeur H.264), l'épreuve n'a rien à éprouver.
pub(crate) fn present() -> bool {
    let la = super::lisible_ici();
    #[cfg(target_os = "linux")]
    let exige = super::gst::GREFFONS.is_some();
    #[cfg(not(target_os = "linux"))]
    let exige = false;
    let exige = exige || std::env::var_os("GLUCOSE_EXIGER_LES_VIDEOS").is_some();
    assert!(
        la || !exige,
        "le lecteur du système manque alors qu'il est exigé"
    );
    la
}

fn rouge(toile: &Toile) -> bool {
    let p = &toile.rgba[(toile.rgba.len() / 2) & !3..][..4];
    p[0] > 200 && p[1] < 60 && p[2] < 60 && p[3] == 255
}

#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_1_l_image_fixe_est_la_premiere_image() {
    if !present() {
        return;
    }
    let lecteur = LecteurVideo::ouvrir(Lieu::Octets(ROUGE.into()), false).expect("ouverte");
    let toile = lecteur.image_arretee().expect("la première image");
    assert_eq!((toile.largeur, toile.hauteur), (64, 48));
    assert_eq!(toile.rgba.len(), 64 * 48 * 4);
    assert!(rouge(&toile), "{:?}", &toile.rgba[..4]);
}

/// La vidéo scellée dans un document se lit dans sa tranche, là où elle est : des octets
/// étrangers avant et après ne la troublent pas.
#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_1_une_tranche_du_document_se_lit_sans_copie() {
    if !present() {
        return;
    }
    let dossier = std::env::temp_dir().join(format!("glucose-video-{}", std::process::id()));
    std::fs::create_dir_all(&dossier).unwrap();
    let chemin = dossier.join("document.glucose");
    let avant = vec![0xAB_u8; 4099];
    std::fs::write(
        &chemin,
        [&avant[..], ROUGE, b"la suite du document"].concat(),
    )
    .unwrap();
    let lieu = Lieu::Fichier {
        chemin,
        debut: avant.len() as u64,
        longueur: Some(ROUGE.len() as u64),
    };
    let toile = LecteurVideo::ouvrir(lieu, false)
        .and_then(|l| l.image_arretee())
        .expect("la tranche se lit");
    assert!(rouge(&toile));
    let _ = std::fs::remove_dir_all(dossier);
}

#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_1_la_video_joue_ses_images_jusqu_au_bout() {
    if !present() {
        return;
    }
    let lecteur = LecteurVideo::ouvrir(Lieu::Octets(ROUGE.into()), true).expect("ouverte");
    lecteur.jouer(true);
    let debut = Instant::now();
    let mut vues = Vec::new();
    loop {
        match lecteur.suivante(Duration::from_millis(100)) {
            Ok(Some(t)) => {
                assert!(rouge(&t));
                vues.push(Instant::now());
            }
            Ok(None) => {}
            Err(fin) => {
                assert_eq!(fin, Fin::AuBout);
                break;
            }
        }
        assert!(debut.elapsed() < Duration::from_secs(10), "jamais finie");
    }
    // Les images sortent à leur heure : dix en une seconde, de la première (0 s) à la dernière
    // (0,9 s) — pas d'un coup. La fin seule ne le prouve pas : le son tient aussi sa seconde.
    assert!(vues.len() >= 5, "{} images", vues.len());
    // Chaque image une fois : la vidéo en a dix, jamais une par regard du lecteur.
    assert!(vues.len() <= 10, "{} images rendues pour dix", vues.len());
    let etalees = vues[vues.len() - 1] - vues[0];
    assert!(etalees >= Duration::from_millis(500), "{etalees:?}");
}

#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_1_des_octets_qui_ne_sont_pas_une_video_n_ouvrent_rien() {
    if !present() {
        return;
    }
    let debut = Instant::now();
    let faux = [&ROUGE[..12], &[0x55; 2000][..]].concat();
    assert!(LecteurVideo::ouvrir(Lieu::Octets(faux.into()), false).is_none());
    assert!(debut.elapsed() < PREPARATION, "la faute se dit vite");
}

/// **La lecture automatique joue en boucle** (VIDEO-2) : au bout, elle revient au début et les
/// images reprennent.
#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_2_au_bout_la_video_revient_au_debut() {
    if !present() {
        return;
    }
    let mut lecteur = LecteurVideo::ouvrir(Lieu::Octets(ROUGE.into()), false).expect("ouverte");
    lecteur.jouer(true);
    let debut = Instant::now();
    let mut tours = 0;
    let mut images = 0;
    while tours < 2 {
        match lecteur.suivante(Duration::from_millis(100)) {
            Ok(Some(_)) => images += 1,
            Ok(None) => {}
            Err(Fin::AuBout) => {
                tours += 1;
                assert!(lecteur.revenir_au_debut(), "le système refuse de revenir");
            }
            Err(Fin::Faute) => panic!("faute"),
        }
        assert!(debut.elapsed() < Duration::from_secs(10), "jamais revenue");
    }
    // Deux tours de dix images : plus d'un tour en a été vu.
    assert!(images > 12, "{images} images en deux tours");
}

/// **La lecture automatique joue en boucle** (VIDEO-2, fiche 170) : sur la vraie vidéo de
/// téléphone en miniature (dix images en une seconde), le joueur en boucle ne finit jamais ; l'autre
/// finit au bout.
fn jouer_pendant(en_boucle: bool, duree: Duration) -> Option<(usize, bool)> {
    if !present() {
        return None;
    }
    let chemin = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/donnees/rouge-64x48.mp4");
    let mut joueur = JoueurVideo::lancer(chemin.to_string(), None, None, en_boucle);
    // L'ouverture a son propre temps (PREPARATION) : sur le Windows de la CI, sans carte son, un
    // moteur s'ouvre en près de 6 s (fiche 175 § 11). La durée se compte de la première image.
    let (ouverture, mut images) = (Instant::now(), 0);
    while images == 0 && ouverture.elapsed() < PREPARATION && !joueur.fini() {
        images += usize::from(joueur.a_poser().is_some());
        std::thread::sleep(Duration::from_millis(5));
    }
    let debut = Instant::now();
    while debut.elapsed() < duree && !joueur.fini() {
        images += usize::from(joueur.a_poser().is_some());
        std::thread::sleep(Duration::from_millis(5));
    }
    Some((images, joueur.fini()))
}

#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_2_en_boucle_le_joueur_repart_au_bout() {
    let Some((images, fini)) = jouer_pendant(true, Duration::from_millis(2600)) else {
        return;
    };
    assert!(!fini, "la boucle a fini");
    assert!(
        images > 12,
        "{images} images en plus de deux tours d'une seconde"
    );
}

#[cfg_attr(not(target_os = "macos"), test)]
pub fn video_2_sans_boucle_le_joueur_finit_au_bout() {
    let Some((_, fini)) = jouer_pendant(false, Duration::from_secs(5)) else {
        return;
    };
    assert!(fini, "jamais finie");
}

/// **Toutes, pour la suite du Mac** : leur nom et elles.
#[cfg(target_os = "macos")]
pub const TOUTES: [(&str, fn()); 7] = [
    (
        "video_1_l_image_fixe_est_la_premiere_image",
        video_1_l_image_fixe_est_la_premiere_image,
    ),
    (
        "video_1_une_tranche_du_document_se_lit_sans_copie",
        video_1_une_tranche_du_document_se_lit_sans_copie,
    ),
    (
        "video_1_la_video_joue_ses_images_jusqu_au_bout",
        video_1_la_video_joue_ses_images_jusqu_au_bout,
    ),
    (
        "video_1_des_octets_qui_ne_sont_pas_une_video_n_ouvrent_rien",
        video_1_des_octets_qui_ne_sont_pas_une_video_n_ouvrent_rien,
    ),
    (
        "video_2_au_bout_la_video_revient_au_debut",
        video_2_au_bout_la_video_revient_au_debut,
    ),
    (
        "video_2_en_boucle_le_joueur_repart_au_bout",
        video_2_en_boucle_le_joueur_repart_au_bout,
    ),
    (
        "video_2_sans_boucle_le_joueur_finit_au_bout",
        video_2_sans_boucle_le_joueur_finit_au_bout,
    ),
];
