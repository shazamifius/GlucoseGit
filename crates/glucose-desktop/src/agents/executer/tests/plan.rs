//! Le plan d'un grand tableau, et des coordonnées au plus court (fiche 90).

use super::{appeler, args};
use crate::agents::json::Json;
use crate::app::GlucoseApp;

/// **Le plan ne rend que les membranes, et ce que chacune contient** : la carte, sans ce qu'elle
/// porte. Un niveau inconnu est refusé, pas deviné.
#[test]
fn le_plan_rend_les_membranes_et_ce_qu_elles_contiennent() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"eau","titre":"Eau","x":0,"y":0}"#,
    );
    for (id, t) in [("a", "L'eau mouille."), ("b", "Eau = H2O.")] {
        let a = format!(r#"{{"id":"{id}","texte":"{t}","dans":"eau"}}"#);
        appeler(&mut app, "glucose_poser_texte", &a);
    }
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"dehors","texte":"Le feu.","x":5000,"y":0}"#,
    );
    let r = appeler(&mut app, "glucose_lire", r#"{"niveau":"plan"}"#);
    let noeuds = r.get("noeuds").and_then(Json::as_liste).expect("des nœuds");
    let ids: Vec<&str> = noeuds
        .iter()
        .filter_map(|n| n.get("id").and_then(Json::as_str))
        .collect();
    assert_eq!(ids, ["eau"], "les seules membranes");
    assert_eq!(
        noeuds[0].get("contient").and_then(Json::as_f64),
        Some(2.0),
        "ce qu'elle porte"
    );
    assert_eq!(
        noeuds[0].get("debut").and_then(Json::as_str),
        Some("Eau"),
        "son titre"
    );
    let sommaire = appeler(&mut app, "glucose_lire", "{}");
    let tous = sommaire
        .get("noeuds")
        .and_then(Json::as_liste)
        .expect("des nœuds");
    assert!(tous.len() > ids.len(), "le sommaire, lui, rend tout");
    assert!(tous.iter().all(|n| n.get("contient").is_none()));
    let refus = app.executer_pour_un_agent("glucose_lire", &args(r#"{"niveau":"carte"}"#));
    assert!(
        refus.is_err_and(|e| e.contains("plan")),
        "un niveau inconnu est refusé"
    );
}

/// **COORD-1 : une coordonnée au plus court, sans rien perdre** — chaque point de la grille du
/// document (1/2⁸ d'unité) revient exactement de son écriture ; la hauteur mesurée en `f32` ne traîne
/// plus ses quinze chiffres.
#[test]
fn coord_1_une_coordonnee_au_plus_court_sans_rien_perdre() {
    let grille = f64::from(1u32 << glucose_core::fixed::FRAC_BITS);
    for k in -2000..2000 {
        let v = 13777.0 + f64::from(k) / grille;
        let Json::Nombre(ecrit) = super::super::decrit::coordonnee(v) else {
            panic!("un nombre");
        };
        assert_eq!((ecrit * grille).round() / grille, v, "{v} écrit {ecrit}");
    }
    let hauteur = f64::from(51.6f32);
    assert_eq!(
        super::super::decrit::coordonnee(hauteur),
        Json::Nombre(51.6)
    );
    assert_eq!(Json::Nombre(51.6).ecrire(), "51.6");
}
