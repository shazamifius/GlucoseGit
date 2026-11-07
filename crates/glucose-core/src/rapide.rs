//! **Un hachage pour ce qui se cherche à chaque image** (PHOTO-1, fiche 87).
//!
//! # Pourquoi pas celui de la bibliothèque standard
//!
//! Le hachage par défaut de Rust (SipHash) protège une table contre un adversaire qui choisirait
//! ses clés. Ici les clés sont les chemins des images d'un document et des numéros de nœud, et
//! personne n'y gagnerait rien. Il coûtait 10 % du fil qui dessine sur huit mille images, à
//! faire plusieurs fois par photo et par image un travail de chiffrement sans objet (fiche 86).
//!
//! # La forme
//!
//! Une **multiplication repliée**, mot par mot : le produit sur 128 bits, dont on combine la moitié
//! haute et la moitié basse. Chaque bit d'entrée y touche tous les bits de sortie.
//!
//! Le mélange plus simple de `FxHash` (pivoter, combiner, multiplier) a été essayé d'abord, et son
//! épreuve l'a refusé : **640 collisions entières sur 8 000 chemins** qui ne diffèrent que par
//! leurs chiffres. Un écart dans les bits hauts d'un mot revenait, par le pivot, dans les bits bas
//! du mot suivant, là où les chiffres diffèrent aussi, et s'y annulait. `rustc-hash` 2 a quitté ce
//! mélange pour les octets pour la même raison, et `foldhash` repose sur ce repli. Le noyau reste
//! sans dépendance : ces quelques lignes valent mieux qu'une crate.

use std::hash::{BuildHasherDefault, Hasher};

/// Le multiplicateur, impair : celui de `rustc-hash` 2.
const K: u64 = 0xf135_7aea_2e62_a9c5;
/// Ce qui empêche un état nul et un mot nul de rester à zéro : des décimales de π.
const GRAIN: u64 = 0x243f_6a88_85a3_08d3;

/// L'état d'un hachage en cours.
#[derive(Default, Clone, Copy)]
pub struct Melange(u64);

impl Melange {
    #[inline]
    fn ajouter(&mut self, mot: u64) {
        let produit = u128::from(self.0 ^ mot ^ GRAIN) * u128::from(K);
        self.0 = (produit as u64) ^ ((produit >> 64) as u64);
    }
}

impl Hasher for Melange {
    #[inline]
    fn write(&mut self, octets: &[u8]) {
        let (mots, reste) = octets.as_chunks::<8>();
        for mot in mots {
            self.ajouter(u64::from_le_bytes(*mot));
        }
        // Le reste, complété par la longueur : `"a\0"` et `"a"` ne se confondent pas, et une clé
        // de huit zéros ne ressemble pas à celle de seize.
        let mut dernier = [0u8; 8];
        dernier[..reste.len()].copy_from_slice(reste);
        dernier[7] = octets.len() as u8;
        self.ajouter(u64::from_le_bytes(dernier));
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.ajouter(u64::from(n));
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        self.ajouter(u64::from(n));
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.ajouter(n);
    }

    #[inline]
    fn write_i32(&mut self, n: i32) {
        self.ajouter(u64::from(n as u32));
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.ajouter(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Ce qu'on donne à une table : `HashMap<K, V, Rapide>`.
pub type Rapide = BuildHasherDefault<Melange>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::{BuildHasher, Hash};

    fn de<T: Hash>(valeur: &T) -> u64 {
        Rapide::default().hash_one(valeur)
    }

    fn brut(octets: &[u8]) -> u64 {
        let mut h = Melange::default();
        h.write(octets);
        h.finish()
    }

    /// **Deux longueurs ne se confondent pas** : le reste d'une clé porte sa longueur.
    #[test]
    fn test_rapide_des_chaines_voisines_ne_se_confondent_pas() {
        let mut vus = HashSet::new();
        for longueur in 0..32 {
            assert!(vus.insert(de(&"a".repeat(longueur))), "longueur {longueur}");
        }
        assert_ne!(brut(&[1, 0]), brut(&[1]));
        assert_ne!(brut(&[0; 7]), brut(&[0; 6]));
        assert_ne!(brut(&[0; 8]), brut(&[0; 16]));
        assert_ne!(
            brut(&[1, 2, 3, 4, 5, 6, 7, 8]),
            brut(&[1, 2, 3, 4, 5, 6, 7])
        );
    }

    /// **Le mot de la fin compte, et le premier aussi** : deux chemins ne diffèrent souvent que
    /// par leur dernier chiffre, et une table les mettrait tous au même endroit.
    #[test]
    fn test_rapide_les_chemins_d_un_lot_se_repartissent() {
        let chemins: Vec<String> = (0..8000)
            .map(|i| format!("/maison/images/photo-{i:05}.jpg"))
            .collect();
        let hachages: HashSet<u64> = chemins.iter().map(de).collect();
        assert_eq!(hachages.len(), chemins.len(), "aucune collision entière");
        // Les bits bas (l'emplacement) et les hauts (le contrôle de `hashbrown`).
        let bas: HashSet<u64> = hachages.iter().map(|h| h & 0xFFF).collect();
        let hauts: HashSet<u64> = hachages.iter().map(|h| h >> 57).collect();
        assert!(bas.len() > 3000, "{} emplacements sur 4096", bas.len());
        assert_eq!(hauts.len(), 128, "les sept bits de contrôle");
    }

    /// **Des entiers qui se suivent se répartissent aussi** : c'est le cas d'un numéro de photo.
    #[test]
    fn test_rapide_des_numeros_qui_se_suivent_se_repartissent() {
        let hachages: Vec<u64> = (0u32..8000).map(|i| de(&i)).collect();
        let bas: HashSet<u64> = hachages.iter().map(|h| h & 0xFFF).collect();
        let hauts: HashSet<u64> = hachages.iter().map(|h| h >> 57).collect();
        assert!(bas.len() > 3000, "{} emplacements sur 4096", bas.len());
        assert_eq!(hauts.len(), 128);
    }
}
