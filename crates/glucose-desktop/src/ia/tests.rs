//! Les épreuves des IA gratuites : le classement des modèles, et les clés qui se relisent telles
//! qu'on les a écrites — sans le réseau.

use super::cles::{self, Cles};
use super::modeles::{self, Fournisseur};
use crate::agents::json::Json;

const LISTE: &str = r#"{"models":[
 {"name":"models/gemini-2.5-flash","supportedGenerationMethods":["generateContent"]},
 {"name":"models/gemini-2.5-pro","supportedGenerationMethods":["generateContent"]},
 {"name":"models/gemini-3.1-pro-preview","supportedGenerationMethods":["generateContent"]},
 {"name":"models/gemini-2.5-flash-image","supportedGenerationMethods":["generateContent"]},
 {"name":"models/text-embedding-004","supportedGenerationMethods":["embedContent"]},
 {"name":"models/gemini-2.5-flash-lite","supportedGenerationMethods":["generateContent"]}
]}"#;

#[test]
fn les_modeles_de_texte_sortent_du_plus_capable_au_moins_capable() {
    let json = Json::lire(LISTE).expect("un JSON bien formé");
    let ids: Vec<String> = modeles::gemini(&json).into_iter().map(|m| m.id).collect();
    assert_eq!(
        ids,
        [
            "gemini-3.1-pro-preview",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
        ]
    );
}

#[test]
fn une_image_un_plongement_ou_une_reponse_vide_ne_sont_pas_des_modeles() {
    let json = Json::lire(r#"{"models":[{"name":"models/gemini-2.5-flash-image","supportedGenerationMethods":["generateContent"]}]}"#).unwrap();
    assert!(modeles::gemini(&json).is_empty());
    assert!(modeles::gemini(&Json::lire("{}").unwrap()).is_empty());
}

#[test]
fn les_autres_familles_ne_faussent_pas_le_rang() {
    let json = Json::lire(r#"{"models":[
     {"name":"models/antigravity-preview-09-2026","supportedGenerationMethods":["generateContent"]},
     {"name":"models/deep-research-max-preview-04-2026","supportedGenerationMethods":["generateContent"]},
     {"name":"models/gemini-2.5-flash","supportedGenerationMethods":["generateContent"]}
    ]}"#)
    .unwrap();
    let ids: Vec<String> = modeles::gemini(&json).into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["gemini-2.5-flash"]);
}

#[test]
fn le_rang_met_la_version_avant_la_gamme() {
    assert!(modeles::rang_gemini("gemini-3.1-flash-lite") > modeles::rang_gemini("gemini-2.5-pro"));
    assert!(modeles::rang_gemini("gemini-2.5-pro") > modeles::rang_gemini("gemini-2.5-flash"));
    assert!(
        modeles::rang_gemini("gemini-2.5-flash") > modeles::rang_gemini("gemini-2.5-flash-lite")
    );
}

#[test]
fn les_fournisseurs_sont_bien_distingues() {
    let json = Json::lire(LISTE).unwrap();
    assert!(modeles::gemini(&json)
        .iter()
        .all(|m| m.fournisseur == Fournisseur::Google));
}

fn dossier_d_essai(nom: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-ia-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn les_cles_se_relisent_telles_qu_ecrites() {
    let d = dossier_d_essai("relire");
    let chemin = d.join("ia-cles.json");
    let cles = Cles {
        google: Some("AQ.exemple-google".into()),
        github: None,
    };
    cles::ecrire_dans(&chemin, &cles).unwrap();
    assert_eq!(cles::lire_depuis(&chemin), cles);
}

#[test]
fn un_fichier_absent_ou_casse_ne_branche_rien() {
    let d = dossier_d_essai("absent");
    assert_eq!(cles::lire_depuis(&d.join("rien.json")), Cles::default());
    let casse = d.join("casse.json");
    std::fs::write(&casse, "{ pas du json").unwrap();
    assert_eq!(cles::lire_depuis(&casse), Cles::default());
}

#[test]
fn une_cle_vide_ne_compte_pas_comme_branchee() {
    let d = dossier_d_essai("vide");
    let chemin = d.join("ia-cles.json");
    std::fs::write(&chemin, r#"{"google":"   ","github":"ghp_x"}"#).unwrap();
    let lu = cles::lire_depuis(&chemin);
    assert_eq!(lu.google, None);
    assert_eq!(lu.github.as_deref(), Some("ghp_x"));
}

#[cfg(unix)]
#[test]
fn le_fichier_des_cles_est_a_ses_seuls_droits() {
    use std::os::unix::fs::PermissionsExt;
    let d = dossier_d_essai("droits");
    let chemin = d.join("ia-cles.json");
    cles::ecrire_dans(
        &chemin,
        &Cles {
            google: Some("k".into()),
            github: None,
        },
    )
    .unwrap();
    let mode = std::fs::metadata(&chemin).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

/// Le vrai réseau : `GLUCOSE_CLE_GOOGLE` porte la clé, et la clé n'est écrite nulle part.
/// Lancée à la main seulement : `cargo test -- --ignored essai_reel_google`.
#[cfg(not(windows))]
#[test]
#[ignore = "touche le réseau et une vraie clé"]
fn essai_reel_google() {
    let cle = std::env::var("GLUCOSE_CLE_GOOGLE").expect("GLUCOSE_CLE_GOOGLE");
    let bilan = super::essai::essayer_google(&cle).expect("l'essai répond");
    assert!(!bilan.modeles.is_empty(), "des modèles de texte");
    assert!(!bilan.qui_repond.is_empty(), "au moins un modèle répond");
}
