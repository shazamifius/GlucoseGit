//! Le banc de la **présentation** — ce que coûte la mise à l'écran, des deux côtés (PRESENT-1).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_presentation
//! cargo run --release -p glucose-desktop --example bench_presentation -- 2560 1600
//! ```
//!
//! # Pourquoi il faut une fenêtre
//!
//! Tous les autres bancs du projet mesurent sans écran : `Renderer::render` écrit dans un
//! `PixmapMut`, qui n'a jamais eu besoin d'une fenêtre pour exister. La présentation, elle, ne
//! se mesure pas ainsi — c'est précisément le passage vers le système qu'on veut chiffrer.
//!
//! Et l'application elle-même ne peut pas servir de banc : sa boucle attend les événements,
//! donc sans main sur la souris elle ne rend rien. Dix secondes d'observation avaient donné
//! **quatre images**, toutes prises pendant le démarrage. Un banc doit décider de sa propre
//! cadence.
//!
//! # Ce qui est comparé
//!
//! La même image, de la même taille, présentée en boucle par les deux chemins :
//!
//! * **le processeur** — convertir chaque pixel en `0RGB`, puis remettre le tampon à la
//!   fenêtre. Un coût de surface, payé intégralement à chaque image ;
//! * **la carte graphique** — téléverser les octets tels quels dans une texture, puis laisser
//!   le compositeur l'afficher.
//!
//! La fenêtre est créée une fois et sert aux deux, pour que ni la taille ni l'écran ne
//! diffèrent entre les deux mesures.

use glucose_desktop::present::{CpuPresenter, GpuPresenter, Presenter};
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::{Color, Pixmap};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Combien d'images par chemin. Assez pour que la médiane veuille dire quelque chose, assez
/// peu pour que le banc se termine seul.
const IMAGES: usize = 200;

/// Les images écartées au début de chaque série : la première allume le pilote, compile les
/// nuanceurs et alloue les tampons. Les compter serait mesurer le démarrage.
const CHAUFFE: usize = 20;

struct Banc {
    taille: (u32, u32),
    window: Option<Arc<Window>>,
    image: Pixmap,
    /// Le chemin en cours de mesure, et les durées déjà relevées.
    serie: Option<(Box<dyn Presenter>, Vec<f64>)>,
    /// Ce qui reste à mesurer : le nom du chemin, et de quoi l'ouvrir.
    restant: Vec<&'static str>,
    resultats: Vec<(&'static str, Vec<f64>)>,
    /// Le chemin dont la série est en cours — deux d'entre eux portent le même `nom()`.
    dernier: &'static str,
}

impl Banc {
    fn new(taille: (u32, u32)) -> Self {
        // Une image qui ressemble à une scène : un dégradé, pour qu'aucun pilote ne puisse
        // tricher sur un aplat uniforme.
        let mut image = Pixmap::new(taille.0, taille.1).expect("une image");
        image.fill(Color::from_rgba8(13, 14, 18, 255));
        let (w, h) = (taille.0 as usize, taille.1 as usize);
        let data = image.data_mut();
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                data[i] = (x % 256) as u8;
                data[i + 1] = (y % 256) as u8;
                data[i + 2] = ((x + y) % 256) as u8;
                data[i + 3] = 255;
            }
        }
        Self {
            taille,
            window: None,
            image,
            serie: None,
            restant: vec!["carte graphique", "processeur", "carte graphique (vsync)"],
            resultats: Vec::new(),
            dernier: "",
        }
    }

    fn ouvre(&mut self, nom: &str) -> Option<Box<dyn Presenter>> {
        let window = self.window.clone()?;
        let w = NonZeroU32::new(self.taille.0)?;
        let h = NonZeroU32::new(self.taille.1)?;
        let mut p: Box<dyn Presenter> = match nom {
            // `Immediate` pour mesurer le TRAVAIL : en `Fifo`, présenter attend le balayage
            // de l'écran, et le banc chiffrerait la période de l'écran au lieu du coût.
            "carte graphique" => {
                match GpuPresenter::avec_cadence(
                    window,
                    w,
                    h,
                    Some(wgpu::PresentMode::Immediate),
                    wgpu::PowerPreference::LowPower,
                ) {
                    Ok(gpu) => {
                        println!("  adaptateur : {}", gpu.adaptateur());
                        println!("  cadence    : {:?}", gpu.cadence());
                        Box::new(gpu)
                    }
                    Err(e) => {
                        println!("  carte graphique indisponible : {e}");
                        return None;
                    }
                }
            }
            "carte graphique (vsync)" => {
                match GpuPresenter::avec_cadence(
                    window,
                    w,
                    h,
                    Some(wgpu::PresentMode::Fifo),
                    wgpu::PowerPreference::LowPower,
                ) {
                    Ok(gpu) => {
                        println!("  cadence    : {:?}", gpu.cadence());
                        Box::new(gpu)
                    }
                    Err(e) => {
                        println!("  carte graphique indisponible : {e}");
                        return None;
                    }
                }
            }
            _ => Box::new(CpuPresenter::new(window).ok()?),
        };
        p.resize(w, h).ok()?;
        Some(p)
    }
}

impl ApplicationHandler for Banc {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("Glucose — banc de présentation")
            .with_inner_size(winit::dpi::PhysicalSize::new(self.taille.0, self.taille.1));
        let window = Arc::new(event_loop.create_window(attrs).expect("une fenêtre"));
        self.window = Some(window.clone());
        event_loop.set_control_flow(ControlFlow::Poll);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if self.serie.is_none() {
                    let Some(nom) = self.restant.first().copied() else {
                        event_loop.exit();
                        return;
                    };
                    self.restant.remove(0);
                    self.dernier = nom;
                    println!("\n{nom} :");
                    match self.ouvre(nom) {
                        Some(p) => self.serie = Some((p, Vec::with_capacity(IMAGES))),
                        None => {
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }
                    }
                }

                let fini = {
                    let (presenter, temps) = self.serie.as_mut().expect("une série en cours");
                    // `frame_begin` / `frame_end` encadrent l'appel pour que les étapes
                    // internes — le téléversement d'un côté, l'attente de l'autre — soient
                    // dites séparément sous `GLUCOSE_PERF=1`. Sans cette décomposition, une
                    // attente et un travail se ressemblent à la milliseconde près.
                    glucose_desktop::perf::frame_begin();
                    let t = std::time::Instant::now();
                    let _ = presenter.present(&self.image);
                    temps.push(t.elapsed().as_secs_f64() * 1000.0);
                    glucose_desktop::perf::frame_end();
                    temps.len() >= IMAGES + CHAUFFE
                };

                if fini {
                    let (presenter, temps) = self.serie.take().expect("une série en cours");
                    let nom = self.dernier;
                    // Le presenter est lâché ici : la surface qu'il tient sur la fenêtre doit
                    // l'être avant que le suivant n'en ouvre une.
                    drop(presenter);
                    self.resultats.push((nom, temps[CHAUFFE..].to_vec()));
                }

                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn mediane(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn centile(v: &[f64], p: f64) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[((v.len() as f64 * p) as usize).min(v.len() - 1)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let taille = match args.as_slice() {
        [w, h] => (w.parse().unwrap_or(2560), h.parse().unwrap_or(1600)),
        _ => (2560, 1600),
    };
    println!(
        "Banc de présentation — {} × {} ({:.1} Mpx), {IMAGES} images par chemin",
        taille.0,
        taille.1,
        (taille.0 as f64 * taille.1 as f64) / 1e6
    );

    let mut banc = Banc::new(taille);
    let event_loop = EventLoop::new().expect("boucle d'événements");
    event_loop.run_app(&mut banc).expect("le banc");

    println!(
        "\n{:<18} {:>10} {:>10} {:>10}",
        "chemin", "médiane", "9e décile", "pire"
    );
    for (nom, temps) in &banc.resultats {
        println!(
            "{nom:<18} {:>9.2}ms {:>9.2}ms {:>9.2}ms",
            mediane(temps),
            centile(temps, 0.9),
            centile(temps, 1.0)
        );
    }
    if let [(_, a), (_, b)] = banc.resultats.as_slice() {
        let (ga, gb) = (mediane(a), mediane(b));
        println!(
            "\nLa carte graphique présente en {ga:.2} ms contre {gb:.2} ms au processeur — \
             {:.1}× moins cher, {:.2} ms rendus au budget de l'image.",
            gb / ga.max(f64::MIN_POSITIVE),
            gb - ga
        );
    }
}
