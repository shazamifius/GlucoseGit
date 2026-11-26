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

mod partage;
mod presse_papiers;
mod video;

/// **Le nom du dossier de Glucose** dans le dossier privé de l'application : celui de ses
/// documents et de ce qu'il reçoit (`telechargements/`, VIDEO-ANDROID-2).
const DOSSIER: &str = "glucose";

/// **Ce que `GameActivity` appelle** au lancement, sur le fil de Glucose.
#[no_mangle]
fn android_main(app: AndroidApp) {
    journal::rediriger_la_sortie();
    selecteur::brancher(&app);
    feuille::brancher(&app);
    installation::brancher(&app);
    batterie::brancher(&app);
    sortie::brancher(&app);
    place::brancher(&app);
    video::brancher(&app);
    glucose_desktop::plateforme::clavier::installer(Box::new(clavier::DuTelephone(app.clone())));
    glucose_desktop::plateforme::doigt::installer(Box::new(doigt::DuTelephone(
        java::Activite::de(&app),
    )));
    glucose_desktop::plateforme::mouvement::installer(mouvement_reduit);
    // Le dossier privé de l'application : Android n'a ni `LOCALAPPDATA` ni `HOME`, et son
    // dossier temporaire n'est pas à elle. Personne d'autre ne le lit, et il part avec elle.
    let dossier = app
        .internal_data_path()
        .unwrap_or_else(std::env::temp_dir)
        .join(DOSSIER);
    // Le dossier que le fournisseur de Glucose sert (`fichiers_partages.xml`), comme la feuille
    // de partage : ce que Glucose copie y vit.
    presse_papiers::brancher(&app, dossier.join("partages"));
    let mut constructeur = winit::event_loop::EventLoop::builder();
    constructeur.with_android_app(app);
    let boucle = match constructeur.build() {
        Ok(boucle) => boucle,
        Err(winit::error::EventLoopError::RecreationAttempt) => repartir(),
        Err(e) => {
            eprintln!("[Glucose] le lancement a echoue : {e}");
            return;
        }
    };
    if let Err(e) = glucose_desktop::demarrage::lancer(boucle, &dossier, None) {
        eprintln!("[Glucose] le lancement a echoue : {e}");
    }
    // La boucle ne finit que si l'activité est détruite (VIE-3, `MainActivity.onDestroy`), et
    // Glucose a fini avec elle. Le processus s'en va : `winit` ne recréerait pas sa boucle pour
    // l'activité suivante (VIE-2), et le prochain toucher sur l'icône repart d'un processus neuf.
    partir()
}

/// **Le processus part, sans rien défaire** (VIE-3, fiche 173) : `_exit`, et non `exit`, qui
/// détruit les objets statiques des bibliothèques C++ pendant que leurs fils tournent encore —
/// `hwuiTask0`, un fil de dessin d'Android, y prenait un verrou détruit, et le processus tombait
/// en `SIGABRT` : Android comptait un plantage (« APP CRASH(NATIVE) », vu au téléphone virtuel).
/// Ce qui devait finir a fini : le scribe a synchronisé chaque geste, `finir` a clos le reste.
fn partir() -> ! {
    // SAFETY : `_exit` termine le processus, sans rien lire ni écrire de ce qui est à nous.
    unsafe { libc::_exit(0) }
}

/// **VIE-3 — `MainActivity.onDestroy`** (fiche 173) : Glucose finit, puis le processus part.
#[no_mangle]
pub extern "system" fn Java_com_glucose_app_MainActivity_recevoirLaFin(
    _env: jni::EnvUnowned<'_>,
    _classe: jni::objects::JClass<'_>,
) {
    glucose_desktop::plateforme::fin::l_activite_finit();
}

/// **VIE-2 — une activité recréée dans un processus qui vit encore** (fiche 72).
///
/// Android peut détruire l'activité de Glucose sans tuer son processus, puis en créer une neuve
/// dans le même : `android_main` est alors appelé une seconde fois. `winit` ne transmet pas la
/// destruction (« TODO: forward onDestroy notification », `platform_impl/android`) et refuse de
/// recréer sa boucle (`RecreationAttempt`) : l'activité neuve se fermait aussitôt, et chaque
/// toucher sur l'icône refaisait la même chose tant que le système gardait le processus. Rien
/// ne prouve que ce soit ce qu'il a vécu le 15/10 — c'est une fermeture « de lui-même » qui se
/// répète, et la ligne `sortie` le dira (SORTIE-1).
///
/// Le processus se quitte : le toucher suivant repart d'un processus neuf, dont le lancement
/// rouvre le travail (BROUILLON-1). Ce qu'il écrivait est déjà sur le disque geste après geste,
/// et un fichier coupé se relit jusqu'à sa dernière entrée saine — la même garantie qu'une mort
/// par le système.
fn repartir() -> ! {
    eprintln!("[Glucose] une activite neuve dans un processus vivant : le processus repart");
    partir()
}

/// **Le sélecteur de photos du système** (fiche 56) : « Ajouter des images et des vidéos… »
/// appelle `MainActivity.choisirDesImages`, et ce qu'on y choisit revient comme un partage.
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

/// **La feuille de partage** (PARTAGE-2, fiche 63) : exporter et enregistrer une image écrivent
/// dans `partages/`, puis `MainActivity.partagerUnFichier` offre le fichier aux autres
/// applications.
mod feuille {
    use winit::platform::android::activity::AndroidApp;

    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        glucose_desktop::plateforme::partager::installer_le_partageur(std::sync::Arc::new(
            move |chemin, type_mime| {
                let chemin = chemin.to_string_lossy();
                activite.appeler_avec_deux(
                    jni::jni_str!("partagerUnFichier"),
                    (&chemin, type_mime),
                    "feuille de partage",
                );
            },
        ));
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

/// **La batterie** (fiche 58) : Android ferme `/sys` aux applications ; `MainActivity` la lit
/// dans l'intention qu'il garde pour tous, et Rust en fait un pourcentage.
mod batterie {
    use glucose_desktop::plateforme::batterie::{installer, Brut};
    use winit::platform::android::activity::AndroidApp;

    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        installer(Box::new(move || {
            let lus = super::java::sur_le_fil(
                |env| {
                    let objet = activite.objet(env);
                    let rendu = env
                        .call_method(
                            &objet,
                            jni::jni_str!("lireLaBatterie"),
                            jni::jni_sig!("()[I"),
                            &[],
                        )?
                        .l()?;
                    let tableau = env.cast_local::<jni::objects::JIntArray>(rendu)?;
                    if tableau.is_null() {
                        return Ok(None);
                    }
                    let mut v = [0; 3];
                    tableau.get_region(env, 0, &mut v)?;
                    Ok(Some(v))
                },
                "batterie",
            )??;
            Some(Brut {
                niveau: lus[0],
                echelle: lus[1],
                prise: lus[2],
            })
        }));
    }
}

/// **La place du dossier privé** (REPRISE-1, fiche 72) : `MainActivity.lireLaPlace`, les
/// octets libres puis le total.
mod place {
    use glucose_desktop::plateforme::place::{depuis_le_tableau, installer};
    use winit::platform::android::activity::AndroidApp;

    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        installer(Box::new(move || {
            super::java::sur_le_fil(
                |env| {
                    let objet = activite.objet(env);
                    let rendu = env
                        .call_method(
                            &objet,
                            jni::jni_str!("lireLaPlace"),
                            jni::jni_sig!("()[J"),
                            &[],
                        )?
                        .l()?;
                    let tableau = env.cast_local::<jni::objects::JLongArray>(rendu)?;
                    if tableau.is_null() {
                        return Ok(None);
                    }
                    let mut v = vec![0; tableau.len(env)?];
                    tableau.get_region(env, 0, &mut v)?;
                    Ok(depuis_le_tableau(&v))
                },
                "place",
            )
            .flatten()
        }));
    }
}

/// **Comment Android a fermé les derniers processus de Glucose** (SORTIE-1, fiche 72) :
/// `MainActivity.lireLesSorties`, sept nombres par mort, lus par la boîte noire au lancement.
mod sortie {
    use glucose_desktop::plateforme::sortie::{depuis_le_tableau, installer};
    use winit::platform::android::activity::AndroidApp;

    pub fn brancher(app: &AndroidApp) {
        let activite = super::java::Activite::de(app);
        installer(Box::new(move || {
            super::java::sur_le_fil(
                |env| {
                    let objet = activite.objet(env);
                    let rendu = env
                        .call_method(
                            &objet,
                            jni::jni_str!("lireLesSorties"),
                            jni::jni_sig!("()[J"),
                            &[],
                        )?
                        .l()?;
                    let tableau = env.cast_local::<jni::objects::JLongArray>(rendu)?;
                    if tableau.is_null() {
                        return Ok(Vec::new());
                    }
                    let mut v = vec![0; tableau.len(env)?];
                    tableau.get_region(env, 0, &mut v)?;
                    Ok(depuis_le_tableau(&v))
                },
                "sorties",
            )
            .unwrap_or_default()
        }));
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

/// **Moins de mouvement** (GIF-3, fiche 159 § 8) : « Supprimer les animations » (accessibilité),
/// ou l'échelle des animations à zéro (options pour les développeurs) — `areAnimatorsEnabled`,
/// Android 8 et plus. Avant lui, la méthode manque : son exception est rendue, et rien ne se lit.
fn mouvement_reduit() -> Option<bool> {
    java::sur_le_fil(
        |env| {
            env.call_static_method(
                jni::jni_str!("android/animation/ValueAnimator"),
                jni::jni_str!("areAnimatorsEnabled"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        },
        "animations du système",
    )
    .map(|animees| !animees)
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
        /// L'activité, comme objet de Java.
        pub fn objet<'l>(self, env: &mut Env<'l>) -> JObject<'l> {
            // SAFETY : une référence globale à l'activité, qu'`android-activity` garde tant
            // qu'elle vit ; `JObject` ne la libère pas en partant.
            unsafe { JObject::from_raw(env, self.0 as jni::sys::jobject) }
        }

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

        /// Appelle une méthode `(String, String)V` de `MainActivity`, avec ces deux chaînes.
        pub fn appeler_avec_deux(
            self,
            methode: &'static JNIStr,
            (premiere, seconde): (&str, &str),
            quoi: &str,
        ) {
            sur_le_fil(
                |env| {
                    // SAFETY : comme dans `appeler`, la référence globale qu'`android-activity`
                    // garde à l'activité tant qu'elle vit.
                    let activite = unsafe { JObject::from_raw(env, self.0 as jni::sys::jobject) };
                    let a = env.new_string(premiere)?;
                    let b = env.new_string(seconde)?;
                    env.call_method(
                        &activite,
                        methode,
                        jni::jni_sig!("(Ljava/lang/String;Ljava/lang/String;)V"),
                        &[jni::JValue::from(&a), jni::JValue::from(&b)],
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
