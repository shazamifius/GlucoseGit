//! **Ce qu'on laisse dormir : quinze cœurs sur seize, et tout le SIMD de la machine.**
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_bandes
//! ```
//!
//! # La question, et pourquoi elle décide de la pixelisation
//!
//! `bench_tuiles` dit qu'agrandir une grille de tuiles entre deux niveaux dyadiques coûte,
//! sur 2560 × 1600 :
//!
//! ```text
//!     au texel le plus proche    3,19 ms     — franchement pixelisé
//!     interpolé                 20,68 ms     — net, et six fois trop cher
//! ```
//!
//! C'est pour cela que Glucose pixelise en mouvement, et c'est ce que l'utilisateur voit :
//! *« la pixelisation est légèrement trop extrême et casse complètement cette idée de
//! smooth »*. La question n'est donc pas « faut-il pixeliser » — c'est **« ces vingt
//! millisecondes sont-elles une limite physique, ou seulement celle d'une boucle scalaire
//! sur un cœur ? »**
//!
//! La charte a déjà tranché une fois dans l'autre sens, et elle avait raison : j'avais mesuré
//! le plancher de *tiny-skia* et l'avais appelé plancher du **processeur**. Ne pas refaire
//! l'erreur : on mesure.
//!
//! # Ce que ce banc isole, et ce qu'il n'isole pas
//!
//! Il ne mesure **que** le découpage en bandes horizontales. Les bandes écrivent dans des
//! lignes disjointes, donc rien ne se partage et rien ne se synchronise — c'est le
//! parallélisme le plus simple qui existe, et le seul qui n'ait besoin d'aucune preuve
//! d'absence de course : le compilateur la fait.
//!
//! Il ne mesure **pas** le SIMD, qui est l'autre moitié du facteur et qui reste entière.
//!
//! # Pourquoi les bandes plutôt que les tuiles
//!
//! Une tuile occupe des lignes qu'une autre occupe aussi, donc deux fils qui posent deux
//! tuiles voisines écrivent dans la même ligne. Des bandes découpent la **destination**, pas
//! la source : chaque fil possède ses lignes, et une tuile à cheval est simplement posée par
//! les deux fils, chacun sur sa part, avec son propre clip. Aucune n'est dessinée deux fois
//! au même endroit.

use glucose_core::occlusion::Boite;
use glucose_core::report::{reporter, Filtre, Melange, Pose, Vue, VueMut};
use std::time::Instant;
use tiny_skia::Pixmap;

/// L'écran mesuré : celui de la machine sur laquelle la chronique a été lue.
const ECRAN: (u32, u32) = (2560, 1600);
/// Le côté d'une tuile, celui que la grille emploie.
const COTE: u32 = 256;
/// Entre deux niveaux dyadiques, le milieu du régime : le cas ordinaire d'un zoom.
const FACTEUR: f32 = 1.5;
/// Combien de mesures, dont les premières servent à chauffer.
const TOURS: usize = 15;
const CHAUFFE: usize = 5;

fn tuile(cote: u32, graine: u32) -> Pixmap {
    let mut p = Pixmap::new(cote, cote).expect("une tuile");
    let octets = p.data_mut();
    for i in 0..(cote as usize * cote as usize) {
        let (x, y) = (i as u32 % cote, i as u32 / cote);
        // Un contenu qui varie par pixel : une tuile unie se comprimerait dans le cache du
        // processeur et ferait paraitre la lecture gratuite.
        let v = ((x * 7 + y * 13 + graine * 29) % 251) as u8;
        octets[i * 4] = v;
        octets[i * 4 + 1] = v.wrapping_add(80);
        octets[i * 4 + 2] = v.wrapping_add(160);
        octets[i * 4 + 3] = 255;
    }
    p
}

/// Pose toute la grille de tuiles dans une bande de l'écran, et rend les pixels écrits.
///
/// `haut` est l'ordonnée de la bande dans l'écran entier : la pose se décale d'autant, parce
/// que la vue de la bande a sa propre origine.
fn poser_la_bande(
    pixels: &mut [[u8; 4]],
    largeur: u32,
    hauteur: u32,
    haut: u32,
    tuiles: &[Pixmap],
) -> u64 {
    let Some(mut vue) = VueMut::nouvelle(pixels, largeur, hauteur) else {
        return 0;
    };
    let clip = Boite::nouvelle(0.0, 0.0, largeur as f32, hauteur as f32);
    let pose_cote = COTE as f32 * FACTEUR;
    let (mut ecrits, mut n) = (0u64, 0usize);
    let mut y = 0.0f32;
    while y < ECRAN.1 as f32 {
        let mut x = 0.0f32;
        while x < largeur as f32 {
            let src = &tuiles[n % tuiles.len()];
            let (octets, _) = src.data().as_chunks::<4>();
            if let Some(source) = Vue::nouvelle(octets, COTE, COTE) {
                ecrits += reporter(
                    &mut vue,
                    &source,
                    Pose {
                        x: x.floor(),
                        y: y.floor() - haut as f32,
                        largeur: pose_cote,
                        hauteur: pose_cote,
                    },
                    clip,
                    Melange::Remplacer,
                    Filtre::Lisse,
                );
            }
            n += 1;
            x += pose_cote;
        }
        y += pose_cote;
    }
    ecrits
}

/// Une image entière, composée par `fils` bandes horizontales. Rend les millisecondes.
fn une_image(ecran: &mut Pixmap, tuiles: &[Pixmap], fils: usize) -> f64 {
    let (largeur, hauteur) = (ECRAN.0, ECRAN.1);
    let debut = Instant::now();
    let (octets, _) = ecran.data_mut().as_chunks_mut::<4>();

    // La dernière bande prend le reste : sans cela, une hauteur qui ne divise pas le nombre
    // de fils laisserait des lignes non peintes, et le banc mesurerait moins de travail.
    let par_bande = (hauteur as usize).div_ceil(fils);
    let bandes: Vec<&mut [[u8; 4]]> = octets.chunks_mut(par_bande * largeur as usize).collect();

    std::thread::scope(|portee| {
        let mut haut = 0u32;
        for bande in bandes {
            let h = (bande.len() / largeur as usize) as u32;
            portee.spawn(move || poser_la_bande(bande, largeur, h, haut, tuiles));
            haut += h;
        }
    });
    debut.elapsed().as_secs_f64() * 1000.0
}

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("des durees finies"));
    v[v.len() / 2]
}

fn mesurer(ecran: &mut Pixmap, tuiles: &[Pixmap], fils: usize) -> f64 {
    let mut temps = Vec::new();
    for i in 0..TOURS {
        let ms = une_image(ecran, tuiles, fils);
        if i >= CHAUFFE {
            temps.push(ms);
        }
    }
    mediane(temps)
}

fn main() {
    let coeurs = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    println!("Composer un ecran INTERPOLE, en bandes paralleles\n");
    println!(
        "  Ecran {} x {}, tuiles de {COTE}px, agrandissement x{FACTEUR}, filtre interpole.",
        ECRAN.0, ECRAN.1
    );
    println!("  La machine annonce {coeurs} fils d'execution.\n");

    let mut ecran = Pixmap::new(ECRAN.0, ECRAN.1).expect("un ecran");
    let tuiles: Vec<Pixmap> = (0..16).map(|g| tuile(COTE, g)).collect();

    // **Les regimes s'alternent dans la MEME execution.** Trois lancements separes du meme
    // banc avaient donne 2,3, 6,2 et 5,7 ms pour la meme mesure : la frequence de la machine
    // varie plus que ce qu'on mesure, et comparer deux executions, c'est comparer le bruit.
    let mut essais: Vec<usize> = Vec::new();
    let mut f = 1;
    while f <= coeurs.max(1) {
        essais.push(f);
        f *= 2;
    }
    if *essais.last().unwrap_or(&1) != coeurs {
        essais.push(coeurs);
    }

    let seul = mesurer(&mut ecran, &tuiles, 1);
    println!(
        "  {:>5}  {:>10}  {:>8}  {:>10}",
        "fils", "duree", "facteur", "par fil"
    );
    for fils in &essais {
        let ms = mesurer(&mut ecran, &tuiles, *fils);
        println!(
            "  {:>5}  {:>8.2}ms  {:>7.2}x  {:>8.2}ms",
            fils,
            ms,
            seul / ms,
            ms * *fils as f64
        );
    }

    println!("\n  Ce que ces colonnes veulent dire\n");
    println!("  « facteur » : ce que le decoupage rapporte, a travail identique.");
    println!("  « par fil » : la duree multipliee par le nombre de fils. Si elle reste");
    println!("                constante, le partage est parfait ; si elle monte, quelque");
    println!("                chose se partage qui ne devrait pas -- la memoire, le plus");
    println!("                souvent, parce que la bande passante, elle, ne se divise pas.");
    println!();
    println!("  Le repere : 3,19 ms, ce que le meme ecran coute AU TEXEL LE PLUS PROCHE sur");
    println!("  un seul fil. Sous cette barre, interpoler cesse de couter plus cher que");
    println!("  pixeliser, et la pixelisation n'a plus de raison d'exister.");
}
