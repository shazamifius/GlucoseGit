//! **Ce qui part, et comment** : les sessions closes de la boîte noire, une à une, sur un fil à
//! part, au lancement — jamais pendant qu'on dessine (fiche 49 § 2.4).
//!
//! Une session partie se retient dans un fichier à côté de l'accord ; une session que le serveur
//! refuse aussi — la renvoyer à chaque lancement ne la ferait pas accepter. Un réseau absent, en
//! revanche, n'est qu'un report : on réessaiera au lancement suivant.

use crate::plateforme::Verbe;
use std::path::{Path, PathBuf};

/// Les sessions déjà parties (ou refusées), une par ligne, à côté de l'accord.
pub const ENVOYEES: &str = "telemetrie-envoyees.txt";

/// **Les sessions à envoyer** : les fichiers de la boîte noire, sauf celle qui s'écrit et celles
/// déjà parties, des plus anciennes aux plus récentes.
pub fn a_envoyer(noms: &[String], courante: Option<&str>, envoyees: &str) -> Vec<String> {
    let parties: Vec<&str> = envoyees.lines().map(str::trim).collect();
    let mut restent: Vec<String> = noms
        .iter()
        .filter(|n| n.ends_with(".jsonl"))
        .filter(|n| Some(n.as_str()) != courante)
        .filter(|n| !parties.contains(&n.as_str()))
        .cloned()
        .collect();
    restent.sort();
    restent
}

/// **L'identifiant d'une session** pour le serveur : rien du nom du fichier ne part (il porte
/// l'heure et le numéro du processus) — son empreinte avec l'installation, 32 chiffres.
pub fn identifiant_de_session(installation: &str, nom: &str) -> String {
    let empreinte = glucose_core::hash::sha256(format!("{installation}/{nom}").as_bytes());
    glucose_core::hash::hex_of(&empreinte)[..32].to_string()
}

/// **Le corps envoyé** : l'installation, la session, et ses lignes — **seulement** celles qui
/// sont du JSON entier. Une session coupée par un plantage finit sur une ligne déchirée, que le
/// serveur refuserait avec tout le reste. `None` si rien ne commence par la ligne de début.
pub fn corps(installation: &str, nom: &str, texte: &str) -> Option<String> {
    use glucose_core::persist::tauri::json;
    let lignes: Vec<&str> = texte.lines().filter(|l| json::lire(l).is_ok()).collect();
    let premiere = json::lire(lignes.first()?).ok()?;
    if premiere.texte("type") != Some("debut") {
        return None;
    }
    let mut tout = lignes.join("\n");
    tout.push('\n');
    Some(format!(
        "{{\"installation\":\"{installation}\",\"session\":\"{}\",\"lignes\":\"{}\"}}",
        identifiant_de_session(installation, nom),
        crate::boite_noire::enregistrement::echapper(&tout)
    ))
}

/// **Envoie, sur un fil à part, ce qui attend.**
pub fn envoyer_en_fond(
    adresse: String,
    dossier: PathBuf,
    installation: String,
    courante: Option<PathBuf>,
) {
    let _ = std::thread::Builder::new()
        .name("telemetrie".into())
        .spawn(move || envoyer_tout(&adresse, &dossier, &installation, courante.as_deref()));
}

fn envoyer_tout(adresse: &str, dossier: &Path, installation: &str, courante: Option<&Path>) {
    let boite = dossier.join(crate::boite_noire::DOSSIER);
    let noms: Vec<String> = std::fs::read_dir(&boite)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let courante = courante
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned());
    let envoyees = elaguer(dossier, &noms);
    let url = format!("{adresse}/v1/sessions");
    for nom in a_envoyer(&noms, courante.as_deref(), &envoyees) {
        let Ok(texte) = std::fs::read_to_string(boite.join(&nom)) else {
            continue;
        };
        let reponse = match corps(installation, &nom, &texte) {
            Some(c) => crate::plateforme::envoyer(&url, Verbe::Deposer, c.as_bytes()),
            None => Ok(0),
        };
        match reponse {
            Ok(200 | 201) => retenir(dossier, &nom, "partie"),
            Ok(code) => {
                println!("[Glucose] journal technique : {nom} refuse ({code})");
                retenir(dossier, &nom, "refusee");
            }
            // Le réseau manque : la suite attendra le lancement suivant.
            Err(e) => {
                println!("[Glucose] journal technique : envoi remis ({e})");
                return;
            }
        }
    }
}

/// Retient qu'une session est partie (ou refusée) : elle ne repartira plus.
fn retenir(dossier: &Path, nom: &str, comment: &str) {
    use std::io::Write;
    let ecrit = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dossier.join(ENVOYEES))
        .and_then(|mut f| writeln!(f, "{nom}"));
    if ecrit.is_ok() {
        println!("[Glucose] journal technique : {nom} {comment}");
    }
}

/// **Efface, sur un fil à part, tout ce que cette installation a envoyé.**
pub fn effacer_en_fond(adresse: String, installation: String) {
    let _ = std::thread::Builder::new()
        .name("telemetrie".into())
        .spawn(move || {
            let url = format!("{adresse}/v1/installations/{installation}");
            match crate::plateforme::envoyer(&url, Verbe::Effacer, &[]) {
                Ok(200) => println!("[Glucose] journal technique : ce qui est parti est efface"),
                autre => println!("[Glucose] journal technique : effacement rate ({autre:?})"),
            }
        });
}

/// **La liste de ce qui est parti, sans ce que la boîte noire a déjà effacé** : elle ne grandit
/// pas plus que le dossier. Elle ne s'oublie jamais autrement — une session partie sous un
/// ancien identifiant ne doit pas repartir sous le nouveau.
fn elaguer(dossier: &Path, noms: &[String]) -> String {
    let chemin = dossier.join(ENVOYEES);
    let envoyees = std::fs::read_to_string(&chemin).unwrap_or_default();
    let gardees: String = envoyees
        .lines()
        .filter(|l| noms.iter().any(|n| n == l.trim()))
        .map(|l| {
            format!(
                "{}
",
                l.trim()
            )
        })
        .collect();
    if gardees != envoyees {
        let _ = std::fs::write(&chemin, &gardees);
    }
    gardees
}
