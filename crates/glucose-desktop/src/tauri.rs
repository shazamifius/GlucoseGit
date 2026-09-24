//! Les documents de **Glucose Tauri** vus depuis le bureau : retrouver les octets de leurs
//! images sur le disque.
//!
//! Tout ce qui se lit sans disque — le binaire Automerge, le JSON d'avant, la traduction en
//! projet, ce qu'un `asset:` désigne — vit dans le noyau
//! ([`glucose_core::persist::tauri`]). Il ne reste ici que ce qui touche au système de
//! fichiers : le magasin global de Tauri sur cette machine, le dossier `objects/` d'un
//! document portable, et la vérification de chaque octet relu contre son empreinte — le nom
//! d'un fichier du magasin **est** le début du SHA-256 de son contenu, et une image abîmée
//! sur le disque se dit, elle n'est jamais posée à la place de la bonne.

use glucose_core::hash::{hex_of, sha256};
use glucose_core::persist::tauri::projet::Provenance;
use std::path::{Path, PathBuf};

/// Le magasin global de Glucose Tauri sur cette machine (`app_data_dir` de l'identifiant
/// `com.glucose.app`, fixé par `tauri.conf.json`).
pub fn magasin_tauri() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    };
    Some(base?.join("com.glucose.app").join("assets"))
}

/// Les octets d'une image, lus et vérifiés.
///
/// `dossier` est celui du document : c'est là que vivent `objects/` (document portable) et
/// les chemins relatifs. Le magasin global se passe en paramètre plutôt que de se deviner,
/// pour que les épreuves l'aient sous la main.
pub fn resoudre(
    provenance: &Provenance,
    dossier: Option<&Path>,
    magasin: Option<&Path>,
) -> Result<Vec<u8>, String> {
    match provenance {
        Provenance::Octets(o) => Ok(o.clone()),
        Provenance::Magasin(nom) => {
            let candidats = dossier
                .map(|d| d.join("objects").join(nom))
                .into_iter()
                .chain(magasin.map(|m| m.join(nom)));
            // Le premier exemplaire **vérifié** l'emporte : une copie abîmée dans `objects/`
            // ne masque pas la bonne du magasin global.
            let mut abimee = None;
            for chemin in candidats {
                if let Ok(octets) = std::fs::read(&chemin) {
                    match verifier(nom, &octets) {
                        Ok(()) => return Ok(octets),
                        Err(e) => abimee = Some(e),
                    }
                }
            }
            Err(abimee.unwrap_or_else(|| {
                format!("« {nom} » introuvable dans le magasin de Glucose Tauri")
            }))
        }
        Provenance::Chemin(p) => {
            let chemin = Path::new(p);
            let chemin = match dossier {
                Some(d) if chemin.is_relative() => d.join(chemin),
                _ => chemin.to_path_buf(),
            };
            std::fs::read(&chemin).map_err(|e| format!("{} : {e}", chemin.display()))
        }
        Provenance::Web(url) => Err(format!("{url} : image web, non rapatriée")),
        Provenance::Inconnue => Err("aucune source d'octets".to_string()),
    }
}

/// Le nom d'un fichier du magasin commence par l'empreinte de son contenu.
fn verifier(nom: &str, octets: &[u8]) -> Result<(), String> {
    let souche = nom.split('.').next().unwrap_or("").to_ascii_lowercase();
    if !souche.is_empty() && hex_of(&sha256(octets)).starts_with(&souche) {
        Ok(())
    } else {
        Err(format!(
            "« {nom} » ne correspond plus à son empreinte : le fichier a été abîmé"
        ))
    }
}
