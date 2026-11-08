//! **Le recensement des domaines** (fiche 88) : ce qu'un domaine traverse, d'un fichier à l'autre.
//!
//! La [vision](../../../docs/VISION.md) dit : *« le domaine, lui, va traverser tout ça »*, les blocs,
//! les membranes et les fichiers. Il n'y a pour autant **aucun catalogue central** : chaque fichier
//! garde les domaines qu'il porte, et les emporte quand on le déplace ou qu'on l'envoie. Un catalogue
//! à part serait une seconde vérité : perdu, il viderait les fichiers de leur sens ; envoyé sans lui,
//! un fichier arriverait muet.
//!
//! Le recensement est donc une **vue déduite** : les fichiers qu'on a lus, mis côte à côte. Il ne
//! s'écrit nulle part, et se refait entièrement depuis les fichiers. Ce module est pur : lire le
//! disque revient à l'application (`glucose-desktop/src/recensement.rs`).
//!
//! # RECENSE-1 — une lignée, un domaine ; un nom, une question
//!
//! Deux fichiers qui portent la même **lignée** (DOM-5) portent le même domaine, et la définition la
//! plus récente l'emporte : renommer dans l'un se lit dans l'autre. Deux lignées sous **le même nom**
//! sont deux créations séparées : rien ne les réunit en silence, et [`homonymes`] les rend pour que
//! Glucose demande à l'humain ou à l'IA (sa règle du 7/11/2026).

use crate::store::agents::{decrire_le_tableau, Decrit};
use crate::store::{cle_de_domaine, lignee_de};
use crate::types::{Domain, Project};
use std::collections::BTreeMap;

/// **Un nœud qui porte au moins un domaine**, et le tableau où il vit.
#[derive(Debug, Clone, PartialEq)]
pub struct NoeudRecense {
    pub tableau: String,
    pub decrit: Decrit,
}

/// **Ce qu'un fichier dit de ses domaines** : son catalogue, et ses nœuds qui en portent.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentRecense {
    pub chemin: String,
    pub catalogue: Vec<Domain>,
    pub noeuds: Vec<NoeudRecense>,
}

/// **Lit ce qu'un document dit de ses domaines.** Un nœud sans domaine, ni porté ni hérité de ses
/// membranes (DOM-6), n'y entre pas : le recensement ne pèse que ce qui sert.
pub fn recenser(chemin: &str, projet: &Project) -> DocumentRecense {
    let noeuds = projet
        .boards
        .iter()
        .flat_map(|tableau| {
            decrire_le_tableau(tableau, &projet.domains)
                .into_iter()
                .filter(|d| !d.domaines.is_empty() || !d.herites.is_empty())
                .map(|decrit| NoeudRecense {
                    tableau: tableau.id.clone(),
                    decrit,
                })
        })
        .collect();
    DocumentRecense {
        chemin: chemin.to_string(),
        catalogue: projet.domains.clone(),
        noeuds,
    }
}

/// **Un domaine dans un fichier** : combien de nœuds l'y portent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DansUnFichier {
    pub chemin: String,
    pub noeuds: usize,
}

/// **Un domaine, tel qu'on le connaît tous fichiers confondus.**
#[derive(Debug, Clone, PartialEq)]
pub struct DomaineConnu {
    pub lignee: String,
    /// La définition la plus récente.
    pub definition: Domain,
    /// Les fichiers qui le portent, triés par chemin.
    pub fichiers: Vec<DansUnFichier>,
}

impl DomaineConnu {
    /// Combien de nœuds le portent, tous fichiers confondus.
    pub fn noeuds(&self) -> usize {
        self.fichiers.iter().map(|f| f.noeuds).sum()
    }
}

/// **Un nœud trouvé par son domaine**, dans le fichier où il vit.
#[derive(Debug, Clone, PartialEq)]
pub struct NoeudTrouve<'a> {
    pub chemin: &'a str,
    pub tableau: &'a str,
    pub decrit: &'a Decrit,
    pub poids: f64,
    /// La membrane qui le lui donne, s'il en hérite (DOM-6).
    pub herite_de: Option<&'a str>,
}

/// **Les fichiers lus jusqu'ici**, par chemin.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Recensement {
    documents: BTreeMap<String, DocumentRecense>,
}

impl Recensement {
    /// Prend ce qu'un fichier dit ; ce qu'il disait avant est oublié.
    pub fn poser(&mut self, document: DocumentRecense) {
        self.documents.insert(document.chemin.clone(), document);
    }

    /// Oublie un fichier : il n'est plus là.
    pub fn oublier(&mut self, chemin: &str) {
        self.documents.remove(chemin);
    }

    /// Les documents à regarder : `courant` passe devant ce qu'on avait lu du même fichier, parce
    /// qu'il est vivant.
    fn documents<'a>(
        &'a self,
        courant: Option<&'a DocumentRecense>,
    ) -> impl Iterator<Item = &'a DocumentRecense> {
        let garde = move |d: &&DocumentRecense| courant.is_none_or(|c| c.chemin != d.chemin);
        self.documents.values().filter(garde).chain(courant)
    }

    /// **Tous les domaines connus**, par ordre de nom. `courant` est le document ouvert, tel qu'il est
    /// à cet instant.
    pub fn vue(&self, courant: Option<&DocumentRecense>) -> Vec<DomaineConnu> {
        let mut connus: BTreeMap<String, DomaineConnu> = BTreeMap::new();
        for document in self.documents(courant) {
            for d in &document.catalogue {
                let lignee = lignee_de(d);
                let noeuds = document
                    .noeuds
                    .iter()
                    .filter(|n| n.decrit.domaines.iter().any(|p| p.id == d.id))
                    .count();
                let fichier = DansUnFichier {
                    chemin: document.chemin.clone(),
                    noeuds,
                };
                connus
                    .entry(lignee.clone())
                    .and_modify(|c| {
                        if d.defini_a > c.definition.defini_a {
                            c.definition = d.clone();
                        }
                        c.fichiers.push(fichier.clone());
                    })
                    .or_insert_with(|| DomaineConnu {
                        lignee,
                        definition: d.clone(),
                        fichiers: vec![fichier.clone()],
                    });
            }
        }
        let mut vue: Vec<DomaineConnu> = connus.into_values().collect();
        for c in &mut vue {
            c.fichiers.sort_by(|a, b| a.chemin.cmp(&b.chemin));
        }
        vue.sort_by_cached_key(|c| (cle_de_domaine(&c.definition.name), c.lignee.clone()));
        vue
    }

    /// **Chaque nœud qui porte cette lignée**, dans tous les fichiers, du plus lourd au plus léger ;
    /// à poids égal, dans l'ordre des fichiers, pour qu'une même question rende la même réponse.
    /// `herites` : avec ceux qui l'héritent de leurs membranes (DOM-6) — ce qu'un agent lit ; sans,
    /// les seuls endroits où il est posé — où l'humain vole (« où »).
    pub fn noeuds_du_domaine<'a>(
        &'a self,
        courant: Option<&'a DocumentRecense>,
        lignee: &str,
        herites: bool,
    ) -> Vec<NoeudTrouve<'a>> {
        let mut trouves: Vec<NoeudTrouve<'a>> = Vec::new();
        for document in self.documents(courant) {
            let ici = document.catalogue.iter().find(|d| lignee_de(d) == lignee);
            let Some(ici) = ici else {
                continue;
            };
            for n in &document.noeuds {
                match n.decrit.poids_dans(&ici.id) {
                    Some((poids, herite_de)) if herites || herite_de.is_none() => {
                        trouves.push(NoeudTrouve {
                            chemin: &document.chemin,
                            tableau: &n.tableau,
                            decrit: &n.decrit,
                            poids,
                            herite_de,
                        });
                    }
                    _ => {}
                }
            }
        }
        trouves.sort_by(|a, b| {
            b.poids
                .total_cmp(&a.poids)
                .then_with(|| a.chemin.cmp(b.chemin))
        });
        trouves
    }
}

/// **Les domaines connus sous ce nom** (RECENSE-1). Un seul : c'est lui. Plusieurs : deux créations
/// séparées, et il faut demander. Aucun : le nom est libre.
pub fn homonymes<'v>(vue: &'v [DomaineConnu], nom: &str) -> Vec<&'v DomaineConnu> {
    let cle = cle_de_domaine(nom);
    if cle.is_empty() {
        return Vec::new();
    }
    vue.iter()
        .filter(|c| cle_de_domaine(&c.definition.name) == cle)
        .collect()
}

#[cfg(test)]
mod tests;
