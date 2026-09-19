//! La mesure qui décide de TUILE-1, avant d'écrire une ligne du cache.
//!
//! # La question
//!
//! Un cache de tuiles ne dessine plus la scène : il **compose** un écran à partir de carrés
//! déjà peints. Toute l'architecture repose donc sur une seule grandeur — ce que cette
//! composition coûte — et sur la façon dont elle varie :
//!
//! * **à l'échelle exacte**, une tuile se pose d'un pixel pour un pixel. C'est le chemin le
//!   moins cher que le programme connaisse, et c'est le régime de l'arrêt ;
//! * **entre deux niveaux dyadiques**, il faut mettre à l'échelle d'un facteur compris entre
//!   un et deux. Les lectures restent locales, contrairement au rééchantillonnage depuis une
//!   photo dix fois trop grande. C'est le régime du geste ;
//! * et la **taille de tuile** décide du reste : trop petite, on paie un appel par carré ;
//!   trop grande, une invalidation jette du travail utile. Le point d'équilibre se mesure.
//!
//! Si la composition à l'échelle exacte ne tient pas largement dans le budget, TUILE-1 ne
//! sert à rien et il faut chercher ailleurs. C'est la réponse que ce banc rapporte.

use glucose_core::occlusion::Boite;
use glucose_core::report::{reporter, Filtre, Melange, Pose, Vue, VueMut};
use std::time::Instant;
use tiny_skia::{Color, Pixmap};

/// Les définitions d'écran à couvrir, et le budget qu'elles ont.
const ECRANS: [(u32, u32, &str); 3] = [
    (1920, 1080, "1920 × 1080"),
    (2560, 1600, "2560 × 1600"),
    (3840, 2160, "3840 × 2160 (4K)"),
];

/// Les tailles de tuile à comparer. Le banc dit laquelle gagne ; aucune n'est postulée.
const TAILLES: [u32; 4] = [128, 256, 512, 1024];

/// Cent quarante images par seconde, en millisecondes.
const BUDGET_MS: f64 = 1000.0 / 140.0;

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Une tuile peinte, opaque, avec un motif pour que rien ne soit optimisé en passant.
fn tuile(cote: u32, graine: u32) -> Pixmap {
    let mut p = Pixmap::new(cote, cote).expect("une tuile");
    p.fill(Color::from_rgba8(
        (graine * 37 % 200 + 30) as u8,
        (graine * 71 % 200 + 30) as u8,
        (graine * 13 % 200 + 30) as u8,
        255,
    ));
    p
}

/// Compose un écran entier à partir de tuiles, et rend le temps que ça prend.
///
/// `facteur` vaut 1 pour le chemin exact, ou une valeur entre 1 et 2 pour le régime de geste.
fn composer(ecran: &mut Pixmap, tuiles: &[Pixmap], cote: u32, facteur: f32, filtre: Filtre) -> f64 {
    let (w, h) = (ecran.width(), ecran.height());
    let pose_cote = cote as f32 * facteur;
    let debut = Instant::now();
    let (octets, _) = ecran.data_mut().as_chunks_mut::<4>();
    let mut vue = VueMut::nouvelle(octets, w, h).expect("l'ecran");
    let clip = Boite::nouvelle(0.0, 0.0, w as f32, h as f32);
    let mut n = 0usize;
    let mut y = 0.0f32;
    while y < h as f32 {
        let mut x = 0.0f32;
        while x < w as f32 {
            let src = &tuiles[n % tuiles.len()];
            let (octets_src, _) = src.data().as_chunks::<4>();
            let source = Vue::nouvelle(octets_src, cote, cote).expect("une tuile");
            reporter(
                &mut vue,
                &source,
                Pose {
                    // Une position entière : c'est ce que le cache garantit, et c'est la
                    // condition du chemin exact.
                    x: x.floor(),
                    y: y.floor(),
                    largeur: pose_cote,
                    hauteur: pose_cote,
                },
                clip,
                Melange::Remplacer,
                filtre,
            );
            n += 1;
            x += pose_cote;
        }
        y += pose_cote;
    }
    debut.elapsed().as_secs_f64() * 1000.0
}

fn mesurer(ecran: &mut Pixmap, tuiles: &[Pixmap], cote: u32, facteur: f32, filtre: Filtre) -> f64 {
    let mut temps = Vec::new();
    for i in 0..13 {
        let ms = composer(ecran, tuiles, cote, facteur, filtre);
        if i >= 3 {
            temps.push(ms);
        }
    }
    mediane(temps)
}

fn main() {
    println!("Composer un ecran a partir de tuiles — la mesure qui decide de TUILE-1\n");
    println!("  Budget vise : {BUDGET_MS:.2} ms par image (140 par seconde)\n");
    println!(
        "  {:<18} {:>7} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "ecran", "tuile", "exact", "x1.1 lisse", "x1.1 proch", "x1.5 lisse", "x1.5 proch"
    );

    for (w, h, nom) in ECRANS {
        let mut ecran = Pixmap::new(w, h).expect("un ecran");
        for cote in TAILLES {
            // Seize tuiles distinctes suffisent à empêcher le cache du processeur de garder
            // la seule et même source : un cache de tuiles réel en manipule des centaines.
            let tuiles: Vec<Pixmap> = (0..16).map(|g| tuile(cote, g)).collect();
            let combien = w.div_ceil(cote) * h.div_ceil(cote);

            let _ = combien;
            let exact = mesurer(&mut ecran, &tuiles, cote, 1.0, Filtre::Lisse);
            let peu_lisse = mesurer(&mut ecran, &tuiles, cote, 1.1, Filtre::Lisse);
            let peu_proche = mesurer(&mut ecran, &tuiles, cote, 1.1, Filtre::PlusProche);
            let loin_lisse = mesurer(&mut ecran, &tuiles, cote, 1.5, Filtre::Lisse);
            let loin_proche = mesurer(&mut ecran, &tuiles, cote, 1.5, Filtre::PlusProche);

            println!(
                "  {:<18} {:>5}px {:>8.3}ms {:>8.3}ms {:>8.3}ms {:>8.3}ms {:>8.3}ms",
                nom, cote, exact, peu_lisse, peu_proche, loin_lisse, loin_proche
            );
        }
        println!();
    }

    println!("  Ce que ces colonnes veulent dire\n");
    println!("  « exact »  : les tuiles se posent a l'echelle 1, sur des entiers — le regime");
    println!("               de l'arret, et celui d'un zoom pose sur une puissance de deux.");
    println!("  « x1.5 »   : entre deux niveaux dyadiques, le pire cas du regime de geste.");
    println!("  « part »   : ce que la composition seule prend du budget de 140 images/s.");
    println!();
    println!("  Si « exact » depasse le budget, aucun cache de tuiles ne tiendra la cadence :");
    println!("  le cout viendrait de la composition, que le cache ne supprime pas.");
}
