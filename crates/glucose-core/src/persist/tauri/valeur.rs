//! La valeur neutre qu'un document de Glucose Tauri devient une fois lu, quel que soit son
//! format.
//!
//! Deux lecteurs la produisent — le binaire Automerge et le JSON d'avant — et un seul
//! traducteur la consomme ([`super::projet`]). C'est ce qui permet de vérifier notre lecteur
//! Automerge contre la bibliothèque de référence : les deux doivent rendre la **même**
//! valeur, arbre pour arbre.

use std::collections::BTreeMap;

/// Une valeur du modèle de données de Glucose Tauri : celui de JSON, plus les octets.
#[derive(Debug, Clone, PartialEq)]
pub enum Valeur {
    Nul,
    Booleen(bool),
    /// Un entier. Automerge distingue entiers signés, non signés, compteurs et horodatages ;
    /// JavaScript n'en fait qu'un nombre, et c'est ainsi que Tauri les écrivait.
    Entier(i64),
    Flottant(f64),
    Texte(String),
    Octets(Vec<u8>),
    Liste(Vec<Valeur>),
    Carte(BTreeMap<String, Valeur>),
}

impl Valeur {
    /// Le champ `cle` d'une carte, ou rien.
    pub fn champ(&self, cle: &str) -> Option<&Valeur> {
        match self {
            Valeur::Carte(m) => m.get(cle).filter(|v| !matches!(v, Valeur::Nul)),
            _ => None,
        }
    }

    pub fn texte(&self, cle: &str) -> Option<&str> {
        match self.champ(cle)? {
            Valeur::Texte(s) => Some(s),
            _ => None,
        }
    }

    /// Un nombre, qu'il ait été écrit comme entier ou comme flottant : JavaScript n'a qu'un
    /// type de nombre, et `14` et `14.0` sont pour lui la même valeur.
    pub fn nombre(&self, cle: &str) -> Option<f64> {
        match self.champ(cle)? {
            Valeur::Entier(n) => Some(*n as f64),
            Valeur::Flottant(x) if x.is_finite() => Some(*x),
            _ => None,
        }
    }

    pub fn entier(&self, cle: &str) -> Option<i64> {
        match self.champ(cle)? {
            Valeur::Entier(n) => Some(*n),
            // Un horodatage écrit par `Date.now()` est entier, mais un nombre JavaScript peut
            // revenir en flottant d'un aller-retour : on ne garde que sa partie entière exacte.
            Valeur::Flottant(x) if x.is_finite() && x.fract() == 0.0 => Some(*x as i64),
            _ => None,
        }
    }

    pub fn booleen(&self, cle: &str) -> Option<bool> {
        match self.champ(cle)? {
            Valeur::Booleen(b) => Some(*b),
            _ => None,
        }
    }

    /// Les éléments de la liste `cle` — vide si le champ manque ou n'est pas une liste.
    pub fn liste(&self, cle: &str) -> &[Valeur] {
        match self.champ(cle) {
            Some(Valeur::Liste(l)) => l,
            _ => &[],
        }
    }

    pub fn est_carte(&self) -> bool {
        matches!(self, Valeur::Carte(_))
    }
}

/// **Détruire un arbre ne descend pas la pile.**
///
/// La destruction que Rust écrit seul est récursive : un document forgé de cent mille listes
/// imbriquées se lit sans récursion ([`super::json`]), puis ferait déborder la pile au moment
/// d'être libéré. Les enfants partent donc dans une liste de travail, et chaque nœud meurt
/// vide.
impl Drop for Valeur {
    fn drop(&mut self) {
        let mut pile = Vec::new();
        enfants(self, &mut pile);
        while let Some(mut v) = pile.pop() {
            enfants(&mut v, &mut pile);
        }
    }
}

/// Retire les enfants d'un nœud et les range dans `pile`.
fn enfants(v: &mut Valeur, pile: &mut Vec<Valeur>) {
    match v {
        Valeur::Liste(l) => pile.append(l),
        Valeur::Carte(m) => pile.extend(std::mem::take(m).into_values()),
        _ => {}
    }
}
