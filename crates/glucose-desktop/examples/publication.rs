//! **Ce fichier est-il signé par la clé de Glucose ?** — la question que ses utilisateurs de
//! Glucose Tauri poseront à chaque installeur que la bascule leur apportera (fiche 48).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example verifier_la_signature -- <fichier>...
//! ```
//!
//! Chaque `<fichier>` doit avoir son `<fichier>.sig` à côté, comme le signataire de Tauri l'écrit.
//! La clé est celle que Glucose porte (`mise_a_jour::CLE_PUBLIQUE`) — celle de Glucose Tauri.
//!
//! # Pourquoi cet outil existe
//!
//! La clé secrète de Glucose vit dans les secrets du dépôt, où personne ne peut la relire : la
//! publication signe par elle, sans la voir. Rien ne dit, sans cet outil, que c'est bien **la**
//! clé dont Glucose Tauri et Glucose Rust portent la moitié publique. Une signature par une autre
//! clé ne se verrait que chez ses utilisateurs : une mise à jour refusée par tous. Sort en erreur
//! au premier fichier qui ne passe pas.

use glucose_desktop::mise_a_jour::{verifier, CLE_PUBLIQUE};

fn main() {
    let fichiers: Vec<String> = std::env::args().skip(1).collect();
    if fichiers.is_empty() {
        eprintln!("usage : verifier_la_signature <fichier>...");
        std::process::exit(2);
    }
    for fichier in &fichiers {
        let octets = std::fs::read(fichier).unwrap_or_else(|e| {
            eprintln!("{fichier} : {e}");
            std::process::exit(1)
        });
        let signature = std::fs::read_to_string(format!("{fichier}.sig")).unwrap_or_else(|e| {
            eprintln!("{fichier}.sig : {e}");
            std::process::exit(1)
        });
        match verifier(&octets, signature.trim(), CLE_PUBLIQUE) {
            Ok(()) => println!("signé par la clé de Glucose : {fichier}"),
            Err(e) => {
                eprintln!("{fichier} : {e}");
                std::process::exit(1)
            }
        }
    }
}
