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

/// Une toile unie de 2 × 2, en RGBA.
fn unie(c: [u8; 3]) -> Vec<u8> {
    [c[0], c[1], c[2], 255].repeat(4)
}

/// **Un PNG animé** (fiche 159 § 9) : deux toiles de 50 ms, sans fin.
#[test]
fn gif_4_un_png_anime_se_lit() {
    let mut octets = Vec::new();
    {
        let mut e = png::Encoder::new(&mut octets, 2, 2);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.set_animated(2, 0).expect("animé");
        e.set_frame_delay(50, 1000).expect("son délai");
        let mut w = e.write_header().expect("l'en-tête");
        w.write_image_data(&unie(ROUGE)).expect("la première");
        w.write_image_data(&unie(BLEU)).expect("la seconde");
        w.finish().expect("la fin");
    }
    assert_eq!(signature::animee(&octets), Ok(true));
    let mut l = Lecteur::nouveau(octets.into()).expect("lu");
    let a = l.suivante().expect("la première");
    assert_eq!(
        (&a.rgba[..4], a.delai),
        (&[255, 0, 0, 255][..], Duration::from_millis(50))
    );
    assert_eq!(
        &l.suivante().expect("la seconde").rgba[..4],
        &[0, 0, 255, 255]
    );
    assert_eq!(
        &l.suivante().expect("sans fin").rgba[..4],
        &[255, 0, 0, 255]
    );
}

/// **Un WebP animé** (fiche 159 § 9), assemblé à la main : `image` n'en écrit pas — deux toiles
/// sans perte de 2 × 2, 80 ms chacune, sans fin.
#[test]
fn gif_4_un_webp_anime_se_lit() {
    let morceau = |nom: &[u8], donnees: &[u8]| {
        let mut m = nom.to_vec();
        m.extend(u32::try_from(donnees.len()).expect("court").to_le_bytes());
        m.extend(donnees);
        if donnees.len() % 2 == 1 {
            m.push(0);
        }
        m
    };
    let toile = |c: [u8; 3]| {
        let mut fixe = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut fixe)
            .encode(&unie(c), 2, 2, image::ExtendedColorType::Rgba8)
            .expect("une toile");
        // Après `RIFF`, sa taille et `WEBP` : le morceau `VP8L`, tel quel.
        let mut anmf = vec![0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 80, 0, 0, 0];
        anmf.extend(&fixe[12..]);
        morceau(b"ANMF", &anmf)
    };
    let mut corps = b"WEBP".to_vec();
    corps.extend(morceau(b"VP8X", &[0x12, 0, 0, 0, 1, 0, 0, 1, 0, 0]));
    corps.extend(morceau(b"ANIM", &[0, 0, 0, 0, 0, 0]));
    corps.extend(toile(ROUGE));
    corps.extend(toile(BLEU));
    let octets = morceau(b"RIFF", &corps);
    assert_eq!(signature::animee(&octets), Ok(true));
    // `image-webp` compose chaque toile sur le fond par un mélange qui arrondit d'un cran.
    let proche = |t: &Toile, c: [u8; 4]| t.rgba[..4].iter().zip(c).all(|(a, b)| a.abs_diff(b) <= 1);
    let mut l = Lecteur::nouveau(octets.into()).expect("lu");
    let a = l.suivante().expect("la première");
    assert!(proche(&a, [255, 0, 0, 255]), "rouge : {:?}", &a.rgba[..4]);
    assert_eq!(a.delai, Duration::from_millis(80));
    assert!(proche(&l.suivante().expect("la seconde"), [0, 0, 255, 255]));
    assert!(proche(&l.suivante().expect("sans fin"), [255, 0, 0, 255]));
}
