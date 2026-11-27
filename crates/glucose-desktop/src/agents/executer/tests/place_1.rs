//! **PLACE-1** (fiche 185 § 4) : le cadre d'une membrane qu'une IA redimensionne ou déplace.

use super::{appeler, board};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::geometry::Rect;

fn vide() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app
}

fn boite(app: &GlucoseApp, id: &str) -> Rect {
    let b = board(app);
    app.store.boite_d_un_noeud(&b, id).expect("le nœud")
}

/// Une membrane « m » en (0, 0), et un texte « t » posé dedans.
fn une_membrane_et_son_texte() -> GlucoseApp {
    let mut app = vide();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"M","x":0,"y":0,"largeur":800,"hauteur":600}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"t","texte":"Dedans.","dans":"m","x":400,"y":300}"#,
    );
    app
}

/// **Rétrécie par une IA, elle garde son contenu** : jamais plus serrée que lui plus l'écart.
#[test]
fn place_1_une_membrane_retrecie_garde_son_contenu() {
    let mut app = une_membrane_et_son_texte();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","largeur":100,"hauteur":100}"#,
    );
    let (m, t) = (boite(&app, "m"), boite(&app, "t"));
    assert!(m.contains_rect(t), "{m:?} ne contient plus {t:?}");
    assert!(
        m.right() >= t.right() + 30.0 && m.bottom() >= t.bottom() + 30.0,
        "{m:?}"
    );
    let dit = appeler(&mut app, "glucose_verifier", "{}");
    assert_eq!(
        dit.get("propre"),
        Some(&Json::Bool(true)),
        "{}",
        dit.ecrire()
    );
}

/// **Déplacée par une IA, son contenu la suit** : le cadre ne part pas seul.
#[test]
fn place_1_une_membrane_deplacee_emporte_son_contenu() {
    let mut app = une_membrane_et_son_texte();
    let t = boite(&app, "t");
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","x":2000,"y":0}"#,
    );
    let (m, t2) = (boite(&app, "m"), boite(&app, "t"));
    assert_eq!((m.left, t2.left - t.left), (2000.0, 2000.0));
    assert!(m.contains_rect(t2), "{m:?} {t2:?}");
}

/// **Agrandie sur une voisine, elle le dit** : le cadre demandé est tenu, et l'IA est avertie.
#[test]
fn place_1_une_membrane_agrandie_sur_un_voisin_le_dit() {
    let mut app = une_membrane_et_son_texte();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"v","texte":"Voisin.","x":1000,"y":100}"#,
    );
    let dit = appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","largeur":700,"hauteur":600}"#,
    );
    assert!(dit.get("attention").is_none(), "{}", dit.ecrire());
    let dit = appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","largeur":1500,"hauteur":600}"#,
    );
    assert!(dit.get("attention").is_some(), "{}", dit.ecrire());
}
