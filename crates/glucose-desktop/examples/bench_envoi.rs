//! **Ce que coûte l'envoi d'une couche ou d'une texture à la carte**, et par quel chemin
//! (ENVOI-1, fiche 43). Hors écran : aucune fenêtre ne s'ouvre.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_envoi
//! ```
//!
//! # Ce que la mesure a trouvé
//!
//! La couche du dessus part par lignes entières (BANDE-1) : un panneau de toute la hauteur — la
//! Time Machine — en fait partir toutes les lignes, 11 Mo à chaque image. Sur le terrain,
//! `blit` et `soumettre` y coûtaient 10 et 9 ms. Avec le réglage de mémoire par défaut, le même
//! envoi coûte 1,2 ms ; **avec celui de l'application** (`MemoryUsage`, ETAGES-1), les blocs de
//! mémoire visible du processeur font 4 Mo, et tout envoi plus gros reçoit une allocation
//! dédiée, créée puis détruite à chaque fois.
//!
//! Ce banc compare, sur chaque carte et sous chaque réglage, `Queue::write_texture` et un
//! tampon d'envoi qui **persiste** d'une image à l'autre (`StagingBelt`, le chemin de
//! [`glucose_desktop::present::envoi`]), pour : quatre bandes de la couche (la chrome sans
//! panneau), la couche entière, un niveau de photo neuf, le rectangle d'un panneau.

use std::time::Instant;

const ECRAN: (u32, u32) = (2160, 1350);
/// Un niveau de photo comme ceux que le zoom demande : 2,3 Mpx.
const PHOTO: (u32, u32) = (1920, 1200);
const IMAGES: usize = 120;
const CHAUFFE: usize = 20;

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
        for (nom, conseil) in [
            ("Performance", wgpu::MemoryHints::Performance),
            (
                "MemoryUsage (l'application)",
                wgpu::MemoryHints::MemoryUsage,
            ),
        ] {
            let Ok((device, queue)) =
                pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor {
                    required_limits: adaptateur.limits(),
                    memory_hints: conseil,
                    ..Default::default()
                }))
            else {
                continue;
            };
            println!("\n  {} -- {nom}", adaptateur.get_info().name);
            println!(
                "    {:<36} {:>14} {:>18}",
                "", "write_texture", "tampon persistant"
            );
            mesurer(&device, &queue);
        }
    }
}

fn texture(device: &wgpu::Device, (w, h): (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("banc"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// Un envoi : des rectangles `(x, y, largeur, hauteur)` de la couche, vers une texture neuve
/// ou vers la couche qui garde son contenu.
struct Cas {
    nom: &'static str,
    zones: Vec<(u32, u32, u32, u32)>,
    neuve: bool,
}

fn mesurer(device: &wgpu::Device, queue: &wgpu::Queue) {
    let (w, h) = ECRAN;
    let couche = texture(device, ECRAN);
    let octets: Vec<u8> = (0..w * h * 4).map(|i| (i % 251) as u8).collect();
    let cas = [
        Cas {
            nom: "quatre bandes, 784 lignes",
            zones: [(0, 80), (200, 300), (700, 300), (1246, 104)]
                .map(|(y, bh)| (0, y, w, bh))
                .to_vec(),
            neuve: false,
        },
        Cas {
            nom: "la couche entière, 11 Mo",
            zones: vec![(0, 0, w, h)],
            neuve: false,
        },
        Cas {
            nom: "un niveau de photo neuf, 9 Mo",
            zones: vec![(0, 0, PHOTO.0, PHOTO.1)],
            neuve: true,
        },
        Cas {
            nom: "le rectangle d'un panneau, 2,3 Mo",
            zones: vec![(w - 510, 90, 486, h - 110)],
            neuve: false,
        },
    ];
    for c in &cas {
        let direct = chronometrer(device, |_| {
            let cible = cible_du_cas(device, c, &couche);
            for &(x, y, zw, zh) in &c.zones {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &cible,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &octets[((y * w + x) * 4) as usize..],
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(w * 4),
                        rows_per_image: Some(zh),
                    },
                    wgpu::Extent3d {
                        width: zw,
                        height: zh,
                        depth_or_array_layers: 1,
                    },
                );
            }
            queue.submit(None);
        });
        let mut envoi = glucose_desktop::present::envoi::Envoi::nouveau(device, queue);
        let persistant = chronometrer(device, |_| {
            let cible = cible_du_cas(device, c, &couche);
            for &(x, y, zw, zh) in &c.zones {
                let depart = ((y * w + x) * 4) as usize;
                envoi.texture(
                    &cible,
                    (x, y),
                    (zw, zh),
                    (&octets[depart..], w as usize * 4),
                );
            }
            envoi.soumettre();
        });
        println!("    {:<36} {direct:>11.2} ms {persistant:>15.2} ms", c.nom);
    }
}

fn cible_du_cas(device: &wgpu::Device, c: &Cas, couche: &wgpu::Texture) -> wgpu::Texture {
    if c.neuve {
        let (_, _, zw, zh) = c.zones[0];
        texture(device, (zw, zh))
    } else {
        couche.clone()
    }
}

/// La médiane, en millisecondes, de ce que coûte `image` — préparer et soumettre —, la carte
/// laissée travailler pendant qu'on prépare la suivante, comme l'application.
fn chronometrer(device: &wgpu::Device, mut image: impl FnMut(usize)) -> f64 {
    let mut durees: Vec<f64> = (0..IMAGES + CHAUFFE)
        .map(|k| {
            let debut = Instant::now();
            image(k);
            let duree = debut.elapsed().as_secs_f64() * 1e3;
            device.poll(wgpu::PollType::Poll).ok();
            duree
        })
        .skip(CHAUFFE)
        .collect();
    durees.sort_by(f64::total_cmp);
    durees[durees.len() / 2]
}
