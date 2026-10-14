//! **Un décodage ne commence que s'il tient dans ce que la machine laisse** (MEMOIRE-3,
//! fiche 66).
//!
//! # Ce qui n'était borné par rien
//!
//! Le magasin borne ce qu'il **garde** : la moitié de ce que le système laisse, relu à chaque
//! image (ADAPT-1). Mais un décodage prend sa mémoire **avant** d'entrer au magasin, et l'atelier
//! en lançait autant que la machine a de cœurs moins un — sept sur son téléphone. Sept photos de
//! cinquante mégapixels demandaient ensemble plus de trois gigaoctets, sans regarder ce qui
//! restait. Un téléphone qui manque de mémoire ne tue pas l'application qu'on regarde : il
//! compresse, il ralentit, puis il se fige — le récit exact du 13/10, à prouver par le
//! téléphone.
//!
//! # La règle, et pourquoi elle n'a pas de nombre
//!
//! Celle du cache, appliquée à ce qui est en vol : l'ensemble des décodages en cours ne prend
//! jamais plus que la moitié de ce que la machine laisse **maintenant**
//! ([`Memoire::part_pour_un_cache`]). Un décodage qui ne tient pas attend qu'un autre finisse —
//! le rendu, lui, n'attend jamais (DECODE-1). Et un décodage **seul** passe toujours : sinon une
//! image plus grande que la moitié de ce qui reste ne viendrait jamais.
//!
//! Ce que la machine laisse se relit à chaque question, et compte déjà ce que les décodages en
//! vol ont pris : la règle est donc prudente — elle compte deux fois ce qui est déjà alloué. Sur
//! un ordinateur qui a vingt gigaoctets de libres, elle ne retient rien ; sur un téléphone
//! serré, les grandes photos passent une à une.

use crate::memoire::Memoire;
use std::sync::{Condvar, Mutex, PoisonError};

/// **Ce que les décodages en cours tiendront au plus**, et la porte qui en admet un de plus.
pub struct Admission {
    en_vol: Mutex<u64>,
    libere: Condvar,
    /// Ce que la machine dit d'elle-même : le système pour l'atelier, une valeur choisie pour
    /// une épreuve — la règle se prouve sur des nombres fixés.
    machine: fn() -> Option<Memoire>,
}

/// **Un décodage admis** : ses octets restent comptés tant qu'il vit, et sa fin réveille ceux
/// qui attendent. Il ne sort jamais de [`Admission::pendant`] : le garder le temps exact du
/// travail ne dépend d'aucun nom de variable.
struct Jeton<'a> {
    admission: &'a Admission,
    octets: u64,
}

/// **La porte de tous les ateliers du processus**, qui lit la machine : la mémoire est celle du
/// processus, et deux fenêtres qui décoderaient chacune derrière sa propre porte en prendraient
/// le double.
pub fn de_la_machine() -> &'static Admission {
    static PORTE: std::sync::OnceLock<Admission> = std::sync::OnceLock::new();
    PORTE.get_or_init(|| Admission::avec(Memoire::du_systeme))
}

impl Admission {
    /// Une porte qui lit la machine par `machine`.
    pub fn avec(machine: fn() -> Option<Memoire>) -> Self {
        Self {
            en_vol: Mutex::new(0),
            libere: Condvar::new(),
            machine,
        }
    }

    /// **Fait `travail` une fois admis**, en comptant ses `octets` le temps qu'il dure — ni
    /// plus tôt, ni plus tard.
    ///
    /// Sur une machine qui ne dit pas sa mémoire, tout passe : la règle d'avant, qui ne
    /// demandait rien.
    pub fn pendant<T>(&self, octets: u64, travail: impl FnOnce() -> T) -> T {
        let _admis = self.entrer(octets);
        travail()
    }

    /// Attend que ces octets tiennent, puis les compte jusqu'à la fin du jeton.
    fn entrer(&self, octets: u64) -> Jeton<'_> {
        let mut en_vol = self.en_vol.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            let tient = *en_vol == 0
                || (self.machine)().is_none_or(|m| *en_vol + octets <= m.part_pour_un_cache());
            if tient {
                *en_vol += octets;
                return Jeton {
                    admission: self,
                    octets,
                };
            }
            en_vol = self
                .libere
                .wait(en_vol)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Ce que les décodages admis tiennent en ce moment.
    pub fn en_vol(&self) -> u64 {
        *self.en_vol.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Jeton<'_> {
    fn drop(&mut self) {
        let mut en_vol = self
            .admission
            .en_vol
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *en_vol -= self.octets;
        self.admission.libere.notify_all();
    }
}

/// **Ce qu'un décodage tiendra au plus**, en octets : ses octets encodés, sa pyramide entière,
/// et ce que le décodeur garde pour travailler.
///
/// Une borne, et chaque terme dit d'où il vient. La pyramide se somme niveau par niveau,
/// exactement. Le travail du décodeur, au-delà de la pyramide, est **mesuré**
/// (`bench_decodage`, 50 mégapixels, chaque chemin dans son processus) : rien pour un JPEG
/// ordinaire — celui des appareils photo —, 0,8 octet par pixel pour un progressif 4:2:0, 3,8
/// pour un progressif 4:4:4, le plus gourmand. D'où quatre octets par pixel. Ma première borne,
/// tirée du papier (six octets : chaque coefficient sur seize bits), comptait 300 Mo de trop
/// pour cette photo — plus que sa pyramide.
pub fn pic(encodes: u64, (largeur, hauteur): (u32, u32)) -> u64 {
    let pyramide: u64 = (0..)
        .map_while(|rang| super::super::photo::dimensions_au_rang((largeur, hauteur), rang))
        .map(|(l, h)| u64::from(l) * u64::from(h) * 4)
        .sum();
    encodes + pyramide + u64::from(largeur) * u64::from(hauteur) * 4
}

#[cfg(test)]
mod tests;
