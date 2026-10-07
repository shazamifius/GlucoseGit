//! **L'accord de cette machine** : a-t-on répondu, et quoi — et l'identifiant tiré au hasard qui
//! relie ses sessions entre elles, sans rien dire de la personne.
//!
//! Un fichier de deux lignes dans le dossier de l'application, lisible par n'importe qui :
//!
//! ```text
//! envoyer=oui
//! installation=3f9c0d…
//! ```

use std::path::Path;

/// Le nom du fichier, dans le dossier de l'application.
pub const FICHIER: &str = "telemetrie.txt";

/// Ce que cette machine a répondu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accord {
    /// `None` tant que personne n'a répondu.
    pub envoyer: Option<bool>,
    /// 32 chiffres hexadécimaux, tirés au hasard.
    pub installation: String,
}

impl Accord {
    /// **Lit l'accord** ; ce qui manque ou ne se reconnaît pas n'est pas une réponse, et un
    /// identifiant qui manque se tire au hasard.
    pub fn lire(texte: &str) -> Self {
        let valeur = |cle: &str| {
            texte
                .lines()
                .find_map(|l| l.trim().strip_prefix(cle)?.strip_prefix('='))
                .map(str::trim)
        };
        let envoyer = match valeur("envoyer") {
            Some("oui") => Some(true),
            Some("non") => Some(false),
            _ => None,
        };
        let installation = valeur("installation")
            .filter(|i| est_un_identifiant(i))
            .map_or_else(nouvel_identifiant, str::to_string);
        Self {
            envoyer,
            installation,
        }
    }

    /// Le texte du fichier.
    pub fn ecrire(&self) -> String {
        let envoyer = match self.envoyer {
            Some(true) => "oui",
            Some(false) => "non",
            None => "",
        };
        format!("envoyer={envoyer}\ninstallation={}\n", self.installation)
    }

    /// L'accord de cette machine, ou aucun s'il n'y a pas encore de fichier.
    pub fn charger(dossier: &Path) -> Self {
        Self::lire(&std::fs::read_to_string(dossier.join(FICHIER)).unwrap_or_default())
    }

    /// **Le retient** pour les lancements suivants.
    pub fn retenir(&self, dossier: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dossier)?;
        std::fs::write(dossier.join(FICHIER), self.ecrire())
    }
}

/// Un identifiant : 32 chiffres hexadécimaux en minuscules — ce que le serveur accepte.
pub fn est_un_identifiant(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// **Un identifiant tiré au hasard**, 128 bits.
///
/// Le hasard est celui que le système donne à la bibliothèque standard pour ses tables de
/// hachage (`RandomState`, dont les clés viennent du générateur du système à chaque fil) :
/// deux états, deux nombres de 64 bits — aucune caisse de plus pour un identifiant qui ne
/// protège rien et ne doit que ne pas se répéter.
pub fn nouvel_identifiant() -> String {
    use std::hash::{BuildHasher, Hasher};
    let tirer = || {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        h.finish()
    };
    format!("{:016x}{:016x}", tirer(), tirer())
}
