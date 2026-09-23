//! **Essaie de rapatrier l'image d'une adresse**, exactement comme un dépôt le ferait.
//!
//! Le téléchargement ne se teste pas dans `cargo test` : un test qui dépend du réseau échoue
//! le jour où le réseau manque, et un test instable est pire qu'aucun test (fiche 17 § 5).
//! Ce banc le vérifie **sur la machine**, sur des adresses réelles — celles que l'utilisateur
//! a glissées —, et dit laquelle des variantes a répondu.
//!
//! ```text
//!     cargo run -p glucose-desktop --release --example essai_rapatriement -- <adresses...>
//! ```

use glucose_desktop::plateforme::{rapatrier, sources};

fn main() {
    let adresses: Vec<String> = std::env::args().skip(1).collect();
    if adresses.is_empty() {
        eprintln!("donne une ou plusieurs adresses");
        return;
    }
    println!("candidats, dans l'ordre :");
    for c in sources::candidats(&adresses) {
        println!("  {c:?}");
    }
    let depart = std::time::Instant::now();
    match rapatrier::chercher(&adresses, None) {
        Some(m) => {
            for chemin in &m.chemins {
                let taille = std::fs::metadata(chemin).map(|m| m.len()).unwrap_or(0);
                println!(
                    "rapatriee en {:.0} ms : {} ({} Ko)",
                    depart.elapsed().as_secs_f64() * 1000.0,
                    chemin.display(),
                    taille / 1024
                );
            }
        }
        None => println!("rien trouve -- le depot retomberait sur son repli"),
    }
}
