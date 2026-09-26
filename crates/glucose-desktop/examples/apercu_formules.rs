//! **Des formules, peintes hors écran** : chaque ligne d'un fichier de LaTeX rendue par le
//! moteur des cartes, en mode bloc, blanc sur le fond du canevas — une image par formule, pour
//! les regarder, ou les confronter à celles de KaTeX lui-même rendues par un navigateur.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_formules -- <formules.txt> <dossier> [corps]
//! ```
//!
//! Le corps vaut 40 pixels par défaut : assez grand pour voir où chaque glyphe se pose.

use glucose_desktop::params::Pen;
use glucose_desktop::renderer::math::MathRenderer;
use tiny_skia::{Color, Pixmap};

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(liste), Some(dossier)) = (args.next(), args.next()) else {
        eprintln!("usage : apercu_formules <formules.txt> <dossier> [corps]");
        return;
    };
    let corps: f32 = args.next().and_then(|c| c.parse().ok()).unwrap_or(40.0);
    let Ok(texte) = std::fs::read_to_string(&liste) else {
        eprintln!("{liste} : illisible");
        return;
    };
    let math = MathRenderer::new();
    let fond = Color::from_rgba8(0x0d, 0x0d, 0x0d, 255);
    let encre = Color::from_rgba8(230, 230, 235, 255);
    for (rang, formule) in texte.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let mode = glucose_math::Mode::Display;
        let Some((largeur, haut, bas)) = math.measure(formule, mode, corps) else {
            println!("{rang:02} : ne compile pas — {formule}");
            continue;
        };
        let marge = corps;
        let (l, h) = (
            (largeur + 2.0 * marge).ceil() as u32,
            (haut + bas + 2.0 * marge).ceil() as u32,
        );
        let Some(mut image) = Pixmap::new(l.max(1), h.max(1)) else {
            continue;
        };
        image.fill(fond);
        let plume = Pen {
            x: marge,
            y: marge + haut,
            font_size: corps,
        };
        math.draw(&mut image.as_mut(), formule, mode, plume, encre);
        let chemin = format!("{dossier}/{rang:02}.png");
        match image.save_png(&chemin) {
            Ok(()) => println!("{rang:02} : {l} x {h} — {formule}"),
            Err(e) => println!("{rang:02} : {e}"),
        }
    }
}
