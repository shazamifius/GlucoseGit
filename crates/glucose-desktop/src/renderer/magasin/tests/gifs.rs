//! **Les GIF qui bougent, dans le magasin** (GIF-2, fiche 159) : un GIF vu change de pyramide à
//! l'heure de chaque toile ; hors de l'écran, son joueur est lâché.

use super::*;
use crate::renderer::photo::{Forme, Pyramide};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};
use std::time::{Duration, Instant};

/// Un GIF de 4 × 4 sur le disque, deux toiles de 20 ms, sans fin.
fn gif_temoin(nom: &str) -> String {
    let dossier = std::env::temp_dir().join("glucose-magasin-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join(nom);
    let mut octets = Vec::new();
    {
        let mut encodeur = GifEncoder::new(&mut octets);
        encodeur.set_repeat(Repeat::Infinite).expect("les tours");
        for couleur in [[255, 0, 0, 255], [0, 0, 255, 255]] {
            let toile = RgbaImage::from_pixel(4, 4, Rgba(couleur));
            let delai = Delay::from_numer_denom_ms(20, 1);
            encodeur
                .encode_frame(Frame::from_parts(toile, 0, 0, delai))
                .expect("une toile");
        }
    }
    std::fs::write(&chemin, octets).expect("écriture du GIF témoin");
    chemin.to_string_lossy().to_string()
}

fn generation(magasin: &Magasin, src: &str) -> u64 {
    magasin.cache.get(src).expect("dans le cache").generation()
}

#[test]
fn gif_2_un_gif_vu_change_de_pyramide_a_l_heure_de_ses_toiles() {
    let src = gif_temoin("magasin-gif-vu.gif");
    let mut magasin = Magasin::nouveau();
    let depart = Instant::now();
    while magasin.pyramide(&src).is_none() && depart.elapsed() < Duration::from_secs(30) {
        magasin.ouvrir();
        magasin.recolter();
        std::thread::yield_now();
    }
    assert!(magasin.gifs.contains(&src), "sa signature est lue");
    let premiere = generation(&magasin, &src);
    // Une vignette faite sur la première toile, celle que la voie processeur repose.
    let forme = Forme::posee(0.0, 0.0, 4.0, 4.0);
    let pyr = Pyramide::nouvelle(tiny_skia::Pixmap::new(4, 4).expect("une toile"));
    for _ in 0..3 {
        magasin.vignettes.ouvrir();
        let _ = magasin.vignettes.pour("n", &src, forme, 16.0);
        (magasin.vignettes).avancer_le_chantier(Duration::from_secs(3600), |_| Some(&pyr));
    }
    assert!(
        magasin.vignettes.a_prete("n", forme),
        "sa vignette est faite"
    );
    // Le rendu, image après image : réclamer, récolter — trois toiles de plus en moins de 5 s.
    let depart = Instant::now();
    while generation(&magasin, &src) < premiere + 3 && depart.elapsed() < Duration::from_secs(5) {
        magasin.ouvrir();
        let _ = magasin.pyramide(&src);
        let _ = magasin.vignettes.pour("n", &src, forme, 16.0);
        magasin.recolter();
        // Ce que le réveil attendrait : l'heure de la toile suivante, jamais dépassée de beaucoup.
        if let Some(heure) = magasin.prochaine_toile() {
            std::thread::sleep(heure.saturating_duration_since(Instant::now()));
        } else {
            std::thread::yield_now();
        }
    }
    assert!(
        generation(&magasin, &src) >= premiere + 3,
        "un GIF à l'écran doit changer de pyramide à chaque toile"
    );
    assert_eq!(magasin.joueurs.len(), 1, "un joueur, le sien");
    assert!(
        !magasin.vignettes.a_prete("n", forme),
        "une toile neuve oublie la vignette de la précédente"
    );

    // Deux images qui ne le réclament plus : son joueur est lâché, et plus rien ne réveille.
    for _ in 0..2 {
        magasin.ouvrir();
        magasin.recolter();
    }
    assert!(
        magasin.joueurs.is_empty(),
        "hors de l'écran, plus de joueur"
    );
    assert_eq!(magasin.prochaine_toile(), None);
}

/// **GIF-3** : moins de mouvement demandé, un GIF à l'écran n'a pas de joueur — et le sien
/// s'arrête si le réglage arrive pendant qu'il joue ; il repart quand le système n'en demande plus.
#[test]
fn gif_3_moins_de_mouvement_fige_les_gifs() {
    let src = gif_temoin("magasin-gif-fige.gif");
    let mut magasin = Magasin::nouveau();
    let depart = Instant::now();
    while magasin.pyramide(&src).is_none() && depart.elapsed() < Duration::from_secs(30) {
        magasin.ouvrir();
        magasin.recolter();
        std::thread::yield_now();
    }
    magasin.ouvrir();
    let _ = magasin.pyramide(&src);
    magasin.recolter();
    assert_eq!(magasin.joueurs.len(), 1, "à l'écran, il joue");

    magasin.reduire_le_mouvement(true);
    let figee = generation(&magasin, &src);
    for _ in 0..20 {
        magasin.ouvrir();
        let _ = magasin.pyramide(&src);
        magasin.recolter();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        magasin.joueurs.is_empty(),
        "moins de mouvement : plus de joueur"
    );
    assert_eq!(generation(&magasin, &src), figee, "il reste sur sa toile");
    assert_eq!(magasin.prochaine_toile(), None, "et rien ne réveille");

    magasin.reduire_le_mouvement(false);
    magasin.ouvrir();
    let _ = magasin.pyramide(&src);
    magasin.recolter();
    assert_eq!(magasin.joueurs.len(), 1, "il repart");
}
