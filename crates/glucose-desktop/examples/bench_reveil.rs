//! **Ce que coûte à une carte graphique d'être réveillée** après un repos, selon sa durée.
//! Hors écran : aucune fenêtre ne s'ouvre.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_reveil
//! ```
//!
//! # Pourquoi ce banc
//!
//! Sa boîte noire du 07/10 : une image coûte 3 ms en médiane, mais la **première image après un
//! repos** gèle — 50 ms après deux secondes et plus, 300 à 500 ms après une demi-seconde à deux
//! secondes. Glucose dort au repos (`ControlFlow::Wait`), et une carte dédiée de portable
//! (Optimus) s'éteint quand plus rien ne lui est demandé. Ce banc demande à chaque carte la même
//! petite image, après des repos de durées croissantes et mêlées, et chronomètre la réponse.

use std::time::{Duration, Instant};

const ECRAN: (u32, u32) = (2160, 1350);
/// Les repos essayés, en millisecondes : mêlés, pour qu'une dérive de la machine pendant le
/// banc ne se lise pas comme un effet de la durée.
const REPOS_MS: [u64; 12] = [
    1500, 100, 3000, 500, 750, 6000, 250, 1000, 2000, 1250, 4000, 50,
];
const TOURS: usize = 3;

fn main() {
    let instance = wgpu::Instance::default();
    for preference in [
        wgpu::PowerPreference::LowPower,
        wgpu::PowerPreference::HighPerformance,
    ] {
        let Ok(adaptateur) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: preference,
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            }))
        else {
            continue;
        };
        let Ok((device, queue)) =
            pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor::default()))
        else {
            continue;
        };
        println!("\n  {}", adaptateur.get_info().name);
        let cible = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("banc"),
            size: wgpu::Extent3d {
                width: ECRAN.0,
                height: ECRAN.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let vue = cible.create_view(&wgpu::TextureViewDescriptor::default());
        // Chauffe : la première image paie la création des pipelines du pilote.
        for _ in 0..20 {
            une_image(&device, &queue, &vue);
        }
        let mut mesures: Vec<(u64, f64)> = Vec::new();
        for _ in 0..TOURS {
            for repos in REPOS_MS {
                std::thread::sleep(Duration::from_millis(repos));
                mesures.push((repos, une_image(&device, &queue, &vue)));
            }
        }
        let mut repos: Vec<u64> = REPOS_MS.to_vec();
        repos.sort_unstable();
        for r in repos {
            let mut v: Vec<f64> = mesures
                .iter()
                .filter(|(x, _)| *x == r)
                .map(|(_, t)| *t)
                .collect();
            v.sort_by(f64::total_cmp);
            println!(
                "    repos {r:>5} ms -> image en {}",
                v.iter()
                    .map(|t| format!("{t:7.1} ms"))
                    .collect::<Vec<_>>()
                    .join("  ")
            );
        }
    }
}

/// Une image : effacer la cible, soumettre, attendre que la carte ait fini. Rend des ms.
fn une_image(device: &wgpu::Device, queue: &wgpu::Queue, vue: &wgpu::TextureView) -> f64 {
    let debut = Instant::now();
    let mut encodeur = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let _passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: vue,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit([encodeur.finish()]);
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .ok();
    debut.elapsed().as_secs_f64() * 1000.0
}
