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
