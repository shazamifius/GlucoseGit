//! MIP-2 — quand une vignette se construit, et ce qu'elle promet de ne pas déplacer.

use super::*;
use tiny_skia::{FilterQuality, PixmapPaint, Transform};

/// Une image à bord net : moitié gauche rouge, moitié droite bleue. Un décalage d'un seul
/// pixel déplace la frontière, et se lit sans ambiguïté.
fn bicolore(largeur: u32, hauteur: u32) -> Pixmap {
    let mut p = Pixmap::new(largeur, hauteur).expect("une image");
    let data = p.data_mut();
    for y in 0..hauteur {
        for x in 0..largeur {
            let i = ((y * largeur + x) * 4) as usize;
            let gauche = x < largeur / 2;
            data[i] = if gauche { 255 } else { 0 };
            data[i + 1] = 0;
            data[i + 2] = if gauche { 0 } else { 255 };
            data[i + 3] = 255;
        }
    }
    p
}

/// La première demande d'une forme ne construit rien : pendant un zoom, chaque image demande
/// une forme neuve, et une vignette serait jetée aussitôt faite.
#[test]
fn test_la_premiere_demande_ne_construit_rien() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(10.0, 20.0, 32.0, 32.0);

    v.ouvrir();
    assert!(v.pour("n", forme, &pyr).is_none());
    assert_eq!(v.faites(), 0);
}

/// La deuxième demande consécutive de la même forme la construit, et les suivantes la
/// réutilisent sans rien refaire.
#[test]
fn test_la_forme_repetee_se_construit_une_fois() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(10.0, 20.0, 32.0, 32.0);

    for _ in 0..20 {
        v.ouvrir();
        v.pour("n", forme, &pyr);
        v.fermer();
    }
    assert_eq!(v.faites(), 1, "vingt images, une seule vignette");
}

/// Un zoom continu ne construit jamais rien : aucune forme ne se répète.
#[test]
fn test_un_zoom_continu_ne_construit_rien() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(256, 256));

    for i in 0..30 {
        v.ouvrir();
        let taille = 40.0 + i as f32;
        v.pour("n", Forme::posee(0.0, 0.0, taille, taille), &pyr);
        v.fermer();
    }
    assert_eq!(v.faites(), 0);
}

/// Deux nœuds montrant la même image à des phases différentes ont chacun leur vignette.
///
/// C'est la raison d'être de ce module : la vignette vivait dans la pyramide, donc une seule
/// pour tout un groupe de copies, qui se la reprenaient à chaque image.
#[test]
fn test_deux_noeuds_sur_la_meme_image_ont_chacun_la_leur() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let a = Forme::posee(10.25, 20.0, 32.0, 32.0);
    let b = Forme::posee(10.75, 20.0, 32.0, 32.0);

    for _ in 0..5 {
        v.ouvrir();
        v.pour("a", a, &pyr);
        v.pour("b", b, &pyr);
        v.fermer();
    }
    assert_eq!(v.faites(), 2, "chaque nœud a construit la sienne, une fois");
    assert_eq!(v.suivis(), 2);
}

/// Un nœud qui sort de l'écran est oublié : le cache est borné par ce qui est dessiné, sans
/// qu'aucun nombre ait été choisi.
#[test]
fn test_un_noeud_non_dessine_est_oublie() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(0.0, 0.0, 32.0, 32.0);

    for _ in 0..3 {
        v.ouvrir();
        v.pour("a", forme, &pyr);
        v.pour("b", forme, &pyr);
        v.fermer();
    }
    assert_eq!(v.suivis(), 2);

    // « b » sort du champ : il n'est plus demandé.
    v.ouvrir();
    v.pour("a", forme, &pyr);
    v.fermer();
    assert_eq!(v.suivis(), 1, "le nœud absent de l'image est oublié");
}

/// **La vignette ne déplace pas l'image.**
///
/// Le chemin par vignette rééchantillonne puis reporte ; le chemin général transforme en une
/// fois. Les deux ne donnent pas les mêmes octets — deux arrondis contre un — mais ils doivent
/// poser la frontière entre les deux couleurs **au même pixel**. Un décalage d'un pixel par
/// image est exactement le genre de défaut qu'aucun test de géométrie ne verrait.
#[test]
fn test_la_vignette_ne_deplace_pas_limage() {
    let source = bicolore(128, 128);
    let (x, y, w, h) = (17.0f32, 23.0f32, 64.0f32, 64.0f32);

    // Chemin général : une seule transformation.
    let mut direct = Pixmap::new(200, 200).expect("un écran");
    direct.draw_pixmap(
        0,
        0,
        source.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..Default::default()
        },
        Transform::from_scale(w / 128.0, h / 128.0).post_translate(x, y),
        None,
    );

    // Chemin par vignette.
    let pyr = Pyramide::nouvelle(source);
    let vignette = pyr.rendre(Forme::posee(x, y, w, h));
    let mut par_vignette = Pixmap::new(200, 200).expect("un écran");
    par_vignette.draw_pixmap(
        x.floor() as i32,
        y.floor() as i32,
        vignette.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    assert_eq!(
        frontiere(&direct, y as u32 + 30),
        frontiere(&par_vignette, y as u32 + 30),
        "la frontière rouge/bleu doit tomber au même pixel par les deux chemins"
    );
}

/// L'abscisse du premier pixel plus bleu que rouge, sur la ligne donnée.
fn frontiere(p: &Pixmap, ligne: u32) -> Option<u32> {
    (0..p.width()).find(|&x| {
        let i = ((ligne * p.width() + x) * 4) as usize;
        let (r, b) = (p.data()[i], p.data()[i + 2]);
        b > r
    })
}
