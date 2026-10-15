//! **Comment le système a fermé Glucose, là où il le garde** (SORTIE-1, fiche 72).
//!
//! Depuis Android 11, le système garde pour chaque application la raison des dernières morts de
//! ses processus (`ActivityManager.getHistoricalProcessExitReasons`) : tuée faute de mémoire,
//! plantée, gelée, arrêtée par l'utilisateur… avec la mémoire qu'elle occupait à cet instant.
//! Sous Windows, c'est le journal Application qui dit les plantages ([`super::journal`]).
//!
//! Le 15/10/2026, son tableau de 6 000 images s'est fermé « de lui-même » sur deux téléphones,
//! et aucune ligne ne pouvait dire pourquoi : une application qu'Android tue n'écrit rien.
//! `MainActivity` lit ces raisons (`lireLesSorties`) et confie les nombres ; la boîte noire
//! reconnaît parmi eux la session d'avant ([`crate::boite_noire::sortie`]).

use std::sync::{Mutex, PoisonError};

/// Une mort d'un processus de Glucose, telle qu'Android l'a gardée : des nombres, rien d'autre
/// — sa description, un texte du système, ne se lit pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sortie {
    /// Le numéro du processus mort.
    pub processus: u32,
    /// L'instant de sa mort, en millisecondes depuis 1970.
    pub instant_ms: u64,
    /// `ApplicationExitInfo.getReason()`.
    pub raison: i32,
    /// Le code de sortie, ou le signal qui l'a tué (`getStatus()`).
    pub statut: i32,
    /// Son importance à cet instant (`getImportance()` : 100 au premier plan, 400 en cache).
    pub importance: i32,
    /// Sa mémoire à sa mort, en kibioctets (`getPss()`, `getRss()`).
    pub pss_ko: u64,
    pub rss_ko: u64,
}

/// Ce qui lit les sorties chez le système.
pub type Lecteur = Box<dyn Fn() -> Vec<Sortie> + Send>;

static LECTEUR: Mutex<Option<Lecteur>> = Mutex::new(None);

/// **Branche ce qui lit les sorties** — sous Android, au lancement.
pub fn installer(lecteur: Lecteur) {
    *LECTEUR.lock().unwrap_or_else(PoisonError::into_inner) = Some(lecteur);
}

/// Les dernières morts des processus de Glucose que le système a gardées — aucune là où rien
/// ne les lit.
pub fn lire() -> Vec<Sortie> {
    LECTEUR
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .map(|l| l())
        .unwrap_or_default()
}

/// **Les sorties, depuis le tableau plat que Java rend** : sept nombres par mort, dans l'ordre
/// de [`Sortie`]. Un reste incomplet, ou un nombre hors de sa grandeur, ne fait pas de sortie —
/// le reste, le type l'écarte (`as_chunks`).
pub fn depuis_le_tableau(v: &[i64]) -> Vec<Sortie> {
    v.as_chunks::<7>()
        .0
        .iter()
        .filter_map(|c| {
            Some(Sortie {
                processus: u32::try_from(c[0]).ok()?,
                instant_ms: u64::try_from(c[1]).ok()?,
                raison: i32::try_from(c[2]).ok()?,
                statut: i32::try_from(c[3]).ok()?,
                importance: i32::try_from(c[4]).ok()?,
                pss_ko: u64::try_from(c[5]).ok()?,
                rss_ko: u64::try_from(c[6]).ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Sept nombres font une sortie**, dans l'ordre que Java les écrit ; un reste incomplet
    /// et un nombre négatif là où il n'en faut pas n'en font aucune.
    #[test]
    fn test_sortie_1_le_tableau_de_java_se_lit_par_sept() {
        let v = [
            4242,
            1_791_000_000_000,
            3,
            9,
            100,
            2_000_000,
            2_500_000, // une mort entière
            -1,
            1,
            2,
            3,
            4,
            5,
            6, // un processus négatif
            1,
            2,
            3, // un reste
        ];
        assert_eq!(
            depuis_le_tableau(&v),
            vec![Sortie {
                processus: 4242,
                instant_ms: 1_791_000_000_000,
                raison: 3,
                statut: 9,
                importance: 100,
                pss_ko: 2_000_000,
                rss_ko: 2_500_000,
            }]
        );
    }
}
