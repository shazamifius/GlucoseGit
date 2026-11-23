//! AGENTS-1 — rien sans son accord : la question, puis l'appel ; et rien pendant son geste.

use super::{Agents, TOUJOURS};
use crate::agents::json::Json;
use crate::agents::protocole::Appel;
use crate::agents::toujours::Toujours;
use crate::agents::{Boite, Demande};
use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::ui::question::{Reponse, Suite};
use std::path::{Path, PathBuf};
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
            portee: Default::default(),
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

/// **MAIN-1 — par la vraie boucle, le geste d'un agent porte son nom, et la main revient à
/// l'humain** (fiche 108) : sans ce retour, chaque geste qui suit l'appel d'un agent lui serait
/// attribué dans l'histoire.
#[test]
fn main_1_apres_un_agent_la_main_revient_a_l_humain() {
    use glucose_core::store::journal::Main;
    use glucose_core::types::Annotation;
    let (mut app, boite) = app_et_boite();
    app.store.journal.prendre_les_ecrits_signes();
    let reponse = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Oui);
    assert!(!en_erreur(&reponse.try_recv().expect("répondu")));
    assert_eq!(
        app.store.journal.main,
        Main::Humain,
        "la main revient à l'humain"
    );
    let b = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&b, Annotation::text("h", 0.0, 0.0, "à la main"));
    let mains: Vec<Main> = app
        .store
        .journal
        .prendre_les_ecrits_signes()
        .into_iter()
        .map(|e| e.main)
        .collect();
    assert_eq!(mains, [Main::Agent("proteo".into()), Main::Humain]);
}

/// Donne son accord à cet agent, par la vraie question.
fn autoriser(app: &mut GlucoseApp, boite: &Boite, client: &str) {
    let _ = appeler(boite, client, "glucose_lire", "{}");
    app.suivre_les_agents();
    repondre(app, Reponse::Oui);
}

/// Les identifiants des nœuds qu'une lecture rend à l'agent, tels qu'il les reçoit.
fn ids_lus(ligne: &str) -> Vec<String> {
    let r = Json::lire(ligne).expect("une réponse");
    let texte = r
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(Json::as_liste)
        .and_then(|l| l.first())
        .and_then(|c| c.get("text"))
        .and_then(Json::as_str)
        .expect("le texte de la lecture");
    let lu = Json::lire(texte).expect("la lecture est du JSON");
    let noeuds = lu.get("noeuds").and_then(Json::as_liste).expect("noeuds");
    noeuds
        .iter()
        .filter_map(|n| n.get("id").and_then(Json::as_str).map(str::to_string))
        .collect()
}

/// **LECTURE-1 — une lecture passe pendant son geste** (fiche 130). Le 22/11, la roue des
/// couleurs restée ouverte a retenu treize minutes la lecture de sa consigne par le relais ;
/// et le geste, lui, reste ouvert.
#[test]
fn lecture_1_une_lecture_passe_pendant_son_geste() {
    let (mut app, boite) = app_et_boite();
    autoriser(&mut app, &boite, "relais");
    app.store.begin_live_edit();
    for outil in ["glucose_lire", "glucose_chercher"] {
        let reponse = appeler(&boite, "relais", outil, r#"{"cherche":"x"}"#);
        app.suivre_les_agents();
        let ligne = reponse.try_recv();
        assert!(ligne.is_ok(), "{outil} répond pendant son geste");
        assert!(!en_erreur(&ligne.unwrap()), "{outil}");
    }
    assert!(app.store.in_live_edit(), "son geste reste ouvert");
}

/// **LECTURE-1 — aucun agent n'attend derrière un autre** : l'écriture de l'un attend la fin du
/// geste, la lecture de l'autre passe.
#[test]
fn lecture_1_un_agent_n_attend_jamais_derriere_un_autre() {
    let (mut app, boite) = app_et_boite();
    autoriser(&mut app, &boite, "proteo");
    autoriser(&mut app, &boite, "relais");
    app.store.begin_live_edit();
    let ecriture = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    let lecture = appeler(&boite, "relais", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(lecture.try_recv().is_ok(), "le relais n'attend pas proteo");
    assert!(ecriture.try_recv().is_err(), "l'écriture attend son geste");
    app.store.end_live_edit();
    app.suivre_les_agents();
    assert!(!en_erreur(&ecriture.try_recv().expect("répondu")));
    assert!(existe(&app, "a"));
}

/// **LECTURE-1 — deux connexions du même nom ne s'attendent pas** (fiche 130), vu au pilote : chaque
/// commande d'un terminal se présente comme « le terminal ». Une écriture qui attend le geste —
/// peut-être déjà abandonnée par son client — ne retient pas la lecture d'un autre terminal, qui
/// ne voit pas d'écriture fantôme.
#[test]
fn lecture_1_deux_connexions_du_meme_nom_ne_s_attendent_pas() {
    let (mut app, boite) = app_et_boite();
    autoriser(&mut app, &boite, "le terminal");
    app.store.begin_live_edit();
    let ecriture = appeler(
        &boite,
        "le terminal",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    let lecture = appeler(&boite, "le terminal", "glucose_lire", "{}");
    app.suivre_les_agents();
    let lu = ids_lus(&lecture.try_recv().expect("la lecture n'attend pas"));
    assert!(!lu.iter().any(|id| id == "a"), "rien de fantôme : {lu:?}");
    assert!(ecriture.try_recv().is_err(), "l'écriture attend son geste");
    app.store.end_live_edit();
    app.suivre_les_agents();
    assert!(!en_erreur(&ecriture.try_recv().expect("répondu")));
    assert!(existe(&app, "a"));
}

/// **LECTURE-1 — la question posée pour un agent ne retient pas ceux qui ont son accord.**
#[test]
fn lecture_1_la_question_d_un_agent_ne_retient_pas_les_autres() {
    let (mut app, boite) = app_et_boite();
    autoriser(&mut app, &boite, "proteo");
    let intrus = appeler(&boite, "intrus", "glucose_lire", "{}");
    let proteo = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    app.suivre_les_agents();
    assert!(
        matches!(&app.ui.question, Some((_, Suite::Agent(c))) if c == "intrus"),
        "on demande pour l'intrus"
    );
    assert!(intrus.try_recv().is_err(), "l'intrus attend sa réponse");
    assert!(!en_erreur(
        &proteo.try_recv().expect("proteo ne l'attend pas")
    ));
    assert!(existe(&app, "a"));
}

/// **« Plus tard » refuse l'appel de l'agent questionné**, pas celui d'un autre qui attend devant
/// lui dans la file (LECTURE-1 : la tête de la file n'est plus forcément la sienne).
#[test]
fn lecture_1_plus_tard_refuse_l_appel_de_cet_agent_et_pas_un_autre() {
    let (mut app, boite) = app_et_boite();
    autoriser(&mut app, &boite, "proteo");
    app.store.begin_live_edit();
    let ecriture = appeler(
        &boite,
        "proteo",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    let intrus = appeler(&boite, "intrus", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(matches!(&app.ui.question, Some((_, Suite::Agent(c))) if c == "intrus"));
    repondre(&mut app, Reponse::Annuler);
    assert!(en_erreur(&intrus.try_recv().expect("l'intrus est refusé")));
    assert!(
        ecriture.try_recv().is_err(),
        "proteo attend toujours son geste"
    );
    app.store.end_live_edit();
    app.suivre_les_agents();
    assert!(!en_erreur(&ecriture.try_recv().expect("répondu")));
}

/// Un document qui vit dans un fichier, et le fichier où Glucose retient l'accord durable — dans
/// un dossier d'épreuve, jamais celui de l'utilisateur.
fn document_et_toujours(nom: &str) -> (PathBuf, PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "glucose-agents-toujours-{nom}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dossier");
    let document = d.join("GLUCOSE PROJET.glucose");
    std::fs::write(&document, "{}").expect("document");
    (document, d.join("agents-toujours.txt"))
}

/// Un Glucose lancé sur ce document, son accord durable lu dans ce fichier — comme au vrai
/// lancement (`ouvrir_la_porte_des_agents`).
fn lancer(document: &Path, toujours: &Path) -> (GlucoseApp, Boite) {
    let (mut app, boite) = app_et_boite();
    app.lancement.agents.toujours = Toujours::depuis(toujours.to_path_buf());
    app.store.identite = document.display().to_string();
    (app, boite)
}

/// Sa réponse « Toujours, dans ce document » à la question posée pour cet agent.
fn toujours(app: &mut GlucoseApp, boite: &Boite, client: &str) -> Receiver<String> {
    let reponse = appeler(boite, client, "glucose_lire", "{}");
    app.suivre_les_agents();
    let Some((question, Suite::Agent(_))) = &app.ui.question else {
        panic!("la question d'accord");
    };
    let offert = question
        .choix
        .iter()
        .any(|(l, r)| l == "Toujours, dans ce document" && *r == TOUJOURS);
    assert!(offert, "le choix s'offre : {:?}", question.choix);
    repondre(app, TOUJOURS);
    reponse
}

/// **AGENTS-TOUJOURS — une fois choisi, tout programme de la machine passe, quel que soit son nom,
/// et Glucose relancé ne repose pas la question** (fiche 131). Le 23/11 au matin, le PC redémarré,
/// il a dû autoriser trois fois à la main : « le terminal », « claude-code »…
#[test]
fn agents_toujours_vaut_pour_tout_agent_de_la_machine_et_survit_au_relancement() {
    let (document, fichier) = document_et_toujours("relance");
    let (mut app, boite) = lancer(&document, &fichier);
    let premiere = toujours(&mut app, &boite, "le terminal");
    assert!(!en_erreur(&premiere.try_recv().expect("répondu")));
    let autre = appeler(
        &boite,
        "claude-code",
        "glucose_poser_texte",
        r#"{"id":"a","texte":"x"}"#,
    );
    app.suivre_les_agents();
    assert!(
        app.ui.question.is_none(),
        "aucune question pour un autre nom"
    );
    assert!(!en_erreur(&autre.try_recv().expect("répondu")));

    let (mut relance, boite) = lancer(&document, &fichier);
    let apres = appeler(
        &boite,
        "un nom jamais vu",
        "glucose_poser_texte",
        r#"{"id":"b","texte":"y"}"#,
    );
    relance.suivre_les_agents();
    assert!(relance.ui.question.is_none(), "relancé, aucune question");
    assert!(!en_erreur(&apres.try_recv().expect("répondu")));
    assert!(existe(&relance, "b"));
}

/// La preuve à l'envers : « Autoriser » ne vaut que jusqu'à la fermeture — relancé, on redemande.
#[test]
fn agents_toujours_autoriser_seul_ne_survit_pas_au_relancement() {
    let (document, fichier) = document_et_toujours("autoriser");
    let (mut app, boite) = lancer(&document, &fichier);
    autoriser(&mut app, &boite, "le terminal");
    let (mut relance, boite) = lancer(&document, &fichier);
    let attente = appeler(&boite, "le terminal", "glucose_lire", "{}");
    relance.suivre_les_agents();
    assert!(matches!(&relance.ui.question, Some((_, Suite::Agent(_)))));
    assert!(attente.try_recv().is_err());
}

/// **Dans ce document seulement** : un autre document ouvert, la question revient.
#[test]
fn agents_toujours_ne_vaut_que_pour_ce_document() {
    let (document, fichier) = document_et_toujours("autre");
    let (mut app, boite) = lancer(&document, &fichier);
    let _ = toujours(&mut app, &boite, "le terminal");
    let ailleurs = document.with_file_name("autre.glucose");
    std::fs::write(&ailleurs, "{}").expect("document");
    let (mut app, boite) = lancer(&ailleurs, &fichier);
    let attente = appeler(&boite, "le terminal", "glucose_lire", "{}");
    app.suivre_les_agents();
    assert!(matches!(&app.ui.question, Some((_, Suite::Agent(_)))));
    assert!(attente.try_recv().is_err());
}

/// Un document jamais enregistré n'a pas d'identité qui survive : le choix ne s'offre pas (il
/// mentirait), la question le tait.
#[test]
fn agents_toujours_ne_s_offre_pas_a_un_document_jamais_enregistre() {
    let (_document, fichier) = document_et_toujours("vierge");
    let (mut app, boite) = app_et_boite();
    app.lancement.agents.toujours = Toujours::depuis(fichier);
    let _ = appeler(&boite, "le terminal", "glucose_lire", "{}");
    app.suivre_les_agents();
    let Some((question, Suite::Agent(_))) = &app.ui.question else {
        panic!("la question d'accord");
    };
    assert!(!question.choix.iter().any(|(_, r)| *r == TOUJOURS));
    assert!(!question.texte.contains("Toujours"));
}

/// **« Retirer l'accord des agents… » le dit, et le reprend** — d'un lancement à l'autre.
#[test]
fn agents_toujours_retirer_l_accord_le_reprend_d_un_lancement_a_l_autre() {
    let (document, fichier) = document_et_toujours("retirer");
    let (mut app, boite) = lancer(&document, &fichier);
    let _ = toujours(&mut app, &boite, "le terminal");
    assert!(app.executer(crate::commandes::Commande::OublierLesAgents));
    let Some((question, Suite::OublierLesAgents)) = &app.ui.question else {
        panic!("la question qui dit qui a son accord");
    };
    assert!(
        question.texte.contains("tout programme de cette machine"),
        "{}",
        question.texte
    );
    repondre(&mut app, Reponse::Oui);
    let (mut relance, boite) = lancer(&document, &fichier);
    let _ = appeler(&boite, "le terminal", "glucose_lire", "{}");
    relance.suivre_les_agents();
    assert!(
        matches!(&relance.ui.question, Some((_, Suite::Agent(_)))),
        "on lui redemande"
    );
}

/// Le disque refuse l'écriture : l'accord vaut pour la session, et la question qui le retire le dit.
#[test]
fn agents_toujours_pas_retenu_sur_le_disque_se_dit() {
    let (document, fichier) = document_et_toujours("disque");
    std::fs::write(&fichier, "").expect("un fichier là où il faudrait un dossier");
    let (mut app, boite) = lancer(&document, &fichier.join("agents-toujours.txt"));
    let _ = toujours(&mut app, &boite, "le terminal");
    assert!(app.executer(crate::commandes::Commande::OublierLesAgents));
    let Some((question, Suite::OublierLesAgents)) = &app.ui.question else {
        panic!("la question qui dit qui a son accord");
    };
    assert!(
        question.texte.contains("pour cette session seulement"),
        "{}",
        question.texte
    );
}
