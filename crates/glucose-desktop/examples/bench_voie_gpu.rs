//! **Ce que coûterait la voie GPU, mesuré au lieu d'être supposé.**
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_voie_gpu
//! ```
//!
//! # Pourquoi ce banc existe, et pourquoi maintenant
//!
//! Glucose dessine sa scène sur le processeur. Le processeur graphique ne sert qu'à **poser
//! l'image finie** à l'écran (`present/gpu.rs`). Ce n'était pas une contrainte : c'était une
//! lecture trop étroite de la charte, qui dit « le GPU reste légitime **en plus**, jamais à
//! la place » — et non « pas de GPU ».
//!
//! L'utilisateur l'a précisé sans ambiguïté : ce qu'il refuse, c'est **basculer parce qu'on
//! échoue**. Ce qu'il veut, ce sont deux voies vraiment différentes, chacune presque parfaite
//! chez elle, et le logiciel qui se pose **là où il y a de la place** — au processeur quand
//! un Blender occupe la carte, à la carte quand un Adobe occupe le processeur.
//!
//! Avant d'écrire cette voie, il faut savoir ce qu'elle donne. Le point de comparaison est
//! `bench_salissure`, sur la même scène et le même écran : **la scène coûte aujourd'hui 4,2
//! à 6,3 ms au processeur**, seize fils compris.
//!
//! # Ce que ce banc mesure, et ce qu'il ne mesure pas
//!
//! Il mesure ce que la voie GPU a de spécifique : **poser N photos texturées, filtrées, à une
//! échelle quelconque**. C'est exactement le travail que le processeur paie le plus cher, et
//! celui que la carte fait avec du silicium dédié.
//!
//! Il ne mesure pas le texte, les formes, ni les lueurs — trois choses qu'un rastériseur GPU
//! demande d'écrire, et qui ne se déduisent pas de ce chiffre.
//!
//! Il emploie **une** texture source pour toutes les photos. En vrai il en faudrait un atlas
//! ou un tableau, ce qui change la façon de lier les ressources mais pas le nombre de pixels
//! écrits — et c'est le remplissage qui domine, comme sur le processeur.

use std::time::Instant;

/// L'écran mesuré : celui de la machine sur laquelle la chronique a été lue.
const ECRAN: (u32, u32) = (2560, 1600);
/// Le côté d'une photo source, en texels.
const PHOTO: u32 = 512;
/// Combien de photos la scène porte — le document de l'utilisateur.
const PHOTOS: u32 = 429;
/// Combien d'images le banc joue, dont les premières chauffent.
const IMAGES: usize = 40;
const CHAUFFE: usize = 10;

const NUANCEUR: &str = r#"
struct Quad { x: f32, y: f32, w: f32, h: f32 };

@group(0) @binding(0) var<storage, read> quads: array<Quad>;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var filtre: sampler;

struct Sortie {
    @builtin(position) place: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Deux triangles par photo, calcules depuis l'indice : aucun tampon de sommets.
@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    let coins = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = coins[v];
    let q = quads[i];
    // Pixels d'ecran -> coordonnees de dessin, en une ligne.
    let px = (q.x + c.x * q.w) / f32(ECRAN_L) * 2.0 - 1.0;
    let py = 1.0 - (q.y + c.y * q.h) / f32(ECRAN_H) * 2.0;
    var s: Sortie;
    s.place = vec4<f32>(px, py, 0.0, 1.0);
    s.uv = c;
    return s;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    return textureSample(source, filtre, e.uv);
}
"#;

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("des durees finies"));
    v[v.len() / 2]
}

fn main() {
    let instance = wgpu::Instance::default();
    let Some(adaptateur) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            // `bas` choisit le GPU integre : c'est le cas des tablettes et des telephones, et
            // celui qu'il faut connaitre avant de conclure quoi que ce soit sur la voie.
            power_preference: if std::env::args().any(|a| a == "bas") {
                wgpu::PowerPreference::LowPower
            } else {
                wgpu::PowerPreference::HighPerformance
            },
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .ok()
    else {
        println!("Aucun adaptateur graphique : ce banc ne peut rien dire sur cette machine.");
        return;
    };
    let info = adaptateur.get_info();
    let (peripherique, file) =
        pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("un peripherique");

    println!("Ce que la voie GPU coute, mesure -- {}\n", info.name);
    println!(
        "  Ecran {} x {}, {PHOTOS} photos de {PHOTO} texels, filtrage lineaire.\n",
        ECRAN.0, ECRAN.1
    );

    // La cible : un ecran hors fenetre, de la taille reelle.
    let cible = peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("ecran"),
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
    let vue_cible = cible.create_view(&wgpu::TextureViewDescriptor::default());

    // La photo source, televersee UNE fois -- c'est tout l'interet de la voie.
    let source = peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("photo"),
        size: wgpu::Extent3d {
            width: PHOTO,
            height: PHOTO,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let octets: Vec<u8> = (0..(PHOTO * PHOTO) as usize)
        .flat_map(|i| {
            let v = (i % 251) as u8;
            [v, v.wrapping_add(80), v.wrapping_add(160), 255]
        })
        .collect();
    file.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &source,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &octets,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(PHOTO * 4),
            rows_per_image: Some(PHOTO),
        },
        wgpu::Extent3d {
            width: PHOTO,
            height: PHOTO,
            depth_or_array_layers: 1,
        },
    );

    // Les photos du mur, en pixels d'ecran : de quoi couvrir l'ecran plusieurs fois, comme
    // un mur de photos le fait.
    let par_ligne = 22u32;
    let cote = 130.0f32;
    let quads: Vec<f32> = (0..PHOTOS)
        .flat_map(|i| {
            let (cx, cy) = (i % par_ligne, i / par_ligne);
            // Une echelle non entiere : le cas ou le processeur doit interpoler.
            [cx as f32 * cote * 0.93, cy as f32 * cote * 0.93, cote, cote]
        })
        .collect();
    let tampon = peripherique.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quads"),
        size: (quads.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Les octets se composent, ils ne se reinterpretent pas : `bytemuck` n'est pas une
    // dependance de ce projet, et une transmutation de tranche n'en vaut pas une.
    let brut: Vec<u8> = quads.iter().flat_map(|v| v.to_le_bytes()).collect();
    file.write_buffer(&tampon, 0, &brut);

    let echantillonneur = peripherique.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let source_vue = source.create_view(&wgpu::TextureViewDescriptor::default());

    let code = NUANCEUR
        .replace("ECRAN_L", &ECRAN.0.to_string())
        .replace("ECRAN_H", &ECRAN.1.to_string());
    let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("voie gpu"),
        source: wgpu::ShaderSource::Wgsl(code.into()),
    });
    let pipeline = peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("photos"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ressources"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: tampon.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&source_vue),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&echantillonneur),
            },
        ],
    });

    let mut temps = Vec::new();
    for i in 0..IMAGES {
        let depart = Instant::now();
        let mut encodeur =
            peripherique.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &vue_cible,
                    resolve_target: None,
                    depth_slice: None,
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
            passe.set_pipeline(&pipeline);
            passe.set_bind_group(0, &liaison, &[]);
            passe.draw(0..6, 0..PHOTOS);
        }
        file.submit(Some(encodeur.finish()));
        // Attendre que la carte ait FINI : sans cela on mesure le temps d'ecrire des ordres,
        // pas celui de les executer -- l'erreur que fait tout premier banc GPU.
        let _ = peripherique.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        if i >= CHAUFFE {
            temps.push(depart.elapsed().as_secs_f64() * 1000.0);
        }
    }

    let med = mediane(temps.clone());
    let pire = temps.iter().copied().fold(0.0f64, f64::max);
    println!("  {PHOTOS} photos posees et filtrees : {med:>7.3} ms mediane, {pire:>7.3} ms pire");
    println!("\n  Le point de comparaison, sur la MEME scene et le meme ecran :");
    println!("    la voie processeur, seize fils compris, coute 4,2 a 6,3 ms par image");
    println!("    (bench_salissure, pincement sur 429 photos).");
    println!("\n  Ce que ce chiffre NE dit pas : ni le texte, ni les formes, ni les lueurs.");
    println!("  Il dit ce que coute le travail que le processeur paie le plus cher.");
}
