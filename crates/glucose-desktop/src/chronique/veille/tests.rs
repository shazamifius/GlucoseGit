//! Ce que la veille promet : elle sépare ce qui se passe **sous la main** de ce qui se passe
//! sans elle, sans que personne ait à s'annoncer, et elle se tait quand elle n'a rien mesuré.

use super::*;

/// Un relevé du système, choisi.
fn vu(mo: u64, cpu_ms: u64) -> Empreinte {
    Empreinte {
        memoire_octets: mo * 1024 * 1024,
        processeur_us: cpu_ms * 1_000,
    }
}

/// Ce que la session a compté : ce que la main a demandé, et ce qui s'est rendu.
fn compte(sous_la_main: u64, rendues: u64) -> Compte {
    Compte {
        sous_la_main,
        rendues,
    }
}

/// Une suite d'instants séparés d'une seconde.
fn seconde(depuis: Instant, n: u64) -> Instant {
    depuis + Duration::from_secs(n)
}

/// **Un intervalle où la main n'a rien demandé se compte à part**, et son processeur ne se
/// mélange pas à l'autre.
///
/// C'est le critère factuel : personne ne déclare qu'il ne touche à rien, et c'est ce qui rend
/// la mesure impossible à fausser par oubli.
#[test]
fn test_un_intervalle_sans_la_main_se_compte_a_part() {
    let t0 = Instant::now();
    let mut v = Veille::default();
    v.noter(t0, compte(0, 0), vu(200, 500));
    // Une seconde sans un geste, et dix millisecondes de processeur.
    v.noter(seconde(t0, 1), compte(0, 0), vu(200, 510));
    // Puis une seconde d'usage : cent images demandées, trois cents millisecondes.
    v.noter(seconde(t0, 2), compte(100, 100), vu(240, 810));

    let repos = v
        .au_repos()
        .expect("un intervalle sans la main a ete observe");
    assert!(
        (repos.coeurs - 0.01).abs() < 0.001,
        "sans la main : {:.4} coeur",
        repos.coeurs
    );
    assert_eq!(repos.sur, Duration::from_secs(1));

    let usage = v.a_l_usage().expect("l'usage a ete observe");
    assert!(
        (usage.coeurs - 0.30).abs() < 0.001,
        "a l'usage : {:.4} coeur",
        usage.coeurs
    );
}

/// **Une application qui se réveille toute seule n'est pas une application dont on se sert.**
///
/// C'est le défaut que la première mesure de terrain a démenti en vingt secondes. Le critère
/// était *« aucune image ne s'est rendue »* ; lancée sans qu'on y touche, l'application a rendu
/// vingt-six images en vingt secondes, et la section a annoncé **« 1,4 % d'un cœur pendant
/// qu'on s'en sert »** alors que personne ne s'en servait.
///
/// Le compteur d'images ne pouvait pas répondre, puisque c'est justement ce qu'on mesure.
/// Celui de la main, si. Et ce que l'application dessine pendant ce temps devient un
/// **résultat** au lieu d'être la question.
#[test]
fn test_une_application_qui_se_reveille_seule_n_est_pas_un_usage() {
    let t0 = Instant::now();
    let mut v = Veille::default();
    v.noter(t0, compte(0, 0), vu(200, 0));
    // Dix secondes, pas un geste, et l'application rend treize images par seconde.
    for s in 1..=10 {
        v.noter(seconde(t0, s), compte(0, 13 * s), vu(200, 14 * s));
    }
    assert_eq!(
        v.a_l_usage(),
        None,
        "aucun geste : rien ne doit etre compte comme de l'usage"
    );
    let sans = v.au_repos().expect("dix secondes ont ete observees");
    assert_eq!(sans.sur, Duration::from_secs(10));
    assert!(
        (v.images_sans_la_main().expect("il y a de quoi le dire") - 13.0).abs() < 0.01,
        "images par seconde sans la main : {:?}",
        v.images_sans_la_main()
    );
}

/// **Le premier relevé n'ouvre aucun intervalle.**
///
/// Il n'a rien derrière lui : lui attribuer le temps écoulé depuis le lancement compterait
/// l'ouverture du document et le démarrage du pilote dans la part de repos, et la mesure
/// dirait le contraire de la vérité.
#[test]
fn test_le_premier_releve_n_ouvre_aucun_intervalle() {
    let mut v = Veille::default();
    v.noter(Instant::now(), compte(0, 0), vu(180, 900));
    assert_eq!(v.au_repos(), None);
    assert_eq!(v.a_l_usage(), None);
    assert_eq!(v.images_sans_la_main(), None);
    assert!(v.a_mesure(), "un releve a bien eu lieu");
}

/// **Rien n'a été mesuré, donc rien n'est dit** — plutôt que zéro.
///
/// Un compteur déclaré et jamais rempli affiche zéro, et un zéro se lit comme une mesure :
/// c'est la faute que la fiche 17 § 3.1 raconte, et sur laquelle tout un raisonnement s'est
/// bâti. Sur une plateforme sans relevé, la section ne doit pas paraître.
#[test]
fn test_sans_releve_la_veille_ne_dit_rien() {
    let v = Veille::default();
    assert!(!v.a_mesure());
    assert_eq!(v.memoire(), (0, 0));
    assert_eq!(v.au_repos(), None);
    assert_eq!(v.images_sans_la_main(), None);
}

/// **La mémoire garde son pire**, même quand elle redescend.
///
/// C'est le nombre qui décide si un logiciel est économe : un pic à huit cents mébioctets
/// pendant un import ne s'efface pas parce que le cache s'est vidé après.
#[test]
fn test_la_memoire_garde_son_pire() {
    let t0 = Instant::now();
    let mut v = Veille::default();
    v.noter(t0, compte(0, 0), vu(200, 0));
    v.noter(seconde(t0, 1), compte(10, 10), vu(800, 100));
    v.noter(seconde(t0, 2), compte(20, 20), vu(250, 200));
    let (maintenant, pire) = v.memoire();
    assert_eq!(maintenant, 250 * 1024 * 1024, "la derniere vue");
    assert_eq!(pire, 800 * 1024 * 1024, "et la plus grande de la session");
}

/// **Un compteur de processeur qui ne bouge pas donne zéro, pas une soustraction négative.**
///
/// Rien n'oblige le système à faire avancer ce compteur entre deux relevés — un processus
/// endormi n'en consomme littéralement pas —, et une soustraction d'entiers non signés qui
/// passe sous zéro produirait un nombre énorme au lieu d'un sommeil parfait.
#[test]
fn test_un_processeur_immobile_donne_un_sommeil_parfait() {
    let t0 = Instant::now();
    let mut v = Veille::default();
    v.noter(t0, compte(0, 0), vu(200, 1_000));
    v.noter(seconde(t0, 1), compte(0, 0), vu(200, 1_000));
    let repos = v.au_repos().expect("l'intervalle a ete observe");
    assert_eq!(repos.coeurs, 0.0);
    assert_eq!(
        v.images_sans_la_main(),
        Some(0.0),
        "et il n'a rien dessine : la seule bonne reponse"
    );
}

/// **Un seul geste suffit à ce que l'intervalle cesse d'être sans main.**
///
/// Un clic, un cran de molette, une touche : la mesure ne cherche pas à savoir combien on a
/// fait, seulement si on a fait quelque chose.
#[test]
fn test_un_seul_geste_suffit_a_reveiller_l_intervalle() {
    let t0 = Instant::now();
    let mut v = Veille::default();
    v.noter(t0, compte(42, 100), vu(200, 0));
    v.noter(seconde(t0, 1), compte(43, 140), vu(200, 20));
    assert_eq!(v.au_repos(), None, "aucun intervalle sans la main");
    assert!(v.a_l_usage().is_some());
}
