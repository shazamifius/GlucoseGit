//! **L'oracle sur les vrais documents** : notre lecteur des fichiers de Glucose Tauri contre la
//! bibliothèque de référence d'Automerge, arbre contre arbre, puis la traduction en projet et
//! la résolution de chaque image.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example oracle_tauri -- <fichier.glucose>...
//! ```

#[path = "../tests/commun/oracle_automerge.rs"]
mod oracle;

use glucose_core::persist::tauri;
use glucose_desktop::tauri as disque;
use std::time::Instant;

fn main() {
    let magasin = disque::magasin_tauri();
    for chemin in std::env::args().skip(1) {
        println!("── {chemin}");
        let octets = match std::fs::read(&chemin) {
            Ok(o) => o,
            Err(e) => {
                println!("   illisible : {e}");
                continue;
            }
        };
        let t = Instant::now();
        let lu = tauri::lire(&octets);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let (valeur, fin) = match lu {
            Ok(v) => v,
            Err(e) => {
                println!("   NOTRE LECTEUR ÉCHOUE : {e}");
                continue;
            }
        };
        println!(
            "   notre lecteur : {ms:.2} ms, {} octets de fin ignorés",
            fin
        );
        if tauri::reconnaitre(&octets) == Some(tauri::Forme::Automerge) {
            let t = Instant::now();
            match oracle::reference(&octets) {
                Ok(reference) => {
                    let ms = t.elapsed().as_secs_f64() * 1e3;
                    match oracle::premiere_difference(&valeur, &reference, "racine") {
                        None => println!("   référence ({ms:.2} ms) : IDENTIQUE, arbre pour arbre"),
                        Some(d) => println!("   référence ({ms:.2} ms) : DIFFÈRE — {d}"),
                    }
                }
                Err(e) => println!("   la référence échoue : {e}"),
            }
        }
        let (projet, rapport) = tauri::projet::traduire(&valeur);
        let dossier = std::path::Path::new(&chemin).parent();
        let mut lues = 0;
        let mut fautes = Vec::new();
        for (id, p) in &rapport.provenances {
            match disque::resoudre(p, dossier, magasin.as_deref()) {
                Ok(_) => lues += 1,
                Err(e) => fautes.push(format!("{id} : {e}")),
            }
        }
        let n_ann: usize = projet.boards.iter().map(|b| b.annotations.len()).sum();
        println!(
            "   projet « {} » : {} tableau(x), {} image(s) dont {} lues, {} annotation(s), omis {:?}",
            projet.name,
            projet.boards.len(),
            rapport.provenances.len(),
            lues,
            n_ann,
            rapport.omis
        );
        for f in fautes.iter().take(5) {
            println!("   image manquante — {f}");
        }
    }
}
