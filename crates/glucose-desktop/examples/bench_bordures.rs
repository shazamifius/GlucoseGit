//! **Ce que la détection de bordures trouve sur de VRAIES images**, et ce que chaque critère
//! en fait (BORDURES-1).
//!
//! # Pourquoi ce banc, et ce qu'il a coûté de ne pas l'avoir
//!
//! Les preuves de `glucose_core::bordures` jouent des images **synthétiques** : des bandes
//! parfaitement unies, un bruit choisi, un damier franc. Elles ont fait rejeter deux critères
//! et en ont retenu un — et l'utilisateur a rapporté, sur ses propres images, que *« ça ne
//! fonctionne pas assez bien »*. Une image réelle n'est pas un damier : elle a du ringing de
//! compression sur chaque bord franc, un dégradé au lieu d'une frontière, et parfois **deux**
//! bordures l'une dans l'autre.
//!
//! Ce banc prend des fichiers réels, mesure ce que chaque critère donne, et l'écrit. Il ne
//! choisit rien : c'est en le lisant qu'on choisit.
//!
//! ```text
//!     cargo run -p glucose-desktop --release --example bench_bordures -- <fichiers...>
//! ```
//!
//! Sans argument, il fabrique ses cas — dont un **passé par un vrai encodeur JPEG**, ce
//! qu'aucun test synthétique ne fait.

use glucose_core::bordures;
use glucose_core::report::{Pixel, Vue};

/// Une image chargée : ses pixels et ses dimensions.
type Image = (Vec<Pixel>, u32, u32);

fn main() {
    let fichiers: Vec<String> = std::env::args().skip(1).collect();
    if fichiers.is_empty() {
        for (nom, (pixels, l, h)) in cas_fabriques() {
            rapporter(&nom, &pixels, l, h);
        }
        return;
    }
    for f in fichiers {
        match charger(&f) {
            Some((pixels, l, h)) => rapporter(&f, &pixels, l, h),
            None => println!("{f} : illisible"),
        }
    }
}

/// Un fichier image, en pixels prémultipliés comme le reste du projet les voit.
fn charger(chemin: &str) -> Option<Image> {
    let img = image::ImageReader::open(chemin)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (l, h) = (img.width(), img.height());
    Some((img.pixels().map(|p| p.0).collect(), l, h))
}

/// Ce que la détection trouve, et ce que chaque critère aurait trouvé.
fn rapporter(nom: &str, pixels: &[Pixel], l: u32, h: u32) {
    let Some(vue) = Vue::nouvelle(pixels, l, h) else {
        println!("{nom} : dimensions incoherentes");
        return;
    };
    let r = bordures::detecter(&vue);
    let (g, ht, d, b) = r.marges();
    let px = |part: f64, dim: u32| (part * f64::from(dim)).round() as u32;
    println!("\n{nom}  ({l} x {h})");
    println!(
        "  retire : gauche {}  haut {}  droite {}  bas {}",
        px(g, l),
        px(ht, h),
        px(d, l),
        px(b, h)
    );

    // Et, bord par bord, ce que chaque critere aurait dit ligne par ligne : c'est la colonne
    // qui montre POURQUOI un critere s'arrete.
    println!("  le haut, ligne par ligne (ecart a la mediane de la ligne 0) :");
    let ref0 = mediane(&ligne(&vue, 0));
    for y in 0..20.min(h) {
        let pix = ligne(&vue, y);
        let mut ecarts: Vec<u8> = pix.iter().map(|p| ecart(*p, ref0)).collect();
        ecarts.sort_unstable();
        let n = ecarts.len();
        let centile = |p: f64| ecarts[((n as f64 * p) as usize).min(n - 1)];
        let moyenne = ecarts.iter().map(|e| u32::from(*e)).sum::<u32>() as f64 / n as f64;
        println!(
            "    y={y:3}  pire {:3}  c99 {:3}  c98 {:3}  c95 {:3}  median {:3}  moyenne {:5.1}",
            ecarts[n - 1],
            centile(0.99),
            centile(0.98),
            centile(0.95),
            centile(0.5),
            moyenne
        );
    }
}

fn ligne(vue: &Vue<'_>, y: u32) -> Vec<Pixel> {
    (0..vue.largeur()).map(|x| vue.pixel(x, y)).collect()
}

fn ecart(p: Pixel, r: Pixel) -> u8 {
    (0..3).map(|c| p[c].abs_diff(r[c])).max().unwrap_or(0)
}

fn mediane(pixels: &[Pixel]) -> Pixel {
    let mut sortie = [0u8; 4];
    for (c, v) in sortie.iter_mut().enumerate() {
        let mut canal: Vec<u8> = pixels.iter().map(|p| p[c]).collect();
        canal.sort_unstable();
        *v = canal[canal.len() / 2];
    }
    sortie
}

/// Les cas qu'on sait fabriquer, dont un **passé par un vrai encodeur JPEG**.
///
/// C'est ce qu'aucun test synthétique ne fait, et c'est précisément là que le critère se
/// juge : un bord franc entre une bande unie et une image produit du ringing sur plusieurs
/// pixels, et ce ringing est du contenu pour tout critère qui regarde le pire pixel.
fn cas_fabriques() -> Vec<(String, Image)> {
    let mut cas = Vec::new();
    let (l, h) = (400u32, 300u32);

    // Une bande noire de trente lignes en haut et en bas, un contenu bruité au milieu.
    let peindre = |x: u32, y: u32, bande: [u8; 4]| -> Pixel {
        if y < 30 || y >= h - 30 {
            bande
        } else {
            let t = ((x * 7 + y * 13) % 200) as u8;
            [60 + t / 2, 90 + t / 3, 40 + t / 4, 255]
        }
    };
    let noire: Vec<Pixel> = (0..h)
        .flat_map(|y| (0..l).map(move |x| (x, y)))
        .map(|(x, y)| peindre(x, y, [0, 0, 0, 255]))
        .collect();
    cas.push((
        "fabriquee : bande noire, sans compression".into(),
        (noire.clone(), l, h),
    ));

    // La même, encodée en JPEG puis relue : le ringing du bord franc apparaît.
    if let Some(jpeg) = par_le_jpeg(&noire, l, h, 80) {
        cas.push((
            "fabriquee : bande noire, JPEG qualite 80".into(),
            (jpeg, l, h),
        ));
    }

    // **Deux bordures l'une dans l'autre** : un liseré blanc de huit pixels, puis une bande
    // noire de vingt-cinq. C'est le cas que l'utilisateur décrit — « pas que du noir mais
    // aussi du blanc ».
    let composite: Vec<Pixel> = (0..h)
        .flat_map(|y| (0..l).map(move |x| (x, y)))
        .map(|(x, y)| {
            let bord = x.min(y).min(l - 1 - x).min(h - 1 - y);
            if bord < 8 {
                [255, 255, 255, 255]
            } else if bord < 33 {
                [0, 0, 0, 255]
            } else {
                let t = ((x * 7 + y * 13) % 200) as u8;
                [60 + t / 2, 90 + t / 3, 40 + t / 4, 255]
            }
        })
        .collect();
    cas.push((
        "fabriquee : liseré blanc PUIS bande noire".into(),
        (composite.clone(), l, h),
    ));
    if let Some(jpeg) = par_le_jpeg(&composite, l, h, 80) {
        cas.push((
            "fabriquee : blanc puis noir, JPEG qualite 80".into(),
            (jpeg, l, h),
        ));
    }
    cas
}

/// Les mêmes pixels, passés par un vrai encodeur JPEG et relus.
fn par_le_jpeg(pixels: &[Pixel], l: u32, h: u32, qualite: u8) -> Option<Vec<Pixel>> {
    let plat: Vec<u8> = pixels.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    let mut octets = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut octets, qualite);
    enc.encode(&plat, l, h, image::ExtendedColorType::Rgb8)
        .ok()?;
    let relu = image::load_from_memory(&octets).ok()?.to_rgba8();
    Some(relu.pixels().map(|p| p.0).collect())
}
