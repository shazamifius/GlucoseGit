//! **Ce que coûte la bande du haut quand elle se refait** — la barre et les onglets.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_bande
//! ```
//!
//! # Pourquoi ce banc
//!
//! Trois sessions de terrain ont vu le poste `bande` monter à 14,95, puis 32,28, puis
//! 20,66 ms sur une seule image — celle où le document change et où la bande se refait —,
//! pour 143 kilopixels que `bench_composant` chiffre à moins d'une milliseconde de
//! remplissage. Un facteur vingt, que personne n'a expliqué : aucune hypothèse ne tient avant
//! d'avoir mesuré.
//!
//! Ce banc rend l'interface à la taille de la fenêtre de l'utilisateur — 2 160 pixels à
//! 150 % —, la bande **refaite** à chaque image contre la bande **en cache**, et il sépare le
//! tout premier dessin, où chaque glyphe se rastérise pour la première fois, des suivants.

use glucose_core::store::Store;
use glucose_desktop::params::Pointer;
use glucose_desktop::theme::Theme;
use glucose_desktop::typography::Typography;
use glucose_desktop::ui::{render_ui, UiState};
use tiny_skia::Pixmap;

/// La fenêtre de l'utilisateur, en pixels physiques, et son échelle d'interface.
const ECRAN: (u32, u32) = (2160, 1350);
const ECHELLE: f32 = 1.5;
const IMAGES: usize = 60;

fn main() {
    let store = Store::new("banc");
    let theme = Theme::dark();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let mut sortie = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");

    // Le tout premier dessin, sur une typographie neuve : chaque glyphe est une première.
    let typo = Typography::new();
    let mut ui = UiState::new();
    ui.scale_factor = ECHELLE;
    let t = std::time::Instant::now();
    render_ui(&mut sortie.as_mut(), &store, &mut ui, &typo, &theme, dehors);
    let premier = ms(t);

    let mut refaite = Vec::with_capacity(IMAGES);
    let mut en_cache = Vec::with_capacity(IMAGES);
    for _ in 0..IMAGES {
        ui.bande_cache = None;
        let t = std::time::Instant::now();
        render_ui(&mut sortie.as_mut(), &store, &mut ui, &typo, &theme, dehors);
        refaite.push(ms(t));
        let t = std::time::Instant::now();
        render_ui(&mut sortie.as_mut(), &store, &mut ui, &typo, &theme, dehors);
        en_cache.push(ms(t));
    }
    println!(
        "interface {} x {} a {:.0} %, {IMAGES} images",
        ECRAN.0,
        ECRAN.1,
        ECHELLE * 100.0
    );
    println!("  tout premier dessin (glyphes froids)  {premier:7.2} ms");
    println!(
        "  bande refaite (glyphes tiedes)        {:7.2} ms en mediane, pire {:.2}",
        mediane(&refaite),
        pire(&refaite)
    );
    println!(
        "  bande en cache                        {:7.2} ms en mediane, pire {:.2}",
        mediane(&en_cache),
        pire(&en_cache)
    );
}

fn ms(t: std::time::Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn mediane(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn pire(v: &[f64]) -> f64 {
    v.iter().copied().fold(0.0, f64::max)
}
