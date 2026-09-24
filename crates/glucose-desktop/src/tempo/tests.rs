//! Ce que le tempo promet : des soumissions à intervalle constant, et un `k` qui suit la
//! machine sans trembler.

use super::*;

const P240: Duration = Duration::from_nanos(4_166_667);

/// Un tirage déterministe : un générateur congruentiel, pour que le test rejoue toujours la
/// même session sans dépendre de quoi que ce soit.
fn uniforme(graine: &mut u64) -> f64 {
    *graine = graine
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*graine >> 11) as f64 / (1u64 << 53) as f64
}

/// Rejoue une suite de rendus et rend les instants de soumission qui en résultent.
///
/// Le temps est simulé, dans l'ordre de la boucle réelle : l'image `n` commence quand la
/// présentation précédente a rendu la main, coûte `rendu`, attend ce que le tempo demande,
/// part, puis sa présentation coûte `presentation` avant que la suivante ne commence.
fn rejouer(tempo: &mut Tempo, rendus: &[Duration], presentation: Duration) -> Vec<Instant> {
    let mut t = Instant::now();
    let mut soumissions = Vec::with_capacity(rendus.len());
    for rendu in rendus {
        let pret = t + *rendu;
        let attente = tempo.attente_avant_de_soumettre(pret);
        let soumission = pret + attente;
        soumissions.push(soumission);
        t = soumission + presentation;
    }
    soumissions
}

fn intervalles(soumissions: &[Instant]) -> Vec<Duration> {
    soumissions.windows(2).map(|w| w[1] - w[0]).collect()
}

/// **Le cas du terrain.** Un rendu à 4,87 ms sur un écran à 4,17 : hier une image sur sept
/// occupait deux balayages ; le tempo les fait toutes en occuper deux, sans exception.
#[test]
fn test_un_rendu_juste_au_dessus_de_la_periode_se_cale_sur_deux_balayages() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let rendus = vec![Duration::from_micros(4_870); 60];
    let intervalles = intervalles(&rejouer(&mut tempo, &rendus, Duration::ZERO));
    // Les premières images ratent (k valait 1), les suivantes sont toutes à deux balayages.
    let regime = &intervalles[4..];
    for (i, d) in regime.iter().enumerate() {
        assert_eq!(
            *d,
            P240.saturating_mul(2),
            "l'image {i} n'occupe pas deux balayages : {d:?}"
        );
    }
    assert_eq!(tempo.balayages(), 2);
}

/// Un rendu qui tient dans la période reste à un balayage, sans attente inutile.
#[test]
fn test_un_rendu_rapide_reste_a_un_balayage() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let rendus = vec![Duration::from_micros(3_000); 30];
    let intervalles = intervalles(&rejouer(&mut tempo, &rendus, Duration::ZERO));
    for d in &intervalles {
        assert_eq!(*d, P240, "chaque image occupe un balayage : {d:?}");
    }
    assert_eq!(tempo.balayages(), 1);
}

/// **Aucune dérive.** Cent images à cadence constante finissent exactement cent intervalles
/// plus tard, à la nanoseconde : la grille se recale sur la cible atteinte, jamais sur
/// l'instant réel.
#[test]
fn test_la_grille_des_soumissions_ne_derive_pas() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let depart = Instant::now();
    tempo.attente_avant_de_soumettre(depart);
    let mut t = depart;
    for _ in 0..100 {
        // Le rendu prend un temps qui n'est pas un multiple de quoi que ce soit.
        let pret = t + Duration::from_micros(3_333);
        let attente = tempo.attente_avant_de_soumettre(pret);
        // L'horloge réelle a un peu de retard sur la cible : c'est le cas normal d'un sommeil.
        t = pret + attente + Duration::from_micros(40);
    }
    let attendu = depart + P240.saturating_mul(100);
    let derniere = tempo.derniere_soumission.expect("des soumissions");
    assert_eq!(
        derniere,
        attendu,
        "cent images plus tard, la grille a derive de {:?}",
        derniere.saturating_duration_since(attendu)
    );
}

/// **La grille ne suit pas la présentation.** Sur le terrain, `present()` enchaîne le
/// téléversement, l'acquisition, l'encodage et la remise au compositeur : de 1,5 à 5 ms,
/// jamais moins que la marge. La première version recalait la grille sur sa fin, et chaque
/// intervalle en portait la variation — l'écran montrait `k` ou `k + 1` balayages au hasard,
/// le décalage que la chronique a lu entre le tempo visé et les balayages observés.
///
/// Ici la présentation varie d'une image à l'autre, et les soumissions restent sur la grille
/// à la nanoseconde une fois `k` trouvé.
#[test]
fn test_une_presentation_lente_ne_recale_pas_la_grille() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let mut t = Instant::now();
    let mut graine = 7u64;
    let mut soumissions = Vec::new();
    for _ in 0..300 {
        let pret = t + Duration::from_micros(2_000);
        let attente = tempo.attente_avant_de_soumettre(pret);
        let soumission = pret + attente;
        soumissions.push(soumission);
        let presentation = Duration::from_micros(1_500 + (uniforme(&mut graine) * 3_500.0) as u64);
        t = soumission + presentation;
    }
    // Deux millisecondes de rendu et jusqu'à cinq de présentation : deux balayages.
    assert_eq!(tempo.balayages(), 2);
    let regime = &intervalles(&soumissions)[10..];
    for (i, d) in regime.iter().enumerate() {
        assert_eq!(
            *d,
            P240.saturating_mul(2),
            "l'intervalle {i} a suivi la présentation : {d:?}"
        );
    }
}

/// `k` redescend quand le rendu le permet — mais pas avant la fin de l'échantillon, pour ne
/// pas trembler.
#[test]
fn test_k_redescend_a_la_fin_de_l_echantillon_et_pas_avant() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    // Un rendu lent fait monter k à deux.
    let mut rendus = vec![Duration::from_micros(4_870); 5];
    // Puis le rendu tombe nettement sous une période.
    rendus.extend(std::iter::repeat_n(Duration::from_micros(2_000), 600));
    let mut t = Instant::now();
    let mut k_par_image = Vec::new();
    for rendu in &rendus {
        let pret = t + *rendu;
        let attente = tempo.attente_avant_de_soumettre(pret);
        t = pret + attente;
        k_par_image.push(tempo.balayages());
    }
    // Un raté isolé ne monte rien ; deux non plus -- ce n'est encore que la tolérance sur
    // l'échantillon entier. Le troisième la dépasse, et k monte.
    assert_eq!(k_par_image[2], 1, "deux rates ne montent pas k");
    assert_eq!(k_par_image[3], 2, "le troisieme rate fait monter k");
    // Deux cents images d'échantillon avant de pouvoir conclure : k tient au moins jusque-là,
    // et est redescendu bien avant la fin.
    assert!(
        k_par_image[3..200].iter().all(|k| *k == 2),
        "k est redescendu avant la fin de l'echantillon"
    );
    assert_eq!(
        *k_par_image.last().expect("des images"),
        1,
        "k n'est jamais redescendu"
    );
}

/// Un rendu qui alterne rapide et lent ne fait pas trembler `k` : il reste au cran que le
/// PIRE rendu impose.
#[test]
fn test_un_rendu_bimodal_ne_fait_pas_osciller_k() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let rendus: Vec<Duration> = (0..600)
        .map(|i| {
            if i % 3 == 0 {
                Duration::from_micros(4_500)
            } else {
                Duration::from_micros(2_500)
            }
        })
        .collect();
    let intervalles = intervalles(&rejouer(&mut tempo, &rendus, Duration::ZERO));
    // Les seuls changements admis sont ceux du demarrage, le temps que k trouve son cran ;
    // ensuite, plus aucun -- c'est cela, ne pas trembler.
    let dernier_changement = intervalles
        .windows(2)
        .rposition(|w| w[0] != w[1])
        .unwrap_or(0);
    assert!(
        dernier_changement < 10,
        "le tempo tremble : un changement d'intervalle a l'image {dernier_changement} sur 600"
    );
    assert_eq!(
        tempo.balayages(),
        2,
        "un tiers de rendus a 4,5 ms impose deux balayages"
    );
}

/// **Un pic isolé ne fait pas monter `k`.** C'est le défaut du terrain : une colonne de
/// tuiles à peindre par seconde bloquait le tempo à sept balayages, quarante-trois images par
/// seconde pour un rendu typique de cinq millisecondes.
#[test]
fn test_un_pic_par_seconde_ne_bloque_pas_k_en_haut() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    // Un rendu typique à 5 ms -- deux balayages -- et un pic à 27 ms toutes les 100 images.
    let rendus: Vec<Duration> = (0..1_200)
        .map(|i| {
            if i % 100 == 50 {
                Duration::from_micros(27_000)
            } else {
                Duration::from_micros(5_000)
            }
        })
        .collect();
    let soumissions = rejouer(&mut tempo, &rendus, Duration::ZERO);
    assert_eq!(
        tempo.balayages(),
        2,
        "le tempo doit se caler sur le rendu typique, pas sur le pic"
    );
    // Et il ne bouge plus une fois calé : chaque pic est UNE image irrégulière -- la sienne --
    // et jamais un changement de cran qui en ferait d'autres.
    let regime = &intervalles(&soumissions)[10..];
    let deux = P240.saturating_mul(2);
    let irregulieres = regime.iter().filter(|d| **d != deux).count();
    let pics = rendus[10..].iter().filter(|r| **r > deux).count();
    assert!(
        irregulieres <= pics,
        "{irregulieres} images irregulieres pour {pics} pics : le tempo a bouge autour du pic"
    );
}

/// Les rendus de la session du 20/09 sur 429 photos, tels que la chronique les a lus, la
/// présentation déduite : une médiane à 8 ms, un p90 à 11, un p99 à 20, un pire à 40. La
/// distribution est reconstruite par interpolation entre ces quantiles.
fn rendus_du_terrain(n: usize) -> Vec<Duration> {
    const QUANTILES: [(f64, f64); 6] = [
        (0.0, 4.0),
        (0.5, 8.0),
        (0.9, 11.0),
        (0.99, 20.0),
        (0.999, 40.0),
        (1.0, 40.0),
    ];
    let mut graine = 0x9E37_79B9_7F4A_7C15u64;
    (0..n)
        .map(|_| {
            let u = uniforme(&mut graine);
            let (a, b) = QUANTILES
                .windows(2)
                .map(|w| (w[0], w[1]))
                .find(|(_, b)| u <= b.0)
                .unwrap_or((QUANTILES[4], QUANTILES[5]));
            let t = if b.0 > a.0 {
                (u - a.0) / (b.0 - a.0)
            } else {
                0.0
            };
            Duration::from_secs_f64((a.1 + t * (b.1 - a.1)) / 1000.0)
        })
        .collect()
}

/// **La session du terrain, rejouée : `k` ne doit pas trembler.**
///
/// La chronique du 20/09 lisait « le tempo, sur 4742 images en mouvement : 4 balayages 42 %,
/// 3 balayages 28 %, 5 balayages 27 % » — trois valeurs, aucune dominante. Un changement de
/// `k` est une image irrégulière au même titre qu'un raté ; le tempo n'a donc le droit d'en
/// produire que sous la même tolérance, une pour cent, une fois calé.
///
/// Ce que ce test ne peut pas exiger, et il faut le dire : UN cran dominant. Le p99 de cette
/// distribution tombe sur une frontière de `k`, et à la frontière exacte deux crans se
/// partagent le temps par nature -- c'est le bruit d'un comptage sur deux cents images, pas
/// un tremblement. La version à l'horizon d'une seconde donnait 47 changements ici ; celle-ci
/// en donne 14.
#[test]
fn test_la_distribution_du_terrain_ne_fait_pas_trembler_k() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let rendus = rendus_du_terrain(6_000);
    let mut t = Instant::now();
    let mut k_par_image = Vec::with_capacity(rendus.len());
    for rendu in &rendus {
        let pret = t + *rendu;
        let attente = tempo.attente_avant_de_soumettre(pret);
        t = pret + attente + Duration::from_micros(2_000);
        k_par_image.push(tempo.balayages());
    }
    // Le calage : le temps que k trouve son cran. Au-delà, chaque changement se compte.
    let regime = &k_par_image[1_000..];
    let changements = regime.windows(2).filter(|w| w[0] != w[1]).count();
    let toleres = regime.len() / 100;
    let mut parts = std::collections::BTreeMap::new();
    for k in regime {
        *parts.entry(*k).or_insert(0usize) += 1;
    }
    assert!(
        changements <= toleres,
        "le tempo tremble : {changements} changements de k sur {} images (tolérés : \
         {toleres}) ; répartition {parts:?}",
        regime.len()
    );
}

/// Sans période connue, le tempo ne retient rien : il n'invente pas de grille.
#[test]
fn test_sans_periode_aucune_attente() {
    let mut tempo = Tempo::nouveau();
    let t = Instant::now();
    tempo.attente_avant_de_soumettre(t);
    let attente = tempo.attente_avant_de_soumettre(t + Duration::from_millis(1));
    assert_eq!(attente, Duration::ZERO);
    assert!(tempo.derniere_soumission.is_none());
}

/// **La cible de la finesse ne dérive pas avec le tempo.**
///
/// C'est la correction d'un cercle que la mesure a attrapé en une session : viser un cran
/// sous ce que le tempo tient faisait monter la cible avec `k`, donc dégrader moins, donc
/// monter `k` encore — jusqu'à neuf balayages et quarante-trois images par seconde. La cible
/// est désormais le plus grand nombre entier de balayages qui tienne le plancher de la
/// charte, et rien ne la fait bouger.
#[test]
fn test_la_cible_de_la_finesse_ne_derive_pas_avec_le_tempo() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    // À 240 Hz, dix millisecondes valent deux balayages pleins : 8,33 ms.
    let deux_balayages = P240.saturating_mul(2);
    assert_eq!(tempo.cible_pour_descendre(), deux_balayages);

    // Un rendu lent fait monter k -- la cible, elle, ne bouge pas d'une nanoseconde.
    let rendus = vec![Duration::from_micros(30_000); 400];
    rejouer(&mut tempo, &rendus, Duration::ZERO);
    assert!(
        tempo.balayages() >= 7,
        "k doit avoir monte : {}",
        tempo.balayages()
    );
    assert_eq!(
        tempo.cible_pour_descendre(),
        deux_balayages,
        "la cible a suivi k : c'est le cercle qu'on vient de casser"
    );

    // Sur un écran de 60 Hz, un seul balayage tient dans le plancher : on ne vise pas
    // l'impossible, et surtout pas zéro.
    let mut lent = Tempo::nouveau();
    lent.accorder(Duration::from_nanos(16_666_667));
    assert_eq!(
        lent.cible_pour_descendre(),
        Duration::from_nanos(16_666_667)
    );

    // Sans période connue, le plancher de la charte reste seul : c'est ce qu'il a toujours
    // voulu dire.
    let muet = Tempo::nouveau();
    assert_eq!(muet.cible_pour_descendre(), crate::cadence::BUDGET_TOTAL);
}

/// **Le tempo dit quelle image a raté son balayage** (TEMPO-2) : celle qui arrive après sa
/// cible, et elle seule — la suivante, à l'heure, ne l'a pas raté.
#[test]
fn test_le_tempo_dit_quelle_image_a_rate_son_balayage() {
    let periode = Duration::from_micros(4_167);
    let mut tempo = Tempo::nouveau();
    tempo.accorder(periode);
    let debut = Instant::now();
    tempo.attente_avant_de_soumettre(debut);
    assert!(!tempo.a_rate(), "la premiere image pose la grille");
    // Cinq periodes plus tard, pour un tempo d'un balayage : ratee.
    let tard = debut + periode * 5;
    tempo.attente_avant_de_soumettre(tard);
    assert!(tempo.a_rate(), "l'image en retard a rate son balayage");
    // La suivante arrive avant sa cible : a l'heure.
    tempo.attente_avant_de_soumettre(tard + periode / 2);
    assert!(!tempo.a_rate(), "l'image a l'heure n'a rien rate");
}
