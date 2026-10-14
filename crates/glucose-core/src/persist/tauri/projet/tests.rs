//! Le lecteur des projets de Glucose Tauri : la grille du storyboard (STORYBOARD-2), que le
//! lecteur omettait.

use super::*;
use crate::persist::tauri::json;

/// Un projet de Tauri à un tableau, dont le storyboard porte ces réglages.
fn projet(storyboard: &str) -> (Project, Rapport) {
    let texte =
        format!(r#"{{"name":"P","boards":[{{"id":"b","name":"B","storyboard":{storyboard}}}]}}"#);
    traduire(&json::lire(&texte).expect("du JSON"))
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
        p.boards[0].storyboard,
        Some(crate::types::Storyboard::neuf(attendue)),
        "la grille de Tauri, aux réglages de naissance"
    );
    assert!(r.omis.is_empty(), "rien d'omis : {:?}", r.omis);
}

/// **Un format que Glucose ne connaît pas se dit au rapport** plutôt que de se deviner ; un
/// tableau sans storyboard n'en a pas.
#[test]
fn test_un_format_inconnu_se_dit_et_l_absence_reste_absente() {
    let (p, r) = projet(r#"{"aspectRatio":"3:2","panelWidth":320,"cols":3,"gap":16}"#);
    assert_eq!(p.boards[0].storyboard, None);
    assert_eq!(r.omis.get("format de storyboard inconnu"), Some(&1));
    let texte = r#"{"name":"P","boards":[{"id":"b","name":"B"}]}"#;
    let (p, _) = traduire(&json::lire(texte).expect("du JSON"));
    assert_eq!(p.boards[0].storyboard, None);
}
