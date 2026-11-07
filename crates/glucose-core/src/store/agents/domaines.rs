//! **Les domaines, pour un agent** (fiche 88) : il les lit comme l'humain les voit, et les pose d'un
//! geste.
//!
//! La vision l'exige : *« un Glucose doit pouvoir être lu par un humain ET par une IA, de quasi la
//! même façon »*. Jusqu'ici, un agent ne savait ni qu'un domaine existait, ni le poser.
//!
//! # AGENT-5 — un agent dit s'il prend un domaine ou s'il en crée un
//!
//! DOM-4 (`store/domains/nom.rs`) interdit deux domaines du même nom, et sa règle exige de
//! **demander** au lieu de deviner. Si « poser python » créait le domaine quand il manque et le
//! prenait quand il existe, l'agent mélangerait le serpent et le langage sans jamais le savoir. Il
//! choisit donc : [`Voulu::Existant`] ou [`Voulu::Nouveau`], et un nouveau nom déjà pris est refusé
//! avec la question.
//!
//! # AGENT-6 — un poids nul retire, et ne fait jamais entrer un domaine
//!
//! Un nœud dont le poids tombe à zéro n'appartient plus au domaine. Le catalogue, lui, ne grandit
//! pas pour un retrait.

use super::{Refus, Store};
use crate::error::CoreError;
use crate::types::{Domain, DomainAssignment};

/// **Un domaine que porte un nœud**, tel qu'un agent le lit : son nom y est, pour qu'il n'ait pas à
/// relire le catalogue pour comprendre un nœud. L'identifiant ne sert qu'à le retrouver.
#[derive(Debug, Clone, PartialEq)]
pub struct DomaineDecrit {
    pub id: String,
    pub nom: String,
    pub poids: f64,
}

/// **Ce qu'un domaine neuf reçoit à sa naissance**, et que le noyau ne sait pas : le thème, qui donne
/// sa couleur selon son rang, et l'horloge. Son sigle, lui, vient de son nom (`sigle_de`) : un
/// domaine « Eau » qui se montrerait « SCI » tromperait l'œil.
#[derive(Debug, Clone, Copy)]
pub struct Naissance {
    pub couleur: fn(usize) -> String,
    pub instant: i64,
}

/// **Le sigle qu'un nom donne** : ses trois premières lettres ou chiffres, en capitales (« Eau » donne
/// « EAU », « Jeu vidéo » « JEU »). Un nom sans lettre garde ses premiers signes.
pub fn sigle_de(nom: &str) -> String {
    let lettres: String = nom
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(3)
        .collect();
    let pris = if lettres.is_empty() {
        nom.trim().chars().take(3).collect()
    } else {
        lettres
    };
    pris.to_uppercase()
}

/// **Le domaine qu'un agent veut poser** (AGENT-5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Voulu<'a> {
    /// Un domaine qui existe : dans ce document, ou dans un autre fichier (sa définition entre alors
    /// au catalogue de celui-ci).
    Existant(&'a Domain),
    /// Un domaine à inscrire, sous ce nom.
    Nouveau(&'a str),
}

/// Les domaines d'un nœud, dans l'ordre où il les porte (DOM-2).
pub(super) fn decrire_les_domaines(
    catalogue: &[Domain],
    portes: &[DomainAssignment],
) -> Vec<DomaineDecrit> {
    portes
        .iter()
        .filter_map(|a| {
            let d = catalogue.iter().find(|d| d.id == a.domain_id)?;
            Some(DomaineDecrit {
                id: d.id.clone(),
                nom: d.name.clone(),
                poids: a.weight,
            })
        })
        .collect()
}

/// Ce que le noyau refuse, dans les mots qu'un agent peut corriger.
fn refus_du_noyau(e: CoreError) -> Refus {
    match e {
        CoreError::BoardNotFound(t) => Refus::TableauInconnu(t),
        CoreError::NodeNotFound(n) => Refus::NoeudInconnu(n),
        autre => Refus::Domaine(autre.to_string()),
    }
}

impl Store {
    /// **Pose ce domaine sur ces nœuds** — ou le leur retire, si le poids est nul (AGENT-6). Rend son
    /// identifiant dans ce document (rien si un retrait n'y trouve rien), et s'il vient de naître.
    /// À appeler dans un [`Store::geste_d_agent`], qui rend le tout atomique.
    pub fn poser_un_domaine_pour_un_agent(
        &mut self,
        board_id: &str,
        ids: &[&str],
        (voulu, poids): (Voulu<'_>, f64),
        naissance: Naissance,
    ) -> Result<(Option<String>, bool), Refus> {
        if poids == 0.0 {
            let id = match voulu {
                Voulu::Existant(d) => self
                    .project
                    .domains
                    .iter()
                    .find(|ici| crate::store::lignee_de(ici) == crate::store::lignee_de(d))
                    .map(|ici| ici.id.clone()),
                Voulu::Nouveau(_) => None,
            };
            if let Some(id) = &id {
                self.retirer_un_domaine_pour_un_agent(board_id, ids, id)?;
            }
            return Ok((id, false));
        }
        let (id, ne) = match voulu {
            Voulu::Existant(d) => (self.adopter_un_domaine(d).map_err(refus_du_noyau)?, false),
            Voulu::Nouveau(nom) => {
                let allure = (
                    (naissance.couleur)(self.project.domains.len()),
                    sigle_de(nom),
                );
                let id = self
                    .creer_un_domaine(nom, allure, naissance.instant)
                    .map_err(refus_du_noyau)?;
                (id, true)
            }
        };
        for noeud in ids {
            self.try_assign_domain_to_node(board_id, noeud, &id, poids)
                .map_err(refus_du_noyau)?;
        }
        Ok((Some(id), ne))
    }

    /// Retire ce domaine des nœuds qui le portent : celui qui ne le porte pas n'a rien à perdre.
    fn retirer_un_domaine_pour_un_agent(
        &mut self,
        board_id: &str,
        ids: &[&str],
        id: &str,
    ) -> Result<(), Refus> {
        for noeud in ids {
            let porte = self
                .node_domains(board_id, noeud)
                .map_err(refus_du_noyau)?
                .iter()
                .any(|a| a.domain_id == id);
            if porte {
                self.try_unassign_domain_from_node(board_id, noeud, id)
                    .map_err(refus_du_noyau)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
