//! Ce qu'une image coûte AVANT qu'on dessine quoi que ce soit.
//!
//! Effacer le fond, recopier l'image précédente, la téléverser : trois opérations qui ne
//! dessinent rien et qu'aucune optimisation d'algorithme ne rend gratuites. Elles sont
//! bornées par la mémoire, pas par le code — et c'est ce plancher qui décide si la cadence
//! visée est atteignable par le processeur, ou pas du tout.

use std::time::Instant;
use tiny_skia::{Color, Pixmap};

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    println!("Le plancher d'une image — ce qui coûte sans rien dessiner\n");
    println!(
        "  {:<22} {:>10} {:>12} {:>12} {:>12}",
        "définition", "Mo/image", "effacer", "recopier", "Go/s effacé"
    );

    for (w, h, nom) in [
        (1440u32, 900u32, "1440 × 900"),
        (2560, 1600, "2560 × 1600"),
        (3840, 2160, "3840 × 2160 (4K)"),
        (7680, 4320, "7680 × 4320 (8K)"),
    ] {
        let mo = (w as f64 * h as f64 * 4.0) / 1_048_576.0;
        let mut a = Pixmap::new(w, h).unwrap();
        let b = Pixmap::new(w, h).unwrap();

        let mut effacer = Vec::new();
        for i in 0..15 {
            let t = Instant::now();
            a.fill(Color::from_rgba8(12, 12, 14, 255));
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            if i >= 3 {
                effacer.push(ms);
            }
        }
        let effacer = mediane(effacer);

        let mut copier = Vec::new();
        for i in 0..15 {
            let t = Instant::now();
            a.data_mut().copy_from_slice(b.data());
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            if i >= 3 {
                copier.push(ms);
            }
        }
        let copier = mediane(copier);

        println!(
            "  {:<22} {:>9.1} {:>10.2}ms {:>10.2}ms {:>11.1}",
            nom,
            mo,
            effacer,
            copier,
            (mo / 1024.0) / (effacer / 1000.0)
        );
    }

    println!("\n  Ce qu'il resterait pour DESSINER, par image :");
    println!(
        "  {:<22} {:>12} {:>14} {:>14}",
        "définition", "à 400 fps", "à 144 fps", "à 60 fps"
    );
    for (w, h, nom) in [
        (2560u32, 1600u32, "2560 × 1600"),
        (3840, 2160, "3840 × 2160 (4K)"),
    ] {
        let mut a = Pixmap::new(w, h).unwrap();
        let mut effacer = Vec::new();
        for i in 0..15 {
            let t = Instant::now();
            a.fill(Color::from_rgba8(12, 12, 14, 255));
            if i >= 3 {
                effacer.push(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let e = mediane(effacer);
        let reste = |fps: f64| 1000.0 / fps - e;
        println!(
            "  {:<22} {:>10.2}ms {:>12.2}ms {:>12.2}ms",
            nom,
            reste(400.0),
            reste(144.0),
            reste(60.0)
        );
    }
    println!("\n  Un reste négatif veut dire que la cadence est hors d'atteinte\n  AVANT même qu'un seul pixel de contenu soit posé.");
}
