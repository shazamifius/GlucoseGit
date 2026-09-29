//! **Comment ce programme est installé**, et donc comment il se remplace (fiche 48).
//!
//! L'updater de Glucose Tauri connaît quatre formes, et les nomme dans `latest.json` : l'installeur
//! NSIS sous Windows ; sous Linux l'AppImage, le paquet `.deb` et le paquet `.rpm`. Glucose Rust
//! les reprend, avec leurs noms — une bascule sans couture, et un seul fichier des versions.
//!
//! * **NSIS** : l'installeur se lance quand Glucose se ferme, et attend qu'il le soit.
//! * **AppImage** : le fichier lui-même, reconnu par la variable `APPIMAGE` que son lanceur pose,
//!   est remplacé d'un bloc — exécutable avant de prendre sa place.
//! * **`.deb`, `.rpm`** : l'exécutable appartient à un paquet du système (`dpkg-query -S`,
//!   `rpm -qf`) ; le nouveau paquet s'installe par `pkexec`, comme chez Tauri.
//!
//! Un Glucose installé autrement — par NixOS, qui le met à jour lui-même ; lancé par `cargo run`
//! — n'a pas d'installation : il ne se propose aucune mise à jour qu'il ne saurait pas poser.

use std::path::{Path, PathBuf};

/// Comment ce programme est installé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installation {
    /// Windows : l'installeur NSIS.
    Nsis,
    /// Linux : ce fichier AppImage.
    AppImage(PathBuf),
    /// Linux : un paquet Debian ; l'exécutable à relancer.
    Deb(PathBuf),
    /// Linux : un paquet RPM ; l'exécutable à relancer.
    Rpm(PathBuf),
}

impl Installation {
    /// **L'installation de ce programme**, ou rien s'il ne sait pas se remplacer.
    pub fn de_ce_programme() -> Option<Self> {
        if cfg!(windows) {
            return Some(Self::Nsis);
        }
        if !cfg!(target_os = "linux") {
            return None;
        }
        if let Some(fichier) = std::env::var_os("APPIMAGE") {
            return Some(Self::AppImage(fichier.into()));
        }
        let exe = std::env::current_exe().ok()?;
        let possede = |outil: &str, option: &str| {
            std::process::Command::new(outil)
                .arg(option)
                .arg(&exe)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        };
        if possede("dpkg-query", "-S") {
            Some(Self::Deb(exe))
        } else if possede("rpm", "-qf") {
            Some(Self::Rpm(exe))
        } else {
            None
        }
    }

    /// Son nom dans `latest.json`, après la plateforme : `windows-x86_64-nsis`,
    /// `linux-x86_64-deb`…
    pub fn nom(&self) -> &'static str {
        match self {
            Self::Nsis => "nsis",
            Self::AppImage(_) => "appimage",
            Self::Deb(_) => "deb",
            Self::Rpm(_) => "rpm",
        }
    }

    /// L'extension d'un installeur de cette forme.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Nsis => ".exe",
            Self::AppImage(_) => ".AppImage",
            Self::Deb(_) => ".deb",
            Self::Rpm(_) => ".rpm",
        }
    }

    /// **Les clés à lire dans `latest.json`, dans l'ordre de Tauri** : celle de cette forme
    /// d'abord, puis celle de la plateforme seule.
    pub fn cles(&self, plateforme: &str) -> [String; 2] {
        [
            format!("{plateforme}-{}", self.nom()),
            plateforme.to_string(),
        ]
    }

    /// **Pose ce qui a été vérifié**, pendant que Glucose tourne encore — sous Linux, un
    /// programme ouvert se remplace sans dommage : il garde l'ancien jusqu'à sa fin. Sous
    /// Windows, rien : l'installeur ne peut prendre la place que d'un Glucose fermé.
    pub fn poser(&self, installeur: &Path) -> Result<(), String> {
        match self {
            Self::Nsis => Ok(()),
            Self::AppImage(fichier) => poser_l_appimage(installeur, fichier),
            Self::Deb(_) => en_administrateur("dpkg", "-i", installeur),
            Self::Rpm(_) => en_administrateur("rpm", "-U", installeur),
        }
    }

    /// **Ce qui se lance quand Glucose se ferme** : l'installeur, qui relancera Glucose (les
    /// arguments de l'updater de Tauri) ; ou, sous Linux, Glucose lui-même, déjà remplacé.
    pub fn relance(&self, installeur: &Path) -> (PathBuf, &'static [&'static str]) {
        match self {
            Self::Nsis => (installeur.to_path_buf(), &["/P", "/R", "/UPDATE"]),
            Self::AppImage(programme) | Self::Deb(programme) | Self::Rpm(programme) => {
                (programme.clone(), &[])
            }
        }
    }
}

/// L'AppImage remplacé d'un bloc par ce qui a été vérifié.
fn poser_l_appimage(installeur: &Path, fichier: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let octets = std::fs::read(installeur).map_err(|e| e.to_string())?;
        crate::persist::atomic::poser_un_programme_d_un_bloc(fichier, &octets)
            .map_err(|e| format!("{} : {e}", fichier.display()))
    }
    #[cfg(not(unix))]
    {
        let _ = (installeur, fichier);
        Err("un AppImage ne se pose que sous Linux".to_string())
    }
}

/// Un paquet du système, installé en administrateur.
fn en_administrateur(outil: &str, option: &str, paquet: &Path) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        crate::plateforme::administrateur::lancer(outil, &[option.as_ref(), paquet.as_os_str()])
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (option, paquet);
        Err(format!("{outil} ne s'emploie que sous Linux"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Les noms et l'ordre de Tauri** : la clé de la forme d'abord, puis la plateforme.
    #[test]
    fn test_les_cles_suivent_l_ordre_de_tauri() {
        let deb = Installation::Deb("/usr/bin/glucose".into());
        assert_eq!(
            deb.cles("linux-x86_64"),
            ["linux-x86_64-deb".to_string(), "linux-x86_64".to_string()]
        );
        assert_eq!(
            Installation::Nsis.cles("windows-x86_64")[0],
            "windows-x86_64-nsis"
        );
        let appimage = Installation::AppImage("/home/x/Glucose.AppImage".into());
        assert_eq!(appimage.nom(), "appimage");
        assert_eq!(Installation::Rpm("/usr/bin/glucose".into()).nom(), "rpm");
    }

    /// **Ce qui se relance** : l'installeur et les arguments de Tauri sous Windows ; Glucose
    /// lui-même, déjà remplacé, sous Linux.
    #[test]
    fn test_ce_qui_se_relance() {
        let i = Path::new("Glucose_2.0.1_installeur.exe");
        assert_eq!(
            Installation::Nsis.relance(i),
            (i.to_path_buf(), &["/P", "/R", "/UPDATE"][..])
        );
        let programme = PathBuf::from("/usr/bin/glucose");
        assert_eq!(
            Installation::Deb(programme.clone()).relance(i),
            (programme, &[][..])
        );
    }
}
