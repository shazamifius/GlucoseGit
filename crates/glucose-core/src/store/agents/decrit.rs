//! **Un nœud, tel qu'un agent le lit** : ce qu'il est, où il est, ce qu'il dit, à quels domaines il
//! appartient. La même description sert au tableau qu'on regarde et au recensement des fichiers
//! fermés (`recensement`) : un agent lit un nœud de la même façon, où qu'il soit.

use super::domaines::{decrire_les_domaines, DomaineDecrit};
use super::{texte_porte, Sorte};
use crate::geometry::Rect;
use crate::types::{Annotation, ArrowPredicate, Board, Domain};

/// **Un nœud, tel qu'un agent le lit.**
#[derive(Debug, Clone, PartialEq)]
pub struct Decrit {
    pub id: String,
    pub sorte: Sorte,
    pub boite: Rect,
    pub texte: Option<String>,
    pub membrane: Option<String>,
    /// Pour une flèche : ce qu'elle relie, et la relation qu'elle dit.
    pub source: Option<String>,
    pub cible: Option<String>,
    pub relation: Option<ArrowPredicate>,
    /// Pour une photo : l'adresse d'où elle vient — la page qu'elle montre.
    pub adresse: Option<String>,
    /// Les domaines qu'il porte, leur nom et leur poids.
    pub domaines: Vec<DomaineDecrit>,
}

/// **Chaque photo et chaque annotation d'un tableau**, avec sa boîte et son texte.
pub(crate) fn decrire_le_tableau(board: &Board, catalogue: &[Domain]) -> Vec<Decrit> {
    let photos = board.images.iter().map(|i| Decrit {
        id: i.id.clone(),
        sorte: Sorte::Photo,
        boite: i.rect(),
        texte: None,
        membrane: i.membrane_id.clone(),
        source: None,
        cible: None,
        relation: None,
        adresse: i.source_url.clone(),
        domaines: decrire_les_domaines(catalogue, &i.domains),
    });
    let annotations = board
        .annotations
        .iter()
        .map(|a| decrire_une_annotation(a, catalogue));
    photos.chain(annotations).collect()
}

fn decrire_une_annotation(a: &Annotation, catalogue: &[Domain]) -> Decrit {
    let (source, cible, relation) = match a {
        Annotation::Arrow {
            source_id,
            target_id,
            predicate,
            ..
        } => (source_id.clone(), target_id.clone(), *predicate),
        _ => (None, None, None),
    };
    Decrit {
        id: a.id().to_string(),
        sorte: Sorte::de(a),
        boite: a.bounds(),
        texte: texte_porte(a),
        membrane: a.membrane_id().map(str::to_string),
        source,
        cible,
        relation,
        adresse: None,
        domaines: decrire_les_domaines(catalogue, a.domains()),
    }
}
