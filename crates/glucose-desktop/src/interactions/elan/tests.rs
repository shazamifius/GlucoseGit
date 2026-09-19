//! Ce que l'élan doit garantir — et d'abord les deux défauts que l'utilisateur a nommés.
//!
//! Les tests jouent de **vraies suites d'événements et d'images**, aux rythmes réels : un pavé
//! tactile à cent hertz, un écran à deux cent quarante. C'est là que vivaient les défauts —
//! aucun test qui pousse un événement par image ne les aurait vus.
//!
//! Le temps est **simulé**, jamais dormi : l'élan reçoit l'instant de chaque événement, donc
//! un test peut jouer une rafale, un rythme régulier ou un silence sans attendre une seule
//! milliseconde, et sans dépendre de la charge de la machine.

use super::*;

/// La diagonale d'un écran ordinaire, pour le seuil d'extinction.
const DIAGONALE: f64 = 3000.0;

/// L'intervalle d'émission d'un pavé tactile : une centaine d'événements par seconde.
const PAVE: Duration = Duration::from_millis(10);

/// La période d'un écran à deux cent quarante hertz.
const IMAGE: f64 = 1.0 / 240.0;

/// Un geste régulier : `combien` apports, espacés du rythme d'un pavé. Rend l'instant d'après.
fn glisser(elan: &mut Elan, depart: Instant, combien: u32, apport: (f64, f64)) -> Instant {
    let mut t = depart;
    for _ in 0..combien {
        elan.pousser_pan(apport.0, apport.1, t);
        t += PAVE;
    }
    t
}

/// Joue des images pendant `duree`, à partir de `depart`, et rend ce qui a été montré.
fn jouer(elan: &mut Elan, depart: Instant, duree: f64) -> (f64, f64, f64) {
    let mut total = (0.0, 0.0, 0.0);
    let images = (duree / IMAGE) as u32;
    for n in 1..=images {
        let t = depart + Duration::from_secs_f64(IMAGE * f64::from(n));
        if let Some(m) = elan.avancer(t, DIAGONALE) {
            total.0 += m.pan.0;
            total.1 += m.pan.1;
            total.2 += m.octaves;
        }
    }
    total
}

/// Le plus grand pas vers la droite que l'élan montre pendant `duree`.
fn pire_pas_a_droite(elan: &mut Elan, depart: Instant, duree: f64) -> f64 {
    let mut pire: f64 = 0.0;
    let images = (duree / IMAGE) as u32;
    for n in 1..=images {
        let t = depart + Duration::from_secs_f64(IMAGE * f64::from(n));
        if let Some(m) = elan.avancer(t, DIAGONALE) {
            pire = pire.max(m.pan.0);
        }
    }
    pire
}

/// **Une rafale ne projette pas la vue.** Windows livre parfois plusieurs événements d'un coup
/// après un silence ; la version d'avant divisait leur somme par la durée d'une image de quatre
/// millisecondes, et en déduisait une vitesse plusieurs fois trop grande. C'est ce que
/// l'utilisateur voyait comme « des sauts d'image comme si on avait 15 fps ».
#[test]
fn une_rafale_ne_projette_pas_la_vue_plus_loin_qu_un_geste_regulier() {
    // Le même déplacement total, demandé de deux façons : régulièrement, puis d'un bloc.
    let regulier = {
        let mut elan = Elan::default();
        let debut = Instant::now();
        let fin = glisser(&mut elan, debut, 10, (10.0, 0.0));
        jouer(&mut elan, fin, 2.0).0
    };
    let en_rafale = {
        let mut elan = Elan::default();
        let debut = Instant::now();
        // Dix événements arrivés dans la même milliseconde, après le même temps de geste.
        let mut t = debut;
        for _ in 0..10 {
            elan.pousser_pan(10.0, 0.0, t);
            t += Duration::from_micros(100);
        }
        jouer(&mut elan, debut + PAVE * 10, 2.0).0
    };

    // La rafale demande exactement le même déplacement ; elle ne doit pas en montrer
    // davantage sous prétexte qu'elle est arrivée groupée.
    assert!(
        en_rafale < regulier * 1.5,
        "la rafale a projete la vue : {en_rafale:.0} px contre {regulier:.0} px"
    );
}

/// **Le frein.** Repartir dans l'autre sens tue la glissade, et la vue ne repart **jamais**
/// dans l'ancien sens — le défaut que l'utilisateur a vu quatre fois en deux minutes.
#[test]
fn repartir_en_sens_inverse_ne_renvoie_jamais_dans_l_ancien_sens() {
    let mut elan = Elan::default();
    let debut = Instant::now();

    // Un vrai geste vers la droite, au rythme d'un pavé, puis on laisse filer.
    let fin_du_geste = glisser(&mut elan, debut, 12, (30.0, 0.0));
    jouer(&mut elan, fin_du_geste, 0.1);

    // La main repart à gauche, franchement.
    let reprise = fin_du_geste + Duration::from_millis(100);
    elan.pousser_pan(-30.0, 0.0, reprise);

    let pire = pire_pas_a_droite(&mut elan, reprise, 1.0);
    assert!(
        pire <= 1e-9,
        "la vue est repartie vers la droite de {pire} px apres le demi-tour"
    );
}

/// **Le frein vaut pour le zoom**, et pour la même raison : c'est la même soustraction.
#[test]
fn inverser_le_zoom_ne_continue_pas_dans_l_ancien_sens() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    let mut t = debut;
    for _ in 0..12 {
        elan.pousser_zoom(0.05, (100.0, 100.0), t);
        t += PAVE;
    }
    jouer(&mut elan, t, 0.1);

    let reprise = t + Duration::from_millis(100);
    elan.pousser_zoom(-0.05, (100.0, 100.0), reprise);

    let mut pire: f64 = 0.0;
    for n in 1..=240 {
        let quand = reprise + Duration::from_secs_f64(IMAGE * f64::from(n));
        if let Some(m) = elan.avancer(quand, DIAGONALE) {
            pire = pire.max(m.octaves);
        }
    }
    assert!(pire <= 1e-12, "le zoom est reparti de {pire} octave");
}

/// **Une diagonale reste une diagonale, même en zoomant.** « À la fois on va à droite, à la
/// fois on va en haut, et en plus on zoome » — les trois composantes ne se parlent pas, donc
/// aucune ne perturbe les autres.
#[test]
fn une_diagonale_zoomee_garde_ses_proportions() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    let mut t = debut;
    for _ in 0..8 {
        elan.pousser_pan(20.0, -10.0, t);
        elan.pousser_zoom(0.03, (50.0, 50.0), t);
        t += PAVE;
    }
    let total = jouer(&mut elan, t, 3.0);

    assert!(total.0 > 0.0 && total.1 < 0.0 && total.2 > 0.0, "{total:?}");
    let rapport = total.0 / -total.1;
    assert!(
        (rapport - 2.0).abs() < 0.05,
        "la diagonale a tourne : rapport {rapport:.4} au lieu de 2"
    );
}

/// **Le trajet ne dépend pas de la cadence.** À 240 Hz comme à 30 Hz, la même dette se
/// rembourse de la même façon — sinon l'écran de l'utilisateur déciderait de la distance que
/// parcourent ses gestes.
#[test]
fn la_glissade_ne_depend_pas_de_la_cadence() {
    let parcours = |dt: f64| {
        let mut elan = Elan::default();
        let debut = Instant::now();
        let fin = glisser(&mut elan, debut, 10, (25.0, 0.0));
        let mut total = 0.0;
        let images = (3.0 / dt) as u32;
        for n in 1..=images {
            let t = fin + Duration::from_secs_f64(dt * f64::from(n));
            if let Some(m) = elan.avancer(t, DIAGONALE) {
                total += m.pan.0;
            }
        }
        total
    };
    let rapide = parcours(IMAGE);
    let lent = parcours(1.0 / 30.0);
    assert!(
        (rapide - lent).abs() < rapide.abs().max(1.0) * 0.05,
        "a 240 Hz {rapide:.1} px, a 30 Hz {lent:.1} px"
    );
}

/// **Tout ce que la main demande finit par se voir**, et rien de plus : la dette se rembourse
/// intégralement, sans en inventer ni en perdre.
#[test]
fn la_dette_se_rembourse_entierement_et_pas_davantage() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    // Un apport isolé : la source n'a aucun intervalle, donc aucune inertie ne s'y ajoute.
    elan.pousser_pan(400.0, -250.0, debut);
    let total = jouer(&mut elan, debut, 5.0);
    assert!(
        (total.0 - 400.0).abs() < 1.0 && (total.1 + 250.0).abs() < 1.0,
        "montre {total:?} pour 400 et -250 demandes"
    );
    assert!(!elan.en_cours(), "et il ne reste rien a montrer");
}

/// **Un geste neuf n'hérite jamais du précédent.** Reprendre la main efface la dette *et* le
/// rythme mesuré : ce qui précède appartient à un geste terminé.
#[test]
fn un_geste_neuf_n_herite_jamais_du_precedent() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    let fin = glisser(&mut elan, debut, 12, (200.0, 0.0));
    jouer(&mut elan, fin, 3.0);

    // Bien plus tard, un geste minuscule et isolé.
    let plus_tard = fin + Duration::from_secs(5);
    elan.avancer(plus_tard, DIAGONALE);
    elan.pousser_pan(3.0, 0.0, plus_tard);
    let total = jouer(&mut elan, plus_tard, 3.0);
    assert!(
        total.0 < 5.0,
        "trois pixels demandes, {:.1} montres : le geste precedent a deteint",
        total.0
    );
}

/// Une image très longue ne fait pas franchir toute la dette d'un coup — ce serait exactement
/// la téléportation qu'on cherche à supprimer.
#[test]
fn une_image_tres_longue_ne_teleporte_pas() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    elan.pousser_pan(1000.0, 0.0, debut);
    let m = elan
        .avancer(debut + Duration::from_secs(3), DIAGONALE)
        .expect("il reste de la dette");
    assert!(
        m.pan.0 < 1000.0,
        "toute la dette a ete franchie d'un coup : {}",
        m.pan.0
    );
}

/// L'ancre est un **lieu** : elle ne se consomme pas avec la dette et survit à la glissade. La
/// confondre avec une quantité faisait tourner le zoom autour du coin de l'écran dès que la
/// main lâchait.
#[test]
fn l_ancre_ne_se_consomme_pas_avec_la_dette() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    elan.pousser_zoom(0.4, (640.0, 360.0), debut);
    for n in 1..=60 {
        let t = debut + Duration::from_secs_f64(IMAGE * f64::from(n));
        if let Some(m) = elan.avancer(t, DIAGONALE) {
            assert_eq!(m.ancre, (640.0, 360.0));
        }
    }
}

/// Sans rien pousser, rien ne bouge — et surtout aucune image n'est demandée pour rien.
#[test]
fn sans_demande_il_n_y_a_rien_a_montrer() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    assert!(!elan.en_cours());
    for n in 1..=10 {
        let t = debut + Duration::from_secs_f64(IMAGE * f64::from(n));
        assert_eq!(elan.avancer(t, DIAGONALE), None);
    }
}

/// **Un silence dans le geste ne le termine pas** tant qu'il reste dans le rythme de la
/// source. Un pavé tactile hoquette ; conclure au lâcher à chaque trou ferait basculer de
/// régime des dizaines de fois par seconde — c'est exactement ce que faisait la version d'avant.
#[test]
fn un_trou_dans_le_rythme_ne_termine_pas_le_geste() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    let mut t = glisser(&mut elan, debut, 8, (20.0, 0.0));

    // Un trou de deux intervalles, puis le geste reprend : la source a déjà montré pire.
    t += PAVE * 2;
    elan.pousser_pan(20.0, 0.0, t);
    // La dette contient les apports, et rien de plus : aucune inertie n'a été injectée.
    let montre = jouer(&mut elan, t, 0.02).0;
    assert!(
        montre < 200.0,
        "un simple trou a declenche l'inertie : {montre:.0} px en vingt millisecondes"
    );
}
