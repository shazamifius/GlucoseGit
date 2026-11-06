//! **MOTEUR-1 — la sonde d'un moteur, par le vrai programme** (fiche 84).
//!
//! `--sonder-moteur` est ce qui rend un moteur écarté à une nouvelle version : s'il mentait sur
//! une mort, Glucose rouvrirait le pilote qui l'a tué ; s'il mentait sur une survie, le moteur
//! resterait écarté pour toujours.
#![cfg(not(target_os = "android"))]

use glucose_desktop::present::moteurs;
use std::process::{Command, Stdio};

fn sonder(moteur: &str, tomber: Option<&str>) -> std::process::ExitStatus {
    let mut sonde = Command::new(env!("CARGO_BIN_EXE_glucose-desktop"));
    sonde
        .args(["--sonder-moteur", moteur])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match tomber {
        Some(m) => sonde.env("GLUCOSE_TOMBER", m),
        None => sonde.env_remove("GLUCOSE_TOMBER"),
    };
    sonde.status().expect("la sonde se lance")
}

#[test]
fn test_moteur_1_chaque_moteur_de_ce_systeme_se_sonde() {
    for (_, nom) in moteurs::ici() {
        assert_eq!(
            sonder(nom, None).code(),
            Some(0),
            "{nom} : il devait survivre"
        );
    }
}

#[test]
fn test_moteur_1_une_sonde_morte_ne_passe_pas_pour_une_survie() {
    let (_, nom) = moteurs::ici().next().expect("un moteur");
    assert!(
        !sonder(nom, Some(nom)).success(),
        "{nom} : elle devait tomber"
    );
}

#[test]
fn test_moteur_1_un_nom_inconnu_est_refuse() {
    assert_eq!(sonder("inconnu", None).code(), Some(2));
}
