//! **Ouvrir un document** : l'ancienne lecture — tout lire, tout hacher — contre l'histoire
//! (HISTOIRE-1), qui lit la base et la queue et laisse les images sur le disque. Lecture
//! seule : aucun fichier n'est touché.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_ouverture -- <fichier.glucose>...
//! ```

use glucose_core::persist::{self, histoire};
use glucose_desktop::persist::objets::{Objets, Source};
use std::time::Instant;

fn main() {
    for chemin in std::env::args().skip(1) {
        println!("── {chemin}");
        let t = Instant::now();
        let ancien = std::fs::read(&chemin).map(|o| (o.len(), persist::decode(&o).is_ok()));
        let ancien_ms = t.elapsed().as_secs_f64() * 1e3;
        let Ok((taille, lisible)) = ancien else {
            println!("   illisible");
            continue;
        };
        println!(
            "   ancienne lecture (tout lire, tout hacher) : {ancien_ms:.1} ms pour {:.1} Mo, {}",
            taille as f64 / 1e6,
            if lisible { "lu" } else { "REFUSÉ" }
        );

        let t = Instant::now();
        let f = std::fs::File::open(&chemin).unwrap();
        let ouvert = match histoire::ouvrir(&mut std::io::BufReader::new(f)) {
            Ok(o) => o,
            Err(e) => {
                println!("   histoire : REFUSÉ — {e}");
                continue;
            }
        };
        let neuf_ms = t.elapsed().as_secs_f64() * 1e3;
        let images: usize = ouvert.projet.toutes_les_images().count();
        println!(
            "   histoire : {neuf_ms:.2} ms — {images} images, {} objets, {} liens, {} gestes, fin ignorée {} o",
            ouvert.objets.len(),
            ouvert.liens.len(),
            ouvert.gestes.len(),
            ouvert.fin_ignoree
        );

        // Une image relue par sa tranche, empreinte vérifiée — ce que l'atelier fera.
        let objets = Objets::nouveau();
        objets.porter(Some(chemin.clone().into()));
        let mut pire: (f64, usize) = (0.0, 0);
        let mut lues = 0;
        for (cle, empreinte) in &ouvert.liens {
            let Some(t) = ouvert.objets.get(empreinte) else {
                continue;
            };
            objets.poser(
                cle,
                Source::Tranche {
                    empreinte: *empreinte,
                    offset: t.offset,
                    longueur: t.longueur,
                },
            );
            let debut = Instant::now();
            if objets.lire(cle).is_some() {
                lues += 1;
            }
            let ms = debut.elapsed().as_secs_f64() * 1e3;
            if ms > pire.0 {
                pire = (ms, t.longueur as usize);
            }
        }
        println!(
            "   {lues} images relues par leur tranche, vérifiées ; la plus lente {:.2} ms pour {:.2} Mo",
            pire.0,
            pire.1 as f64 / 1e6
        );
    }
}
