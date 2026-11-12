//! **MAIN-1 — qui fait un geste, et ce qu'il prend aux agents** (fiche 108) : la main de chaque
//! transaction, la provenance des nœuds, et ce qu'un geste de l'humain reprend — où le journal des
//! rectifications regarde (RECTIF-1, fiche 110). Extrait du journal, qui passait ses six cents
//! lignes.

use super::{Edit, Transaction};
use std::collections::HashSet;

/// **Qui fait un geste** (MAIN-1) : l'humain devant Glucose, ou un agent — sous le nom que son
/// client se donne, déclaré et jamais vérifié, comme la question « Autoriser » le montre.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Main {
    #[default]
    Humain,
    Agent(String),
}

/// **Ce qu'un geste change à la provenance des nœuds** (MAIN-1) : une annotation qu'un agent pose
/// est à lui ; celle que l'humain pose, change ou retire devient sienne ; une annotation retirée
/// n'est plus à personne. Un agent qui change ce que l'humain a fait ne le lui prend pas.
///
/// Rend **ce que l'humain vient de reprendre aux agents** (RECTIF-1, fiche 110) : les nœuds qui
/// étaient à un agent, et que ce geste de l'humain change ou retire — là où il corrige l'IA.
pub fn suivre_les_mains(
    des_agents: &mut HashSet<String>,
    transaction: &Transaction,
    agent: bool,
) -> Vec<String> {
    let mut repris = Vec::new();
    for edit in &transaction.edits {
        let Edit::Annotation { slot, .. } = edit else {
            continue;
        };
        let Some(id) = slot
            .after
            .as_deref()
            .or(slot.before.as_deref())
            .map(|a| a.id())
        else {
            continue;
        };
        if slot.after.is_none() || !agent {
            if des_agents.remove(id) && !agent {
                repris.push(id.to_string());
            }
        } else if slot.before.is_none() {
            des_agents.insert(id.to_string());
        }
    }
    repris
}

/// **Un geste appliqué, et la main qui l'a fait** : ce que la file de sortie porte (JRN-5,
/// MAIN-1).
#[derive(Debug, Clone)]
pub struct Ecrit {
    pub transaction: Transaction,
    pub main: Main,
    /// **Ce que ce geste de l'humain a repris aux agents** ([`suivre_les_mains`]) : là où le
    /// journal des rectifications regarde ce que l'IA en pensait (RECTIF-1).
    pub repris: Vec<String>,
}
