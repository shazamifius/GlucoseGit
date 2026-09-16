//! Ce que coûte **poser une image**, selon la façon de la poser.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_blit
//! ```
//!
//! # Pourquoi mesurer une primitive
//!
//! Le banc des photos dit que trente-six images coûtent 37 ms, soit 7,8 ns par pixel de
//! destination. C'est dix fois le prix d'une copie mémoire, donc le coût n'est pas dans les
//! pixels mais dans la **façon** de les produire. Reste à savoir laquelle : le filtre, la
//! transformation, ou la distance entre deux texels lus.
//!
//! Chaque ligne isole une variable. Ce banc ne dit pas quoi faire — il dit ce qu'on paie, et
//! ce qu'on paierait autrement.

use std::time::Instant;
use tiny_skia::{BlendMode, FilterQuality, Pixmap, PixmapPaint, Transform};

/// Le tampon d'écran mesuré.
const ECRAN: (u32, u32) = (2560, 1600);
/// La taille à laquelle une image est posée, en pixels écran.
const POSEE: (u32, u32) = (420, 315);
/// Combien d'images par image-écran.
const N: usize = 36;

fn photo(largeur: u32, hauteur: u32) -> Pixmap {
    let mut p = Pixmap::new(largeur, hauteur).expect("une source");
    let octets = p.data_mut();
    for i in 0..(largeur as usize * hauteur as usize) {
        octets[i * 4] = (i % 256) as u8;
        octets[i * 4 + 1] = ((i / 7) % 256) as u8;
        octets[i * 4 + 2] = ((i / 13) % 256) as u8;
        octets[i * 4 + 3] = 255;
    }
    p
}

/// Pose `N` fois la source, en la transformant comme demandé, et rend la médiane en ms.
fn mesurer(
    source: &Pixmap,
    quality: FilterQuality,
    transforme: impl Fn(usize) -> Transform,
) -> f64 {
    mesurer_avec(source, quality, BlendMode::SourceOver, transforme)
}

fn mesurer_avec(
    source: &Pixmap,
    quality: FilterQuality,
    blend_mode: BlendMode,
    transforme: impl Fn(usize) -> Transform,
) -> f64 {
    let mut ecran = Pixmap::new(ECRAN.0, ECRAN.1).expect("un écran");
    let paint = PixmapPaint {
        quality,
        blend_mode,
        ..Default::default()
    };
    let mut temps = Vec::new();
    for _ in 0..7 {
        let t = Instant::now();
        for i in 0..N {
            ecran.draw_pixmap(0, 0, source.as_ref(), &paint, transforme(i), None);
        }
        temps.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    temps.sort_by(f64::total_cmp);
    temps[temps.len() / 2]
}

/// La position de la i-ième image sur l'écran, en pixels entiers.
fn place(i: usize) -> (f32, f32) {
    let colonnes = 6;
    (
        (i % colonnes) as f32 * (POSEE.0 as f32 + 8.0),
        (i / colonnes) as f32 * (POSEE.1 as f32 + 8.0),
    )
}

fn ligne(nom: &str, ms: f64) {
    let pixels = (N * POSEE.0 as usize * POSEE.1 as usize) as f64;
    println!("  {nom:<52} {ms:>7.2}ms {:>8.2} ns/px", ms * 1e6 / pixels);
}

fn main() {
    println!(
        "Coût de poser {N} images de {}×{} sur {}×{}\n",
        POSEE.0, POSEE.1, ECRAN.0, ECRAN.1
    );

    let grande = photo(4032, 3024);
    let moyenne = photo(1920, 1080);
    let exacte = photo(POSEE.0, POSEE.1);

    // ── Ce que fait l'application aujourd'hui ───────────────────────────────
    let echelle_grande = (POSEE.0 as f32 / 4032.0, POSEE.1 as f32 / 3024.0);
    ligne(
        "12 Mpx réduite à la volée, bilinéaire (ce que fait Glucose)",
        mesurer(&grande, FilterQuality::Bilinear, |i| {
            let (x, y) = place(i);
            Transform::from_scale(echelle_grande.0, echelle_grande.1).post_translate(x, y)
        }),
    );
    ligne(
        "12 Mpx réduite à la volée, au plus proche",
        mesurer(&grande, FilterQuality::Nearest, |i| {
            let (x, y) = place(i);
            Transform::from_scale(echelle_grande.0, echelle_grande.1).post_translate(x, y)
        }),
    );

    let echelle_moyenne = (POSEE.0 as f32 / 1920.0, POSEE.1 as f32 / 1080.0);
    ligne(
        "2 Mpx réduite à la volée, bilinéaire",
        mesurer(&moyenne, FilterQuality::Bilinear, |i| {
            let (x, y) = place(i);
            Transform::from_scale(echelle_moyenne.0, echelle_moyenne.1).post_translate(x, y)
        }),
    );

    // ── Ce que coûterait une source déjà à la bonne taille ──────────────────
    ligne(
        "déjà à la taille posée, translation entière, bilinéaire",
        mesurer(&exacte, FilterQuality::Bilinear, |i| {
            let (x, y) = place(i);
            Transform::from_translate(x, y)
        }),
    );
    ligne(
        "déjà à la taille posée, translation entière, au plus proche",
        mesurer(&exacte, FilterQuality::Nearest, |i| {
            let (x, y) = place(i);
            Transform::from_translate(x, y)
        }),
    );

    // ── Le facteur qui reste apres un niveau de pyramide ────────────────────
    //
    // Une pyramide dyadique laisse un facteur entre 1/2 et 1. La question est de savoir si le
    // rasteriseur a un chemin rapide pour « presque 1 », ou seulement pour « exactement 1 ».
    for (source_w, source_h, nom) in [
        (
            504u32,
            378u32,
            "source à 504 px, réduite à 420 (facteur 0,83)",
        ),
        (840, 630, "source à 840 px, réduite à 420 (facteur 0,50)"),
        (421, 316, "source à 421 px, réduite à 420 (facteur 0,998)"),
    ] {
        let src = photo(source_w, source_h);
        let e = (
            POSEE.0 as f32 / source_w as f32,
            POSEE.1 as f32 / source_h as f32,
        );
        ligne(
            nom,
            mesurer(&src, FilterQuality::Bilinear, |i| {
                let (x, y) = place(i);
                Transform::from_scale(e.0, e.1).post_translate(x, y)
            }),
        );
    }

    // ── Et une translation non entiere, comme un pan en donne ───────────────
    ligne(
        "à la bonne taille, translation de 0,5 px, bilinéaire",
        mesurer(&exacte, FilterQuality::Bilinear, |i| {
            let (x, y) = place(i);
            Transform::from_translate(x + 0.5, y + 0.5)
        }),
    );

    // ── Et si l'on cesse de composer ce qui est opaque ──────────────────────
    //
    // Une photo opaque n'a rien a melanger avec le fond : « source par-dessus » calcule un
    // melange dont le resultat est la source. La question est de savoir si le rasteriseur le
    // sait, ou s'il paie le melange quand meme.
    ligne(
        "déjà à la bonne taille, en remplacement (sans composer)",
        mesurer_avec(&exacte, FilterQuality::Bilinear, BlendMode::Source, |i| {
            let (x, y) = place(i);
            Transform::from_translate(x, y)
        }),
    );
    ligne(
        "12 Mpx réduite à la volée, bilinéaire, en remplacement",
        mesurer_avec(&grande, FilterQuality::Bilinear, BlendMode::Source, |i| {
            let (x, y) = place(i);
            Transform::from_scale(echelle_grande.0, echelle_grande.1).post_translate(x, y)
        }),
    );

    // ── Et sans passer par une transformation du tout ───────────────────────
    let mut ecran = Pixmap::new(ECRAN.0, ECRAN.1).expect("un écran");
    let paint = PixmapPaint::default();
    let mut temps = Vec::new();
    for _ in 0..7 {
        let t = Instant::now();
        for i in 0..N {
            let (x, y) = place(i);
            ecran.draw_pixmap(
                x as i32,
                y as i32,
                exacte.as_ref(),
                &paint,
                Transform::identity(),
                None,
            );
        }
        temps.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    temps.sort_by(f64::total_cmp);
    ligne(
        "déjà à la taille posée, posée à un entier, sans transformation",
        temps[temps.len() / 2],
    );
}
