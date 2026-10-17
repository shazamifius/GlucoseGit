//! **`glucose --mcp`, de bout en bout** (fiche 75) : le vrai programme, lancé comme un client
//! MCP le lance — ses flux standard pour seul canal —, relaie vers une porte d'épreuve
//! (`GLUCOSE_PORTE`) et rend chaque réponse. Rien d'autre que le protocole sur sa sortie.

#![cfg(not(target_os = "android"))]

use glucose_desktop::agents::json::Json;
use glucose_desktop::agents::porte::Porte;
use glucose_desktop::agents::{protocole, Boite};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;

fn nom_de_porte() -> (String, Option<std::path::PathBuf>) {
    if cfg!(windows) {
        (
            format!(r"\\.\pipe\glucose-relais-{}", std::process::id()),
            None,
        )
    } else {
        let d = std::env::temp_dir().join(format!("glucose-relais-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        (d.join("p.sock").to_string_lossy().into_owned(), Some(d))
    }
}

/// La boucle de Glucose, en plus simple : chaque appel d'outil reçoit son nom.
fn boucle(boite: Boite) {
    std::thread::spawn(move || loop {
        for d in boite.relever() {
            let r = Json::objet([("outil", Json::from(d.appel.nom.as_str()))]);
            let _ = d.reponse.send(protocole::reponse_d_outil(&d.appel, Ok(r)));
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    });
}

/// La sortie du relais, ligne à ligne, lue sur un fil : une réponse qui ne vient pas fait tomber
/// l'épreuve par son délai au lieu de la figer (le sabotage de RELAIS-1 l'a montré).
fn lignes_bornees(sortie: std::process::ChildStdout) -> std::sync::mpsc::Receiver<String> {
    let (envoi, reception) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
            if envoi.send(ligne).is_err() {
                return;
            }
        }
    });
    reception
}

#[test]
fn le_relais_porte_le_protocole_entre_un_client_et_glucose() {
    let (nom, dossier) = nom_de_porte();
    let boite = Boite::nouvelle(Arc::new(|| {}));
    let _porte = Porte::ouvrir(&nom, boite.clone()).expect("la porte d'épreuve");
    boucle(boite);
    let mut relais = Command::new(env!("CARGO_BIN_EXE_glucose-desktop"))
        .arg("--mcp")
        .env("GLUCOSE_PORTE", &nom)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("le relais se lance");
    let mut entree = relais.stdin.take().expect("son entrée");
    let sortie = lignes_bornees(relais.stdout.take().expect("sa sortie"));
    // Un client de l'ancienne époque, comme la plupart aujourd'hui — qui attend chaque réponse
    // avant d'écrire la requête suivante : c'est ainsi que le relais se figeait (TUYAU-1).
    let requetes = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"essai"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"glucose_lire"}}"#,
    ];
    let mut lignes = Vec::new();
    for r in requetes {
        writeln!(entree, "{r}").expect("écrit");
        if !r.contains("\"id\"") {
            continue;
        }
        let ligne = sortie
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("une réponse, avant le délai");
        lignes.push(Json::lire(ligne.trim()).unwrap_or_else(|e| panic!("« {ligne} » : {e}")));
    }
    assert_eq!(
        lignes[0]
            .get("result")
            .and_then(|r| r.get("protocolVersion"))
            .and_then(Json::as_str),
        Some("2025-06-18")
    );
    assert!(lignes[1].ecrire().contains("glucose_poser_page"));
    assert!(
        lignes[2].ecrire().contains(r#"\"outil\":\"glucose_lire\""#),
        "{}",
        lignes[2].ecrire()
    );
    // Le client ferme l'entrée : le relais finit, de lui-même.
    drop(entree);
    let fin = relais.wait().expect("il finit");
    assert!(fin.success());
    if let Some(d) = dossier {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// **RELAIS-1** : sans aucune porte ouverte, le vrai relais répond à la poignée de main et à la
/// liste des outils, puis finit quand le client ferme — il n'a rien ouvert.
#[test]
fn le_relais_n_ouvre_rien_tant_qu_aucun_outil_n_est_appele() {
    let mut relais = Command::new(env!("CARGO_BIN_EXE_glucose-desktop"))
        .arg("--mcp")
        .env("GLUCOSE_PORTE", nom_de_porte().0 + "-absente")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("le relais se lance");
    let mut entree = relais.stdin.take().expect("son entrée");
    let sortie = lignes_bornees(relais.stdout.take().expect("sa sortie"));
    for r in [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    ] {
        writeln!(entree, "{r}").expect("écrit");
        let ligne = sortie
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("une réponse, avant le délai");
        assert!(ligne.contains("\"result\""), "{ligne}");
    }
    drop(entree);
    assert!(relais.wait().expect("il finit").success());
}
