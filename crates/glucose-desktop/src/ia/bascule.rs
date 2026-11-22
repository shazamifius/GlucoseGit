//! **La bascule** (fiche 129), sa parole du 21/11 : *« si ça atteint des limites, prévoir des
//! systèmes de switch de modèle avec un avertissement à l'utilisateur que ça vient de switcher, et
//! dès qu'un autre modèle plus évolué est de retour, ça rebasculera automatiquement »*. Pure : les
//! heures lui sont données.
//!
//! # BASCULE-1 — dormir le temps que le fournisseur dit
//!
//! Chaque requête part au **premier modèle éveillé** de la liste, du plus capable au moins. Un
//! refus l'écarte, et **aucune durée n'est choisie ici** :
//!
//! * **429 avec un délai** (`RetryInfo`) : il dort ce délai. Un quota du jour épuisé dort jusqu'à
//!   minuit, heure du Pacifique ; un modèle sans quota gratuit (« limit: 0 », les Pro depuis 2026)
//!   aussi — et se réessaie le lendemain, au cas où le compte aurait changé ; un quota par minute,
//!   quelques secondes ;
//! * **503, un 429 sans délai, un 400 ou un 500** : il cède **cette question**, et revient en tête
//!   à la suivante. Un 400 ne le fait pas sortir : une requête mal formée de notre part viderait
//!   la liste ;
//! * **404** (« plus disponible pour les nouveaux comptes ») : il sort ;
//! * **401, 403** : la clé est refusée, aucun autre modèle n'y changera rien — on s'arrête.
//!
//! L'heure est celle du mur, en millisecondes depuis 1970, pas celle du processus : un quota du
//! jour repart à minuit même si la machine a dormi entre-temps — ce qu'une horloge monotone ne
//! compte pas sous Linux (VEILLE-1, fiche 126).
//!
//! # BASCULE-2 — « dès qu'un plus évolué est de retour »
//!
//! Rien ne le guette : chaque requête repart du premier éveillé, et un sommeil fini l'y remet. Le
//! changement se **dit** quand celui qui répond n'est plus celui d'avant : plus bas dans la liste,
//! c'est un relais, et l'on dit pourquoi le premier s'est tu ; plus haut, c'est un retour.

use super::gemini::Refus;
use super::modeles::Modele;

/// Ce qu'on sait d'un modèle.
#[derive(Debug, Clone, PartialEq)]
enum Etat {
    /// Il répond, ou l'on ne sait pas encore.
    Eveille,
    /// Il dort jusqu'à cette heure : le fournisseur l'a dit.
    Dort { jusqu_a: u64, pourquoi: String },
    /// Il a cédé cette question ; il revient en tête à la suivante.
    Cede { pourquoi: String },
    /// Il n'existe plus pour cette clé.
    Sorti { pourquoi: String },
}

/// Ce que le fournisseur a répondu à un modèle.
#[derive(Debug, Clone, PartialEq)]
pub enum Issue {
    Repondu,
    Refuse(Refus),
}

/// La liste, et ce qu'on sait de chacun.
#[derive(Debug, Clone, PartialEq)]
pub struct Bascule {
    modeles: Vec<Modele>,
    etats: Vec<Etat>,
    /// Le dernier qui a répondu : un autre qui répond se dit.
    dernier: Option<usize>,
}

impl Bascule {
    pub fn nouvelle(modeles: Vec<Modele>) -> Self {
        let etats = vec![Etat::Eveille; modeles.len()];
        Self {
            modeles,
            etats,
            dernier: None,
        }
    }

    pub fn modeles(&self) -> &[Modele] {
        &self.modeles
    }

    pub fn modele(&self, i: usize) -> &Modele {
        &self.modeles[i]
    }

    /// **Une question neuve** : ceux qui ont cédé la précédente reviennent.
    pub fn nouvelle_question(&mut self) {
        for etat in &mut self.etats {
            if matches!(etat, Etat::Cede { .. }) {
                *etat = Etat::Eveille;
            }
        }
    }

    /// **Le premier à qui demander, maintenant** : son rang dans la liste.
    pub fn premier(&self, maintenant: u64) -> Option<usize> {
        self.etats.iter().position(|e| match e {
            Etat::Eveille => true,
            Etat::Dort { jusqu_a, .. } => *jusqu_a <= maintenant,
            Etat::Cede { .. } | Etat::Sorti { .. } => false,
        })
    }

    /// **Ce qui est arrivé à `i`** : l'annonce à faire, s'il y en a une — ou l'arrêt, quand la
    /// clé elle-même est refusée.
    pub fn apprendre(
        &mut self,
        i: usize,
        issue: &Issue,
        maintenant: u64,
    ) -> Result<Option<String>, String> {
        let refus = match issue {
            Issue::Repondu => {
                self.etats[i] = Etat::Eveille;
                let avant = self.dernier.replace(i);
                return Ok(avant
                    .filter(|&d| d != i)
                    .map(|d| self.annonce(d, i, maintenant)));
            }
            Issue::Refuse(r) => r,
        };
        let pourquoi = refus.message.clone();
        self.etats[i] = match (refus.statut, refus.revenir_dans) {
            (401 | 403, _) => return Err(format!("Google refuse la clé : {pourquoi}")),
            (404, _) => Etat::Sorti {
                pourquoi: "Google ne l'ouvre plus à ce compte".into(),
            },
            (429, Some(d)) => Etat::Dort {
                jusqu_a: maintenant.saturating_add(d.as_millis().try_into().unwrap_or(u64::MAX)),
                pourquoi: "sa limite gratuite est atteinte".into(),
            },
            (429, None) => Etat::Cede {
                pourquoi: "sa limite gratuite est atteinte".into(),
            },
            (503, _) => Etat::Cede {
                pourquoi: "il est surchargé".into(),
            },
            _ => Etat::Cede { pourquoi },
        };
        Ok(None)
    }

    /// Ce qui se dit quand `i` répond à la place de `d`.
    fn annonce(&self, d: usize, i: usize, maintenant: u64) -> String {
        let (avant, apres) = (&self.modeles[d].nom, &self.modeles[i].nom);
        if i < d {
            return format!("{apres} est de retour : il reprend la main.");
        }
        let pourquoi = self.raison(d, maintenant);
        format!("{avant} ne répond plus ({pourquoi}) : {apres} prend le relais.")
    }

    /// **Pourquoi `i` se tait**, en mots — et quand il reviendra, s'il dort.
    pub fn raison(&self, i: usize, maintenant: u64) -> String {
        match &self.etats[i] {
            Etat::Dort { jusqu_a, pourquoi } => {
                format!("{pourquoi}, il revient {}", dans(*jusqu_a, maintenant))
            }
            Etat::Cede { pourquoi } | Etat::Sorti { pourquoi } => pourquoi.clone(),
            Etat::Eveille => "il répond".into(),
        }
    }

    /// **Personne ne répond** : pourquoi, et quand le premier reviendra.
    pub fn personne(&self, maintenant: u64) -> String {
        let reveil = self
            .etats
            .iter()
            .filter_map(|e| match e {
                Etat::Dort { jusqu_a, .. } => Some(*jusqu_a),
                _ => None,
            })
            .min();
        match reveil {
            Some(t) => format!(
                "Aucun modèle gratuit ne répond pour l'instant : le premier revient {}.",
                dans(t, maintenant)
            ),
            None if self.modeles.is_empty() => {
                "Cette clé n'ouvre aucun modèle de conversation.".into()
            }
            None => {
                "Aucun modèle ne répond pour l'instant : renvoie ta question dans un moment.".into()
            }
        }
    }
}

/// « dans 14 h 52 », « dans 3 min », « dans 40 s ».
pub fn dans(jusqu_a: u64, maintenant: u64) -> String {
    let s = jusqu_a.saturating_sub(maintenant).div_ceil(1000);
    match s {
        0..=59 => format!("dans {s} s"),
        60..=3599 => format!("dans {} min", s.div_ceil(60)),
        _ => format!("dans {} h {:02}", s / 3600, (s % 3600) / 60),
    }
}
