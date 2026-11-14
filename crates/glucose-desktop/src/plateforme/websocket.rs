//! **Un WebSocket sortant** (RFC 6455) — la connexion par laquelle un Glucose se relie lui-même à
//! son relais (la porte publique, fiche 116), comme un navigateur ouvre une page : aucun port de
//! la box ne s'ouvre.
//!
//! Une porte, une voie par système, comme [`super::telecharger`] : **WinHTTP** sous Windows, qui
//! sait le WebSocket depuis Windows 8 et prend le proxy et les certificats du système (une école
//! qui filtre sa sortie) ; ailleurs, **`rustls`** — déjà là par `ureq`, mêmes fonctions, rien de
//! plus ne se compile (`decisions/16`) — et les trames écrites ici ([`trames`]).
//!
//! Un [`Canal`] n'a que deux gestes : envoyer un message texte, attendre le suivant un temps
//! donné. Le ping et le pong du protocole se répondent tout seuls ; ce qui dit qu'une connexion
//! vit encore se décide au-dessus (le relais répond `pong` à `ping` sans se réveiller).

use std::time::Duration;

#[cfg(not(windows))]
mod flux;
pub mod trames;
#[cfg(windows)]
mod windows;

/// **Une connexion WebSocket ouverte.**
pub trait Canal: Send {
    /// Envoie un message texte entier.
    fn envoyer(&mut self, texte: &str) -> Result<(), String>;
    /// **Le prochain message texte**, s'il en arrive un avant `delai` ; `Ok(None)` sinon. Une
    /// erreur dit la connexion perdue : il faut en ouvrir une autre.
    fn recevoir(&mut self, delai: Duration) -> Result<Option<String>, String>;
}

/// **Ouvre un WebSocket** vers `ws://` ou `wss://`, avec ces en-têtes en plus (l'autorisation).
pub fn ouvrir(url: &str, entetes: &[(&str, &str)]) -> Result<Box<dyn Canal>, String> {
    let http = if let Some(reste) = url.strip_prefix("wss://") {
        format!("https://{reste}")
    } else if let Some(reste) = url.strip_prefix("ws://") {
        format!("http://{reste}")
    } else {
        return Err(format!("pas une adresse de WebSocket : {url}"));
    };
    let adresse =
        super::sources::decouper(&http).ok_or_else(|| format!("adresse refusee : {url}"))?;
    #[cfg(windows)]
    let voie = windows::ouvrir;
    #[cfg(not(windows))]
    let voie = flux::ouvrir;
    voie(&adresse, entetes)
}

#[cfg(test)]
mod tests;
