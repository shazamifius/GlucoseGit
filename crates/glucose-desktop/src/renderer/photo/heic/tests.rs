//! Les épreuves du HEIC : un fichier fabriqué ici par l'encodeur de `heifer` (en dépendance de
//! développement seulement), jamais une photo commitée.

use super::*;
use std::io::Cursor;

/// Un dégradé de `l` × `h`, opaque, encodé en HEIC.
fn fabriquer(l: u32, h: u32) -> (Vec<u8>, Vec<u8>) {
    let rgba = degrade(l, h);
    let heic = heifer::encode_rgba8(l, h, &rgba, &heifer::EncodeOptions::new()).unwrap();
    (heic, rgba)
}

/// Le rouge croît vers la droite, le vert vers le bas.
fn degrade(l: u32, h: u32) -> Vec<u8> {
    (0..l * h)
        .flat_map(|i| {
            let (x, y) = (i % l, i / l);
            [(x * 255 / l) as u8, (y * 255 / h) as u8, 128, 255]
        })
        .collect()
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

// ── EXIF-2 : une photo d'iPhone tenue en portrait (fiche 165 § 7) ─────────────────────────

/// **Un HEIC tel qu'un iPhone tenu en portrait l'écrit** : ses pixels couchés (le dégradé de `l`
/// × `h`), un quart de tour dans le sens des aiguilles (`irot` 3 : trois quarts dans l'autre sens)
/// que l'image et sa vignette partagent, et la balise EXIF qui dit la même chose (6) — seulement
/// informative en HEIF : la suivre aussi tournerait la photo deux fois.
fn tenu_en_portrait(l: u32, h: u32) -> Vec<u8> {
    let image = heifer::Image {
        width: l,
        height: h,
        bit_depth: 8,
        has_alpha: false,
        data: degrade(l, h).into_iter().map(u16::from).collect(),
    };
    // Un EXIF gros-boutiste d'une seule entrée : l'orientation (0x0112, un SHORT), 6.
    let exif = b"MM\0\x2a\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x06\0\0\0\0\0\0".to_vec();
    let metadata = heifer::Metadata {
        exif: Some(exif),
        ..Default::default()
    };
    let options = heifer::EncodeOptions::new();
    let mut heic = heifer::encode_with_metadata(&image, &metadata, &options).unwrap();
    // L'encodeur la remet à 1, ses pixels étant droits ; l'iPhone la laisse dire 6.
    let entree = b"\x01\x12\0\x03\0\0\0\x01\0";
    let at = heic
        .windows(9)
        .position(|w| w == entree)
        .expect("l'orientation")
        + 9;
    heic[at] = 6;
    ajouter_un_irot(&heic, 3)
}

/// Un entier gros-boutiste de quatre octets, lu ou augmenté sur place.
fn u32_a(o: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(o[at..at + 4].try_into().unwrap())
}
fn grandir(o: &mut [u8], at: usize, de: u32) {
    let v = u32_a(o, at) + de;
    o[at..at + 4].copy_from_slice(&v.to_be_bytes());
}

/// **Ajoute un `irot`** (`angle` quarts de tour dans le sens inverse des aiguilles) à chaque image
/// de ce HEIC, comme l'iPhone, et recule d'autant ce que `iloc` situe dans `mdat`. Le fichier de
/// l'encodeur de `heifer` : `iloc` version 0 (champs de quatre octets, une étendue par élément),
/// `ipma` version 0 (index sur sept bits), dans cet ordre.
fn ajouter_un_irot(heic: &[u8], angle: u8) -> Vec<u8> {
    let boite = |nom: &[u8; 4]| heic.windows(4).position(|w| w == nom).expect("boîte") - 4;
    let (iloc, ipco, ipma) = (boite(b"iloc"), boite(b"ipco"), boite(b"ipma"));
    assert!(iloc < ipco && ipco < ipma);
    assert_eq!(
        (heic[iloc + 8], heic[iloc + 12], u32_a(heic, ipma + 8)),
        (0, 0x44, 0)
    );
    let (fin_ipco, fin_ipma) = (
        ipco + u32_a(heic, ipco) as usize,
        ipma + u32_a(heic, ipma) as usize,
    );
    // Son rang dans `ipco` : après les propriétés qui y sont.
    let (mut at, mut rang) = (ipco + 8, 1u8);
    while at < fin_ipco {
        at += u32_a(heic, at) as usize;
        rang += 1;
    }
    // Chaque image l'associe, essentiel (le bit 0x80), après ses autres propriétés.
    let mut nouvelle = heic[ipma..ipma + 16].to_vec();
    let images = u32_a(heic, ipma + 12) as usize;
    let mut at = ipma + 16;
    for _ in 0..images {
        let n = heic[at + 2] as usize;
        nouvelle.extend_from_slice(&heic[at..at + 2]);
        nouvelle.push(heic[at + 2] + 1);
        nouvelle.extend_from_slice(&heic[at + 3..at + 3 + n]);
        nouvelle.push(0x80 | rang);
        at += 3 + n;
    }
    let irot = [0, 0, 0, 9, b'i', b'r', b'o', b't', angle];
    let mut o = [
        &heic[..fin_ipco],
        &irot,
        &heic[fin_ipco..ipma],
        &nouvelle,
        &heic[fin_ipma..],
    ]
    .concat();
    let ajout = (irot.len() + images) as u32;
    grandir(&mut o, ipco, irot.len() as u32);
    grandir(&mut o, ipma + irot.len(), images as u32);
    grandir(&mut o, boite(b"iprp"), ajout);
    grandir(&mut o, boite(b"meta"), ajout);
    for i in 0..u16::from_be_bytes([heic[iloc + 14], heic[iloc + 15]]) as usize {
        assert_eq!(heic[iloc + 16 + i * 14 + 4..iloc + 16 + i * 14 + 6], [0, 1]);
        grandir(&mut o, iloc + 16 + i * 14 + 6, ajout);
    }
    o
}

/// **Le dégradé debout** (`l` × `h` RGBA) : tourné d'un quart dans le sens des aiguilles, le
/// rouge, qui croissait vers la droite, croît vers le bas ; le vert, vers le bas, croît vers la
/// gauche. L'ancien coin en haut à gauche est en haut à droite.
fn est_debout(octets: &[u8], (l, h): (usize, usize)) {
    assert_eq!(octets.len(), l * h * 4);
    let px = |x: usize, y: usize| &octets[(y * l + x) * 4..(y * l + x) * 4 + 2];
    let (haut_droit, bas_gauche) = (px(l - 2, 1), px(1, h - 2));
    assert!(haut_droit[0] < 35 && haut_droit[1] < 35, "{haut_droit:?}");
    assert!(bas_gauche[0] > 220 && bas_gauche[1] > 220, "{bas_gauche:?}");
}

#[test]
fn exif_2_un_portrait_d_iphone_se_decode_debout_une_seule_fois() {
    let heic = tenu_en_portrait(64, 48);
    // Le chemin de l'atelier : la pyramide née du décodeur, tournée comme il le lui dit.
    let decodeur = lecteur(&heic).into_decoder().unwrap();
    let p = crate::renderer::photo::Pyramide::depuis_le_decodeur(decodeur).expect("décodée");
    assert_eq!(
        p.dimensions_natives(),
        (48, 64),
        "debout, et une seule fois"
    );
    est_debout(p.niveau(1).expect("le niveau natif").data(), (48, 64));
    // Ce que lisent aussi le dépôt et le miroir (`dimensions_du_fichier`) : la taille droite, et
    // plus rien à tourner — l'EXIF, informatif, se tait.
    let mut decodeur = lecteur(&heic).into_decoder().unwrap();
    let orientation = decodeur.orientation().unwrap();
    assert_eq!(
        (decodeur.dimensions(), orientation),
        ((48, 64), image::metadata::Orientation::NoTransforms)
    );
}

#[test]
fn exif_2_sa_vignette_aussi() {
    let heic = tenu_en_portrait(800, 600);
    let a = vignette::apercu(&heic).expect("une vignette tournée comme l'image");
    assert_eq!((a.natives, a.rang), ((600, 800), 2));
    est_debout(&a.niveaux[0], (150, 200));
}
