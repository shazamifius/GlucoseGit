//! **Les GIF qui bougent, dans le magasin** (GIF-2, fiche 159) : un GIF vu change de pyramide à
//! l'heure de chaque toile ; hors de l'écran, son joueur est lâché.

use super::*;
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
    // Le rendu, image après image : réclamer, récolter — trois toiles de plus en moins de 5 s.
    let depart = Instant::now();
    while generation(&magasin, &src) < premiere + 3 && depart.elapsed() < Duration::from_secs(5) {
        magasin.ouvrir();
        let _ = magasin.pyramide(&src);
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
