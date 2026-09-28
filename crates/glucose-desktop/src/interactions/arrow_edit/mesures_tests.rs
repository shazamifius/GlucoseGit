//! **Ce que coûte le contournement**, par le vrai index de l'application : des scènes de
//! tableaux réels — une grille de cartes, un mur de photos, une mosaïque de murs, une colonne de
//! cartes — chacune cherchée plusieurs fois, la médiane et le minimum rapportés.
//!
//! `cargo test --release -p glucose-desktop --lib mesure_des_scenes -- --ignored --nocapture`

use super::contour_tests::{application, carte, chemin};
use crate::app::GlucoseApp;
use glucose_core::types::BoardImage;

/// Une scène : les deux cartes « s » et « t » reliées par « f », et ce qui se pose entre elles.
fn scene(poser: impl FnOnce(&mut glucose_core::types::Board)) -> GlucoseApp {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.remove_annotations(&board, &["o"]);
    if let Some(b) = app.store.active_board_mut() {
        poser(b);
    }
    app.store.bump_version();
    app.renderer.sync_spatial_index(&app.store);
    app
}

/// Replace la carte `id` à cette place.
fn placer(b: &mut glucose_core::types::Board, id: &str, (x, y): (f64, f64)) {
    b.annotations.retain(|a| a.id() != id);
    b.annotations.push(carte(id, (x, y), (100.0, 60.0)));
}

/// **Une mosaïque de murs** : `n × n` murs de `c × c` photos serrées, séparés par des couloirs
/// où une flèche passe ; « s » à gauche, « t » à droite, la droite qui les relie en travers.
fn mosaique(n: usize, c: usize) -> GlucoseApp {
    scene(|b| {
        let (photo, pas, couloir) = (100.0, 105.0, 150.0);
        let mur = c as f64 * pas + couloir;
        for (mi, mj) in (0..n * n).map(|k| (k % n, k / n)) {
            for (i, j) in (0..c * c).map(|k| (k % c, k / c)) {
                let x = 300.0 + mi as f64 * mur + i as f64 * pas;
                let y = -(n as f64) * mur / 2.0 + 60.0 + mj as f64 * mur + j as f64 * pas;
                let id = format!("m{mi}-{mj}-{i}-{j}");
                b.images.push(BoardImage::new(id, x, y, photo, photo));
            }
        }
        placer(b, "t", (400.0 + n as f64 * mur, 0.0));
    })
}

/// **Une colonne de cartes séparées** : `n` cartes de 100 × 60 espacées de 40 ; « s » en haut,
/// « t » en bas, la flèche le long de la colonne.
fn colonne(n: usize) -> GlucoseApp {
    scene(|b| {
        for k in 1..n - 1 {
            b.annotations.push(carte(
                &format!("c{k}"),
                (0.0, k as f64 * 100.0),
                (100.0, 60.0),
            ));
        }
        placer(b, "t", (0.0, (n - 1) as f64 * 100.0));
    })
}

/// **Un mur** de `c × c` photos de 200 entre les deux cartes.
fn mur(c: usize) -> GlucoseApp {
    scene(|b| {
        for (i, j) in (0..c * c).map(|k| (k % c, k / c)) {
            let (x, y) = (
                300.0 + i as f64 * 205.0,
                30.0 - c as f64 * 102.5 + j as f64 * 205.0,
            );
            b.images
                .push(BoardImage::new(format!("p{i}-{j}"), x, y, 200.0, 200.0));
        }
        placer(b, "t", (320.0 + c as f64 * 205.0, 0.0));
    })
}

/// Cherche l'itinéraire de « f » `fois` fois, la mémoire oubliée chaque fois : la médiane et le
/// minimum en microsecondes, et le nombre d'étapes.
fn chronometrer(app: &GlucoseApp, fois: usize) -> (f64, f64, usize) {
    let mut durees: Vec<f64> = (0..fois)
        .map(|_| {
            app.renderer.itineraires.oublier();
            let debut = std::time::Instant::now();
            std::hint::black_box(chemin(app));
            debut.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    durees.sort_by(f64::total_cmp);
    (durees[fois / 2], durees[0], chemin(app).len() - 2)
}

#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine"]
fn mesure_des_scenes() {
    let scenes: [(&str, GlucoseApp); 6] = [
        ("mur 20×20", mur(20)),
        ("mur 100×100", mur(100)),
        ("mosaïque 3×3 murs de 6×6", mosaique(3, 6)),
        ("mosaïque 5×5 murs de 6×6", mosaique(5, 6)),
        ("colonne de 50 cartes", colonne(50)),
        ("colonne de 200 cartes", colonne(200)),
    ];
    for (nom, app) in &scenes {
        let (mediane, minimum, etapes) = chronometrer(app, 21);
        eprintln!(
            "MESURE {nom:28} médiane {mediane:9.1} µs  min {minimum:9.1} µs  {etapes} étapes"
        );
    }
}
