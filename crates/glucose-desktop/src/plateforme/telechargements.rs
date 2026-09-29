//! **Le dossier des téléchargements de l'utilisateur** : là où un navigateur range ce qu'on
//! télécharge, et où Glucose range ce qu'une page lui livre et qui n'est pas une image
//! (DEPOT-4).
//!
//! # Pourquoi là, et plus dans le dossier temporaire
//!
//! Un fichier glissé depuis une page — un PDF, une archive — devient une tuile qui **mène** à
//! lui : il lui faut un vrai fichier, qui dure. Il s'écrivait dans le dossier temporaire du
//! système, que Windows vide : la tuile aurait fini par ne plus mener nulle part. Une image,
//! elle, n'a besoin d'aucun fichier — ses octets entrent dans le document.
//!
//! Les téléchargements sont ce qu'un navigateur aurait fait du même fichier : l'utilisateur l'y
//! retrouve, il lui appartient, et Glucose ne l'efface jamais. Un nom déjà pris n'est jamais
//! écrasé : `rapport (1).pdf`, comme un navigateur.

use super::moisson::Recu;
use std::path::{Path, PathBuf};

/// Le dossier des téléchargements de cet utilisateur, tel que le système le connaît.
#[cfg(windows)]
pub fn dossier() -> Option<PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Downloads, SHGetKnownFolderPath, KNOWN_FOLDER_FLAG};
    // SAFETY : l'appel ne lit que ses arguments ; la chaîne qu'il rend est à nous, et se rend
    // au système par `CoTaskMemFree`, une fois copiée.
    unsafe {
        let chaine = SHGetKnownFolderPath(&FOLDERID_Downloads, KNOWN_FOLDER_FLAG(0), None).ok()?;
        let chemin = chaine.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(chaine.0 as *const core::ffi::c_void));
        chemin
    }
}

/// Le dossier des téléchargements de cet utilisateur : celui que `user-dirs.dirs` nomme —
/// il est traduit (« Téléchargements » sur un bureau français) —, sinon `~/Downloads`.
#[cfg(not(windows))]
pub fn dossier() -> Option<PathBuf> {
    let personnel = PathBuf::from(std::env::var_os("HOME")?);
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| personnel.join(".config"));
    let nomme = std::fs::read_to_string(config.join("user-dirs.dirs"))
        .ok()
        .and_then(|texte| dossier_nomme(&texte, &personnel));
    Some(nomme.unwrap_or_else(|| personnel.join("Downloads")))
}

/// La ligne `XDG_DOWNLOAD_DIR="$HOME/…"` d'un `user-dirs.dirs`, lue. Pure, pour s'éprouver.
#[cfg(not(windows))]
fn dossier_nomme(texte: &str, personnel: &Path) -> Option<PathBuf> {
    let valeur = texte
        .lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("XDG_DOWNLOAD_DIR="))?
        .trim_matches('"');
    match valeur.strip_prefix("$HOME") {
        Some(reste) => Some(personnel.join(reste.trim_start_matches('/'))),
        None => Some(PathBuf::from(valeur)).filter(|p| p.is_absolute()),
    }
}

/// Pose ce reçu dans ce dossier, sous son nom, ou `nom (1).ext`, `nom (2).ext`… s'il est pris :
/// un fichier de l'utilisateur ne s'écrase jamais.
///
/// Le nom se **réserve** d'abord — créé vide, et seulement s'il n'existait pas —, puis le
/// contenu y est posé d'un bloc ([`crate::persist::atomic`]) : un arrêt en chemin laisse un
/// fichier vide, jamais un fichier à moitié écrit qui passerait pour entier.
pub fn poser_dans(dossier: &Path, recu: &Recu) -> Option<PathBuf> {
    std::fs::create_dir_all(dossier).ok()?;
    let nom = Path::new(&recu.nom);
    let souche = nom
        .file_stem()
        .map_or_else(|| recu.nom.clone(), |s| s.to_string_lossy().into());
    let extension = nom.extension().map(|e| format!(".{}", e.to_string_lossy()));
    for rang in 0usize.. {
        let essai = match rang {
            0 => recu.nom.clone(),
            n => format!("{souche} ({n}){}", extension.as_deref().unwrap_or("")),
        };
        let chemin = dossier.join(essai);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&chemin)
        {
            Ok(reserve) => {
                drop(reserve);
                return crate::persist::atomic::ecrire_d_un_bloc(&chemin, &recu.octets)
                    .ok()
                    .map(|()| chemin);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dossier_d_epreuve(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "glucose-telechargements-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// **Un nom pris ne s'écrase jamais** : le second prend `(1)`, comme chez un navigateur, et
    /// le premier garde ses octets.
    #[test]
    fn test_un_nom_pris_ne_s_ecrase_jamais() {
        let d = dossier_d_epreuve("pris");
        let a = Recu::nouveau("rapport.pdf", b"premier".to_vec()).expect("un reçu");
        let b = Recu::nouveau("rapport.pdf", b"second".to_vec()).expect("un reçu");
        let pa = poser_dans(&d, &a).expect("posé");
        let pb = poser_dans(&d, &b).expect("posé à côté");
        assert_eq!(pa, d.join("rapport.pdf"));
        assert_eq!(pb, d.join("rapport (1).pdf"));
        assert_eq!(std::fs::read(&pa).unwrap(), b"premier");
        assert_eq!(std::fs::read(&pb).unwrap(), b"second");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Le dossier que `user-dirs.dirs` nomme — traduit, relatif à `$HOME` ou absolu.
    #[cfg(not(windows))]
    #[test]
    fn test_le_dossier_nomme_se_lit_dans_user_dirs() {
        let perso = Path::new("/home/moi");
        let texte = "# commentaire\nXDG_DESKTOP_DIR=\"$HOME/Bureau\"\nXDG_DOWNLOAD_DIR=\"$HOME/Téléchargements\"\n";
        assert_eq!(
            dossier_nomme(texte, perso),
            Some(PathBuf::from("/home/moi/Téléchargements"))
        );
        assert_eq!(
            dossier_nomme("XDG_DOWNLOAD_DIR=\"/data/dl\"", perso),
            Some(PathBuf::from("/data/dl"))
        );
        assert_eq!(dossier_nomme("XDG_DOWNLOAD_DIR=\"relatif\"", perso), None);
        assert_eq!(dossier_nomme("", perso), None);
    }
}
