//! **Ce que le système a noté des plantages et des gels** (fiche 49) : sous Windows, le journal
//! Application — que tout utilisateur interactif peut lire —, où le rapporteur d'erreurs écrit
//! « Application Error » (1000) pour un plantage et « Application Hang » (1002) pour un gel
//! qu'il a fermé.
//!
//! Ce module ne juge de rien : il rend ce que le système a écrit. La boîte noire reconnaît
//! parmi ces événements celui de sa session ([`crate::boite_noire::plantage`]). Ailleurs qu'à
//! Windows, il ne rend rien encore : une session qui s'y arrête reste « arrêtée sans rien dire ».

/// Un plantage ou un gel, tel que le système l'a noté.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evenement {
    /// Un gel (1002) plutôt qu'un plantage (1000).
    pub gel: bool,
    pub processus: u32,
    /// La création du processus, en millisecondes depuis 1970.
    pub creation_ms: u64,
    /// L'instant où le système a noté l'événement, en millisecondes depuis 1970.
    pub instant_ms: u64,
    pub module: Option<String>,
    pub version_du_module: Option<String>,
    /// Le code de l'exception, en hexadécimal, tel qu'écrit.
    pub code: Option<String>,
    /// Le décalage dans le module, en hexadécimal, tel qu'écrit.
    pub decalage: Option<String>,
}

/// **Les plantages et les gels que le système a notés depuis `depuis_ms`** (millisecondes
/// depuis 1970), de tous les programmes : c'est à qui lit de reconnaître le sien.
pub fn plantages_depuis(depuis_ms: u64) -> Vec<Evenement> {
    #[cfg(windows)]
    {
        imp::lire(depuis_ms).unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        let _ = depuis_ms;
        Vec::new()
    }
}

/// Un instant de Windows (`FILETIME` : des centaines de nanosecondes depuis 1601), en
/// millisecondes depuis 1970.
pub fn filetime_en_ms(filetime: u64) -> u64 {
    /// De 1601 à 1970, en millisecondes.
    const DE_1601_A_1970_MS: u64 = 11_644_473_600_000;
    (filetime / 10_000).saturating_sub(DE_1601_A_1970_MS)
}

#[cfg(windows)]
mod imp {
    use super::{filetime_en_ms, Evenement};
    use windows::core::{w, HSTRING, PCWSTR};
    use windows::Win32::System::EventLog::{
        EvtClose, EvtCreateRenderContext, EvtNext, EvtQuery, EvtQueryChannelPath,
        EvtQueryReverseDirection, EvtRender, EvtRenderContextValues, EvtRenderEventValues,
        EvtVarTypeFileTime, EvtVarTypeHexInt32, EvtVarTypeHexInt64, EvtVarTypeString,
        EvtVarTypeUInt16, EvtVarTypeUInt32, EvtVarTypeUInt64, EVT_HANDLE, EVT_VARIANT,
        EVT_VARIANT_TYPE, EVT_VARIANT_TYPE_MASK,
    };

    /// Ce qu'on lit de chaque événement, dans cet ordre — les noms des gabarits de Windows
    /// (`Get-WinEvent -ListProvider 'Application Error'`, `'Application Hang'`).
    const CHEMINS: [PCWSTR; 9] = [
        w!("Event/System/EventID"),
        w!("Event/System/TimeCreated/@SystemTime"),
        w!("Event/EventData/Data[@Name='ProcessId']"),
        // La création du processus : `ProcessCreationTime` pour un plantage, `StartTime` pour
        // un gel.
        w!("Event/EventData/Data[@Name='ProcessCreationTime']"),
        w!("Event/EventData/Data[@Name='StartTime']"),
        w!("Event/EventData/Data[@Name='ModuleName']"),
        w!("Event/EventData/Data[@Name='ModuleVersion']"),
        w!("Event/EventData/Data[@Name='ExceptionCode']"),
        w!("Event/EventData/Data[@Name='FaultingOffset']"),
    ];

    /// Une poignée du journal, fermée quoi qu'il arrive.
    struct Poignee(EVT_HANDLE);

    impl Drop for Poignee {
        fn drop(&mut self) {
            // SAFETY : une poignée rendue par le journal, fermée une seule fois.
            let _ = unsafe { EvtClose(self.0) };
        }
    }

    pub fn lire(depuis_ms: u64) -> windows::core::Result<Vec<Evenement>> {
        let ecart = crate::boite_noire::sondes::maintenant_ms().saturating_sub(depuis_ms);
        let requete = format!(
            "*[System[((Provider[@Name='Application Error'] and EventID=1000) or \
             (Provider[@Name='Application Hang'] and EventID=1002)) and \
             TimeCreated[timediff(@SystemTime) <= {ecart}]]]"
        );
        let drapeaux = EvtQueryChannelPath.0 | EvtQueryReverseDirection.0;
        // SAFETY : des chaînes terminées par un zéro, que le journal copie.
        let resultats = Poignee(unsafe {
            EvtQuery(None, w!("Application"), &HSTRING::from(requete), drapeaux)
        }?);
        // SAFETY : un tableau de chemins valides, lu pendant l'appel.
        let contexte =
            Poignee(unsafe { EvtCreateRenderContext(Some(&CHEMINS), EvtRenderContextValues.0) }?);
        let mut lus = Vec::new();
        loop {
            let mut lot = [0isize; 16];
            let mut rendus = 0u32;
            // SAFETY : `lot` reçoit au plus sa longueur de poignées ; la fin des résultats se
            // dit par une erreur (`ERROR_NO_MORE_ITEMS`).
            if unsafe { EvtNext(resultats.0, &mut lot, u32::MAX, 0, &mut rendus) }.is_err() {
                break;
            }
            for &h in &lot[..rendus as usize] {
                let evenement = Poignee(EVT_HANDLE(h));
                lus.extend(rendre(&contexte, &evenement));
            }
        }
        Ok(lus)
    }

    /// Les valeurs d'un événement, dans l'ordre de [`CHEMINS`].
    fn rendre(contexte: &Poignee, evenement: &Poignee) -> Option<Evenement> {
        let (mut taille, mut nombre) = (0u32, 0u32);
        // Le premier appel dit la place qu'il faut (`ERROR_INSUFFICIENT_BUFFER`).
        // SAFETY : sans tampon, l'appel n'écrit que les deux tailles.
        let _ = unsafe {
            EvtRender(
                Some(contexte.0),
                evenement.0,
                EvtRenderEventValues.0,
                0,
                None,
                &mut taille,
                &mut nombre,
            )
        };
        // Des mots de huit octets : l'alignement d'`EVT_VARIANT`. Les textes que les valeurs
        // désignent vivent dans ce même tampon, après elles.
        let mut tampon = vec![0u64; (taille as usize).div_ceil(8)];
        // SAFETY : un tampon de `taille` octets au moins, que l'appel remplit.
        unsafe {
            EvtRender(
                Some(contexte.0),
                evenement.0,
                EvtRenderEventValues.0,
                taille,
                Some(tampon.as_mut_ptr().cast()),
                &mut taille,
                &mut nombre,
            )
        }
        .ok()?;
        if (nombre as usize) < CHEMINS.len() {
            return None;
        }
        // SAFETY : l'appel a écrit `nombre` valeurs au début du tampon, qui vit jusqu'à la fin
        // de cette fonction — et avec lui les textes qu'elles désignent.
        let v = unsafe { std::slice::from_raw_parts(tampon.as_ptr().cast::<EVT_VARIANT>(), 9) };
        let gel = entier(&v[0])? == 1002;
        Some(Evenement {
            gel,
            instant_ms: filetime_en_ms(entier(&v[1])?),
            processus: u32::try_from(entier(&v[2])?).ok()?,
            creation_ms: filetime_en_ms(entier(if gel { &v[4] } else { &v[3] })?),
            module: texte(&v[5]),
            version_du_module: texte(&v[6]),
            code: texte(&v[7]),
            decalage: texte(&v[8]),
        })
    }

    fn sorte(v: &EVT_VARIANT) -> EVT_VARIANT_TYPE {
        EVT_VARIANT_TYPE((v.Type & EVT_VARIANT_TYPE_MASK) as i32)
    }

    /// Un nombre, quelle que soit la forme que le gabarit lui donne.
    fn entier(v: &EVT_VARIANT) -> Option<u64> {
        let s = sorte(v);
        // SAFETY : on ne lit du champ que la forme que `Type` annonce.
        unsafe {
            if s == EvtVarTypeUInt16 {
                Some(u64::from(v.Anonymous.UInt16Val))
            } else if s == EvtVarTypeUInt32 || s == EvtVarTypeHexInt32 {
                Some(u64::from(v.Anonymous.UInt32Val))
            } else if s == EvtVarTypeUInt64 || s == EvtVarTypeHexInt64 {
                Some(v.Anonymous.UInt64Val)
            } else if s == EvtVarTypeFileTime {
                Some(v.Anonymous.FileTimeVal)
            } else {
                None
            }
        }
    }

    /// Un texte, s'il y en a un.
    fn texte(v: &EVT_VARIANT) -> Option<String> {
        if sorte(v) != EvtVarTypeString {
            return None;
        }
        // SAFETY : un texte terminé par un zéro, dans le tampon qui vit encore.
        let t = unsafe { v.Anonymous.StringVal.to_string() }.ok()?;
        (!t.is_empty()).then_some(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Un instant de Windows devient un instant de 1970** — lu sur un vrai événement : un
    /// processus créé à `0x01dd4ccd3c6eedb7`, le 25/09/2026 à 09 h 07 min 05,689 s (UTC).
    #[test]
    fn test_un_instant_de_windows_se_lit_depuis_1970() {
        assert_eq!(filetime_en_ms(0x01dd_4ccd_3c6e_edb7), 1_790_327_225_689);
        assert_eq!(
            filetime_en_ms(0),
            0,
            "avant 1970 : rien, pas un nombre qui déborde"
        );
    }

    /// **Le journal se lit** sans erreur, sur n'importe quelle machine : chaque événement
    /// rendu est un plantage ou un gel, noté avant maintenant, d'un processus créé avant lui.
    /// (Seule, cette épreuve passerait avec un lecteur qui ne rend rien : c'est la suivante qui
    /// prouve qu'il lit.)
    #[test]
    fn test_le_journal_se_lit() {
        let maintenant = crate::boite_noire::sondes::maintenant_ms();
        let semaine = 7 * 24 * 3600 * 1000;
        for e in plantages_depuis(maintenant.saturating_sub(semaine)) {
            assert!(e.instant_ms <= maintenant, "{e:?}");
            assert!(e.creation_ms <= e.instant_ms, "{e:?}");
        }
    }

    /// Le code de l'exception que l'épreuve lève : « GLU », parmi les codes que Windows laisse
    /// aux applications (`0xE…`) — aucun autre programme ne tombe sur celui-ci.
    #[cfg(windows)]
    const CODE_D_EPREUVE: u32 = 0xE047_4C55;

    /// **Un vrai plantage se lit dans le journal.** Un processus d'épreuve lève une exception
    /// que personne n'attrape — le rapporteur d'erreurs de Windows en silence, sans fenêtre —,
    /// et le lecteur retrouve, parmi ce que Windows a noté, **ce** processus et **ce** code ; la
    /// boîte noire le reconnaît pour une session qu'il aurait ouverte.
    ///
    /// Sur une machine jetable de GitHub seulement : un plantage fait écrire à Windows un
    /// rapport dans ses propres dossiers.
    #[cfg(windows)]
    #[test]
    fn test_un_vrai_plantage_se_lit_dans_le_journal() {
        let jetable = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
            && std::env::var("RUNNER_ENVIRONMENT").as_deref() == Ok("github-hosted");
        if !jetable {
            eprintln!("sautée : elle ne fait tomber un processus que sur une machine de GitHub");
            return;
        }
        let depuis = crate::boite_noire::sondes::maintenant_ms();
        let mut enfant = std::process::Command::new(std::env::current_exe().expect("l'épreuve"))
            .args(["--exact", "plateforme::journal::tests::planter_ici"])
            .args(["--ignored", "--test-threads=1"])
            .env("GLUCOSE_PLANTER_ICI", "1")
            .spawn()
            .expect("le processus qui va tomber");
        let processus = enfant.id();
        // Deux minutes ne sont que la patience de l'épreuve, jamais une règle du lecteur : si
        // une fenêtre du rapporteur retenait le processus, l'épreuve tombe au lieu d'attendre.
        let limite = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let statut = loop {
            if let Some(s) = enfant.try_wait().expect("son état") {
                break s;
            }
            if std::time::Instant::now() >= limite {
                let _ = enfant.kill();
                panic!("le processus {processus} n'est pas tombé : quelque chose le retient");
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        };
        assert_eq!(
            statut.code().map(|c| c as u32),
            Some(CODE_D_EPREUVE),
            "il est tombé sur l'exception levée"
        );
        // Le rapporteur note l'événement après la chute : on relit jusqu'à le trouver.
        let vu = loop {
            let lus = plantages_depuis(depuis);
            if let Some(e) = lus.into_iter().find(|e| e.processus == processus) {
                break e;
            }
            assert!(
                std::time::Instant::now() < limite,
                "Windows n'a rien noté du processus {processus}"
            );
            std::thread::sleep(std::time::Duration::from_millis(500));
        };
        assert!(!vu.gel, "{vu:?}");
        assert!(
            vu.creation_ms <= vu.instant_ms && depuis <= vu.instant_ms,
            "{vu:?}"
        );
        let p = crate::boite_noire::plantage::reconnaitre(
            std::slice::from_ref(&vu),
            processus,
            vu.creation_ms,
        )
        .expect("la boîte noire le reconnaît");
        assert_eq!(p.code, Some(CODE_D_EPREUVE), "{vu:?}");
        assert!(p.module.is_some(), "le module où il est tombé : {vu:?}");
    }

    /// Le processus que l'épreuve précédente fait tomber — lancé par elle seule.
    #[cfg(windows)]
    #[test]
    #[ignore = "lancé par test_un_vrai_plantage_se_lit_dans_le_journal"]
    fn planter_ici() {
        use windows::Win32::System::Diagnostics::Debug::RaiseException;
        use windows::Win32::System::ErrorReporting::{
            WerSetFlags, WER_FAULT_REPORTING, WER_FAULT_REPORTING_FLAG_QUEUE,
            WER_FAULT_REPORTING_NO_UI,
        };
        if std::env::var("GLUCOSE_PLANTER_ICI").as_deref() != Ok("1") {
            return;
        }
        /// `EXCEPTION_NONCONTINUABLE` : une exception dont on ne revient pas.
        const SANS_RETOUR: u32 = 1;
        // Le rapport en file, sans fenêtre (`werapi.h` : `WER_FAULT_REPORTING_NO_UI`) : rien ne
        // s'affiche, rien ne retient le processus.
        let sans_fenetre =
            WER_FAULT_REPORTING(WER_FAULT_REPORTING_FLAG_QUEUE.0 | WER_FAULT_REPORTING_NO_UI);
        // SAFETY : régler le rapporteur de ce processus, puis lever une exception que personne
        // n'attrape — le processus tombe, c'est ce qu'on veut.
        unsafe {
            let _ = WerSetFlags(sans_fenetre);
            RaiseException(CODE_D_EPREUVE, SANS_RETOUR, None);
        }
    }
}
