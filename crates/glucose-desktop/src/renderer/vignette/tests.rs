//! MIP-2 et CASCADE-1 — quand une vignette entre au chantier, quand elle en sort, et ce
//! qu'elle promet de ne pas déplacer.

use super::*;
use tiny_skia::{FilterQuality, PixmapPaint, Transform};

/// Fait tourner l'atelier sans lui mettre de limite : ce que le temps libre permet n'est pas
/// le sujet de ces tests, seul l'est ce qui **mérite** d'être construit.
fn atelier(v: &mut Vignettes, pyr: &Pyramide) -> usize {
    v.avancer_le_chantier(Duration::from_secs(3600), |_| Some(pyr))
}

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

/// La première demande d'une forme n'ouvre même pas de chantier : pendant un zoom, chaque
/// image demande une forme neuve, et une vignette serait jetée aussitôt faite.
#[test]
fn test_la_premiere_demande_ne_construit_rien() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(10.0, 20.0, 32.0, 32.0);

    v.ouvrir();
    assert!(v.pour("n", "f.png", forme, 1024.0).is_none());
    assert_eq!(v.faites(), 0);
    assert_eq!(v.en_chantier(), 0, "rien ne merite encore d'etre construit");
    assert_eq!(atelier(&mut v, &pyr), 0);
}

/// **CASCADE-1, l'invariant central de ce module.** Le rendu ne construit JAMAIS.
///
/// Quel que soit le nombre d'images et de nœuds, tant que l'atelier n'a pas tourné, pas une
/// seule vignette n'existe. C'est ce qui garantit qu'aucune image ne peut geler à cause
/// d'elles — le défaut mesuré chez l'utilisateur valait 471 ms pour quatre-vingt-neuf photos.
#[test]
fn test_le_rendu_ne_construit_jamais() {
    let mut v = Vignettes::new();
    let forme = Forme::posee(10.0, 20.0, 32.0, 32.0);

    for _ in 0..50 {
        v.ouvrir();
        for n in 0..89 {
            v.pour(&format!("n{n}"), "f.png", forme, 1024.0);
        }
        v.fermer();
    }
    assert_eq!(v.faites(), 0, "le rendu seul n'a rien construit");
    assert_eq!(v.en_chantier(), 89, "mais il a note ce qui vaut la peine");
}

/// Le chantier sert d'abord ce dont on voit le plus : c'est la quantité de travail épargné.
#[test]
fn test_le_chantier_sert_la_plus_grande_surface_en_premier() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(0.0, 0.0, 32.0, 32.0);

    for _ in 0..2 {
        v.ouvrir();
        v.pour("petit", "f.png", forme, 10.0);
        v.pour("grand", "f.png", forme, 10_000.0);
        v.pour("moyen", "f.png", forme, 500.0);
        v.fermer();
    }
    assert_eq!(v.en_chantier(), 3);

    // Des tranches si courtes qu'elles n'avancent que d'une ligne : l'atelier ne change pas
    // de chantier avant d'avoir fini celui qu'il a ouvert, donc la premiere vignette prete
    // est celle qu'il a choisie en premier.
    while v.faites() == 0 {
        v.avancer_le_chantier(Duration::from_nanos(1), |_| Some(&pyr));
    }
    v.ouvrir();
    assert!(
        v.pour("grand", "f.png", forme, 10_000.0).is_some(),
        "la plus visible devait sortir la premiere"
    );
    assert!(v.pour("petit", "f.png", forme, 10.0).is_none());
}

/// **La garantie de progrès.** Une tranche de durée nulle avance quand même d'une ligne.
///
/// Sans elle, une machine dont les images mangent déjà leur période ne construirait jamais
/// rien — donc ses images resteraient chères, donc le temps libre resterait nul. Le cercle se
/// refermerait sur elle, et c'est précisément la machine qu'il fallait aider.
///
/// Le test le vérifie par la seule voie observable : à force de tranches vides, la vignette
/// finit par sortir. Si une seule d'entre elles n'avançait pas, la boucle ne finirait jamais.
#[test]
fn test_une_tranche_vide_avance_quand_meme_d_une_ligne() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(0.0, 0.0, 32.0, 32.0);
    for _ in 0..2 {
        v.ouvrir();
        v.pour("n", "f.png", forme, 1024.0);
        v.fermer();
    }
    assert_eq!(v.en_chantier(), 1);

    let mut tranches = 0;
    while v.faites() == 0 {
        v.avancer_le_chantier(Duration::ZERO, |_| Some(&pyr));
        tranches += 1;
        assert!(tranches <= 64, "le chantier n'avance pas : famine");
    }
    assert!(
        tranches > 1,
        "une tranche vide ne doit pas construire la vignette entiere"
    );
}

/// Une tranche large fait sortir la vignette d'un coup : le découpage ne bride pas une
/// machine qui a du temps devant elle.
#[test]
fn test_une_tranche_large_finit_la_vignette() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(64, 64));
    let forme = Forme::posee(0.0, 0.0, 32.0, 32.0);
    for _ in 0..2 {
        v.ouvrir();
        v.pour("n", "f.png", forme, 1024.0);
        v.fermer();
    }
    assert_eq!(
        v.avancer_le_chantier(Duration::from_secs(3600), |_| Some(&pyr)),
        1
    );
    assert_eq!(v.en_chantier(), 0);
}

/// Une photo qui a quitté le cache voit son chantier abandonné, sans boucler.
#[test]
fn test_une_photo_disparue_quitte_le_chantier() {
    let mut v = Vignettes::new();
    let forme = Forme::posee(0.0, 0.0, 32.0, 32.0);
    for _ in 0..2 {
        v.ouvrir();
        v.pour("n", "partie.png", forme, 1024.0);
        v.fermer();
    }
    assert_eq!(v.en_chantier(), 1);
    assert_eq!(v.avancer_le_chantier(Duration::from_secs(1), |_| None), 0);
    assert_eq!(v.en_chantier(), 0, "le chantier sans source est abandonne");
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
        v.pour("n", "f.png", forme, 1024.0);
        atelier(&mut v, &pyr);
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
        v.pour("n", "f.png", Forme::posee(0.0, 0.0, taille, taille), 1024.0);
        atelier(&mut v, &pyr);
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
        v.pour("a", "f.png", a, 1024.0);
        v.pour("b", "f.png", b, 1024.0);
        atelier(&mut v, &pyr);
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
        v.pour("a", "f.png", forme, 1024.0);
        v.pour("b", "f.png", forme, 1024.0);
        atelier(&mut v, &pyr);
        v.fermer();
    }
    assert_eq!(v.suivis(), 2);

    // « b » sort du champ : il n'est plus demandé.
    v.ouvrir();
    v.pour("a", "f.png", forme, 1024.0);
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

/// Un chantier dont la forme n'est plus demandée est **abandonné**, pas mené à terme.
///
/// Une vignette met plusieurs images à sortir. La finir pour une forme que la vue a quittée
/// coûte tout et ne rapporte rien : mesuré chez l'utilisateur, trois cent quarante-trois
/// vignettes prêtes et les trois cent quarante-trois périmées.
#[test]
fn test_un_chantier_perime_est_abandonne() {
    let mut v = Vignettes::new();
    let pyr = Pyramide::nouvelle(bicolore(256, 256));
    let avant = Forme::posee(0.0, 0.0, 200.0, 200.0);
    let apres = Forme::posee(0.5, 0.0, 200.0, 200.0);

    for _ in 0..2 {
        v.ouvrir();
        v.pour("n", "f.png", avant, 1000.0);
        v.fermer();
    }
    // Une tranche si courte que le chantier s'ouvre sans pouvoir finir.
    v.avancer_le_chantier(Duration::from_nanos(1), |_| Some(&pyr));
    assert_eq!(v.faites(), 0, "la vignette ne peut pas etre finie si vite");
    assert_eq!(v.abandonnes(), 0);

    // La vue bouge d'un demi-pixel : la forme demandee n'est plus la meme.
    v.ouvrir();
    v.pour("n", "f.png", apres, 1000.0);
    v.fermer();
    v.avancer_le_chantier(Duration::from_nanos(1), |_| Some(&pyr));
    assert!(
        v.abandonnes() >= 1,
        "le chantier devenu perime devait etre abandonne, pas poursuivi"
    );
}
