//! Ce que l'élan doit garantir — et la première garantie est celle qui manquait le plus :
//! une diagonale est une diagonale.

use super::*;

/// La diagonale de l'écran servant de référence aux tests : elle ne sert qu'au seuil d'arrêt
/// du zoom, et une valeur ordinaire suffit à le rendre représentatif.
const DIAGONALE: f64 = 2000.0;

fn a(ms: u64) -> Instant {
    // Un instant fixe plus un décalage : les tests ne dorment jamais, le temps est une donnée.
    origine() + Duration::from_millis(ms)
}

/// `Instant` ne se construit pas depuis un nombre : on fixe une origine une fois pour toute
/// la suite, et chaque test n'exprime plus que des décalages. Rien ne dort jamais ici — le
/// temps est une donnée du test, pas une attente.
fn origine() -> Instant {
    static ORIGINE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *ORIGINE.get_or_init(Instant::now)
}

/// **Le défaut qui se ressentait dans la main.** Windows livre l'horizontal et le vertical
/// dans deux messages séparés ; s'ils ne se rejoignent pas dans la même image, le mouvement
/// en biais est joué comme un escalier.
#[test]
fn deux_poussees_perpendiculaires_font_une_diagonale_et_pas_un_escalier() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);

    elan.pousser_pan(10.0, 0.0);
    elan.pousser_pan(0.0, 10.0);

    let m = elan.avancer(a(16), DIAGONALE).expect("la demande existe");
    assert_eq!(
        m.pan,
        (10.0, 10.0),
        "les deux composantes doivent partir ensemble"
    );
}

/// Pendant le geste, la caméra suit **exactement** la demande : le contenu colle au doigt.
#[test]
fn tant_que_la_main_pousse_la_camera_suit_exactement() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_pan(37.0, -12.0);
    let m = elan.avancer(a(16), DIAGONALE).unwrap();
    assert_eq!(m.pan, (37.0, -12.0));
}

/// Une image sans demande et sans élan ne demande rien : c'est ce qui laisse la machine au
/// repos au lieu de tourner pour rien.
#[test]
fn au_repos_l_elan_ne_demande_rien() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    assert!(elan.avancer(a(16), DIAGONALE).is_none());
    assert!(!elan.en_cours());
}

/// **La propriété qui justifie l'exponentielle.** Deux demi-pas donnent le même résultat
/// qu'un pas entier : la glissade est donc identique à 30 images par seconde et à 240.
#[test]
fn la_glissade_ne_depend_pas_de_la_cadence() {
    let distance = |pas: &[u64]| {
        let mut elan = Elan::default();
        elan.avancer(a(0), DIAGONALE);
        elan.pousser_pan(100.0, 0.0);
        elan.avancer(a(100), DIAGONALE);
        let mut total = 0.0;
        for t in pas {
            if let Some(m) = elan.avancer(a(100 + t), DIAGONALE) {
                total += m.pan.0;
            }
        }
        total
    };

    // Tous les pas restent sous `PAS_MAX`, sinon ce ne serait pas la meme duree qu'on
    // comparerait : les deux parcours vont de 100 ms a 700 ms, decoupes autrement.
    let gros: Vec<u64> = (1..=12).map(|i| i * 50).collect();
    let fin: Vec<u64> = (1..=60).map(|i| i * 10).collect();
    let (gros, fin) = (distance(&gros), distance(&fin));
    assert!(
        (gros - fin).abs() < gros.abs() * 0.01,
        "meme glissade attendue : {gros} contre {fin}"
    );
}

/// La distance totale d'une glissade vaut `v·τ`. C'est ce qui donne un sens à `τ`, et ce qui
/// permet de le régler sans tâtonner.
#[test]
fn la_glissade_parcourt_la_vitesse_multipliee_par_tau() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    // Une poussée entretenue pendant bien plus que `τ` : la vitesse lissée rejoint la vraie.
    for i in 1..=200 {
        elan.pousser_pan(10.0, 0.0);
        elan.avancer(a(i * 10), DIAGONALE);
    }
    // 10 px toutes les 10 ms = 1000 px/s.
    let mut total = 0.0;
    for i in 1..=400 {
        if let Some(m) = elan.avancer(a(2000 + i * 5), DIAGONALE) {
            total += m.pan.0;
        }
    }
    let attendu = 1000.0 * TAU;
    assert!(
        (total - attendu).abs() < attendu * 0.05,
        "glissade de {total} px, attendue autour de {attendu}"
    );
}

/// Le mouvement s'éteint sur l'invisible, pas sur un epsilon choisi : ce qui reste à parcourir
/// tient sous le demi-pixel.
#[test]
fn la_glissade_s_eteint_quand_il_ne_reste_plus_un_demi_pixel() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_pan(1000.0, 1000.0);
    elan.avancer(a(10), DIAGONALE);

    let mut t = 10;
    while elan.en_cours() && t < 10_000 {
        t += 10;
        elan.avancer(a(t), DIAGONALE);
    }
    assert!(t < 10_000, "une glissade doit finir");
    assert!(!elan.en_cours());
}

/// Une image très longue ne fait pas bondir la caméra : le pas de temps est borné, parce
/// qu'au-delà la mesure ne décrit plus un mouvement.
///
/// Se vérifie sans connaître la vitesse : une suspension de cinq secondes doit donner
/// **exactement** le même pas qu'une image de cent millisecondes, puisque c'est là que la
/// borne tombe.
#[test]
fn une_image_suspendue_ne_fait_pas_bondir_la_camera() {
    let pas_apres = |attente: u64| {
        let mut elan = Elan::default();
        elan.avancer(a(0), DIAGONALE);
        elan.pousser_pan(1000.0, 0.0);
        elan.avancer(a(10), DIAGONALE);
        elan.avancer(a(10 + attente), DIAGONALE).map(|m| m.pan.0)
    };
    let borne = pas_apres(PAS_MAX.as_millis() as u64).expect("la glissade a commence");
    let suspendue = pas_apres(5_000).expect("une suspension glisse encore");
    assert!(
        (borne - suspendue).abs() < 1e-9,
        "au-dela de la borne, le pas ne grandit plus : {borne} contre {suspendue}"
    );
}

/// Le zoom suit la même mécanique, en octaves — et s'arrête sur le même critère, ramené aux
/// pixels que le bord de l'écran parcourrait.
#[test]
fn le_zoom_glisse_et_s_eteint_comme_le_deplacement() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_zoom(0.5, (100.0, 200.0));
    let m = elan.avancer(a(16), DIAGONALE).unwrap();
    assert_eq!(m.octaves, 0.5);
    assert_eq!(m.ancre, (100.0, 200.0), "l'ancre suit le geste");

    let mut t = 16;
    while elan.en_cours() && t < 10_000 {
        t += 10;
        elan.avancer(a(t), DIAGONALE);
    }
    assert!(t < 10_000, "une glissade de zoom doit finir");
}

/// Les poussées de zoom d'une même image s'additionnent : une rafale de pincement ne doit pas
/// coûter une image par événement.
#[test]
fn les_poussees_de_zoom_d_une_image_s_additionnent() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_zoom(0.1, (10.0, 10.0));
    elan.pousser_zoom(0.1, (20.0, 20.0));
    let m = elan.avancer(a(16), DIAGONALE).unwrap();
    assert!((m.octaves - 0.2).abs() < 1e-12);
    assert_eq!(m.ancre, (20.0, 20.0), "la derniere ancre est la bonne");
}
