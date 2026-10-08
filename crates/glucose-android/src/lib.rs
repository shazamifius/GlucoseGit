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
    installation::brancher(&app);
    glucose_desktop::plateforme::clavier::installer(Box::new(clavier::DuTelephone(app.clone())));
    glucose_desktop::plateforme::doigt::installer(Box::new(doigt::DuTelephone(
        java::Activite::de(&app),
    )));
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
    use winit::platform::android::activity::AndroidApp;

    /// Branche le sélecteur de cette activité : chaque activité a le sien, et la dernière née
    /// remplace la précédente.
    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        glucose_desktop::plateforme::partage::installer_le_selecteur(Box::new(move || {
            activite.appeler(jni::jni_str!("choisirDesImages"), "selecteur de photos");
        }));
    }
}

/// **La mise à jour confiée à Android** (MAJ-ANDROID-1, fiche 58) : l'APK vérifié par la clé de
/// Glucose part à `MainActivity.installerUnApk` ; ce qu'Android en dit revient par
/// `recevoirLInstallation`, et Glucose le dit dans ses mots.
mod installation {
    use glucose_desktop::plateforme::installation::{installer_le_confieur, recevoir, Etat};
    use jni::objects::{JClass, JString};
    use jni::sys::jint;
    use jni::EnvUnowned;
    use winit::platform::android::activity::AndroidApp;

    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        installer_le_confieur(std::sync::Arc::new(move |apk| {
            let chemin = apk.to_string_lossy();
            activite.appeler_avec(jni::jni_str!("installerUnApk"), &chemin, "mise a jour");
        }));
    }

    /// `MainActivity.recevoirLInstallation` : 1, autoriser ; 2, confirmer ; sinon, l'échec.
    #[no_mangle]
    pub extern "system" fn Java_com_glucose_app_MainActivity_recevoirLInstallation<'l>(
        mut env: EnvUnowned<'l>,
        _classe: JClass<'l>,
        etat: jint,
        detail: JString<'l>,
    ) {
        let detail = env
            .with_env(|env| -> jni::errors::Result<Option<String>> {
                if detail.is_null() {
                    Ok(None)
                } else {
                    detail.try_to_string(env).map(Some)
                }
            })
            .resolve::<jni::errors::LogErrorAndDefault>();
        recevoir(match etat {
            1 => Etat::Autoriser,
            2 => Etat::Confirmer,
            _ => Etat::Echec(detail.unwrap_or_else(|| "sans raison donnée".into())),
        });
    }
}

/// **Les marges du système** (BORD-1, fiche 57) : ce que `MainActivity.onApplyWindowInsets`
/// voit, en pixels de la fenêtre — sur le fil de l'interface d'Android, qui n'attend pas.
mod marges {
    use glucose_desktop::plateforme::marges::{recevoir, Marges};
    use jni::objects::JClass;
    use jni::sys::jint;
    use jni::EnvUnowned;

    #[no_mangle]
    pub extern "system" fn Java_com_glucose_app_MainActivity_recevoirLesMarges<'l>(
        _env: EnvUnowned<'l>,
        _classe: JClass<'l>,
        gauche: jint,
        haut: jint,
        droite: jint,
        bas: jint,
        clavier: jint,
    ) {
        let px = |v: jint| v.max(0) as f32;
        recevoir(Marges {
            gauche: px(gauche),
            haut: px(haut),
            droite: px(droite),
            bas: px(bas),
            clavier: px(clavier),
        });
    }
}

/// **Ce qu'Android dit du doigt** (APPUI-1, fiche 57) : le délai de l'appui long que chacun
/// règle dans l'accessibilité, et la vibration qui dit qu'il a pris.
mod doigt {
    use glucose_desktop::plateforme::doigt::Doigt;
    use std::time::Duration;

    pub struct DuTelephone(pub super::java::Activite);

    impl Doigt for DuTelephone {
        fn appui_long(&self) -> Option<Duration> {
            let ms = super::java::sur_le_fil(
                |env| {
                    env.call_static_method(
                        jni::jni_str!("android/view/ViewConfiguration"),
                        jni::jni_str!("getLongPressTimeout"),
                        jni::jni_sig!("()I"),
                        &[],
                    )?
                    .i()
                },
                "delai de l'appui long",
            )?;
            u64::try_from(ms)
                .ok()
                .filter(|ms| *ms > 0)
                .map(Duration::from_millis)
        }

        fn sentir_l_appui(&self) {
            self.0
                .appeler(jni::jni_str!("sentirLAppui"), "vibration de l'appui long");
        }
    }
}

/// **Parler à Java** depuis le fil de Glucose : s'y attacher, appeler, et ne jamais laisser une
/// exception Java en suspens — la suivante ferait tomber le processus.
mod java {
    use jni::objects::JObject;
    use jni::strings::JNIStr;
    use jni::Env;
    use winit::platform::android::activity::AndroidApp;

    /// L'activité de Glucose, par son adresse : elle traverse les fils, et ne se lit qu'en JNI.
    #[derive(Clone, Copy)]
    pub struct Activite(usize);

    impl Activite {
        pub fn de(app: &AndroidApp) -> Self {
            Self(app.activity_as_ptr() as usize)
        }

        /// Appelle une méthode `()V` de `MainActivity` ; un échec se dit dans le journal.
        pub fn appeler(self, methode: &'static JNIStr, quoi: &str) {
            sur_le_fil(
                |env| {
                    // SAFETY : une référence globale à l'activité, qu'`android-activity` garde
                    // tant qu'elle vit ; `JObject` ne la libère pas en partant.
                    let activite = unsafe { JObject::from_raw(env, self.0 as jni::sys::jobject) };
                    env.call_method(&activite, methode, jni::jni_sig!("()V"), &[])?;
                    Ok(())
                },
                quoi,
            );
        }
    }

    impl Activite {
        /// Appelle une méthode `(String)V` de `MainActivity`, avec cette chaîne.
        pub fn appeler_avec(self, methode: &'static JNIStr, chaine: &str, quoi: &str) {
            sur_le_fil(
                |env| {
                    // SAFETY : comme dans `appeler`, la référence globale qu'`android-activity`
                    // garde à l'activité tant qu'elle vit.
                    let activite = unsafe { JObject::from_raw(env, self.0 as jni::sys::jobject) };
                    let texte = env.new_string(chaine)?;
                    env.call_method(
                        &activite,
                        methode,
                        jni::jni_sig!("(Ljava/lang/String;)V"),
                        &[jni::JValue::from(&texte)],
                    )?;
                    Ok(())
                },
                quoi,
            );
        }
    }

    /// Attache ce fil à la machine Java le temps de `f`. Une erreur s'écrit au journal, et
    /// l'exception qu'elle a laissée s'efface.
    pub fn sur_le_fil<T>(
        f: impl FnOnce(&mut Env) -> jni::errors::Result<T>,
        quoi: &str,
    ) -> Option<T> {
        let vm = jni::JavaVM::singleton().ok()?;
        vm.attach_current_thread(|env| -> jni::errors::Result<Option<T>> {
            let rendu = f(env);
            if rendu.is_err() && env.exception_check() {
                env.exception_clear();
            }
            Ok(rendu.map_err(|e| eprintln!("[Glucose] {quoi} : {e}")).ok())
        })
        .ok()
        .flatten()
    }
}

/// **Le clavier virtuel** (CLAVIER-1, fiche 57) : `GameActivity` tient le texte qu'il réécrit,
/// `winit` jette l'évènement qui le dit ; Glucose relit et réécrit cet état par `AndroidApp`.
/// Les positions restent en unités UTF-16, celles de Java : le miroir les convertit.
mod clavier {
    use glucose_desktop::plateforme::clavier::{Clavier, EtatDuClavier};
    use winit::platform::android::activity::input::{
        ImeOptions, InputType, TextInputAction, TextInputState, TextSpan,
    };
    use winit::platform::android::activity::AndroidApp;

    pub struct DuTelephone(pub AndroidApp);

    impl Clavier for DuTelephone {
        fn montrer(&self, etat: &EtatDuClavier) {
            // **Multiligne** : sans ce drapeau, `GameTextInput` change Entrée en « action » et
            // filtre les sauts de ligne (lu dans `InputConnection`, `games-activity` 4.4.0) — or
            // Entrée va à la ligne dans un nœud, comme au bureau.
            let genre = InputType::TYPE_CLASS_TEXT
                | InputType::TYPE_TEXT_FLAG_MULTI_LINE
                | InputType::TYPE_TEXT_FLAG_CAP_SENTENCES
                | InputType::TYPE_TEXT_FLAG_AUTO_CORRECT;
            // Jamais le clavier plein écran du paysage : il cacherait le nœud qu'on écrit.
            let options = ImeOptions::IME_FLAG_NO_FULLSCREEN | ImeOptions::IME_FLAG_NO_ENTER_ACTION;
            self.0
                .set_ime_editor_info(genre, TextInputAction::None, options);
            self.ecrire(etat);
            // `false` : une demande explicite — l'utilisateur a touché le texte.
            self.0.show_soft_input(false);
        }

        fn ecrire(&self, etat: &EtatDuClavier) {
            let span = |(start, end): (usize, usize)| TextSpan { start, end };
            self.0.set_text_input_state(TextInputState {
                text: etat.texte.clone(),
                selection: span(etat.selection),
                compose_region: etat.composition.map(span),
            });
        }

        fn cacher(&self) {
            self.0.hide_soft_input(false);
        }

        fn lire(&self) -> EtatDuClavier {
            let etat = self.0.text_input_state();
            EtatDuClavier {
                texte: etat.text,
                selection: (etat.selection.start, etat.selection.end),
                composition: etat.compose_region.map(|s| (s.start, s.end)),
            }
        }
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
