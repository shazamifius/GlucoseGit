//! Ce que le tempo promet : des soumissions à intervalle constant, et un `k` qui suit la
//! machine sans trembler.

use super::*;

const P240: Duration = Duration::from_nanos(4_166_667);

/// Rejoue une suite de rendus et rend les intervalles de soumission qui en résultent.
///
/// Le temps est simulé : l'image `n` commence à la soumission précédente, coûte `rendu`, et
/// part après l'attente que le tempo demande.
fn rejouer(tempo: &mut Tempo, rendus: &[Duration]) -> Vec<Duration> {
    let mut t = Instant::now();
    tempo.soumise(t);
    let mut soumissions = vec![t];
    for rendu in rendus {
        let pret = t + *rendu;
        let attente = tempo.attente_avant_de_soumettre(pret, *rendu);
        t = pret + attente;
        tempo.soumise(t);
        soumissions.push(t);
    }
    soumissions.windows(2).map(|w| w[1] - w[0]).collect()
}

/// **Le cas du terrain.** Un rendu à 4,87 ms sur un écran à 4,17 : hier une image sur sept
/// occupait deux balayages ; le tempo les fait toutes en occuper deux, sans exception.
#[test]
fn test_un_rendu_juste_au_dessus_de_la_periode_se_cale_sur_deux_balayages() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    let rendus = vec![Duration::from_micros(4_870); 60];
    let intervalles = rejouer(&mut tempo, &rendus);
    // La première image rate (k valait 1), les suivantes sont toutes à deux balayages.
    let regime = &intervalles[2..];
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
    let intervalles = rejouer(&mut tempo, &rendus);
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
    tempo.soumise(depart);
    let mut t = depart;
    for _ in 0..100 {
        // Le rendu prend un temps qui n'est pas un multiple de quoi que ce soit.
        let pret = t + Duration::from_micros(3_333);
        let attente = tempo.attente_avant_de_soumettre(pret, Duration::from_micros(3_333));
        // L'horloge réelle a un peu de retard sur la cible : c'est le cas normal d'un sommeil.
        t = pret + attente + Duration::from_micros(40);
        tempo.soumise(t);
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

/// `k` redescend quand le rendu le permet — mais pas avant l'horizon, pour ne pas trembler.
#[test]
fn test_k_redescend_apres_l_horizon_et_pas_avant() {
    let mut tempo = Tempo::nouveau();
    tempo.accorder(P240);
    // Un rendu lent fait monter k à deux.
    let mut rendus = vec![Duration::from_micros(4_870); 5];
    // Puis le rendu tombe nettement sous une période.
    rendus.extend(std::iter::repeat_n(Duration::from_micros(2_000), 400));
    let mut t = Instant::now();
    tempo.soumise(t);
    let mut k_par_image = Vec::new();
    for rendu in &rendus {
        let pret = t + *rendu;
        let attente = tempo.attente_avant_de_soumettre(pret, *rendu);
        t = pret + attente;
        tempo.soumise(t);
        k_par_image.push(tempo.balayages());
    }
    // Le premier raté ne monte rien -- un raté isolé est une image irrégulière, et c'est
    // tout. Le deuxième fait monter k.
    assert_eq!(k_par_image[0], 1, "un rate isole ne monte pas k");
    assert_eq!(k_par_image[1], 2, "le deuxieme rate fait monter k");
    // À deux balayages par image, une seconde vaut cent vingt images : k doit tenir au moins
    // jusque-là, et être redescendu bien avant la fin.
    assert!(
        k_par_image[1..100].iter().all(|k| *k == 2),
        "k est redescendu avant l'horizon"
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
    let intervalles = rejouer(&mut tempo, &rendus);
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
    rejouer(&mut tempo, &rendus);
    assert_eq!(
        tempo.balayages(),
        2,
        "le tempo doit se caler sur le rendu typique, pas sur le pic"
    );
}

/// Sans période connue, le tempo ne retient rien : il n'invente pas de grille.
#[test]
fn test_sans_periode_aucune_attente() {
    let mut tempo = Tempo::nouveau();
    let t = Instant::now();
    tempo.soumise(t);
    let attente =
        tempo.attente_avant_de_soumettre(t + Duration::from_millis(1), Duration::from_millis(1));
    assert_eq!(attente, Duration::ZERO);
}
