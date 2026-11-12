//! **MAIN-1 et NOTE-1 — la Time Machine pour un agent** (fiche 108) : un agent signe ses gestes de
//! son nom, attache un écrit à l'un d'eux, et relit l'histoire avec ses auteurs, ce que chaque
//! geste a fait, et ses écrits — par ses outils, sur une application d'épreuve qui écrit son
//! brouillon.

use super::appeler;
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::epreuve::Dossier;
use glucose_core::persist::histoire;
use glucose_core::store::journal::Main;
use glucose_core::types::Annotation;

/// Le texte d'un champ d'un objet JSON.
fn texte<'a>(j: &'a Json, cle: &str) -> &'a str {
    j.get(cle).and_then(Json::as_str).unwrap_or_default()
}

#[test]
fn main_1_un_agent_signe_ses_gestes_et_y_attache_un_ecrit() {
    let dossier = Dossier::nouveau("temps-1");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("h", 0.0, 0.0, "écrit à la main"));
    app.consigner();

    // Ce que fait la boucle autour de l'appel d'un agent (`suivre_les_agents`).
    app.store.journal.main = Main::Agent("proteo".into());
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p1","texte":"L'eau"}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p2","texte":"La pluie"}"#,
    );
    let pourquoi = "La pluie est de l'eau qui tombe : ces deux textes vont ensemble.";
    let note = appeler(
        &mut app,
        "glucose_noter",
        &format!(r#"{{"texte":"{pourquoi}"}}"#),
    );
    let lue = appeler(&mut app, "glucose_histoire", "{}");
    app.store.journal.main = Main::Humain;

    let gestes = lue
        .get("gestes")
        .and_then(Json::as_liste)
        .expect("des gestes");
    let derniers = &gestes[gestes.len() - 3..];
    let auteurs: Vec<&str> = derniers.iter().map(|g| texte(g, "auteur")).collect();
    assert_eq!(auteurs, ["humain", "agent proteo", "agent proteo"]);
    let rang = derniers[2].get("rang").and_then(Json::as_f64);
    assert_eq!(
        note.get("geste").and_then(Json::as_f64),
        rang,
        "son dernier geste"
    );
    let fait = derniers[2]
        .get("fait")
        .and_then(Json::as_liste)
        .expect("ce qu'il a fait");
    assert!(
        fait.iter()
            .any(|f| f.as_str() == Some("pose le texte « p2 »")),
        "{fait:?}"
    );
    let ecrits = derniers[2]
        .get("ecrits")
        .and_then(Json::as_liste)
        .expect("ses écrits");
    assert_eq!(ecrits.len(), 1);
    assert_eq!(texte(&ecrits[0], "auteur"), "agent proteo");
    assert_eq!(texte(&ecrits[0], "texte"), pourquoi);
    assert!(
        derniers[1]
            .get("ecrits")
            .and_then(Json::as_liste)
            .is_some_and(<[Json]>::is_empty),
        "l'écrit ne s'attache qu'à son geste"
    );

    // Le fichier déclare l'agent une fois, avant son premier geste.
    let chemin = app
        .disque
        .ecriture
        .as_ref()
        .expect("un brouillon")
        .chemin
        .clone();
    let octets = std::fs::read(&chemin).expect("le brouillon");
    let o = histoire::ouvrir(&mut std::io::Cursor::new(octets)).expect("il se relit");
    assert_eq!(o.auteurs.len(), 1, "une déclaration par auteur");
    let proteo = histoire::Auteur::d_un_agent("proteo");
    assert_eq!(o.auteurs.get(&proteo.id), Some(&proteo));

    // Rouvert, le document sait ce que l'agent y a posé, et ce que l'humain a fait (MAIN-1).
    // Un seul scribe par fichier : la première fenêtre se ferme avant que l'autre l'ouvre.
    drop(app);
    let mut rouvert = GlucoseApp::new();
    rouvert.habiter(&dossier);
    rouvert.open_from(chemin);
    let b = rouvert.store.project.active_board_id.clone();
    assert!(
        rouvert.store.project.annotation(&b, "p1").is_some(),
        "le document s'est rouvert"
    );
    let des_agents = &rouvert.store.journal.des_agents;
    assert!(
        des_agents.contains("p1") && des_agents.contains("p2"),
        "{des_agents:?}"
    );
    assert!(
        !des_agents.contains("h"),
        "le texte de l'humain reste à l'humain"
    );
}

/// **Un écrit vide, ou sur un geste qui n'existe pas, est refusé** — et un agent qui n'a encore
/// rien fait doit nommer le geste qu'il commente.
#[test]
fn note_1_un_ecrit_sans_objet_est_refuse() {
    let dossier = Dossier::nouveau("temps-1-refus");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("h", 0.0, 0.0, "à la main"));
    app.consigner();
    app.store.journal.main = Main::Agent("proteo".into());
    for (arguments, raison) in [
        (r#"{"texte":"  "}"#, "vide"),
        (r#"{"texte":"pourquoi"}"#, "aucun geste"),
        (r#"{"texte":"pourquoi","geste":999}"#, "n'existe pas"),
        (r#"{"texte":"pourquoi","geste":1.5}"#, "rang"),
    ] {
        let refus = app
            .executer_pour_un_agent("glucose_noter", &super::args(arguments))
            .expect_err(arguments);
        assert!(refus.contains(raison), "{arguments} : {refus}");
    }
    let relu = |app: &GlucoseApp| {
        let ecriture = app.disque.ecriture.as_ref().expect("un brouillon");
        ecriture.synchroniser().expect("sur le disque");
        let octets = std::fs::read(&ecriture.chemin).expect("le brouillon");
        histoire::ouvrir(&mut std::io::Cursor::new(octets)).expect("il se relit")
    };
    let o = relu(&app);
    assert!(
        o.auteurs.is_empty() && o.notes.is_empty(),
        "un écrit refusé ne laisse rien dans le fichier"
    );
    // Nommer un geste de l'humain : un agent peut commenter ce que l'humain a fait.
    appeler(
        &mut app,
        "glucose_noter",
        r#"{"texte":"le premier texte","geste":0}"#,
    );
    let o = relu(&app);
    assert_eq!((o.auteurs.len(), o.notes.len()), (1, 1));
}
