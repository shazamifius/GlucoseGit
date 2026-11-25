//! Les épreuves du HEIC : un fichier fabriqué ici par l'encodeur de `heifer` (en dépendance de
//! développement seulement), jamais une photo commitée.

use super::*;
use std::io::Cursor;

/// Un dégradé de `l` × `h`, opaque, encodé en HEIC.
fn fabriquer(l: u32, h: u32) -> (Vec<u8>, Vec<u8>) {
    let rgba: Vec<u8> = (0..l * h)
        .flat_map(|i| {
            let (x, y) = (i % l, i / l);
            [(x * 255 / l) as u8, (y * 255 / h) as u8, 128, 255]
        })
        .collect();
    let heic = heifer::encode_rgba8(l, h, &rgba, &heifer::EncodeOptions::new()).unwrap();
    (heic, rgba)
}

fn lecteur(octets: &[u8]) -> image::ImageReader<Cursor<&[u8]>> {
    enregistrer();
    image::ImageReader::new(Cursor::new(octets))
        .with_guessed_format()
        .unwrap()
}

#[test]
fn la_signature_suffit_a_reconnaitre_un_heic() {
    let (heic, _) = fabriquer(64, 48);
    assert_eq!(lecteur(&heic).into_dimensions().unwrap(), (64, 48));
}

#[test]
fn les_dimensions_se_lisent_sans_decoder() {
    // Le conteneur intact, l'image HEVC (le contenu de la boîte `mdat`) effacée : les
    // dimensions se lisent encore — donc sans décoder —, le décodage échoue.
    let (mut heic, _) = fabriquer(64, 48);
    let mdat = heic.windows(4).position(|w| w == b"mdat").unwrap();
    let taille = u32::from_be_bytes(heic[mdat - 4..mdat].try_into().unwrap()) as usize;
    heic[mdat + 4..mdat - 4 + taille].fill(0);
    assert_eq!(lecteur(&heic).into_dimensions().unwrap(), (64, 48));
    assert!(lecteur(&heic).decode().is_err());
}

#[test]
fn les_pixels_sont_ceux_de_l_image() {
    let (heic, rgba) = fabriquer(64, 48);
    let lu = lecteur(&heic).decode().unwrap().to_rgba8();
    assert_eq!(lu.dimensions(), (64, 48));
    let ecart: u64 = lu
        .as_raw()
        .iter()
        .zip(&rgba)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum::<u64>()
        / rgba.len() as u64;
    assert!(ecart <= 4, "écart moyen {ecart}");
}

#[test]
fn un_heic_tronque_est_refuse_sans_panique() {
    let (heic, _) = fabriquer(64, 48);
    for fin in [12, 40, heic.len() / 2, heic.len() - 1] {
        let lu = std::panic::catch_unwind(|| {
            lecteur(&heic[..fin]).into_decoder().and_then(|d| {
                let mut tampon = vec![0; d.total_bytes() as usize];
                d.read_image(&mut tampon)
            })
        });
        assert!(matches!(lu, Ok(Err(_))), "tronqué à {fin}");
    }
}

#[test]
fn le_recadrage_suit_celui_de_heifer() {
    // Le `clap` d'une image de 4032 de côté réduit à 4000, centré : 4000 ; décalé de 100 :
    // il commence en 116 (2 115,5 − 1 999,5), borné par le bord à 3 916 ; plus grand que
    // l'image : l'image ; un dénominateur nul : refusé.
    assert_eq!(recadrer(4032, (4000, 1), (0, 1)), Some(4000));
    assert_eq!(recadrer(4032, (4000, 1), (100, 1)), Some(3916));
    assert_eq!(recadrer(4032, (9000, 1), (0, 1)), Some(4032));
    assert_eq!(recadrer(4032, (4000, 0), (0, 1)), None);
}

#[test]
fn les_transformations_dans_leur_ordre() {
    // Un quart de tour impair échange les côtés, pair non, le miroir jamais ; le recadrage
    // s'applique au côté qu'il trouve : avant le quart de tour, à la largeur codée.
    let clap = |l| Property::CleanAperture {
        width: (l, 1),
        height: (100, 1),
        horiz_offset: (0, 1),
        vert_offset: (0, 1),
    };
    let t = |p: &[Property]| transformer((4032, 3024), p);
    assert_eq!(t(&[Property::Rotation(1)]), Some((3024, 4032)));
    assert_eq!(t(&[Property::Rotation(3)]), Some((3024, 4032)));
    assert_eq!(t(&[Property::Rotation(2)]), Some((4032, 3024)));
    assert_eq!(t(&[Property::Mirror(true)]), Some((4032, 3024)));
    assert_eq!(t(&[clap(4000), Property::Rotation(1)]), Some((100, 4000)));
    assert_eq!(t(&[Property::Rotation(1), clap(3000)]), Some((3000, 100)));
}

#[test]
fn des_dimensions_fausses_sont_refusees_sans_panique() {
    // Si le conteneur et le décodage ne s'accordaient pas, l'image est refusée — jamais un
    // tampon rempli de travers.
    let (octets, _) = fabriquer(64, 48);
    let faux = Heic {
        octets,
        dimensions: (10, 10),
    };
    assert!(faux.read_image(&mut [0; 400]).is_err());
}

#[test]
fn la_signature_d_un_heic() {
    let (heic, _) = fabriquer(16, 16);
    assert!(est_heic(&heic));
    assert!(!est_heic(b"\x89PNG\r\n\x1a\n\0\0\0\0"));
    assert!(!est_heic(b"\0\0\0\x1cftypavif"));
}
