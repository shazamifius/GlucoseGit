//! **GStreamer, ouvert à l'exécution** (VIDEO-1, fiche 169) : les lecteurs du système sous Linux.
//!
//! Lié à la compilation, Glucose ne démarrerait plus sur une machine sans GStreamer — la charte
//! n'en exclut aucune. Il est donc ouvert au premier besoin (`dlopen`), comme Firefox le faisait
//! (bug 859199) : absent, une vidéo garde son image fixe, et rien d'autre ne change. Son ABI est
//! stable depuis la 1.0 (2012) : une seule table vaut pour toutes les versions 1.x.
//!
//! Seules les fonctions dont le lecteur a besoin sont ici, chacune vérifiée à l'ouverture :
//! une table incomplète n'ouvre rien.

use libloading::Library;
use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void, CString};
use std::sync::OnceLock;

pub(super) type Ptr = *mut c_void;

/// `GstMapInfo` : ce que `gst_buffer_map` remplit (la mise en page de la 1.0, inchangée).
#[repr(C)]
pub(super) struct MapInfo {
    memory: Ptr,
    flags: c_int,
    pub data: *mut u8,
    pub size: usize,
    maxsize: usize,
    user_data: [Ptr; 4],
    reserved: [Ptr; 4],
}

impl MapInfo {
    pub(super) fn vide() -> Self {
        Self {
            memory: std::ptr::null_mut(),
            flags: 0,
            data: std::ptr::null_mut(),
            size: 0,
            maxsize: 0,
            user_data: [std::ptr::null_mut(); 4],
            reserved: [std::ptr::null_mut(); 4],
        }
    }
}

/// `GstAppSrcCallbacks` : la source qui lit les octets d'une vidéo là où ils sont.
#[repr(C)]
pub(super) struct RappelsSource {
    pub need_data: unsafe extern "C" fn(Ptr, c_uint, Ptr),
    pub enough_data: unsafe extern "C" fn(Ptr, Ptr),
    pub seek_data: unsafe extern "C" fn(Ptr, u64, Ptr) -> c_int,
    pub reserved: [usize; 4],
}

/// Chaque bibliothèque, puis ses fonctions (`; ...` : des arguments variables, comme en C).
macro_rules! table {
    ($([$lib:ident] $($nom:ident : fn($($a:ty),* $(; $va:tt)?) $(-> $r:ty)?;)*)*) => {
        /// Les fonctions de GStreamer et de GObject dont le lecteur se sert.
        #[allow(non_snake_case)]
        pub(super) struct Gst {
            $($(pub $nom: unsafe extern "C" fn($($a,)* $($va)?) $(-> $r)?,)*)*
            _bibliotheques: Vec<Library>,
        }

        impl Gst {
            fn ouvrir(bibliotheques: Vec<Library>) -> Option<Self> {
                let [$($lib),*] = &bibliotheques[..] else { return None };
                // SAFETY : chaque symbole est pris avec la signature de l'en-tête de GStreamer
                // 1.x ; les bibliothèques vivent dans la table, donc autant que ses pointeurs.
                unsafe {
                    Some(Self {
                        $($($nom: *$lib
                            .get::<unsafe extern "C" fn($($a,)* $($va)?) $(-> $r)?>(
                                concat!(stringify!($nom), "\0").as_bytes(),
                            )
                            .ok()?,)*)*
                        _bibliotheques: bibliotheques,
                    })
                }
            }
        }
    };
}

table! {
    [gstreamer]
        gst_init_check: fn(*mut c_int, *mut c_void, *mut c_void) -> c_int;
        gst_element_factory_make: fn(*const c_char, *const c_char) -> Ptr;
        gst_element_set_state: fn(Ptr, c_int) -> c_int;
        gst_element_get_state: fn(Ptr, *mut c_int, *mut c_int, u64) -> c_int;
        gst_element_get_bus: fn(Ptr) -> Ptr;
        gst_element_seek_simple: fn(Ptr, c_int, c_int, i64) -> c_int;
        gst_bus_pop_filtered: fn(Ptr, c_uint) -> Ptr;
        gst_object_unref: fn(Ptr);
        gst_mini_object_unref: fn(Ptr);
        gst_caps_from_string: fn(*const c_char) -> Ptr;
        gst_caps_get_structure: fn(Ptr, c_uint) -> Ptr;
        gst_structure_get_int: fn(Ptr, *const c_char, *mut c_int) -> c_int;
        gst_sample_get_buffer: fn(Ptr) -> Ptr;
        gst_sample_get_caps: fn(Ptr) -> Ptr;
        gst_buffer_map: fn(Ptr, *mut MapInfo, c_int) -> c_int;
        gst_buffer_unmap: fn(Ptr, *mut MapInfo);
        gst_buffer_new_allocate: fn(Ptr, usize, Ptr) -> Ptr;
        gst_buffer_set_size: fn(Ptr, isize);
    [gobject]
        g_object_set: fn(Ptr, *const c_char; ...);
        g_signal_connect_data: fn(Ptr, *const c_char, Ptr, Ptr, Ptr, c_int) -> c_ulong;
    [gstapp]
        gst_app_sink_try_pull_sample: fn(Ptr, u64) -> Ptr;
        gst_app_sink_try_pull_preroll: fn(Ptr, u64) -> Ptr;
        gst_app_sink_is_eos: fn(Ptr) -> c_int;
        gst_app_src_push_buffer: fn(Ptr, Ptr) -> c_int;
        gst_app_src_end_of_stream: fn(Ptr) -> c_int;
        gst_app_src_set_callbacks: fn(Ptr, *const RappelsSource, Ptr, Ptr);
}

/// Les noms sous lesquels chaque système range ces bibliothèques (le `.so.0` de leur ABI).
/// GStreamer d'abord : il apporte GObject, que le système retrouve alors par son nom, même là où
/// GLib n'est dans aucun chemin standard (NixOS).
const BIBLIOTHEQUES: [&str; 3] = [
    "libgstreamer-1.0.so.0",
    "libgobject-2.0.so.0",
    "libgstapp-1.0.so.0",
];

/// Où les greffons ont été trouvés à la compilation (sous NixOS, `nix/glucose.nix`) ; ailleurs,
/// GStreamer cherche dans ceux du système. Ils lui sont annoncés au lancement
/// ([`super::annoncer_les_greffons`]).
pub(super) const GREFFONS: Option<&str> = option_env!("GLUCOSE_GREFFONS_GST");

/// **GStreamer, ouvert une fois pour tout le processus** — ou rien, s'il manque ici.
pub(super) fn gst() -> Option<&'static Gst> {
    static GST: OnceLock<Option<Gst>> = OnceLock::new();
    GST.get_or_init(|| {
        let ouvert = ouvrir();
        if ouvert.is_none() {
            crate::dire_erreur!(
                "[video] GStreamer introuvable : les vidéos restent sur leur image fixe"
            );
        }
        ouvert
    })
    .as_ref()
}

fn ouvrir() -> Option<Gst> {
    let bibliotheques = (BIBLIOTHEQUES.iter())
        // SAFETY : ouvrir une bibliothèque du système exécute ses constructeurs ; ceux de GLib
        // et de GStreamer n'ont pas d'effet de bord hors d'elles.
        .map(|nom| {
            unsafe { Library::new(nom) }
                .map_err(|e| crate::dire_erreur!("[video] {nom} : {e}"))
                .ok()
        })
        .collect::<Option<Vec<_>>>()?;
    let gst = Gst::ouvrir(bibliotheques)?;
    // SAFETY : `gst_init_check` accepte des arguments nuls ; appelé une seule fois (OnceLock).
    let debut = std::time::Instant::now();
    let pret = unsafe {
        (gst.gst_init_check)(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    // Une seconde ici, c'est son registre refait (des greffons neufs) ; quelques millisecondes,
    // c'est son cache (fiche 169 § 4).
    crate::dire!("[video] GStreamer prêt en {:?}", debut.elapsed());
    (pret != 0).then_some(gst)
}

/// Une chaîne C pour un nom de propriété ou d'élément (jamais d'octet nul : des littéraux).
pub(super) fn c(texte: &str) -> CString {
    CString::new(texte).unwrap_or_default()
}
