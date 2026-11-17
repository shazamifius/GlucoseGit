//! **La salle d'un canevas partagé, vue du client** (fiche 117) : ce qui part d'un Glucose vers la
//! salle du relais (`outils/porte/src/salle.js`) et ce qui en arrive — **pur**, sans réseau ni
//! hasard, pour s'éprouver entièrement ici.
//!
//! * [`code`] : un code d'invitation, d'où tout se dérive — la salle, la clé, le secret d'écriture ;
//! * [`lettre`] : le message en clair — qui parle, et la base, un geste ou un curseur ;
//! * [`morceaux`] : la lettre scellée en morceaux, et le réassemblage à l'arrivée.
//!
//! Ce que le magasin fait des gestes — les mettre en attente, rejouer l'ordre de la salle, défaire
//! les siens — vit dans `store::participation`.

pub mod code;
pub mod lettre;
pub mod morceaux;

#[cfg(test)]
mod tests;
