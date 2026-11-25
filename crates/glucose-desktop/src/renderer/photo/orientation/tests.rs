//! Les huit orientations, chacune comme `image` la fait — chaque pixel d'une image asymétrique à sa place.

use super::*;

const TOUTES: [Orientation; 8] = [
    Orientation::NoTransforms,
    Orientation::FlipHorizontal,
    Orientation::Rotate180,
    Orientation::FlipVertical,
    Orientation::Rotate90FlipH,
    Orientation::Rotate90,
    Orientation::Rotate270FlipH,
    Orientation::Rotate270,
];

#[test]
fn test_chaque_orientation_tourne_comme_image_la_tourne() {
    // 3 × 2, chaque pixel différent : un seul pixel mal placé se voit.
    let (l, h) = (3u32, 2u32);
    let octets: Vec<u8> = (0..l * h)
        .flat_map(|i| {
            let v = 40 * (i as u8 + 1);
            [v, 255 - v, v / 2, 255]
        })
        .collect();
    for orientation in TOUTES {
        let niveau = Niveau::depuis_octets(l, h, &octets).expect("niveau");
        let droit = niveau.oriente(orientation).expect("tourné");
        let mut attendu = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(l, h, octets.clone()).expect("image"),
        );
        attendu.apply_orientation(orientation);
        assert_eq!(
            (droit.largeur, droit.hauteur),
            (attendu.width(), attendu.height()),
            "{orientation:?}"
        );
        assert_eq!(
            droit.vue().expect("tenu").data(),
            attendu.to_rgba8().as_raw().as_slice(),
            "{orientation:?}"
        );
    }
}

/// 20 × 10, chaque pixel différent.
fn asymetrique() -> image::DynamicImage {
    let (l, h) = (20u32, 10u32);
    let octets: Vec<u8> = (0..l * h)
        .flat_map(|i| {
            [
                (i % 256) as u8,
                (i / 256) as u8 + 7,
                255 - (i % 251) as u8,
                255,
            ]
        })
        .collect();
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(l, h, octets).expect("image"))
}

fn tournee(mut img: image::DynamicImage, orientation: Orientation) -> image::DynamicImage {
    img.apply_orientation(orientation);
    img
}

/// **Le recadrage d'un nœud redressé montre la même région de la photo** (EXIF-1, fiche 165
/// § 3.3) : ce qu'il gardait des pixels couchés, tourné, est ce qu'il garde des pixels droits —
/// la convention du noyau (`Recadrage::oriente`) tenue contre celle d'`image`.
#[test]
fn test_le_recadrage_oriente_garde_la_meme_region() {
    use glucose_core::types::Recadrage;
    let couchee = asymetrique();
    let cadre = Recadrage::depuis_les_marges(0.1, 0.2, 0.25, 0.3);
    let region = |img: &image::DynamicImage, cadre: Recadrage| {
        let (g, t, d, b) = cadre.marges();
        let (l, h) = (f64::from(img.width()), f64::from(img.height()));
        let px = |v: f64| v.round() as u32;
        img.crop_imm(
            px(g * l),
            px(t * h),
            px(l * (1.0 - g - d)),
            px(h * (1.0 - t - b)),
        )
    };
    for orientation in TOUTES {
        let attendue = tournee(region(&couchee, cadre), orientation);
        let droite = tournee(couchee.clone(), orientation);
        let vue = region(&droite, cadre.oriente(orientation.to_exif()));
        assert_eq!(vue.to_rgba8(), attendue.to_rgba8(), "{orientation:?}");
    }
}

/// **Le nœud qu'il avait redressé à la main revient à zéro, et c'est bien le bon** : de ses sept
/// poses tournées d'un quart de tour ou retournées, une seule montrait la photo couchée debout —
/// celle que le noyau ramène à zéro (le nœud jamais touché y reste, lui, et montre enfin la photo
/// debout).
#[test]
fn test_le_noeud_redresse_a_la_main_est_celui_qui_montrait_la_photo_debout() {
    use glucose_core::types::BoardImage;
    let couchee = asymetrique();
    for orientation in TOUTES.into_iter().filter(|o| matches!(o.to_exif(), 5..=8)) {
        let debout = tournee(couchee.clone(), orientation).to_rgba8();
        let mut ramenes = 0;
        for quarts in 0..4u8 {
            for miroir in [false, true].into_iter().filter(|&m| quarts > 0 || m) {
                let mut noeud = BoardImage::new("i", 0.0, 0.0, 20.0, 10.0);
                noeud.rotation = f64::from(quarts) * std::f64::consts::FRAC_PI_2;
                noeud.miroir = miroir;
                assert!(noeud.redresser(orientation.to_exif(), (10, 20)));
                if (noeud.rotation, noeud.miroir) != (0.0, false) {
                    continue;
                }
                ramenes += 1;
                // Ce qu'il montrait : le miroir d'abord, puis les quarts dans le sens des aiguilles.
                let mut vue = if miroir {
                    couchee.fliph()
                } else {
                    couchee.clone()
                };
                for _ in 0..quarts {
                    vue = vue.rotate90();
                }
                assert_eq!(
                    vue.to_rgba8(),
                    debout,
                    "{orientation:?}, {quarts}, {miroir}"
                );
            }
        }
        assert_eq!(ramenes, 1, "{orientation:?}");
    }
}

/// **Une photo de téléphone tenue en portrait**, pour les épreuves (EXIF-1) : `l × h` pixels écrits
/// en paysage, la moitié du haut noire, celle du bas blanche ; un APP1 « Exif » d'une seule entrée,
/// l'orientation (0x0112), glissé après le SOI.
pub(crate) fn jpeg_marque((l, h): (u32, u32), exif: u8) -> Vec<u8> {
    let mut rgb = Vec::with_capacity((l * h * 3) as usize);
    for y in 0..h {
        for _ in 0..l {
            rgb.extend(if y < h / 2 {
                [0, 0, 0]
            } else {
                [255, 255, 255]
            });
        }
    }
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
        .encode(&rgb, l, h, image::ExtendedColorType::Rgb8)
        .expect("encodé");
    let mut tiff = b"Exif\0\0II*\0\x08\0\0\0\x01\0".to_vec();
    tiff.extend_from_slice(&[0x12, 0x01, 3, 0, 1, 0, 0, 0, exif, 0, 0, 0, 0, 0, 0, 0]);
    let mut marque = jpeg[..2].to_vec();
    marque.extend_from_slice(&[0xFF, 0xE1]);
    marque.extend_from_slice(&u16::try_from(tiff.len() + 2).expect("court").to_be_bytes());
    marque.extend_from_slice(&tiff);
    marque.extend_from_slice(&jpeg[2..]);
    marque
}
