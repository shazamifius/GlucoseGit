//! Ce que la machine a de mémoire, et ce qu'il en reste (ADAPT-1).
//!
//! # Pourquoi ce module existe
//!
//! Une borne de cache écrite en dur est fausse deux fois : ridicule sur une machine à trente-
//! deux gigaoctets, mortelle sur un téléphone à deux. La seule borne juste se lit sur la
//! machine, et elle se relit — parce qu'une autre application peut réclamer la mémoire pendant
//! qu'on tourne.
//!
//! # Pourquoi sans dépendance
//!
//! Trois appels système, trois plateformes, une soixantaine de lignes. Une bibliothèque
//! d'introspection en apporterait une vingtaine de crates pour la même chose, et nous ferait
//! dépendre d'elle pour un renseignement que le système donne directement.
//!
//! Chaque plateforme est isolée dans son propre `mod`, et toutes rendent le même type. Ajouter
//! une plateforme n'oblige à toucher à rien d'autre.
//!
//! # Ce que ce module promet quand il ne sait pas
//!
//! Il rend `None`, et ne devine pas. Un appelant qui ne sait pas combien il y a de mémoire doit
//! se comporter prudemment — pas se fier à un chiffre inventé ici.

/// Ce que le système dit de sa mémoire, en octets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memoire {
    /// La mémoire physique totale de la machine.
    pub totale: u64,
    /// Ce qui peut être alloué **maintenant** sans faire pagner le système.
    ///
    /// C'est cette valeur-là qui compte, et non la totale : un navigateur ouvert change le
    /// second chiffre sans toucher au premier.
    pub disponible: u64,
}

impl Memoire {
    /// Ce que le système déclare, ou `None` s'il ne le dit pas.
    ///
    /// Relire à chaque fois est délibéré, et c'est tout l'intérêt : la valeur bouge, et une
    /// borne qui ne la relit pas cesse d'être adaptative à la première autre application
    /// lancée. Le coût est un appel système ou la lecture d'un fichier virtuel — de l'ordre de
    /// la microseconde, à ne pas faire par pixel mais sans aucun problème par image.
    pub fn du_systeme() -> Option<Self> {
        plateforme::lire()
    }

    /// Ce qu'un cache a le droit de garder : **la moitié de ce qui est disponible**.
    ///
    /// # Le facteur n'est pas choisi, c'est un point fixe
    ///
    /// La règle énoncée est « un cache ne prend jamais plus que ce qu'il laisse ». Elle n'a
    /// qu'une solution, et c'est un demi. Ce n'est donc pas un réglage qu'on pourrait vouloir
    /// ajuster : c'est la traduction exacte de la phrase.
    ///
    /// Trois propriétés en découlent sans rien ajouter : sur trente-deux gigaoctets libres on
    /// peut en prendre seize, sur un gigaoctet libre on se contente de cinq cents mégaoctets,
    /// et **la borne se contracte d'elle-même** quand une autre application réclame — puisque
    /// le disponible baisse et qu'on le relit.
    pub fn part_pour_un_cache(&self) -> u64 {
        self.disponible / 2
    }
}

/// **Ce que le système accorde à Glucose sur la carte graphique, et ce qu'il y occupe**, en
/// octets (VRAM-1).
///
/// Le budget n'est pas la taille de la carte : c'est la part que le système réserve à ce
/// processus **maintenant**, et elle rétrécit quand une autre application en réclame. Il se lit
/// par [`crate::plateforme::graphique::Sonde`] ; ce type ne porte que la règle, qui se teste
/// sans carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoireGraphique {
    pub budget: u64,
    pub utilisee: u64,
}

impl MemoireGraphique {
    /// Ce qu'un cache de textures a le droit de garder, sachant qu'il en garde déjà `en_cache`.
    ///
    /// La règle de la mémoire vive, et pour la même raison : un cache ne prend jamais plus que
    /// ce qu'il laisse — la moitié de ce que le budget laisserait **sans lui**. Quand une autre
    /// application prend de la mémoire graphique, le budget baisse, et le cache rend ce qu'il
    /// gardait, les plus anciennes d'abord.
    pub fn part_pour_un_cache(&self, en_cache: u64) -> u64 {
        self.budget
            .saturating_sub(self.utilisee.saturating_sub(en_cache))
            / 2
    }
}

#[cfg(target_os = "windows")]
mod plateforme {
    use super::Memoire;

    /// La structure que `GlobalMemoryStatusEx` remplit.
    ///
    /// Déclarée ici plutôt qu'importée d'une bibliothèque de liaisons : elle est figée depuis
    /// Windows 2000, et la recopier coûte douze lignes contre une vingtaine de crates.
    #[repr(C)]
    #[derive(Default)]
    struct MemoryStatusEx {
        longueur: u32,
        charge: u32,
        total_physique: u64,
        dispo_physique: u64,
        total_fichier_page: u64,
        dispo_fichier_page: u64,
        total_virtuel: u64,
        dispo_virtuel: u64,
        dispo_virtuel_etendu: u64,
    }

    unsafe extern "system" {
        fn GlobalMemoryStatusEx(etat: *mut MemoryStatusEx) -> i32;
    }

    pub(super) fn lire() -> Option<Memoire> {
        let mut etat = MemoryStatusEx {
            longueur: u32::try_from(size_of::<MemoryStatusEx>()).ok()?,
            ..Default::default()
        };
        // Sûr : la structure est locale, correctement dimensionnée par son propre champ
        // `longueur`, et l'appel ne fait qu'y écrire des entiers.
        let ok = unsafe { GlobalMemoryStatusEx(&mut etat) } != 0;
        if !ok || etat.total_physique == 0 {
            return None;
        }
        Some(Memoire {
            totale: etat.total_physique,
            disponible: etat.dispo_physique,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod plateforme {
    use super::Memoire;

    /// `/proc/meminfo`, qui existe sur Linux comme sur Android.
    ///
    /// `MemAvailable` est préféré à `MemFree` et c'est important : `MemFree` ignore le cache de
    /// fichiers, que le noyau rendra sans broncher. S'y fier ferait croire la machine pleine
    /// alors qu'elle ne l'est pas — et Glucose se priverait sur une machine qui a de la place.
    pub(super) fn lire() -> Option<Memoire> {
        let texte = std::fs::read_to_string("/proc/meminfo").ok()?;
        let champ = |nom: &str| -> Option<u64> {
            texte
                .lines()
                .find(|l| l.starts_with(nom))?
                .split_whitespace()
                .nth(1)?
                .parse::<u64>()
                .ok()
                .map(|kio| kio * 1024)
        };
        let totale = champ("MemTotal:")?;
        let disponible = champ("MemAvailable:").or_else(|| champ("MemFree:"))?;
        Some(Memoire { totale, disponible })
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod plateforme {
    use super::Memoire;

    unsafe extern "C" {
        fn sysctlbyname(
            nom: *const std::ffi::c_char,
            valeur: *mut std::ffi::c_void,
            taille: *mut usize,
            neuf: *mut std::ffi::c_void,
            taille_neuf: usize,
        ) -> i32;
    }

    fn entier(nom: &std::ffi::CStr) -> Option<u64> {
        let mut valeur: u64 = 0;
        let mut taille = size_of::<u64>();
        // Sûr : `valeur` et `taille` sont locales et correctement appariées ; l'appel n'écrit
        // qu'au plus `taille` octets dans `valeur`.
        let code = unsafe {
            sysctlbyname(
                nom.as_ptr(),
                (&raw mut valeur).cast(),
                &raw mut taille,
                std::ptr::null_mut(),
                0,
            )
        };
        (code == 0 && valeur != 0).then_some(valeur)
    }

    /// # Ce que cette plateforme ne sait pas encore dire, et pourquoi c'est dit
    ///
    /// `hw.memsize` donne la mémoire totale, exactement. Le **disponible**, lui, se lit par
    /// `host_statistics64`, qui demande le port Mach de la machine — nettement plus de code.
    ///
    /// En attendant, on rend la totale et on la déclare comme disponible : la borne du cache
    /// vaudra donc la moitié de la mémoire physique. C'est plus généreux que la vérité, et
    /// c'est un compromis à nommer plutôt qu'à cacher — sur macOS, un cache d'images peut
    /// pousser à la pagination là où les autres plateformes se contracteraient.
    pub(super) fn lire() -> Option<Memoire> {
        let totale = entier(c"hw.memsize")?;
        Some(Memoire {
            totale,
            disponible: totale,
        })
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
)))]
mod plateforme {
    use super::Memoire;

    /// Une plateforme dont on ne sait rien ne se devine pas : l'appelant se comportera
    /// prudemment, ce qui vaut mieux qu'un chiffre inventé ici.
    pub(super) fn lire() -> Option<Memoire> {
        None
    }
}

#[cfg(test)]
mod tests;
