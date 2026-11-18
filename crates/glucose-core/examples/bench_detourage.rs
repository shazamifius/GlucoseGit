//! **Le banc du détourage** (DETOURER-1, fiche 121) : le temps du moteur sur une image, et l'image
//! détourée, pour la regarder.
//!
//! ```text
//! cargo run --release -p glucose-core --example bench_detourage -- <entrée.rvba> <l> <h> <sortie.rvba> [fils] [fond]
//! ```
//!
//! Les fichiers sont des pixels RVBA bruts (le noyau ne lit aucun format d'image) ;
//! `outils/detourage/banc.py` les fait depuis des PNG ou JPEG et les y remet. `fond` : retirer le
//! fond uni au lieu de détourer.

use glucose_core::detourage;
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rvba = std::fs::read(&a[1]).expect("l'entrée");
    let (l, h): (usize, usize) = (a[2].parse().expect("l"), a[3].parse().expect("h"));
    let fils = a
        .get(5)
        .and_then(|f| f.parse().ok())
        .unwrap_or_else(detourage::fils_disponibles);
    let t = Instant::now();
    let sortie = if a.get(6).map(String::as_str) == Some("fond") {
        detourage::fond_uni::retirer(&rvba, (l, h)).expect("un fond")
    } else {
        let modele = detourage::embarque().expect("le modèle");
        let lu = t.elapsed();
        let m = Instant::now();
        let masque = detourage::masque(modele, &rvba, (l, h), fils);
        let dm = m.elapsed();
        let p = Instant::now();
        let s = detourage::detourer(modele, &rvba, (l, h), fils);
        eprintln!(
            "lecture du modèle {:.0} ms ; masque {:.0} ms ; tout (masque et premier plan) {:.0} ms ; {fils} fils ; {} pixels opaques",
            lu.as_secs_f64() * 1e3,
            dm.as_secs_f64() * 1e3,
            p.elapsed().as_secs_f64() * 1e3,
            masque.iter().filter(|&&a| a > 0.5).count()
        );
        s
    };
    eprintln!(
        "{} : {:.0} ms en tout",
        a[4],
        t.elapsed().as_secs_f64() * 1e3
    );
    std::fs::write(&a[4], sortie).expect("la sortie");
}
