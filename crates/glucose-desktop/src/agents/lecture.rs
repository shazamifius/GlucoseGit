//! **Lire par niveaux** (fiche 79) : un agent ne reprend que ce dont il a besoin.
//!
//! Lu d'un bloc, un tableau rend le texte entier de chacun de ses nœuds : pour un petit modèle de
//! langage, dont la fenêtre se remplit, c'est l'inverse d'une mémoire. Protéo l'a vécu le
//! 21/10/2026 : un manuel de 1 037 lignes lu en entier occupait presque toute sa fenêtre. Le
//! **sommaire** dit de chaque nœud ce qu'il est, où il est, sa première phrase et sa taille ;
//! l'agent lit ensuite **en entier** les nœuds qu'il choisit (`ids`), ou **cherche** une phrase
//! dans tous (le passage trouvé, pas le texte). Aucune longueur n'est fixée ici : la phrase est
//! l'unité, celle du texte lui-même.
//!
//! **Le plan** (fiche 90) : sur un grand tableau, le sommaire lui-même déborde — l'article « Eau » de
//! Wikipédia, posé par les commandes, y pesait 140 000 caractères. Le plan ne rend que les
//! **membranes**, emboîtées, et ce que chacune contient : la géographie de la carte, d'où l'agent
//! descend ensuite (`dans`). Sa règle : « une IA doit savoir où se situe quoi » (VISION, 22/10).

use glucose_core::hash::{hex_of, sha256};
use glucose_core::store::agents::{Decrit, Sorte};

/// **L'empreinte d'un texte** : les seize premiers chiffres de son sha256 — un mot machine, 64
/// bits. Un agent la garde d'une lecture à l'autre : si elle a changé, l'utilisateur a récrit le
/// nœud, et il ne relit que celui-là. Calculée à la lecture, jamais rangée : la porte reste sans
/// état, et le fichier ne change pas.
pub fn empreinte(texte: &str) -> String {
    let mut e = hex_of(&sha256(texte.as_bytes()));
    e.truncate(16);
    e
}

/// Ce qu'un agent demande à lire.
pub struct Demande<'a> {
    /// Seulement les nœuds qui portent ce domaine (son identifiant dans ce document).
    pub domaine: Option<&'a str>,
    /// Seulement ce qu'une membrane contient (elle comprise).
    pub dans: Option<&'a str>,
    /// Ces nœuds, en entier.
    pub ids: Option<Vec<&'a str>>,
    /// Les nœuds dont le texte contient ces mots (sans tenir compte des majuscules).
    pub cherche: Option<&'a str>,
    /// Le sommaire, le texte entier de tous les nœuds, ou le seul plan des membranes.
    pub niveau: Niveau,
}

/// **Le niveau d'une lecture.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Niveau {
    /// Chaque nœud, sa première phrase et sa taille.
    #[default]
    Sommaire,
    /// Le texte entier de chaque nœud, comme avant les niveaux.
    Complet,
    /// Les seules membranes : la carte, sans ce qu'elle porte.
    Plan,
}

impl Niveau {
    /// Le niveau qu'un agent nomme ; un nom inconnu est refusé, pas deviné.
    pub fn lire(nom: Option<&str>) -> Result<Self, String> {
        match nom {
            None | Some("sommaire") => Ok(Self::Sommaire),
            Some("complet") => Ok(Self::Complet),
            Some("plan") => Ok(Self::Plan),
            Some(autre) => Err(format!(
                "le niveau « {autre} » n'existe pas : sommaire, complet ou plan"
            )),
        }
    }
}

/// Comment un nœud est rendu.
#[derive(Debug, PartialEq)]
pub enum Rendu {
    /// Sa première phrase et sa taille, pas son texte.
    Sommaire,
    /// Son texte entier.
    Entier,
    /// La phrase qui contient ce qu'on cherche.
    Passage(String),
}

// La phrase est l'unité, celle du texte lui-même, et la même loi sert `Ctrl+F` : elle vit dans le
// noyau (`glucose_core::recherche`), sans majuscules ni accents (PLI-1).
pub use glucose_core::recherche::{passage, phrases, premiere_phrase};

/// Les nœuds à rendre, et comment : `ids` d'abord, sinon `cherche`, sinon tous en sommaire (ou
/// en entier, si l'agent l'a demandé).
pub fn choisir<'d>(decrits: &'d [Decrit], d: &Demande<'_>) -> Vec<(&'d Decrit, Rendu)> {
    let dans = |n: &Decrit| {
        d.dans
            .is_none_or(|m| n.membrane.as_deref() == Some(m) || n.id == m)
            // DOM-6 : ceux qui le portent, et ceux qui l'héritent de leurs membranes.
            && d.domaine.is_none_or(|x| n.poids_dans(x).is_some())
    };
    let plan = d.niveau == Niveau::Plan;
    let pris = decrits
        .iter()
        .filter(|n| dans(n) && (!plan || n.sorte == Sorte::Membrane));
    if let Some(ids) = &d.ids {
        return pris
            .filter(|n| ids.contains(&n.id.as_str()))
            .map(|n| (n, Rendu::Entier))
            .collect();
    }
    if let Some(c) = d.cherche {
        return pris
            .filter_map(|n| {
                let p = passage(n.texte.as_deref()?, c)?;
                Some((n, Rendu::Passage(p)))
            })
            .collect();
    }
    let rendu = || match d.niveau {
        Niveau::Complet => Rendu::Entier,
        Niveau::Sommaire | Niveau::Plan => Rendu::Sommaire,
    };
    pris.map(|n| (n, rendu())).collect()
}

#[cfg(test)]
mod tests;
