//! **Les images décodées, par numéro** (PHOTO-1, fiche 87).
//!
//! Un chemin de fichier se compare et se hache lettre à lettre ; chez un tableau de 8 000 images,
//! ces chaînes se clonaient, se hachaient et se comparaient plusieurs fois par photo et par image :
//! une mort à petits coups, 60 % du fil qui dessine (fiche 87 § 2). Ici un chemin se cherche **une
//! fois par image**, et donne un numéro : tout le reste se lit dans un tableau.
//!
//! **Un numéro ne sert jamais à deux fichiers**, même après une éviction : la carte graphique
//! nomme ses textures par numéro, et une photo réattribuée y montrerait les pixels d'une autre. La
//! table ne grandit donc que d'un chemin par fichier que la session a vu : le prix de cette garantie.
//!
//! Les méthodes portent les noms de celles d'une table (`get`, `insert`, `contains_key`…) : le
//! magasin lit un cache, et ce cache reste adressable par son chemin partout où ce n'est pas
//! chaud.

use super::Entree;
use glucose_core::rapide::Rapide;
use std::collections::HashMap;

/// **Le numéro d'un fichier d'image**, pour toute la session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Numero(pub u32);

/// Les images décodées, et le numéro de chaque fichier qu'on a vu.
#[derive(Default)]
pub struct Cache {
    numeros: HashMap<String, Numero, Rapide>,
    /// Le chemin de chaque numéro, et ce qu'on en tient : rien pour un fichier évincé.
    places: Vec<(String, Option<Entree>)>,
    /// Combien de places portent une entrée.
    tenues: usize,
    /// Ce que ces entrées occupent : la taille d'un niveau est fixée à sa naissance, et seul ce qui
    /// entre ou sort la change. La compter balayait tout le cache à chaque image.
    octets: usize,
}

impl Cache {
    /// **Le numéro de ce chemin**, attribué à sa première apparition.
    pub fn numero(&mut self, src: &str) -> Numero {
        if let Some(&n) = self.numeros.get(src) {
            return n;
        }
        let n = Numero(self.places.len() as u32);
        self.numeros.insert(src.to_string(), n);
        self.places.push((src.to_string(), None));
        n
    }

    /// Le numéro de ce chemin, s'il en a un.
    pub fn numero_connu(&self, src: &str) -> Option<Numero> {
        self.numeros.get(src).copied()
    }

    /// Le chemin de ce numéro.
    pub fn chemin(&self, n: Numero) -> Option<&str> {
        self.places.get(n.0 as usize).map(|(src, _)| src.as_str())
    }

    /// Le numéro de ce chemin et son entrée : ce que le rendu demande pour chaque photo.
    pub fn numero_et_entree(&mut self, src: &str) -> Option<(Numero, &mut Entree)> {
        let n = self.numero_connu(src)?;
        Some((n, self.par_numero_mut(n)?))
    }

    /// Le chemin de ce numéro et son entrée, empruntés ensemble.
    pub fn chemin_et_entree(&mut self, n: Numero) -> Option<(&str, &mut Entree)> {
        let (src, entree) = self.places.get_mut(n.0 as usize)?;
        Some((src.as_str(), entree.as_mut()?))
    }

    /// L'entrée de ce numéro.
    pub fn par_numero(&self, n: Numero) -> Option<&Entree> {
        self.places.get(n.0 as usize)?.1.as_ref()
    }

    /// L'entrée de ce numéro, à écrire.
    pub fn par_numero_mut(&mut self, n: Numero) -> Option<&mut Entree> {
        self.places.get_mut(n.0 as usize)?.1.as_mut()
    }

    pub fn get(&self, src: &str) -> Option<&Entree> {
        self.par_numero(self.numero_connu(src)?)
    }

    pub fn get_mut(&mut self, src: &str) -> Option<&mut Entree> {
        let n = self.numero_connu(src)?;
        self.par_numero_mut(n)
    }

    pub fn contains_key(&self, src: &str) -> bool {
        self.get(src).is_some()
    }

    /// Pose cette entrée, et rend celle qu'elle remplace.
    pub fn insert(&mut self, src: String, entree: Entree) -> Option<Entree> {
        let n = self.numero(&src);
        self.octets += entree.pyramide.octets();
        let ancienne = self.places[n.0 as usize].1.replace(entree);
        match &ancienne {
            Some(partie) => self.octets -= partie.pyramide.octets(),
            None => self.tenues += 1,
        }
        ancienne
    }

    /// Retire l'entrée de ce chemin. Son numéro reste le sien.
    pub fn remove(&mut self, src: &str) -> Option<Entree> {
        let n = self.numero_connu(src)?;
        let ancienne = self.places[n.0 as usize].1.take();
        if let Some(partie) = &ancienne {
            self.tenues -= 1;
            self.octets -= partie.pyramide.octets();
        }
        ancienne
    }

    /// Ce que les images occupent, tous états confondus.
    pub fn octets(&self) -> usize {
        self.octets
    }

    /// Combien d'images sont tenues.
    pub fn len(&self) -> usize {
        self.tenues
    }

    pub fn is_empty(&self) -> bool {
        self.tenues == 0
    }

    /// Chaque image tenue, et son chemin.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entree)> {
        self.places
            .iter()
            .filter_map(|(src, e)| e.as_ref().map(|e| (src, e)))
    }

    /// Chaque image tenue, et son chemin, à écrire.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Entree)> {
        self.places
            .iter_mut()
            .filter_map(|(src, e)| e.as_mut().map(|e| (&*src, e)))
    }

    /// Les images tenues.
    pub fn values(&self) -> impl Iterator<Item = &Entree> {
        self.places.iter().filter_map(|(_, e)| e.as_ref())
    }
}

impl std::ops::Index<&str> for Cache {
    type Output = Entree;

    fn index(&self, src: &str) -> &Entree {
        self.get(src).expect("cette image est dans le cache")
    }
}

impl std::ops::Index<&String> for Cache {
    type Output = Entree;

    fn index(&self, src: &String) -> &Entree {
        self.get(src).expect("cette image est dans le cache")
    }
}

impl<'a> IntoIterator for &'a Cache {
    type Item = (&'a String, &'a Entree);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::photo::Pyramide;
    use std::time::Duration;

    fn entree(cote: u32) -> Entree {
        Entree::nouvelle(
            Pyramide::nouvelle(tiny_skia::Pixmap::new(cote, cote).expect("une image")),
            Duration::from_millis(1),
            0,
            0,
        )
    }

    /// **Un numéro ne sert jamais à deux fichiers**, même évincé : la carte nomme ses textures
    /// par lui, et une photo réattribuée y montrerait les pixels d'une autre.
    #[test]
    fn test_photo_1_un_numero_ne_sert_jamais_a_deux_fichiers() {
        let mut cache = Cache::default();
        cache.insert("a".into(), entree(4));
        cache.insert("b".into(), entree(4));
        let (na, nb) = (cache.numero("a"), cache.numero("b"));
        assert_ne!(na, nb);
        assert!(cache.remove("a").is_some());
        assert!(
            cache.par_numero(na).is_none(),
            "évincée : rien sous ce numéro"
        );
        cache.insert("c".into(), entree(4));
        assert_ne!(
            cache.numero("c"),
            na,
            "la place d'une évincée ne se réattribue pas"
        );
        cache.insert("a".into(), entree(4));
        assert_eq!(cache.numero("a"), na, "un fichier revenu retrouve le sien");
        assert_eq!(cache.chemin(nb), Some("b"));
    }

    /// **Le compte suit ce qui entre et sort**, sans balayer : remplacer une entrée ne la compte
    /// pas deux fois, et un retrait de trop ne le fait pas descendre.
    #[test]
    fn test_photo_1_le_compte_suit_ce_qui_entre_et_sort() {
        let mut cache = Cache::default();
        assert!(cache.is_empty());
        cache.insert("a".into(), entree(8));
        cache.insert("b".into(), entree(4));
        let total: usize = cache.values().map(|e| e.pyramide.octets()).sum();
        assert_eq!(cache.octets(), total);
        assert_eq!(cache.len(), 2);
        cache.insert("a".into(), entree(16));
        let total: usize = cache.values().map(|e| e.pyramide.octets()).sum();
        assert_eq!(
            cache.octets(),
            total,
            "une entrée remplacée ne compte qu'une fois"
        );
        assert_eq!(cache.len(), 2);
        cache.remove("a");
        assert!(cache.remove("a").is_none());
        assert!(cache.remove("jamais vue").is_none());
        let total: usize = cache.values().map(|e| e.pyramide.octets()).sum();
        assert_eq!(cache.octets(), total);
        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache
                .iter()
                .map(|(src, _)| src.as_str())
                .collect::<Vec<_>>(),
            ["b"]
        );
    }
}
