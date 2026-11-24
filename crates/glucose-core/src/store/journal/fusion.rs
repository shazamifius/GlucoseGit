//! **REVENIR-1 — revenir dans sa salle par une fusion à trois** (fiche 146 § 4).
//!
//! # Ce qu'elle résout
//!
//! Sa note du 24/11 : *« je perds la sync, j'ai bien le fichier mais seul sans mise à jour »*. Un
//! document partagé qui revient dans sa salle après un arrêt net ou un « Quitter » a pu changer **ici**
//! (son fichier) pendant que la salle changeait **là-bas** (ses appareils de loin). Adopter la salle
//! effacerait le travail d'ici ; imposer le fichier effacerait celui de loin. La salle garde tout depuis
//! sa base : on y rebâtit **l'ancêtre** (le document au dernier numéro que le fichier contient) et
//! **là-bas** (maintenant), et on compare les trois, **élément par élément, par le nom**.
//!
//! # La règle — celle de Git (`git merge-file`, diff3)
//!
//! | ici | là-bas | on garde | ce qui part dans la salle |
//! |---|---|---|---|
//! | comme l'ancêtre | quoi que ce soit | là-bas | rien |
//! | comme là-bas | — | là-bas | rien |
//! | changé (posé, changé, retiré) | comme l'ancêtre | ici | le changement d'ici |
//! | changé | changé autrement | **là-bas** | rien — **le conflit est rendu**, la version d'ici avec |
//!
//! **Un ancêtre trop ancien ne gêne pas** : un même changement des deux côtés se fond sans conflit (la
//! deuxième ligne). C'est ce qui permet de reprendre après un arrêt net, où l'on ne connaît du numéro
//! exact qu'une borne basse. **Rien ne se perd** : dans un conflit, la salle gagne (elle est ce que
//! voient déjà les autres), et la version d'ici est rendue à l'appelant, qui la garde à côté et le
//! dit. Un élément retiré d'un côté et changé de l'autre est un conflit, pas un retrait : un retrait
//! n'efface jamais le travail fait de l'autre côté.
//!
//! # Ce qui part
//!
//! Les changements d'ici deviennent des éditions ([`Slot`], [`Whole`]) à appliquer **dans l'ordre** à
//! la liste de là-bas : exactement ce qu'un geste ordinaire envoie, nommé par l'élément entier qu'il
//! porte (PARTAGE-1). Un élément posé ici prend place derrière son voisin d'ici le plus proche qui
//! existe là-bas — sinon en tête.

use super::partage::Nomme;
use super::{Slot, Whole};
use std::collections::HashMap;

/// **Un élément changé des deux côtés, et différemment** : la salle a gagné.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflit<T> {
    pub nom: String,
    /// La version d'ici, rendue pour être gardée à côté — `None` : retiré ici.
    pub ici: Option<T>,
    /// La version de la salle, gardée — `None` : retiré là-bas.
    pub la_bas: Option<T>,
}

/// **Une liste fusionnée.**
#[derive(Debug, Clone, PartialEq)]
pub struct FusionDeListe<T> {
    /// La liste de là-bas, plus ce qui n'a changé qu'ici.
    pub liste: Vec<T>,
    /// Ce qui n'a changé qu'ici, en éditions à appliquer dans l'ordre à la liste de là-bas.
    pub a_envoyer: Vec<Slot<T>>,
    pub conflits: Vec<Conflit<T>>,
}

/// **Une valeur entière fusionnée** (un nom de tableau, l'ordre des onglets, les zones).
#[derive(Debug, Clone, PartialEq)]
pub struct FusionDeValeur<T> {
    pub valeur: T,
    pub a_envoyer: Option<Whole<T>>,
    /// Changée des deux côtés : la version d'ici, rendue.
    pub conflit: Option<T>,
}

/// Ce que dit la règle pour un élément, de ses trois versions.
#[derive(Debug, PartialEq)]
enum Decision {
    PrendreLaBas,
    Envoyer,
    Conflit,
}

fn decider<T: PartialEq>(ancetre: Option<&T>, ici: Option<&T>, la_bas: Option<&T>) -> Decision {
    if ici == la_bas || ici == ancetre {
        Decision::PrendreLaBas
    } else if la_bas == ancetre {
        Decision::Envoyer
    } else {
        Decision::Conflit
    }
}

/// **Fusionne une valeur entière.**
pub fn fondre_la_valeur<T: Clone + PartialEq>(
    ancetre: &T,
    ici: &T,
    la_bas: &T,
) -> FusionDeValeur<T> {
    let mut fusion = FusionDeValeur {
        valeur: la_bas.clone(),
        a_envoyer: None,
        conflit: None,
    };
    match decider(Some(ancetre), Some(ici), Some(la_bas)) {
        Decision::PrendreLaBas => {}
        Decision::Envoyer => {
            fusion.valeur = ici.clone();
            fusion.a_envoyer = Some(Whole::new(la_bas.clone(), ici.clone()));
        }
        Decision::Conflit => fusion.conflit = Some(ici.clone()),
    }
    fusion
}

/// **Fusionne une liste d'éléments nommés**, élément par élément.
pub fn fondre_la_liste<T: Nomme + Clone + PartialEq>(
    ancetre: &[T],
    ici: &[T],
    la_bas: &[T],
) -> FusionDeListe<T> {
    let par_nom = |liste: &[T]| -> HashMap<String, usize> {
        let paires = liste.iter().enumerate();
        paires.map(|(i, e)| (e.nom().to_string(), i)).collect()
    };
    let (a, l, r) = (par_nom(ancetre), par_nom(ici), par_nom(la_bas));
    let mut fusion = FusionDeListe {
        liste: la_bas.to_vec(),
        a_envoyer: Vec::new(),
        conflits: Vec::new(),
    };
    // Les noms d'ici dans leur ordre, puis ceux que l'ancêtre avait et qu'ici a retirés. Ce qui n'est
    // que là-bas n'est pas à revoir : il est déjà dans la liste.
    let retires = ancetre.iter().filter(|e| !l.contains_key(e.nom()));
    let noms = ici.iter().chain(retires).map(|e| e.nom().to_string());
    let mut vus = std::collections::HashSet::new();
    for nom in noms.filter(|n| vus.insert(n.clone())).collect::<Vec<_>>() {
        let version = |index: &HashMap<String, usize>, liste: &'_ [T]| {
            index.get(&nom).map(|&i| liste[i].clone())
        };
        let (va, vl, vr) = (version(&a, ancetre), version(&l, ici), version(&r, la_bas));
        match decider(va.as_ref(), vl.as_ref(), vr.as_ref()) {
            Decision::PrendreLaBas => {}
            Decision::Conflit => fusion.conflits.push(Conflit {
                nom,
                ici: vl,
                la_bas: vr,
            }),
            Decision::Envoyer => {
                if let Some(slot) = fusion.appliquer(&nom, vl, vr, ici) {
                    fusion.a_envoyer.push(slot);
                }
            }
        }
    }
    fusion
}

impl<T: Nomme + Clone + PartialEq> FusionDeListe<T> {
    /// Applique à la liste le changement d'ici : l'édition qui le dit à la salle.
    fn appliquer(
        &mut self,
        nom: &str,
        ici: Option<T>,
        la_bas: Option<T>,
        liste_d_ici: &[T],
    ) -> Option<Slot<T>> {
        let rang = |liste: &[T], nom: &str| liste.iter().position(|e| e.nom() == nom);
        Some(match (ici, la_bas) {
            (Some(neuf), Some(ancien)) => {
                let index = rang(&self.liste, nom)?;
                self.liste[index] = neuf.clone();
                Slot::changed(index, ancien, neuf)
            }
            (Some(neuf), None) => {
                // Derrière son voisin d'ici le plus proche qui existe déjà dans la liste.
                let rang_ici = rang(liste_d_ici, nom)?;
                let voisin = liste_d_ici[..rang_ici]
                    .iter()
                    .rev()
                    .find_map(|e| rang(&self.liste, e.nom()));
                let index = voisin.map_or(0, |v| v + 1);
                self.liste.insert(index, neuf.clone());
                Slot::inserted(index, neuf)
            }
            (None, Some(ancien)) => {
                let index = rang(&self.liste, nom)?;
                self.liste.remove(index);
                Slot::removed(index, ancien)
            }
            (None, None) => return None,
        })
    }
}

#[cfg(test)]
#[path = "fusion_tests.rs"]
mod tests;
