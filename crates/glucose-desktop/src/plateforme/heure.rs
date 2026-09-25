//! **L'heure que montre l'horloge de l'utilisateur**, pour un instant donné.
//!
//! Un jalon de la Time Machine disait « il y a 3 h » ; il demande **la date exacte** (registre de
//! Tauri, entrée 12). Un instant du document est en millisecondes Unix, donc en temps universel :
//! l'écrire à l'heure locale demande le fuseau **et la règle d'heure d'été de cette date-là** —
//! un jalon posé en juillet, relu en décembre, garde son heure d'été. Seul le système la connaît,
//! pour chaque date passée : c'est ce que `SystemTimeToTzSpecificLocalTime` fait sous Windows.
//!
//! Ailleurs, rien pour l'instant : sans fuseau connu, `None`, et le panneau garde la durée
//! relative, qui reste vraie. Linux et macOS demanderont `localtime_r` (fiche 36, phase 9).

/// Une date et une heure locales, à la minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeureLocale {
    pub annee: u16,
    pub mois: u8,
    pub jour: u8,
    pub heure: u8,
    pub minute: u8,
}

impl std::fmt::Display for HeureLocale {
    /// Le style court de `fr-FR`, celui des dates de Glucose Tauri : `25/09/2026 22:19`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}/{:02}/{} {:02}:{:02}",
            self.jour, self.mois, self.annee, self.heure, self.minute
        )
    }
}

/// L'heure locale de l'instant `instant_ms` (millisecondes Unix), ou `None` si le système ne
/// la donne pas.
pub fn heure_locale(instant_ms: i64) -> Option<HeureLocale> {
    imp::heure_locale(instant_ms)
}

#[cfg(windows)]
mod imp {
    use super::HeureLocale;
    use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows::Win32::System::Time::{FileTimeToSystemTime, SystemTimeToTzSpecificLocalTime};

    /// Du 1er janvier 1601 (l'origine de Windows) au 1er janvier 1970, en centaines de
    /// nanosecondes : 369 ans, dont 89 bissextiles.
    const DE_1601_A_1970: i64 = 116_444_736_000_000_000;

    pub(super) fn heure_locale(instant_ms: i64) -> Option<HeureLocale> {
        let t = instant_ms
            .checked_mul(10_000)?
            .checked_add(DE_1601_A_1970)?;
        let t = u64::try_from(t).ok()?;
        let ft = FILETIME {
            dwLowDateTime: t as u32,
            dwHighDateTime: (t >> 32) as u32,
        };
        let mut utc = SYSTEMTIME::default();
        let mut local = SYSTEMTIME::default();
        // SAFETY : les deux appels lisent et écrivent des structures possédées ici, vivantes
        // pendant l'appel ; `None` demande le fuseau courant du système.
        unsafe {
            FileTimeToSystemTime(&ft, &mut utc).ok()?;
            SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).ok()?;
        }
        Some(HeureLocale {
            annee: local.wYear,
            mois: u8::try_from(local.wMonth).ok()?,
            jour: u8::try_from(local.wDay).ok()?,
            heure: u8::try_from(local.wHour).ok()?,
            minute: u8::try_from(local.wMinute).ok()?,
        })
    }
}

#[cfg(not(windows))]
mod imp {
    pub(super) fn heure_locale(_instant_ms: i64) -> Option<super::HeureLocale> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le format est celui de Tauri, zéros compris.
    #[test]
    fn test_une_heure_s_ecrit_au_style_court_francais() {
        let h = HeureLocale {
            annee: 2026,
            mois: 9,
            jour: 5,
            heure: 8,
            minute: 3,
        };
        assert_eq!(h.to_string(), "05/09/2026 08:03");
    }

    /// **L'instant se lit à l'heure locale** — confronté à un autre chemin du système :
    /// l'heure que `GetLocalTime` donne maintenant doit être celle que la conversion donne de
    /// l'instant présent, à la minute. Une conversion qui rendrait le temps universel ne
    /// passerait que sur une machine réglée sur Greenwich.
    #[cfg(windows)]
    #[test]
    fn test_l_instant_present_se_lit_comme_l_horloge_du_systeme() {
        use windows::Win32::System::SystemInformation::GetLocalTime;
        let maintenant = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("apres 1970")
            .as_millis() as i64;
        // SAFETY : l'appel ne fait que rendre une structure par valeur.
        let horloge = unsafe { GetLocalTime() };
        let converti = heure_locale(maintenant).expect("Windows connaît son fuseau");
        let minutes = |h: u16, m: u16| i32::from(h) * 60 + i32::from(m);
        let ecart = minutes(u16::from(converti.heure), u16::from(converti.minute))
            - minutes(horloge.wHour, horloge.wMinute);
        assert!(
            ecart.abs() <= 1 || ecart.abs() >= 1439,
            "la conversion dit {converti}, l'horloge {:02}:{:02}",
            horloge.wHour,
            horloge.wMinute
        );
        assert_eq!(
            (
                converti.annee,
                u16::from(converti.mois),
                u16::from(converti.jour)
            ),
            (horloge.wYear, horloge.wMonth, horloge.wDay),
            "{converti}"
        );
    }
}
