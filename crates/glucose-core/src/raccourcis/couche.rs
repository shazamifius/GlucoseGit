//! **Une couche de raccourcis, et la pile qui les mélange.**
//!
//! # Le format d'une couche
//!
//! Un texte, une liaison par ligne — l'accord, puis les commandes qu'il désigne, la première qui
//! s'applique l'emportant :
//!
//! ```text
//! # Un commentaire
//! Ctrl+Z        annuler
//! 1             fleche.relation.1 | signet.aller.1
//! Ctrl+S        -
//! ```
//!
//! `-` retire l'accord des couches du dessous. Une ligne fausse ne fait pas tomber la couche :
//! elle est dite, avec son numéro, et la couche garde tout le reste.

use super::Accord;

/// **Une couche** : un nom, et pour chaque accord qu'elle connaît, la liste des commandes qu'il
/// désigne — vide quand elle le retire. **Dans l'ordre du fichier** : le premier accord écrit
/// pour une commande est celui que le menu dit à côté d'elle — l'auteur du preset en décide.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Couche {
    pub nom: String,
    liaisons: Vec<(Accord, Vec<String>)>,
}

/// Une ligne qu'une couche n'a pas pu lire, et pourquoi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErreurDeLigne {
    pub ligne: usize,
    pub raison: String,
}

impl Couche {
    /// Une couche vide.
    pub fn nommee(nom: &str) -> Self {
        Self {
            nom: nom.to_string(),
            liaisons: Vec::new(),
        }
    }

    /// **Lit une couche** : elle-même, et les lignes qu'elle n'a pas pu lire.
    pub fn lire(nom: &str, texte: &str) -> (Self, Vec<ErreurDeLigne>) {
        let mut couche = Self::nommee(nom);
        let mut erreurs = Vec::new();
        for (i, ligne) in texte.lines().enumerate() {
            let ligne = ligne.split('#').next().unwrap_or("").trim();
            if ligne.is_empty() {
                continue;
            }
            match lire_une_liaison(ligne) {
                Ok((accord, _)) if couche.liaison(&accord).is_some() => {
                    erreurs.push(ErreurDeLigne {
                        ligne: i + 1,
                        raison: format!("{} est déjà lié plus haut", accord.ecrire()),
                    });
                }
                Ok(liaison) => couche.liaisons.push(liaison),
                Err(raison) => erreurs.push(ErreurDeLigne {
                    ligne: i + 1,
                    raison,
                }),
            }
        }
        (couche, erreurs)
    }

    /// **Lie cet accord** à ces commandes, ou le retire (`commandes` vide).
    pub fn lier(&mut self, accord: Accord, commandes: &[&str]) {
        let commandes: Vec<String> = commandes.iter().map(|c| c.to_string()).collect();
        match self.liaisons.iter_mut().find(|(a, _)| *a == accord) {
            Some((_, deja)) => *deja = commandes,
            None => self.liaisons.push((accord, commandes)),
        }
    }

    /// **La couche écrite**, que [`Self::lire`] relit à l'identique.
    pub fn ecrire(&self) -> String {
        let mut t = String::new();
        for (accord, commandes) in &self.liaisons {
            let cible = if commandes.is_empty() {
                "-".to_string()
            } else {
                commandes.join(" | ")
            };
            t.push_str(&format!("{:<14}{cible}\n", accord.ecrire()));
        }
        t
    }

    /// Ce que cette couche dit de cet accord : rien (`None`), une liste, ou un retrait (vide).
    pub fn liaison(&self, accord: &Accord) -> Option<&[String]> {
        self.liaisons
            .iter()
            .find(|(a, _)| a == accord)
            .map(|(_, c)| c.as_slice())
    }

    /// Chaque commande que cette couche nomme.
    pub fn commandes(&self) -> impl Iterator<Item = &str> {
        self.liaisons
            .iter()
            .flat_map(|(_, c)| c)
            .map(String::as_str)
    }

    fn accords(&self) -> impl Iterator<Item = &Accord> {
        self.liaisons.iter().map(|(a, _)| a)
    }
}

fn lire_une_liaison(ligne: &str) -> Result<(Accord, Vec<String>), String> {
    let (accord, cible) = ligne
        .split_once(char::is_whitespace)
        .ok_or_else(|| "un accord sans commande".to_string())?;
    let accord = Accord::lire(accord)?;
    let cible = cible.trim();
    if cible == "-" {
        return Ok((accord, Vec::new()));
    }
    let commandes: Vec<String> = cible.split('|').map(|c| c.trim().to_string()).collect();
    if commandes
        .iter()
        .any(|c| c.is_empty() || c.contains(char::is_whitespace))
    {
        return Err(format!("« {cible} » : des commandes séparées par |"));
    }
    Ok((accord, commandes))
}

/// **La pile des couches** — la première en dessous, la dernière au-dessus : un preset, les
/// raccourcis qu'on lui ajoute, ceux de l'utilisateur.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Clavier {
    couches: Vec<Couche>,
}

impl Clavier {
    pub fn de(couches: Vec<Couche>) -> Self {
        Self { couches }
    }

    pub fn couches(&self) -> &[Couche] {
        &self.couches
    }

    /// **Ce que cet accord désigne** : la liste de la plus haute couche qui le connaît — vide
    /// si elle le retire, ou si aucune ne le connaît.
    pub fn commandes(&self, accord: &Accord) -> &[String] {
        self.couches
            .iter()
            .rev()
            .find_map(|c| c.liaison(accord))
            .unwrap_or(&[])
    }

    /// **Les accords qui mènent à cette commande**, une fois les couches mélangées — ce que le
    /// menu et la palette disent à côté d'elle, le premier d'abord : ceux de la couche du dessus,
    /// puis dans l'ordre de chaque fichier. Un accord qu'une couche du dessus a lié à autre
    /// chose, ou retiré, n'y mène plus.
    pub fn accords_de(&self, commande: &str) -> Vec<Accord> {
        let mut accords: Vec<Accord> = Vec::new();
        for a in self.couches.iter().rev().flat_map(Couche::accords) {
            if !accords.contains(a) && self.commandes(a).iter().any(|c| c == commande) {
                accords.push(*a);
            }
        }
        accords
    }
}
