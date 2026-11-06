//! **Ce qui est choisi** (CHOIX-1, fiche 86) : l'ordre des choix, et une réponse immédiate à
//! « celui-ci en est-il ? ».
//!
//! La sélection était une liste, et chaque nœud visible la parcourait à chaque image : tout
//! choisir sur 4 000 textes coûtait 123 ms par image, et la barre d'action comparait chaque image
//! du tableau à chaque image choisie — soixante-quatre millions de fois à huit mille. Une liste se
//! lit très bien ; c'est la question qui demandait une autre forme.
//!
//! La lecture reste celle d'une liste (`Deref` vers une tranche), parce que l'ordre compte : le
//! dernier choisi, le premier d'un alignement. [`Choix::contains`] passe devant celui de la
//! tranche. Tout ce qui écrit passe par ici : l'ordre et l'ensemble ne peuvent pas diverger.

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;

/// Des identifiants choisis, dans l'ordre où ils l'ont été.
#[derive(Debug, Clone, Default)]
pub struct Choix {
    ordre: Vec<String>,
    ensemble: HashSet<String>,
}

impl Choix {
    /// **Celui-ci en est-il ?** Immédiat, quelle que soit la taille de la sélection.
    pub fn contains<Q>(&self, id: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.ensemble.contains(id)
    }

    /// Ajoute celui-ci à la fin, s'il n'y est pas déjà : un doublon ne veut rien dire.
    pub fn push(&mut self, id: String) {
        if self.ensemble.insert(id.clone()) {
            self.ordre.push(id);
        }
    }

    pub fn clear(&mut self) {
        self.ordre.clear();
        self.ensemble.clear();
    }

    /// Ne garde que ceux que `garder` accepte, dans leur ordre.
    pub fn retain(&mut self, mut garder: impl FnMut(&String) -> bool) {
        let ensemble = &mut self.ensemble;
        self.ordre.retain(|id| {
            let garde = garder(id);
            if !garde {
                ensemble.remove(id);
            }
            garde
        });
    }

    /// La liste, pour qui la veut à lui.
    pub fn to_vec(&self) -> Vec<String> {
        self.ordre.clone()
    }
}

impl Deref for Choix {
    type Target = [String];
    fn deref(&self) -> &[String] {
        &self.ordre
    }
}

impl FromIterator<String> for Choix {
    fn from_iter<I: IntoIterator<Item = String>>(ids: I) -> Self {
        let mut choix = Self::default();
        ids.into_iter().for_each(|id| choix.push(id));
        choix
    }
}

impl From<Vec<String>> for Choix {
    fn from(ids: Vec<String>) -> Self {
        ids.into_iter().collect()
    }
}

impl From<Choix> for Vec<String> {
    fn from(choix: Choix) -> Self {
        choix.ordre
    }
}

impl<'a> IntoIterator for &'a Choix {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.ordre.iter()
    }
}

impl IntoIterator for Choix {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.ordre.into_iter()
    }
}

/// Deux sélections sont égales quand elles ont le même ordre : l'ensemble en découle.
impl PartialEq for Choix {
    fn eq(&self, autre: &Self) -> bool {
        self.ordre == autre.ordre
    }
}

impl Eq for Choix {}

impl Hash for Choix {
    fn hash<H: std::hash::Hasher>(&self, etat: &mut H) {
        self.ordre.hash(etat);
    }
}

impl PartialEq<Vec<String>> for Choix {
    fn eq(&self, autre: &Vec<String>) -> bool {
        &self.ordre == autre
    }
}

impl<T: AsRef<str>> PartialEq<[T]> for Choix {
    fn eq(&self, autre: &[T]) -> bool {
        self.ordre.len() == autre.len()
            && self.ordre.iter().zip(autre).all(|(a, b)| a == b.as_ref())
    }
}

impl<T: AsRef<str>, const N: usize> PartialEq<[T; N]> for Choix {
    fn eq(&self, autre: &[T; N]) -> bool {
        self == &autre[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choix(ids: &[&str]) -> Choix {
        ids.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn test_choix_1_l_ordre_est_celui_des_gestes_et_la_reponse_lui_correspond() {
        let c = choix(&["b", "a", "c"]);
        assert_eq!(c, ["b", "a", "c"]);
        assert_eq!(c.last().map(String::as_str), Some("c"));
        assert!(c.contains("a"));
        assert!(c.contains(&"a".to_string()));
        assert!(!c.contains("z"));
    }

    #[test]
    fn test_choix_1_un_doublon_ne_compte_qu_une_fois() {
        let mut c = choix(&["a"]);
        c.push("a".into());
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn test_choix_1_retirer_retire_des_deux_cotes() {
        let mut c = choix(&["a", "b", "c"]);
        c.retain(|id| id != "b");
        assert_eq!(c, ["a", "c"]);
        assert!(!c.contains("b"), "l'ensemble suit la liste");
        c.push("b".into());
        assert_eq!(c, ["a", "c", "b"], "et il peut revenir, à la fin");
        c.clear();
        assert!(c.is_empty());
        assert!(!c.contains("a"));
    }

    /// **La forme qui compte** : une question ne parcourt pas la liste. Un parcours de cent mille
    /// chaînes, cent mille fois, ne finirait pas ; l'épreuve ne mesure aucun temps.
    #[test]
    fn test_choix_1_la_question_ne_parcourt_pas_la_liste() {
        let c: Choix = (0..100_000).map(|i| format!("noeud-{i}")).collect();
        let trouves = (0..100_000)
            .filter(|i| c.contains(format!("noeud-{i}").as_str()))
            .count();
        assert_eq!(trouves, 100_000);
    }
}
