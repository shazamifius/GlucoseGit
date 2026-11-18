//! **DEVANT-1 — « toujours devant », sur chaque bureau Linux** (fiche 120).
//!
//! # Ce qu'il a vu
//!
//! Sa capture du 17/11 : *« tout le système de REF avec Ctrl+Maj+A ne fonctionne tout simplement
//! pas pour l'intégralité des bureaux Linux »*. Le mode référence demande à `winit` une fenêtre
//! « toujours devant » (`WindowLevel::AlwaysOnTop`) : Windows, macOS et X11 la donnent ;
//! **Wayland n'a aucune requête** pour qu'une application se pose devant les autres — elle est en
//! débat depuis des années ([wayland#305](https://gitlab.freedesktop.org/wayland/wayland/-/issues/305)),
//! et la demande se perdait en silence.
//!
//! # La loi
//!
//! Sous Wayland, chaque bureau a sa voie, que **son propre outil** ouvre — celui qu'il livre avec
//! lui, rien à installer :
//!
//! * **niri** : la fenêtre passe dans la couche flottante, qui est toujours au-dessus des fenêtres
//!   en mosaïque ([niri](https://niri-wm.github.io/niri/Floating-Windows.html)) — niri n'a pas
//!   encore de « toujours devant » ([FAQ](https://github.com/niri-wm/niri/wiki/FAQ)) ;
//! * **Hyprland** et **sway** : la même couche flottante, par `hyprctl` et `swaymsg` ;
//! * **KDE Plasma** : un script de KWin, chargé par D-Bus, pose « garder au-dessus » (`keepAbove`)
//!   sur la fenêtre de ce processus — la voie que KDE indique tant que Wayland n'en a pas
//!   ([KDE 460873](https://bugs.kde.org/show_bug.cgi?id=460873)) ;
//! * **GNOME** et les autres : aucune voie pour une application. Glucose le **dit**, avec le geste
//!   du bureau, au lieu de faire semblant (R1 : un bouton qui ne fait rien encore le dit).
//!
//! La fenêtre se désigne par **le numéro de ce processus**, jamais par « celle qui a le focus » : au
//! lancement, une autre application peut l'avoir. Et la commande part **sur un fil** : un bureau
//! qui tarde à répondre ne fait jamais attendre l'image.

use std::process::Command;

/// Le bureau Wayland sous lequel Glucose tourne, lu dans son environnement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bureau {
    Niri,
    Hyprland,
    Sway,
    Plasma,
    Gnome,
    Autre,
}

impl Bureau {
    /// **Le bureau, d'après les variables qu'il pose** : la prise de son outil d'abord (une seule
    /// existe), puis le nom que `XDG_CURRENT_DESKTOP` donne.
    pub fn de(env: impl Fn(&str) -> Option<String>) -> Self {
        let pose = |cle: &str| env(cle).is_some_and(|v| !v.is_empty());
        let nomme = |nom: &str| {
            env("XDG_CURRENT_DESKTOP")
                .is_some_and(|v| v.split(':').any(|d| d.eq_ignore_ascii_case(nom)))
        };
        if pose("NIRI_SOCKET") {
            Self::Niri
        } else if pose("HYPRLAND_INSTANCE_SIGNATURE") {
            Self::Hyprland
        } else if pose("SWAYSOCK") {
            Self::Sway
        } else if nomme("KDE") || pose("KDE_SESSION_VERSION") {
            Self::Plasma
        } else if nomme("GNOME") {
            Self::Gnome
        } else {
            Self::Autre
        }
    }

    /// Le bureau de ce processus.
    pub fn courant() -> Self {
        Self::de(|cle| std::env::var(cle).ok())
    }
}

/// **Ce que la demande devient** : prise par le bureau, ou un geste à faire à la main.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Devant {
    Fait,
    AlaMain(&'static str),
}

/// Ce que GNOME laisse faire : son menu de fenêtre.
pub const GNOME: &str = "GNOME ne laisse aucune application se garder devant les autres : \
     Alt+Espace, puis « Toujours au premier plan ».";
/// Un bureau qu'on ne connaît pas.
pub const AUTRE: &str = "Ce bureau ne dit pas comment garder une fenêtre devant les autres : \
     cherche « toujours au premier plan » dans le menu de sa fenêtre.";

/// **Garde la fenêtre de ce processus devant les autres** (`oui`), ou la rend à sa place — par la
/// voie de `bureau`. La commande part sur un fil ; ce qui ne se peut pas se dit tout de suite.
pub fn garder_devant(bureau: Bureau, oui: bool) -> Devant {
    let pid = std::process::id();
    match bureau {
        Bureau::Gnome => Devant::AlaMain(GNOME),
        Bureau::Autre => Devant::AlaMain(AUTRE),
        _ => {
            let fil = std::thread::Builder::new()
                .name("glucose-devant".into())
                .spawn(move || {
                    if let Err(e) = executer(bureau, pid, oui) {
                        crate::dire_erreur!("[Glucose] toujours devant ({bureau:?}) : {e}");
                    }
                });
            match fil {
                Ok(_) => Devant::Fait,
                Err(_) => Devant::AlaMain(AUTRE),
            }
        }
    }
}

/// La voie de chaque bureau, jouée jusqu'au bout.
fn executer(bureau: Bureau, pid: u32, oui: bool) -> Result<(), String> {
    match bureau {
        Bureau::Niri => {
            let fenetres = sortie(&["niri", "msg", "--json", "windows"])?;
            let id = id_niri(&fenetres, pid).ok_or("la fenêtre de Glucose est introuvable")?;
            jouer(&commande(bureau, pid, oui, Some(id)))
        }
        Bureau::Plasma => kwin(pid, oui),
        _ => jouer(&commande(bureau, pid, oui, None)),
    }
}

/// **La commande qui pose la fenêtre** de `pid` dans la couche flottante (`oui`) ou la rend à la
/// mosaïque — niri la désigne par son numéro à lui (`id`).
pub fn commande(bureau: Bureau, pid: u32, oui: bool, id: Option<u64>) -> Vec<String> {
    match bureau {
        Bureau::Niri => {
            let action = if oui {
                "move-window-to-floating"
            } else {
                "move-window-to-tiling"
            };
            let id = id.map_or_else(String::new, |i| i.to_string());
            ["niri", "msg", "action", action, "--id", id.as_str()]
                .map(String::from)
                .to_vec()
        }
        Bureau::Hyprland => {
            let action = if oui { "setfloating" } else { "settiled" };
            vec![
                "hyprctl".into(),
                "dispatch".into(),
                action.into(),
                format!("pid:{pid}"),
            ]
        }
        Bureau::Sway => {
            let etat = if oui { "enable" } else { "disable" };
            vec!["swaymsg".into(), format!("[pid={pid}] floating {etat}")]
        }
        _ => Vec::new(),
    }
}

/// **Le numéro que niri donne à la fenêtre de `pid`**, dans la liste que `niri msg --json windows`
/// écrit.
pub fn id_niri(fenetres: &str, pid: u32) -> Option<u64> {
    let liste = crate::agents::json::Json::lire(fenetres).ok()?;
    liste.as_liste()?.iter().find_map(|f| {
        let sien = f.get("pid")?.as_f64()? == f64::from(pid);
        sien.then(|| f.get("id")?.as_f64())
            .flatten()
            .map(|i| i as u64)
    })
}

/// **Le script de KWin** qui pose `keepAbove` sur les fenêtres de `pid` — Plasma 6 les liste par
/// `windowList`, Plasma 5 par `clientList`.
pub fn script_kwin(pid: u32, oui: bool) -> String {
    format!(
        "const fenetres = workspace.windowList ? workspace.windowList() : workspace.clientList();\n\
         for (const f of fenetres) {{ if (f.pid === {pid}) {{ f.keepAbove = {oui}; }} }}\n"
    )
}

/// Charge le script dans KWin par D-Bus, le joue, puis le retire : `keepAbove` reste à la fenêtre.
fn kwin(pid: u32, oui: bool) -> Result<(), String> {
    let nom = format!("glucose-devant-{pid}");
    let chemin = std::env::temp_dir().join(format!("{nom}.js"));
    std::fs::write(&chemin, script_kwin(pid, oui)).map_err(|e| e.to_string())?;
    let appel = |objet: &str, methode: &str, args: &[String]| {
        let mut mots = vec![
            "dbus-send".to_string(),
            "--session".into(),
            "--print-reply".into(),
            "--dest=org.kde.KWin".into(),
            objet.into(),
            methode.into(),
        ];
        mots.extend(args.iter().cloned());
        sortie(&mots.iter().map(String::as_str).collect::<Vec<_>>())
    };
    let decharger = || {
        appel(
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            &[format!("string:{nom}")],
        )
    };
    let _ = decharger();
    let charge = appel(
        "/Scripting",
        "org.kde.kwin.Scripting.loadScript",
        &[
            format!("string:{}", chemin.display()),
            format!("string:{nom}"),
        ],
    )?;
    let numero = numero_du_script(&charge).ok_or("KWin n'a pas chargé le script")?;
    // Plasma 6 range ses scripts sous `/Scripting/ScriptN`, Plasma 5 sous `/N`.
    let joue = appel(
        &format!("/Scripting/Script{numero}"),
        "org.kde.kwin.Script.run",
        &[],
    )
    .or_else(|_| appel(&format!("/{numero}"), "org.kde.kwin.Script.run", &[]));
    let _ = decharger();
    let _ = std::fs::remove_file(&chemin);
    joue.map(|_| ())
}

/// Le numéro que `loadScript` rend, dans la réponse de `dbus-send` (« int32 3 »).
pub fn numero_du_script(reponse: &str) -> Option<i64> {
    let apres = reponse.split("int32").nth(1)?;
    apres
        .split_whitespace()
        .next()?
        .parse()
        .ok()
        .filter(|n| *n >= 0)
}

/// Joue une commande ; une sortie en échec dit pourquoi.
fn jouer(mots: &[String]) -> Result<(), String> {
    sortie(&mots.iter().map(String::as_str).collect::<Vec<_>>()).map(|_| ())
}

/// Joue une commande et rend ce qu'elle a écrit.
fn sortie(mots: &[&str]) -> Result<String, String> {
    let (programme, args) = mots.split_first().ok_or("aucune commande")?;
    let fait = Command::new(programme)
        .args(args)
        .output()
        .map_err(|e| format!("{programme} : {e}"))?;
    if fait.status.success() {
        Ok(String::from_utf8_lossy(&fait.stdout).into_owned())
    } else {
        Err(format!(
            "{programme} : {}",
            String::from_utf8_lossy(&fait.stderr).trim()
        ))
    }
}

#[cfg(test)]
#[path = "devant/tests.rs"]
mod tests;
