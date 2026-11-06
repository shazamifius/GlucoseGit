//! **Ce que la machine fait, et ce que Glucose y prend** (CHARGE-1, fiche 86). Sa demande du 6/11 :
//! « dans les logs on ne voit pas l'utilisation actuelle du PC ».
//!
//! Trois parts, qui répondent à des questions différentes :
//!
//! * **`machine_pct`** : tout l'ordinateur, le chiffre du gestionnaire des tâches ;
//! * **`processus_pct`** : Glucose, sur toute la machine, à la même échelle ;
//! * **`fil_pct`** : **le fil qui dessine, sur un seul cœur**. C'est lui qui explique « je lague
//!   alors que rien n'est à 100 % » : à 97 %, il n'y a plus de marge, même à 8 % de la machine.
//!
//! On lit ce que le système a compté, comme [`super::empreinte`] : des compteurs cumulés, dont
//! seule la différence entre deux relevés veut dire quelque chose. Ce qu'un système ne dit pas
//! reste `None`, et se lit « inconnu », jamais zéro. **La carte graphique n'y est pas** : aucun
//! système ne le dit de la même façon, et un chiffre faux serait pire que pas de chiffre.

use std::time::Instant;

/// Ce que le système a compté, à un instant.
#[derive(Debug, Clone, Copy)]
pub struct Compteurs {
    pub quand: Instant,
    /// Toute la machine : le temps occupé, et le temps total, tous cœurs ensemble.
    pub machine: Option<(u64, u64)>,
    /// Le processeur consommé par Glucose, en microsecondes.
    pub processus_us: Option<u64>,
    /// Le processeur consommé par le fil qui appelle, en microsecondes.
    pub fil_us: Option<u64>,
}

/// Les parts entre deux relevés, en pour cent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts {
    pub machine_pct: Option<u64>,
    pub processus_pct: Option<u64>,
    pub fil_pct: Option<u64>,
}

/// **Le relevé**, à faire depuis le fil qui dessine : c'est de lui que `fil_us` parle.
pub fn relever() -> Compteurs {
    Compteurs {
        quand: Instant::now(),
        machine: natif::machine(),
        processus_us: super::empreinte::relever().map(|e| e.processeur_us),
        fil_us: natif::fil_us(),
    }
}

/// Le nombre de cœurs que la machine annonce.
pub fn coeurs() -> u64 {
    std::thread::available_parallelism().map_or(1, |n| n.get() as u64)
}

/// **Les parts entre deux relevés.** Un intervalle vide ne dit rien. Une part se borne à cent :
/// des compteurs relus à des instants un peu différents peuvent la faire dépasser d'un cheveu.
pub fn parts(avant: &Compteurs, apres: &Compteurs, coeurs: u64) -> Parts {
    let duree_us = apres.quand.duration_since(avant.quand).as_micros() as u64;
    let part = |consomme: Option<(u64, u64)>, sur: u64| {
        let (a, b) = consomme?;
        (sur > 0).then(|| (b.saturating_sub(a).saturating_mul(100) / sur).min(100))
    };
    let de = |f: fn(&Compteurs) -> Option<u64>| Some((f(avant)?, f(apres)?));
    Parts {
        machine_pct: avant
            .machine
            .zip(apres.machine)
            .and_then(|((oa, ta), (ob, tb))| part(Some((oa, ob)), tb.saturating_sub(ta))),
        processus_pct: part(de(|c| c.processus_us), duree_us.saturating_mul(coeurs)),
        fil_pct: part(de(|c| c.fil_us), duree_us),
    }
}

/// **Le temps occupé et le temps total de toute la machine**, lus dans la première ligne de
/// `/proc/stat` : `cpu  user nice system idle iowait irq softirq steal …`. Attendre le disque
/// (`iowait`) ne fait travailler aucun cœur. Une fonction pure : le calcul se prouve sur un texte.
#[cfg(any(target_os = "linux", target_os = "android", test))]
fn machine_de_proc_stat(stat: &str) -> Option<(u64, u64)> {
    let mut champs = stat.lines().next()?.split_whitespace();
    if champs.next()? != "cpu" {
        return None;
    }
    let valeurs: Vec<u64> = champs
        .take(8)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let total: u64 = valeurs.iter().sum();
    let repos = valeurs.get(3)? + valeurs.get(4)?;
    Some((total.saturating_sub(repos), total))
}

#[cfg(windows)]
mod natif {
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::{GetCurrentThread, GetSystemTimes, GetThreadTimes};

    fn cent_ns(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }

    /// `GetSystemTimes` compte le repos **dans** le temps du noyau.
    pub fn machine() -> Option<(u64, u64)> {
        let (mut repos, mut noyau, mut utilisateur) = Default::default();
        // SAFETY : trois `FILETIME` à nous, que l'appel remplit.
        unsafe {
            GetSystemTimes(
                Some(&raw mut repos),
                Some(&raw mut noyau),
                Some(&raw mut utilisateur),
            )
            .ok()?;
        }
        let total = cent_ns(noyau) + cent_ns(utilisateur);
        Some((total.saturating_sub(cent_ns(repos)), total))
    }

    pub fn fil_us() -> Option<u64> {
        let (mut creation, mut fin, mut noyau, mut utilisateur) = Default::default();
        // SAFETY : le pseudo-handle de ce fil ne s'ouvre ni ne se ferme, et les quatre `FILETIME`
        // sont à nous.
        unsafe {
            GetThreadTimes(
                GetCurrentThread(),
                &raw mut creation,
                &raw mut fin,
                &raw mut noyau,
                &raw mut utilisateur,
            )
            .ok()?;
        }
        Some((cent_ns(noyau) + cent_ns(utilisateur)) / 10)
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod natif {
    /// Android refuse souvent `/proc/stat` à une application : `None`, donc « inconnu ».
    pub fn machine() -> Option<(u64, u64)> {
        super::machine_de_proc_stat(&std::fs::read_to_string("/proc/stat").ok()?)
    }

    pub fn fil_us() -> Option<u64> {
        super::super::empreinte::temps_de_stat_us(
            &std::fs::read_to_string("/proc/thread-self/stat").ok()?,
        )
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "android")))]
mod natif {
    pub fn machine() -> Option<(u64, u64)> {
        None
    }

    pub fn fil_us() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn compteurs(quand: Instant, machine: (u64, u64), processus: u64, fil: u64) -> Compteurs {
        Compteurs {
            quand,
            machine: Some(machine),
            processus_us: Some(processus),
            fil_us: Some(fil),
        }
    }

    /// **Le cas de son essai** : un fil à fond sur douze cœurs, c'est 8 % de la machine.
    #[test]
    fn test_charge_1_un_fil_a_fond_se_voit_meme_a_huit_pour_cent_de_la_machine() {
        let t = Instant::now();
        let avant = compteurs(t, (1_000, 10_000), 0, 0);
        let apres = compteurs(
            t + Duration::from_secs(1),
            (1_180, 11_000),
            1_000_000,
            970_000,
        );
        assert_eq!(
            parts(&avant, &apres, 12),
            Parts {
                machine_pct: Some(18),
                processus_pct: Some(8),
                fil_pct: Some(97),
            }
        );
    }

    #[test]
    fn test_charge_1_ce_que_le_systeme_ne_dit_pas_reste_inconnu() {
        let t = Instant::now();
        let vide = Compteurs {
            quand: t,
            machine: None,
            processus_us: None,
            fil_us: None,
        };
        let apres = Compteurs {
            quand: t + Duration::from_secs(1),
            ..vide
        };
        assert_eq!(
            parts(&vide, &apres, 4),
            Parts {
                machine_pct: None,
                processus_pct: None,
                fil_pct: None,
            }
        );
    }

    #[test]
    fn test_charge_1_un_intervalle_vide_ne_dit_rien_et_une_part_ne_depasse_pas_cent() {
        let t = Instant::now();
        let a = compteurs(t, (0, 0), 0, 0);
        assert_eq!(parts(&a, &a, 4).fil_pct, None);
        let apres = compteurs(t + Duration::from_secs(1), (0, 0), 0, 2_000_000);
        assert_eq!(parts(&a, &apres, 4).fil_pct, Some(100));
    }

    #[test]
    fn test_charge_1_proc_stat_ne_compte_pas_l_attente_comme_du_travail() {
        let stat = "cpu  100 0 100 700 100 0 0 0 0 0\ncpu0 1 1 1 1 1 1 1 1 1 1\n";
        assert_eq!(machine_de_proc_stat(stat), Some((200, 1000)));
        assert_eq!(machine_de_proc_stat("rien"), None);
    }

    /// **Le temps d'un fil ou d'un processus**, sur une ligne de `stat` dont le nom contient des
    /// parenthèses et des espaces : les champs se comptent après la dernière fermante.
    #[test]
    fn test_charge_1_une_ligne_de_stat_se_lit_apres_son_nom() {
        let stat = "12 (nom (bizarre) a) S 1 1 1 1 1 1 1 1 1 1 250 50 0 0\n";
        assert_eq!(
            super::super::empreinte::temps_de_stat_us(stat),
            Some(3_000_000),
            "250 + 50 tics, à cent par seconde"
        );
        assert_eq!(super::super::empreinte::temps_de_stat_us("sans nom"), None);
    }

    /// **Sur cette machine, le relevé répond**, et le temps ne recule pas.
    #[cfg(target_os = "linux")]
    #[test]
    fn test_charge_1_le_releve_repond_sous_linux() {
        let a = relever();
        assert!(a.machine.is_some() && a.processus_us.is_some() && a.fil_us.is_some());
        let debut = Instant::now();
        while debut.elapsed() < Duration::from_millis(50) {
            std::hint::spin_loop();
        }
        let b = relever();
        assert!(b.fil_us >= a.fil_us);
    }
}
