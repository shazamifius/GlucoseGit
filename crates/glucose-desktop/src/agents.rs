//! **La porte des agents** (fiche 75) : par où un agent — Protéo, Claude, tout programme qui
//! parle MCP — lit le tableau que l'utilisateur regarde et y travaille, **sous ses yeux**.
//!
//! # Le chemin d'un appel
//!
//! ```text
//! agent ──stdio──► glucose --mcp (le relais) ──porte locale──► fil de la connexion
//!                                                                  │ protocole (pur)
//!                                         boîte aux lettres ◄──────┘ un appel d'outil
//!                                                │ réveil
//!                         boucle de Glucose : l'accord, puis l'outil, en un geste du journal
//! ```
//!
//! * **La porte** ([`porte`]) n'est pas un port réseau : un tuyau nommé sous Windows, une
//!   prise Unix ailleurs. Aucune page web ne peut l'atteindre (le piège des serveurs locaux
//!   en HTTP, que la spécification MCP doit défendre par l'en-tête `Origin`), et le système ne
//!   l'ouvre qu'à l'utilisateur qui a lancé Glucose (AGENTS-2).
//! * **Le relais** ([`relais`]) : un client MCP lance un programme et lui parle par ses flux
//!   standard. `glucose --mcp` est ce programme ; il ne fait que recopier les lignes entre ses
//!   flux et la porte — et ouvre Glucose s'il ne l'est pas.
//! * **Le protocole** ([`protocole`]) est pur : il lit une ligne et dit quoi faire.
//! * **La boucle** est le seul fil qui touche au document : un appel d'outil y arrive par une
//!   boîte aux lettres ([`Boite`]), comme le choix d'un sélecteur de fichiers (DIAL-2).
//!
//! # AGENTS-1 — rien sans son accord
//!
//! La première fois qu'un agent appelle un outil, Glucose demande à l'utilisateur s'il
//! l'autorise (une question dessinée, QUESTION-1) ; l'appel attend sa réponse. Le choix vaut
//! jusqu'à la fermeture de Glucose. Lire le tableau compte : son contenu est à lui.
//!
//! # AGENTS-2 — la porte n'est qu'à lui
//!
//! Sous Windows, le tuyau ne s'ouvre qu'au compte qui l'a créé, refuse les clients du réseau,
//! et le relais vérifie que celui qui le tient est bien du même compte — un autre compte de la
//! machine qui aurait pris le nom le premier ne reçoit rien. Ailleurs, la prise vit dans un
//! dossier que seul l'utilisateur peut ouvrir.

pub mod executer;
pub mod json;
pub mod lecture;
pub mod outils;
pub mod porte;
pub mod protocole;
#[cfg(not(target_os = "android"))]
pub mod relais;

use crate::plateforme::Reveil;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex, PoisonError};

/// **Un appel d'outil en route vers la boucle**, et où écrire sa réponse — la ligne entière,
/// prête à partir.
pub struct Demande {
    pub appel: protocole::Appel,
    pub reponse: mpsc::Sender<String>,
}

/// **La boîte aux lettres** entre les fils de la porte et la boucle de Glucose.
#[derive(Clone)]
pub struct Boite {
    file: Arc<Mutex<VecDeque<Demande>>>,
    reveil: Reveil,
}

impl Boite {
    pub fn nouvelle(reveil: Reveil) -> Self {
        Self {
            file: Arc::default(),
            reveil,
        }
    }

    /// Dépose une demande, et réveille la boucle qui dort peut-être.
    pub fn deposer(&self, demande: Demande) {
        self.file
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_back(demande);
        (self.reveil)();
    }

    /// Ce qui est arrivé depuis la dernière relève, dans l'ordre.
    pub fn relever(&self) -> Vec<Demande> {
        self.file
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
            .collect()
    }
}
