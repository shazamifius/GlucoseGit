//! **Ce que la carte a besoin d'avoir une fois pour toutes** : le nuanceur, le pipeline, les
//! dispositions et le tampon des poses.
//!
//! Extrait de [`super`], à 598 lignes quand ATLAS-1 y est entré : ici ce qui ne change jamais,
//! là-bas ce que la carte détient et ce qu'elle pose à chaque image.

use super::pose::OCTETS_POSE;
use super::SceneGpu;

/// Le nuanceur : deux triangles par photo, calculés depuis leur indice.
///
/// Aucun tampon de sommets, aucun maillage : le rectangle se déduit de `vertex_index`, et sa
/// place d'un tampon que l'**indice d'instance** adresse.
///
/// # Pourquoi un tampon et pas des constantes poussées
///
/// Les constantes poussées seraient plus courtes — trente-deux octets par photo, sans
/// allocation. Mais elles demandent une capacité que toutes les couches graphiques n'offrent
/// pas, WebGL en tête. **L'exiger exclurait du matériel**, ce que la charte interdit sans
/// exception, et la demander sans l'exiger obligerait à porter deux nuanceurs pour gagner
/// quelques microsecondes sur un chemin qui n'en coûte déjà qu'une fraction.
///
/// Un tampon de poses, lu par `instance_index`, marche partout et se remplit d'un coup.
const NUANCEUR: &str = include_str!("photos.wgsl");

impl SceneGpu {
    /// Les deux groupes de ressources : ce qui vaut pour toute l'image, et ce qui change par
    /// photo.
    ///
    /// Les séparer n'est pas cosmétique : une liaison qui change à chaque dessin coûte, et
    /// celle-là ne porte plus qu'une texture.
    pub(super) fn dispositions(
        peripherique: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
        let commun = peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene: commun"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let photo = peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene: une photo"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        (commun, photo)
    }

    pub(super) fn pipeline_des_photos(
        peripherique: &wgpu::Device,
        format: wgpu::TextureFormat,
        commun: &wgpu::BindGroupLayout,
        photo: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene: photos"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene"),
            bind_group_layouts: &[Some(commun), Some(photo)],
            immediate_size: 0,
        });
        peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene: photos"),
            layout: Some(&agencement),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Source-over en prémultiplié : la même loi que `Melange::Composer` du
                    // noyau, pour que les deux voies composent de la même façon.
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
    }

    pub(super) fn tampon_des_poses(peripherique: &wgpu::Device, combien: usize) -> wgpu::Buffer {
        peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene: poses"),
            size: (combien * OCTETS_POSE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(super) fn liaison_commune(
        peripherique: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        poses: &wgpu::Buffer,
        ecran: &wgpu::Buffer,
        filtre: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene: commun"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: poses.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ecran.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(filtre),
                },
            ],
        })
    }
}
