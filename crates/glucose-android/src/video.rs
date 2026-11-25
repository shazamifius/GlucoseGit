//! **Les vidéos d'Android** (VIDEO-ANDROID-1, fiche 172) : `MainActivity.ouvrirUneVideo` et
//! `imageDUneVideo`, puis `LecteurVideo`, branchés dans la porte de `glucose-desktop` — qui
//! décide de tout (`plateforme::video`).

use glucose_desktop::plateforme::video::{
    image_fixe_de_java, installer, plans, Attente, ImageFixe, Lecteur, Ou, Plans, Porte, MESURES,
};
use jni::objects::{JByteBuffer, JIntArray, JObject, JObjectArray};
use jni::refs::Global;
use jni::{Env, JValue};
use std::time::Duration;
use winit::platform::android::activity::AndroidApp;

struct DuTelephone(super::java::Activite);

/// Un `LecteurVideo` de Java.
struct LecteurJava(Global<JObject<'static>>);

/// Le chemin, le début et la longueur (-1 : jusqu'au bout) d'une vidéo dans un fichier. Des
/// octets en mémoire ne passent pas encore cette porte (fiche 172 § 4).
fn fichier<'o>(ou: &'o Ou<'_>) -> Option<(&'o str, i64, i64)> {
    let Ou::Fichier {
        chemin,
        debut,
        longueur,
    } = ou
    else {
        return None;
    };
    let debut = i64::try_from(*debut).ok()?;
    let longueur = longueur.map_or(Some(-1), |l| i64::try_from(l).ok())?;
    Some((chemin.to_str()?, debut, longueur))
}

impl Porte for DuTelephone {
    fn image_fixe(&self, ou: &Ou<'_>) -> Option<ImageFixe> {
        let (chemin, debut, longueur) = fichier(ou)?;
        super::java::sur_le_fil(
            |env| {
                let activite = self.0.objet(env);
                let c: JObject = env.new_string(chemin)?.into();
                let rendu = env
                    .call_method(
                        &activite,
                        jni::jni_str!("imageDUneVideo"),
                        jni::jni_sig!("(Ljava/lang/String;JJ)[I"),
                        &[
                            JValue::from(&c),
                            JValue::Long(debut),
                            JValue::Long(longueur),
                        ],
                    )?
                    .l()?;
                let pixels = env.cast_local::<JIntArray>(rendu)?;
                if pixels.is_null() {
                    return Ok(None);
                }
                let mut v = vec![0; pixels.len(env)?];
                pixels.get_region(env, 0, &mut v)?;
                Ok(image_fixe_de_java(&v))
            },
            "video",
        )?
    }

    fn ouvrir(&self, ou: &Ou<'_>, avec_le_son: bool) -> Option<Box<dyn Lecteur>> {
        let (chemin, debut, longueur) = fichier(ou)?;
        let objet = super::java::sur_le_fil(
            |env| {
                let activite = self.0.objet(env);
                let c: JObject = env.new_string(chemin)?.into();
                let lecteur = env
                    .call_method(
                        &activite,
                        jni::jni_str!("ouvrirUneVideo"),
                        jni::jni_sig!("(Ljava/lang/String;JJZ)Lcom/glucose/app/LecteurVideo;"),
                        &[
                            JValue::from(&c),
                            JValue::Long(debut),
                            JValue::Long(longueur),
                            JValue::from(avec_le_son),
                        ],
                    )?
                    .l()?;
                if lecteur.is_null() {
                    return Ok(None);
                }
                env.new_global_ref(lecteur).map(Some)
            },
            "video",
        )??;
        Some(Box::new(LecteurJava(objet)))
    }
}

impl Lecteur for LecteurJava {
    fn attendre(&self, delai: Duration, voir: &mut dyn FnMut(&Plans<'_>)) -> Attente {
        let ms = i64::try_from(delai.as_millis()).unwrap_or(i64::MAX);
        super::java::sur_le_fil(
            |env| {
                let lecteur = self.0.as_obj();
                let code = env
                    .call_method(
                        lecteur,
                        jni::jni_str!("attendre"),
                        jni::jni_sig!("(J)I"),
                        &[JValue::Long(ms)],
                    )?
                    .i()?;
                let attente = match code {
                    0 => Attente::Rien,
                    1 => Attente::Image,
                    2 => Attente::AuBout,
                    _ => Attente::Faute,
                };
                if attente == Attente::Image {
                    preter(env, lecteur, voir)?;
                }
                Ok(attente)
            },
            "video",
        )
        .unwrap_or(Attente::Faute)
    }

    fn jouer(&self, oui: bool) {
        self.booleen(jni::jni_str!("jouer"), oui);
    }

    fn couper_le_son(&self, oui: bool) {
        self.booleen(jni::jni_str!("couperLeSon"), oui);
    }

    fn revenir_au_debut(&self) -> bool {
        super::java::sur_le_fil(
            |env| {
                env.call_method(
                    self.0.as_obj(),
                    jni::jni_str!("revenirAuDebut"),
                    jni::jni_sig!("()Z"),
                    &[],
                )?
                .z()
            },
            "video",
        )
        .unwrap_or(false)
    }
}

impl LecteurJava {
    /// Appelle une méthode `(boolean)V` du lecteur.
    fn booleen(&self, methode: &'static jni::strings::JNIStr, oui: bool) {
        super::java::sur_le_fil(
            |env| {
                env.call_method(
                    self.0.as_obj(),
                    methode,
                    jni::jni_sig!("(Z)V"),
                    &[JValue::from(oui)],
                )?;
                Ok(())
            },
            "video",
        );
    }
}

/// **Prête l'image que Java tient** à `voir`, le temps de la lire : ses tampons sont ceux du
/// lecteur, valides jusqu'à l'attente suivante. Des nombres qui ne se lisent pas : `voir` n'est
/// pas appelé, et l'image est une faute (`telephone::issue`).
fn preter(
    env: &mut Env,
    lecteur: &JObject,
    voir: &mut dyn FnMut(&Plans<'_>),
) -> jni::errors::Result<()> {
    let tenue = env
        .call_method(
            lecteur,
            jni::jni_str!("tenue"),
            jni::jni_sig!("()[Ljava/lang/Object;"),
            &[],
        )?
        .l()?;
    let tableau = env.cast_local::<JObjectArray>(tenue)?;
    let mesures = tableau.get_element(env, 0)?;
    let mesures = env.cast_local::<JIntArray>(mesures)?;
    let mut m = [0; MESURES];
    if mesures.len(env)? != MESURES {
        return Ok(());
    }
    mesures.get_region(env, 0, &mut m)?;
    let mut tampons: [&[u8]; 3] = [&[]; 3];
    for (i, tampon) in tampons.iter_mut().enumerate() {
        let b = tableau.get_element(env, i + 1)?;
        let b = env.cast_local::<JByteBuffer>(b)?;
        let adresse = env.get_direct_buffer_address(&b)?;
        let capacite = env.get_direct_buffer_capacity(&b)?;
        if adresse.is_null() {
            return Ok(());
        }
        // SAFETY : le tampon direct d'un plan de l'image que Java tient ; il vit jusqu'à
        // l'attente suivante (`lacher`), bien après `voir`, qui ne le garde pas.
        *tampon = unsafe { std::slice::from_raw_parts(adresse, capacite) };
    }
    if let Some(p) = plans(&m, tampons) {
        voir(&p);
    }
    Ok(())
}

impl Drop for LecteurJava {
    fn drop(&mut self) {
        super::java::sur_le_fil(
            |env| {
                env.call_method(
                    self.0.as_obj(),
                    jni::jni_str!("fermer"),
                    jni::jni_sig!("()V"),
                    &[],
                )?;
                Ok(())
            },
            "video",
        );
    }
}

/// Branche les vidéos de cette activité.
pub fn brancher(app: &AndroidApp) {
    installer(std::sync::Arc::new(DuTelephone(super::java::Activite::de(
        app,
    ))));
}
