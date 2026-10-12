//! **PARTAGE-2 — la feuille de partage du système** (fiche 63).
//!
//! Au bureau, exporter et « Enregistrer l'image sous… » demandent où écrire, par le sélecteur
//! de fichiers. Au téléphone, ce sélecteur n'existe pas, et les deux gestes **ne faisaient
//! rien** : le bouton Exporter mentait. Le téléphone fait comme Procreate et Concepts : le
//! fichier s'écrit dans le dossier privé de Glucose (`partages/`), puis la feuille de partage
//! d'Android l'offre à l'application que l'on choisit — Fichiers, la galerie, Discord, Drive.
//!
//! La porte se branche au lancement (`glucose-android` : `MainActivity.partagerUnFichier`) ; là
//! où rien ne la branche, les deux gestes gardent le sélecteur. **Ce qui décide est la porte, pas
//! le système** : une épreuve la branche à la main, et suit le même chemin que le téléphone.

use std::path::Path;
use std::sync::Arc;

/// Ce qui offre un fichier aux autres applications : son chemin, et son type (MIME).
pub type Partageur = Arc<dyn Fn(&Path, &str) + Send + Sync>;

#[cfg(not(test))]
static PARTAGEUR: std::sync::Mutex<Option<Partageur>> = std::sync::Mutex::new(None);

// Les épreuves tournent en parallèle : chacune a sa porte, comme sa boîte de sélecteurs.
#[cfg(test)]
thread_local! {
    static PARTAGEUR: std::cell::RefCell<Option<Partageur>> =
        const { std::cell::RefCell::new(None) };
}

/// **Branche la feuille de partage** — sous Android, au lancement.
pub fn installer_le_partageur(partageur: Partageur) {
    #[cfg(not(test))]
    {
        *PARTAGEUR
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(partageur);
    }
    #[cfg(test)]
    PARTAGEUR.with(|p| *p.borrow_mut() = Some(partageur));
}

/// La porte, si elle est branchée — prise hors du verrou : on l'appelle verrou relâché.
fn la_porte() -> Option<Partageur> {
    #[cfg(not(test))]
    {
        PARTAGEUR
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    #[cfg(test)]
    PARTAGEUR.with(|p| p.borrow().clone())
}

/// **Y a-t-il une feuille de partage ?** Alors exporter et enregistrer une image passent par
/// elle, et non par un sélecteur qui n'existe pas.
pub fn disponible() -> bool {
    la_porte().is_some()
}

/// **Offre ce fichier** aux autres applications. Sans feuille de partage, dit pourquoi rien
/// ne s'ouvre.
pub fn partager(fichier: &Path) -> Result<(), String> {
    let porte = la_porte().ok_or("aucune feuille de partage ici")?;
    porte(fichier, type_de(fichier));
    Ok(())
}

/// Le type d'un fichier, d'après son extension : ce que la feuille montre, et ce que
/// l'application choisie sait ouvrir.
pub fn type_de(fichier: &Path) -> &'static str {
    let extension = fichier
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "md" => "text/markdown",
        _ => "application/octet-stream",
    }
}
