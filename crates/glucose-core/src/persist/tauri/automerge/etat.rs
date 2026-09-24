//! De l'ensemble des opérations à l'**état** du document.
//!
//! Deux règles suffisent, et ce sont celles d'Automerge :
//!
//! 1. **Une valeur est visible tant que rien ne l'a remplacée.** Une opération qui a un
//!    successeur — une écriture plus récente, une suppression — est masquée. Seul un
//!    incrément laisse visible le compteur qu'il augmente.
//! 2. **Entre plusieurs valeurs visibles, la plus grande horloge l'emporte** : le compteur de
//!    l'opération, puis les octets de son auteur (horloge de Lamport). C'est le cas de deux
//!    personnes qui ont écrit le même champ en même temps.
//!
//! L'ordre d'une liste ou d'un texte suit l'arbre RGA : chaque insertion désigne l'élément
//! après lequel elle se place, et les insertions au même endroit se rangent de la plus récente
//! à la plus ancienne. Un parcours en profondeur de cet arbre donne la séquence.

use super::colonnes::Primitive;
use super::operations::{Cle, Objet, Op, OpId};
use super::Auteurs;
use crate::persist::tauri::Valeur;
use std::collections::{BTreeMap, HashMap, HashSet};

const CREER_CARTE: u64 = 0;
const ECRIRE: u64 = 1;
const CREER_LISTE: u64 = 2;
const CREER_TEXTE: u64 = 4;
const INCREMENTER: u64 = 5;
/// Une table (API historique d'Automerge) se lit comme une carte.
const CREER_TABLE: u64 = 6;

struct Etat {
    ops: Vec<Op>,
    successeurs: Vec<Vec<OpId>>,
    index: HashMap<OpId, usize>,
    ordre_auteurs: Vec<u32>,
    par_objet: HashMap<Objet, Vec<usize>>,
}

/// Reconstruit la valeur racine.
pub fn reconstruire(document: Vec<Op>, deltas: Vec<Op>, auteurs: &Auteurs) -> Valeur {
    Etat::new(document, deltas, auteurs).racine()
}

/// Un objet en cours de construction : ce qu'il lui reste à lire, et ce qu'il a déjà.
struct Cadre {
    objet: Objet,
    /// La clé sous laquelle il se range dans sa carte parente — aucune dans une liste.
    cle: Option<String>,
    corps: Corps,
}

enum Corps {
    Carte {
        entrees: std::vec::IntoIter<(String, usize)>,
        sortie: BTreeMap<String, Valeur>,
    },
    Liste {
        elements: std::vec::IntoIter<usize>,
        sortie: Vec<Valeur>,
    },
}

impl Cadre {
    fn ranger(&mut self, cle: Option<String>, v: Valeur) {
        match &mut self.corps {
            Corps::Carte { sortie, .. } => {
                sortie.insert(cle.unwrap_or_default(), v);
            }
            Corps::Liste { sortie, .. } => sortie.push(v),
        }
    }

    /// Le prochain enfant à construire, et sa clé.
    fn suivant(&mut self) -> Option<(Option<String>, usize)> {
        match &mut self.corps {
            Corps::Carte { entrees, .. } => entrees.next().map(|(k, i)| (Some(k), i)),
            Corps::Liste { elements, .. } => elements.next().map(|i| (None, i)),
        }
    }

    fn valeur(self) -> Valeur {
        match self.corps {
            Corps::Carte { sortie, .. } => Valeur::Carte(sortie),
            Corps::Liste { sortie, .. } => Valeur::Liste(sortie),
        }
    }
}

impl Etat {
    fn new(document: Vec<Op>, deltas: Vec<Op>, auteurs: &Auteurs) -> Self {
        let mut ops = Vec::with_capacity(document.len() + deltas.len());
        let mut index = HashMap::new();
        let mut successeurs = Vec::new();
        // Les successeurs d'un document sont écrits ; ceux d'un delta se déduisent de ses
        // prédécesseurs. Une opération lue deux fois (un delta déjà fondu dans le document)
        // ne compte qu'une fois.
        let mut predecesseurs = Vec::new();
        for (op, delta) in document
            .into_iter()
            .map(|o| (o, false))
            .chain(deltas.into_iter().map(|o| (o, true)))
        {
            if index.contains_key(&op.id) {
                continue;
            }
            index.insert(op.id, ops.len());
            if delta {
                predecesseurs.push((op.id, op.liens.clone()));
                successeurs.push(Vec::new());
            } else {
                successeurs.push(op.liens.clone());
            }
            ops.push(op);
        }
        for (id, preds) in predecesseurs {
            for p in preds {
                if let Some(&i) = index.get(&p) {
                    if !successeurs[i].contains(&id) {
                        successeurs[i].push(id);
                    }
                }
            }
        }
        let mut par_objet: HashMap<Objet, Vec<usize>> = HashMap::new();
        for (i, op) in ops.iter().enumerate() {
            par_objet.entry(op.objet).or_default().push(i);
        }
        Self {
            ops,
            successeurs,
            index,
            ordre_auteurs: auteurs.ordre(),
            par_objet,
        }
    }

    /// L'horloge de Lamport d'une opération : son compteur, puis l'ordre de son auteur.
    fn horloge(&self, id: OpId) -> (u64, u32) {
        (
            id.compteur,
            self.ordre_auteurs
                .get(id.auteur as usize)
                .copied()
                .unwrap_or(0),
        )
    }

    fn porte_une_valeur(action: u64) -> bool {
        matches!(
            action,
            CREER_CARTE | ECRIRE | CREER_LISTE | CREER_TEXTE | CREER_TABLE
        )
    }

    /// Visible : porte une valeur, et rien d'autre qu'un incrément ne l'a remplacée. Un
    /// successeur introuvable est une suppression — le format « document » les omet.
    fn visible(&self, i: usize) -> bool {
        Self::porte_une_valeur(self.ops[i].action)
            && self.successeurs[i].iter().all(|s| {
                self.index
                    .get(s)
                    .is_some_and(|&j| self.ops[j].action == INCREMENTER)
            })
    }

    /// La meilleure des opérations visibles : celle dont l'horloge est la plus grande.
    fn gagnante(&self, candidates: impl Iterator<Item = usize>) -> Option<usize> {
        candidates
            .filter(|&i| self.visible(i))
            .max_by_key(|&i| self.horloge(self.ops[i].id))
    }

    /// Construit l'arbre entier **sans récursion** : chaque objet ouvert attend sur une pile
    /// explicite que ses enfants soient construits. Un document forgé de cent mille objets
    /// imbriqués ne fait donc déborder aucune pile d'appels — la destruction de la valeur ne
    /// le fait pas non plus ([`Valeur`] se détruit itérativement).
    fn racine(&self) -> Valeur {
        let mut pile = vec![self.cadre(Objet::Racine, CREER_CARTE, None)];
        // Les objets ouverts : un objet qui se contiendrait lui-même — un fichier forgé —
        // devient nul au premier retour au lieu de boucler.
        let mut ouverts: HashSet<Objet> = HashSet::from([Objet::Racine]);
        loop {
            let suivant = pile.last_mut().and_then(Cadre::suivant);
            match suivant {
                Some((cle, i)) => {
                    let op = &self.ops[i];
                    let objet = Objet::Cree(op.id);
                    let v = if op.action == ECRIRE {
                        self.scalaire(i)
                    } else if op.action == CREER_TEXTE {
                        Valeur::Texte(self.texte(objet))
                    } else if ouverts.insert(objet) {
                        pile.push(self.cadre(objet, op.action, cle));
                        continue;
                    } else {
                        Valeur::Nul
                    };
                    if let Some(haut) = pile.last_mut() {
                        haut.ranger(cle, v);
                    }
                }
                None => {
                    let Some(fini) = pile.pop() else {
                        return Valeur::Nul;
                    };
                    ouverts.remove(&fini.objet);
                    let cle = fini.cle.clone();
                    let v = fini.valeur();
                    match pile.last_mut() {
                        Some(parent) => parent.ranger(cle, v),
                        None => return v,
                    }
                }
            }
        }
    }

    /// Ouvre un objet : ses enfants à construire, dans l'ordre.
    fn cadre(&self, objet: Objet, creation: u64, cle: Option<String>) -> Cadre {
        let ops: &[usize] = self.par_objet.get(&objet).map_or(&[], Vec::as_slice);
        let corps = if creation == CREER_LISTE {
            Corps::Liste {
                elements: self.sequence(ops).into_iter(),
                sortie: Vec::new(),
            }
        } else {
            Corps::Carte {
                entrees: self.entrees(ops).into_iter(),
                sortie: BTreeMap::new(),
            }
        };
        Cadre { objet, cle, corps }
    }

    /// Pour chaque clé d'une carte, l'opération gagnante.
    fn entrees(&self, ops: &[usize]) -> Vec<(String, usize)> {
        let mut cles: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for &i in ops {
            if let Cle::Carte(k) = &self.ops[i].cle {
                cles.entry(k.as_str()).or_default().push(i);
            }
        }
        cles.into_iter()
            .filter_map(|(k, candidates)| {
                Some((k.to_string(), self.gagnante(candidates.into_iter())?))
            })
            .collect()
    }

    /// Les opérations qui portent la valeur de chaque élément visible, dans l'ordre de la
    /// séquence.
    fn sequence(&self, ops: &[usize]) -> Vec<usize> {
        let mut enfants: HashMap<&Cle, Vec<usize>> = HashMap::new();
        let mut ecrasements: HashMap<OpId, Vec<usize>> = HashMap::new();
        for &i in ops {
            let op = &self.ops[i];
            if op.insertion {
                enfants.entry(&op.cle).or_default().push(i);
            } else if let Cle::Element(e) = op.cle {
                ecrasements.entry(e).or_default().push(i);
            }
        }
        for freres in enfants.values_mut() {
            // Du plus ancien au plus récent : la pile dépile donc le plus récent d'abord.
            freres.sort_by_key(|&i| self.horloge(self.ops[i].id));
        }
        let mut pile: Vec<usize> = enfants.get(&Cle::Tete).cloned().unwrap_or_default();
        let mut vus = HashSet::new();
        let mut sortie = Vec::new();
        while let Some(e) = pile.pop() {
            if !vus.insert(e) {
                continue;
            }
            let id = self.ops[e].id;
            let candidates =
                std::iter::once(e).chain(ecrasements.get(&id).into_iter().flatten().copied());
            if let Some(g) = self.gagnante(candidates) {
                sortie.push(g);
            }
            if let Some(suivants) = enfants.get(&Cle::Element(id)) {
                pile.extend(suivants.iter().copied());
            }
        }
        sortie
    }

    fn texte(&self, objet: Objet) -> String {
        let ops: &[usize] = self.par_objet.get(&objet).map_or(&[], Vec::as_slice);
        let mut s = String::new();
        for i in self.sequence(ops) {
            if let Primitive::Texte(t) = &self.ops[i].valeur {
                s.push_str(t);
            }
        }
        s
    }

    /// La valeur d'une écriture simple.
    fn scalaire(&self, i: usize) -> Valeur {
        match &self.ops[i].valeur {
            Primitive::Nul | Primitive::Inconnu => Valeur::Nul,
            Primitive::Booleen(b) => Valeur::Booleen(*b),
            Primitive::Entier(n) => Valeur::Entier(*n),
            Primitive::Flottant(x) => Valeur::Flottant(*x),
            Primitive::Texte(t) => Valeur::Texte(t.clone()),
            Primitive::Octets(o) => Valeur::Octets(o.clone()),
            Primitive::Compteur(depart) => Valeur::Entier(depart + self.increments(i)),
        }
    }

    /// Ce que les incréments ont ajouté à un compteur.
    fn increments(&self, i: usize) -> i64 {
        self.successeurs[i]
            .iter()
            .filter_map(|s| self.index.get(s))
            .filter_map(|&j| match &self.ops[j].valeur {
                Primitive::Entier(n) | Primitive::Compteur(n) => Some(*n),
                _ => None,
            })
            .sum()
    }
}
