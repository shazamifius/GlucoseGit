//! **Qui a écrit quoi, et quand** (MCP-AUTEUR, fiche 181 ; `relais-rep-mcp`, point 8 : *« qui a
//! écrit quoi : l'auteur et la date de chaque nœud »*). Pour chaque nœud, les mains qui l'ont
//! écrit — posé, ou changé son texte —, la première d'abord, et l'instant de la dernière fois.
//! Le déplacer, l'agrandir, le colorer n'est pas l'écrire.
//!
//! Relue dans l'histoire à l'ouverture, puis suivie à chaque geste écrit au disque, à l'instant
//! que l'histoire lui donne : la date qu'une IA lit est celle de la Time Machine. Le noyau n'a pas
//! d'horloge (il se construit aussi pour le web) : l'instant lui est donné.

use super::effets::Effet;
use super::Transaction;
use std::collections::HashMap;

/// Les mots de l'humain de ce Glucose : il n'a pas d'auteur déclaré dans l'histoire.
pub const HUMAIN: &str = "humain";

/// **Ce qu'on sait de l'écriture d'un nœud.**
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Provenance {
    /// Les mains qui l'ont écrit, chacune une fois, la première d'abord : « humain », un agent,
    /// un participant, par leurs mots ([`crate::persist::histoire::Auteur::en_mots`]).
    pub par: Vec<String>,
    /// La dernière fois qu'on l'a écrit, en millisecondes depuis 1970.
    pub le: i64,
}

/// Par tableau, puis par identifiant de nœud.
pub type Provenances = HashMap<String, HashMap<String, Provenance>>;

/// **Ce qu'un geste écrit** : un nœud posé, ou dont le texte change, est écrit par `qui` à
/// `instant`. Retiré, il n'a plus de provenance : reposé, il repart de qui le pose, et un
/// identifiant repris plus tard ne porte pas les mains d'un autre nœud.
pub fn suivre_l_ecriture(
    provenances: &mut Provenances,
    transaction: &Transaction,
    qui: &str,
    instant: i64,
) {
    for effet in transaction.effets() {
        let Effet::Noeud(t) = effet else {
            continue;
        };
        if !t.apres {
            if let Some(tableau) = provenances.get_mut(&t.board) {
                tableau.remove(&t.id);
            }
            continue;
        }
        if t.avant && t.textes[0] == t.textes[1] {
            continue;
        }
        let p = (provenances.entry(t.board).or_default())
            .entry(t.id)
            .or_default();
        if !p.par.iter().any(|m| m == qui) {
            p.par.push(qui.to_string());
        }
        p.le = instant;
    }
}

#[cfg(test)]
mod tests;
