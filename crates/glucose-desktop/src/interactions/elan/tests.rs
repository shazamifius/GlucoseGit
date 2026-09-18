//! Ce que l'élan doit garantir. La première garantie est celle qui manquait : un geste
//! interrompt ce qu'il contredit.

use super::*;

/// La diagonale de l'écran servant de référence : elle ne sert qu'au seuil d'arrêt, et une
/// valeur ordinaire suffit à le rendre représentatif.
const DIAGONALE: f64 = 2000.0;

/// `Instant` ne se construit pas depuis un nombre : on fixe une origine une fois pour toute la
/// suite, et chaque test n'exprime plus que des décalages. Rien ne dort jamais ici — le temps
/// est une donnée du test, pas une attente.
fn a(ms: u64) -> Instant {
    static ORIGINE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *ORIGINE.get_or_init(Instant::now) + Duration::from_millis(ms)
}

/// Pousse `pas` fois `(dx, dy)` à dix millisecondes d'intervalle, en partant de `depart`.
/// Rend l'instant de la dernière image.
fn glisser(elan: &mut Elan, depart: u64, pas: u64, (dx, dy): (f64, f64)) -> u64 {
    for i in 1..=pas {
        elan.pousser_pan(dx, dy);
        elan.avancer(a(depart + i * 10), DIAGONALE);
    }
    depart + pas * 10
}

// ── Le frein ────────────────────────────────────────────────────────────────

/// **Le défaut que cette réécriture corrige.** Une glissade vers la droite, puis un geste vers
/// la gauche : la vue doit partir à gauche et ne jamais repartir à droite.
///
/// La version précédente gardait une vitesse lissée sur plus d'une demi-seconde, donc la
/// première image sans événement relançait la vue dans l'ancien sens. Le geste ne pouvait pas
/// interrompre ce qu'il contredisait.
#[test]
fn reprendre_la_main_dans_l_autre_sens_tue_la_glissade() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    let t = glisser(&mut elan, 0, 20, (30.0, 0.0));

    // La glissade est bien lancée vers la droite.
    let filee = elan.avancer(a(t + 10), DIAGONALE).expect("la vue file");
    assert!(filee.pan.0 > 0.0, "vers la droite : {:?}", filee.pan);

    // La main repart à gauche.
    let t = glisser(&mut elan, t + 10, 5, (-30.0, 0.0));

    // Tout ce qui suit doit aller à gauche, sans exception.
    for i in 1..=40 {
        let Some(m) = elan.avancer(a(t + i * 10), DIAGONALE) else {
            break;
        };
        assert!(
            m.pan.0 <= 0.0,
            "la vue est repartie a droite {i} image(s) apres le geste : {:?}",
            m.pan
        );
    }
}

/// Le frein vaut aussi pour l'arrêt franc : reprendre la main puis ne rien demander ne doit
/// pas ressusciter l'élan d'avant, seulement celui du dernier geste.
#[test]
fn un_geste_neuf_n_herite_jamais_de_la_vitesse_du_precedent() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    let t = glisser(&mut elan, 0, 20, (100.0, 0.0));

    // Un geste lent, dans le meme sens : la glissade qui suit doit etre celle du geste LENT.
    let t = glisser(&mut elan, t, 10, (2.0, 0.0));
    let apres = elan.avancer(a(t + 10), DIAGONALE).expect("une glissade lente");
    assert!(
        apres.pan.0 < 5.0,
        "la vitesse du geste rapide a survecu : {:?}",
        apres.pan
    );
}

// ── La conduite ─────────────────────────────────────────────────────────────

/// **Le défaut qui se ressentait dans la main.** Windows livre l'horizontal et le vertical
/// dans deux messages séparés ; s'ils ne se rejoignent pas dans la même image, le mouvement en
/// biais est joué comme un escalier.
#[test]
fn deux_poussees_perpendiculaires_font_une_diagonale_et_pas_un_escalier() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_pan(10.0, 0.0);
    elan.pousser_pan(0.0, 10.0);
    let m = elan.avancer(a(16), DIAGONALE).expect("la demande existe");
    assert_eq!(m.pan, (10.0, 10.0), "les deux composantes partent ensemble");
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

// ── La glissade ─────────────────────────────────────────────────────────────

/// **La propriété qui justifie l'exponentielle.** Deux demi-pas donnent le même résultat qu'un
/// pas entier : la glissade est donc identique à 30 images par seconde et à 240.
#[test]
fn la_glissade_ne_depend_pas_de_la_cadence() {
    let distance = |pas: &[u64]| {
        let mut elan = Elan::default();
        elan.avancer(a(0), DIAGONALE);
        let t = glisser(&mut elan, 0, 10, (10.0, 0.0));
        let mut total = 0.0;
        for d in pas {
            if let Some(m) = elan.avancer(a(t + d), DIAGONALE) {
                total += m.pan.0;
            }
        }
        total
    };
    // Tous les pas restent sous `PAS_MAX` : les deux parcours couvrent la meme duree, decoupee
    // autrement.
    let gros: Vec<u64> = (1..=12).map(|i| i * 50).collect();
    let fin: Vec<u64> = (1..=60).map(|i| i * 10).collect();
    let (gros, fin) = (distance(&gros), distance(&fin));
    assert!(
        (gros - fin).abs() < gros.abs() * 0.01,
        "meme glissade attendue : {gros} contre {fin}"
    );
}

/// La distance totale d'une glissade vaut `v·τ`. C'est ce qui donne un sens à `τ` et permet de
/// le régler sans tâtonner.
#[test]
fn la_glissade_parcourt_la_vitesse_multipliee_par_tau() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    // 10 px toutes les 10 ms = 1000 px/s, entretenu bien au-dela de la fenetre de mesure.
    let t = glisser(&mut elan, 0, 60, (10.0, 0.0));

    let mut total = 0.0;
    for i in 1..=2000 {
        if let Some(m) = elan.avancer(a(t + i * 5), DIAGONALE) {
            total += m.pan.0;
        }
    }
    let attendu = 1000.0 * TAU_PAN;
    assert!(
        (total - attendu).abs() < attendu * 0.02,
        "glissade de {total} px, attendue autour de {attendu}"
    );
}

/// Le mouvement s'éteint sur l'invisible, pas sur un epsilon choisi.
#[test]
fn la_glissade_s_eteint_et_ne_traine_pas() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    let t = glisser(&mut elan, 0, 20, (100.0, 100.0));

    let mut i = 0;
    while elan.en_cours() && i < 2_000 {
        i += 1;
        elan.avancer(a(t + i * 10), DIAGONALE);
    }
    assert!(i < 2_000, "une glissade doit finir");
    assert!(!elan.en_cours());
}

/// Une image très longue ne fait pas bondir la caméra : le pas de temps est borné, et au-delà
/// de la borne le pas ne grandit plus du tout.
#[test]
fn une_image_suspendue_ne_fait_pas_bondir_la_camera() {
    let pas_apres = |attente: u64| {
        let mut elan = Elan::default();
        elan.avancer(a(0), DIAGONALE);
        let t = glisser(&mut elan, 0, 20, (100.0, 0.0));
        elan.avancer(a(t + attente), DIAGONALE).map(|m| m.pan.0)
    };
    let borne = pas_apres(PAS_MAX.as_millis() as u64).expect("la glissade a commence");
    let suspendue = pas_apres(5_000).expect("une suspension glisse encore");
    assert!(
        (borne - suspendue).abs() < 1e-9,
        "au-dela de la borne, le pas ne grandit plus : {borne} contre {suspendue}"
    );
}

// ── Le zoom ─────────────────────────────────────────────────────────────────

/// Le zoom suit la même mécanique, en octaves, et s'arrête sur le même critère.
#[test]
fn le_zoom_glisse_et_s_eteint_comme_le_deplacement() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_zoom(0.5, (100.0, 200.0));
    let m = elan.avancer(a(16), DIAGONALE).unwrap();
    assert_eq!(m.octaves, 0.5);
    assert_eq!(m.ancre, (100.0, 200.0), "l'ancre suit le geste");

    let mut i = 0;
    while elan.en_cours() && i < 2_000 {
        i += 1;
        elan.avancer(a(16 + i * 10), DIAGONALE);
    }
    assert!(i < 2_000, "une glissade de zoom doit finir");
}

/// Les poussées d'une même image s'additionnent : une rafale de pincement ne doit pas coûter
/// une image par événement.
#[test]
fn les_poussees_d_une_image_s_additionnent() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_zoom(0.1, (10.0, 10.0));
    elan.pousser_zoom(0.1, (20.0, 20.0));
    let m = elan.avancer(a(16), DIAGONALE).unwrap();
    assert!((m.octaves - 0.2).abs() < 1e-12);
    assert_eq!(m.ancre, (20.0, 20.0), "la derniere ancre est la bonne");
}

/// **Le bug du « point d'origine ».** L'ancre est un lieu, pas une quantité : quand elle
/// vivait dans la demande, la vider la remettait à `(0, 0)`, et toute la glissade de zoom
/// tournait autour du coin supérieur gauche de la fenêtre.
#[test]
fn l_ancre_ne_se_consomme_pas_avec_la_demande() {
    let mut elan = Elan::default();
    elan.avancer(a(0), DIAGONALE);
    elan.pousser_zoom(0.3, (640.0, 360.0));
    let pendant = elan.avancer(a(16), DIAGONALE).expect("la demande existe");
    assert_eq!(pendant.ancre, (640.0, 360.0));
    let apres = elan.avancer(a(32), DIAGONALE).expect("la glissade continue");
    assert_eq!(
        apres.ancre,
        (640.0, 360.0),
        "la glissade tourne autour du meme point que le geste"
    );
}

// ── La mesure de vitesse ────────────────────────────────────────────────────

/// **La propriété qui rend le frein possible.** Deux déplacements opposés s'annulent dans la
/// fenêtre, donc un demi-tour est vu à l'instant où il se produit.
#[test]
fn deux_deplacements_opposes_s_annulent_dans_la_fenetre() {
    let mut f = Fenetre::default();
    for _ in 0..5 {
        f.noter(0.01, &Mouvement { pan: (10.0, 0.0), ..Default::default() });
    }
    for _ in 0..5 {
        f.noter(0.01, &Mouvement { pan: (-10.0, 0.0), ..Default::default() });
    }
    let v = f.vitesse();
    assert!(v.pan.0.abs() < 1e-9, "vitesse residuelle {:?}", v.pan);
}

/// La fenêtre ne regarde pas plus loin que son dixième de seconde : ce qui est vieux a cessé
/// de décrire le geste en cours.
#[test]
fn la_fenetre_oublie_ce_qui_precede_son_dixieme_de_seconde() {
    let mut f = Fenetre::default();
    // Bien au-dela de la fenetre, a grande vitesse.
    for _ in 0..20 {
        f.noter(0.01, &Mouvement { pan: (100.0, 0.0), ..Default::default() });
    }
    // Puis exactement la fenetre, a vitesse lente.
    for _ in 0..10 {
        f.noter(0.01, &Mouvement { pan: (1.0, 0.0), ..Default::default() });
    }
    let v = f.vitesse();
    assert!(
        v.pan.0 < 200.0,
        "le passe lointain pese encore : {:?} px/s",
        v.pan
    );
}

/// Un geste plus court que la fenêtre a quand même une vitesse : sans cela, une chiquenaude
/// ne lancerait rien du tout.
#[test]
fn un_geste_plus_court_que_la_fenetre_a_une_vitesse() {
    let mut f = Fenetre::default();
    f.noter(0.01, &Mouvement { pan: (10.0, 0.0), ..Default::default() });
    let v = f.vitesse();
    assert!((v.pan.0 - 1000.0).abs() < 1e-9, "{:?}", v.pan);
}

/// L'anneau ne déborde pas : au-delà de sa taille, les plus anciennes sortent, ce qui est leur
/// destin puisqu'elles seraient hors fenêtre de toute façon.
#[test]
fn l_anneau_ne_deborde_pas_et_garde_les_plus_recentes() {
    let mut f = Fenetre::default();
    for _ in 0..ECHANTILLONS * 3 {
        f.noter(0.001, &Mouvement { pan: (1.0, 0.0), ..Default::default() });
    }
    assert_eq!(f.remplies, ECHANTILLONS);
    let v = f.vitesse();
    assert!((v.pan.0 - 1000.0).abs() < 1e-9, "{:?}", v.pan);
}

/// Vider la fenêtre efface tout : c'est ce que fait un geste neuf, et rien ne doit survivre.
#[test]
fn vider_la_fenetre_efface_toute_vitesse() {
    let mut f = Fenetre::default();
    for _ in 0..10 {
        f.noter(0.01, &Mouvement { pan: (50.0, 50.0), ..Default::default() });
    }
    f.vider();
    assert_eq!(f.vitesse(), Vitesse::default());
}
