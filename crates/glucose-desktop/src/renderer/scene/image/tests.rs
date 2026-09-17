//! OCCLUSION-1 — ce qu'on se permet de ne pas dessiner, et ce qu'on refuse de sauter.
//!
//! Ces tests portent sur une decision **visuelle** : retirer du dessin. Une erreur ici ne fait
//! pas ralentir, elle fait disparaitre quelque chose de l'ecran -- le pire defaut possible,
//! parce qu'il se voit et qu'on ne sait pas d'ou il vient. D'ou une regle unique : on ne saute
//! une image que si les trois conditions sont **constatees**, jamais estimees.

use super::*;
use glucose_core::types::{BoardImage, Viewport};
use tiny_skia::Pixmap;

const FENETRE: (f32, f32) = (1000.0, 800.0);

fn clip() -> Clip {
    Clip {
        width: FENETRE.0,
        height: FENETRE.1,
        top: 0.0,
    }
}

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
fn test_une_image_opaque_qui_couvre_l_ecran_cache_celles_d_avant() {
    // Le cas mesure sur le terrain : en zoom proche, une photo remplit la fenetre et les
    // vingt-six autres sont dessinees pour rien, sous elle.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("photo.png", true);

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "photo.png");
    let dessus = image("b", 500.0, 400.0, 2000.0, 1600.0, "photo.png");
    let visibles = vec![&dessous, &dessus];

    assert_eq!(
        premiere_utile(&visibles, &pass, &clip(), &magasin),
        1,
        "on part de l'image qui couvre tout, et pas avant"
    );
}

#[test]
fn test_une_image_translucide_ne_cache_rien() {
    // Le fond doit transparaitre a travers elle : tout ce qu'il y a dessous reste visible.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("voile.png", false);

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "voile.png");
    let dessus = image("b", 500.0, 400.0, 2000.0, 1600.0, "voile.png");
    let visibles = vec![&dessous, &dessus];

    assert_eq!(premiere_utile(&visibles, &pass, &clip(), &magasin), 0);
}

#[test]
fn test_une_image_tournee_ne_cache_rien() {
    // La zone couverte est alors un parallelogramme : les coins du rectangle qui l'entoure
    // laisseraient voir ce qu'il y a dessous.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("photo.png", true);

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "photo.png");
    let mut dessus = image("b", 500.0, 400.0, 2000.0, 1600.0, "photo.png");
    dessus.rotation = 0.3;
    let visibles = vec![&dessous, &dessus];

    assert_eq!(premiere_utile(&visibles, &pass, &clip(), &magasin), 0);
}

#[test]
fn test_une_image_qui_ne_couvre_pas_tout_ne_cache_rien() {
    // Il suffit qu'un seul bord laisse passer : la condition se verifie bord a bord.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("photo.png", true);

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "photo.png");
    // Large mais pas assez haute : le bas de l'ecran reste decouvert.
    let dessus = image("b", 500.0, 300.0, 2000.0, 400.0, "photo.png");
    let visibles = vec![&dessous, &dessus];

    assert_eq!(premiere_utile(&visibles, &pass, &clip(), &magasin), 0);
}

#[test]
fn test_une_image_pas_encore_decodee_ne_cache_rien() {
    // Elle se dessine comme un cadre, a travers lequel on voit le fond : elle ne peut donc
    // rien masquer. C'est aussi le cas normal pendant un import (DECODE-1).
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = Magasin::nouveau();

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "absente.png");
    let dessus = image("b", 500.0, 400.0, 2000.0, 1600.0, "absente.png");
    let visibles = vec![&dessous, &dessus];

    assert_eq!(premiere_utile(&visibles, &pass, &clip(), &magasin), 0);
}

#[test]
fn test_la_derniere_occultante_l_emporte() {
    // Deux images couvrent tout : c'est la plus HAUTE qui decide, sinon on redessinerait
    // inutilement tout ce qui la separe de la precedente.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("photo.png", true);

    let a = image("a", 500.0, 400.0, 2000.0, 1600.0, "photo.png");
    let b = image("b", 100.0, 100.0, 50.0, 50.0, "photo.png");
    let c = image("c", 500.0, 400.0, 2000.0, 1600.0, "photo.png");
    let visibles = vec![&a, &b, &c];

    assert_eq!(premiere_utile(&visibles, &pass, &clip(), &magasin), 2);
}

#[test]
fn test_sans_aucune_image_on_part_de_zero() {
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = Magasin::nouveau();
    assert_eq!(premiere_utile(&[], &pass, &clip(), &magasin), 0);
}

#[test]
fn test_le_bandeau_ne_compte_pas_comme_une_zone_a_couvrir() {
    // La scene ne dessine pas sous le bandeau : une image qui couvre tout ce qui est SOUS lui
    // couvre bien la zone visible, meme si elle ne monte pas jusqu'a l'ordonnee zero.
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = passe(&rangs, &index);
    let magasin = magasin_avec("photo.png", true);
    let avec_bandeau = Clip {
        width: FENETRE.0,
        height: FENETRE.1,
        top: 56.0,
    };

    let dessous = image("a", 100.0, 100.0, 50.0, 50.0, "photo.png");
    // Elle commence a y = 56 exactement, et descend jusqu'en bas.
    let dessus = image("b", 500.0, 428.0, 2000.0, 744.0, "photo.png");
    let visibles = vec![&dessous, &dessus];

    assert_eq!(premiere_utile(&visibles, &pass, &avec_bandeau, &magasin), 1);
}
