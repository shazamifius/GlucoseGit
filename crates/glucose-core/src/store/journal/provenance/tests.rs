//! MCP-AUTEUR — qui a écrit quoi, et quand (fiche 181).

use super::*;
use crate::store::journal::{Edit, Slot};
use crate::types::Annotation;

fn geste(avant: Option<&Annotation>, apres: Option<&Annotation>) -> Transaction {
    Transaction {
        edits: vec![Edit::Annotation {
            board: "b".to_string(),
            slot: Slot {
                index: 0,
                before: avant.cloned().map(Box::new),
                after: apres.cloned().map(Box::new),
            },
        }],
    }
}

fn de(p: &Provenances, id: &str) -> Option<(Vec<String>, i64)> {
    let n = p.get("b")?.get(id)?;
    Some((n.par.clone(), n.le))
}

/// **Écrire, c'est poser ou changer son texte** : chaque main une fois, la première d'abord, et
/// l'instant de la dernière fois ; le déplacer n'est pas l'écrire ; retiré, il n'a plus de
/// provenance ; reposé, il repart de qui le pose.
#[test]
fn mcp_auteur_qui_a_ecrit_quoi_et_quand() {
    let mut p = Provenances::new();
    let eau = Annotation::text("a", 0.0, 0.0, "Eau");
    suivre_l_ecriture(&mut p, &geste(None, Some(&eau)), "agent proteo", 10);
    let pluie = Annotation::text("a", 0.0, 0.0, "Eau et pluie");
    suivre_l_ecriture(&mut p, &geste(Some(&eau), Some(&pluie)), HUMAIN, 20);
    suivre_l_ecriture(&mut p, &geste(Some(&pluie), Some(&eau)), "agent proteo", 30);
    let par = vec!["agent proteo".to_string(), HUMAIN.to_string()];
    assert_eq!(de(&p, "a"), Some((par.clone(), 30)));

    let deplacee = Annotation::text("a", 500.0, 500.0, "Eau");
    suivre_l_ecriture(
        &mut p,
        &geste(Some(&eau), Some(&deplacee)),
        "agent rangeur",
        40,
    );
    assert_eq!(de(&p, "a"), Some((par, 30)), "déplacer n'est pas écrire");

    suivre_l_ecriture(&mut p, &geste(Some(&deplacee), None), HUMAIN, 50);
    assert_eq!(de(&p, "a"), None, "retiré");
    suivre_l_ecriture(&mut p, &geste(None, Some(&eau)), HUMAIN, 60);
    assert_eq!(de(&p, "a"), Some((vec![HUMAIN.to_string()], 60)), "reposé");
}
