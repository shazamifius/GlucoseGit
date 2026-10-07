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
    selecteur::brancher(&app);
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

/// **Ce qu'une autre application partage vers Glucose** (PARTAGE-1, fiche 56) : l'activité
/// Java a ouvert les fichiers, Rust les lit et les confie au dépôt de Glucose.
mod partage {
    use glucose_desktop::plateforme::partage::{lire, recevoir, Partage};
    use jni::objects::{JClass, JIntArray, JLongArray, JString};
    use jni::refs::Reference;
    use jni::{Env, EnvUnowned};
    use std::os::fd::FromRawFd;

    /// Les arguments de Java, copiés en Rust : les descripteurs, leurs débuts, leurs
    /// longueurs, le texte.
    type Arguments = (Vec<i32>, Vec<i64>, Vec<i64>, Option<String>);

    /// `MainActivity.recevoirUnPartage`, sur le fil de Java qui a ouvert les fichiers — jamais
    /// celui de l'interface : la lecture peut prendre son temps.
    #[no_mangle]
    pub extern "system" fn Java_com_glucose_app_MainActivity_recevoirUnPartage<'l>(
        mut env: EnvUnowned<'l>,
        _classe: JClass<'l>,
        fichiers: JIntArray<'l>,
        debuts: JLongArray<'l>,
        longueurs: JLongArray<'l>,
        texte: JString<'l>,
    ) {
        let lus = env
            .with_env(|env| -> jni::errors::Result<_> {
                Ok(copier(env, (&fichiers, &debuts, &longueurs), &texte)
                    .map_err(|e| eprintln!("[Glucose] partage : {e}"))
                    .ok())
            })
            .resolve::<jni::errors::LogErrorAndDefault>();
        // Un partage dont les arguments ne se lisent pas laisse ses descripteurs ouverts : ils
        // partent avec le processus. Il n'arrive que si Java et Rust ne s'accordent plus.
        let Some((fichiers, debuts, longueurs, texte)) = lus else {
            return;
        };
        let mut partage = Partage {
            texte,
            ..Partage::default()
        };
        for ((fd, debut), longueur) in fichiers.into_iter().zip(debuts).zip(longueurs) {
            let recu = (fd >= 0)
                .then(|| {
                    // SAFETY : Java a détaché ce descripteur pour nous (`detachFd`) : il est à
                    // Rust seul, et `File` le fermera.
                    let fichier = unsafe { std::fs::File::from_raw_fd(fd) };
                    let longueur = u64::try_from(longueur).ok();
                    lire(fichier, u64::try_from(debut).unwrap_or(0), longueur)
                })
                .flatten();
            match recu {
                Some(recu) => partage.recus.push(recu),
                None => partage.illisibles += 1,
            }
        }
        println!(
            "[Glucose] partage : {} image(s) lue(s), {} illisible(s), {}",
            partage.recus.len(),
            partage.illisibles,
            if partage.texte.is_some() {
                "un texte"
            } else {
                "sans texte"
            }
        );
        recevoir(partage);
    }

    /// Les tableaux et le texte de Java, en Rust.
    fn copier(
        env: &mut Env,
        (fichiers, debuts, longueurs): (&JIntArray, &JLongArray, &JLongArray),
        texte: &JString,
    ) -> jni::errors::Result<Arguments> {
        let n = fichiers.len(env)?;
        let (mut f, mut d, mut l) = (vec![0; n], vec![0; n], vec![0; n]);
        fichiers.get_region(env, 0, &mut f)?;
        debuts.get_region(env, 0, &mut d)?;
        longueurs.get_region(env, 0, &mut l)?;
        let texte = if texte.is_null() {
            None
        } else {
            Some(texte.try_to_string(env)?)
        };
        Ok((f, d, l, texte))
    }
}

/// **Le sélecteur de photos du système** (fiche 56) : « Ajouter des images… » appelle
/// `MainActivity.choisirDesImages`, et ce qu'on y choisit revient comme un partage.
mod selecteur {
    use jni::objects::JObject;
    use winit::platform::android::activity::AndroidApp;

    /// Branche le sélecteur de cette activité : chaque activité a le sien, et la dernière née
    /// remplace la précédente.
    pub fn brancher(app: &AndroidApp) {
        // Une adresse plutôt qu'un pointeur : elle traverse les fils, et ne se lit qu'en JNI.
        let activite = app.activity_as_ptr() as usize;
        glucose_desktop::plateforme::partage::installer_le_selecteur(Box::new(move || {
            if let Err(e) = ouvrir(activite) {
                eprintln!("[Glucose] selecteur de photos : {e}");
            }
        }));
    }

    fn ouvrir(activite: usize) -> jni::errors::Result<()> {
        let vm = jni::JavaVM::singleton()?;
        vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            // SAFETY : une référence globale à l'activité, qu'`android-activity` garde tant
            // qu'elle vit ; `JObject` ne la libère pas en partant.
            let activite = unsafe { JObject::from_raw(env, activite as jni::sys::jobject) };
            env.call_method(
                &activite,
                jni::jni_str!("choisirDesImages"),
                jni::jni_sig!("()V"),
                &[],
            )?;
            Ok(())
        })
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
