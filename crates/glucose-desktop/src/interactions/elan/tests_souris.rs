//! La voie de la souris (fiche 51 § 1) : ce qu'elle demande se montre en entier à l'image
//! suivante, et rien ne reste après.
//!
//! Chaque épreuve porte sa **preuve à l'envers** : le même geste, joué par la porte du doigt —
//! celle que la souris empruntait jusqu'à la 2.0.1 —, montre le défaut que l'utilisateur a
//! décrit. Sans elle, une épreuve qui passe ne dirait pas qu'elle sait voir ce défaut.

use super::*;

const DIAGONALE: f64 = 3000.0;
const IMAGE: Duration = Duration::from_nanos(4_166_667);
const CRAN: f64 = 0.125;

/// Ce que l'élan montre pendant `images` images, à partir de `depuis`.
fn montrer(elan: &mut Elan, depuis: Instant, images: u32) -> (f64, f64) {
    let mut total = (0.0, 0.0);
    for n in 1..=images {
        if let Some(m) = elan.avancer(depuis + IMAGE * n, IMAGE, DIAGONALE) {
            total.0 += m.pan.0;
            total.1 += m.octaves;
        }
    }
    total
}

/// **Un cran de molette donne tout son zoom dans l'image suivante, et rien après.** Autour du
/// point visé, et sans laisser l'élan « en cours » : plus rien ne réveille Glucose.
#[test]
fn un_cran_se_montre_tout_entier_a_l_image_suivante() {
    let mut elan = Elan::default();
    let t = Instant::now();
    elan.placer_zoom(CRAN, (300.0, 200.0));
    let m = elan
        .avancer(t + IMAGE, IMAGE, DIAGONALE)
        .expect("le cran se montre");
    assert_eq!(m.octaves, CRAN, "tout le cran, dans cette image");
    assert_eq!(m.ancre, (300.0, 200.0), "autour du point visé");
    assert!(!elan.en_cours(), "plus rien n'est dû");
    assert!(elan.bouge(), "mais cette image est une image de mouvement");
    assert_eq!(
        montrer(&mut elan, t + IMAGE, 480),
        (0.0, 0.0),
        "et rien après"
    );
    assert!(!elan.bouge(), "la vue s'est arrêtée");

    // À l'envers : par la porte du doigt, le même cran ne se montre qu'en partie.
    let mut doigt = Elan::default();
    doigt.pousser_zoom(CRAN, (300.0, 200.0), t);
    let premier = doigt.avancer(t + IMAGE, IMAGE, DIAGONALE).unwrap().octaves;
    assert!(
        premier < CRAN * 0.5,
        "le doigt lisse : {premier} sur {CRAN}"
    );
}

/// **Une série de crans ne laisse aucune glissade.** Huit crans espacés comme une main qui
/// tourne la molette : à la dernière image, la vue a doublé, et elle ne bouge plus.
#[test]
fn une_serie_de_crans_ne_laisse_aucune_glissade() {
    let jouer = |souris: bool| {
        let mut elan = Elan::default();
        let mut t = Instant::now();
        let mut montre = 0.0;
        for _ in 0..8 {
            if souris {
                elan.placer_zoom(CRAN, (0.0, 0.0));
            } else {
                elan.pousser_zoom(CRAN, (0.0, 0.0), t);
            }
            for _ in 0..6 {
                t += IMAGE;
                montre += elan.avancer(t, IMAGE, DIAGONALE).map_or(0.0, |m| m.octaves);
            }
        }
        (montre, montrer(&mut elan, t, 480).1)
    };
    let (pendant, apres) = jouer(true);
    assert!(
        (pendant - 1.0).abs() < 1e-12,
        "huit crans, une octave : {pendant}"
    );
    assert_eq!(apres, 0.0, "rien ne glisse après le dernier cran");

    let (_, glissade) = jouer(false);
    assert!(
        glissade > 0.1,
        "à l'envers, le doigt glisse encore de {glissade} octave"
    );
}

/// **Un glisser à la souris suit le curseur au pixel, et s'arrête net au lâcher.** Une souris
/// à mille hertz, un écran à deux cent quarante : à chaque image, la vue a montré exactement
/// ce que la main a demandé jusque-là.
#[test]
fn un_glisser_suit_la_main_et_s_arrete_net() {
    let jouer = |souris: bool| {
        let mut elan = Elan::default();
        let debut = Instant::now();
        let (mut demande, mut montre, mut pire_retard) = (0.0, 0.0, 0.0_f64);
        for ms in 1..=300u32 {
            let t = debut + Duration::from_millis(u64::from(ms));
            if souris {
                elan.placer_pan(2.0, 0.0);
            } else {
                elan.pousser_pan(2.0, 0.0, t);
            }
            demande += 2.0;
            if ms % 4 == 0 {
                montre += elan.avancer(t, IMAGE, DIAGONALE).map_or(0.0, |m| m.pan.0);
                pire_retard = pire_retard.max(demande - montre);
            }
        }
        let fin = debut + Duration::from_millis(300);
        (pire_retard, montrer(&mut elan, fin, 480).0)
    };
    let (retard, reste) = jouer(true);
    assert!(
        retard < 1e-9,
        "la vue suit la main sans retard : {retard} px"
    );
    assert_eq!(reste, 0.0, "lâcher ne laisse rien filer");

    let (retard, glissade) = jouer(false);
    assert!(retard > 1.0, "à l'envers, le doigt traîne de {retard} px");
    assert!(
        glissade > 100.0,
        "et file encore de {glissade} px après le lâcher"
    );
}

/// **Un dixième de pixel demandé à la souris est un dixième de pixel montré.** Au doigt, une
/// dette sous le demi-pixel se solde ; à la souris, ce serait une dérive sous le curseur.
#[test]
fn un_glisser_lent_ne_perd_rien() {
    let jouer = |souris: bool| {
        let mut elan = Elan::default();
        let mut t = Instant::now();
        let mut montre = 0.0;
        for _ in 0..40 {
            if souris {
                elan.placer_pan(0.1, 0.0);
            } else {
                elan.pousser_pan(0.1, 0.0, t);
            }
            t += IMAGE;
            montre += elan.avancer(t, IMAGE, DIAGONALE).map_or(0.0, |m| m.pan.0);
        }
        montre
    };
    let souris = jouer(true);
    assert!(
        (souris - 4.0).abs() < 1e-9,
        "quarante dixièmes, quatre pixels : {souris}"
    );
    let doigt = jouer(false);
    assert!(
        doigt < 3.0,
        "à l'envers, le doigt en perd : {doigt} px sur 4"
    );
}

/// **La souris arrête net la glissade du doigt**, comme un nouveau geste du doigt le fait :
/// la glissade était une prédiction, la souris vient de dire autre chose.
#[test]
fn la_souris_arrete_la_glissade_du_doigt() {
    let mut elan = Elan::default();
    let mut t = Instant::now();
    for _ in 0..20 {
        elan.pousser_pan(20.0, 0.0, t);
        t += Duration::from_millis(10);
    }
    // La main lâche : la glissade commence.
    montrer(&mut elan, t, 30);
    assert!(elan.en_cours(), "le doigt glisse encore");
    elan.placer_pan(5.0, 0.0);
    let t = t + IMAGE * 31;
    let m = elan
        .avancer(t, IMAGE, DIAGONALE)
        .expect("la souris se montre");
    assert_eq!(
        m.pan.0, 5.0,
        "ce que la souris demande, et rien de la glissade"
    );
    assert_eq!(montrer(&mut elan, t, 480), (0.0, 0.0), "et rien après");
}

/// **Le doigt qui reprend après la souris retrouve sa conduite et son élan** : la voie suit la
/// porte du dernier apport, elle ne se fige pas.
#[test]
fn le_doigt_retrouve_son_elan_apres_la_souris() {
    let mut elan = Elan::default();
    let mut t = Instant::now();
    elan.placer_zoom(CRAN, (0.0, 0.0));
    elan.avancer(t + IMAGE, IMAGE, DIAGONALE);
    for _ in 0..20 {
        elan.pousser_pan(20.0, 0.0, t);
        t += Duration::from_millis(10);
    }
    // Le doigt a demandé quatre cents pixels : la conduite les lisse, puis l'élan les prolonge.
    let premiere = elan
        .avancer(t + IMAGE, IMAGE, DIAGONALE)
        .map_or(0.0, |m| m.pan.0);
    assert!(
        premiere < 400.0,
        "la conduite lisse : {premiere} px d'un coup"
    );
    let total = premiere + montrer(&mut elan, t + IMAGE, 480).0;
    assert!(
        total > 500.0,
        "et l'élan prolonge le geste : {total} px pour 400"
    );
}
