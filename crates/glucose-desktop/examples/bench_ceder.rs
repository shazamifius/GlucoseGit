//! **Ce que coûte au fil qui dessine un atelier qui ne lui cède pas** (CEDER-1).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_ceder
//! ```
//!
//! # La question
//!
//! Les pires images d'un zoom portaient 21 à 34 ms sur le poste `docks`, qui en coûte un
//! d'ordinaire — là même où le magasin réveille les ouvriers de l'atelier. L'hypothèse : des
//! ouvriers de même priorité, dopés au réveil, passent devant le fil qui dessine et devant les
//! bandes que chaque image lance, pour un quantum de l'ordonnanceur.
//!
//! # Ce que le banc refait
//!
//! Autant d'ouvriers que l'atelier en lance (tous les cœurs sauf un), tenus pleins de tâches
//! de 12 à 24 ms — une rafale de décodages. Pendant ce temps, 1 500 « images » de 2 ms de
//! travail : un demi-milliseconde seul, une passe en bandes sur tous les cœurs comme la
//! composition des tuiles, vingt ordres confiés un par un comme le fait le magasin, puis une
//! milliseconde seul. Trois réglages des ouvriers : la priorité du fil qui dessine, un cran
//! dessous, et un cran dessous sans dopage — ce que fait
//! [`glucose_desktop::plateforme::priorite::ceder_au_fil_qui_dessine`].

use glucose_desktop::plateforme::priorite::ceder_au_fil_qui_dessine;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Les ordres qu'une image confie à l'atelier, et la durée de base d'une tâche.
const ORDRES: usize = 20;
const TACHE_US: u64 = 12_000;
const IMAGES: usize = 1_500;

fn tourner(d: Duration) -> u64 {
    let t = Instant::now();
    let mut x = 1u64;
    while t.elapsed() < d {
        for _ in 0..200 {
            x = x
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
        }
    }
    x
}

#[derive(Default)]
struct Atelier {
    file: Mutex<(VecDeque<Duration>, bool)>,
    reveil: Condvar,
}

#[derive(Clone, Copy)]
enum Reglage {
    CommeLeFilQuiDessine,
    UnCranDessous,
    Cede,
}

fn un_cran_dessous() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
        };
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
    }
}

fn essai(reglage: Reglage) -> Vec<f64> {
    let n = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1))
        .unwrap_or(1)
        .max(1);
    let atelier = Arc::new(Atelier::default());
    let ouvriers: Vec<_> = (0..n)
        .map(|_| {
            let a = Arc::clone(&atelier);
            std::thread::spawn(move || {
                match reglage {
                    Reglage::CommeLeFilQuiDessine => {}
                    Reglage::UnCranDessous => un_cran_dessous(),
                    Reglage::Cede => ceder_au_fil_qui_dessine(),
                }
                let mut acc = 0u64;
                loop {
                    let tache = {
                        let mut f = a.file.lock().expect("file");
                        loop {
                            if let Some(d) = f.0.pop_front() {
                                break Some(d);
                            }
                            if f.1 {
                                break None;
                            }
                            f = a.reveil.wait(f).expect("file");
                        }
                    };
                    match tache {
                        Some(d) => acc ^= tourner(d),
                        None => return acc,
                    }
                }
            })
        })
        .collect();
    let mut durees = Vec::with_capacity(IMAGES);
    for image in 0..IMAGES {
        let t = Instant::now();
        tourner(Duration::from_micros(500));
        std::thread::scope(|p| {
            for _ in 0..=n {
                p.spawn(|| tourner(Duration::from_micros(400)));
            }
        });
        {
            let mut f = atelier.file.lock().expect("file");
            if f.0.len() < 4 * n {
                for k in 0..ORDRES {
                    let variation = ((image * 7 + k * 13) as u64) % TACHE_US;
                    f.0.push_back(Duration::from_micros(TACHE_US + variation));
                }
            }
        }
        for _ in 0..ORDRES {
            atelier.reveil.notify_one();
        }
        tourner(Duration::from_millis(1));
        durees.push(t.elapsed().as_secs_f64() * 1000.0);
        std::thread::sleep(Duration::from_millis(2));
    }
    atelier.file.lock().expect("file").1 = true;
    atelier.reveil.notify_all();
    for o in ouvriers {
        let _ = o.join();
    }
    durees.sort_by(f64::total_cmp);
    durees
}

fn main() {
    let n = std::thread::available_parallelism().map_or(1, |n| n.get());
    println!(
        "{n} fils logiques, {} ouvriers pleins",
        n.saturating_sub(1).max(1)
    );
    for (nom, reglage) in [
        ("comme le fil qui dessine", Reglage::CommeLeFilQuiDessine),
        ("un cran dessous", Reglage::UnCranDessous),
        ("cede (un cran dessous, sans dopage)", Reglage::Cede),
    ] {
        let d = essai(reglage);
        let q = |p: f64| d[((d.len() - 1) as f64 * p) as usize];
        let lentes = d.iter().filter(|&&x| x > 10.0).count();
        println!(
            "{nom:<38} image de 2 ms : médiane {:>5.2}  p99 {:>5.2}  p99,9 {:>5.2}  pire {:>5.2} ms \
             | {lentes} au-dessus de 10 ms sur {}",
            q(0.5),
            q(0.99),
            q(0.999),
            d[d.len() - 1],
            d.len()
        );
    }
}
