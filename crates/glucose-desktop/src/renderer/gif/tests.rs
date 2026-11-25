use super::*;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};

/// Un GIF de 2 × 2 : une toile par couleur, chacune avec son délai en millisecondes.
fn gif(toiles: &[([u8; 3], u32)], repeat: Repeat) -> Arc<[u8]> {
    let mut octets = Vec::new();
    {
        let mut encodeur = GifEncoder::new(&mut octets);
        encodeur.set_repeat(repeat).expect("les tours");
        for &([r, g, b], ms) in toiles {
            let image = RgbaImage::from_pixel(2, 2, Rgba([r, g, b, 255]));
            let delai = Delay::from_numer_denom_ms(ms, 1);
            encodeur
                .encode_frame(Frame::from_parts(image, 0, 0, delai))
                .expect("une toile");
        }
    }
    octets.into()
}

const ROUGE: [u8; 3] = [255, 0, 0];
const BLEU: [u8; 3] = [0, 0, 255];

#[test]
fn gif_1_les_toiles_se_suivent_avec_leurs_delais() {
    let mut l = Lecteur::nouveau(gif(&[(ROUGE, 50), (BLEU, 200)], Repeat::Infinite)).expect("lu");
    let a = l.suivante().expect("la première");
    assert_eq!((a.largeur, a.hauteur), (2, 2));
    assert_eq!(&a.rgba[..4], &[255, 0, 0, 255]);
    assert_eq!(a.delai, Duration::from_millis(50));
    let b = l.suivante().expect("la seconde");
    assert_eq!(&b.rgba[..4], &[0, 0, 255, 255]);
    assert_eq!(b.delai, Duration::from_millis(200));
    // Sans fin : le tour suivant reprend du début.
    assert_eq!(
        &l.suivante().expect("un tour de plus").rgba[..4],
        &[255, 0, 0, 255]
    );
}

#[test]
fn gif_1_dix_millisecondes_ou_moins_valent_cent() {
    let mut l = Lecteur::nouveau(gif(&[(ROUGE, 10), (BLEU, 20)], Repeat::Infinite)).expect("lu");
    assert_eq!(l.suivante().expect("1").delai, Duration::from_millis(100));
    assert_eq!(l.suivante().expect("2").delai, Duration::from_millis(20));
}

#[test]
fn gif_1_les_tours_demandes_se_jouent_puis_s_arretent() {
    let mut l = Lecteur::nouveau(gif(&[(ROUGE, 50), (BLEU, 50)], Repeat::Finite(2))).expect("lu");
    let rangs: Vec<u8> = std::iter::from_fn(|| l.suivante())
        .map(|t| t.rgba[2])
        .take(10)
        .collect();
    assert_eq!(rangs, [0, 255, 0, 255]);
}

#[test]
fn gif_1_une_seule_image_ne_se_joue_pas() {
    let mut l = Lecteur::nouveau(gif(&[(ROUGE, 50)], Repeat::Infinite)).expect("lu");
    assert!(l.suivante().is_some(), "la seule toile se lit");
    assert!(l.suivante().is_none(), "et rien ne la rejoue");
}

#[test]
fn gif_1_ce_qui_n_est_pas_un_gif_ne_se_lit_pas() {
    assert!(Lecteur::nouveau(Arc::from(&b"PNG pas un gif"[..])).is_none());
}
