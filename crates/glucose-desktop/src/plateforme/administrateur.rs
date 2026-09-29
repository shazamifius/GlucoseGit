//! **Lancer une commande en administrateur, sous Linux** — ce qu'installer un paquet du système
//! demande (fiche 48).
//!
//! Par `pkexec` : la fenêtre d'autorisation du bureau, celle que l'updater de Tauri ouvrait déjà
//! pour la même chose. Un programme qui est déjà administrateur — une machine d'épreuve —
//! lance la commande lui-même.

/// Lance `programme` avec `arguments` en administrateur, et attend sa fin.
pub fn lancer(programme: &str, arguments: &[&std::ffi::OsStr]) -> Result<(), String> {
    // SAFETY : `geteuid` ne lit que l'identité du processus ; elle ne peut pas échouer.
    let deja = unsafe { libc::geteuid() } == 0;
    let mut commande = if deja {
        std::process::Command::new(programme)
    } else {
        let mut c = std::process::Command::new("pkexec");
        c.arg(programme);
        c
    };
    let statut = commande
        .args(arguments)
        .status()
        .map_err(|e| format!("{programme} ne démarre pas : {e}"))?;
    if statut.success() {
        Ok(())
    } else {
        Err(format!("{programme} a refusé ({statut})"))
    }
}
