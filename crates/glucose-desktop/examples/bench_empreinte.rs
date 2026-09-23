//! **Deux images sont-elles la même ?** L'empreinte perceptuelle, mesurée sur de VRAIES paires
//! (fiche 27, étape 2).
//!
//! # Pourquoi ce banc existe avant le module
//!
//! La recherche d'origine (fiche 27 § 4) ne croira pas le score d'un service : elle
//! téléchargera chaque candidat et le comparera à l'image du canevas. Cette comparaison est
//! une **empreinte perceptuelle** — 64 bits qui ne bougent pas quand une image est
//! recompressée ou redimensionnée — et une distance de Hamming entre deux empreintes. Tout
//! repose alors sur **un seuil**, et un seuil ne se choisit pas : il se mesure, sur des paires
//! dont on sait la vérité.
//!
//! Ce banc les mesure. Les fichiers se nomment `<groupe>-<variante>.<ext>` : les variantes
//! d'un même groupe sont la **même** image — l'épingle en 236, en 736 et en original —, et
//! deux groupes différents sont deux images **différentes**. Un fichier sans tiret est seul
//! de son groupe. Le seuil honnête est dans l'écart entre la plus grande distance d'un même
//! groupe et la plus petite entre deux groupes ; s'il n'y a pas d'écart, aucun seuil n'existe.
//!
//! Aucun module de production n'est écrit tant que ce banc n'a pas parlé : un module sans
//! appelant est interdit (fiche 05 § 7.6), et un seuil deviné est pire qu'aucun.
//!
//! ```text
//!     cargo run -p glucose-desktop --release --example bench_empreinte -- <fichiers...>
//! ```
//!
//! # Les deux empreintes
//!
//! * **pHash**, la version de référence de phash.org : l'image en niveaux de gris réduite à
//!   32 × 32, sa transformée en cosinus, les coefficients `1..=8 × 1..=8` — la composante
//!   continue, qui ne dit que la luminosité moyenne, est écartée — comparés à leur médiane.
//! * **dHash**, celui de Neal Krawetz : l'image réduite à 9 × 8, chaque pixel comparé à son
//!   voisin de droite.

use std::collections::BTreeMap;
use std::f64::consts::PI;

fn main() {
    let mut empreintes: Vec<(String, String, u64, u64)> = Vec::new();
    for f in std::env::args().skip(1) {
        let Some(gris) = gris(&f) else {
            println!("{f} : illisible");
            continue;
        };
        let nom = std::path::Path::new(&f)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let groupe = nom.split('-').next().unwrap_or(&nom).to_string();
        empreintes.push((groupe, nom, phash(&gris), dhash(&gris)));
    }
    for (nom, choisir) in [("pHash", 2usize), ("dHash", 3usize)] {
        rapporter(nom, &empreintes, choisir);
    }
    detailler(&empreintes);
}

/// **Chaque variante contre son original** : quand un groupe porte un `<groupe>-original`, la
/// distance de chacune de ses autres variantes à lui, par nature de variante, au pire des
/// groupes. C'est ce qui dit jusqu'où la même image s'éloigne selon ce qu'on lui a fait.
fn detailler(empreintes: &[(String, String, u64, u64)]) {
    let mut par_nature: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for e in empreintes {
        let Some(original) = empreintes
            .iter()
            .find(|o| o.0 == e.0 && o.1.ends_with("-original"))
        else {
            continue;
        };
        if std::ptr::eq(e, original) {
            continue;
        }
        let nature = e.1.split_once('-').map_or("", |(_, n)| n).to_string();
        let (p, d) = (
            (e.2 ^ original.2).count_ones(),
            (e.3 ^ original.3).count_ones(),
        );
        let pire = par_nature.entry(nature).or_insert((0, 0));
        *pire = (pire.0.max(p), pire.1.max(d));
    }
    if par_nature.is_empty() {
        return;
    }
    println!("\n== chaque variante contre son original, au pire des groupes");
    println!("    variante              pHash  dHash");
    for (nature, (p, d)) in &par_nature {
        println!("    {nature:20} {p:6} {d:6}");
    }
}

/// L'image en niveaux de gris, telle que `image` la décode.
fn gris(chemin: &str) -> Option<image::GrayImage> {
    let img = image::ImageReader::open(chemin)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    Some(img.to_luma8())
}

/// **pHash** : 32 × 32, transformée en cosinus, les 8 × 8 basses fréquences hors composante
/// continue, comparées à leur médiane.
fn phash(gris: &image::GrayImage) -> u64 {
    let petit = image::imageops::resize(gris, 32, 32, image::imageops::FilterType::Triangle);
    let f = |x: usize, y: usize| f64::from(petit.get_pixel(x as u32, y as u32).0[0]);
    let base = |u: usize, x: usize| ((2 * x + 1) as f64 * u as f64 * PI / 64.0).cos();
    let mut coefficients = Vec::with_capacity(64);
    for v in 1..=8 {
        for u in 1..=8 {
            let mut somme = 0.0;
            for y in 0..32 {
                for x in 0..32 {
                    somme += f(x, y) * base(u, x) * base(v, y);
                }
            }
            coefficients.push(somme);
        }
    }
    let mut tries = coefficients.clone();
    tries.sort_by(f64::total_cmp);
    let mediane = (tries[31] + tries[32]) / 2.0;
    coefficients
        .iter()
        .enumerate()
        .fold(0u64, |bits, (i, &c)| bits | (u64::from(c > mediane) << i))
}

/// **dHash** : 9 × 8, chaque pixel plus clair que son voisin de droite donne un bit.
fn dhash(gris: &image::GrayImage) -> u64 {
    let petit = image::imageops::resize(gris, 9, 8, image::imageops::FilterType::Triangle);
    let mut bits = 0u64;
    for y in 0..8 {
        for x in 0..8 {
            let (a, b) = (petit.get_pixel(x, y).0[0], petit.get_pixel(x + 1, y).0[0]);
            bits |= u64::from(a > b) << (y * 8 + x);
        }
    }
    bits
}

/// Les distances d'un même groupe, celles entre groupes, et l'écart qui les sépare.
fn rapporter(nom: &str, empreintes: &[(String, String, u64, u64)], choisir: usize) {
    let bits = |e: &(String, String, u64, u64)| if choisir == 2 { e.2 } else { e.3 };
    let mut dedans: BTreeMap<String, u32> = BTreeMap::new();
    let mut entre: Vec<(u32, String, String)> = Vec::new();
    for (i, a) in empreintes.iter().enumerate() {
        for b in &empreintes[i + 1..] {
            let d = (bits(a) ^ bits(b)).count_ones();
            if a.0 == b.0 {
                let pire = dedans.entry(a.0.clone()).or_insert(0);
                *pire = (*pire).max(d);
            } else {
                entre.push((d, a.1.clone(), b.1.clone()));
            }
        }
    }
    entre.sort();
    println!("\n== {nom}, sur 64 bits");
    println!("  la meme image (pire distance dans chaque groupe) :");
    for (g, d) in &dedans {
        println!("    {g:12} {d:2}");
    }
    let pire_dedans = dedans.values().copied().max().unwrap_or(0);
    let Some((meilleur_entre, a, b)) = entre.first().cloned() else {
        println!("  aucune paire entre groupes");
        return;
    };
    println!(
        "  deux images differentes, sur {} paires : la plus proche {meilleur_entre} ({a} / {b})",
        entre.len()
    );
    for (d, a, b) in entre.iter().skip(1).take(4) {
        println!("    puis {d:2}  ({a} / {b})");
    }
    let mediane = entre[entre.len() / 2].0;
    println!("    mediane {mediane}");
    if meilleur_entre > pire_dedans {
        println!(
            "  => un ecart existe : la meme image jusqu'a {pire_dedans}, deux images a partir de {meilleur_entre}"
        );
    } else {
        println!("  => AUCUN ecart : {pire_dedans} dedans contre {meilleur_entre} entre -- pas de seuil honnete");
    }
}
