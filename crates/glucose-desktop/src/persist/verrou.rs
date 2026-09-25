//! **Un seul scribe par fichier.**
//!
//! Deux fenêtres qui ajouteraient chacune sa suite au même document casseraient sa chaîne à
//! la première écriture de la seconde : tout ce qu'elle écrirait ensuite serait perdu à la
//! relecture. Le fichier d'un document s'ouvre donc pour y écrire **seul** — sous Windows en
//! ne partageant que la lecture, ailleurs sous un verrou exclusif (`flock`, consultatif : il
//! n'arrête que ceux qui le demandent, c'est-à-dire les autres Glucose). Lire reste permis à
//! tous : la Time Machine, une autre fenêtre qui ouvre le document pour le regarder.
//!
//! Une fenêtre qui trouve le document tenu l'ouvre comme un fichier en lecture seule : ses
//! changements vont dans un brouillon, et l'ouverture dit pourquoi.

use std::fs::{File, OpenOptions};
use std::path::Path;

/// Ouvre ce fichier pour y écrire, seul. Rien n'y est vidé ni écrit ici : un fichier qu'une
/// autre fenêtre tient ne doit pas être touché par celle-ci — hors de Windows, le verrou ne
/// se prend qu'une fois le fichier ouvert.
pub fn ouvrir_seul(o: &mut OpenOptions, chemin: &Path) -> Result<File, String> {
    o.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        /// `FILE_SHARE_READ` : les autres lisent, personne d'autre n'écrit.
        const LECTURE_SEULE: u32 = 1;
        o.share_mode(LECTURE_SEULE);
    }
    let f = o.open(chemin).map_err(|e| dire(chemin, &e))?;
    #[cfg(not(windows))]
    if let Err(std::fs::TryLockError::WouldBlock) = f.try_lock() {
        return Err(deja_tenu(chemin));
    }
    Ok(f)
}

/// Un autre Glucose écrit-il dans ce fichier ? Sous Windows, s'ouvrir sans aucun partage
/// échoue tant qu'un autre le tient.
#[cfg(windows)]
pub fn tenu_ailleurs(chemin: &Path) -> bool {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(chemin)
        .is_err()
}

/// Un autre Glucose écrit-il dans ce fichier ? Ailleurs qu'à Windows, son verrou le dit.
#[cfg(not(windows))]
pub fn tenu_ailleurs(chemin: &Path) -> bool {
    File::open(chemin).is_ok_and(|f| matches!(f.try_lock(), Err(std::fs::TryLockError::WouldBlock)))
}

/// **Une autre fenêtre de Glucose écrit-elle ce document ?** L'épreuve même que passerait son
/// scribe — l'ouvrir seul en écriture, puis le lâcher —, si bien qu'un lecteur ordinaire (un
/// antivirus, l'aperçu de l'Explorateur) ne la fait pas échouer : seul un autre écrivain.
pub fn ecrit_ailleurs(chemin: &Path) -> bool {
    matches!(
        ouvrir_seul(&mut OpenOptions::new(), chemin),
        Err(e) if e == deja_tenu(chemin)
    )
}

/// Ces deux chemins désignent-ils le même fichier ? `C:\x` et `c:/x` aussi.
pub fn meme_fichier(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Ce qu'un fichier qui ne s'ouvre pas en écriture veut dire, dans les mots de l'utilisateur.
pub fn dire(chemin: &Path, e: &std::io::Error) -> String {
    /// `ERROR_SHARING_VIOLATION` : un autre le tient déjà.
    const PARTAGE_REFUSE: i32 = 32;
    if cfg!(windows) && e.raw_os_error() == Some(PARTAGE_REFUSE) {
        return deja_tenu(chemin);
    }
    format!("{} : {e}", chemin.display())
}

pub fn deja_tenu(chemin: &Path) -> String {
    format!(
        "{} est déjà ouvert en écriture ailleurs — dans une autre fenêtre de Glucose ?",
        chemin.display()
    )
}
