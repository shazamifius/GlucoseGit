//! **Ce qu'une autre application partage vers Glucose** (PARTAGE-1, fiche 56) : l'activité
//! Java a ouvert les fichiers, Rust les lit et les confie au dépôt de Glucose.

use glucose_desktop::plateforme::partage::{recevoir, Partage};
use jni::objects::{JClass, JIntArray, JLongArray, JObjectArray, JString};
use jni::refs::Reference;
use jni::{Env, EnvUnowned};
use std::os::fd::FromRawFd;

/// Les arguments de Java, copiés en Rust : les descripteurs, leurs débuts, leurs
/// longueurs, leurs noms (RECU-2), le texte.
type Arguments = (
    Vec<i32>,
    Vec<i64>,
    Vec<i64>,
    Vec<Option<String>>,
    Option<String>,
);

/// `MainActivity.recevoirUnPartage`, sur le fil de Java qui a ouvert les fichiers — jamais
/// celui de l'interface : la lecture peut prendre son temps.
#[no_mangle]
pub extern "system" fn Java_com_glucose_app_MainActivity_recevoirUnPartage<'l>(
    mut env: EnvUnowned<'l>,
    _classe: JClass<'l>,
    fichiers: JIntArray<'l>,
    debuts: JLongArray<'l>,
    longueurs: JLongArray<'l>,
    noms: JObjectArray<'l, JString<'l>>,
    texte: JString<'l>,
) {
    let lus = env
        .with_env(|env| -> jni::errors::Result<_> {
            Ok(copier(env, (&fichiers, &debuts, &longueurs, &noms), &texte)
                .map_err(|e| eprintln!("[Glucose] partage : {e}"))
                .ok())
        })
        .resolve::<jni::errors::LogErrorAndDefault>();
    // Un partage dont les arguments ne se lisent pas laisse ses descripteurs ouverts : ils
    // partent avec le processus. Il n'arrive que si Java et Rust ne s'accordent plus.
    let Some((fichiers, debuts, longueurs, noms, texte)) = lus else {
        return;
    };
    let mut partage = Partage {
        texte,
        ..Partage::default()
    };
    let ouverts = fichiers.into_iter().zip(debuts).zip(longueurs);
    for (((fd, debut), longueur), nom) in ouverts.zip(noms) {
        let ouvert = (fd >= 0).then(|| {
            // SAFETY : Java a détaché ce descripteur pour nous (`detachFd`) : il est à
            // Rust seul, et `File` le fermera.
            let fichier = unsafe { std::fs::File::from_raw_fd(fd) };
            let longueur = u64::try_from(longueur).ok();
            (fichier, u64::try_from(debut).unwrap_or(0), longueur)
        });
        partage.ajouter(ouvert, nom.as_deref());
    }
    println!(
        "[Glucose] partage : {} image(s) lue(s), {} fichier(s) copie(s), {} illisible(s), {}",
        partage.recus.len(),
        partage.chemins.len(),
        partage.illisibles,
        if partage.texte.is_some() {
            "un texte"
        } else {
            "sans texte"
        }
    );
    recevoir(partage);
}

/// `MainActivity.connaitreLeDossier`, avant que le fil de Glucose naisse (VIDEO-ANDROID-2,
/// fiche 172 § 5) : où va ce qu'il reçoit et qui n'est pas une image — le dossier privé,
/// celui de ses documents, jamais le cache que le système vide.
#[no_mangle]
pub extern "system" fn Java_com_glucose_app_MainActivity_connaitreLeDossier<'l>(
    mut env: EnvUnowned<'l>,
    _classe: JClass<'l>,
    dossier: JString<'l>,
) {
    let lu = env
        .with_env(|env| -> jni::errors::Result<String> { dossier.try_to_string(env) })
        .resolve::<jni::errors::LogErrorAndDefault>();
    if !lu.is_empty() {
        let application = std::path::Path::new(&lu).join(super::DOSSIER);
        glucose_desktop::plateforme::telechargements::habiter(&application);
    }
}

/// Les tableaux et le texte de Java, en Rust.
fn copier(
    env: &mut Env,
    (fichiers, debuts, longueurs, noms): Tableaux,
    texte: &JString,
) -> jni::errors::Result<Arguments> {
    let n = fichiers.len(env)?;
    let (mut f, mut d, mut l) = (vec![0; n], vec![0; n], vec![0; n]);
    fichiers.get_region(env, 0, &mut f)?;
    debuts.get_region(env, 0, &mut d)?;
    longueurs.get_region(env, 0, &mut l)?;
    let mut lus = Vec::with_capacity(n);
    for i in 0..n {
        let nom = noms.get_element(env, i)?;
        lus.push(texte_ou_rien(env, &nom)?);
    }
    Ok((f, d, l, lus, texte_ou_rien(env, texte)?))
}

/// Les tableaux de Java : descripteurs, débuts, longueurs, noms.
type Tableaux<'a, 'l> = (
    &'a JIntArray<'l>,
    &'a JLongArray<'l>,
    &'a JLongArray<'l>,
    &'a JObjectArray<'l, JString<'l>>,
);

/// Un texte de Java, ou rien s'il est nul.
fn texte_ou_rien(env: &mut Env, texte: &JString) -> jni::errors::Result<Option<String>> {
    match texte.is_null() {
        true => Ok(None),
        false => texte.try_to_string(env).map(Some),
    }
}
