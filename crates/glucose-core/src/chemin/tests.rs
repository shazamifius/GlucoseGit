//! Ce que le chemin garantit : il part d'où l'on est, arrive où l'on va, se parcourt pareil
//! dans les deux sens, et prend de la hauteur quand c'est loin.

use super::*;

const ECRAN: ScreenSize = ScreenSize {
    width: 1920.0,
    height: 1080.0,
};

/// Un cadrage dont le centre de l'écran montre ce point du monde, à cette échelle.
fn centre_sur(monde: (f64, f64), scale: f64) -> Viewport {
    Viewport {
        scale,
        x: ECRAN.width / 2.0 - monde.0 * scale,
        y: ECRAN.height / 2.0 - monde.1 * scale,
    }
}

/// De combien de pixels un coin de l'écran bouge, au pire, entre deux cadrages.
fn ecart(a: Viewport, b: Viewport) -> f64 {
    [
        (0.0, 0.0),
        (ECRAN.width, 0.0),
        (0.0, ECRAN.height),
        (ECRAN.width, ECRAN.height),
    ]
    .into_iter()
    .map(|(sx, sy)| {
        let monde = ((sx - a.x) / a.scale, (sy - a.y) / a.scale);
        (monde.0 * b.scale + b.x - sx).hypot(monde.1 * b.scale + b.y - sy)
    })
    .fold(0.0, f64::max)
}

/// Deux vues très zoomées, à cent mille unités l'une de l'autre.
fn loin_et_zoome() -> (Viewport, Viewport) {
    (
        centre_sur((0.0, 0.0), 4.0),
        centre_sur((100_000.0, 30_000.0), 2.0),
    )
}

/// **Le chemin part de la vue de départ et arrive à celle d'arrivée**, au millième de pixel.
#[test]
fn le_chemin_part_d_ou_l_on_est_et_arrive_ou_l_on_va() {
    let cas = [
        loin_et_zoome(),
        (
            centre_sur((0.0, 0.0), 1.0),
            centre_sur((300.0, -200.0), 1.0),
        ),
        (centre_sur((0.0, 0.0), 0.01), centre_sur((50.0, 50.0), 8.0)),
        (
            centre_sur((10.0, 10.0), 1.0),
            centre_sur((10.0, 10.0), 32.0),
        ),
    ];
    for (depart, arrivee) in cas {
        let chemin = Chemin::entre(depart, arrivee, ECRAN);
        assert!(
            ecart(chemin.vue_a(0.0), depart) < 1e-3,
            "{depart:?} -> {arrivee:?}"
        );
        assert!(
            ecart(chemin.vue_a(chemin.longueur()), arrivee) < 1e-3,
            "{depart:?} -> {arrivee:?} : arrive a {:?}",
            chemin.vue_a(chemin.longueur())
        );
    }
}

/// **Loin et zoomé, le vol prend de la hauteur, puis redescend** — la seconde proposition de
/// l'utilisateur, que le chemin fait de lui-même : au plus haut, l'échelle passe sous celle des
/// deux extrémités, et elle ne descend qu'une fois.
#[test]
fn loin_et_zoome_le_chemin_dezoome_puis_rezoome() {
    let (depart, arrivee) = loin_et_zoome();
    let chemin = Chemin::entre(depart, arrivee, ECRAN);
    let echelles: Vec<f64> = (0..=200)
        .map(|i| chemin.vue_a(chemin.longueur() * f64::from(i) / 200.0).scale)
        .collect();
    let plus_haut = echelles.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        plus_haut < arrivee.scale.min(depart.scale) / 100.0,
        "le vol doit prendre de la hauteur : au plus haut {plus_haut}"
    );
    let creux = echelles
        .iter()
        .position(|&e| e == plus_haut)
        .expect("un creux");
    assert!(
        echelles[..=creux].windows(2).all(|p| p[1] <= p[0]),
        "il ne fait que monter"
    );
    assert!(
        echelles[creux..].windows(2).all(|p| p[1] >= p[0]),
        "puis que redescendre"
    );
}

/// **Un zoom sur place garde son point** : sans déplacement visible, le chemin ne fait que
/// zoomer, d'un même facteur à chaque pas, et le centre de l'écran montre le même point du
/// monde tout du long.
#[test]
fn un_zoom_sur_place_garde_son_point() {
    let (depart, arrivee) = (
        centre_sur((10.0, 10.0), 1.0),
        centre_sur((10.0, 10.0), 32.0),
    );
    let chemin = Chemin::entre(depart, arrivee, ECRAN);
    for i in 0..=10 {
        let vue = chemin.vue_a(chemin.longueur() * f64::from(i) / 10.0);
        let centre = (
            (ECRAN.width / 2.0 - vue.x) / vue.scale,
            (ECRAN.height / 2.0 - vue.y) / vue.scale,
        );
        assert!((centre.0 - 10.0).abs() < 1e-9 && (centre.1 - 10.0).abs() < 1e-9);
        // Un même facteur par pas : l'échelle à mi-chemin est la moyenne géométrique.
        if i == 5 {
            assert!((vue.scale - 32f64.sqrt()).abs() < 1e-9, "{}", vue.scale);
        }
    }
}

/// **Le chemin est le même dans les deux sens** : aller de A à B et revenir de B à A passent par
/// les mêmes vues. C'est une propriété du plus court chemin, et la preuve que la forme close est
/// juste — une erreur de signe la briserait.
#[test]
fn le_chemin_est_le_meme_dans_les_deux_sens() {
    let (a, b) = loin_et_zoome();
    let aller = Chemin::entre(a, b, ECRAN);
    let retour = Chemin::entre(b, a, ECRAN);
    assert!((aller.longueur() - retour.longueur()).abs() < 1e-9);
    for i in 0..=20 {
        let s = aller.longueur() * f64::from(i) / 20.0;
        let (v, w) = (aller.vue_a(s), retour.vue_a(aller.longueur() - s));
        assert!(ecart(v, w) < 1e-3, "a {s} : {v:?} contre {w:?}");
    }
}

/// **La longueur est celle de l'article** : glisser d'une largeur de vue sans zoomer vaut `ρ`
/// — à la limite des petits déplacements —, zoomer d'un facteur `e` vaut `1/ρ`.
#[test]
fn la_longueur_est_celle_de_la_mesure_percue() {
    let zoom = Chemin::entre(
        centre_sur((0.0, 0.0), 1.0),
        centre_sur((0.0, 0.0), std::f64::consts::E),
        ECRAN,
    );
    assert!(
        (zoom.longueur() - 1.0 / RHO).abs() < 1e-9,
        "{}",
        zoom.longueur()
    );
    // Un centième de largeur de vue : le chemin reste presque plat.
    let largeur = ECRAN.width;
    let glisse = Chemin::entre(
        centre_sur((0.0, 0.0), 1.0),
        centre_sur((largeur / 100.0, 0.0), 1.0),
        ECRAN,
    );
    assert!(
        (glisse.longueur() - RHO / 100.0).abs() < 1e-5,
        "{}",
        glisse.longueur()
    );
}
