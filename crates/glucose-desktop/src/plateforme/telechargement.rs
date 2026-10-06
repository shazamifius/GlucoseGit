//! **Télécharger une adresse hors de Windows** : `ureq`, `rustls` et les racines de Mozilla
//! (note [`decisions/05`](../../../../docs/carnet/decisions/05-LE-RESEAU-HORS-DE-WINDOWS.md)).
//!
//! # Pourquoi une pile embarquée ici, et pas sous Windows
//!
//! Windows porte une pile HTTPS que tous ses programmes partagent, WinHTTP, et Glucose s'en sert
//! (note 02). Linux n'en a pas : chaque distribution a sa bibliothèque TLS, dans sa version, et
//! un AppImage ne peut compter sur aucune. `rustls` est écrite en Rust, les racines de Mozilla
//! voyagent avec le programme — et les mises à jour les renouvellent.
//!
//! Ce module ne décide de rien, comme celui de Windows : il reçoit une adresse déjà découpée et
//! jugée par [`super::sources`], et rend des octets ou la raison de n'en pas rendre.

use super::sources::Adresse;
use super::{DELAI, NAVIGATEUR};

/// **Les octets que cette adresse rend**, au plus `limite`, ou la raison de n'en pas rendre.
///
/// Seul un `200` compte : une redirection est suivie par `ureq` lui-même, et tout le reste —
/// une page introuvable, un refus — n'est pas ce qu'on demandait.
pub(super) fn telecharger(adresse: &Adresse, limite: usize) -> Result<Vec<u8>, String> {
    let schema = if adresse.securise { "https" } else { "http" };
    let url = format!(
        "{schema}://{}:{}{}",
        adresse.hote, adresse.port, adresse.chemin
    );
    let config = ureq::Agent::config_builder()
        .timeout_resolve(Some(DELAI))
        .timeout_connect(Some(DELAI))
        .timeout_send_request(Some(DELAI))
        .timeout_recv_response(Some(DELAI))
        .user_agent(NAVIGATEUR)
        .http_status_as_error(false)
        .build();
    let mut reponse = ureq::Agent::new_with_config(config)
        .get(&url)
        .call()
        .map_err(|e| format!("requete : {e}"))?;
    let statut = reponse.status().as_u16();
    if statut != 200 {
        return Err(format!("le serveur repond {statut}"));
    }
    reponse
        .body_mut()
        .with_config()
        .limit(limite as u64)
        .read_to_vec()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(_) => format!("plus de {} Mo", limite / (1024 * 1024)),
            autre => format!("lecture : {autre}"),
        })
}
