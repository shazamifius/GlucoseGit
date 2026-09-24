//! **Une mémoire que le système peut reprendre** quand une autre application en a besoin
//! (ETAGES-1, fiche 32).
//!
//! # Le renseignement qu'aucun relevé ne donne à temps
//!
//! Une image décodée en mémoire vive est une **réserve** : la carte graphique la détient
//! déjà, ou l'écran ne la montre pas. La garder coûte de la place à tout le monde ; la rendre
//! oblige à la redécoder — seize millisecondes par mégapixel, mesurées sur les 243 photos de
//! l'utilisateur. Le bon moment pour la rendre est celui où **une autre application** a besoin
//! de la place, et ce moment-là, seul le système le connaît.
//!
//! Relever la mémoire libre à chaque image ne suffit pas : au repos, Glucose ne dessine rien,
//! donc ne relève rien. Un Blender qui se met à rendre pendant ce temps ferait écrire nos
//! images sur le disque de pagination, puis relire page à page au prochain regard.
//!
//! Windows sait faire mieux, et le dit : `OfferVirtualMemory` laisse les pages où elles sont,
//! les retire de la mémoire de travail, et permet au système de les **jeter** — sans les
//! écrire nulle part — quand il manque de place. `ReclaimVirtualMemory` dit ensuite si elles
//! sont intactes. Aucun seuil n'est choisi, aucune horloge ne tourne : le système décide, au
//! moment exact où il en a besoin, et seulement alors.
//!
//! # Ce que coûtent les deux gestes, mesuré (`bench_offre`)
//!
//! ```text
//!     offrir 10 Mo          3,0 ms
//!     reprendre 10 Mo       4,6 ms    intacts, 24 sur 24
//!     mémoire de travail    244 Mo -> 4 Mo une fois offerts
//! ```
//!
//! Trop cher pour le fil qui dessine : ces gestes appartiennent aux ouvriers de l'atelier,
//! comme les décodages. Et nettement moins cher que redescendre au disque — une image reprise
//! coûte environ 2,4 ms par mégapixel, une image redécodée seize.
//!
//! # Ce que le type garantit
//!
//! Une région **offerte** ne se lit pas : ses pages sont inaccessibles, et y toucher ferait
//! tomber le processus. Le type le dit au compilateur — seule une [`Tenue`] prête ses octets,
//! et on n'obtient une `Tenue` d'une [`Offerte`] qu'en la reprenant.
//!
//! # Ailleurs
//!
//! Les autres systèmes ont leur geste (`vm_purgable_control` sur macOS, `ashmem` sur Android).
//! Tant qu'il n'est pas écrit, une région y est un tampon ordinaire : offrir n'y rend rien, et
//! reprendre la retrouve toujours intacte — le comportement d'avant, pas une régression.

/// Une région dont on peut lire et écrire les octets.
pub struct Tenue(imp::Bloc);

/// Une région offerte au système : il peut la jeter tant qu'on ne l'a pas reprise.
pub struct Offerte {
    bloc: imp::Bloc,
    /// Faux si le système a refusé l'offre : les pages sont alors restées, et la reprise n'a
    /// rien à lui demander.
    offerte: bool,
}

impl Tenue {
    /// Une région neuve de `octets` octets, mis à zéro par le système, ou `None` s'il la
    /// refuse.
    pub fn nouvelle(octets: usize) -> Option<Self> {
        imp::Bloc::nouveau(octets).map(Self)
    }

    /// Ses octets, exactement ceux demandés.
    pub fn octets(&self) -> &[u8] {
        self.0.octets()
    }

    /// Ses octets, pour les écrire.
    pub fn octets_mut(&mut self) -> &mut [u8] {
        self.0.octets_mut()
    }

    /// Le système peut-il la reprendre ? Non pour moins d'une page, qui vit dans le tas.
    pub fn offrable(&self) -> bool {
        self.0.offrable()
    }

    /// **L'offre au système**, qui pourra la jeter s'il manque de place.
    pub fn offrir(mut self) -> Offerte {
        let offerte = self.0.offrir();
        Offerte {
            bloc: self.0,
            offerte,
        }
    }
}

impl Offerte {
    /// **La reprend**, intacte — ou `None` si le système l'a jetée entre-temps, auquel cas la
    /// mémoire lui est rendue et le contenu est à refaire.
    pub fn reprendre(mut self) -> Option<Tenue> {
        (!self.offerte || self.bloc.reprendre()).then_some(Tenue(self.bloc))
    }

    /// Combien d'octets elle porte.
    pub fn longueur(&self) -> usize {
        self.bloc.longueur()
    }

    /// Où elle vit, pour qu'une épreuve demande au système si ses pages sont présentes.
    #[cfg(test)]
    pub(crate) fn adresse(&self) -> (*const u8, usize) {
        self.bloc.adresse()
    }
}

#[cfg(windows)]
mod imp {
    use windows::Win32::System::Memory::{
        OfferVirtualMemory, ReclaimVirtualMemory, VirtualAlloc, VirtualFree, VirtualQuery,
        VmOfferPriorityNormal, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_READWRITE,
    };

    /// Où vivent les octets d'une région.
    pub enum Bloc {
        /// Des pages obtenues du système : elles seules peuvent être offertes.
        Pages(Pages),
        /// Moins d'une page : le système n'offre que des pages entières, et en donner une
        /// entière à quelques octets en gaspillerait le reste. Elle vit dans le tas, et
        /// l'offre n'y rend rien — la reprise la retrouve donc toujours.
        Tas(Vec<u8>),
    }

    pub struct Pages {
        debut: std::ptr::NonNull<u8>,
        /// Ce que l'appelant a demandé.
        longueur: usize,
        /// Ce que le système a engagé : la longueur arrondie à ses pages, lue et non supposée.
        /// C'est sur elle que l'offre et la reprise portent, puisqu'elles veulent des pages
        /// entières.
        engagee: usize,
    }

    // Sûr : les pages appartiennent à ce seul bloc, comme les octets d'un `Vec` ; le déplacer
    // vers un autre fil ne partage rien.
    unsafe impl Send for Pages {}

    /// La taille d'une page de ce système, lue une fois.
    fn page() -> usize {
        static PAGE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *PAGE.get_or_init(|| {
            let mut info = windows::Win32::System::SystemInformation::SYSTEM_INFO::default();
            // Sûr : `info` est local et du type que l'appel remplit.
            unsafe { windows::Win32::System::SystemInformation::GetSystemInfo(&mut info) };
            (info.dwPageSize as usize).max(1)
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
                // Sûr : les pages sont engagées pour `longueur` octets et vivent autant que le
                // bloc ; seule une `Tenue` y donne accès, jamais une région offerte.
                Self::Pages(p) => unsafe {
                    std::slice::from_raw_parts(p.debut.as_ptr(), p.longueur)
                },
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
                // Sûr : des pages entières, engagées par ce bloc.
                Self::Pages(p) => unsafe {
                    OfferVirtualMemory(p.toutes(), VmOfferPriorityNormal) == 0
                },
                Self::Tas(_) => false,
            }
        }

        pub fn reprendre(&mut self) -> bool {
            match self {
                // Sûr : les pages que `offrir` a offertes. Tout autre code que le succès — la
                // mémoire jetée, ou un refus — se lit comme un contenu perdu : le refaire est
                // toujours juste, s'en servir ne le serait pas.
                Self::Pages(p) => unsafe { ReclaimVirtualMemory(p.toutes()) == 0 },
                Self::Tas(_) => true,
            }
        }

        #[cfg(test)]
        pub fn adresse(&self) -> (*const u8, usize) {
            match self {
                Self::Pages(p) => (p.debut.as_ptr().cast_const(), p.engagee),
                Self::Tas(v) => (v.as_ptr(), v.len()),
            }
        }
    }

    impl Pages {
        fn nouvelles(longueur: usize) -> Option<Self> {
            // Sûr : une allocation neuve, rendue dans `drop`.
            let p = unsafe { VirtualAlloc(None, longueur, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
            let debut = std::ptr::NonNull::new(p.cast::<u8>())?;
            let mut info = MEMORY_BASIC_INFORMATION::default();
            // Sûr : `info` est local et du type que l'appel remplit.
            let lu = unsafe {
                VirtualQuery(
                    Some(p.cast_const()),
                    &mut info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            let engagee = if lu == 0 { longueur } else { info.RegionSize.max(longueur) };
            Some(Self {
                debut,
                longueur,
                engagee,
            })
        }

        fn toutes(&mut self) -> &mut [u8] {
            // Sûr : les pages engagées ; l'offre et la reprise ne les lisent pas, elles en
            // changent l'état.
            unsafe { std::slice::from_raw_parts_mut(self.debut.as_ptr(), self.engagee) }
        }
    }

    impl Drop for Pages {
        fn drop(&mut self) {
            // Sûr : l'adresse vient de `VirtualAlloc` ; `MEM_RELEASE` veut une taille nulle.
            let _ = unsafe { VirtualFree(self.debut.as_ptr().cast(), 0, MEM_RELEASE) };
        }
    }
}

#[cfg(not(windows))]
mod imp {
    /// Un tampon ordinaire : sur cette plateforme, le geste d'offre n'est pas encore écrit.
    pub struct Bloc(Vec<u8>);

    impl Bloc {
        pub fn nouveau(longueur: usize) -> Option<Self> {
            (longueur > 0).then(|| Self(vec![0; longueur]))
        }

        pub fn longueur(&self) -> usize {
            self.0.len()
        }

        pub fn octets(&self) -> &[u8] {
            &self.0
        }

        pub fn octets_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }

        pub fn offrable(&self) -> bool {
            false
        }

        /// Rien n'est rendu : la reprise retrouvera tout.
        pub fn offrir(&mut self) -> bool {
            false
        }

        pub fn reprendre(&mut self) -> bool {
            true
        }

        #[cfg(test)]
        pub fn adresse(&self) -> (*const u8, usize) {
            (self.0.as_ptr(), self.0.len())
        }
    }
}

#[cfg(test)]
mod tests;
