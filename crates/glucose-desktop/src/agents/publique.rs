//! **La porte publique** (fiche 116) : par où une IA du nuage — Claude.ai, ChatGPT, toute IA qui
//! parle MCP sur le web — lit et écrit le document ouvert, sous les yeux de l'utilisateur.
//!
//! # Le chemin d'une demande
//!
//! ```text
//! IA du nuage ──https──► relais (Cloudflare, outils/porte) ──WebSocket sortant──► lien (un fil)
//!                                                                                   │ http (pur)
//!                                        guichet (OAuth) ◄──────────────────────────┤
//!                                        boîte aux lettres ◄── un appel d'outil ────┘
//!                                                │
//!                         boucle de Glucose : l'outil, en un geste du journal, signé de son nom
//! ```
//!
//! * **Le relais ne lit rien, ne garde rien** : il passe chaque demande au Glucose dont l'adresse
//!   la porte, et sa réponse en retour. Il ne sert qu'une page à lui : celle qui dit « va dans
//!   Glucose », avec un code.
//! * **Chaque Glucose est son propre serveur d'autorisation** ([`oauth`]) : il donne les clés, les
//!   vérifie, et les retire — à la seconde, chez lui.
//! * **Le protocole est celui de la porte locale** ([`super::protocole`]), dans une **portée**
//!   ([`super::outils::Portee`]) : le document ouvert seulement, et ce que l'utilisateur a choisi
//!   — *Lire et écrire* ou *Lire seulement*.
//!
//! # PUBLIQUE-1 — rien ne s'ouvre sans lui
//!
//! La porte est fermée par défaut. Ouverte, une IA n'entre qu'avec une clé que Glucose lui a
//! donnée après **son** clic, dans Glucose, devant le même code que la page de l'IA ; elle signe
//! chacun de ses gestes de son nom vérifié (« Claude (claude.ai) ») ; un clic retire sa clé.

pub mod accords;
pub mod http;
pub mod lien;
pub mod oauth;

/// **Le relais de Glucose**, sur le compte Cloudflare de l'auteur (`outils/porte`). Un fork, une
/// école, peuvent avoir le leur : `GLUCOSE_RELAIS` le nomme — et les épreuves, un relais local.
pub const RELAIS: &str = "https://glucose-porte.ferme-nilslamber.workers.dev";

pub fn relais() -> String {
    std::env::var("GLUCOSE_RELAIS")
        .ok()
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| RELAIS.to_string())
}

/// **Les adresses d'un Glucose** sur son relais — toutes tirées de deux choses : le relais, et
/// l'identifiant de ce Glucose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adresses {
    racine: String,
    id: String,
}

impl Adresses {
    pub fn nouvelles(racine: &str, id: &str) -> Self {
        Self {
            racine: racine.trim_end_matches('/').to_string(),
            id: id.to_string(),
        }
    }

    /// La base de ce Glucose : l'émetteur de ses clés (RFC 8414 `issuer`).
    pub fn base(&self) -> String {
        format!("{}/g/{}", self.racine, self.id)
    }

    /// **L'adresse à donner à une IA** : celle de sa porte MCP.
    pub fn mcp(&self) -> String {
        format!("{}/mcp", self.base())
    }

    /// Ce que la ressource dit d'elle (RFC 9728 § 3.1 : à la racine, le chemin à la suite).
    pub fn metadonnees(&self) -> String {
        format!(
            "{}/.well-known/oauth-protected-resource/g/{}/mcp",
            self.racine, self.id
        )
    }

    /// Le WebSocket par lequel Glucose se relie.
    pub fn websocket(&self) -> String {
        let racine = self
            .racine
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        format!("{racine}/g/{}/glucose", self.id)
    }
}

#[cfg(test)]
mod tests;
