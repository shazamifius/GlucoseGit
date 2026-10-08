//! **Ce que la machine dit d'elle-même** : la batterie, la charge, et depuis quand elle est
//! allumée.
//!
//! Chaque système a ses sondes ; là où il n'en a pas — ou pas encore ici —, la sonde dit
//! qu'elle ne sait pas (`None`), et la boîte noire écrit `null` plutôt qu'un zéro qui se lirait
//! comme une mesure. La chaleur viendra avec les téléphones : Android la dit (fiche 44 § 1.1).
//!
//! Les sondes se lisent sur le fil d'écriture, jamais sur celui qui dessine : lire `/sys` est une
//! entrée-sortie.

/// L'état de la machine qu'on relève.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EtatMachine {
    /// La charge de la batterie, de 0 à 100 ; `None` sans batterie, ou si le système ne le dit
    /// pas.
    pub batterie_pct: Option<u8>,
    /// Branchée sur le secteur ?
    pub en_charge: Option<bool>,
}

/// L'état de la machine, maintenant.
pub fn etat_de_la_machine() -> EtatMachine {
    imp::etat()
}

/// **Le démarrage de l'appareil**, en millisecondes depuis 1970 : l'heure présente moins le
/// temps écoulé depuis le démarrage, que le système compte sommeil compris.
pub fn demarrage_de_l_appareil_ms() -> Option<u64> {
    let maintenant = maintenant_ms();
    imp::depuis_le_demarrage_ms().map(|d| maintenant.saturating_sub(d))
}

/// L'heure présente, en millisecondes depuis 1970.
pub fn maintenant_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis().min(u128::from(u64::MAX)) as u64)
}

#[cfg(windows)]
mod imp {
    use super::EtatMachine;
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

    /// `BatteryFlag` : aucune batterie dans ce système.
    const SANS_BATTERIE: u8 = 128;
    /// La valeur que Windows donne quand il ne sait pas.
    const INCONNU: u8 = 255;

    pub fn etat() -> EtatMachine {
        let mut s = SYSTEM_POWER_STATUS::default();
        // SAFETY : `s` est une structure valide, que l'appel remplit.
        if unsafe { GetSystemPowerStatus(&mut s) }.is_err() {
            return EtatMachine::default();
        }
        let batterie = s.BatteryFlag != SANS_BATTERIE && s.BatteryFlag != INCONNU;
        EtatMachine {
            batterie_pct: (batterie && s.BatteryLifePercent <= 100).then_some(s.BatteryLifePercent),
            en_charge: match s.ACLineStatus {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            },
        }
    }

    pub fn depuis_le_demarrage_ms() -> Option<u64> {
        // Sommeil et veille prolongée compris : c'est ce qui fait de la différence avec
        // l'heure présente un instant fixe.
        // SAFETY : un compteur du système, sans argument ni état partagé.
        Some(unsafe { windows::Win32::System::SystemInformation::GetTickCount64() })
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod imp {
    use super::EtatMachine;
    use std::fs;

    /// La première batterie que le noyau déclare, et ce qu'elle dit — sous Android, ce que le
    /// système en dit à tous (`plateforme::batterie`) : il ferme `/sys` aux applications.
    pub fn etat() -> EtatMachine {
        if let Some(brut) = crate::plateforme::batterie::lire() {
            return EtatMachine {
                batterie_pct: brut.pourcentage(),
                en_charge: brut.en_charge(),
            };
        }
        let mut etat = EtatMachine::default();
        let Ok(sources) = fs::read_dir("/sys/class/power_supply") else {
            return etat;
        };
        for source in sources.flatten() {
            let p = source.path();
            let lire = |nom: &str| fs::read_to_string(p.join(nom)).map(|t| t.trim().to_string());
            match lire("type").as_deref() {
                Ok("Battery") if etat.batterie_pct.is_none() => {
                    etat.batterie_pct = lire("capacity")
                        .ok()
                        .and_then(|t| t.parse::<u8>().ok())
                        .filter(|&n| n <= 100);
                }
                // Le secteur, ou un chargeur USB : l'un branché suffit.
                Ok("Mains") | Ok("USB") => {
                    if let Ok(t) = lire("online") {
                        etat.en_charge = Some(etat.en_charge == Some(true) || t == "1");
                    }
                }
                _ => {}
            }
        }
        etat
    }

    /// `CLOCK_BOOTTIME` : le temps depuis le démarrage, sommeil compris — l'horloge que le
    /// noyau tient, sans lire de fichier ; Android ferme `/proc/uptime` à certaines
    /// applications (son journal l'envoyait vide).
    pub fn depuis_le_demarrage_ms() -> Option<u64> {
        // SAFETY : `t` est une structure de nombres, que l'appel remplit.
        let mut t: libc::timespec = unsafe { std::mem::zeroed() };
        if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut t) } != 0 {
            return None;
        }
        let ms = i128::from(t.tv_sec) * 1000 + i128::from(t.tv_nsec) / 1_000_000;
        u64::try_from(ms).ok()
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "android")))]
mod imp {
    use super::EtatMachine;

    /// Le Mac viendra en son temps (fiche 44, phase 8) : d'ici là, la sonde ne sait pas.
    pub fn etat() -> EtatMachine {
        EtatMachine::default()
    }

    pub fn depuis_le_demarrage_ms() -> Option<u64> {
        None
    }
}
