//! La traduction d'une image du document en calque geometrique (OCCLUSION-2).
//!
//! Le noyau decide de ce qui se voit ; ce module lui dit ce que sont les images. C'est ici que
//! les trois conditions d'opacite se **constatent**, et une erreur ici ferait disparaitre
//! quelque chose de l'ecran -- le pire defaut possible, parce qu'il se voit et qu'on ne sait
//! pas d'ou il vient.

use super::prevision::boite_ecran;
use super::*;
use glucose_core::types::{BoardImage, Viewport};
use tiny_skia::Pixmap;

fn vue() -> Viewport {
    Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    }
}

/// Une image centree en `(cx, cy)` de taille `(w, h)`, qui porte ce fichier.
fn image(id: &str, cx: f64, cy: f64, w: f64, h: f64, src: &str) -> BoardImage {
    let mut img = BoardImage::new(id, cx, cy, w, h);
    img.src = Some(src.to_string());
    img
}

/// Un magasin ou `src` est decode, opaque ou non selon `opaque`.
fn magasin_avec(src: &str, opaque: bool) -> Magasin {
    let mut magasin = Magasin::nouveau();
    let mut pixmap = Pixmap::new(64, 64).expect("une image");
    let alpha = if opaque { 255 } else { 128 };
    for bloc in pixmap.data_mut().as_chunks_mut::<4>().0 {
        bloc.copy_from_slice(&[10, 20, 30, alpha]);
    }
    magasin.cache.insert(
        src.to_string(),
        crate::renderer::magasin::Entree::pour_test(photo::Pyramide::nouvelle(pixmap)),
    );
    magasin
}

fn passe<'a>(rangs: &'a [u32], index: &'a glucose_core::quadtree::SpatialHash) -> ViewPass<'a> {
    ViewPass {
        vp: vue(),
        visibles: rangs,
        index,
        header_h: 0.0,
    }
}

#[test]
fn test_un_calque_dit_la_verite_sur_ce_qu_il_cache() {
    // Le desktop ne decide plus de l'occlusion -- le noyau s'en charge (OCCLUSION-2). Ce qui
    // reste ici est la TRADUCTION : une image du document devient un calque geometrique, et
    // c'est la que les trois conditions se constatent.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);

    let opaque = magasin_avec("photo.png", true);
    let img = image("a", 0.0, 0.0, 200.0, 100.0, "photo.png");
    assert!(
        calque_de(&img, &pass, &opaque).opaque,
        "une image opaque, droite et decodee cache ce qu'il y a dessous"
    );

    // Tournee : sa boite n'est plus ce qu'elle couvre.
    let mut tournee = img.clone();
    tournee.rotation = 0.3;
    assert!(!calque_de(&tournee, &pass, &opaque).opaque);

    // Translucide : le fond transparait a travers elle.
    let voile = magasin_avec("photo.png", false);
    assert!(!calque_de(&img, &pass, &voile).opaque);

    // Pas encore decodee : elle se dessine comme un cadre, a travers lequel on voit le fond.
    // C'est le cas normal pendant un import (DECODE-1).
    assert!(!calque_de(&img, &pass, &Magasin::nouveau()).opaque);
}

#[test]
fn test_la_boite_ecran_suit_la_vue() {
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    // Une image centree a l'origine du monde, de 200 x 100 : son coin haut-gauche est en
    // (-100, -50), et la vue ne la deplace pas.
    let img = image("a", 0.0, 0.0, 200.0, 100.0, "photo.png");
    let (sx, sy, sw, sh) = boite_ecran(&img, &pass);
    assert_eq!((sx, sy), (-100.0, -50.0));
    assert_eq!((sw, sh), (200.0, 100.0));
}
