//! **RANGER-1 par la porte** (fiche 134) : une IA range une note et sa réponse dans l'archive, en
//! un geste que l'humain défait d'un `Ctrl+Z` ; la lecture du tableau du dossier les trouve.

use super::{appeler, args, board};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::CanvasFolder;

/// Les identifiants que la lecture de ce tableau rend.
fn lus(app: &mut GlucoseApp, tableau: &str) -> Vec<String> {
    let r = appeler(
        app,
        "glucose_lire",
        &format!(r#"{{"tableau":"{tableau}"}}"#),
    );
    r.get("noeuds")
        .and_then(Json::as_liste)
        .map(|l| {
            l.iter()
                .filter_map(|n| n.get("id").and_then(Json::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Le dossier « archive », et une note reliée à sa réponse, posées par une IA.
fn une_archive() -> (GlucoseApp, String, String) {
    let mut app = GlucoseApp::new();
    let racine = board(&app);
    let mut f = CanvasFolder::new("archive", "archive", "");
    (f.x, f.y) = (4000.0, 0.0);
    app.store.create_folder(&racine, f);
    let ouvre = app.store.active_board().expect("un tableau").folders[0]
        .child_board_id
        .clone();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"bogue","texte":"Un bogue."}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"rep","texte":"Corrigé."}"#,
    );
    appeler(
        &mut app,
        "glucose_relier",
        r#"{"id":"f","source":"bogue","cible":"rep"}"#,
    );
    (app, racine, ouvre)
}

/// **Une IA range la note et sa réponse dans l'archive** : elles quittent le tableau, la lecture du
/// tableau du dossier les trouve, flèche comprise ; un `Ctrl+Z` les rend.
#[test]
fn test_ranger_1_une_ia_range_dans_l_archive_et_ctrl_z_rend() {
    let (mut app, racine, ouvre) = une_archive();
    let r = appeler(
        &mut app,
        "glucose_ranger",
        r#"{"ids":["bogue","rep"],"dossier":"archive"}"#,
    );
    assert_eq!(r.get("ouvre").and_then(Json::as_str), Some(ouvre.as_str()));
    assert_eq!(r.get("ranges").and_then(Json::as_f64), Some(3.0));
    let dedans = lus(&mut app, &ouvre);
    for id in ["bogue", "rep", "f"] {
        assert!(
            dedans.iter().any(|i| i == id),
            "{id} dans l'archive : {dedans:?}"
        );
    }
    assert!(!lus(&mut app, &racine).iter().any(|i| i == "bogue"));
    assert!(app.store.undo(), "un geste");
    assert!(lus(&mut app, &racine).iter().any(|i| i == "bogue"));
    assert!(lus(&mut app, &ouvre).is_empty());
}

/// **Un refus se dit à l'IA**, et rien ne change.
#[test]
fn test_ranger_1_un_refus_se_dit() {
    let (mut app, _, _) = une_archive();
    let r = app.executer_pour_un_agent(
        "glucose_ranger",
        &args(r#"{"ids":["bogue"],"dossier":"rep"}"#),
    );
    assert_eq!(
        r.err().as_deref(),
        Some("« rep » n'est pas un dossier de ce tableau")
    );
}

/// **Une IA du nuage range aussi** : l'outil ne regarde que le document ouvert ; il écrit, donc
/// « Lire seulement » ne le donne pas.
#[test]
fn test_ranger_1_ouvert_au_nuage_qui_ecrit() {
    use crate::agents::outils::{permis, Portee};
    let nuage = Portee {
        machine: false,
        ecrire: true,
    };
    assert!(permis("glucose_ranger", nuage));
    let lecteur = Portee {
        machine: true,
        ecrire: false,
    };
    assert!(!permis("glucose_ranger", lecteur));
}
