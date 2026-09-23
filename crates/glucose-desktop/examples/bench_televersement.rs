//! **Ce que coûte le téléversement d'une photo sur la carte, selon sa taille** (fiche 29 § 4.3).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_televersement
//! ```
//!
//! # La question
//!
//! La voie graphique téléversait toujours la texture **native** d'une photo, quelle que soit
//! sa taille à l'écran : une épingle rapatriée en original, 2 297 × 3 062, pèse 28 Mo, envoyés
//! même pour une vignette de deux cents pixels. Le téléversement est budgété (CASCADE-2) : ce
//! qui ne tient pas attend l'image suivante — d'où les photos qui arrivent « en vagues ».
//!
//! Avant de choisir le niveau à envoyer, il faut savoir ce qu'un envoi coûte vraiment, et où :
//! la copie de la source (`Pixmap::clone`, ce que faisait l'application), le travail du
//! processeur pour remettre les octets à la carte (`write_texture`), et le temps jusqu'à ce que
//! la carte les ait. Sur les deux cartes de la machine : l'intégrée partage la mémoire du
//! processeur, la dédiée la reçoit par le bus.

use std::time::{Duration, Instant};

/// Les tailles mesurées : les niveaux dyadiques d'une épingle rapatriée, et l'original.
const TAILLES: [(u32, u32); 6] = [
    (2297, 3062),
    (1149, 1531),
    (575, 766),
    (288, 383),
    (144, 192),
    (72, 96),
];
const TOURS: usize = 24;

fn main() {
    let instance = wgpu::Instance::default();
    for (nom, preference) in [
        ("carte integree (LowPower)", wgpu::PowerPreference::LowPower),
        (
            "carte rapide (HighPerformance)",
            wgpu::PowerPreference::HighPerformance,
        ),
    ] {
        let Ok(adaptateur) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: preference,
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            }))
        else {
            println!("{nom} : aucune");
            continue;
        };
        let info = adaptateur.get_info();
        let Ok((peripherique, file)) =
            pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor::default()))
        else {
            println!("{nom} : peripherique refuse");
            continue;
        };
        println!("\n{nom} -- {} ({:?})", info.name, info.backend);
        println!(
            "  {:>11} {:>7}   {:>9} {:>13} {:>12}",
            "taille", "Mo", "copie", "write_texture", "jusqu'a fin"
        );
        for (l, h) in TAILLES {
            mesurer(&peripherique, &file, (l, h));
        }
    }
}

/// Une taille : la copie de la source, le travail du processeur, et le temps jusqu'à ce que la
/// carte ait fini — en médiane sur `TOURS` envois.
fn mesurer(peripherique: &wgpu::Device, file: &wgpu::Queue, (l, h): (u32, u32)) {
    let mut source = tiny_skia::Pixmap::new(l, h).expect("une source");
    for (i, o) in source.data_mut().iter_mut().enumerate() {
        *o = (i % 251) as u8;
    }
    let (mut copies, mut ecritures, mut totaux) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..TOURS {
        let debut = Instant::now();
        let copie = source.clone();
        let copiee = debut.elapsed();

        let debut = Instant::now();
        let taille = wgpu::Extent3d {
            width: l,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
            label: Some("banc"),
            size: taille,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        file.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            copie.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(l * 4),
                rows_per_image: Some(h),
            },
            taille,
        );
        let ecrite = debut.elapsed();
        file.submit(None);
        let _ = peripherique.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        let fini = debut.elapsed();
        copies.push(copiee);
        ecritures.push(ecrite);
        totaux.push(fini);
    }
    let mo = f64::from(l) * f64::from(h) * 4.0 / 1_048_576.0;
    println!(
        "  {:>5}x{:<5} {:>7.2}   {:>7.3}ms {:>11.3}ms {:>10.3}ms",
        l,
        h,
        mo,
        ms(mediane(&mut copies)),
        ms(mediane(&mut ecritures)),
        ms(mediane(&mut totaux))
    );
}

fn mediane(v: &mut [Duration]) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
