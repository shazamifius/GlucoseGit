//! **Media Foundation, ouvert à l'exécution** (VIDEO-WINDOWS-1, fiche 175 ; `decisions/21`) : les
//! cinq fonctions de `mfplat.dll` dont le lecteur a besoin, prises par `GetProcAddress`. Le reste
//! passe par COM (`CoCreateInstance`), qui ne lie rien.
//!
//! **Pourquoi pas lié** : `windows` lie ses fonctions comme des imports ordinaires du binaire.
//! Les éditions « N » de Windows n'ont pas `mfplat.dll` sans leur « Media Feature Pack » : lié,
//! Glucose n'y démarrerait plus — la charte n'exclut aucune machine. Ouvert, son absence ne coûte
//! que les vidéos, comme GStreamer absent sous Linux.

use std::ffi::c_void;
use std::sync::OnceLock;
use windows::core::{s, w, Interface, GUID, HRESULT, PCSTR};
use windows::Win32::Media::MediaFoundation::{
    IMFActivate, IMFAttributes, IMFByteStream, MFMediaType_Video, MFVideoFormat_H264,
    MFSTARTUP_FULL, MFT_CATEGORY_VIDEO_DECODER, MFT_ENUM_FLAG_ALL, MFT_REGISTER_TYPE_INFO,
    MF_VERSION,
};
use windows::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, CoUninitialize, IStream, COINIT_MULTITHREADED,
};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
};

type Demarrer = unsafe extern "system" fn(u32, u32) -> HRESULT;
type Arreter = unsafe extern "system" fn() -> HRESULT;
type Attributs = unsafe extern "system" fn(*mut *mut c_void, u32) -> HRESULT;
type FlotSur = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT;
type Enumerer = unsafe extern "system" fn(
    GUID,
    u32,
    *const MFT_REGISTER_TYPE_INFO,
    *const MFT_REGISTER_TYPE_INFO,
    *mut *mut Option<IMFActivate>,
    *mut u32,
) -> HRESULT;

/// **Les fonctions de `mfplat.dll`** : `MFStartup`, `MFShutdown`, `MFCreateAttributes`,
/// `MFCreateMFByteStreamOnStream`, `MFTEnumEx`. La bibliothèque reste chargée jusqu'à la fin de
/// Glucose.
pub(super) struct Mf {
    demarrer: Demarrer,
    arreter: Arreter,
    attributs: Attributs,
    flot_sur: FlotSur,
    enumerer: Enumerer,
}

/// **Media Foundation, s'il est là et qu'il décode le H.264** — ce que filment les téléphones ;
/// sans décodeur (un Windows Server sans son rôle « Media Foundation »), une vidéo ne se lirait
/// pas : elle reste un lanceur, comme déposée. Cherché une fois.
pub(super) fn mf() -> Option<&'static Mf> {
    static MF: OnceLock<Option<Mf>> = OnceLock::new();
    MF.get_or_init(|| charger().filter(decode_le_h264)).as_ref()
}

fn charger() -> Option<Mf> {
    // SAFETY : `mfplat.dll` se cherche dans le dossier du système seul (jamais celui du document
    // ou de Glucose) ; chaque fonction prend le type de son en-tête (`mfapi.h`, `mfidl.h`).
    unsafe {
        let dll = LoadLibraryExW(w!("mfplat.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
        let f = |nom: PCSTR| GetProcAddress(dll, nom);
        type Brute = unsafe extern "system" fn() -> isize;
        Some(Mf {
            demarrer: std::mem::transmute::<Brute, Demarrer>(f(s!("MFStartup"))?),
            arreter: std::mem::transmute::<Brute, Arreter>(f(s!("MFShutdown"))?),
            attributs: std::mem::transmute::<Brute, Attributs>(f(s!("MFCreateAttributes"))?),
            flot_sur: std::mem::transmute::<Brute, FlotSur>(f(s!("MFCreateMFByteStreamOnStream"))?),
            enumerer: std::mem::transmute::<Brute, Enumerer>(f(s!("MFTEnumEx"))?),
        })
    }
}

/// **Un décodeur H.264 est-il inscrit ?** (`MFTEnumEx`, de la même bibliothèque.)
fn decode_le_h264(mf: &Mf) -> bool {
    let Some(_demarre) = Demarre::de(mf) else {
        return false;
    };
    let entree = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: MFVideoFormat_H264,
    };
    let (mut liste, mut n) = (std::ptr::null_mut(), 0u32);
    // SAFETY : `MFTEnumEx` rend un tableau de `n` activateurs alloué par COM : chacun est lâché,
    // puis le tableau rendu à `CoTaskMemFree`.
    unsafe {
        let trouve = (mf.enumerer)(
            MFT_CATEGORY_VIDEO_DECODER,
            MFT_ENUM_FLAG_ALL.0 as u32,
            &entree,
            std::ptr::null(),
            &mut liste,
            &mut n,
        )
        .is_ok();
        if !liste.is_null() {
            for i in 0..n as usize {
                drop(std::ptr::read(liste.add(i)));
            }
            CoTaskMemFree(Some(liste as *const c_void));
        }
        trouve && n > 0
    }
}

impl Mf {
    /// **Un jeu d'attributs** vide, prêt pour `n` valeurs.
    pub(super) fn attributs(&self, n: u32) -> windows::core::Result<IMFAttributes> {
        let mut brut = std::ptr::null_mut();
        // SAFETY : `MFCreateAttributes` rend une référence à nous, que `from_raw` prend.
        unsafe {
            (self.attributs)(&mut brut, n).ok()?;
            Ok(IMFAttributes::from_raw(brut))
        }
    }

    /// **Le flot d'octets de Media Foundation** sur un `IStream` à nous.
    pub(super) fn flot_sur(&self, flot: &IStream) -> windows::core::Result<IMFByteStream> {
        let mut brut = std::ptr::null_mut();
        // SAFETY : le flot reste à nous (la fonction prend sa propre référence) ; le flot rendu
        // est une référence à nous, que `from_raw` prend.
        unsafe {
            (self.flot_sur)(flot.as_raw(), &mut brut).ok()?;
            Ok(IMFByteStream::from_raw(brut))
        }
    }
}

/// **COM et Media Foundation démarrés sur ce fil**, jusqu'à la fin de ce garde : à garder plus
/// longtemps que tout objet de Media Foundation créé sous lui.
pub(super) struct Demarre {
    arreter: Arreter,
    com: bool,
}

impl Demarre {
    pub(super) fn de(mf: &Mf) -> Option<Self> {
        // SAFETY : chaque démarrage réussi (`S_FALSE` compris) a sa fin, dans `Drop` ; un fil
        // déjà en appartement unique (`RPC_E_CHANGED_MODE`) le reste et n'a rien à rendre.
        let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        // SAFETY : manqué, `MFStartup` n'a rien à rendre ; COM, si.
        if unsafe { (mf.demarrer)(MF_VERSION, MFSTARTUP_FULL) }.is_err() {
            if com {
                unsafe { CoUninitialize() };
            }
            return None;
        }
        Some(Self {
            arreter: mf.arreter,
            com,
        })
    }
}

impl Drop for Demarre {
    fn drop(&mut self) {
        // SAFETY : le démarrage de `de` a réussi.
        unsafe {
            let _ = (self.arreter)();
            if self.com {
                CoUninitialize();
            }
        }
    }
}
