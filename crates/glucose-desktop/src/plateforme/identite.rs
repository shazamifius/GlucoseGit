//! **Qui est Glucose pour Windows** (fiche 48) : l'identité par laquelle la barre des tâches
//! reconnaît ses fenêtres, ses raccourcis et ses épingles — son AppUserModelID.
//!
//! # Pourquoi la déclarer
//!
//! L'installeur de Glucose Tauri posait `com.glucose.app` sur ses raccourcis, mais son programme
//! ne la déclarait jamais (lu dans les sources de Tauri : seul le greffon des notifications s'en
//! sert). Une fenêtre sans identité n'est rattachée à son épingle que si elle a été lancée par ce
//! raccourci-là ; relancée par une mise à jour, ou ouverte autrement, elle prend un second bouton
//! à côté d'elle. Microsoft l'écrit : une identité explicite se porte **partout** — processus,
//! raccourcis, épingles — ou nulle part
//! (learn.microsoft.com/windows/win32/shell/appids).
//!
//! Glucose Rust garde celle de Glucose Tauri : les épingles de ses utilisateurs la portent déjà.
//! Son installeur la pose sur chaque raccourci (`outils/installeur/glucose.nsi`, `IDENTITE`).

/// L'identité de Glucose, celle de Glucose Tauri.
pub const IDENTITE: &str = "com.glucose.app";

/// **Déclare l'identité de ce processus**, avant toute fenêtre et tout dialogue — Microsoft
/// l'exige avant la première interface. Un refus ne coûte que le regroupement des boutons.
pub fn declarer() {
    #[cfg(windows)]
    {
        let identite = windows::core::HSTRING::from(IDENTITE);
        // SAFETY : une chaîne terminée par un zéro, que le système copie ; aucun état partagé.
        if let Err(e) =
            unsafe { windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(&identite) }
        {
            eprintln!("[Glucose] identité refusée par Windows : {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IDENTITE;

    /// **Le programme et son installeur portent la même identité** : écrite deux fois, elle
    /// divergerait un jour, et la barre des tâches séparerait la fenêtre de son épingle.
    #[test]
    fn test_l_installeur_pose_l_identite_que_le_programme_declare() {
        let nsi = include_str!("../../../../outils/installeur/glucose.nsi");
        let attendue = format!("!define IDENTITE \"{IDENTITE}\"");
        assert!(
            nsi.lines().any(|l| l.trim() == attendue),
            "glucose.nsi doit définir {attendue}"
        );
    }

    /// Ce que Microsoft admet : 128 caractères au plus, sans espace.
    #[test]
    fn test_l_identite_a_la_forme_que_windows_admet() {
        assert!(IDENTITE.len() <= 128 && !IDENTITE.contains(' '));
    }
}
