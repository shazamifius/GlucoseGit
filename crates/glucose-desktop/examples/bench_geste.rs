//! Le banc du **geste** — ce que coûte une image pendant qu'on zoome, et non à l'arrêt.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_geste
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Le banc de frame annonce moins d'une milliseconde par image, et l'application est décrite
//! comme « laggy de fou » sur une bonne machine. Un tel écart ne se comble pas en optimisant :
//! il se comble en mesurant ce que le banc ne mesurait pas. Or le banc de frame rend toujours
//! **la même image** — il est donc structurellement aveugle à tout ce qui dépend du changement
//! de cadrage, à commencer par les caches dont la clé le contient.
//!
//! La charte l'avait posé avant qu'on s'en aperçoive : *« pendant le mouvement, c'est une
//! question à peser, tester et mesurer — surtout pas à trancher sur le papier »*.
//!
//! # Ce que la comparaison isole
//!
//! Trois colonnes pour le même document et la même définition :
//!
//! * **à l'arrêt** — ce que le banc de frame mesure depuis toujours ;
//! * **en zoomant** — la caméra reste dans le document, les bornes de la minimap ne bougent pas ;
//! * **en dézoomant** — la caméra finit par déborder le contenu, et ce qui dépend du cadrage
//!   doit se refaire.
//!
//! Si la troisième colonne décroche des deux autres, le coupable est un cache lié au cadrage,
//! et il est nommé par construction. Si les trois se ressemblent, l'hypothèse est fausse et le
//! coût est ailleurs — dans la présentation du framebuffer, que seul l'application mesure.

use glucose_core::synth::{self, Shape};
use glucose_desktop::bench::{self, BUDGET_MS};
use tiny_skia::Pixmap;

/// Les tailles mesurées. Un document de mille nœuds est déjà le cas courant.
const TAILLES: &[usize] = &[1_000, 10_000];

/// Combien d'images par geste. Un zoom à la main dure une bonne seconde, soit une centaine
/// d'images — assez pour que la médiane veuille dire quelque chose.
const IMAGES: usize = 120;

/// La même graine que le banc de frame : deux bancs qui mesurent des documents différents ne
/// se comparent pas.
const GRAINE: u64 = 0x91ac05e;

/// L'étendue croît avec la taille, pour garder une densité comparable — reprise du banc de
/// frame, où elle est expliquée.
fn span_pour(n: usize) -> f64 {
    2_000.0 * (n as f64).sqrt()
}

fn main() {
    let definitions: &[(&str, u32, u32)] = &[("1080p", 1920, 1080), ("4K", 3840, 2160)];

    println!("Banc du geste — médiane sur {IMAGES} images, budget {BUDGET_MS} ms\n");
    println!(
        "{:<8} {:>8} {:>11} {:>11} {:>11} {:>11}   pire image",
        "déf.", "nœuds", "arrêt ×1", "arrêt ×0,02", "en zoomant", "en dézoomant"
    );

    for (nom, w, h) in definitions {
        for taille in TAILLES {
            let mut store = synth::document(*taille, span_pour(*taille), Shape::Clustered, GRAINE);

            bench::frame_document(&mut store, 1.0, *w, *h);
            let arret = bench::measure(&store, *w, *h, IMAGES);

            // La comparaison honnête du dézoom : à l'arrêt, **à la même échelle**. Sans elle
            // on attribuerait au mouvement ce qui n'est que le nombre de nœuds devenus
            // visibles — au très fort dézoom, le culling ne retient plus rien.
            bench::frame_document(&mut store, 0.02, *w, *h);
            let arret_loin = bench::measure(&store, *w, *h, IMAGES);

            // Vers le zoom : la caméra reste au milieu du contenu.
            let zoom = bench::measure_sweep(&mut store, *w, *h, IMAGES, (1.0, 4.0));
            // Vers le dézoom : la caméra finit par voir plus large que le document.
            let dezoom = bench::measure_sweep(&mut store, *w, *h, IMAGES, (1.0, 0.02));

            println!(
                "{:<8} {:>8} {:>8.2} ms {:>8.2} ms {:>8.2} ms {:>8.2} ms   {:.2} ms",
                nom,
                taille,
                arret.median_ms,
                arret_loin.median_ms,
                zoom.median_ms,
                dezoom.median_ms,
                dezoom.max_ms
            );
        }
    }

    println!(
        "\nLe budget de {BUDGET_MS} ms vaut pour les trois colonnes : un geste n'a pas droit à \
         moins d'images qu'un document au repos."
    );

    mesure_du_transfert();
}

/// Ce que coûte la **conversion** du tampon, seul poste d'une image que le banc n'a jamais vu.
///
/// `blit_and_present` traduit chaque pixel du format de `tiny-skia` vers celui de la fenêtre,
/// un par un. C'est du calcul pur : il se mesure sans fenêtre, contrairement à la
/// présentation elle-même. En 4K cela fait 8,3 millions de pixels à chaque image — un coût de
/// **surface**, que ni le culling ni aucun cache ne réduiront jamais.
fn mesure_du_transfert() {
    println!(
        "
La conversion du tampon, payée à chaque image :"
    );
    for (nom, w, h) in [
        ("1080p", 1920u32, 1080u32),
        ("1440p", 2560, 1440),
        ("4K", 3840, 2160),
    ] {
        let pixmap = Pixmap::new(w, h).expect("un pixmap");
        let mut sortie = vec![0u32; (w * h) as usize];
        let mut par_octets = Vec::with_capacity(30);
        let mut par_mots = Vec::with_capacity(30);
        for _ in 0..30 {
            let t = std::time::Instant::now();
            let (src, _) = pixmap.data().as_chunks::<4>();
            for (dst, chunk) in sortie.iter_mut().zip(src) {
                *dst =
                    (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
            }
            par_octets.push(t.elapsed().as_secs_f64() * 1000.0);

            // La même conversion, dite en une seule opération par pixel. Un pixel de
            // `tiny-skia` vaut r,g,b,a en mémoire, donc `a<<24 | b<<16 | g<<8 | r` en mot ;
            // l'échanger bout à bout donne `r<<24 | g<<16 | b<<8 | a`, et un décalage de huit
            // bits laisse exactement `r<<16 | g<<8 | b`, ce que la fenêtre attend.
            let t = std::time::Instant::now();
            let (mots, _) = pixmap.data().as_chunks::<4>();
            for (dst, px) in sortie.iter_mut().zip(mots) {
                *dst = u32::from_le_bytes(*px).swap_bytes() >> 8;
            }
            par_mots.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        par_octets.sort_by(f64::total_cmp);
        par_mots.sort_by(f64::total_cmp);
        let millions = f64::from(w) * f64::from(h) / 1e6;
        println!(
            "  {nom:<8} octet à octet {:>6.2} ms   échange de mot {:>6.2} ms   ({millions:.1} M pixels)",
            par_octets[par_octets.len() / 2],
            par_mots[par_mots.len() / 2]
        );
    }
}
