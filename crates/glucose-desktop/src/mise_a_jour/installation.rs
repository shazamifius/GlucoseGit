//! **Comment ce programme est installé**, et donc comment il se remplace (fiche 48).
//!
//! L'updater de Glucose Tauri connaît quatre formes, et les nomme dans `latest.json` : l'installeur
//! NSIS sous Windows ; sous Linux l'AppImage, le paquet `.deb` et le paquet `.rpm`. Glucose Rust
//! les reprend, avec leurs noms — une bascule sans couture, et un seul fichier des versions.
//!
//! * **NSIS** : l'installeur se lance quand Glucose se ferme, et attend qu'il le soit. Un Glucose
//!   posé par lui se reconnaît au désinstalleur qu'il écrit à côté de `glucose.exe`.
//! * **AppImage** : le fichier lui-même, reconnu par la variable `APPIMAGE` que son lanceur pose,
//!   est remplacé d'un bloc — exécutable avant de prendre sa place.
//! * **`.deb`, `.rpm`** : l'exécutable appartient à un paquet du système (`dpkg-query -S`,
//!   `rpm -qf`) ; le nouveau paquet s'installe par `pkexec`, comme chez Tauri.
//! * **APK** (fiche 58, MAJ-ANDROID-1) : sous Android, toute installation est un APK, et
//!   « poser » l'APK, c'est le **confier à l'installeur du système**
//!   ([`crate::plateforme::installation`]) — qui remplace Glucose lui-même. Rien à relancer.
//!
//! Un Glucose installé autrement — par NixOS, qui le met à jour lui-même ; lancé par `cargo run`
//! — n'a pas d'installation : il ne se propose aucune mise à jour qu'il ne saurait pas poser.

use std::path::{Path, PathBuf};

/// Le désinstalleur que l'installeur Windows écrit à côté de `glucose.exe`.
pub const DESINSTALLEUR: &str = "uninstall.exe";

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
    /// Android : un APK, que le système installe à la place de Glucose.
    Apk,
}

impl Installation {
    /// **L'installation de ce programme**, ou rien s'il ne sait pas se remplacer.
    pub fn de_ce_programme() -> Option<Self> {
        if cfg!(target_os = "android") {
            return Some(Self::Apk);
        }
        if cfg!(windows) {
            return Self::de_windows(&std::env::current_exe().ok()?);
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

    /// **Sous Windows, posé par son installeur** : le désinstalleur qu'il écrit à côté de
    /// `glucose.exe` (`outils/installeur/glucose.nsi`, `WriteUninstaller`) — l'équivalent de
    /// « l'exécutable appartient à un paquet » sous Linux. Une construction de travail
    /// (`cargo run`) n'en a pas : elle ne se propose aucune mise à jour, qui s'installerait
    /// ailleurs et fermerait la session d'essai.
    pub fn de_windows(exe: &Path) -> Option<Self> {
        exe.with_file_name(DESINSTALLEUR)
            .is_file()
            .then_some(Self::Nsis)
    }

    /// Son nom dans `latest.json`, après la plateforme : `windows-x86_64-nsis`,
    /// `linux-x86_64-deb`…
    pub fn nom(&self) -> &'static str {
        match self {
            Self::Nsis => "nsis",
            Self::AppImage(_) => "appimage",
            Self::Deb(_) => "deb",
            Self::Rpm(_) => "rpm",
            Self::Apk => "apk",
        }
    }

    /// L'extension d'un installeur de cette forme.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Nsis => ".exe",
            Self::AppImage(_) => ".AppImage",
            Self::Deb(_) => ".deb",
            Self::Rpm(_) => ".rpm",
            Self::Apk => ".apk",
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
            Self::Apk => crate::plateforme::installation::confier(installeur),
        }
    }

    /// **Ce qui se lance quand Glucose se ferme** : l'installeur, qui relancera Glucose (les
    /// arguments de l'updater de Tauri) ; ou, sous Linux, Glucose lui-même, déjà remplacé.
    /// Sous Android, rien : le système a l'APK, et remplace Glucose quand il l'installe.
    pub fn relance(&self, installeur: &Path) -> Option<(PathBuf, &'static [&'static str])> {
        match self {
            Self::Nsis => Some((installeur.to_path_buf(), &["/P", "/R", "/UPDATE"])),
            Self::AppImage(programme) | Self::Deb(programme) | Self::Rpm(programme) => {
                Some((programme.clone(), &[]))
            }
            Self::Apk => None,
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

    /// **Sous Windows, seul un Glucose posé par son installeur se remplace** : à côté de
    /// `glucose.exe`, le désinstalleur que l'installeur écrit. Une construction de travail —
    /// `target\release\glucose-desktop.exe` — n'en a pas, et ne se propose rien.
    #[test]
    fn test_sous_windows_seul_un_glucose_installe_se_remplace() {
        let d = std::env::temp_dir().join(format!("glucose-installation-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let exe = d.join("glucose.exe");
        std::fs::write(&exe, b"x").unwrap();
        assert_eq!(
            Installation::de_windows(&exe),
            None,
            "une construction de travail"
        );
        std::fs::write(d.join(DESINSTALLEUR), b"x").unwrap();
        assert_eq!(Installation::de_windows(&exe), Some(Installation::Nsis));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **Le désinstalleur que Glucose cherche est celui que l'installeur écrit** : écrit deux
    /// fois, le nom divergerait un jour, et plus aucun Glucose installé ne se mettrait à jour.
    #[test]
    fn test_l_installeur_ecrit_le_desinstalleur_que_glucose_cherche() {
        let nsi = include_str!("../../../../outils/installeur/glucose.nsi");
        let attendue = format!("WriteUninstaller \"$INSTDIR\\{DESINSTALLEUR}\"");
        assert!(
            nsi.lines().any(|l| l.trim() == attendue),
            "glucose.nsi doit écrire {attendue}"
        );
    }

    /// **Ce qui se relance** : l'installeur et les arguments de Tauri sous Windows ; Glucose
    /// lui-même, déjà remplacé, sous Linux.
    #[test]
    fn test_ce_qui_se_relance() {
        let i = Path::new("Glucose_2.0.1_installeur.exe");
        assert_eq!(
            Installation::Nsis.relance(i),
            Some((i.to_path_buf(), &["/P", "/R", "/UPDATE"][..]))
        );
        let programme = PathBuf::from("/usr/bin/glucose");
        assert_eq!(
            Installation::Deb(programme.clone()).relance(i),
            Some((programme, &[][..]))
        );
        assert_eq!(
            Installation::Apk.relance(i),
            None,
            "Android remplace Glucose lui-même"
        );
    }

    /// **Sous Android, l'APK et ses clés** (MAJ-ANDROID-1) : `android-aarch64-apk`, puis la
    /// plateforme seule, comme le manifeste les écrit ; et sans installeur du système, poser
    /// le dit au lieu de faire semblant.
    #[test]
    fn test_maj_android_1_l_apk_et_ses_cles() {
        assert_eq!(
            Installation::Apk.cles("android-aarch64"),
            [
                "android-aarch64-apk".to_string(),
                "android-aarch64".to_string()
            ]
        );
        assert_eq!(Installation::Apk.extension(), ".apk");
        let manifeste = include_str!("../../../../outils/publication/manifeste.py");
        for cle in [
            "android-aarch64",
            "android-aarch64-apk",
            "android-armv7",
            "android-armv7-apk",
        ] {
            assert!(
                manifeste.contains(&format!("\"{cle}\": apk")),
                "le manifeste écrit {cle}"
            );
        }
    }
}
