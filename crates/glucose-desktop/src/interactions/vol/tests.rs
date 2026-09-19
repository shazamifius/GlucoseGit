//! Ce qu'un vol doit garantir — sans écran, sans horloge réelle.

use super::*;

const ECRAN: ScreenSize = ScreenSize {
    width: 1920.0,
    height: 1080.0,
};

fn vue(x: f64, y: f64, scale: f64) -> Viewport {
    Viewport { x, y, scale }
}

/// Jouer un vol jusqu'à son terme, et rendre le cadrage final et le nombre d'images.
fn jusqu_au_bout(vol: &mut Vol, depart: Viewport, dt: f64) -> (Viewport, usize) {
    let mut courante = depart;
    for image in 1..=10_000 {
        match vol.avancer(courante, ECRAN, dt) {
            Some(suivante) => courante = suivante,
            None => return (courante, image),
        }
    }
    panic!("un vol doit se terminer");
}

/// **Un vol arrive, et il arrive exactement.** Sans cela il resterait une dérive d'une
/// fraction de pixel à chaque destination, qui s'accumulerait de vol en vol.
#[test]
fn un_vol_atteint_sa_cible_au_pixel_pres_et_s_arrete() {
    let mut vol = Vol::default();
    let cible = vue(-4000.0, 2500.0, 0.37);
    vol.viser(cible);
    assert!(vol.en_cours());

    let (arrivee, _) = jusqu_au_bout(&mut vol, vue(0.0, 0.0, 1.0), 1.0 / 60.0);
    assert_eq!(arrivee, cible, "le vol pose exactement la cible");
    assert!(!vol.en_cours(), "et il ne se poursuit pas apres");
}

/// **Le trajet ne dépend pas de la cadence.** À 240 Hz comme à 30 Hz, la même durée de vol
/// doit avoir comblé la même fraction du chemin — sinon l'écran de l'utilisateur déciderait
/// de la vitesse de ses déplacements.
#[test]
fn la_duree_du_vol_ne_depend_pas_de_la_cadence() {
    let depart = vue(0.0, 0.0, 1.0);
    let cible = vue(-3000.0, 0.0, 1.0);

    let apres = |dt: f64, duree: f64| {
        let mut vol = Vol::default();
        vol.viser(cible);
        let mut courante = depart;
        let images = (duree / dt).round() as usize;
        for _ in 0..images {
            let Some(suivante) = vol.avancer(courante, ECRAN, dt) else {
                break;
            };
            courante = suivante;
        }
        courante.x
    };

    let rapide = apres(1.0 / 240.0, 0.2);
    let lent = apres(1.0 / 30.0, 0.2);
    assert!(
        (rapide - lent).abs() < 1.0,
        "apres 200 ms : {rapide} px a 240 Hz contre {lent} px a 30 Hz"
    );
}

/// **L'altitude s'interpole en octaves.** À mi-parcours d'un vol de ×1 à ×16, on doit être à
/// ×4 — la moyenne géométrique — et non à ×8,5, la moyenne arithmétique.
///
/// C'est ce qui rend la descente régulière à l'œil : chaque fraction du trajet multiplie
/// l'échelle d'autant, au lieu de la précipiter puis de la faire ramper.
#[test]
fn l_altitude_se_parcourt_en_octaves_pas_en_facteurs() {
    let mut vol = Vol::default();
    let depart = vue(0.0, 0.0, 1.0);
    // Une cible qui ne change QUE l'échelle : même point du monde au centre de l'écran.
    let (centre, _) = decomposer(depart, ECRAN);
    vol.viser(composer(centre, 4.0, ECRAN));

    // La fraction comblée après une durée t vaut 1 − e^(−t/τ) ; on cherche t tel qu'elle
    // vaille exactement la moitié, pour lire l'échelle au milieu du trajet.
    let demi = TAU * 2.0f64.ln();
    let milieu = vol
        .avancer(depart, ECRAN, demi)
        .expect("le vol est en cours");

    assert!(
        (milieu.scale - 4.0).abs() < 1e-9,
        "a mi-parcours de x1 vers x16 on attend x4, on a x{}",
        milieu.scale
    );
}

/// **Le point visé ne dérive pas pendant que l'altitude change.** Un vol qui zoome sans se
/// déplacer doit garder le même point du monde au centre, sinon la cible glisse sous l'œil.
#[test]
fn un_vol_qui_ne_fait_que_zoomer_ne_deplace_pas_le_centre() {
    let depart = vue(137.0, -42.0, 0.6);
    let (centre_depart, _) = decomposer(depart, ECRAN);

    let mut vol = Vol::default();
    vol.viser(composer(centre_depart, 2.5, ECRAN));

    let mut courante = depart;
    for _ in 0..20 {
        let Some(suivante) = vol.avancer(courante, ECRAN, 1.0 / 60.0) else {
            break;
        };
        courante = suivante;
        let (centre, _) = decomposer(courante, ECRAN);
        assert!(
            (centre.0 - centre_depart.0).abs() < 1e-9 && (centre.1 - centre_depart.1).abs() < 1e-9,
            "le centre a derive : {centre:?} au lieu de {centre_depart:?}"
        );
    }
}

/// **Redéfinir la destination en plein vol ne fait aucune secousse** — c'est ce que demande
/// une minimap qu'on maintient : la cible suit le curseur, image après image.
#[test]
fn changer_de_cible_en_plein_vol_ne_saute_pas() {
    let mut vol = Vol::default();
    vol.viser(vue(-2000.0, 0.0, 1.0));

    let mut courante = vue(0.0, 0.0, 1.0);
    for _ in 0..5 {
        courante = vol.avancer(courante, ECRAN, 1.0 / 60.0).expect("en vol");
    }
    let avant = courante;

    // La destination change du tout au tout ; la première image d'après doit rester proche.
    vol.viser(vue(3000.0, 1200.0, 1.0));
    let apres = vol.avancer(courante, ECRAN, 1.0 / 60.0).expect("en vol");

    let saut = ecart_max_en_pixels(avant, apres, ECRAN);
    assert!(
        saut < 400.0,
        "une image ne doit pas teleporter : {saut} px d'un coup"
    );
}

/// **Poser annule le vol et laisse la vue où elle est** : un vol est une intention passée,
/// elle ne discute jamais avec le geste présent.
#[test]
fn poser_abandonne_le_vol_sans_bouger_la_vue() {
    let mut vol = Vol::default();
    vol.viser(vue(-5000.0, 0.0, 1.0));
    let courante = vol
        .avancer(vue(0.0, 0.0, 1.0), ECRAN, 1.0 / 60.0)
        .expect("en vol");

    vol.poser();
    assert!(!vol.en_cours());
    assert_eq!(
        vol.avancer(courante, ECRAN, 1.0 / 60.0),
        None,
        "plus rien a faire, et surtout pas un retour en arriere"
    );
}

/// L'écart se mesure bien **au pire coin**, et il est nul entre un cadrage et lui-même.
#[test]
fn l_ecart_se_lit_en_pixels_au_pire_coin() {
    let a = vue(0.0, 0.0, 1.0);
    assert_eq!(ecart_max_en_pixels(a, a, ECRAN), 0.0);

    // Une translation pure : tous les coins bougent pareil.
    let translate = vue(30.0, 40.0, 1.0);
    assert!((ecart_max_en_pixels(a, translate, ECRAN) - 50.0).abs() < 1e-9);

    // Un zoom autour du coin haut-gauche : c'est le coin opposé qui bouge le plus.
    let zoome = vue(0.0, 0.0, 2.0);
    let attendu = ECRAN.width.hypot(ECRAN.height);
    assert!((ecart_max_en_pixels(a, zoome, ECRAN) - attendu).abs() < 1e-9);
}

const BANDEAU: f64 = 78.0;

fn rect(left: f64, top: f64, width: f64, height: f64) -> Rect {
    Rect {
        left,
        top,
        width,
        height,
    }
}

/// Où un point du monde se pose à l'écran, sous ce cadrage.
fn a_l_ecran(vue: Viewport, monde: (f64, f64)) -> (f64, f64) {
    (monde.0 * vue.scale + vue.x, monde.1 * vue.scale + vue.y)
}

/// **Tout le contenu tient à l'écran, et sous le bandeau.** C'est la seule chose que la
/// touche `F` promet.
#[test]
fn le_cadrage_montre_tout_le_contenu_sous_le_bandeau() {
    let contenu = rect(-1200.0, 340.0, 4000.0, 900.0);
    let vue = cadrage_du_contenu(contenu, ECRAN, BANDEAU);

    for coin in [
        (contenu.left, contenu.top),
        (contenu.left + contenu.width, contenu.top),
        (contenu.left, contenu.top + contenu.height),
        (contenu.left + contenu.width, contenu.top + contenu.height),
    ] {
        let (x, y) = a_l_ecran(vue, coin);
        assert!(
            (0.0..=ECRAN.width).contains(&x),
            "le coin {coin:?} sort de l'ecran en x : {x}"
        );
        assert!(
            (BANDEAU..=ECRAN.height).contains(&y),
            "le coin {coin:?} passe sous le bandeau ou sous le bord : {y}"
        );
    }
}

/// **Il n'y a pas de point d'origine**, et c'est ce que ce test verrouille : le même contenu,
/// posé à un million d'unités de là, donne exactement le même cadrage à la translation près.
///
/// L'ancienne touche `F` faisait l'inverse — elle ramenait à l'origine du monde, qui n'est le
/// centre de rien dans un canva infini.
#[test]
fn le_cadrage_ne_connait_aucun_point_d_origine() {
    let ici = cadrage_du_contenu(rect(0.0, 0.0, 800.0, 600.0), ECRAN, BANDEAU);
    let loin = cadrage_du_contenu(rect(1e6, -3e6, 800.0, 600.0), ECRAN, BANDEAU);

    assert!(
        (ici.scale - loin.scale).abs() < 1e-9,
        "la meme etendue se voit a la meme echelle, ou qu'elle soit"
    );
    // Et le centre du contenu tombe au même endroit de l'écran dans les deux cas.
    let centre_ici = a_l_ecran(ici, (400.0, 300.0));
    let centre_loin = a_l_ecran(loin, (1e6 + 400.0, -3e6 + 300.0));
    assert!(
        (centre_ici.0 - centre_loin.0).abs() < 1e-6 && (centre_ici.1 - centre_loin.1).abs() < 1e-6,
        "{centre_ici:?} contre {centre_loin:?}"
    );
}

/// **Une échelle bornée reste centrée.** Un seul nœud demanderait un grossissement que le
/// modèle refuse ; borner l'échelle sans refaire le centrage laisserait le cadrage à côté de
/// sa propre cible — donc `F` sur un tableau d'un seul nœud ne le montrerait pas.
#[test]
fn un_contenu_minuscule_reste_centre_malgre_la_borne() {
    let contenu = rect(5000.0, -5000.0, 0.5, 0.5);
    let vue = cadrage_du_contenu(contenu, ECRAN, BANDEAU);

    let (max, _) = (Viewport::SCALE_RANGE.1, ());
    assert!(
        vue.scale <= max,
        "l'echelle reste dans les bornes du modele"
    );

    let centre = a_l_ecran(vue, (contenu.left + 0.25, contenu.top + 0.25));
    assert!(
        (centre.0 - ECRAN.width / 2.0).abs() < 1e-6,
        "centre en x : {}",
        centre.0
    );
    assert!(
        (centre.1 - (BANDEAU + (ECRAN.height - BANDEAU) / 2.0)).abs() < 1e-6,
        "centre en y, sous le bandeau : {}",
        centre.1
    );
}
