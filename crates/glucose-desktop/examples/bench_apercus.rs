//! **Ce que les aperçus changent à l'ouverture d'un document** (ETAGES-4, fiche 32).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_apercus -- document.glucose dossier
//! ```
//!
//! Deux sessions sur le même document, par le vrai magasin : la première décode tout et écrit
//! les vues d'ensemble dans `dossier` ; la seconde les relit. Chacune dit combien de temps il
//! faut pour que **toutes** les images du tableau aient quelque chose à montrer, et ce que les
//! aperçus prennent sur le disque. L'écran est celui de l'utilisateur, 2160 × 1350.

use glucose_desktop::renderer::magasin::Magasin;
use std::time::{Duration, Instant};

const ECRAN: (u32, u32) = (2160, 1350);

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(document), Some(dossier)) = (args.next(), args.next()) else {
        println!("donner le document et le dossier des apercus");
        return;
    };
    let dossier = std::path::PathBuf::from(dossier);
    let _ = std::fs::remove_dir_all(&dossier);
    let fichier = match glucose_desktop::persist::read_project_file(std::path::Path::new(&document))
    {
        Ok(f) => f,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    let mut store = glucose_core::store::Store::new("banc");
    store.load_project(fichier.project);
    let images: Vec<(String, f64)> = store
        .active_board()
        .map(|b| {
            b.images
                .iter()
                .filter_map(|i| Some((i.src.clone()?, i.width)))
                .collect()
        })
        .unwrap_or_default();
    println!("{} images dans le tableau\n", images.len());

    for session in [
        "premiere (decoder, puis ecrire)",
        "seconde (relire les apercus)",
    ] {
        let mut magasin = Magasin::nouveau();
        magasin.brancher_les_apercus(dossier.clone());
        let t = Instant::now();
        let tout_la = loop {
            magasin.ouvrir();
            magasin.recolter();
            magasin.regler_la_vue_d_ensemble(&store, ECRAN);
            let la = images
                .iter()
                .filter(|(src, largeur)| magasin.reclamer(src, *largeur))
                .count();
            magasin.fermer();
            if la == images.len() || t.elapsed() > Duration::from_secs(60) {
                break t.elapsed();
            }
            std::thread::yield_now();
        };
        magasin.attendre_le_chantier();
        println!(
            "session {session} : toutes les images ont de quoi se montrer en {:.0} ms",
            tout_la.as_secs_f64() * 1e3
        );
    }
    let (fichiers, octets) = std::fs::read_dir(&dossier)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter_map(|e| e.metadata().ok())
                .fold((0, 0u64), |(n, o), m| (n + 1, o + m.len()))
        })
        .unwrap_or_default();
    println!(
        "\napercus sur le disque : {fichiers} fichiers, {:.1} Mo",
        octets as f64 / 1_048_576.0
    );
}
