//! Le décodage sur place donne les mêmes octets que le chemin d'avant, au bit près, pour chaque
//! couleur qu'il étale — et laisse au chemin d'avant celles qu'il n'étale pas.

use super::*;
use image::{ExtendedColorType, ImageEncoder};

/// Une image de `l × h` aux pixels tous différents, de `k` canaux, alpha varié s'il y en a un.
fn pixels(l: u32, h: u32, k: usize) -> Vec<u8> {
    (0..l * h)
        .flat_map(|i| (0..k).map(move |c| ((i * 37 + c as u32 * 91 + i / 7) % 256) as u8))
        .collect()
}

fn png(couleur: ExtendedColorType, (l, h): (u32, u32), brut: &[u8]) -> Vec<u8> {
    let mut sortie = Vec::new();
    image::codecs::png::PngEncoder::new(&mut sortie)
        .write_image(brut, l, h, couleur)
        .expect("png encodé");
    sortie
}

fn jpeg(couleur: ExtendedColorType, (l, h): (u32, u32), brut: &[u8]) -> Vec<u8> {
    let mut sortie = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut sortie, 90)
        .write_image(brut, l, h, couleur)
        .expect("jpeg encodé");
    sortie
}

fn decodeur(octets: &[u8]) -> impl ImageDecoder + '_ {
    image::ImageReader::new(std::io::Cursor::new(octets))
        .with_guessed_format()
        .expect("format reconnu")
        .into_decoder()
        .expect("un décodeur")
}

/// Le chemin d'avant : l'image décodée à part, sa copie en RGBA, puis la pyramide.
fn par_le_chemin_d_avant(octets: &[u8]) -> Pyramide {
    let rgba = image::load_from_memory(octets).expect("décodée").to_rgba8();
    let (l, h) = rgba.dimensions();
    Pyramide::depuis_rgba(l, h, rgba.as_raw()).expect("la pyramide")
}

/// Chaque niveau, octet pour octet.
fn niveaux(p: &Pyramide) -> Vec<Vec<u8>> {
    (0..p.niveaux_construits())
        .map(|rang| p.niveau(1 << rang).expect("tenu").data().to_vec())
        .collect()
}

/// **MEMOIRE-3 — au bit près, pour chaque couleur qui s'étale** : gris, gris et alpha, RGB et
/// RGBA en PNG — l'alpha varié éprouve la prémultiplication —, RGB et gris en JPEG. Une taille
/// impaire éprouve les bords de la pyramide.
#[test]
fn test_memoire_3_sur_place_les_memes_octets_que_le_chemin_d_avant() {
    let (l, h) = (37, 23);
    let cas = [
        (
            "png gris",
            png(ExtendedColorType::L8, (l, h), &pixels(l, h, 1)),
        ),
        (
            "png gris alpha",
            png(ExtendedColorType::La8, (l, h), &pixels(l, h, 2)),
        ),
        (
            "png rgb",
            png(ExtendedColorType::Rgb8, (l, h), &pixels(l, h, 3)),
        ),
        (
            "png rgba",
            png(ExtendedColorType::Rgba8, (l, h), &pixels(l, h, 4)),
        ),
        (
            "jpeg rgb",
            jpeg(ExtendedColorType::Rgb8, (l, h), &pixels(l, h, 3)),
        ),
        (
            "jpeg gris",
            jpeg(ExtendedColorType::L8, (l, h), &pixels(l, h, 1)),
        ),
    ];
    for (nom, octets) in cas {
        let d = decodeur(&octets);
        assert!(
            Pyramide::octets_par_pixel(d.color_type()).is_some(),
            "{nom} s'étale sur place"
        );
        let sur_place = Pyramide::depuis_le_decodeur(d).expect("décodée sur place");
        assert_eq!(
            niveaux(&sur_place),
            niveaux(&par_le_chemin_d_avant(&octets)),
            "{nom}"
        );
    }
}

/// **Seize bits ne s'étalent pas sur place** : le décodeur les dit tels, et l'atelier garde
/// pour eux le chemin d'avant.
#[test]
fn test_memoire_3_seize_bits_gardent_le_chemin_d_avant() {
    let brut: Vec<u8> = pixels(5, 3, 8);
    let octets = png(ExtendedColorType::Rgba16, (5, 3), &brut);
    let d = decodeur(&octets);
    assert_eq!(Pyramide::octets_par_pixel(d.color_type()), None);
    assert!(Pyramide::depuis_le_decodeur(d).is_none());
}
