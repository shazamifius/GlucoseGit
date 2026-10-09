//! COPIER-1 : ce que rassembler une sélection rend, sur une vraie application.

use super::*;
use crate::persist::disque::tests::{application, dossier};

/// Un PNG de quatre sur trois, de cette teinte : deux teintes, deux images différentes.
fn png(teinte: u8) -> Vec<u8> {
    let pixels = image::RgbaImage::from_pixel(4, 3, image::Rgba([teinte, 10, 200, 255]));
    let mut o = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(&mut o, image::ImageFormat::Png)
        .expect("encodé");
    o.into_inner()
}

/// Pose ces fichiers, tous choisis, et rend le lot de la sélection.
fn lot_de(app: &mut crate::app::GlucoseApp, fichiers: &[std::path::PathBuf]) -> Project {
    let b = app.store.project.active_board_id.clone();
    let mut ids = Vec::new();
    for (k, f) in fichiers.iter().enumerate() {
        app.place_image_file(&b, f, (k as f64 * 300.0, 0.0))
            .expect("posée");
        ids.extend(app.store.selected_image_ids.clone());
    }
    app.store.set_selected_image_ids(ids);
    app.store.extraire_la_selection(&b).expect("un lot")
}

/// **Une image seule part aussi en image** : son PNG, ses pixels — et en fichier, sous son nom
/// d'origine, octet pour octet.
#[test]
fn test_une_image_seule_part_en_image_et_en_fichier() {
    let d = dossier("formes-une");
    let mut app = application(&d);
    let chemin = d.join("ma photo.png");
    std::fs::write(&chemin, png(30)).expect("écrit");
    let lot = lot_de(&mut app, std::slice::from_ref(&chemin));
    let formes = rassembler(&lot, &app.disque.objets, (Some("note".into()), true));
    let image = formes.image.as_ref().expect("l'image seule");
    assert_eq!(image.png, png(30), "un PNG part tel quel");
    assert_eq!(formes.fichiers.len(), 1);
    assert_eq!(formes.fichiers[0].nom, "ma photo.png");
    assert_eq!(formes.fichiers[0].octets.as_slice(), png(30).as_slice());
    assert_eq!(formes.texte.as_deref(), Some("note"));
    let relu = glucose_core::persist::decode(&formes.lot).expect("le lot se relit");
    assert_eq!(relu.project.toutes_les_images().count(), 1);
}

/// **Plusieurs images partent en fichiers, et pas en image** : une image de plus ferait une pièce
/// jointe de trop. Une image posée deux fois n'est qu'un fichier. Et le glisser ne demande pas
/// l'image seule.
#[test]
fn test_plusieurs_images_partent_en_fichiers() {
    let d = dossier("formes-plusieurs");
    let mut app = application(&d);
    let a = d.join("a.png");
    let b = d.join("b.png");
    std::fs::write(&a, png(30)).expect("écrit");
    std::fs::write(&b, png(90)).expect("écrit");
    let lot = lot_de(&mut app, &[a.clone(), b, a]);
    let formes = rassembler(&lot, &app.disque.objets, (None, true));
    assert!(formes.image.is_none(), "pas d'image seule");
    let noms: Vec<&str> = formes.fichiers.iter().map(|f| f.nom.as_str()).collect();
    assert_eq!(noms, ["a.png", "b.png"], "deux fichiers, dans l'ordre");
    let relu = glucose_core::persist::decode(&formes.lot).expect("le lot se relit");
    assert_eq!(relu.project.toutes_les_images().count(), 3, "trois nœuds");

    let seule = d.join("seule.png");
    std::fs::write(&seule, png(150)).expect("écrit");
    let lot = lot_de(&mut app, &[seule]);
    let glisser = rassembler(&lot, &app.disque.objets, (None, false));
    assert!(glisser.image.is_none(), "le glisser ne la demande pas");
    assert_eq!(glisser.fichiers.len(), 1);
}

/// **Le nom d'une image sans fichier d'origine** : « image », et l'extension que ses octets disent.
#[test]
fn test_une_image_sans_fichier_s_appelle_image() {
    assert_eq!(nom_du_fichier("lot:0123abcd", &png(1)), "image.png");
    assert_eq!(nom_du_fichier("sha256:ff00", b"GIF89a....."), "image.gif");
}
