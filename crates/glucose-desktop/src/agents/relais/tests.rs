//! RELAIS-1 — le relais répond seul jusqu'au premier outil.

use super::*;
use std::io::Cursor;

const INITIALISER: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"claude-code"}}}"#;
const INITIALISE: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LISTER: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const APPELER: &str =
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"glucose_lire"}}"#;

fn reponses(sortie: Vec<u8>) -> Vec<Json> {
    String::from_utf8(sortie)
        .expect("UTF-8")
        .lines()
        .map(|l| Json::lire(l).expect("JSON"))
        .collect()
}

#[test]
fn relais_1_la_poignee_de_main_et_la_liste_se_repondent_sans_glucose() {
    let entree = [INITIALISER, INITIALISE, LISTER].join("\n");
    let mut sortie = Vec::new();
    let suite = repondre_seul(&mut Cursor::new(entree.into_bytes()), &mut sortie);
    assert!(
        suite.is_none(),
        "le client a fermé avant tout outil : rien à brancher"
    );
    let r = reponses(sortie);
    assert_eq!(r.len(), 2);
    assert!(r[1].ecrire().contains("glucose_poser_page"));
}

#[test]
fn relais_1_le_premier_outil_rend_la_poignee_a_rejouer() {
    let entree = [INITIALISER, INITIALISE, LISTER, APPELER, LISTER].join("\n");
    let mut sortie = Vec::new();
    let (prelude, premier) =
        repondre_seul(&mut Cursor::new(entree.into_bytes()), &mut sortie).expect("un outil appelé");
    assert_eq!(
        prelude,
        [INITIALISER, INITIALISE],
        "la liste ne se rejoue pas"
    );
    assert_eq!(premier, APPELER);
    assert_eq!(
        reponses(sortie).len(),
        2,
        "l'appel part à Glucose, pas au relais"
    );
    assert_eq!(
        prelude.iter().filter(|l| a_une_reponse(l)).count(),
        1,
        "une réponse à taire"
    );
}
