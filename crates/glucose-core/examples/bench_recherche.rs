//! **Ce que coûte une recherche** (`Ctrl+F`, fiche 89) : un document de `n` textes d'environ deux
//! cents lettres, et trois recherches — un mot qui répond souvent, un mot rare, un mot absent.
//!
//! ```text
//! cargo run --release -p glucose-core --example bench_recherche -- 24000
//! ```

use glucose_core::recherche::{chercher, passage_du, Index};
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use std::time::Instant;

const MOTS: [&str; 16] = [
    "lumière", "onde", "énergie", "matière", "été", "eau", "courbure", "étoile", "galaxie",
    "noyau", "charge", "masse", "vitesse", "temps", "espace", "champ",
];

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(24_000);
    let mut store = Store::new("Banc");
    let board = store.project.active_board_id.clone();
    let mut g: u64 = 7;
    for i in 0..n {
        let texte: Vec<&str> = (0..30)
            .map(|_| {
                g ^= g << 13;
                g ^= g >> 7;
                g ^= g << 17;
                MOTS[(g % MOTS.len() as u64) as usize]
            })
            .collect();
        let texte = format!("Note {i}. {}.", texte.join(" "));
        store.add_annotation(&board, Annotation::text(format!("n{i}"), 0.0, 0.0, texte));
    }
    let debut = Instant::now();
    let index = Index::du(&store.project);
    println!(
        "{n} textes : l'index, une fois par version du document, {:.1} ms",
        debut.elapsed().as_secs_f64() * 1000.0
    );
    for requete in ["eau", "rare-introuvable", "galaxie vitesse"] {
        let mut meilleur = f64::MAX;
        let mut trouves = Vec::new();
        for _ in 0..5 {
            let debut = Instant::now();
            trouves = index.chercher(requete);
            // Les dix rangées qu'une palette montre.
            for t in trouves.iter().take(10) {
                std::hint::black_box(passage_du(&store.project, t, requete));
            }
            meilleur = meilleur.min(debut.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "{n} textes, « {requete} », par l'index et dix phrases : {} trouvés, {meilleur:.1} ms",
            trouves.len()
        );
    }
    for requete in ["eau", "rare-introuvable", "galaxie vitesse"] {
        let mut meilleur = f64::MAX;
        let mut trouves = 0;
        for _ in 0..5 {
            let debut = Instant::now();
            trouves = chercher(&store.project, requete).len();
            meilleur = meilleur.min(debut.elapsed().as_secs_f64() * 1000.0);
        }
        println!("{n} textes, « {requete} », d'un coup (index compris) : {trouves} trouvés, {meilleur:.1} ms");
    }
}
