//! **Ce que la porte publique garde sur le disque** (fiche 116) : le secret de ce Glucose, si sa
//! porte est ouverte, et **les IA qui ont ta porte** — chacune avec ce qu'elle peut faire, et
//! l'empreinte de sa clé (jamais la clé : un fichier volé n'ouvre rien).
//!
//! Un texte JSON, `porte-publique.json`, dans le dossier de l'application ; lisible seulement par
//! l'utilisateur. Le secret prouve au relais que ce Glucose est bien lui : qui le prendrait
//! recevrait les demandes adressées à ce Glucose — pas son canevas, que personne d'autre ne tient.

use crate::agents::json::Json;
use glucose_core::encodage::{base64, base64_adresse};
use glucose_core::hash::{hex_of, sha256};

/// Le nom du fichier, dans le dossier de l'application.
pub const FICHIER: &str = "porte-publique.json";

/// **Ce qu'une IA peut faire par ta porte** — ton choix, dans Glucose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Droits {
    Lire,
    Ecrire,
}

impl Droits {
    fn mot(self) -> &'static str {
        match self {
            Droits::Lire => "lire",
            Droits::Ecrire => "ecrire",
        }
    }
}

/// **Une IA qui a ta porte.**
#[derive(Debug, Clone, PartialEq)]
pub struct Accord {
    /// Comment elle s'est présentée : l'adresse de son document d'identité (CIMD), ou celle que
    /// la porte lui a donnée à son inscription (`dcr:…`).
    pub client_id: String,
    /// Le nom qu'elle se donne (« Claude ») — déclaré.
    pub nom: String,
    /// **Le domaine d'où elle vient** (« claude.ai ») — vérifié : celui de son document
    /// d'identité, ou de son adresse de retour.
    pub hote: String,
    pub droits: Droits,
    /// L'empreinte SHA-256 de sa clé d'accès, en hexadécimal.
    pub empreinte: String,
    /// Depuis quand, en secondes depuis 1970.
    pub depuis: u64,
}

impl Accord {
    /// **Le nom sous lequel elle signe ses gestes** (MAIN-1) : « Claude (claude.ai) ».
    pub fn signature(&self) -> String {
        format!("{} ({})", self.nom, self.hote)
    }
}

/// **L'état de la porte**, tel qu'il se garde.
#[derive(Debug, Clone, PartialEq)]
pub struct Etat {
    pub secret: [u8; 32],
    pub ouverte: bool,
    pub accords: Vec<Accord>,
}

/// L'empreinte d'une clé d'accès, telle que l'état la garde.
pub fn empreinte(cle: &str) -> String {
    hex_of(&sha256(cle.as_bytes()))
}

impl Etat {
    /// Un état neuf : un secret tiré au hasard du système, la porte fermée.
    pub fn neuf() -> Result<Etat, String> {
        Ok(Etat {
            secret: crate::plateforme::hasard::octets::<32>()?,
            ouverte: false,
            accords: Vec::new(),
        })
    }

    /// **L'identifiant de ce Glucose** sur le relais : les 32 premiers chiffres hexadécimaux du
    /// SHA-256 de son secret — le relais refait le calcul, sans rien garder.
    pub fn identifiant(&self) -> String {
        hex_of(&sha256(&self.secret))[..32].to_string()
    }

    /// Le secret, tel qu'il se présente au relais.
    pub fn secret_texte(&self) -> String {
        base64_adresse(&self.secret)
    }

    /// L'IA à qui appartient cette clé d'accès, s'il en est une.
    pub fn accord_de(&self, cle: &str) -> Option<&Accord> {
        let e = empreinte(cle);
        self.accords.iter().find(|a| a.empreinte == e)
    }

    /// **Donne la porte** : une IA déjà venue sous le même nom d'identité reprend sa place —
    /// sa clé d'avant ne vaut plus rien.
    pub fn accorder(&mut self, accord: Accord) {
        self.accords.retain(|a| a.client_id != accord.client_id);
        self.accords.push(accord);
    }

    pub fn lire(texte: &str) -> Option<Etat> {
        let j = Json::lire(texte).ok()?;
        let secret: [u8; 32] = base64(j.get("secret")?.as_str()?)?.try_into().ok()?;
        let accords = j
            .get("accords")
            .and_then(Json::as_liste)
            .unwrap_or_default()
            .iter()
            .filter_map(accord_lu)
            .collect();
        Some(Etat {
            secret,
            ouverte: j.get("ouverte").and_then(Json::as_bool).unwrap_or(false),
            accords,
        })
    }

    pub fn ecrire(&self) -> String {
        let accords = self
            .accords
            .iter()
            .map(|a| {
                Json::objet([
                    ("client_id", Json::from(a.client_id.as_str())),
                    ("nom", Json::from(a.nom.as_str())),
                    ("hote", Json::from(a.hote.as_str())),
                    ("droits", Json::from(a.droits.mot())),
                    ("empreinte", Json::from(a.empreinte.as_str())),
                    ("depuis", Json::Nombre(a.depuis as f64)),
                ])
            })
            .collect();
        Json::objet([
            ("secret", Json::from(self.secret_texte())),
            ("ouverte", Json::Bool(self.ouverte)),
            ("accords", Json::Liste(accords)),
        ])
        .ecrire_en_retrait()
    }
}

fn accord_lu(j: &Json) -> Option<Accord> {
    let texte = |c: &str| j.get(c).and_then(Json::as_str).map(str::to_string);
    let droits = match j.get("droits")?.as_str()? {
        "lire" => Droits::Lire,
        "ecrire" => Droits::Ecrire,
        _ => return None,
    };
    Some(Accord {
        client_id: texte("client_id")?,
        nom: texte("nom")?,
        hote: texte("hote")?,
        droits,
        empreinte: texte("empreinte")?,
        depuis: j.get("depuis").and_then(Json::as_f64).unwrap_or(0.0) as u64,
    })
}

/// **Lit l'état** du fichier, ou en fait un neuf s'il n'existe pas. Un fichier illisible n'est
/// jamais écrasé : la porte reste fermée, et l'erreur le dit.
pub fn charger(chemin: &std::path::Path) -> Result<Etat, String> {
    match std::fs::read_to_string(chemin) {
        Ok(texte) => {
            Etat::lire(&texte).ok_or_else(|| format!("{} ne se lit pas", chemin.display()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Etat::neuf(),
        Err(e) => Err(format!("{} : {e}", chemin.display())),
    }
}

/// **Écrit l'état** d'un bloc (la loi de `persist::atomic` : jamais à moitié), lisible seulement
/// par l'utilisateur.
pub fn enregistrer(chemin: &std::path::Path, etat: &Etat) -> Result<(), String> {
    crate::persist::atomic::ecrire_d_un_bloc(chemin, etat.ecrire().as_bytes())
        .map_err(|e| format!("{} : {e}", chemin.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o600)).ok();
    }
    Ok(())
}
