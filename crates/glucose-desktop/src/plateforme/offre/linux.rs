//! **L'offre sous Linux et Android** : `madvise(MADV_FREE)` (fiche 44, phase 3).
//!
//! Le noyau peut alors jeter les pages quand il manque de place, sans les écrire nulle part —
//! le geste de `OfferVirtualMemory`. Mais il ne dit pas, à la reprise, s'il l'a fait, ce que
//! `ReclaimVirtualMemory` dit sous Windows.
//!
//! # Savoir sans demander
//!
//! Une page jetée revient **entièrement nulle**. Il suffit donc de retenir, à l'offre, la place
//! d'un octet non nul de chaque page — son **témoin** : à la reprise, s'il l'est encore, la page
//! est intacte. Une page toute nulle n'a rien à retenir : jetée ou non, elle revient la même.
//! Aucune empreinte à calculer, aucun seuil.
//!
//! # Reprendre sans course
//!
//! Une page offerte reste lisible, et le noyau peut la jeter à tout instant — entre deux
//! lectures. Une **écriture** l'arrête : une page écrite après l'offre n'est plus jetable. La
//! reprise écrit donc le témoin par une seule opération atomique qui ne change pas sa valeur :
//! passée avant que le noyau jette la page, elle la garde, intacte ; après, elle lit le zéro de
//! la page neuve. Entre les deux, rien.
//!
//! Un noyau trop ancien (avant 4.5) refuse `MADV_FREE` : l'offre échoue, les pages restent, et
//! la reprise n'a rien à vérifier — le comportement d'avant.

use std::ptr::NonNull;
use std::sync::atomic::{AtomicU8, Ordering};

/// Où vivent les octets d'une région.
pub enum Bloc {
    /// Des pages obtenues du système : elles seules peuvent être offertes.
    Pages(Pages),
    /// Moins d'une page : elle vit dans le tas, et l'offre n'y rend rien.
    Tas(Vec<u8>),
}

pub struct Pages {
    debut: NonNull<u8>,
    /// Ce que l'appelant a demandé.
    longueur: usize,
    /// Ce qui est obtenu du système : la longueur arrondie à ses pages.
    engagee: usize,
    /// Pour chaque page offerte, la place de son témoin — ou rien pour une page toute nulle.
    temoins: Vec<Option<usize>>,
}

// Sûr : les pages appartiennent à ce seul bloc, comme les octets d'un `Vec`.
unsafe impl Send for Pages {}

/// La taille d'une page de ce système, lue une fois : 4 Kio, mais 16 ou 64 Kio sur certains ARM.
fn page() -> usize {
    static PAGE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    // Sûr : une question au système, sans argument à valider.
    *PAGE.get_or_init(|| {
        usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) })
            .unwrap_or(4096)
            .max(1)
    })
}

impl Bloc {
    pub fn nouveau(longueur: usize) -> Option<Self> {
        match longueur {
            0 => None,
            l if l < page() => Some(Self::Tas(vec![0; l])),
            l => Pages::nouvelles(l).map(Self::Pages),
        }
    }

    pub fn longueur(&self) -> usize {
        match self {
            Self::Pages(p) => p.longueur,
            Self::Tas(v) => v.len(),
        }
    }

    pub fn octets(&self) -> &[u8] {
        match self {
            // Sûr : les pages sont obtenues pour `engagee` octets et vivent autant que le bloc ;
            // seule une `Tenue` y donne accès.
            Self::Pages(p) => unsafe { std::slice::from_raw_parts(p.debut.as_ptr(), p.longueur) },
            Self::Tas(v) => v,
        }
    }

    pub fn octets_mut(&mut self) -> &mut [u8] {
        match self {
            // Sûr : comme `octets`, et l'emprunt exclusif interdit toute autre vue.
            Self::Pages(p) => unsafe {
                std::slice::from_raw_parts_mut(p.debut.as_ptr(), p.longueur)
            },
            Self::Tas(v) => v,
        }
    }

    pub fn offrable(&self) -> bool {
        matches!(self, Self::Pages(_))
    }

    pub fn offrir(&mut self) -> bool {
        match self {
            Self::Pages(p) => p.offrir(),
            Self::Tas(_) => false,
        }
    }

    pub fn reprendre(&mut self) -> bool {
        match self {
            Self::Pages(p) => p.reprendre(),
            Self::Tas(_) => true,
        }
    }
}

impl Pages {
    fn nouvelles(longueur: usize) -> Option<Self> {
        let engagee = longueur.div_ceil(page()) * page();
        // Sûr : une projection anonyme neuve, rendue dans `drop` ; le système la met à zéro.
        let p = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                engagee,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if p == libc::MAP_FAILED {
            return None;
        }
        Some(Self {
            debut: NonNull::new(p.cast::<u8>())?,
            longueur,
            engagee,
            temoins: Vec::new(),
        })
    }

    /// Retient le témoin de chaque page, puis l'offre au noyau.
    fn offrir(&mut self) -> bool {
        let taille = page();
        let base = self.debut.as_ptr();
        self.temoins = (0..self.engagee / taille)
            .map(|k| {
                // Sûr : la page `k` est dans la projection, tenue, et personne d'autre n'y écrit.
                let page = unsafe { std::slice::from_raw_parts(base.add(k * taille), taille) };
                page.iter().position(|&o| o != 0)
            })
            .collect();
        // Sûr : des pages entières de cette projection, désignées par pointeur.
        let offertes = unsafe { libc::madvise(base.cast(), self.engagee, libc::MADV_FREE) } == 0;
        if !offertes {
            self.temoins.clear();
        }
        offertes
    }

    /// Reprend chaque page, et dit si toutes sont intactes.
    fn reprendre(&mut self) -> bool {
        let taille = page();
        let base = self.debut.as_ptr();
        let temoins = std::mem::take(&mut self.temoins);
        temoins.iter().enumerate().all(|(k, temoin)| {
            temoin.is_none_or(|i| {
                // Sûr : l'octet est dans la projection, aligné (un octet l'est toujours), et
                // l'emprunt exclusif du bloc interdit tout autre accès pendant l'opération.
                let octet = unsafe { AtomicU8::from_ptr(base.add(k * taille + i)) };
                // L'écriture, même d'une valeur inchangée, retire la page de l'offre ; l'ordre
                // n'importe pas, seul compte qu'elle soit une écriture.
                octet.fetch_or(0, Ordering::Relaxed) != 0
            })
        })
    }
}

impl Drop for Pages {
    fn drop(&mut self) {
        // Sûr : la projection vient de `mmap`, avec cette taille.
        let _ = unsafe { libc::munmap(self.debut.as_ptr().cast(), self.engagee) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages_de(nombre: usize, octet: u8) -> Bloc {
        let mut b = Bloc::nouveau(nombre * page()).expect("des pages");
        b.octets_mut().fill(octet);
        b
    }

    /// **Une page jetée se sait** : ce que le noyau ferait d'elle — la rendre nulle — et la
    /// reprise le dit.
    #[test]
    fn test_une_page_jetee_se_sait() {
        let mut b = pages_de(4, 7);
        assert!(b.offrir(), "le noyau accepte l'offre");
        if let Bloc::Pages(p) = &b {
            // Sûr : la troisième page de la projection.
            unsafe { std::ptr::write_bytes(p.debut.as_ptr().add(2 * page()), 0, page()) };
        }
        assert!(
            !b.reprendre(),
            "une page jetée se lit comme un contenu perdu"
        );
    }

    /// **Une page toute nulle n'a rien à perdre** : jetée, elle revient la même.
    #[test]
    fn test_une_page_nulle_n_a_rien_a_perdre() {
        let mut b = pages_de(3, 0);
        assert!(b.offrir());
        assert!(b.reprendre());
        assert!(b.octets().iter().all(|&o| o == 0));
    }

    /// **Sans manque de place, tout revient**, octet pour octet.
    #[test]
    fn test_sans_manque_de_place_tout_revient() {
        let mut b = pages_de(5, 0);
        for (i, o) in b.octets_mut().iter_mut().enumerate() {
            *o = (i as u8).wrapping_mul(13).wrapping_add(1);
        }
        assert!(b.offrir());
        assert!(b.reprendre());
        assert!(b
            .octets()
            .iter()
            .enumerate()
            .all(|(i, &o)| o == (i as u8).wrapping_mul(13).wrapping_add(1)));
    }
}
