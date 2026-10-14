//! Le lecteur des projets de Glucose Tauri : la grille du storyboard (STORYBOARD-2), que le
//! lecteur omettait.

use super::*;
use crate::persist::tauri::json;

/// Une case de Tauri.
const CASE: &str = r#"{"id":"p","order":0,"x":0,"y":0,"width":320,"height":136}"#;

/// Un projet de Tauri à un tableau, dont le storyboard porte ces réglages, et ces cases.
fn projet_avec(storyboard: &str, cases: &str) -> (Project, Rapport) {
    let texte = format!(
        r#"{{"name":"P","boards":[{{"id":"b","name":"B","storyboard":{storyboard},"panels":[{cases}]}}]}}"#
    );
    traduire(&json::lire(&texte).expect("du JSON"))
}

/// Un projet de Tauri à un tableau et une case, dont le storyboard porte ces réglages.
fn projet(storyboard: &str) -> (Project, Rapport) {
    projet_avec(storyboard, CASE)
}

/// **La grille de Tauri se lit**, champ par champ — aucun n'est la valeur par défaut, un champ
/// lu à la place d'un autre se verrait.
#[test]
fn test_la_grille_du_storyboard_de_tauri_se_lit() {
    let (p, r) = projet(r#"{"aspectRatio":"2.35:1","panelWidth":320,"cols":3,"gap":16}"#);
    let attendue = Grille {
        format: Format::Scope,
        largeur: 320.0,
        colonnes: 3,
        ecart: 16.0,
    };
    assert_eq!(
        p.boards[0].storyboards,
        vec![crate::types::Storyboard::neuf(STORYBOARD_D_AVANT, attendue)],
        "la grille de Tauri, aux réglages de naissance, sous le nom d'avant"
    );
    assert_eq!(p.boards[0].panels[0].storyboard, STORYBOARD_D_AVANT);
    assert!(r.omis.is_empty(), "rien d'omis : {:?}", r.omis);
}

/// **Des réglages sans aucune case ne font pas de storyboard** (SEQUENCE-1, fiche 68) : il ne
/// montrerait rien, et rien ne le désignerait.
#[test]
fn test_une_grille_sans_case_ne_fait_pas_de_storyboard() {
    let (p, _) = projet_avec(
        r#"{"aspectRatio":"16:9","panelWidth":320,"cols":3,"gap":16}"#,
        "",
    );
    assert!(p.boards[0].storyboards.is_empty());
}

/// **Un format que Glucose ne connaît pas se dit au rapport** plutôt que de se deviner ; un
/// tableau sans storyboard n'en a pas.
#[test]
fn test_un_format_inconnu_se_dit_et_l_absence_reste_absente() {
    let (p, r) = projet(r#"{"aspectRatio":"3:2","panelWidth":320,"cols":3,"gap":16}"#);
    assert_eq!(
        p.boards[0].storyboards,
        vec![crate::types::Storyboard::neuf(
            STORYBOARD_D_AVANT,
            Grille::default()
        )],
        "un format inconnu : la case garde un storyboard, à la grille de Tauri"
    );
    assert_eq!(r.omis.get("format de storyboard inconnu"), Some(&1));
    let texte = r#"{"name":"P","boards":[{"id":"b","name":"B"}]}"#;
    let (p, _) = traduire(&json::lire(texte).expect("du JSON"));
    assert!(p.boards[0].storyboards.is_empty());
}
