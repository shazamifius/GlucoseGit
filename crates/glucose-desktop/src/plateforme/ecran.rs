//! **ECRAN-1 — la carte graphique qui tient l'écran de la fenêtre**, lue chez le système.
//!
//! # Le défaut : un balancier entre deux cartes, une session sur deux
//!
//! Sa session du 07/10 au matin : une image coûtait 3 ms, et Glucose gelait pourtant de 300 à
//! 500 ms, toutes les minutes, **dans `present`**. Son portable était en mode « carte dédiée
//! directe » (le commutateur d'Armoury Crate) : l'écran branché sur la RTX, le compositeur de
//! Windows sur la RTX, et Glucose qui dessinait sur l'Intel Arc. Chaque image traversait donc
//! d'une carte à l'autre, et l'Intel, qui n'affiche rien, s'endormait dès que Glucose se
//! reposait — `bench_reveil` : 50 ms pour la réveiller après une seconde et quart de repos.
//!
//! L'arbitre (ARBITRE-1) partait de l'économe et n'en changeait qu'au lancement suivant, après
//! avoir vu des gels. La veille au soir, sur la RTX, un hoquet de quelques dizaines de
//! millisecondes lui avait fait retenir l'économe : il alternait, comme son propre code
//! l'avouait — « deux cartes qui gèlent toutes deux le feraient alterner ».
//!
//! # Ce qu'on lit, et pourquoi ce n'est pas une étiquette
//!
//! La charte refuse de choisir par une étiquette (« la puissante ») : ce serait interroger le
//! matériel au lieu de l'observer. Ici on ne demande rien des capacités de la carte : on lit
//! **où vont les pixels**. La carte qui tient l'écran est sur le chemin de toute image —
//! c'est elle qui compose ([Microsoft, *Cross Adapter Scan-Out*](https://devblogs.microsoft.com/directx/optimizing-hybrid-laptop-performance-with-cross-adapter-scan-out-caso/) :
//! dessiner sur une carte pour un écran branché sur une autre fait recopier chaque image par
//! le compositeur). Dessiner ailleurs n'évite donc jamais sa charge — un Blender qui la sature
//! ralentit la composition quoi qu'on fasse — et ajoute une copie par image à travers une carte
//! qui s'endort. Se nicher là où il reste de la place reste juste ; s'en aller de l'écran n'est
//! pas un moyen de le faire.
//!
//! # Ce qui reste ouvert
//!
//! La topologie se lit **au lancement**, comme le souvenir de l'arbitre : une fenêtre glissée
//! sur un écran branché sur l'autre carte garde la sienne jusqu'au lancement suivant (changer
//! de carte en cours de route fige l'affichage, ARBITRE-4). Et hors de Windows, rien ne se lit
//! encore : l'arbitre garde la main, comme avant.

use crate::present::arbitre::Preference;

/// Les identifiants matériels d'une carte : le vendeur et l'appareil, les mêmes chez DXGI et
/// chez `wgpu`.
pub type Identite = (u32, u32);

/// **La préférence qui ouvre la carte de l'écran**, d'après ce que le système range en tête de
/// chaque préférence. `None` quand l'écran est tenu par une carte qu'aucune n'ouvre — un
/// adaptateur virtuel, un pilote de secours : on ne sait pas, et on le dit.
///
/// Une machine à une seule carte rend l'économe : les deux préférences désignent la même, et
/// il n'y a rien à arbitrer.
pub fn preference_de_l_ecran(
    ecran: Identite,
    rapide: Option<Identite>,
    econome: Option<Identite>,
) -> Option<Preference> {
    if econome == Some(ecran) {
        Some(Preference::Econome)
    } else if rapide == Some(ecran) {
        Some(Preference::Rapide)
    } else {
        None
    }
}

/// La carte qui tient l'écran de cette fenêtre, et la préférence qui l'ouvre.
pub fn carte_de_l_ecran(fenetre: &winit::window::Window) -> Option<(Identite, Option<Preference>)> {
    imp::carte_de_l_ecran(fenetre)
}

#[cfg(windows)]
mod imp {
    use super::{preference_de_l_ecran, Identite, Preference};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory6, DXGI_GPU_PREFERENCE,
        DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, DXGI_GPU_PREFERENCE_MINIMUM_POWER,
    };
    use windows::Win32::Graphics::Gdi::{MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTONEAREST};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    pub fn carte_de_l_ecran(
        fenetre: &winit::window::Window,
    ) -> Option<(Identite, Option<Preference>)> {
        let poignee = fenetre.window_handle().ok()?;
        let RawWindowHandle::Win32(w) = poignee.as_raw() else {
            return None;
        };
        carte_de_la_poignee(HWND(w.hwnd.get() as *mut core::ffi::c_void))
    }

    /// La même, pour une poignée de fenêtre Windows : ce que l'épreuve emploie, sans `winit`.
    pub(super) fn carte_de_la_poignee(hwnd: HWND) -> Option<(Identite, Option<Preference>)> {
        // Sûr : une poignée de fenêtre vivante ; l'appel ne fait que la lire.
        let moniteur = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        // Sûr : des appels COM sans pointeur brut ; les erreurs deviennent `None`.
        let fabrique: IDXGIFactory6 = unsafe { CreateDXGIFactory1() }.ok()?;
        let ecran = (0..)
            .map_while(|rang| unsafe { fabrique.EnumAdapters1(rang) }.ok())
            .find(|a| porte(a, moniteur))
            .and_then(|a| identite(&a))?;
        let premiere = |preference: DXGI_GPU_PREFERENCE| {
            unsafe { fabrique.EnumAdapterByGpuPreference::<IDXGIAdapter1>(0, preference) }
                .ok()
                .and_then(|a| identite(&a))
        };
        let rapide = premiere(DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE);
        let econome = premiere(DXGI_GPU_PREFERENCE_MINIMUM_POWER);
        Some((ecran, preference_de_l_ecran(ecran, rapide, econome)))
    }

    /// Une des sorties de cette carte porte-t-elle ce moniteur ?
    fn porte(carte: &IDXGIAdapter1, moniteur: HMONITOR) -> bool {
        (0..)
            .map_while(|rang| unsafe { carte.EnumOutputs(rang) }.ok())
            .any(|sortie| unsafe { sortie.GetDesc() }.is_ok_and(|d| d.Monitor == moniteur))
    }

    fn identite(carte: &IDXGIAdapter1) -> Option<Identite> {
        unsafe { carte.GetDesc1() }
            .ok()
            .map(|d| (d.VendorId, d.DeviceId))
    }
}

#[cfg(not(windows))]
mod imp {
    use super::{Identite, Preference};

    /// Sur cette plateforme, la topologie ne se lit pas encore : l'arbitre garde la main.
    pub fn carte_de_l_ecran(
        _fenetre: &winit::window::Window,
    ) -> Option<(Identite, Option<Preference>)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTEL: Identite = (0x8086, 0x7d51);
    const RTX: Identite = (0x10de, 0x2d58);
    const PARSEC: Identite = (0x1414, 0x0001);

    /// **ECRAN-1** : l'écran sur la carte dédiée ouvre la rapide — sa machine du 07/10 —, sur
    /// l'intégrée l'économe, sur une carte qu'aucune préférence n'ouvre, rien.
    #[test]
    fn test_ecran_1_la_carte_de_l_ecran_designe_sa_preference() {
        let (rapide, econome) = (Some(RTX), Some(INTEL));
        assert_eq!(
            preference_de_l_ecran(RTX, rapide, econome),
            Some(Preference::Rapide)
        );
        assert_eq!(
            preference_de_l_ecran(INTEL, rapide, econome),
            Some(Preference::Econome)
        );
        assert_eq!(preference_de_l_ecran(PARSEC, rapide, econome), None);
    }

    /// **L'écran passe avant le souvenir** : un souvenir est une conclusion tirée de quelques
    /// gels, la topologie est un fait. Celui d'un hoquet sur la RTX avait renvoyé sa session du
    /// 07/10 sur l'Intel, qui n'affichait rien.
    #[test]
    fn test_ecran_1_l_ecran_passe_avant_le_souvenir() {
        use crate::present::gpu::succession::carte_de_depart;
        if crate::present::gpu::succession::carte_imposee().is_some() {
            return; // `GLUCOSE_CARTE` tranche avant tout, et c'est voulu.
        }
        let dossier = std::env::temp_dir().join(format!("glucose-ecran-1-{}", std::process::id()));
        let chemin = dossier.join(crate::present::souvenir::FICHIER);
        crate::present::souvenir::ecrire(&chemin, Preference::Econome).expect("le souvenir");
        let ecran = carte_de_depart(&chemin, Some(Preference::Rapide));
        let sans_ecran = carte_de_depart(&chemin, None);
        std::fs::remove_dir_all(&dossier).ok();
        assert_eq!(
            ecran,
            Preference::Rapide,
            "le souvenir l'a emporte sur l'ecran"
        );
        assert_eq!(
            sans_ecran,
            Preference::Econome,
            "sans ecran, le souvenir decide"
        );
    }

    /// **Sur la machine qui fait tourner l'épreuve**, la lecture aboutit à une carte réelle —
    /// sur une fenêtre posée à −32 000 pixels, hors de tout écran, jamais montrée. Une machine
    /// sans sortie (un serveur de GitHub) n'a pas d'écran à lire : la lecture le dit par `None`.
    #[cfg(windows)]
    #[test]
    fn test_ecran_1_la_lecture_aboutit_sur_cette_machine() {
        use windows::core::w;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
        };
        let fenetre = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!("glucose-epreuve"),
                WS_POPUP,
                -32_000,
                -32_000,
                8,
                8,
                None,
                None,
                None,
                None,
            )
        }
        .expect("une fenetre hors de l'ecran");
        let lu = imp::carte_de_la_poignee(fenetre);
        let _ = unsafe { DestroyWindow(fenetre) };
        eprintln!("ECRAN-1 sur cette machine : {lu:x?}");
        if let Some(((vendeur, _), _)) = lu {
            assert_ne!(vendeur, 0, "une carte sans vendeur : la lecture est fausse");
        }
    }

    /// **Quand l'écran a tranché, il n'y a pas d'arbitre** : c'est ce qui arrête le balancier.
    /// Sans écran lisible ni carte imposée, l'arbitre garde la main, comme avant.
    #[test]
    fn test_ecran_1_l_ecran_connu_ne_laisse_rien_a_arbitrer() {
        use crate::present::gpu::succession::faut_il_un_arbitre;
        assert!(!faut_il_un_arbitre(None, Some(Preference::Rapide)));
        assert!(!faut_il_un_arbitre(Some(Preference::Econome), None));
        assert!(faut_il_un_arbitre(None, None));
    }

    /// Une seule carte : les deux préférences la désignent, et l'économe suffit.
    #[test]
    fn test_ecran_1_une_seule_carte_rend_l_econome() {
        assert_eq!(
            preference_de_l_ecran(INTEL, Some(INTEL), Some(INTEL)),
            Some(Preference::Econome)
        );
    }
}
