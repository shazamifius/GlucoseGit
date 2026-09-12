//! Les attributs que peu de nœuds portent : tables creuses — `std` uniquement.
//!
//! # Le problème, et la formule qui le tranche
//!
//! Le tronc de l'arène range ce que *tous* les nœuds ont : position, taille, genre, parent.
//! Mais un nœud peut porter une couleur personnalisée, une ancre temporelle, une taille de
//! police, un lien de miroir — et presque aucun ne les porte toutes. Les ranger dans le tronc
//! ferait payer à dix millions de nœuds ce que quelques milliers utilisent.
//!
//! La question « tableau dense ou table creuse ? » n'appelle pas un jugement, elle a une
//! réponse exacte. Soit `n` nœuds, `k` porteurs, `s` octets par valeur :
//!
//! ```text
//!     dense  =  n × s                    (une valeur par nœud, sentinelle comprise)
//!     creux  =  k × (4 + s)              (l'identifiant en plus de la valeur)
//!
//!     creux < dense   ⟺   k/n  <  s / (4 + s)
//! ```
//!
//! | Taille de la valeur | Seuil |
//! |---|---:|
//! | 1 octet (un drapeau, un genre) | 20 % |
//! | 4 octets (une couleur, un [`NodeId`]) | 50 % |
//! | 8 octets (un `f64`, un intervalle) | 67 % |
//! | 40 octets (une ancre temporelle) | 91 % |
//!
//! [`prefer_sparse`] est cette formule, et [`Sparse::would_fit_denser`] la pose sur une table
//! réelle : une table creuse qui a trop grossi le dit elle-même, plutôt que d'attendre qu'on
//! s'en aperçoive. C'est une constante qui n'existe pas — il n'y a rien à régler.
//!
//! # Ce que la table creuse coûte
//!
//! Les identifiants sont **triés** : la lecture est une recherche binaire, et le chargement
//! d'un document — qui pose les attributs dans l'ordre des nœuds — n'est qu'une suite d'ajouts
//! en queue. L'écriture d'un identifiant déjà dépassé décale la fin de la table ; c'est le prix
//! d'une structure dont les lectures sont fréquentes et les écritures rares, ce qu'est
//! exactement un attribut de nœud.

use super::NodeId;

/// Vrai si une table creuse pèse moins qu'un tableau dense, pour `carriers` porteurs sur
/// `nodes` nœuds et une valeur de `value_bytes` octets.
///
/// C'est la formule de l'en-tête du module, sans arrondi : `k × (4 + s) < n × s`. Elle décide,
/// il n'y a pas de seuil à choisir.
pub const fn prefer_sparse(carriers: usize, nodes: usize, value_bytes: usize) -> bool {
    carriers * (std::mem::size_of::<NodeId>() + value_bytes) < nodes * value_bytes
}

/// Un attribut porté par une minorité de nœuds : les identifiants triés d'un côté, les valeurs
/// de l'autre.
///
/// Les deux vecteurs sont de même longueur — invariant SPR-1, vérifié par [`Sparse::check`] —
/// et `ids` est strictement croissant, ce qui rend la lecture binaire et interdit les doublons.
#[derive(Debug, Clone)]
pub struct Sparse<V> {
    ids: Vec<NodeId>,
    vals: Vec<V>,
}

impl<V> Default for Sparse<V> {
    fn default() -> Self {
        Self { ids: Vec::new(), vals: Vec::new() }
    }
}

impl<V> Sparse<V> {
    /// Une table vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Une table vide, dimensionnée pour `k` porteurs.
    pub fn with_capacity(k: usize) -> Self {
        Self { ids: Vec::with_capacity(k), vals: Vec::with_capacity(k) }
    }

    /// Le nombre de porteurs.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Vrai si aucun nœud ne porte cet attribut.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Les octets occupés par la table, hors ce que les valeurs allouent elles-mêmes.
    pub fn bytes(&self) -> usize {
        self.len() * (std::mem::size_of::<NodeId>() + std::mem::size_of::<V>())
    }

    /// Vrai si un tableau dense pèserait moins que cette table, sur un document de `nodes`
    /// nœuds. Une table creuse qui a trop grossi le dit elle-même.
    pub fn would_fit_denser(&self, nodes: usize) -> bool {
        !prefer_sparse(self.len(), nodes, std::mem::size_of::<V>())
    }

    /// Le rang d'un identifiant, ou le rang où il s'insérerait.
    fn seek(&self, id: NodeId) -> Result<usize, usize> {
        self.ids.binary_search(&id)
    }

    /// La valeur portée par un nœud, si elle existe.
    pub fn get(&self, id: NodeId) -> Option<&V> {
        self.seek(id).ok().map(|i| &self.vals[i])
    }

    /// La valeur portée par un nœud, modifiable.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut V> {
        self.seek(id).ok().map(|i| &mut self.vals[i])
    }

    /// Vrai si le nœud porte cet attribut.
    pub fn has(&self, id: NodeId) -> bool {
        self.seek(id).is_ok()
    }

    /// Pose la valeur d'un nœud et rend celle qu'elle remplace.
    ///
    /// Poser l'attribut d'un identifiant supérieur à tous les autres — ce que fait le
    /// chargement d'un document, qui parcourt les nœuds dans l'ordre — est un ajout en queue.
    /// Rend `None` et n'écrit rien pour [`NodeId::NONE`].
    pub fn set(&mut self, id: NodeId, v: V) -> Option<V> {
        if !id.is_some() {
            return None;
        }
        match self.seek(id) {
            Ok(i) => Some(std::mem::replace(&mut self.vals[i], v)),
            Err(i) => {
                self.ids.insert(i, id);
                self.vals.insert(i, v);
                None
            }
        }
    }

    /// Retire l'attribut d'un nœud et rend sa valeur.
    pub fn remove(&mut self, id: NodeId) -> Option<V> {
        match self.seek(id) {
            Ok(i) => {
                self.ids.remove(i);
                Some(self.vals.remove(i))
            }
            Err(_) => None,
        }
    }

    /// Les porteurs et leurs valeurs, dans l'ordre des identifiants.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &V)> {
        self.ids.iter().copied().zip(self.vals.iter())
    }

    /// Vérifie l'invariant SPR-1 : autant de valeurs que d'identifiants, strictement croissants.
    pub fn check(&self) -> Result<(), String> {
        if self.ids.len() != self.vals.len() {
            return Err(format!(
                "SPR-1 : {} identifiants pour {} valeurs",
                self.ids.len(),
                self.vals.len()
            ));
        }
        for pair in self.ids.windows(2) {
            if pair[0] >= pair[1] {
                return Err(format!(
                    "SPR-1 : les identifiants ne sont pas strictement croissants ({} puis {})",
                    pair[0].index(),
                    pair[1].index()
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
