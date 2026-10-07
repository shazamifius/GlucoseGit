//! **Essaie de rapatrier l'image d'une adresse**, exactement comme un dépôt le ferait.
//!
//! Le téléchargement ne se teste pas dans `cargo test` : un test qui dépend du réseau échoue
//! le jour où le réseau manque, et un test instable est pire qu'aucun test (fiche 17 § 5).
//! Ce banc le vérifie **sur la machine**, sur des adresses réelles — celles que l'utilisateur
//! a glissées —, et dit laquelle des variantes a répondu.
//!
//! ```text
//!     cargo run -p glucose-desktop --release --example essai_rapatriement -- <adresses...>
//!     cargo run -p glucose-desktop --release --example essai_rapatriement -- --a-la-suite <adresses...>
//! ```
//!
//! Sans option, les adresses sont **un** dépôt, et chacune est d'abord téléchargée seule et
//! chronométrée : le 07/10, une page d'épingle mettait 20 s à venir nue, et c'est ce détail qui
//! a montré où le temps passait. Avec `--a-la-suite`, chaque adresse est **un dépôt de plus**,
//! dans le même processus — comme une série d'épingles glissées l'une après l'autre, où les
//! connexions déjà ouvertes servent encore (fiche 53 § 7).

use glucose_desktop::plateforme::{rapatrier, sources};

fn main() {
    let mut adresses: Vec<String> = std::env::args().skip(1).collect();
    let a_la_suite = adresses.first().is_some_and(|a| a == "--a-la-suite");
    if a_la_suite {
        adresses.remove(0);
    }
    if adresses.is_empty() {
        eprintln!("donne une ou plusieurs adresses");
        return;
    }
    if a_la_suite {
        for a in &adresses {
            chercher(std::slice::from_ref(a));
        }
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
    chercher(&adresses);
}

/// Un dépôt : la recherche entière, chronométrée. L'image arrive en mémoire (DEPOT-4) : le banc
/// dit son nom, son poids et sa taille.
fn chercher(adresses: &[String]) {
    let depart = std::time::Instant::now();
    match rapatrier::chercher(adresses, None) {
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
