//! **Glucose sur Android** (fiches 50 et 54) : le point d'entrée que `GameActivity` appelle.
//!
//! Tout Glucose vit dans `glucose-desktop` ; il ne reste ici que ce que le téléphone a de
//! propre — sa boucle, qui naît de l'activité, son dossier privé, et sa console, qui n'existe
//! pas : la sortie de Glucose part dans le journal du système (`adb logcat -s Glucose`).
//!
//! Sur un bureau, cette caisse est vide : elle ne compile que pour Android.
#![cfg(target_os = "android")]

use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;

/// **Ce que `GameActivity` appelle** au lancement, sur le fil de Glucose.
#[no_mangle]
fn android_main(app: AndroidApp) {
    journal::rediriger_la_sortie();
    // Le dossier privé de l'application : Android n'a ni `LOCALAPPDATA` ni `HOME`, et son
    // dossier temporaire n'est pas à elle. Personne d'autre ne le lit, et il part avec elle.
    let dossier = app
        .internal_data_path()
        .unwrap_or_else(std::env::temp_dir)
        .join("glucose");
    let mut constructeur = winit::event_loop::EventLoop::builder();
    constructeur.with_android_app(app);
    let lancement = constructeur
        .build()
        .map_err(|e| e.to_string())
        .and_then(|boucle| {
            glucose_desktop::demarrage::lancer(boucle, &dossier).map_err(|e| e.to_string())
        });
    if let Err(e) = lancement {
        eprintln!("[Glucose] le lancement a echoue : {e}");
    }
}

/// **La sortie de Glucose dans le journal d'Android.** Un tube remplace la sortie standard et
/// la sortie d'erreur ; un fil en lit chaque ligne et l'écrit au journal, sous l'étiquette
/// « Glucose ». `liblog` est dans chaque Android : aucune caisse de plus.
mod journal {
    use std::ffi::CString;
    use std::io::{BufRead, BufReader};
    use std::os::fd::FromRawFd;
    use std::os::raw::{c_char, c_int};

    /// `ANDROID_LOG_INFO`.
    const INFO: c_int = 4;

    #[link(name = "log")]
    extern "C" {
        fn __android_log_write(
            priorite: c_int,
            etiquette: *const c_char,
            texte: *const c_char,
        ) -> c_int;
    }

    pub fn rediriger_la_sortie() {
        let mut tube = [0 as c_int; 2];
        // SAFETY : `pipe` remplit deux descripteurs neufs ; `dup2` les pose sur 1 et 2, que ce
        // processus possède ; le bout lu n'appartient qu'au fil qui suit.
        unsafe {
            if libc::pipe(tube.as_mut_ptr()) != 0 {
                return;
            }
            libc::dup2(tube[1], libc::STDOUT_FILENO);
            libc::dup2(tube[1], libc::STDERR_FILENO);
        }
        let lecture = unsafe { std::fs::File::from_raw_fd(tube[0]) };
        let _ = std::thread::Builder::new()
            .name("journal".into())
            .spawn(move || {
                let etiquette = c"Glucose";
                for ligne in BufReader::new(lecture).lines().map_while(Result::ok) {
                    if let Ok(texte) = CString::new(ligne) {
                        // SAFETY : deux chaînes C valides, terminées par un zéro.
                        unsafe { __android_log_write(INFO, etiquette.as_ptr(), texte.as_ptr()) };
                    }
                }
            });
    }
}
