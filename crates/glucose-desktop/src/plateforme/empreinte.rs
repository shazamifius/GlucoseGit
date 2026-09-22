//! **EMPREINTE-1** : ce que Glucose coûte à la machine quand personne ne le regarde.
//!
//! # Rien n'en était mesuré, et c'est une promesse du projet
//!
//! *« L'idée c'est que ce soit un logiciel ultra rapide et économe. »* Cinq sessions ont
//! mesuré ce qu'une **image** coûte, au centile, poste par poste. Aucune n'a jamais mesuré ce
//! que le **processus** coûte : combien de mémoire il occupe, et combien de processeur il
//! consomme pendant qu'on ne le touche pas.
//!
//! Les deux chiffres sont ceux qu'un utilisateur juge sans les nommer. Un logiciel qui prend
//! huit cents mébioctets à l'ouverture d'un document vide n'est pas économe, quelle que soit
//! sa cadence ; un logiciel qui occupe un dixième de cœur pendant qu'il dort empêche un
//! portable de se rendormir, et c'est de l'autonomie en moins.
//!
//! # Deux grandeurs, et on n'interroge que le système
//!
//! * **La mémoire de travail** — ce que le système compte comme réellement présent en RAM
//!   pour ce processus. C'est le nombre que montre le gestionnaire des tâches, donc celui que
//!   l'utilisateur verrait s'il regardait.
//! * **Le temps processeur consommé**, noyau et utilisateur ensemble, cumulé depuis le
//!   lancement. Sa **différence** entre deux relevés, divisée par le temps mural écoulé, donne
//!   la part d'un cœur — 1,00 veut dire un cœur entier, 0,00 veut dire endormi.
//!
//! Ce ne sont pas des mesures à nous : on demande au système ce qu'il a compté. C'est la seule
//! façon honnête de répondre, et c'est cohérent avec la règle de la charte — on n'interroge pas
//! le matériel sur ce qu'il *pourrait* faire, on lit ce qu'il a *fait*.
//!
//! # Ce qu'une plateforme sans relevé donne
//!
//! Rien, et le rapport ne dit alors rien plutôt que zéro — *« un compteur déclaré et jamais
//! lu vaut zéro, et un zéro se lit comme une mesure »* (fiche 17 § 3.1, cliquet 9). Windows et
//! Linux répondent ; macOS demanderait `task_info`, et tant qu'il n'est pas écrit, la section
//! ne paraît pas.

/// Ce que le système compte pour ce processus, à un instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Empreinte {
    /// La mémoire réellement présente en RAM pour ce processus, en octets.
    pub memoire_octets: u64,
    /// Le temps processeur consommé depuis le lancement, noyau et utilisateur, en
    /// microsecondes.
    ///
    /// **Cumulé**, donc seule sa différence entre deux relevés veut dire quelque chose. Le
    /// garder cumulé plutôt que de calculer une part ici est délibéré : c'est l'appelant qui
    /// sait entre quels instants il veut lire, et une part calculée trop tôt ne se recompose
    /// plus.
    pub processeur_us: u64,
}

/// **Ce que le système compte en ce moment**, ou rien sur une plateforme qui ne sait pas le
/// dire.
pub fn relever() -> Option<Empreinte> {
    natif::relever()
}

#[cfg(windows)]
mod natif {
    use super::Empreinte;
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    /// Un `FILETIME` compte des intervalles de cent nanosecondes.
    const PAR_MICROSECONDE: u64 = 10;

    pub fn relever() -> Option<Empreinte> {
        // Le pseudo-handle du processus courant : il ne s'ouvre ni ne se ferme, donc rien ne
        // peut fuir ici.
        let moi = unsafe { GetCurrentProcess() };
        let mut compteurs = PROCESS_MEMORY_COUNTERS::default();
        let taille = u32::try_from(std::mem::size_of::<PROCESS_MEMORY_COUNTERS>()).ok()?;
        unsafe { GetProcessMemoryInfo(moi, &mut compteurs, taille) }.ok()?;

        let (mut creation, mut fin, mut noyau, mut utilisateur) = Default::default();
        unsafe {
            GetProcessTimes(moi, &mut creation, &mut fin, &mut noyau, &mut utilisateur).ok()?;
        }
        Some(Empreinte {
            memoire_octets: compteurs.WorkingSetSize as u64,
            processeur_us: (cent_ns(noyau) + cent_ns(utilisateur)) / PAR_MICROSECONDE,
        })
    }

    /// Les deux moitiés d'un `FILETIME`, recomposées.
    ///
    /// `pub(super)` pour une seule raison : c'est **ici** que se cache la faute qui produit un
    /// nombre plausible et faux d'un facteur quatre milliards, et une fonction pure se prouve
    /// sans ordonnanceur. La prouver par une mesure de charge la rendrait dépendante de ce que
    /// font les autres fils du processus — ce que `GetProcessTimes` compte aussi.
    pub(super) fn cent_ns(t: windows::Win32::Foundation::FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }
}

#[cfg(target_os = "linux")]
mod natif {
    use super::Empreinte;

    /// Le nombre de tics d'horloge par seconde que `/proc` emploie, et il vaut cent partout où
    /// Linux tourne sur une machine de bureau.
    ///
    /// Le lire proprement demanderait `sysconf(_SC_CLK_TCK)`, donc `libc`, donc une caisse de
    /// plus pour une constante que le noyau n'a pas changée depuis vingt ans. Si elle devait
    /// différer, la part de processeur serait fausse d'un facteur entier — visible tout de
    /// suite, et jamais silencieuse.
    const TICS_PAR_SECONDE: u64 = 100;

    pub fn relever() -> Option<Empreinte> {
        let pages = std::fs::read_to_string("/proc/self/statm").ok()?;
        // La deuxième colonne est la résidence, en pages.
        let resident: u64 = pages.split_whitespace().nth(1)?.parse().ok()?;

        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        // Le nom du processus peut contenir des espaces et des parenthèses : les champs se
        // comptent donc APRÈS la dernière parenthèse fermante, jamais depuis le début.
        let apres = &stat[stat.rfind(')')? + 1..];
        let champs: Vec<&str> = apres.split_whitespace().collect();
        // `utime` et `stime` sont les quatorzième et quinzième champs de `stat` ; le découpage
        // commence au troisième, donc ils sont ici aux rangs onze et douze.
        let utime: u64 = champs.get(11)?.parse().ok()?;
        let stime: u64 = champs.get(12)?.parse().ok()?;
        Some(Empreinte {
            memoire_octets: resident.saturating_mul(4096),
            processeur_us: (utime + stime).saturating_mul(1_000_000) / TICS_PAR_SECONDE,
        })
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod natif {
    use super::Empreinte;

    /// Aucun relevé ici, et le rapport ne dira donc rien — plutôt que zéro.
    pub fn relever() -> Option<Empreinte> {
        None
    }
}

#[cfg(test)]
mod tests;
