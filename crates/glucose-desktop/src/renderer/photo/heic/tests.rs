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

// ── HEIC-3 : la vignette, aperçu de la première vue (fiche 161 § 8) ───────────────────────

/// L'élément de la vignette de ce fichier : l'encodeur en ajoute une, comme l'iPhone, au-delà
/// de 512 pixels.
fn sa_vignette(octets: &[u8]) -> u32 {
    let fichier = HeifFile::parse(octets).unwrap();
    fichier.referencing_items(fichier.primary_id, b"thmb")[0]
}

#[test]
fn heic_3_la_vignette_devient_l_apercu_de_l_image() {
    let (heic, _) = fabriquer(800, 600);
    let a = vignette::apercu(&heic).expect("une vignette, un aperçu");
    // 320 × 240 couvre 200 × 150 (le rang 2), pas 400 × 300.
    assert_eq!((a.natives, a.rang), ((800, 600), 2));
    let attendus: Vec<_> = (2..)
        .map_while(|r| crate::renderer::photo::dimensions_au_rang((800, 600), r))
        .collect();
    assert_eq!(a.niveaux.len(), attendus.len(), "du rang au pixel");
    for (octets, (l, h)) in a.niveaux.iter().zip(&attendus) {
        assert_eq!(octets.len(), (l * h * 4) as usize);
    }
    // Ses pixels sont ceux de l'image : le rouge croît vers la droite, le vert vers le bas.
    let premier = &a.niveaux[0];
    let px = |x: usize, y: usize| &premier[(y * 200 + x) * 4..(y * 200 + x) * 4 + 4];
    assert!(
        px(195, 2)[0] > 220 && px(2, 2)[0] < 35,
        "{:?} {:?}",
        px(195, 2),
        px(2, 2)
    );
    assert!(px(2, 145)[1] > 220 && px(2, 2)[1] < 35, "{:?}", px(2, 145));
    // Une pyramide partielle : ce que la vignette ne couvre pas se redécodera.
    let pyramide = crate::renderer::photo::Pyramide::depuis_apercu(a).unwrap();
    assert!(!pyramide.entiere());
    assert_eq!(pyramide.dimensions_natives(), (800, 600));
}

#[test]
fn heic_3_sans_vignette_pas_d_apercu() {
    let (heic, _) = fabriquer(64, 48);
    assert!(vignette::apercu(&heic).is_none());
}

#[test]
fn heic_3_la_vignette_se_decode_en_image_premiere() {
    // Le chemin de l'image entière, sur une copie dont la vignette est l'image première : les
    // mêmes pixels que `decode_thumbnail` (ce fichier n'a pas de profil à convertir).
    let (heic, _) = fabriquer(800, 600);
    let id = sa_vignette(&heic);
    let mut copie = heic.clone();
    vignette::designer_l_image_premiere(&mut copie, id).unwrap();
    assert_eq!(HeifFile::parse(&copie).unwrap().primary_id, id);
    let par_la_copie = heifer::decode_srgb_rgba8(&copie, &heifer::Options::new()).unwrap();
    let attendue = heifer::decode_thumbnail(&heic).unwrap().unwrap();
    assert_eq!((par_la_copie.width, par_la_copie.height), (320, 240));
    assert_eq!(attendue.bit_depth, 8);
    let attendue: Vec<u8> = attendue.data.iter().map(|&v| v as u8).collect();
    assert_eq!(par_la_copie.data, attendue);
}

#[test]
fn heic_3_le_rang_d_une_vignette() {
    use vignette::rang_de_la_vignette as rang;
    // Celle de l'iPhone, couchée et debout.
    assert_eq!(rang((4032, 3024), (320, 240)), Some((4, (252, 189))));
    assert_eq!(rang((3024, 4032), (240, 320)), Some((4, (189, 252))));
    // Un pixel d'écart sur le petit côté : l'arrondi de l'encodeur. Trois : une autre image.
    assert_eq!(rang((4032, 3024), (320, 241)), Some((4, (252, 189))));
    assert_eq!(rang((4032, 3024), (320, 243)), None);
    // Les deux côtés couverts : à 320 × 241, le rang 1 agrandirait d'un pixel.
    assert_eq!(rang((640, 482), (320, 240)), Some((2, (160, 121))));
    // Un quart de tour que la vignette n'aurait pas.
    assert_eq!(rang((4032, 3024), (240, 320)), None);
    // Jamais l'image entière : une vignette aussi grande qu'elle couvre au mieux le rang 1.
    assert_eq!(rang((320, 240), (320, 240)), Some((1, (160, 120))));
}

#[test]
fn heic_3_un_fichier_abime_ne_donne_rien_sans_panique() {
    let (heic, _) = fabriquer(800, 600);
    for n in [0, 8, 12, 40, 100, heic.len() / 2, heic.len() - 1] {
        assert!(vignette::apercu(&heic[..n]).is_none(), "tronqué à {n}");
        let mut copie = heic[..n].to_vec();
        let _ = vignette::designer_l_image_premiere(&mut copie, 2);
    }
    // Une boîte qui dit plus long que le fichier.
    let mut faux = heic.clone();
    faux[24..28].copy_from_slice(&u32::MAX.to_be_bytes());
    let _ = vignette::apercu(&faux);
}
