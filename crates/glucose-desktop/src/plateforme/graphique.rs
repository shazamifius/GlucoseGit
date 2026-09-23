//! **Ce que le système accorde à Glucose sur la carte graphique, et ce qu'il y occupe**
//! (VRAM-1).
//!
//! # Le signal de l'adaptation, observé et non deviné
//!
//! Une carte graphique n'appartient pas à Glucose. Un Blender qui rend, un jeu, un navigateur
//! plein d'onglets y prennent leur part, et le système répartit : il accorde à chaque processus
//! un **budget**, qu'il rétrécit quand un autre réclame. Dépasser ce budget, c'est faire
//! déplacer ses textures par le pilote, au prix de saccades pour tout le monde.
//!
//! Ce budget est exactement le renseignement dont un cache de textures a besoin pour se nicher
//! là où il reste de la place (charte, « se mettre là où il y a de la place ») : il ne se
//! choisit pas, il se lit, et il se relit à chaque image.
//!
//! # Par où il se lit
//!
//! Sous Windows, DXGI le donne pour n'importe quelle couche graphique — Vulkan compris — parce
//! que c'est le gestionnaire de mémoire vidéo du système qui le tient, pas l'API qui dessine.
//! La carte se retrouve par ses identifiants matériels, les mêmes des deux côtés.
//!
//! Ailleurs, la sonde ne sait rien et le dit : `None`. Le cache de la carte garde alors ce que
//! l'écran demande, et rien de plus — ce qu'il faisait avant d'avoir un budget.

pub use imp::Sonde;

#[cfg(windows)]
mod imp {
    use crate::memoire::MemoireGraphique;
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO,
    };

    /// La carte graphique qui présente, vue par le gestionnaire de mémoire vidéo de Windows.
    pub struct Sonde {
        adaptateur: IDXGIAdapter3,
    }

    impl Sonde {
        /// La sonde de la carte qui porte ces identifiants matériels, ou `None` si Windows ne
        /// la connaît pas ou ne sait pas dire son budget.
        ///
        /// Faite une fois, à l'ouverture de la carte : énumérer les adaptateurs coûte, lire le
        /// budget d'un adaptateur connu ne coûte presque rien.
        pub fn pour(vendeur: u32, appareil: u32) -> Option<Self> {
            // Sûr : des appels COM sans pointeur brut ; les erreurs deviennent `None`.
            let fabrique: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
            (0..)
                .map_while(|rang| unsafe { fabrique.EnumAdapters1(rang) }.ok())
                .find(|a: &IDXGIAdapter1| {
                    unsafe { a.GetDesc1() }
                        .is_ok_and(|d| d.VendorId == vendeur && d.DeviceId == appareil)
                })?
                .cast::<IDXGIAdapter3>()
                .ok()
                .map(|adaptateur| Self { adaptateur })
        }

        /// Le budget que le système accorde maintenant à ce processus sur la carte, et ce
        /// qu'il y occupe — ou `None` si Windows ne le dit pas.
        pub fn lire(&self) -> Option<MemoireGraphique> {
            let mut vu = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            // Sûr : `vu` est local et du type que l'appel remplit.
            unsafe {
                self.adaptateur
                    .QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut vu)
            }
            .ok()?;
            (vu.Budget > 0).then_some(MemoireGraphique {
                budget: vu.Budget,
                utilisee: vu.CurrentUsage,
            })
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::memoire::MemoireGraphique;

    /// Sur cette plateforme, la sonde ne sait rien encore, et le dit.
    pub struct Sonde;

    impl Sonde {
        pub fn pour(_vendeur: u32, _appareil: u32) -> Option<Self> {
            None
        }

        pub fn lire(&self) -> Option<MemoireGraphique> {
            None
        }
    }
}

#[cfg(test)]
mod tests;
