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

/// Le pas que [`crate::horloge`] fournit à cette cadence.
///
/// **Il vient désormais du dehors, et c'est le fond de la correction.** L'élan mesurait son
/// propre temps entre deux appels, c'est-à-dire entre deux débuts de rendu ; l'écran, lui,
/// montre le résultat pendant l'intervalle qui sépare deux présentations. Les tests passent
/// donc le pas, comme la boucle le fait.
const PAS: Duration = Duration::from_nanos(4_166_667);

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
        if let Some(m) = elan.avancer(t, PAS, DIAGONALE) {
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
        if let Some(m) = elan.avancer(t, PAS, DIAGONALE) {
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
        if let Some(m) = elan.avancer(quand, PAS, DIAGONALE) {
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
        // Le pas est celui de CETTE cadence : c'est ce que l'horloge fournirait, et le passer
        // constant ferait mesurer « le trajet depend du nombre d'images », ce qui n'est pas la
        // question posee.
        let pas = Duration::from_secs_f64(dt);
        for n in 1..=images {
            let t = fin + Duration::from_secs_f64(dt * f64::from(n));
            if let Some(m) = elan.avancer(t, pas, DIAGONALE) {
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
    elan.avancer(plus_tard, PAS, DIAGONALE);
    elan.pousser_pan(3.0, 0.0, plus_tard);
    let total = jouer(&mut elan, plus_tard, 3.0);
    assert!(
        total.0 < 5.0,
        "trois pixels demandes, {:.1} montres : le geste precedent a deteint",
        total.0
    );
}

/// **L'élan ne décide plus rien sur la durée, et c'est voulu.**
///
/// Ce test disait l'inverse : une image très longue ne devait pas franchir toute la dette,
/// grâce à une borne de cent millisecondes posée ici. Elle se trompait de problème.
///
/// Si la boucle a été bloquée trois secondes — un dialogue natif, une fenêtre réduite — la
/// glissade **est** terminée : trois secondes ont réellement passé, et son amortissement l'a
/// éteinte depuis longtemps. La borne ne supprimait pas le saut ; elle le remplaçait par pire,
/// une glissade qui reprend au ralenti pendant trente images pour rattraper un retard que
/// personne n'attendait.
///
/// Le pas vient donc de [`crate::horloge`], qui ne borne rien non plus — et un pas de trois
/// secondes solde la dette, ce qui est le résultat exact.
#[test]
fn un_pas_de_trois_secondes_solde_la_dette_et_c_est_le_bon_resultat() {
    let mut elan = Elan::default();
    let debut = Instant::now();
    elan.pousser_pan(1000.0, 0.0, debut);
    let m = elan
        .avancer(
            debut + Duration::from_secs(3),
            Duration::from_secs(3),
            DIAGONALE,
        )
        .expect("il reste de la dette");
    assert!(
        m.pan.0 > 999.0,
        "trois secondes ont passe : la dette doit etre soldee, pas etalee ({})",
        m.pan.0
    );
    assert!(!elan.en_cours(), "et il ne reste plus rien a montrer");
}

/// **Le pas décide seul de ce qui avance**, quel que soit le temps écoulé par ailleurs.
///
/// C'est la propriété qui rend l'élan indépendant du coût du rendu : deux images séparées de
/// six millisecondes ou de soixante-sept montrent la même chose si l'écran a montré la même
/// chose. Un test le prouve plutôt qu'un commentaire.
#[test]
fn deux_images_de_couts_opposes_montrent_le_meme_mouvement() {
    let montre_pour = |cout_ms: u64| {
        let mut elan = Elan::default();
        let debut = Instant::now();
        elan.pousser_pan(500.0, 0.0, debut);
        elan.avancer(debut + Duration::from_millis(cout_ms), PAS, DIAGONALE)
            .map_or(0.0, |m| m.pan.0)
    };
    let rapide = montre_pour(6);
    let lente = montre_pour(67);
    assert!(
        (rapide - lente).abs() < 1e-9,
        "le cout du rendu ne doit plus entrer dans la trajectoire : {rapide} contre {lente}"
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
        if let Some(m) = elan.avancer(t, PAS, DIAGONALE) {
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
        assert_eq!(elan.avancer(t, PAS, DIAGONALE), None);
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

/// **À main régulière, la vue avance régulièrement.** C'est la propriété que l'utilisateur
/// jugeait absente — « ce n'est absolument pas fluide » — et qu'aucun test ne vérifiait.
///
/// Le défaut venait d'une constante de temps recalculée à chaque image depuis le nombre
/// d'événements reçus. Ce nombre oscille avec la livraison du pilote, donc la fraction montrée
/// oscillait avec lui, pour un geste qui, lui, ne changeait pas.
///
/// Le test compare chaque image à la précédente. Un geste régulier ne doit produire aucun
/// sursaut — et c'est bien un rapport qu'on mesure, pas une valeur : la vitesse elle-même a le
/// droit de monter et de descendre, à condition de le faire continûment.
#[test]
fn une_main_reguliere_ne_produit_aucun_sursaut() {
    let mut elan = Elan::default();
    let debut = Instant::now();

    // Un geste long et régulier, au rythme d'un pavé, pendant qu'on joue les images à 240 Hz.
    // Les deux rythmes ne tombent pas juste — c'est précisément le cas réel.
    let mut prochain_evenement = debut;
    let mut precedent: Option<f64> = None;
    let mut pire_rapport: f64 = 1.0;

    for n in 1..=240u32 {
        let t = debut + Duration::from_secs_f64(IMAGE * f64::from(n));
        while prochain_evenement <= t {
            elan.pousser_pan(10.0, 0.0, prochain_evenement);
            prochain_evenement += PAVE;
        }
        let montre = elan.avancer(t, PAS, DIAGONALE).map_or(0.0, |m| m.pan.0);
        // Les toutes premières images remplissent la dette : on regarde le régime établi.
        if n < 30 {
            continue;
        }
        if let Some(avant) = precedent {
            let (petit, grand) = if montre < avant {
                (montre, avant)
            } else {
                (avant, montre)
            };
            if petit > 0.01 {
                pire_rapport = pire_rapport.max(grand / petit);
            }
        }
        precedent = Some(montre);
    }

    assert!(
        pire_rapport < 1.6,
        "une image a montre {pire_rapport:.2} fois ce que la precedente montrait, \
         pour une main parfaitement reguliere"
    );
}

/// **La constante de temps ne dépend pas de ce que le pilote vient de livrer.** C'est la
/// formulation directe du défaut : le même déplacement, sur la même durée, doit montrer la
/// même chose — que le pilote l'ait livré en cinq morceaux ou en vingt.
///
/// Les deux gestes sont observés **au même instant**, un instant après la fin commune : sans
/// cela on comparerait deux états qui n'ont pas le même âge, et l'écart mesuré ne dirait rien
/// de la livraison.
#[test]
fn le_rythme_de_livraison_ne_change_pas_ce_qui_est_montre() {
    const GESTE: f64 = 0.1;
    let montre_pour = |combien: u32| {
        let mut elan = Elan::default();
        let debut = Instant::now();
        let pas = GESTE / f64::from(combien);
        for k in 0..combien {
            elan.pousser_pan(
                100.0 / f64::from(combien),
                0.0,
                debut + Duration::from_secs_f64(pas * f64::from(k)),
            );
        }
        // Le même instant pour les deux : la fin du geste, plus une image.
        elan.avancer(
            debut + Duration::from_secs_f64(GESTE + IMAGE),
            PAS,
            DIAGONALE,
        )
        .map_or(0.0, |m| m.pan.0)
    };

    let peu = montre_pour(5);
    let beaucoup = montre_pour(20);
    assert!(
        (peu - beaucoup).abs() < peu.max(beaucoup) * 0.02,
        "cinq evenements montrent {peu:.3} px, vingt en montrent {beaucoup:.3} —          pour le meme deplacement sur la meme duree"
    );
}
