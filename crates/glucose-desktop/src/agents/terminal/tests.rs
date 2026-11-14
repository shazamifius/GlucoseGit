//! TERMINAL-1 et 2 — une commande par outil, lue par son schéma ; la même porte, le même accord.

use super::*;
use crate::agents::protocole::{Connexion, Suite};

fn mots(m: &[&str]) -> Vec<String> {
    m.iter().map(|s| (*s).to_string()).collect()
}

/// Les arguments de cette commande ; l'entrée standard, si on la lit, rend `entree`.
fn args_avec(commande: &str, m: &[&str], entree: &str) -> Result<Json, String> {
    let outil = outil_de(commande).expect("une commande");
    let schema = outils::schema(outil).expect("son schéma");
    let mut lue = 0;
    let r = arguments::lire(&schema, &mots(m), &mut || {
        lue += 1;
        Ok(entree.to_string())
    });
    assert!(lue <= 1, "l'entrée standard se lit une fois");
    r
}

fn args(commande: &str, m: &[&str]) -> Result<Json, String> {
    args_avec(commande, m, "")
}

/// **TERMINAL-1 : chaque outil a sa commande, et chaque commande son outil** — sans qu'aucune ne
/// tombe sur un mot réservé (l'aide, une option).
#[test]
fn terminal_1_chaque_outil_a_sa_commande() {
    for outil in outils::noms() {
        let c = commande_de(outil);
        assert_eq!(outil_de(&c), Some(outil), "{c}");
        assert!(!AIDE.contains(&c.as_str()) && !c.starts_with('-'), "{c}");
        assert_eq!(lire(&mots(&[&c])), Ok(Demande::Outil(outil, Vec::new())));
        assert!(aide(None).expect("l'aide").contains(&format!("\n  {c}\n")));
    }
    assert_eq!(commande_de("glucose_poser_texte"), "poser-texte");
    assert_eq!(outil_de("chercher"), Some("glucose_chercher"));
}

/// **La ligne de commande s'aiguille** : ouvrir, l'aide, une option de `main`, une commande, un
/// document ; une faute de frappe est refusée au lieu d'ouvrir Glucose.
#[test]
fn la_ligne_de_commande_s_aiguille() {
    assert_eq!(lire(&[]), Ok(Demande::Ouvrir(None)));
    assert_eq!(
        lire(&mots(&["aide", "lire"])),
        Ok(Demande::Aide(Some("lire".into())))
    );
    assert_eq!(lire(&mots(&["--help"])), Ok(Demande::Aide(None)));
    assert_eq!(lire(&mots(&["--version"])), Ok(Demande::Ailleurs));
    assert_eq!(
        lire(&mots(&["chercher", "eau"])),
        Ok(Demande::Outil("glucose_chercher", mots(&["eau"])))
    );
    let dossier = crate::epreuve::Dossier::nouveau("terminal-ouvrir");
    let doc = dossier.join("eau.glucose");
    std::fs::write(&doc, b"").expect("un fichier");
    let chemin = doc.to_string_lossy().into_owned();
    assert_eq!(
        lire(&mots(&[&chemin])),
        Ok(Demande::Ouvrir(Some(doc.clone())))
    );
    let _ = std::fs::remove_file(&doc);
    let introuvable = lire(&mots(&[&chemin])).expect_err("parti");
    assert!(introuvable.contains("introuvable"), "{introuvable}");
    let faute = lire(&mots(&["lir"])).expect_err("une faute");
    assert!(faute.contains("ni une commande"), "{faute}");
    assert!(lire(&mots(&["lir", "ids=a"])).is_err());
    assert!(aide(Some("lir")).is_err());
}

/// **Les arguments suivent le schéma de l'outil** : chaque valeur à son type, la clé répétée qui
/// fait une liste, les mots nus à l'argument exigé, `-` qui lit l'entrée standard.
#[test]
fn les_arguments_suivent_le_schema() {
    assert_eq!(
        args(
            "poser-texte",
            &["x=10", "post_it=true", "Bonjour", "le", "monde"]
        ),
        Json::lire(r#"{"x":10,"post_it":true,"texte":"Bonjour le monde"}"#)
    );
    assert_eq!(
        args("lire", &["ids=a", "ids=b", "niveau=complet"]),
        Json::lire(r#"{"ids":["a","b"],"niveau":"complet"}"#)
    );
    assert_eq!(
        args("retirer", &["n1", "n2"]),
        Json::lire(r#"{"ids":["n1","n2"]}"#)
    );
    assert_eq!(
        args(
            "poser-texte",
            &[r#"domaines=[{"nouveau":"eau"}]"#, "texte=x"]
        ),
        Json::lire(r#"{"domaines":[{"nouveau":"eau"}],"texte":"x"}"#)
    );
    assert_eq!(
        args_avec("poser-texte", &["-"], "# Titre\nUne ligne."),
        Json::lire(r##"{"texte":"# Titre\nUne ligne."}"##)
    );
    assert_eq!(
        args_avec("poser-membrane", &["titre=-", "dans=-"], "Eau"),
        Json::lire(r#"{"titre":"Eau","dans":"Eau"}"#),
        "lue une fois, servie deux"
    );
    let inconnue = args("lire", &["ids_=a"]).expect_err("une clé inconnue");
    assert!(
        inconnue.contains("niveau"),
        "la liste des clés : {inconnue}"
    );
    assert!(args("poser-texte", &["texte=a", "x=loin"]).is_err());
    assert!(args("poser-texte", &["texte=a", "post_it=peut-etre"]).is_err());
    assert!(args("lire", &["eau"]).is_err(), "lire n'exige rien");
}

/// **La requête est celle d'un client moderne nommé « le terminal »** : le protocole de la porte la
/// lit comme un appel d'outil — c'est sous ce nom que l'accord se demande (TERMINAL-2).
#[test]
fn terminal_2_la_requete_est_un_appel_du_terminal() {
    let arguments = Json::lire(r#"{"niveau":"complet"}"#).expect("JSON");
    let ligne = requete("glucose_lire", arguments).ecrire();
    let Suite::Executer(appel) = Connexion::default().traiter(&ligne) else {
        panic!("un appel d'outil : {ligne}");
    };
    assert_eq!(appel.client, CLIENT);
    assert_eq!(appel.nom, "glucose_lire");
    assert!(appel.moderne, "sans poignée de main");
    assert_eq!(
        appel.arguments.get("niveau").and_then(Json::as_str),
        Some("complet")
    );
}

/// **Ce que dit la réponse** : le JSON de l'outil, son refus, une erreur du protocole.
#[test]
fn le_resultat_se_lit_de_la_reponse() {
    let appel = crate::agents::protocole::Appel {
        id: Json::from(1.0),
        nom: "glucose_lire".into(),
        arguments: Json::objet([]),
        client: CLIENT.into(),
        moderne: true,
        portee: Default::default(),
    };
    let ok = protocole::reponse_d_outil(&appel, Ok(Json::objet([("total", Json::from(2.0))])));
    assert_eq!(resultat_de(&ok), Json::lire(r#"{"total":2}"#));
    let non = protocole::reponse_d_outil(&appel, Err("« x » n'existe pas".into()));
    assert_eq!(resultat_de(&non), Err("« x » n'existe pas".into()));
    let erreur = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"pas d'outil"}}"#;
    assert_eq!(resultat_de(erreur), Err("pas d'outil".into()));
}

/// **L'aide d'une commande** dit chacun de ses arguments, et celui qu'elle exige.
#[test]
fn l_aide_d_une_commande_dit_ses_arguments() {
    let a = aide(Some("poser-texte")).expect("l'aide");
    assert!(a.starts_with("glucose poser-texte — Pose un texte"), "{a}");
    assert!(
        a.contains("texte : un texte, exigé — s'écrit aussi sans « clé= »"),
        "{a}"
    );
    assert!(a.contains("post_it : true ou false"), "{a}");
    let l = aide(Some("lire")).expect("l'aide");
    assert!(l.contains("niveau : sommaire, complet ou plan"), "{l}");
}

/// **De bout en bout, par une vraie porte** (TERMINAL-2) : la commande attend l'accord, demandé au
/// nom du terminal ; une fois donné, le texte est posé, se trouve par `chercher`, et `Ctrl+Z` le
/// défait.
#[cfg(unix)]
#[test]
fn terminal_2_une_commande_passe_par_la_porte_et_l_accord() {
    use crate::app::agents::Agents;
    use crate::interactions::question::tests::repondre;
    use crate::ui::question::{Reponse, Suite as Question};
    use std::time::{Duration, Instant};

    let mut app = crate::app::GlucoseApp::new();
    let boite = crate::agents::Boite::nouvelle(std::sync::Arc::new(|| {}));
    app.lancement.agents = Agents::avec_la_boite(boite.clone());
    let dossier = crate::epreuve::Dossier::nouveau("terminal-porte");
    let nom = dossier.join("porte.sock").to_string_lossy().into_owned();
    let _porte = porte::Porte::ouvrir(&nom, boite).expect("la porte");
    // Une commande, sur son fil : celui de l'épreuve joue la boucle de Glucose.
    let lancer = |outil: &'static str, a: &str| {
        let (n, a) = (nom.clone(), Json::lire(a).expect("JSON"));
        std::thread::spawn(move || appeler(&n, false, outil, a))
    };
    fn attendre<T>(app: &mut crate::app::GlucoseApp, fil: &std::thread::JoinHandle<T>) {
        let debut = Instant::now();
        while !fil.is_finished() && app.ui.question.is_none() {
            assert!(
                debut.elapsed() < Duration::from_secs(20),
                "la porte ne répond pas"
            );
            app.suivre_les_agents();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    let pose = lancer(
        "glucose_poser_texte",
        r#"{"id":"vu","texte":"La vapeur monte."}"#,
    );
    attendre(&mut app, &pose);
    match &app.ui.question {
        Some((_, Question::Agent(client))) => assert_eq!(client, CLIENT),
        autre => panic!("la question d'accord : {autre:?}"),
    }
    repondre(&mut app, Reponse::Oui);
    attendre(&mut app, &pose);
    let rendu = pose.join().expect("le fil").expect("posé");
    assert_eq!(rendu.get("id").and_then(Json::as_str), Some("vu"));

    let cherche = lancer("glucose_chercher", r#"{"cherche":"VAPEUR"}"#);
    attendre(&mut app, &cherche);
    let trouve = cherche.join().expect("le fil").expect("cherché");
    assert_eq!(trouve.get("total").and_then(Json::as_f64), Some(1.0));

    app.store.undo();
    let tableau = app.store.project.active_board_id.clone();
    assert!(
        app.store.project.annotation(&tableau, "vu").is_none(),
        "Ctrl+Z le défait"
    );
}
