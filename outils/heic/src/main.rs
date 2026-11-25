//! Banc HEIC-1 (fiche 161 § 5) : `heifer` seul, hors de Glucose. Un décodage par processus,
//! pour que la mémoire de pointe soit la sienne :
//!   /run/current-system/sw/bin/time -f "%M Ko" target/release/banc-heic <mode> <fichier>
//! mode : `type` (la signature), `vignette` (le `thmb` de l'iPhone), `entier1` (un fil),
//! `entierN` (tous les cœurs). Le poids : `strip` de `banc-heic` moins celui de `vide`.
//! `apercu <heic> <png> <facteur>` écrit l'image réduite, pour la regarder.
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (mode, chemin) = (a[1].as_str(), &a[2]);
    let octets = std::fs::read(chemin).expect("lecture");
    let t = Instant::now();
    match mode {
        "type" => println!("{:?}", heifer::check_file_type(&octets)),
        "vignette" => match heifer::decode_thumbnail(&octets) {
            Ok(Some(i)) => println!("vignette {}x{}", i.width, i.height),
            Ok(None) => println!("vignette aucune"),
            Err(e) => println!("vignette erreur {e}"),
        },
        "entier1" | "entierN" => {
            let fils = if mode == "entier1" { 1 } else { 0 };
            let o = heifer::Options::new().with_max_threads(fils);
            match heifer::decode_srgb_rgba8(&octets, &o) {
                Ok(i) => println!("entier {}x{} alpha={}", i.width, i.height, i.has_alpha),
                Err(e) => println!("entier erreur {e}"),
            }
        }
        _ => panic!("mode"),
    }
    println!("{} ms", t.elapsed().as_secs_f64() * 1000.0);
}
