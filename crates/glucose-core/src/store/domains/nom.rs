//! **Le nom d'un domaine, et son identité d'un fichier à l'autre** (fiche 88).
//!
//! # DOM-4 — un nom, un domaine
//!
//! Sa règle du 7/11/2026 : on ne peut pas inscrire deux domaines du même nom. Il faut **demander** à
//! l'humain ou à l'IA de préciser le mot, ce qui donne « Python code » et « Python serpent ». Le
//! noyau le tient pour un document ([`Store::try_add_domain`], [`Store::try_update_domain`]) ;
//! l'application le tient pour tout ce qu'elle connaît (`recensement`). Ni la casse ni les blancs en
//! trop ne font un autre nom. Les accents, si : « pêche » et « péché » ne sont pas le même sujet.
//!
//! **Une limite connue** : le noyau n'a aucune dépendance, donc aucune table Unicode. Un « é » écrit
//! en deux caractères (un `e` et son accent, ce que certains systèmes collent) reste différent d'un
//! « é » écrit en un seul.
//!
//! # DOM-5 — l'identité naît une fois, et le nom peut changer
//!
//! Un domaine a pour identifiant son premier nom et l'instant de sa naissance : `eau@1762500000000`.
//! Le nom se change ensuite sans toucher à l'identifiant, ni aux nœuds qui le portent, ni aux autres
//! fichiers : ceux-ci apprennent le nouveau nom à leur ouverture, par la date la plus récente.
//!
//! Deux créations séparées du même nom ont donc **deux identités** : quand leurs fichiers se
//! rencontrent, rien ne les réunit en silence. Glucose demande (`recensement::homonymes`).
//!
//! # Les domaines d'avant
//!
//! Ils s'appellent `domain-3`, et deux documents peuvent avoir chacun le leur. Leur identifiant reste
//! tel quel : l'histoire d'un fichier se rejoue strictement, et le réécrire hors du journal
//! arrêterait ce rejeu, donc perdrait du travail. Leur identité se lit dans leur nom et leur date.

use super::each_assignment_list;
use crate::error::{CoreError, CoreResult};
use crate::store::journal::{Edit, Slot};
use crate::store::Store;
use crate::types::Domain;

/// **La forme d'un nom qui sert à le reconnaître** : sans casse, et sans blanc en trop.
pub fn cle_de_domaine(nom: &str) -> String {
    nom_propre(nom).to_lowercase()
}

/// Le nom tel qu'on l'écrit, sans ses blancs en trop.
fn nom_propre(nom: &str) -> String {
    nom.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Un identifiant du générateur d'avant (`domain-3`), qui ne veut rien dire hors de son document.
pub(super) fn ne_d_avant(id: &str) -> bool {
    id.strip_prefix("domain-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// **Ce qui fait qu'un domaine est le même d'un fichier à l'autre** (DOM-5).
pub fn lignee_de(d: &Domain) -> String {
    if ne_d_avant(&d.id) {
        format!("{}@{}", cle_de_domaine(&d.name), d.defini_a)
    } else {
        d.id.clone()
    }
}

impl Store {
    /// **Ce nom est-il libre dans ce document ?** `sauf` est le domaine qu'on renomme : changer sa
    /// casse ne le fait pas entrer en conflit avec lui-même.
    pub(super) fn nom_libre(&self, nom: &str, sauf: Option<&str>) -> CoreResult<()> {
        let cle = cle_de_domaine(nom);
        if cle.is_empty() {
            return Err(CoreError::NomDeDomaineVide);
        }
        let pris = self
            .project
            .domains
            .iter()
            .find(|d| Some(d.id.as_str()) != sauf && cle_de_domaine(&d.name) == cle);
        match pris {
            Some(d) => Err(CoreError::NomDeDomainePris(d.name.clone())),
            None => Ok(()),
        }
    }

    /// **Le domaine qu'on désigne** : par son identifiant, sinon par son nom. Un nom vide ne
    /// désigne rien, même un domaine sans nom venu d'un ancien document.
    pub fn trouver_un_domaine(&self, designe: &str) -> Option<&Domain> {
        let domaines = &self.project.domains;
        if let Some(d) = domaines.iter().find(|d| d.id == designe) {
            return Some(d);
        }
        let cle = cle_de_domaine(designe);
        if cle.is_empty() {
            return None;
        }
        domaines.iter().find(|d| cle_de_domaine(&d.name) == cle)
    }

    /// **Combien de nœuds porte chaque domaine**, dans l'ordre du catalogue.
    ///
    /// O(nœuds) : pour un agent qui lit, jamais dans une image (loi L2).
    pub fn porteurs_des_domaines(&self) -> Vec<usize> {
        let domaines = &self.project.domains;
        let mut comptes = vec![0; domaines.len()];
        for liste in each_assignment_list(&self.project) {
            for a in liste {
                if let Some(rang) = domaines.iter().position(|d| d.id == a.domain_id) {
                    comptes[rang] += 1;
                }
            }
        }
        comptes
    }

    /// **Inscrit un domaine neuf** : son identité naît ici (DOM-5). Un nom pris est refusé (DOM-4).
    /// `allure` est sa couleur et son sigle, que le thème choisit ; l'instant vient de l'appelant.
    ///
    /// Deux naissances du même nom dans la même milliseconde (un nom d'attente, « Domaine 1 », repris
    /// aussitôt) auraient la même identité : la seconde naît à la milliseconde suivante.
    pub fn creer_un_domaine(
        &mut self,
        nom: &str,
        (color, icon): (String, String),
        instant: i64,
    ) -> CoreResult<String> {
        let name = nom_propre(nom);
        let cle = cle_de_domaine(&name);
        let pris = |id: &str| self.project.domains.iter().any(|d| d.id == id);
        let instant = (instant..)
            .find(|i| !pris(&format!("{cle}@{i}")))
            .unwrap_or(instant);
        let id = format!("{cle}@{instant}");
        self.try_add_domain(Domain {
            id: id.clone(),
            name,
            color,
            icon,
            defini_a: instant,
        })?;
        Ok(id)
    }

    /// **Le domaine d'un autre fichier, ici aussi** : sa définition entre au catalogue de ce
    /// document, pour que ce fichier reste entier quand il part ailleurs. Rend son identifiant dans
    /// ce document ; s'il y est déjà, rien ne change. Un autre sujet de même nom est refusé (DOM-4).
    pub fn adopter_un_domaine(&mut self, definition: &Domain) -> CoreResult<String> {
        let lignee = lignee_de(definition);
        if let Some(d) = self.project.domains.iter().find(|d| lignee_de(d) == lignee) {
            return Ok(d.id.clone());
        }
        self.try_add_domain(Domain {
            id: lignee.clone(),
            ..definition.clone()
        })?;
        Ok(lignee)
    }

    /// **« C'est le même sujet »** : ce domaine prend l'identité et la définition d'un homonyme venu
    /// d'un autre fichier, et tout ce qui le porte suit, en un seul geste. Rend le nombre de nœuds
    /// qui ont suivi. Défaire rend tout ; les fichiers fermés ne sont pas touchés.
    pub fn reunir_un_domaine(&mut self, id: &str, definition: &Domain) -> CoreResult<usize> {
        let Some(rang) = self.project.domains.iter().position(|d| d.id == id) else {
            return Err(CoreError::DomainNotFound(id.to_string()));
        };
        self.nom_libre(&definition.name, Some(id))?;
        let lignee = lignee_de(definition);
        if lignee != id && self.project.domains.iter().any(|d| d.id == lignee) {
            return Err(CoreError::DuplicateDomainId(lignee));
        }
        let avant = self.project.domains[rang].clone();
        let apres = Domain {
            id: lignee.clone(),
            ..definition.clone()
        };
        self.project.domains[rang] = apres.clone();
        let mut edits = vec![Edit::Domain {
            slot: Slot::changed(rang, avant, apres),
        }];
        let (portes, suivis) = self.reecrire_les_porteurs(id, |liste| {
            for a in liste.iter_mut().filter(|a| a.domain_id == id) {
                a.domain_id = lignee.clone();
            }
        });
        edits.extend(portes);
        self.record_as_one_gesture(edits);
        Ok(suivis)
    }
}

#[cfg(test)]
mod tests;
