//! **Le canevas partagé, côté application** (CLIENT-1, fiche 117) : le fil relié à la salle du
//! relais ([`lien`]). Ce que les gestes deviennent — en attente, rejoués dans l'ordre de la salle,
//! défaits par leur main — vit dans le noyau (`store::participation`) ; ce qui part et arrive, scellé
//! en morceaux, aussi (`glucose_core::salle`). La boucle les relie (`app/collaborer.rs`).

pub mod lien;

/// **Les adresses d'une salle** sur le relais : son WebSocket.
pub fn adresse(racine: &str, salle: &str) -> String {
    let racine = racine
        .trim_end_matches('/')
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    format!("{racine}/s/{salle}/salle")
}
