//! **Le presse-papiers d'Android** (fiche 171) : `MainActivity.ecrireLePressePapiers`,
//! `lireLEtiquette` et `lireLePressePapiers`, branchés dans la porte de `glucose-desktop` — qui
//! décide de tout.

use glucose_desktop::plateforme::presse_papiers::telephone::{installer, Lu, Ouvert, Porte};
use jni::objects::{JLongArray, JObject, JObjectArray, JString};
use jni::Env;
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use winit::platform::android::activity::AndroidApp;

struct DuTelephone(super::java::Activite);

/// Une chaîne de Java, ou `null`.
fn chaine<'l>(env: &mut Env<'l>, texte: Option<&str>) -> jni::errors::Result<JObject<'l>> {
    texte.map_or(Ok(JObject::null()), |t| {
        env.new_string(t).map(JObject::from)
    })
}

/// Une chaîne rendue par Java, `null` compris.
fn lue(env: &mut Env, objet: JObject) -> jni::errors::Result<Option<String>> {
    let texte = env.cast_local::<JString>(objet)?;
    if texte.is_null() {
        return Ok(None);
    }
    texte.try_to_string(env).map(Some)
}

impl Porte for DuTelephone {
    fn ecrire(
        &self,
        etiquette: &str,
        texte: Option<&str>,
        image: Option<&Path>,
    ) -> Result<(), String> {
        let image = image.map(|i| i.to_string_lossy().into_owned());
        super::java::sur_le_fil(
            |env| {
                let activite = self.0.objet(env);
                let e = env.new_string(etiquette)?;
                let t = chaine(env, texte)?;
                let i = chaine(env, image.as_deref())?;
                env.call_method(
                    &activite,
                    jni::jni_str!("ecrireLePressePapiers"),
                    jni::jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                    &[
                        jni::JValue::from(&e),
                        jni::JValue::from(&t),
                        jni::JValue::from(&i),
                    ],
                )?;
                Ok(())
            },
            "presse-papiers",
        )
        .ok_or_else(|| "le presse-papiers d'Android a refuse la copie".to_string())
    }

    fn etiquette(&self) -> Option<String> {
        super::java::sur_le_fil(
            |env| {
                let activite = self.0.objet(env);
                let rendu = env
                    .call_method(
                        &activite,
                        jni::jni_str!("lireLEtiquette"),
                        jni::jni_sig!("()Ljava/lang/String;"),
                        &[],
                    )?
                    .l()?;
                lue(env, rendu)
            },
            "presse-papiers",
        )?
    }

    fn lire(&self, ouvrir: bool) -> Option<Lu> {
        super::java::sur_le_fil(
            |env| {
                let activite = self.0.objet(env);
                let rendu = env
                    .call_method(
                        &activite,
                        jni::jni_str!("lireLePressePapiers"),
                        jni::jni_sig!("(Z)[Ljava/lang/Object;"),
                        &[jni::JValue::from(ouvrir)],
                    )?
                    .l()?;
                let tableau = env.cast_local::<JObjectArray>(rendu)?;
                if tableau.is_null() {
                    return Ok(None);
                }
                let ouverts = tableau.get_element(env, 0)?;
                let ouverts = env.cast_local::<JLongArray>(ouverts)?;
                let mut v = vec![0; ouverts.len(env)?];
                ouverts.get_region(env, 0, &mut v)?;
                let texte = tableau.get_element(env, 1)?;
                let texte = lue(env, texte)?;
                Ok(Some(Lu {
                    fichiers: v.as_chunks::<3>().0.iter().map(ouvert).collect(),
                    texte,
                }))
            },
            "presse-papiers",
        )?
    }
}

/// Un fichier que Java a ouvert : son descripteur (-1 s'il n'a pas pu), son début, sa longueur
/// (-1 si le système ne la dit pas).
fn ouvert(&[fd, debut, longueur]: &[i64; 3]) -> Option<Ouvert> {
    let fd = i32::try_from(fd).ok().filter(|fd| *fd >= 0)?;
    // SAFETY : Java a détaché ce descripteur pour nous (`detachFd`) : il est à Rust seul, et
    // `File` le fermera.
    let fichier = unsafe { std::fs::File::from_raw_fd(fd) };
    Some((
        fichier,
        u64::try_from(debut).unwrap_or(0),
        u64::try_from(longueur).ok(),
    ))
}

/// Branche le presse-papiers de cette activité, avec le dossier que son fournisseur sert.
pub fn brancher(app: &AndroidApp, partages: PathBuf) {
    let porte = DuTelephone(super::java::Activite::de(app));
    installer(std::sync::Arc::new(porte), partages);
}
