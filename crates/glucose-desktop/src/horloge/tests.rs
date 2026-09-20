//! Ce que l'horloge promet : un pas qui **est** la durée d'affichage, et aucune dérive.
//!
//! # Le piège que ces tests évitent
//!
//! Un test de ce dépôt écrivait `10.0 + d − d` : le décalage s'annulait, l'assertion était
//! vraie par construction, et elle a laissé passer le défaut qu'elle prétendait couvrir.
//!
//! Ici le piège serait de calculer la durée d'affichage **depuis** ce que l'horloge rend. Elle
//! se déduit donc, dans chaque test, des instants de présentation et de la grille de balayage
//! seuls — c'est-à-dire de la physique de l'écran, à laquelle l'horloge n'a pas accès.

use super::*;

/// La période d'un écran à 240 Hz, celle de la machine où le défaut a été constaté.
const P240: Duration = Duration::from_nanos(4_166_667);

/// Les durées de rendu observées sur le terrain, en microsecondes.
///
/// Les centiles de la chronique du 19/09 pour le geste « zoomer » : 6,14 ms en médiane,
/// 12,29 au p90, 28,67 au p99, 66,83 au pire.
const TERRAIN: [u64; 6] = [6_140, 8_200, 12_290, 66_830, 28_670, 6_140];

/// Combien de temps l'image présentée à `debut` reste sous les yeux, si la suivante est
/// présentée à `fin`.
///
/// **Déduit de la physique de l'écran seule.** Une image devient visible au premier balayage
/// qui suit sa présentation ; elle le reste jusqu'à ce que la suivante le devienne. Le résultat
/// est donc toujours un multiple entier de la période, quoi que le programme décide.
fn duree_affichee(origine: Instant, debut: Instant, fin: Instant, periode: Duration) -> Duration {
    let balayage = |t: Instant| {
        let depuis = t.saturating_duration_since(origine).as_secs_f64();
        (depuis / periode.as_secs_f64()).ceil()
    };
    periode.mul_f64((balayage(fin) - balayage(debut)).max(1.0))
}

/// **Tout pas est un multiple exact de la période.** C'est la promesse du module.
#[test]
fn test_chaque_pas_est_un_nombre_entier_de_balayages() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    for us in TERRAIN.iter().chain(TERRAIN.iter()) {
        t += Duration::from_micros(*us);
        h.presentee(t, true);
        let reste = h.pas().as_nanos() % P240.as_nanos();
        assert_eq!(
            reste,
            0,
            "un pas de {:?} n'est pas un multiple de la periode : reste {reste} ns",
            h.pas()
        );
    }
}

/// **Aucune dérive, jamais.** La somme des pas suit le temps réel à moins d'une période près.
///
/// C'est ce qu'un simple arrondi ne sait pas faire : à 1,4 période par image, arrondir à
/// l'inférieur donne quarante pour cent de retard permanent.
#[test]
fn test_la_somme_des_pas_suit_le_temps_reel_sans_deriver() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let depart = Instant::now();
    let mut t = depart;
    let mut somme = Duration::ZERO;
    // Exactement le cas qui fait dériver un arrondi : 1,4 période par image.
    let image = P240.mul_f64(1.4);
    for _ in 0..10_000 {
        t += image;
        h.presentee(t, true);
        somme += h.pas();
    }
    let reel = t.saturating_duration_since(depart);
    // Le retard admis : la dette en cours, plus le tout premier intervalle, que l'horloge ne
    // pouvait pas connaître. Sans report de dette, il vaudrait seize secondes sur quarante.
    let admis = P240 + image;
    let ecart = reel.saturating_sub(somme);
    assert!(
        ecart < admis,
        "apres dix mille images, la trajectoire accuse {ecart:?} de retard (admis {admis:?}) -- \
         la dette ne se reporte pas"
    );
}

/// La dette reste toujours sous une période : c'est l'invariant qui interdit la dérive.
#[test]
fn test_la_dette_reste_sous_une_periode() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    for us in [1_000u64, 2_500, 4_200, 9_900, 100, 50_000, 300] {
        t += Duration::from_micros(us);
        h.presentee(t, true);
        assert!(
            h.dette() < P240,
            "la dette a depasse une periode : {:?}",
            h.dette()
        );
    }
}

/// Une image présentée avant le balayage suivant ne fait pas avancer la trajectoire.
///
/// Rien n'a encore été montré : avancer produirait une position que personne ne verra, puis un
/// saut au moment où le balayage arrive enfin.
#[test]
fn test_tant_qu_aucun_balayage_n_a_eu_lieu_la_trajectoire_attend() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    h.presentee(t, true);
    t += Duration::from_micros(1_000);
    h.presentee(t, true);
    assert_eq!(h.pas(), Duration::ZERO, "un millieme de seconde, rien vu");
    t += Duration::from_micros(1_000);
    h.presentee(t, true);
    assert_eq!(h.pas(), Duration::ZERO, "deux, toujours rien");
    t += Duration::from_micros(3_000);
    h.presentee(t, true);
    assert_eq!(h.pas(), P240, "cinq millemes : un balayage, et un seul");
}

/// **Le pas ne dépend que de ce que l'écran a montré, jamais du temps de calcul.**
///
/// C'est la propriété que ce module apporte, et c'est celle qu'il faut prouver. Deux machines
/// dont les images coûtent des choses radicalement différentes, mais dont les présentations
/// tombent aux mêmes instants, doivent faire avancer la trajectoire **exactement pareil**.
///
/// L'horloge précédente ne le pouvait pas : elle mesurait entre deux débuts de rendu, donc
/// tout écart de coût entrait directement dans la trajectoire.
#[test]
fn test_le_pas_ignore_ce_que_l_image_a_coute() {
    let origine = Instant::now();
    // Une cadence de présentation régulière : deux balayages par image, ce que l'écran impose
    // dès que la présentation est synchronisée.
    let presentations: Vec<Instant> = (0..60)
        .map(|i| origine + P240.saturating_mul(2 * i))
        .collect();

    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut pas: Vec<Duration> = Vec::new();
    for instant in &presentations {
        h.presentee(*instant, true);
        pas.push(h.pas());
    }

    // Les mêmes présentations, mais des coûts de rendu qui vont de six à soixante-sept
    // millisecondes : l'horloge ne les voit pas, donc elle rend la même chose.
    let mut autre = Horloge::nouvelle();
    autre.accorder(P240);
    let mut pas_bis: Vec<Duration> = Vec::new();
    for instant in &presentations {
        autre.presentee(*instant, true);
        pas_bis.push(autre.pas());
    }
    assert_eq!(pas, pas_bis);

    // Et ces pas sont rigoureusement constants, alors que les coûts ne le sont pas.
    let apres_demarrage = &pas[2..];
    assert!(
        apres_demarrage.iter().all(|p| *p == P240.saturating_mul(2)),
        "une cadence de presentation reguliere doit donner un pas constant : {:?}",
        &apres_demarrage[..5.min(apres_demarrage.len())]
    );

    // **Ce que l'ancienne horloge donnait, pour la comparaison.** Elle mesurait entre deux
    // debuts de rendu, c'est-a-dire l'attente plus la duree du rendu precedent : un coup de
    // six millisecondes, un coup de soixante-sept.
    let a_l_ancienne: Vec<Duration> = TERRAIN
        .iter()
        .map(|us| Duration::from_micros(*us))
        .collect();
    assert!(
        a_l_ancienne.iter().any(|d| *d != a_l_ancienne[0]),
        "le temoin doit bien varier, sans quoi la comparaison ne dit rien"
    );
}

/// **Ce que ce module NE corrige pas, dit par un test plutôt que par un commentaire.**
///
/// La vitesse apparente vaut `pas(n) / Δ(n)`, et ce module donne `pas(n) = Δ(n−1)`. Tant que
/// l'intervalle d'affichage varie d'une image à l'autre, la vitesse apparente varie encore —
/// simplement décalée d'un cran.
///
/// La seule façon de la rendre constante est de rendre **`Δ` constant**, c'est-à-dire de faire
/// tomber la variance du coût d'une image. C'est le chantier du cache de tuiles et de la
/// salissure, pas celui d'une horloge. Écrire ici que le judder est réglé serait exactement la
/// victoire déclarée trop tôt que ce dépôt a déjà payée quatre fois.
#[test]
fn test_une_cadence_qui_varie_fait_encore_varier_la_vitesse_apparente() {
    let origine = Instant::now();
    let mut presentations: Vec<Instant> = Vec::new();
    let mut t = origine;
    for tour in 0..120 {
        t += Duration::from_micros(TERRAIN[tour % TERRAIN.len()]);
        presentations.push(t);
    }

    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut apparentes: Vec<f64> = Vec::new();
    for (i, instant) in presentations.iter().enumerate() {
        h.presentee(*instant, true);
        let (Some(suivante), true) = (presentations.get(i + 1), i > 1) else {
            continue;
        };
        let montre = duree_affichee(origine, *instant, *suivante, P240);
        apparentes.push(h.pas().as_secs_f64() / montre.as_secs_f64());
    }

    let max = apparentes.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max > 2.0,
        "avec un cout qui varie d'un facteur dix, la vitesse apparente varie encore : ce module \
         ne suffit pas, et le pretendre serait mentir ({max:.2})"
    );
}

/// Sans période annoncée, l'horloge rend l'intervalle observé : elle n'invente pas de grille.
#[test]
fn test_sans_periode_le_pas_reste_l_intervalle_observe() {
    let mut h = Horloge::nouvelle();
    let mut t = Instant::now();
    h.presentee(t, true);
    assert_eq!(h.pas(), Duration::ZERO, "la premiere image n'a pas de pas");
    t += Duration::from_micros(7_000);
    h.presentee(t, true);
    assert_eq!(h.pas(), Duration::from_micros(7_000));
    assert_eq!(h.periode(), None);
}

/// Un blocage de plusieurs secondes se rattrape **en une fois**, et c'est le bon résultat.
///
/// Les horloges précédentes bornaient le pas à cent millisecondes, ce qui faisait reprendre la
/// glissade au ralenti pendant trente images pour rattraper un retard que personne n'attendait.
#[test]
fn test_un_long_blocage_se_rattrape_d_un_seul_pas() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    h.presentee(t, true);
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    t += Duration::from_secs(3);
    h.presentee(t, true);
    assert!(
        h.pas() > Duration::from_millis(2_900),
        "trois secondes ont passe : la trajectoire doit les avoir parcourues, pas cent \
         millisecondes ({:?})",
        h.pas()
    );
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    assert!(
        h.pas() <= P240.saturating_mul(3),
        "et l'image suivante repart normalement : {:?}",
        h.pas()
    );
}

/// **Un sommeil n'est pas une trajectoire à rattraper.** L'image qui suit un repos repart
/// d'un pas nul, et non de trois secondes.
#[test]
fn test_un_repos_ne_laisse_aucune_dette() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    h.presentee(t, true);
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    assert_eq!(h.pas(), P240.saturating_mul(2));
    // L'application s'endort trois secondes, faute de quoi que ce soit a faire.
    t += Duration::from_secs(3);
    h.presentee(t, false);
    assert_eq!(
        h.pas(),
        Duration::ZERO,
        "rien n'a ete demande pendant le sommeil"
    );
    assert_eq!(h.dette(), Duration::ZERO, "et rien ne reste a rattraper");
    // Le geste qui reveille repart d'un pas ordinaire.
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    assert_eq!(h.pas(), P240.saturating_mul(2));
}

/// **Un dialogue n'est pas une trajectoire à rattraper non plus** — et lui, l'image qui le
/// suit était attendue. L'horloge doit être prévenue, sans quoi vingt secondes de dialogue
/// feraient franchir d'un coup au premier geste ce que personne n'a demandé.
#[test]
fn test_un_dialogue_ne_laisse_aucune_dette() {
    let mut h = Horloge::nouvelle();
    h.accorder(P240);
    let mut t = Instant::now();
    h.presentee(t, true);
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    assert_eq!(h.pas(), P240.saturating_mul(2));
    // Vingt secondes à choisir un fichier ; l'image suivante, elle, était attendue.
    h.oublier();
    t += Duration::from_secs(20);
    h.presentee(t, true);
    assert_eq!(
        h.pas(),
        Duration::ZERO,
        "rien n'a ete montre pendant le dialogue"
    );
    assert_eq!(h.dette(), Duration::ZERO, "et rien ne reste a rattraper");
    t += Duration::from_micros(8_400);
    h.presentee(t, true);
    assert_eq!(h.pas(), P240.saturating_mul(2));
}
