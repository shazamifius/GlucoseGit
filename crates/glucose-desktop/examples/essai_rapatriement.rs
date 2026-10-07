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
//!
//! Chaque adresse est aussi **téléchargée seule et chronométrée** d'abord : le 07/10, une page
//! d'épingle mettait 20 s à venir nue, et c'est ce détail qui a montré où le temps passait.

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
    for a in &adresses {
        let t = std::time::Instant::now();
        match glucose_desktop::plateforme::telecharger(a, 256 * 1024 * 1024) {
            Ok(o) => println!(
                "  la page seule : {} Ko en {:.0} ms",
                o.len() / 1024,
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => println!("  la page seule : {e}"),
        }
    }
    let depart = std::time::Instant::now();
    // L'image arrive en memoire (DEPOT-4) : le banc dit son nom, son poids et sa taille.
    match rapatrier::chercher(&adresses, None) {
        Ok(m) => {
            for recu in &m.recus {
                let taille = image::load_from_memory(&recu.octets)
                    .map(|i| format!("{} x {}", i.width(), i.height()))
                    .unwrap_or_else(|_| "illisible".into());
                println!(
                    "rapatriee en {:.0} ms : {} ({} Ko, {taille})",
                    depart.elapsed().as_secs_f64() * 1000.0,
                    recu.nom,
                    recu.octets.len() / 1024
                );
            }
        }
        Err(raison) => println!("rien trouve -- {raison} ; le depot retomberait sur son repli"),
    }
}
