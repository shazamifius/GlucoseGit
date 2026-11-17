//! **CODE-1 — un code, deux pouvoirs** (fiche 117).
//!
//! Le code « écrire » est trente-deux octets du hasard ; **tout en découle par empreinte** :
//!
//! ```text
//!   écrire E ──► lire L = H(« lire » ‖ E) ──► la salle   H(« identite » ‖ L)
//!            │                            └─► la clé     H(« chiffre » ‖ L)
//!            └─► le secret d'écriture  H(« ecrire » ‖ E)  — dont la salle ne garde que l'empreinte
//! ```
//!
//! Qui a le code « lire » trouve la salle et ouvre ses gestes ; il ne retrouve jamais le code
//! « écrire » ni le secret d'écriture (une empreinte ne se remonte pas). La salle ne voit que son
//! identité et l'empreinte du secret : ni la clé, ni le code. **Rien ne se garde** : le code est
//! tout ce qu'il faut — le relais n'a aucune clé à perdre, Glucose aucun trousseau à tenir.
//!
//! Chaque dérivée a son étiquette, écrite **avant** le secret : deux usages ne partagent jamais une
//! empreinte (la séparation des domaines d'une dérivation de clé par hachage, NIST SP 800-56C).

use crate::encodage::{base64, base64_adresse};
use crate::hash::sha256;

const ECRIRE: &str = "glucose-ecrire-";
const LIRE: &str = "glucose-lire-";
/// Trente-deux octets en base64 sans `=` : quarante-trois signes.
const SIGNES: usize = 43;

/// **Un code d'invitation** : ce qu'on copie pour qu'un autre rejoigne le canevas.
#[derive(Clone, PartialEq, Eq)]
pub struct Code {
    lire: [u8; 32],
    ecrire: Option<[u8; 32]>,
}

/// Un code ne s'écrit jamais dans un journal : `{:?}` ne dit que son pouvoir.
impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pouvoir = if self.ecrire.is_some() {
            "écrire"
        } else {
            "lire"
        };
        write!(f, "Code({pouvoir})")
    }
}

fn deriver(etiquette: &str, secret: &[u8; 32]) -> [u8; 32] {
    let mut m = Vec::with_capacity(etiquette.len() + 32);
    m.extend_from_slice(etiquette.as_bytes());
    m.extend_from_slice(secret);
    sha256(&m)
}

impl Code {
    /// **Un canevas qu'on partage** : un code neuf, de trente-deux octets du hasard du système que
    /// l'appelant tire (le noyau n'a ni hasard ni horloge).
    pub fn neuf(hasard: [u8; 32]) -> Self {
        Self {
            lire: deriver("glucose/salle/lire", &hasard),
            ecrire: Some(hasard),
        }
    }

    /// **Lit un code collé** : `glucose-ecrire-…` ou `glucose-lire-…`, quarante-trois signes ; les
    /// blancs autour se pardonnent (un copier-coller), rien d'autre.
    pub fn lire_le_texte(texte: &str) -> Option<Self> {
        let t = texte.trim();
        let (ecrire, reste) = match t.strip_prefix(ECRIRE) {
            Some(r) => (true, r),
            None => (false, t.strip_prefix(LIRE)?),
        };
        let propre = reste.len() == SIGNES
            && reste
                .bytes()
                .all(|o| o.is_ascii_alphanumeric() || o == b'-' || o == b'_');
        if !propre {
            return None;
        }
        let octets: [u8; 32] = base64(reste)?.try_into().ok()?;
        Some(if ecrire {
            Self::neuf(octets)
        } else {
            Self {
                lire: octets,
                ecrire: None,
            }
        })
    }

    /// Le code qui écrit, s'il est en main.
    pub fn texte_ecrire(&self) -> Option<String> {
        self.ecrire
            .map(|e| format!("{ECRIRE}{}", base64_adresse(&e)))
    }

    /// Le code qui ne fait que lire — toujours en main.
    pub fn texte_lire(&self) -> String {
        format!("{LIRE}{}", base64_adresse(&self.lire))
    }

    pub fn peut_ecrire(&self) -> bool {
        self.ecrire.is_some()
    }

    /// **L'identité de la salle** : trente-deux chiffres hexadécimaux, ce que son adresse porte
    /// (`/s/<identité>/salle`).
    pub fn salle(&self) -> String {
        deriver("glucose/salle/identite", &self.lire)[..16]
            .iter()
            .map(|o| format!("{o:02x}"))
            .collect()
    }

    /// La clé du chiffre des gestes (ChaCha20-Poly1305).
    pub fn cle(&self) -> [u8; 32] {
        deriver("glucose/salle/chiffre", &self.lire)
    }

    /// **Le secret d'écriture**, présenté à la salle (`Authorization: Bearer …`), qui n'en garde que
    /// l'empreinte, fixée par le premier écrivain.
    pub fn secret_d_ecriture(&self) -> Option<String> {
        self.ecrire
            .map(|e| base64_adresse(&deriver("glucose/salle/ecrire", &e)))
    }
}
