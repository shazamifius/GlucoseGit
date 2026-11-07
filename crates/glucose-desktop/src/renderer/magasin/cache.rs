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
        let ancienne = self.places[n.0 as usize].1.replace(entree);
        if ancienne.is_none() {
            self.tenues += 1;
        }
        ancienne
    }

    /// Retire l'entrée de ce chemin. Son numéro reste le sien.
    pub fn remove(&mut self, src: &str) -> Option<Entree> {
        let n = self.numero_connu(src)?;
        let ancienne = self.places[n.0 as usize].1.take();
        if ancienne.is_some() {
            self.tenues -= 1;
        }
        ancienne
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
