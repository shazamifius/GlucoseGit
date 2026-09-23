//! Ce que le rythme doit savoir dire — et le piège à ne pas retomber dedans.
//!
//! # Un test qui construit son propre résultat ne prouve rien
//!
//! Un test de ce dépôt écrivait `10.0 + d - d` : le décalage s'annulait, l'assertion était
//! vraie par construction, et elle a laissé passer le défaut qu'elle prétendait couvrir.
//!
//! Ici le piège serait de fabriquer les intervalles **depuis** les pas de temps. Les tests qui
//! suivent posent donc les deux indépendamment — un scénario de machine d'un côté, un scénario
//! de mouvement de l'autre — exactement comme la réalité les produit.

use super::*;

/// Une machine parfaitement régulière : chaque image coûte la même chose.
///
/// Le mouvement s'intègre alors sur le même pas que celui pendant lequel il est montré, et
/// c'est le seul cas où l'œil voit exactement la trajectoire calculée.
#[test]
fn test_une_machine_reguliere_ne_produit_aucun_saut() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    let periode = Duration::from_micros(8_333);
    for _ in 0..200 {
        t += periode;
        r.presentee(t, t - Duration::from_micros(2_000), periode, 1_000.0, true);
    }
    let (median, _, _, _) = r.intervalles();
    assert!(
        (8_000..=10_000).contains(&median),
        "l'intervalle median doit valoir la periode de la boucle : {median}"
    );
    assert_eq!(
        r.sauts().2,
        0,
        "aucun saut : le temps montre est le temps integre"
    );
    let (mediane, pire) = r.fidelite().expect("le mouvement a ete observe");
    assert!(
        (0.95..=1.05).contains(&mediane),
        "la fidelite d'une machine reguliere vaut un : {mediane}"
    );
    assert!(
        (0.95..=1.05).contains(&pire),
        "et elle ne s'ecarte jamais : {pire}"
    );
    assert_eq!(
        r.irregularite(),
        Some(0.0),
        "chaque image occupe le meme nombre de balayages"
    );
}

/// **Le test qui démontre la thèse de ce module.**
///
/// Le mouvement est parfait : la caméra avance de la durée exacte qui sépare deux débuts de
/// rendu, à vitesse rigoureusement constante. La machine, elle, alterne une image rapide et
/// une image lente — ce que la chronique de terrain montre (6 ms en médiane, 67 ms au pire).
///
/// Aucune mesure de coût ne verrait quoi que ce soit : les durées sont ce qu'elles sont, la
/// vitesse est constante, la cadence est excellente. Ce module, lui, doit voir le tressaut.
#[test]
fn test_une_machine_irreguliere_fait_sauter_un_mouvement_pourtant_parfait() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");

    // Le scénario de la machine : les durées de rendu, alternées.
    let durees = [Duration::from_micros(6_000), Duration::from_micros(30_000)];
    let vitesse = 1_000.0;

    // On rejoue la boucle telle qu'elle est écrite : le pas d'intégration court d'un début de
    // rendu au suivant, la présentation a lieu à la fin. Les deux ne coïncident pas.
    let mut debut = Instant::now();
    let mut debut_precedent: Option<Instant> = None;
    for i in 0..200 {
        let duree = durees[i % durees.len()];
        let pas = debut_precedent.map_or(Duration::ZERO, |avant| {
            debut.saturating_duration_since(avant)
        });
        let presentation = debut + duree;
        if !pas.is_zero() {
            r.presentee(presentation, debut, pas, vitesse, true);
        }
        debut_precedent = Some(debut);
        debut = presentation;
    }

    let (_, _, pire) = r.sauts();
    // Les deux durées diffèrent de 24 ms ; à mille pixels par seconde, cela fait vingt-quatre
    // pixels d'écart entre où le contenu est montré et où il devrait être.
    assert!(
        pire >= 20,
        "le saut de position doit se voir, et il vaut la variation de duree fois la vitesse : {pire} px"
    );
    let avance = r.avance_mediane();
    assert!(
        pire > avance,
        "le saut ({pire} px) doit depasser l'avance attendue ({avance} px) : c'est ce qui fait \
         reculer le contenu"
    );
    let irreguliere = r.irregularite().expect("des images ont ete comparees");
    assert!(
        irreguliere > 0.5,
        "une image sur deux change de nombre de balayages : {irreguliere}"
    );
    let (_, pire_fidelite) = r.fidelite().expect("le mouvement a ete observe");
    assert!(
        pire_fidelite < 0.6,
        "sur l'image lente, le contenu n'avance que d'une fraction de ce que sa duree \
         d'affichage demandait : {pire_fidelite}"
    );
}

/// Une vue immobile n'a pas de fidélité : il n'y a pas de mouvement à montrer.
#[test]
fn test_une_vue_immobile_ne_compte_pas_dans_la_fidelite() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    for _ in 0..50 {
        t += Duration::from_micros(40_000);
        r.presentee(
            t,
            t - Duration::from_micros(2_000),
            Duration::from_micros(8_000),
            0.0,
            true,
        );
    }
    assert_eq!(
        r.fidelite(),
        None,
        "aucune image ne bougeait : la fidelite n'a rien mesure"
    );
    assert!(
        r.intervalles().0 > 0,
        "les intervalles se comptent quand meme : ils disent le rythme de la boucle"
    );
}

/// La cadence vue est celle des images consécutives, jamais « images ÷ durée ».
#[test]
fn test_la_cadence_vue_ignore_le_temps_ou_rien_n_etait_demande() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    // Cent images à cent par seconde, d'affilée.
    for _ in 0..100 {
        t += Duration::from_micros(10_000);
        r.presentee(
            t,
            t - Duration::from_micros(2_000),
            Duration::from_micros(10_000),
            100.0,
            true,
        );
    }
    // Puis un long sommeil, et une seule image.
    t += Duration::from_secs(10);
    r.presentee(
        t,
        t - Duration::from_micros(2_000),
        Duration::from_micros(10_000),
        100.0,
        true,
    );

    let vue = r.cadence_vue().expect("des intervalles ont ete mesures");
    assert!(
        (80.0..=130.0).contains(&vue),
        "la cadence vue doit rester celle des images consecutives : {vue}"
    );
}

/// Sans période d'écran connue, le module ne raconte rien sur les balayages.
#[test]
fn test_sans_periode_connue_aucun_balayage_n_est_invente() {
    let mut r = Rythme::nouveau();
    let mut t = Instant::now();
    for _ in 0..20 {
        t += Duration::from_micros(10_000);
        r.presentee(
            t,
            t - Duration::from_micros(2_000),
            Duration::from_micros(10_000),
            500.0,
            true,
        );
    }
    assert_eq!(r.periode(), None);
    assert_eq!(r.irregularite(), None, "rien a comparer sans periode");
    assert!(
        r.periodes_occupees().is_empty(),
        "aucun balayage ne se compte quand l'ecran n'a rien annonce"
    );
    assert!(
        r.fidelite().is_some(),
        "la fidelite, elle, ne depend pas de l'ecran"
    );
}

/// La première image n'a rien à comparer : elle ne doit pas inventer un intervalle.
#[test]
fn test_la_premiere_image_ne_mesure_rien() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let t = Instant::now();
    let m = r.presentee(
        t,
        t - Duration::from_micros(2_000),
        Duration::from_micros(8_000),
        900.0,
        true,
    );
    assert_eq!(m, Mesure::default());
    assert_eq!(r.comparees(), 0);
}

/// **Un sommeil n'est pas un gel.** Une application qui n'a rien a faire dort, et l'intervalle
/// qui suit ne dit rien de ce que l'oeil a recu -- personne ne regardait un mouvement.
#[test]
fn test_un_sommeil_ne_compte_pas_comme_un_gel() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    for _ in 0..20 {
        let due = t;
        t += Duration::from_micros(8_333);
        r.en_retard(t, t - Duration::from_micros(2_000), Some(due));
        r.presentee(
            t,
            t - Duration::from_micros(2_000),
            Duration::from_micros(8_333),
            500.0,
            true,
        );
    }
    // Trois secondes de sommeil, puis une image que rien n'attendait -- sinon le geste qui
    // l'a demandee, trois millisecondes avant son rendu (GEL-1).
    t += Duration::from_secs(3);
    r.en_retard(
        t,
        t - Duration::from_micros(2_000),
        Some(t - Duration::from_millis(3)),
    );
    r.presentee(
        t,
        t - Duration::from_micros(2_000),
        Duration::ZERO,
        0.0,
        false,
    );
    let (_, _, _, pire) = r.intervalles();
    assert!(
        pire < 20_000,
        "le sommeil de trois secondes ne doit pas paraitre dans les intervalles : {pire} us"
    );
    let (_, gel, _) = r.pire_intervalle();
    assert!(
        gel < Duration::from_millis(20),
        "ni comme le pire gel : {gel:?}"
    );
}

/// **Un dialogue n'est pas un gel non plus** — et lui, l'image qui le suit L'ATTENDAIT : un
/// décodage en cours, un toast, un geste posé juste avant. Sans l'oubli, une ouverture de
/// fichier se lisait « le pire gel : 19 836 ms à la 21,2e seconde ».
#[test]
fn test_un_dialogue_ne_compte_pas_comme_un_gel() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    for _ in 0..20 {
        let due = t;
        t += Duration::from_micros(8_333);
        r.en_retard(t, t - Duration::from_micros(2_000), Some(due));
        r.presentee(
            t,
            t - Duration::from_micros(2_000),
            Duration::from_micros(8_333),
            500.0,
            true,
        );
    }
    // Vingt secondes à choisir un fichier, puis une image que le décodage attendait.
    r.oublier();
    let demandee_avant_le_dialogue = t;
    t += Duration::from_secs(20);
    r.en_retard(
        t,
        t - Duration::from_micros(2_000),
        Some(demandee_avant_le_dialogue),
    );
    r.presentee(
        t,
        t - Duration::from_micros(2_000),
        Duration::from_micros(8_333),
        0.0,
        true,
    );
    let (_, _, _, pire) = r.intervalles();
    assert!(
        pire < 20_000,
        "le dialogue ne doit pas paraitre dans les intervalles : {pire} us"
    );
    let (_, gel, _) = r.pire_intervalle();
    assert!(
        gel < Duration::from_millis(20),
        "ni comme le pire gel : {gel:?}"
    );
    // Et la mesure reprend dès l'image d'après.
    t += Duration::from_micros(8_333);
    r.presentee(
        t,
        t - Duration::from_micros(2_000),
        Duration::from_micros(8_333),
        500.0,
        true,
    );
    let (_, _, _, pire) = r.intervalles();
    assert!(
        (8_000..=10_000).contains(&pire),
        "la mesure a repris : {pire} us"
    );
}

/// **GEL-1** — un gel se compte depuis l'instant où l'image était due, et seulement lui.
///
/// Deux cas qui se ressemblent dans un intervalle et n'ont rien à voir : une pause de cinq
/// secondes que termine un geste, et une image due que rien ne rend pendant sept dixièmes de
/// seconde. Le premier était compté comme un gel de cinq secondes ; le second est le seul
/// que l'œil voie figé.
#[test]
fn test_un_gel_se_compte_depuis_l_echeance_de_l_image() {
    let mut r = Rythme::nouveau();
    r.observer_la_machine(Duration::from_micros(4_166), "Fifo");
    let mut t = Instant::now();
    r.presentee(t, t, Duration::ZERO, 0.0, true);

    // Cinq secondes de pause, puis un geste : l'image est due au geste.
    let geste = t + Duration::from_secs(5);
    t = geste + Duration::from_millis(4);
    r.en_retard(t, geste + Duration::from_millis(1), Some(geste));
    r.presentee(
        t,
        geste + Duration::from_millis(1),
        Duration::ZERO,
        0.0,
        true,
    );
    let (_, apres_la_pause, _) = r.pire_intervalle();
    assert!(
        apres_la_pause < Duration::from_millis(20),
        "une pause terminee par un geste n'est pas un gel : {apres_la_pause:?}"
    );

    // Puis une image due tout de suite, que le rendu ne commence que 700 ms plus tard.
    let due = t;
    t += Duration::from_millis(705);
    r.en_retard(t, t - Duration::from_millis(5), Some(due));
    r.presentee(t, t - Duration::from_millis(5), Duration::ZERO, 0.0, true);
    let (_, gel, dont_attente) = r.pire_intervalle();
    assert_eq!(gel, Duration::from_millis(705));
    assert_eq!(dont_attente, Duration::from_millis(700));
    let (perdu, _) = r.perdu_a_ne_pas_dessiner();
    assert_eq!(
        perdu,
        Duration::from_millis(690),
        "le temps perdu au-dela du plancher, et lui seul"
    );
}
