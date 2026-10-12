//! **Ranger par la couleur un vrai document**, sans l'écrire : la couleur moyenne de chaque
//! image (le sommet de sa pyramide, MOYENNE-1), sa chroma, et des planches pour regarder ce
//! que le tri donne (ORDONNER-2).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example essai_ordonner -- copie.glucose dossier_de_sortie
//! ```
//!
//! À jouer sur une **copie** de ses documents. Le document s'ouvre en lecture seule.
//!
//! # Ce que la mesure a décidé (fiche 62)
//!
//! Deux définitions d'une image « grise » ont été posées côte à côte sur ses 38 photos :
//!
//! * **sa couleur moyenne** ne se distingue pas d'un gris — ce que Glucose fait : 20 images
//!   grises ; l'arc-en-ciel des autres est net, et les grises font un dégradé propre ;
//! * **ses pixels**, en moyenne, ne se distinguent pas du leur (la chroma moyenne des pixels,
//!   la « vivacité ») : 8 images grises seulement, les vraies photos en noir et blanc — mais
//!   une image aux teintes équilibrées, rouge et bleu, n'a pas de teinte dominante, et
//!   l'arc-en-ciel se brouille de celles-là (`par-teinte-vivacite.png`).

use glucose_core::couleur::{self, Oklch, Tri, DIFFERENCE_A_PEINE_VISIBLE};
use glucose_core::persist::histoire;
use glucose_desktop::renderer::photo::Pyramide;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// La hauteur d'une vignette sur la planche, et combien par rangée.
const VIGNETTE: u32 = 72;
const PAR_RANGEE: usize = 16;

/// Ce qu'on retient d'une image : sa vignette, sa couleur moyenne, et la même couleur dont la
/// chroma serait celle de ses pixels.
struct Mesure {
    vignette: image::RgbaImage,
    moyenne: Option<Oklch>,
    vive: Option<Oklch>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(chemin), Some(sortie)) = (args.next(), args.next()) else {
        eprintln!("usage : essai_ordonner <copie.glucose> <dossier de sortie>");
        return;
    };
    let f = std::fs::File::open(&chemin).expect("le document");
    let mut lecteur = BufReader::new(&f);
    let ouvert = histoire::ouvrir(&mut lecteur).expect("un document lisible");

    let mut mesures = Vec::new();
    for img in ouvert.projet.boards.iter().flat_map(|b| &b.images) {
        let Some(src) = img.src.as_deref() else {
            continue;
        };
        let Some(rgba) = lire(&ouvert, &mut lecteur, src) else {
            continue;
        };
        let m = mesurer(&rgba);
        if let (Some(moy), Some(vive)) = (m.moyenne, m.vive) {
            println!(
                "{:>3}  L {:.3}  chroma de la moyenne {:.4}  des pixels {:.4}  teinte {:>4.0}°  {}",
                mesures.len(),
                moy.l,
                moy.c,
                vive.c,
                moy.h.to_degrees(),
                src.rsplit(['/', '\\']).next().unwrap_or(src)
            );
        }
        mesures.push(m);
    }
    resumer(&mesures);

    std::fs::create_dir_all(&sortie).expect("le dossier de sortie");
    let moyennes: Vec<_> = mesures.iter().map(|m| m.moyenne).collect();
    let vives: Vec<_> = mesures.iter().map(|m| m.vive).collect();
    for (couleurs, tri, nom) in [
        (&moyennes, Tri::Teinte, "par-teinte.png"),
        (&moyennes, Tri::SombreVersClair, "du-sombre-au-clair.png"),
        (&vives, Tri::Teinte, "par-teinte-vivacite.png"),
    ] {
        let ordre = couleur::ordonner(couleurs, tri);
        planche(&mesures, &ordre, &Path::new(&sortie).join(nom));
    }
}

/// Les octets de cette image — scellés dans le document, ou dans son fichier — décodés.
fn lire(
    ouvert: &histoire::Ouvert,
    lecteur: &mut BufReader<&std::fs::File>,
    src: &str,
) -> Option<image::RgbaImage> {
    let octets = match ouvert.liens.get(src).and_then(|e| ouvert.objets.get(e)) {
        Some(t) => {
            let mut o = vec![0u8; t.longueur as usize];
            lecteur.seek(SeekFrom::Start(t.offset)).ok()?;
            lecteur.read_exact(&mut o).ok()?;
            o
        }
        None => std::fs::read(src).ok()?,
    };
    Some(image::load_from_memory(&octets).ok()?.to_rgba8())
}

/// La couleur moyenne par le sommet de la pyramide ; la chroma des pixels sur une réduction.
fn mesurer(rgba: &image::RgbaImage) -> Mesure {
    let pyramide =
        Pyramide::depuis_rgba(rgba.width(), rgba.height(), rgba.as_raw()).expect("la place");
    let moyenne = pyramide.couleur_moyenne().and_then(Oklch::du_pixel);
    let petite = image::imageops::thumbnail(rgba, 96, 96);
    let chromas: Vec<f64> = petite
        .pixels()
        .map(|p| {
            let [_, a, b] = couleur::oklab([0, 1, 2].map(|k| f64::from(p.0[k]) / 255.0));
            a.hypot(b)
        })
        .collect();
    let vivacite = chromas.iter().sum::<f64>() / chromas.len().max(1) as f64;
    Mesure {
        vignette: image::imageops::thumbnail(
            rgba,
            (VIGNETTE * rgba.width() / rgba.height().max(1)).max(1),
            VIGNETTE,
        ),
        moyenne,
        vive: moyenne.map(|m| Oklch { c: vivacite, ..m }),
    }
}

/// La chroma des couleurs moyennes, et combien sont grises par chaque définition.
fn resumer(mesures: &[Mesure]) {
    let mut chromas: Vec<f64> = mesures
        .iter()
        .filter_map(|m| m.moyenne)
        .map(|c| c.c)
        .collect();
    chromas.sort_by(f64::total_cmp);
    let quantile = |q: f64| chromas[((chromas.len() - 1) as f64 * q).round() as usize];
    let grises = |f: fn(&Mesure) -> Option<Oklch>| {
        mesures
            .iter()
            .filter_map(f)
            .filter(|c| !c.a_une_teinte())
            .count()
    };
    println!(
        "{} images ; chroma de la couleur moyenne : min {:.4}, médiane {:.4}, max {:.4} ; \
         grises (seuil {DIFFERENCE_A_PEINE_VISIBLE}) par la moyenne : {}, par les pixels : {}",
        mesures.len(),
        quantile(0.0),
        quantile(0.5),
        quantile(1.0),
        grises(|m| m.moyenne),
        grises(|m| m.vive),
    );
}

/// Les vignettes dans cet ordre, en rangées, sur un fond gris neutre.
fn planche(mesures: &[Mesure], ordre: &[usize], chemin: &Path) {
    let rangees: Vec<&[usize]> = ordre.chunks(PAR_RANGEE).collect();
    let largeur = rangees
        .iter()
        .map(|r| {
            r.iter()
                .map(|&k| mesures[k].vignette.width() + 4)
                .sum::<u32>()
        })
        .max()
        .unwrap_or(1);
    let hauteur = (rangees.len() as u32 * (VIGNETTE + 4)).max(1);
    let mut toile = image::RgbaImage::from_pixel(largeur, hauteur, image::Rgba([40, 40, 40, 255]));
    for (r, rangee) in rangees.iter().enumerate() {
        let mut x = 0i64;
        for &k in *rangee {
            let y = r as i64 * i64::from(VIGNETTE + 4);
            image::imageops::overlay(&mut toile, &mesures[k].vignette, x, y);
            x += i64::from(mesures[k].vignette.width() + 4);
        }
    }
    toile.save(chemin).expect("la planche");
    println!("{}", chemin.display());
}
