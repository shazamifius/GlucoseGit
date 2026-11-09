//! **Un nœud, tel qu'un agent le lit** : ce qu'il est, où il est, ce qu'il dit, à quels domaines il
//! appartient. La même description sert au tableau qu'on regarde et au recensement des fichiers
//! fermés (`recensement`) : un agent lit un nœud de la même façon, où qu'il soit.
//!
//! # DOM-6 — le nom d'une région vaut pour ce qu'elle contient (fiche 94)
//!
//! Un domaine posé sur une membrane vaut pour tout ce qu'elle porte, comme le nom d'un pays vaut
//! pour ses villes : posées sur l'article « Eau » de Wikipédia, ses sept catégories n'ont plus à se
//! répéter sur ses 198 paragraphes. **Rien ne s'écrit sur les nœuds** : l'héritage se lit, ici. Un
//! nœud garde ses domaines **propres** (le sommaire ne dit qu'eux, le compte des endroits aussi), et
//! porte à côté ceux qu'il **hérite** : de la membrane la plus proche qui les porte ; son propre
//! poids l'emporte toujours — il dit autre chose.

use super::domaines::{decrire_les_domaines, DomaineDecrit};
use super::{texte_porte, Sorte};
use crate::geometry::Rect;
use crate::types::{Annotation, ArrowPredicate, Board, Domain, StickyOperator};

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
    /// Pour un post-it : l'opérateur qu'il est devenu (ET, OU, MAIS, PARCE QUE) — ce que l'humain
    /// voit à sa place, une pilule (OPERATEUR-2, fiche 98).
    pub operateur: Option<StickyOperator>,
    /// **Sa couleur à lui** — celle que l'humain lui a donnée par la barre de couleur, ou un agent
    /// (COULEUR-2, fiche 98) : le papier d'un post-it, la teinte d'un texte ou d'une membrane.
    pub couleur: Option<String>,
    /// Pour une photo : l'adresse d'où elle vient — la page qu'elle montre.
    pub adresse: Option<String>,
    /// Les domaines qu'il porte, leur nom et leur poids.
    pub domaines: Vec<DomaineDecrit>,
    /// **Ceux qu'il hérite de ses membranes** (DOM-6), et de laquelle — sans ceux qu'il porte.
    pub herites: Vec<Herite>,
}

/// **Un domaine hérité** : le domaine, et la membrane qui le donne.
#[derive(Debug, Clone, PartialEq)]
pub struct Herite {
    pub domaine: DomaineDecrit,
    pub de: String,
}

impl Decrit {
    /// **Son poids dans ce domaine**, et la membrane qui le donne s'il en hérite — ou rien s'il n'y
    /// appartient pas.
    pub fn poids_dans(&self, domaine: &str) -> Option<(f64, Option<&str>)> {
        if let Some(p) = self.domaines.iter().find(|p| p.id == domaine) {
            return Some((p.poids, None));
        }
        self.herites
            .iter()
            .find(|h| h.domaine.id == domaine)
            .map(|h| (h.domaine.poids, Some(h.de.as_str())))
    }
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
        operateur: None,
        couleur: None,
        adresse: i.source_url.clone(),
        domaines: decrire_les_domaines(catalogue, &i.domains),
        herites: Vec::new(),
    });
    let annotations = board
        .annotations
        .iter()
        .map(|a| decrire_une_annotation(a, catalogue));
    let mut decrits: Vec<Decrit> = photos.chain(annotations).collect();
    heriter(&mut decrits);
    decrits
}

/// **DOM-6** : chaque nœud reçoit les domaines de ses membranes, de la plus proche à la plus
/// lointaine, sans ceux qu'il porte ni ceux qu'une plus proche a déjà donnés.
fn heriter(decrits: &mut [Decrit]) {
    let rang: std::collections::HashMap<String, usize> = decrits
        .iter()
        .enumerate()
        .map(|(i, d)| (d.id.clone(), i))
        .collect();
    for i in 0..decrits.len() {
        let mut herites: Vec<Herite> = Vec::new();
        let mut vues: Vec<String> = Vec::new();
        let mut membrane = decrits[i].membrane.clone();
        // Une membrane ne se contient jamais elle-même (MEMB-1) ; si elle le faisait, on s'arrête.
        while let Some(m) = membrane.filter(|m| !vues.contains(m)) {
            let Some(&j) = rang.get(&m) else {
                break;
            };
            for p in &decrits[j].domaines {
                let deja = decrits[i].domaines.iter().any(|q| q.id == p.id)
                    || herites.iter().any(|h| h.domaine.id == p.id);
                if !deja {
                    herites.push(Herite {
                        domaine: p.clone(),
                        de: m.clone(),
                    });
                }
            }
            membrane = decrits[j].membrane.clone();
            vues.push(m);
        }
        decrits[i].herites = herites;
    }
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
    let operateur = match a {
        Annotation::Sticky { operator, .. } => *operator,
        _ => None,
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
        operateur,
        couleur: a.couleur_propre().map(str::to_string),
        adresse: None,
        domaines: decrire_les_domaines(catalogue, a.domains()),
        herites: Vec::new(),
    }
}
