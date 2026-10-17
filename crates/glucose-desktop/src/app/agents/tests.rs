//! AGENTS-1 — rien sans son accord : la question, puis l'appel ; et rien pendant son geste.

use super::Agents;
use crate::agents::json::Json;
use crate::agents::protocole::Appel;
use crate::agents::{Boite, Demande};
use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::ui::question::{Reponse, Suite};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// Une application dont la boîte reçoit ce qu'une épreuve y dépose.
fn app_et_boite() -> (GlucoseApp, Boite) {
    let mut app = GlucoseApp::new();
    let boite = Boite::nouvelle(Arc::new(|| {}));
    app.lancement.agents = Agents::avec_la_boite(boite.clone());
    (app, boite)
}

/// Un agent dépose un appel ; on garde de quoi lire sa réponse.
fn appeler(boite: &Boite, client: &str, nom: &str, arguments: &str) -> Receiver<String> {
    let (envoi, attente) = channel();
    boite.deposer(Demande {
        appel: Appel {
            id: Json::Nombre(1.0),
            nom: nom.to_string(),
            arguments: Json::lire(arguments).expect("JSON"),
            client: client.to_string(),
            moderne: true,
        },
        reponse: envoi,
    });
    attente
}

/// La réponse est-elle une erreur d'outil ?
fn en_erreur(ligne: &str) -> bool {
    let r = Json::lire(ligne).expect("une réponse");
    r.get("result")
        .and_then(|r| r.get("isError"))
        .and_then(Json::as_bool)
        .expect("isError")
}

fn existe(app: &GlucoseApp, id: &str) -> bool {
    let b = app.store.project.active_board_id.clone();
    app.store.project.annotation(&b, id).is_some()
}

#[test]
fn agents_1_le_premier_appel_attend_sa_reponse_puis_passe() {
    let (mut app, boite) = app_et_boite();
    let reponse = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    app.suivre_les_agents();
    let Some((question, Suite::Agent(client))) = &app.ui.question else {
        panic!("la question d'accord");
    };
    assert_eq!(client, "proteo");
    assert!(question.texte.contains("« proteo »"));
    assert!(reponse.try_recv().is_err(), "l'appel attend");
    assert!(!existe(&app, "a"), "rien n'est écrit avant sa réponse");

    repondre(&mut app, Reponse::Oui);
    assert!(!en_erreur(&reponse.try_recv().expect("répondu")));
    assert!(existe(&app, "a"));
    // L'accord vaut pour la suite : aucune question de plus.
    let suivante = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"b","texte":"y"}"#,
    );
    app.suivre_les_agents();
    assert!(app.ui.question.is_none());
    assert!(!en_erreur(&suivante.try_recv().expect("répondu")));
}

#[test]
fn agents_1_refuser_vaut_pour_toute_la_session() {
    let (mut app, boite) = app_et_boite();
    let premiere = appeler(&boite, "intrus", "glucose_lire", "{}");
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Non);
    assert!(en_erreur(&premiere.try_recv().expect("répondu")));
    let seconde = appeler(
        &boite,
        "intrus",
        "glucose_poser_texte",
        r#"{"id":"z","texte":"x"}"#,
    );
    app.suivre_les_agents();
    assert!(app.ui.question.is_none(), "on ne lui redemande pas");
    assert!(en_erreur(&seconde.try_recv().expect("répondu")));
    assert!(!existe(&app, "z"));
}

#[test]
fn agents_1_plus_tard_refuse_cet_appel_et_redemande_au_suivant() {
    let (mut app, boite) = app_et_boite();
    let premiere = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Annuler);
    assert!(en_erreur(&premiere.try_recv().expect("répondu")));
    let _seconde = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(matches!(&app.ui.question, Some((_, Suite::Agent(_)))));
}

#[test]
fn agents_1_sa_question_n_est_jamais_chassee() {
    let (mut app, boite) = app_et_boite();
    app.demander(Default::default(), Suite::Telemetrie);
    let reponse = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(matches!(&app.ui.question, Some((_, Suite::Telemetrie))));
    assert!(reponse.try_recv().is_err());
}

#[test]
fn agent_1_rien_ne_passe_pendant_son_geste() {
    let (mut app, boite) = app_et_boite();
    let _ = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Oui);
    app.store.begin_live_edit();
    let reponse = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    app.suivre_les_agents();
    assert!(
        reponse.try_recv().is_err(),
        "l'appel attend la fin du geste"
    );
    app.store.end_live_edit();
    app.suivre_les_agents();
    assert!(!en_erreur(&reponse.try_recv().expect("répondu")));
    assert!(existe(&app, "a"));
}

#[test]
fn retirer_l_accord_reposera_la_question_au_prochain_appel() {
    let (mut app, boite) = app_et_boite();
    let _ = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Oui);
    assert!(app.executer(crate::commandes::Commande::OublierLesAgents));
    let Some((question, Suite::OublierLesAgents)) = &app.ui.question else {
        panic!("la question qui dit qui a son accord");
    };
    assert!(question.texte.contains("« proteo »"));
    repondre(&mut app, Reponse::Oui);
    let suivante = appeler(&boite, "proteo", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(
        matches!(&app.ui.question, Some((_, Suite::Agent(_)))),
        "on lui redemande"
    );
    assert!(suivante.try_recv().is_err());
}

#[test]
fn sans_agent_autorise_la_question_le_dit_et_ne_retire_rien() {
    let (mut app, _boite) = app_et_boite();
    assert!(app.executer(crate::commandes::Commande::OublierLesAgents));
    let Some((question, _)) = &app.ui.question else {
        panic!("une question");
    };
    assert!(question.texte.contains("Aucun agent"));
    assert_eq!(question.choix.len(), 1);
}
