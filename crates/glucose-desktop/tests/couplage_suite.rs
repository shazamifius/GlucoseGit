//! **La règle S**, vérifiée mécaniquement : une fonctionnalité passe par l'API du `Store`,
//! jamais par les champs du modèle.
//!
//! # Pourquoi ce test existe
//!
//! La fondation de la phase B — l'arène compacte, les identifiants entiers — attend d'être
//! substituée au modèle actuel. Cette substitution est faisable tant qu'elle reste **locale au
//! `store`** : un module qui appelle `store.add_annotation(...)` survit intact au changement de
//! représentation ; un module qui écrit `board.annotations.push(...)` est à réécrire.
//!
//! Le plan d'exécution (fiche 12 § 3) parie là-dessus pour repousser la substitution après les
//! fonctionnalités. Ce pari ne tient que si le couplage ne grandit pas — et une règle qu'on ne
//! mesure pas est une règle qu'on perd. Celle-ci est donc comptée, pas affirmée.
//!
//! # Ce qui n'est pas compté
//!
//! Les tests et les preuves : ils ont le droit de regarder le modèle de près, c'est leur
//! travail. Seul le code qui **tourne chez l'utilisateur** est soumis à la règle.

use std::fs;
use std::path::Path;

/// Les champs de collection du modèle. Les toucher directement, c'est connaître la
/// représentation.
const CHAMPS: &[&str] = &[".annotations", ".images", ".folders", ".boards"];

/// Le plafond, relevé au commit qui a introduit ce test. Il ne doit que **descendre**.
///
/// Ce n'est pas un objectif de qualité mais un cliquet : chaque fonctionnalité neuve qui passe
/// par l'API du `Store` laisse ce nombre où il est, et chaque conversion d'un site existant le
/// fait baisser. Le jour où il atteint zéro, la substitution ne touche plus que le `store`.
const PLAFOND_DESKTOP: usize = 66;

fn compte_dans(racine: &Path) -> usize {
    let mut total = 0;
    let Ok(entrees) = fs::read_dir(racine) else { return 0 };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        let nom = chemin.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if chemin.is_dir() {
            total += compte_dans(&chemin);
        } else if nom.ends_with(".rs") && !nom.contains("tests") && !nom.contains("proof") {
            let Ok(source) = fs::read_to_string(&chemin) else { continue };
            for ligne in source.lines() {
                let nue = ligne.trim_start();
                // Les commentaires parlent du modèle sans y toucher.
                if nue.starts_with("//") {
                    continue;
                }
                if CHAMPS.iter().any(|c| ligne.contains(c)) {
                    total += 1;
                }
            }
        }
    }
    total
}

/// **Le couplage du desktop au modèle ne grandit pas.**
///
/// Si ce test échoue en disant que le compte a monté, c'est qu'un module neuf lit ou écrit les
/// collections du modèle en direct. Le correctif n'est jamais de relever le plafond : c'est de
/// passer par l'API du `Store`, en l'étendant si elle ne sait pas encore faire ce qu'il faut.
///
/// S'il échoue en disant que le compte a **baissé**, c'est une bonne nouvelle : il faut
/// abaisser le plafond d'autant, pour que le terrain gagné ne se reperde pas.
#[test]
fn test_le_couplage_du_desktop_au_modele_ne_grandit_pas() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let compte = compte_dans(&src);

    assert!(
        compte <= PLAFOND_DESKTOP,
        "le couplage au modèle a monté : {compte} accès directs contre {PLAFOND_DESKTOP} \
         autorisés. Passer par l'API du Store (règle S, fiche 12 § 3) plutôt que par les \
         champs du modèle."
    );
    assert!(
        compte >= PLAFOND_DESKTOP.saturating_sub(4),
        "le couplage est descendu à {compte} : abaisser PLAFOND_DESKTOP à cette valeur pour \
         que le terrain gagné ne se reperde pas"
    );
}
