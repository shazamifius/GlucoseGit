//! **Le souvenir de l'arbitre** : la carte qu'il a choisie, retenue pour le lancement suivant
//! (ARBITRE-4).
//!
//! # Pourquoi l'arbitre ne change plus de carte en cours de route
//!
//! Cinq sessions de terrain, et la ligne de partage n'a jamais bougé : la carte NVIDIA
//! ouverte **au lancement** n'a jamais figé ; la même carte ouverte **en cours de route** a
//! figé à chaque fois. La session du 23/09 à 14 h 44 a dit pourquoi, et c'est pire qu'un
//! ralentissement :
//!
//! ```text
//!     8,3 s   l'arbitre passe sur la NVIDIA
//!     8,8 s   premiere image sur la nouvelle carte : 500 ms
//!     9,4 s   Glucose dessine et presente des centaines d'images, a 103 par seconde,
//!     → 14 s  la latence du geste reste normale, et l'utilisateur voit un canevas FIGE
//! ```
//!
//! **Les images partent, et n'arrivent pas à l'écran.** Chaque présentation répond « réussie »,
//! donc rien dans l'application ne peut le voir. C'est exactement ce que l'utilisateur décrit
//! depuis trois jours — *« le canva freeze, mais Ctrl+O, Ctrl+S fonctionnent »* : ces fenêtres-
//! là sont dessinées par Windows, pas par Glucose.
//!
//! La recherche en donne deux causes plausibles sur un portable hybride, et aucune ne se
//! vérifie de l'intérieur : une fenêtre ne porte qu'une chaîne d'images à la fois, et la
//! précédente doit être **entièrement** détruite avant la suivante ; et une présentation sur
//! la carte dédiée peut répondre `DXGI_STATUS_OCCLUDED` — un code de **succès** — en jetant
//! l'image. C'est aussi pourquoi les logiciels professionnels demandent un redémarrage pour
//! changer de carte.
//!
//! # Ce que l'adaptation garde, et ce qu'elle perd
//!
//! L'arbitre observe toujours, avec la même loi (ARBITRE-2). Quand il conclut, il **écrit son
//! choix** au lieu de l'appliquer, et le lancement suivant ouvre directement cette carte —
//! le chemin qui n'a jamais figé. L'adaptation prend effet au lancement, pas à la seconde :
//! c'est moins « temps réel » que la charte ne l'ambitionne, et c'est la seule forme qui ne
//! casse pas ce qu'elle répare.
//!
//! Un souvenir n'est pas une étiquette : c'est une observation faite **sur cette machine**.
//! Si elle cesse d'être vraie — Blender sature la carte retenue —, l'arbitre la voit geler
//! et écrit l'autre.

use crate::present::arbitre::Preference;
use std::path::{Path, PathBuf};

/// Le nom du fichier, dans le dossier de l'application.
pub const FICHIER: &str = "carte.txt";

/// **Le dossier où l'application garde ce qu'elle a appris de la machine.**
///
/// Pur : les variables d'environnement lui sont données, pour que le choix se teste sans
/// toucher à celles du processus — `cargo test` lance seize fils à la fois, et une variable
/// modifiée par l'un se lit dans tous les autres.
///
/// Dans l'ordre : le dossier local des applications de Windows, le dossier d'état que la
/// norme XDG prévoit sous Linux, le dossier personnel, et en dernier recours le répertoire
/// temporaire — où le souvenir peut s'effacer, ce qui ne coûte qu'un réapprentissage.
pub fn dossier_depuis(
    appdata_local: Option<PathBuf>,
    etat_xdg: Option<PathBuf>,
    personnel: Option<PathBuf>,
    temporaire: PathBuf,
) -> PathBuf {
    appdata_local
        .map(|d| d.join("Glucose"))
        .or_else(|| etat_xdg.map(|d| d.join("glucose")))
        .or_else(|| personnel.map(|d| d.join(".local").join("state").join("glucose")))
        .unwrap_or_else(|| temporaire.join("glucose"))
}

/// Le dossier de l'application sur cette machine.
pub fn dossier() -> PathBuf {
    let var = |nom: &str| std::env::var_os(nom).map(PathBuf::from);
    dossier_depuis(
        var("LOCALAPPDATA"),
        var("XDG_STATE_HOME"),
        var("HOME"),
        std::env::temp_dir(),
    )
}

/// Le fichier du souvenir sur cette machine.
pub fn chemin() -> PathBuf {
    dossier().join(FICHIER)
}

/// **La carte retenue**, ou rien si aucune session ne l'a encore choisie.
///
/// Un contenu qu'on ne reconnaît pas n'est pas une erreur : c'est l'absence de souvenir, et
/// l'arbitre repart de l'économe.
pub fn lire(chemin: &Path) -> Option<Preference> {
    let texte = std::fs::read_to_string(chemin).ok()?;
    Preference::depuis_nom(texte.trim())
}

/// **Retient cette carte** pour le lancement suivant.
pub fn ecrire(chemin: &Path, carte: Preference) -> std::io::Result<()> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    std::fs::write(chemin, carte.nom())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_le_souvenir_se_relit_tel_qu_il_a_ete_ecrit() {
        let dossier = std::env::temp_dir().join(format!("glucose-souvenir-{}", std::process::id()));
        let chemin = dossier.join(FICHIER);
        assert_eq!(lire(&chemin), None, "aucune session n'a encore choisi");
        for carte in [Preference::Rapide, Preference::Econome] {
            ecrire(&chemin, carte).expect("ecriture du souvenir");
            assert_eq!(lire(&chemin), Some(carte));
        }
        std::fs::write(&chemin, "une carte inconnue").expect("ecriture");
        assert_eq!(
            lire(&chemin),
            None,
            "un contenu inconnu n'est pas un souvenir"
        );
        std::fs::remove_dir_all(&dossier).ok();
    }

    #[test]
    fn test_le_dossier_suit_la_plateforme_puis_se_replie() {
        let t = PathBuf::from("T");
        assert_eq!(
            dossier_depuis(
                Some("L".into()),
                Some("X".into()),
                Some("H".into()),
                t.clone()
            ),
            PathBuf::from("L").join("Glucose")
        );
        assert_eq!(
            dossier_depuis(None, Some("X".into()), Some("H".into()), t.clone()),
            PathBuf::from("X").join("glucose")
        );
        assert_eq!(
            dossier_depuis(None, None, Some("H".into()), t.clone()),
            PathBuf::from("H")
                .join(".local")
                .join("state")
                .join("glucose")
        );
        assert_eq!(
            dossier_depuis(None, None, None, t.clone()),
            t.join("glucose")
        );
    }
}
