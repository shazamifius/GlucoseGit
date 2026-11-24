//! **Le flou du reste, autour de ce qu'on a choisi, sur la carte** (Q12, fiche 137) : la loi du
//! noyau (`glucose_core::flou`), transcrite par `flou_gpu/flou.wgsl`.
//!
//! # Où il se pose
//!
//! Quand il vit, la scène — tout jusqu'à la couche du dessus — se compose dans une texture à part
//! (`nette`) au lieu de l'écran ; elle s'y réduit `etages` fois, la plus petite grille se floute
//! par la gaussienne en deux passes, remonte jusqu'au premier étage, et une dernière passe mêle sur
//! l'écran la scène nette et son flou par le masque ; la chrome se pose ensuite par-dessus,
//! nette. Sans flou, rien de tout cela n'existe : la scène se compose sur l'écran comme avant.
//!
//! # Ce qui se garde d'une image à l'autre
//!
//! Les textures, tant que l'écran garde sa taille et le plan son nombre d'étages ; les réglages
//! et le masque s'écrivent à chaque image où le flou vit — quelques centaines d'octets, et la
//! grille la plus grossière (130 Ko pour un écran de 1920 × 1080).

use crate::renderer::flou::Flou;

const NUANCEUR: &str = include_str!("flou_gpu/flou.wgsl");

/// **Le format des grilles du flou** : seize bits flottants par canal — l'écart d'un niveau de
/// huit bits ne s'y perd pas d'une passe à l'autre.
pub const GRILLE_PRECISE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// **Les grilles que cette carte sait porter** : seize bits flottants si elle sait les cibler et les
/// filtrer — certains pilotes OpenGL ES ne le savent pas —, sinon huit bits par canal, que toutes
/// savent (un arrondi par passe, au plus un niveau chacune). Aucune machine exclue.
pub fn grille_pour(adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
    let f = adapter.get_texture_format_features(GRILLE_PRECISE);
    let usages = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    let filtre = wgpu::TextureFormatFeatureFlags::FILTERABLE;
    if f.allowed_usages.contains(usages) && f.flags.contains(filtre) {
        GRILLE_PRECISE
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    }
}

/// La taille d'un réglage : la source, douze poids, le reste (`flou.wgsl`).
const REGLAGE: u64 = 4 * (4 + 12 + 4);

/// Une texture et sa vue.
struct Tex {
    texture: wgpu::Texture,
    vue: wgpu::TextureView,
    taille: (u32, u32),
}

/// Une passe : sa cible, son réglage, ses liaisons, son nuanceur.
struct Passe {
    cible: usize,
    reglage: wgpu::Buffer,
    liaison: wgpu::BindGroup,
    sorte: Sorte,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Sorte {
    Reduire,
    Gauss,
    Agrandir,
}

/// Ce que le flou tient pour un écran et un plan : ses textures et ses passes.
struct Etat {
    /// La taille de l'écran, et le nombre d'étages.
    pour: ((u32, u32), u32),
    nette: Tex,
    /// Les grilles `1..=etages` — la grille entière seule quand il n'y a aucun étage —, puis le
    /// tampon de la gaussienne, à la taille de la plus grossière.
    grilles: Vec<Tex>,
    masque: Tex,
    passes: Vec<Passe>,
    /// La passe qui mêle, sur l'écran.
    meler: (wgpu::Buffer, wgpu::BindGroup),
}

/// **Le flou sur la carte** : ses nuanceurs, et ce qu'il tient pour l'image en cours.
pub struct FlouGpu {
    format: wgpu::TextureFormat,
    grille: wgpu::TextureFormat,
    disposition: wgpu::BindGroupLayout,
    echantillonneur: wgpu::Sampler,
    reduire: wgpu::RenderPipeline,
    gauss: wgpu::RenderPipeline,
    agrandir: wgpu::RenderPipeline,
    meler: wgpu::RenderPipeline,
    etat: Option<Etat>,
    /// Le flou vit-il dans cette image ?
    vit: bool,
}

impl FlouGpu {
    pub fn nouveau(
        peripherique: &wgpu::Device,
        (format, grille): (wgpu::TextureFormat, wgpu::TextureFormat),
    ) -> Self {
        let disposition = disposition(peripherique);
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("flou"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("flou"),
            bind_group_layouts: &[Some(&disposition)],
            immediate_size: 0,
        });
        let pipeline = |point: &str, cible: wgpu::TextureFormat| {
            peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("flou"),
                layout: Some(&agencement),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: cible,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let echantillonneur = peripherique.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("flou"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            format,
            grille,
            reduire: pipeline("fs_reduire", grille),
            gauss: pipeline("fs_gauss", grille),
            agrandir: pipeline("fs_agrandir", grille),
            meler: pipeline("fs_meler", format),
            disposition,
            echantillonneur,
            etat: None,
            vit: false,
        }
    }

    /// **Prépare le flou de cette image**, s'il vit : ses textures (refaites seulement quand
    /// l'écran ou le nombre d'étages change), ses réglages, son masque. À faire avant d'encoder.
    pub fn preparer(
        &mut self,
        (peripherique, file): (&wgpu::Device, &wgpu::Queue),
        ecran: (u32, u32),
        flou: Option<&Flou>,
    ) {
        self.vit = flou.is_some() && ecran.0 > 0 && ecran.1 > 0;
        let Some(flou) = flou.filter(|_| self.vit) else {
            return;
        };
        let pour = (ecran, flou.plan.etages);
        if self.etat.as_ref().is_none_or(|e| e.pour != pour) {
            self.etat = Some(self.etat_pour(peripherique, pour));
        }
        let Some(etat) = &self.etat else {
            return;
        };
        for p in &etat.passes {
            let source = source_de(etat, p);
            let pas = match p.sorte {
                Sorte::Gauss if p.cible + 1 == etat.grilles.len() => [1.0, 0.0],
                _ => [0.0, 1.0],
            };
            file.write_buffer(&p.reglage, 0, &reglage(source, pas, flou, [0.0; 3]));
        }
        let m = &flou.masque;
        let sous = if flou.plan.etages == 0 { 1.0 } else { 2.0 };
        let divers = [sous, flou.plan.maille() as f32, flou.haut];
        let premier = etat.grilles[0].taille;
        file.write_buffer(&etat.meler.0, 0, &reglage(premier, [0.0; 2], flou, divers));
        file.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &etat.masque.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &m.octets,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(m.largeur as u32),
                rows_per_image: Some(m.hauteur as u32),
            },
            wgpu::Extent3d {
                width: m.largeur as u32,
                height: m.hauteur as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    /// **La texture où la scène se compose quand le flou vit** — `None` sinon : elle se compose
    /// alors sur l'écran.
    pub fn nette(&self) -> Option<&wgpu::TextureView> {
        self.etat
            .as_ref()
            .filter(|_| self.vit)
            .map(|e| &e.nette.vue)
    }

    /// **Floute** : réduire, la gaussienne, remonter — chaque passe dans sa grille.
    pub fn flouter(&self, encodeur: &mut wgpu::CommandEncoder) {
        let Some(etat) = self.etat.as_ref().filter(|_| self.vit) else {
            return;
        };
        for p in &etat.passes {
            let mut passe = ouvrir(encodeur, &etat.grilles[p.cible].vue, "flou");
            passe.set_pipeline(match p.sorte {
                Sorte::Reduire => &self.reduire,
                Sorte::Gauss => &self.gauss,
                Sorte::Agrandir => &self.agrandir,
            });
            passe.set_bind_group(0, &p.liaison, &[]);
            passe.draw(0..3, 0..1);
        }
    }

    /// **Mêle sur l'écran** la scène nette et son flou, dans la passe en cours.
    pub fn meler(&self, passe: &mut wgpu::RenderPass<'_>) {
        let Some(etat) = self.etat.as_ref().filter(|_| self.vit) else {
            return;
        };
        passe.set_pipeline(&self.meler);
        passe.set_bind_group(0, &etat.meler.1, &[]);
        passe.draw(0..3, 0..1);
    }
}

/// La taille de la source d'une passe.
fn source_de(etat: &Etat, p: &Passe) -> (u32, u32) {
    match p.sorte {
        // Réduire vers la grille k lit la k − 1, ou l'écran pour la première.
        Sorte::Reduire if p.cible == 0 => etat.nette.taille,
        Sorte::Reduire => etat.grilles[p.cible - 1].taille,
        // La gaussienne lit une grille de la taille de la plus grossière.
        Sorte::Gauss => etat.grilles[etat.grilles.len() - 1].taille,
        // Agrandir vers la grille k lit la k + 1.
        Sorte::Agrandir => etat.grilles[p.cible + 1].taille,
    }
}

/// **Les tailles des grilles** : la moitié prise `etages` fois — la grille entière seule sans
/// étage —, puis le tampon de la gaussienne, à la taille de la plus grossière.
fn tailles_des_grilles(ecran: (u32, u32), etages: u32) -> Vec<(u32, u32)> {
    let mut tailles: Vec<(u32, u32)> = (0..etages)
        .scan(ecran, |t, _| {
            *t = (t.0.div_ceil(2), t.1.div_ceil(2));
            Some(*t)
        })
        .collect();
    if tailles.is_empty() {
        tailles.push(ecran);
    }
    tailles.push(tailles[tailles.len() - 1]);
    tailles
}

/// Les octets d'un réglage (`flou.wgsl`).
fn reglage(source: (u32, u32), pas: [f32; 2], flou: &Flou, divers: [f32; 3]) -> Vec<u8> {
    let noyau = &flou.plan.noyau;
    let mut poids = [0f32; 12];
    for (p, w) in poids.iter_mut().zip(noyau) {
        *p = *w;
    }
    let rayon = (noyau.len().min(12) - 1) as f32;
    [source.0 as f32, source.1 as f32, pas[0], pas[1]]
        .into_iter()
        .chain(poids)
        .chain([rayon, divers[0], divers[1], divers[2]])
        .flat_map(f32::to_le_bytes)
        .collect()
}

impl FlouGpu {
    /// **Les textures et les passes** d'un flou de `etages` étages sur un écran de cette taille.
    fn etat_pour(&self, peripherique: &wgpu::Device, (ecran, etages): ((u32, u32), u32)) -> Etat {
        let tex = |taille: (u32, u32), format, usage| texture(peripherique, taille, format, usage);
        let dessin = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let nette = tex(ecran, self.format, dessin);
        let tailles = tailles_des_grilles(ecran, etages);
        let grossiere = tailles[tailles.len() - 1];
        let grilles: Vec<Tex> = tailles
            .iter()
            .map(|&t| tex(t, self.grille, dessin))
            .collect();
        let copie = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING;
        let masque = tex(grossiere, wgpu::TextureFormat::R8Unorm, copie);
        let n = etages as usize;
        let tampon = grilles.len() - 1;
        let derniere = tampon - 1;
        // Réduire vers 1..=n ; la gaussienne vers le tampon puis vers la plus grossière (ou la
        // grille entière, sans étage) ; agrandir de n − 1 jusqu'à 1.
        let mut plan: Vec<(Sorte, usize, &wgpu::TextureView)> = (0..n)
            .map(|k| {
                let source = if k == 0 {
                    &nette.vue
                } else {
                    &grilles[k - 1].vue
                };
                (Sorte::Reduire, k, source)
            })
            .collect();
        let entree = if n == 0 {
            &nette.vue
        } else {
            &grilles[derniere].vue
        };
        plan.push((Sorte::Gauss, tampon, entree));
        plan.push((Sorte::Gauss, derniere, &grilles[tampon].vue));
        plan.extend(
            (0..derniere)
                .rev()
                .map(|k| (Sorte::Agrandir, k, &grilles[k + 1].vue)),
        );
        let lier = |source: &wgpu::TextureView| {
            let reglage = peripherique.create_buffer(&wgpu::BufferDescriptor {
                label: Some("flou: reglage"),
                size: REGLAGE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let liaison = self.liaison(peripherique, (&reglage, source), (&nette.vue, &masque.vue));
            (reglage, liaison)
        };
        let passes = plan
            .into_iter()
            .map(|(sorte, cible, source)| {
                let (reglage, liaison) = lier(source);
                Passe {
                    cible,
                    reglage,
                    liaison,
                    sorte,
                }
            })
            .collect();
        let meler = lier(&grilles[0].vue);
        Etat {
            pour: (ecran, etages),
            nette,
            grilles,
            masque,
            passes,
            meler,
        }
    }

    fn liaison(
        &self,
        peripherique: &wgpu::Device,
        (reglage, source): (&wgpu::Buffer, &wgpu::TextureView),
        (nette, masque): (&wgpu::TextureView, &wgpu::TextureView),
    ) -> wgpu::BindGroup {
        let vue = |binding, vue| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(vue),
        };
        peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("flou"),
            layout: &self.disposition,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: reglage.as_entire_binding(),
                },
                vue(1, source),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.echantillonneur),
                },
                vue(3, nette),
                vue(4, masque),
            ],
        })
    }
}

/// Une passe qui remplace tout ce qu'elle couvre.
fn ouvrir<'e>(
    encodeur: &'e mut wgpu::CommandEncoder,
    cible: &wgpu::TextureView,
    nom: &str,
) -> wgpu::RenderPass<'e> {
    encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(nom),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: cible,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

fn texture(
    peripherique: &wgpu::Device,
    (largeur, hauteur): (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> Tex {
    let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("flou"),
        size: wgpu::Extent3d {
            width: largeur.max(1),
            height: hauteur.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let vue = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        vue,
        taille: (largeur, hauteur),
    }
}

fn disposition(peripherique: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("flou"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            texture(3),
            texture(4),
        ],
    })
}
