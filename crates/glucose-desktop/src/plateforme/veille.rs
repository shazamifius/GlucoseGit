//! **VEILLE-1 — le temps endormi que l'horloge des images compte** (fiches 123 § 6, 124 § 7, 126 § 2).
//!
//! La durée d'une image se lit à un `Instant`. Sous Windows, il lit `QueryPerformanceCounter`, qui
//! **compte** le temps passé en veille : une image interrompue par une mise en veille a duré 415 s
//! chez un utilisateur à carte AMD, sa machine à 0 % pendant ces sept minutes — et la « pire image »
//! de la télémétrie disait un gel qui n'avait pas eu lieu. Sous Linux et Android, `Instant` lit
//! `CLOCK_MONOTONIC`, et sous macOS une horloge qui s'arrête de même : la veille n'y entre pas.
//!
//! [`endormi`] rend **le temps passé en veille depuis le démarrage, tel que l'`Instant` de cette
//! plateforme le compte** : sous Windows, l'écart entre le temps d'interruption, qui compte la
//! veille, et le temps « non biaisé », qui ne la compte pas (`QueryUnbiasedInterruptTimePrecise` :
//! *« does not include time the system spends in sleep or hibernation »*) ; ailleurs, **zéro par
//! construction** — retrancher « BOOTTIME moins MONOTONIC » d'une durée qui ne compte pas la veille
//! la rendrait négative (la loi de la fiche 124 § 7 valait pour Windows seul). Une image retranche
//! ce qui en a grandi pendant elle : elle garde ce qu'elle a coûté, et un vrai gel, la machine
//! éveillée, reste entier.

use std::time::Duration;

/// Le temps passé en veille depuis le démarrage, tel que l'`Instant` de cette plateforme le compte.
#[cfg(windows)]
pub fn endormi() -> Duration {
    use windows::Win32::System::WindowsProgramming::{
        QueryInterruptTimePrecise, QueryUnbiasedInterruptTimePrecise,
    };
    // SAFETY : deux lectures d'horloge sans argument, de la même famille (« Precise », Windows 10
    // et après), en unités de 100 ns.
    let (avec, sans) = unsafe {
        (
            QueryInterruptTimePrecise(),
            QueryUnbiasedInterruptTimePrecise(),
        )
    };
    Duration::from_nanos(avec.saturating_sub(sans).saturating_mul(100))
}

/// Le temps passé en veille depuis le démarrage, tel que l'`Instant` de cette plateforme le compte :
/// aucun — il ne compte pas la veille.
#[cfg(not(windows))]
pub fn endormi() -> Duration {
    Duration::ZERO
}

/// Ce que l'horloge des images a compté de veille depuis cette lecture d'[`endormi`].
pub fn depuis(avant: Duration) -> Duration {
    endormi().saturating_sub(avant)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Le temps endormi ne recule jamais** ; hors de Windows, il est nul — `Instant` n'y compte
    /// pas la veille, et rien ne doit être retranché.
    #[test]
    fn test_veille_1_le_temps_endormi_ne_recule_pas() {
        let avant = endormi();
        let apres = endormi();
        assert!(apres >= avant, "{avant:?} puis {apres:?}");
        if cfg!(not(windows)) {
            assert_eq!((avant, depuis(avant)), (Duration::ZERO, Duration::ZERO));
        }
    }
}
