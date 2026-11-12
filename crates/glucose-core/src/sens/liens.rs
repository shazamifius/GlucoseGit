//! **PARENTE-4 — la sûreté d'un lien** (fiche 107) : qu'une flèche relie deux choses qui parlent
//! de la même chose.
//!
//! Ses mots du 11/11/2026, sur la carte de Wikipédia : *« trop de flèches partout »*, *« on a juste
//! des flèches random »*, et *« passer par l'intelligence artificielle pour que ça fasse sens »*.
//! Les lecteurs de Wikipédia font ce tri d'eux-mêmes : ils suivent surtout les liens vers des
//! articles proches par le sens (Dimitrov, Singer, Lemmerich et Strohmaier, WWW 2017), et beaucoup
//! de liens sont posés par habitude de l'auteur plutôt que par parenté (West et al., 2009).
//!
//! # Le sens d'une région
//!
//! Une flèche de la carte va du texte d'un article à la **membrane** d'un autre. Une membrane dit ce
//! qu'elle porte : son sens est la somme des sens de ses textes — ceux qu'elle porte elle-même et
//! ceux de ses membranes filles — et de son nom, ramenée à 1 ([`Region::de`]).
//!
//! # Le hasard d'une cible
//!
//! « Sûr » veut dire : la source ressemble à la cible plus qu'un texte du document pris au hasard.
//! Ce hasard se lit **sur la cible**, et **hors d'elle** :
//!
//! * **sur la cible** : la somme de beaucoup de textes ressemble un peu à tout — ils partagent la
//!   direction commune de la langue —, et contre le hasard du document (deux textes au hasard),
//!   toute flèche vers une grande région paraissait sûre. La ressemblance de chaque texte du
//!   document à la cible, lue par sa médiane et son écart absolu médian (PARENTE-1), dit le hasard
//!   propre à cette cible ;
//! * **hors d'elle** : les textes qu'une région porte lui ressemblent forcément ; comptés dans son
//!   hasard, ils le haussaient jusqu'à rendre nulle « Océan → Eau » (la région « Eau » porte des
//!   centaines de paragraphes de l'article). Un texte se porte lui-même.
//!
//! # La loi
//!
//! La paire est **donnée**, pas cherchée parmi d'autres : sa chance contre le hasard de sa cible est
//! celle d'une seule comparaison, `1 − Φ(z)`, sans Šidák ; puis le **taux local de fausses
//! découvertes** d'Efron sur les chances de tous les liens du document (PARENTE-1). Ce n'est pas la
//! sûreté de ses bouts : deux textes chacun d'une famille sûre peuvent être reliés par habitude.
//!
//! Mesuré sur la carte de Wikipédia (le modèle entier, en Python, fiche 107) : sur 1 746 liens d'un
//! texte vers une région, 39 % au-dessus de 0,5 et 36 % nuls ; vers « Eau », « Atmosphère
//! terrestre » 0,93, « Océan » 0,72, « Lune » 0 ; « Lune → Système solaire » sûr, « Mathématiques →
//! Afrique » nul.

use super::familles::{efron, Hasard};
use super::{ressemblance, textes_du_tableau};
use crate::membrane_space::{items_of_board, parent_map};
use crate::types::{Annotation, Board};
use std::collections::{HashMap, HashSet};

/// **Une région** : son sens, et les rangs des textes qu'elle porte, rangés.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub sens: Vec<f32>,
    pub porte: Vec<usize>,
}

impl Region {
    /// **Le sens d'une région** : la somme des sens des textes qu'elle porte (des rangs de `sens`)
    /// et de son nom, ramenée à 1 ; rien si elle ne porte rien et n'a pas de nom.
    pub fn de(sens: &[Vec<f32>], mut porte: Vec<usize>, nom: Option<&[f32]>) -> Option<Self> {
        porte.sort_unstable();
        porte.dedup();
        let d = sens
            .first()
            .map_or_else(|| nom.map_or(0, <[f32]>::len), Vec::len);
        let mut somme = vec![0.0f32; d];
        for v in porte.iter().map(|&t| sens[t].as_slice()).chain(nom) {
            for (s, x) in somme.iter_mut().zip(v) {
                *s += x;
            }
        }
        let norme = somme.iter().map(|x| x * x).sum::<f32>().sqrt();
        (norme > 0.0).then(|| Self {
            sens: somme.into_iter().map(|x| x / norme).collect(),
            porte,
        })
    }
}

/// **Un bout de lien** : un texte du document, ou une région — leurs rangs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bout {
    Texte(usize),
    Region(usize),
}

/// **La sûreté de chaque lien**, de 0 à 1 : que sa source et sa cible parlent de la même chose,
/// contre le hasard de sa cible, hors d'elle.
pub fn surete_des_liens(sens: &[Vec<f32>], regions: &[Region], liens: &[(Bout, Bout)]) -> Vec<f32> {
    let vecteur = |b: Bout| match b {
        Bout::Texte(t) => sens[t].as_slice(),
        Bout::Region(r) => regions[r].sens.as_slice(),
    };
    let mut hasards: HashMap<Bout, Option<Hasard>> = HashMap::new();
    let chances: Vec<f64> = liens
        .iter()
        .map(|&(source, cible)| {
            let hasard = *hasards.entry(cible).or_insert_with(|| {
                let dedans = |t: usize| match cible {
                    Bout::Texte(c) => t == c,
                    Bout::Region(r) => regions[r].porte.binary_search(&t).is_ok(),
                };
                let mut dehors: Vec<f32> = (0..sens.len())
                    .filter(|&t| !dedans(t))
                    .map(|t| ressemblance(&sens[t], vecteur(cible)))
                    .collect();
                Hasard::de(&mut dehors)
            });
            hasard.map_or(1.0, |h| {
                h.chance(ressemblance(vecteur(source), vecteur(cible)), 1)
            })
        })
        .collect();
    efron(&chances)
}

/// Une membrane que touche une flèche : son identité, son nom, et les textes qu'elle porte.
pub type RegionTouchee = (String, Option<String>, Vec<String>);

/// **Les flèches d'un tableau qu'un agent a posées** (`des_agents`, MAIN-1) dont les deux bouts
/// sont nommés — la flèche, sa source, sa cible —, et les membranes qu'elles touchent : leur nom,
/// et les textes qu'elles portent, filles comprises. Dans l'ordre du document.
///
/// Une flèche que l'humain a tirée n'y est pas (fiche 108) : *« il ne faut pas que l'IA juge les
/// flèches, mais qu'elle comprenne leur importance »*.
pub fn liens_du_tableau(
    tableau: &Board,
    des_agents: &HashSet<String>,
) -> (Vec<[String; 3]>, Vec<RegionTouchee>) {
    let fleches: Vec<[String; 3]> = tableau
        .annotations
        .iter()
        .filter_map(|a| match a {
            Annotation::Arrow {
                id,
                source_id: Some(s),
                target_id: Some(c),
                ..
            } if des_agents.contains(id) => Some([id.clone(), s.clone(), c.clone()]),
            _ => None,
        })
        .collect();
    let touchees: HashSet<&str> = fleches
        .iter()
        .flat_map(|[_, s, c]| [s.as_str(), c.as_str()])
        .collect();
    let mut regions: Vec<(String, Option<String>, Vec<String>)> = tableau
        .annotations
        .iter()
        .filter_map(|a| match a {
            Annotation::Membrane { id, text, .. } if touchees.contains(id.as_str()) => {
                Some((id.clone(), text.clone(), Vec::new()))
            }
            _ => None,
        })
        .collect();
    if !regions.is_empty() {
        let place: HashMap<String, usize> = regions
            .iter()
            .enumerate()
            .map(|(k, r)| (r.0.clone(), k))
            .collect();
        // La table des mères est sans cycle : la remonter finit.
        let meres = parent_map(&items_of_board(tableau));
        for (texte, _) in textes_du_tableau(tableau) {
            let mut mere = meres.get(texte);
            while let Some(m) = mere {
                if let Some(&k) = place.get(m) {
                    regions[k].2.push(texte.to_string());
                }
                mere = meres.get(m);
            }
        }
    }
    (fleches, regions)
}

#[cfg(test)]
mod tests;
