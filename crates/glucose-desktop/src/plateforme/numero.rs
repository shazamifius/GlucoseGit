//! **Le numéro qu'un système donne à un fichier** (RECENTS-1, fiche 174) : ce qui le désigne quel
//! que soit son nom ou son dossier, tant qu'il reste sur le même disque.
//!
//! Un document qu'on déplace ou qu'on renomme garde son numéro : c'est par lui que la liste des
//! documents le retrouve à sa nouvelle place. Sous Linux, Android et le Mac, le disque et l'inode
//! (`stat`) ; sous Windows, le numéro de série du volume et l'index du fichier
//! (`GetFileInformationByHandle`, que le suivi des liens de Windows emploie de même —
//! learn.microsoft.com/windows/win32/fileio/distributed-link-tracking-and-object-identifiers).
//!
//! **Un inode libéré resservit aussitôt** : un document effacé, puis un autre créé, et le second
//! portait le numéro du premier (vu par la suite entière, le 26/11). Sous Unix, le numéro porte donc
//! aussi l'instant où le fichier est né (`statx`, sa date de naissance) : déplacé ou renommé, un
//! fichier garde la sienne ; un inode réutilisé naît de nouveau. Sous Windows, l'index du fichier
//! porte déjà, dans ses seize bits hauts, le compte de ses réemplois (NTFS).
//!
//! Sa limite, dite : copié puis effacé (vers une clé USB, un autre disque), un fichier change de
//! numéro. La liste ne s'y fie donc que pour un document introuvable, et seulement vers un autre
//! document de Glucose.

use std::path::Path;

/// **Le numéro de ce fichier**, écrit en texte (`disque-numéro`) ; `None` s'il n'existe pas, ou si
/// le système n'en donne aucun.
pub fn de(chemin: &Path) -> Option<String> {
    systeme(chemin)
}

#[cfg(unix)]
fn systeme(chemin: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let m = std::fs::metadata(chemin).ok()?;
    let numero = format!("{}-{}", m.dev(), m.ino());
    // Un système qui ne note pas les naissances n'a que l'inode à donner.
    let ne = m
        .created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
    Some(match ne {
        Some(ne) => format!("{numero}-{}", ne.as_nanos()),
        None => numero,
    })
}

#[cfg(windows)]
fn systeme(chemin: &Path) -> Option<String> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    // Ouvert en lecture, partagé : un document que Glucose écrit en ce moment s'ouvre aussi.
    let fichier = std::fs::File::open(chemin).ok()?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY : la poignée vient d'un fichier ouvert, vivant jusqu'à la fin de cette fonction ;
    // l'appel n'écrit que dans `info`, qui nous appartient.
    unsafe { GetFileInformationByHandle(HANDLE(fichier.as_raw_handle()), &mut info) }.ok()?;
    let index = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
    Some(format!("{}-{index}", info.dwVolumeSerialNumber))
}

#[cfg(not(any(unix, windows)))]
fn systeme(_: &Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Déplacé ou renommé, un fichier garde son numéro** ; un autre fichier en a un autre.
    #[test]
    fn recents_1_un_fichier_deplace_garde_son_numero() {
        let d = std::env::temp_dir().join(format!("glucose-numero-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("ailleurs")).expect("le dossier");
        let (avant, apres, autre) = (
            d.join("a.glucose"),
            d.join("ailleurs/b.glucose"),
            d.join("c"),
        );
        std::fs::write(&avant, b"un").expect("le fichier");
        std::fs::write(&autre, b"deux").expect("l'autre");
        let numero = de(&avant).expect("un numéro");
        crate::persist::atomic::renommer_sans_ecraser(&avant, &apres).expect("déplacé");
        assert_eq!(de(&apres), Some(numero.clone()), "il le garde");
        assert_ne!(
            de(&autre),
            Some(numero.clone()),
            "un autre fichier, un autre numéro"
        );
        assert_eq!(de(&avant), None, "plus rien à son ancienne place");
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = std::fs::metadata(&apres).expect("lu");
            let parts: Vec<String> = numero.split('-').map(str::to_string).collect();
            assert_eq!(parts[1], m.ino().to_string(), "il nomme l'inode");
            // Un inode réutilisé naît de nouveau : là où le système note les naissances, le numéro
            // porte la sienne.
            if m.created().is_ok() {
                assert_eq!(parts.len(), 3, "et sa naissance : {numero}");
            }
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}
