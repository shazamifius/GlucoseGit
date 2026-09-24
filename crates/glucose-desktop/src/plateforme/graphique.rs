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
//!
//! # Et quand Glucose dort (ETAGES-2)
//!
//! Relire le budget à chaque image ne sert à rien au repos : il ne s'en dessine aucune. Un
//! Blender qui se met à rendre pendant que Glucose attend trouvait donc la carte encombrée des
//! textures que Glucose gardait hors de l'écran. Windows signale lui-même chaque changement
//! de budget par un événement : un fil y dort, sans rien coûter tant que rien ne bouge, et
//! réveille la boucle quand il se passe quelque chose.

pub use imp::Sonde;

/// Ce qui réveille la boucle d'images depuis un autre fil ; rend faux quand il n'y a plus
/// personne à réveiller.
pub type Reveil = Box<dyn Fn() -> bool + Send>;

#[cfg(windows)]
mod imp {
    use crate::memoire::MemoireGraphique;
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows::core::Interface;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO,
    };
    use windows::Win32::System::Threading::{SetEvent, WaitForSingleObject, INFINITE};

    /// La carte graphique qui présente, vue par le gestionnaire de mémoire vidéo de Windows.
    pub struct Sonde {
        adaptateur: IDXGIAdapter3,
        veilleur: Option<Veilleur>,
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
                .map(|adaptateur| Self {
                    adaptateur,
                    veilleur: None,
                })
        }

        /// **Veille le budget** : à chaque changement que Windows signale, `reveil` est appelé
        /// depuis un fil qui dort le reste du temps. Sans effet si Windows refuse.
        pub fn veiller(&mut self, reveil: super::Reveil) {
            self.veilleur = Veilleur::nouveau(&self.adaptateur, reveil);
        }

        /// Le budget a-t-il changé depuis la dernière question ? Faux sans veilleur.
        pub fn a_change(&self) -> bool {
            self.veilleur
                .as_ref()
                .is_some_and(|v| v.change.swap(false, Ordering::Relaxed))
        }

        /// Fait comme si Windows avait signalé un changement : l'épreuve ne peut pas en
        /// provoquer un vrai sans occuper la carte.
        #[cfg(test)]
        pub fn signaler(&self) {
            if let Some(v) = &self.veilleur {
                // Sûr : l'événement vit autant que le veilleur.
                let _ = unsafe { SetEvent(v.evenement.0) };
            }
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

    unsafe extern "system" {
        /// Déclarée ici : la liaison de la bibliothèque demande tout son module de sécurité
        /// pour un argument qu'on laisse nul.
        fn CreateEventW(
            attributs: *const std::ffi::c_void,
            manuel: i32,
            initial: i32,
            nom: *const u16,
        ) -> *mut std::ffi::c_void;
    }

    /// Un événement de Windows, qu'un autre fil peut attendre.
    struct Evenement(HANDLE);

    // Sûr : un handle d'événement s'attend et se signale depuis n'importe quel fil ; c'est
    // l'usage pour lequel il existe.
    unsafe impl Send for Evenement {}
    unsafe impl Sync for Evenement {}

    /// **Le fil qui dort sur l'événement du budget**, et ce qu'il a vu.
    struct Veilleur {
        adaptateur: IDXGIAdapter3,
        cookie: u32,
        evenement: std::sync::Arc<Evenement>,
        change: std::sync::Arc<AtomicBool>,
        fermer: std::sync::Arc<AtomicBool>,
        fil: Option<std::thread::JoinHandle<()>>,
    }

    impl Veilleur {
        fn nouveau(adaptateur: &IDXGIAdapter3, reveil: super::Reveil) -> Option<Self> {
            // Sûr : un événement anonyme, à remise automatique, fermé dans `drop`.
            let brut = unsafe { CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
            if brut.is_null() {
                return None;
            }
            let evenement = std::sync::Arc::new(Evenement(HANDLE(brut)));
            // Sûr : l'événement vit autant que le veilleur, qui se désinscrit avant de le fermer.
            let cookie = match unsafe {
                adaptateur.RegisterVideoMemoryBudgetChangeNotificationEvent(evenement.0)
            } {
                Ok(c) => c,
                Err(_) => {
                    let _ = unsafe { CloseHandle(evenement.0) };
                    return None;
                }
            };
            let change = std::sync::Arc::new(AtomicBool::new(false));
            let fermer = std::sync::Arc::new(AtomicBool::new(false));
            let fil = {
                let (evenement, change, fermer) =
                    (evenement.clone(), change.clone(), fermer.clone());
                std::thread::spawn(move || loop {
                    // Sûr : l'événement vit tant que ce fil vit — `drop` l'attend avant de fermer.
                    unsafe { WaitForSingleObject(evenement.0, INFINITE) };
                    if fermer.load(Ordering::Relaxed) {
                        return;
                    }
                    change.store(true, Ordering::Relaxed);
                    if !reveil() {
                        return;
                    }
                })
            };
            Some(Self {
                adaptateur: adaptateur.clone(),
                cookie,
                evenement,
                change,
                fermer,
                fil: Some(fil),
            })
        }
    }

    impl Drop for Veilleur {
        fn drop(&mut self) {
            // Sûr : le cookie vient de l'inscription ; l'événement, réveillé une dernière fois,
            // n'est fermé qu'après que le fil qui l'attend s'est terminé.
            unsafe {
                self.adaptateur
                    .UnregisterVideoMemoryBudgetChangeNotification(self.cookie);
                self.fermer.store(true, Ordering::Relaxed);
                let _ = SetEvent(self.evenement.0);
            }
            if let Some(fil) = self.fil.take() {
                let _ = fil.join();
            }
            let _ = unsafe { CloseHandle(self.evenement.0) };
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

        pub fn veiller(&mut self, _reveil: super::Reveil) {}

        pub fn a_change(&self) -> bool {
            false
        }
    }
}

#[cfg(test)]
mod tests;
