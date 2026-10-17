//! **La porte sous Linux et macOS** : une prise Unix (*Unix domain socket*), dans un dossier que
//! seul l'utilisateur peut ouvrir (AGENTS-2) — `$XDG_RUNTIME_DIR`, que le système crée ainsi,
//! ou à défaut un dossier `porte` du dossier de Glucose, créé en `0700`. Personne d'autre ne peut
//! y poser une prise à la place de la nôtre, ni ouvrir la nôtre.

use crate::agents::Boite;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

/// **La porte ouverte** ; sa prise disparaît avec elle.
pub struct Porte {
    nom: String,
}

impl Porte {
    /// **Ouvre la porte** — `None` si un autre Glucose la tient déjà (sa prise répond).
    pub fn ouvrir(nom: &str, boite: Boite) -> Option<Self> {
        let chemin = Path::new(nom);
        if UnixStream::connect(chemin).is_ok() {
            return None;
        }
        // Une prise qui ne répond pas est celle d'un Glucose qui a fini sans la retirer.
        let _ = std::fs::remove_file(chemin);
        let ecoute = UnixListener::bind(chemin).ok()?;
        let _ = std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o600));
        std::thread::Builder::new()
            .name("glucose-porte".into())
            .spawn(move || {
                for flux in ecoute.incoming().flatten() {
                    let b = boite.clone();
                    let _ = std::thread::Builder::new()
                        .name("glucose-agent".into())
                        .spawn(move || super::servir(&flux, &flux, &b));
                }
            })
            .ok()?;
        Some(Self {
            nom: nom.to_string(),
        })
    }

    pub fn nom(&self) -> &str {
        &self.nom
    }
}

impl Drop for Porte {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.nom);
    }
}

/// **Le chemin de la prise de cet utilisateur.**
pub fn nom_par_defaut() -> Option<String> {
    let dossier = match std::env::var_os("XDG_RUNTIME_DIR").filter(|d| !d.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => {
            let d = crate::present::souvenir::dossier().join("porte");
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(&d)
                .ok()?;
            // Un dossier qui existait déjà, ouvert aux autres : on n'y met rien.
            let mode = std::fs::metadata(&d).ok()?.permissions().mode();
            if mode & 0o077 != 0 {
                return None;
            }
            d
        }
    };
    Some(
        dossier
            .join("glucose-agents.sock")
            .to_string_lossy()
            .into_owned(),
    )
}

/// **Se connecte à la porte.**
pub fn connecter(nom: &str) -> std::io::Result<UnixStream> {
    UnixStream::connect(nom)
}
