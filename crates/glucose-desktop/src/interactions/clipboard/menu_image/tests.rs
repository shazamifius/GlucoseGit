//! L'image seule d'une copie (COPIER-1) et « Enregistrer l'image sous… » (fiche 51 § 3).

use super::*;
use crate::persist::disque::tests::{application, dossier, image_suivante};

/// Une image de huit sur six dont chaque pixel diffère, dans ce format.
fn octets(format: image::ImageFormat) -> (Vec<u8>, Vec<u8>) {
    let pixels = image::RgbaImage::from_fn(8, 6, |x, y| {
        image::Rgba([(x * 30) as u8, (y * 40) as u8, 200, 255])
    });
    let mut o = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(pixels)
        .to_rgb8()
        .write_to(&mut o, format)
        .expect("encodé");
    let o = o.into_inner();
    let decode = image::load_from_memory(&o)
        .expect("relu")
        .to_rgba8()
        .into_raw();
    (o, decode)
}

/// **Un PNG part tel quel** : ses octets mêmes, sans réencodage.
#[test]
fn test_un_png_part_tel_quel() {
    let (png, pixels) = octets(image::ImageFormat::Png);
    let posee = preparer(&png).expect("prêt");
    assert_eq!(posee.png, png, "les octets d'origine");
    assert_eq!(posee.rgba, pixels);
    assert_eq!((posee.largeur, posee.hauteur), (8, 6));
}

/// **Un JPEG part en PNG, sans perte de ce qu'il montre** : le PNG se décode en les pixels que
/// le JPEG donne.
#[test]
fn test_un_jpeg_part_en_png_sans_perte() {
    let (jpeg, pixels) = octets(image::ImageFormat::Jpeg);
    let posee = preparer(&jpeg).expect("prêt");
    assert_eq!(
        image::guess_format(&posee.png).ok(),
        Some(image::ImageFormat::Png)
    );
    let relu = image::load_from_memory(&posee.png)
        .expect("un PNG")
        .to_rgba8();
    assert_eq!(relu.into_raw(), pixels);
}

/// **Le bitmap est celui que Windows attend** : un en-tête de 124 octets, puis les rangées du bas
/// vers le haut, en bleu-vert-rouge-alpha.
#[test]
fn test_le_bitmap_se_lit_du_bas_vers_le_haut() {
    // Deux pixels sur deux rangées : rouge en haut, bleu en bas.
    let rgba = [
        255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 128, 0, 0, 255, 128,
    ];
    let dib = crate::plateforme::presse_papiers::dib_v5(2, 2, &rgba);
    assert_eq!(dib.len(), 124 + 16);
    assert_eq!(u32::from_le_bytes(dib[0..4].try_into().unwrap()), 124);
    assert_eq!(
        i32::from_le_bytes(dib[8..12].try_into().unwrap()),
        2,
        "hauteur positive"
    );
    assert_eq!(
        &dib[124..128],
        &[255, 0, 0, 128],
        "la rangée du bas d'abord, en BGRA"
    );
    assert_eq!(&dib[132..136], &[0, 0, 255, 255], "puis celle du haut");
}

/// **`Ctrl+C` sur une image seule pose l'image elle-même** (COPIER-1, fiche 59) : ses pixels,
/// que Discord ou Paint collent — « Copier l'image » n'est plus un geste à part —, et le lot,
/// que Glucose recolle en bloc. Sur le presse-papiers à soi de l'épreuve.
#[test]
fn test_copier_une_image_seule_pose_ses_pixels_et_le_lot() {
    let d = dossier("menu-image-copier");
    let mut app = application(&d);
    let (jpeg, pixels) = octets(image::ImageFormat::Jpeg);
    let chemin = d.join("photo.jpg");
    std::fs::write(&chemin, &jpeg).expect("écrit");
    let b = app.store.project.active_board_id.clone();
    app.place_image_file(&b, &chemin, (0.0, 0.0))
        .expect("posée");
    app.copy_selection(false);
    app.suivre_les_echanges(true);
    let mut acces = crate::interactions::presse_papiers::ouvrir().expect("ouvert");
    let colle = acces.image().expect("une image dans le presse-papiers");
    assert_eq!(colle.bytes.as_slice(), pixels.as_slice());
    assert!(acces.lot().is_some(), "et le lot, que Glucose recolle");
}

/// **L'image enregistrée est identique, octet pour octet, à celle qui avait été posée** — après
/// son scellement dans le document, lue de lui seul.
#[test]
fn test_l_image_enregistree_est_celle_qui_avait_ete_posee() {
    let d = dossier("menu-image-enregistrer");
    let mut app = application(&d);
    app.save_to(d.join("doc.glucose"));
    let (jpeg, _) = octets(image::ImageFormat::Jpeg);
    let chemin = d.join("photo.jpg");
    std::fs::write(&chemin, &jpeg).expect("écrit");
    let b = app.store.project.active_board_id.clone();
    app.place_image_file(&b, &chemin, (0.0, 0.0))
        .expect("posée");
    image_suivante(&mut app);
    std::fs::remove_file(&chemin).expect("l'original disparaît");
    let image = app.image_seule_choisie().expect("l'image choisie");
    let cle = image.src.expect("sa clé");
    assert!(app.disque.objets.est_scellee(&cle));
    let scelles = app
        .disque
        .objets
        .lire_en_attendant(&cle)
        .expect("ses octets");
    assert_eq!(extension_de(&scelles).0, "jpg");
    let sortie = d.join("enregistree.jpg");
    ecrire_l_image(&sortie, &scelles).expect("enregistrée");
    assert_eq!(
        std::fs::read(&sortie).expect("relue"),
        jpeg,
        "octet pour octet"
    );
}
