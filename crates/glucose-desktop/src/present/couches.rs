//! **Composer une image en trois temps** : ce qui passe sous les photos, les photos, ce qui
//! passe dessus.
//!
//! # Pourquoi trois temps et pas deux
//!
//! Quand la carte pose les photos, le processeur produit ce qui les entoure — et pas d'un
//! seul tenant. L'ordre de la scène met le fond, les lueurs, les membranes et les dossiers
//! **sous** les photos, les annotations et les repères du geste **dessus**. Les réunir
//! mettrait une membrane par-dessus la photo qu'elle contient.
//!
//! La frontière n'est donc pas choisie : c'est celle du modèle, et [`super::super::renderer::Couche`]
//! la porte.
//!
//! # Ce que coûte la seconde couche, et pourquoi c'est acceptable
//!
//! Deux téléversements d'écran au lieu d'un. Sur 2560 × 1600 cela fait deux fois quinze
//! mébioctets, là où `blit` en coûtait un seul — environ 1,5 ms de plus dans le pire cas.
//!
//! En face, la voie graphique retire 3 à 5 ms à la scène **et cesse de pixeliser**. Le marché
//! est donc largement bon, et il le restera : la couche du dessous est presque toujours
//! statique — un fond et des lueurs qui ne bougent pas — donc elle se prête exactement au
//! suivi de salissure qui sert déjà à la bande du haut. C'est la suite, pas un préalable.

use tiny_skia::Pixmap;

/// Le nuanceur des couches : un triangle qui déborde de l'écran, et la texture telle quelle.
///
/// Trois sommets suffisent à couvrir toute la surface, et ils se calculent depuis leur propre
/// indice — ni tampon de sommets, ni maillage, ni matrice. C'est le même principe que la
/// présentation ([`super::gpu`]), dont ce module est le prolongement.
const NUANCEUR: &str = r#"
struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var filtre: sampler;

@vertex
fn vs(@builtin(vertex_index) v: u32) -> Sortie {
    let uv = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    var s: Sortie;
    s.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // L'axe vertical d'une image descend, celui du dessin monte.
    s.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return s;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    return textureSample(source, filtre, e.uv);
}
"#;

/// Une couche du processeur, portée par la carte.
struct Portee {
    texture: wgpu::Texture,
    liaison: wgpu::BindGroup,
    taille: (u32, u32),
}

/// De quoi poser les deux couches du processeur autour des photos.
pub struct Couches {
    /// Pose la couche du dessous : elle remplace, puisqu'elle porte le fond.
    remplace: wgpu::RenderPipeline,
    /// Pose la couche du dessus : elle se compose, puisqu'elle est transparente partout où
    /// les photos doivent se voir.
    compose: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    echantillonneur: wgpu::Sampler,
    dessous: Option<Portee>,
    dessus: Option<Portee>,
}

impl Couches {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("couches"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let disposition = peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("couches"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
        let remplace = Self::pipeline(peripherique, &module, &disposition, format, None);
        let compose = Self::pipeline(
            peripherique,
            &module,
            &disposition,
            format,
            Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        );
        Self {
            remplace,
            compose,
            disposition,
            echantillonneur: peripherique.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("couches: au plus proche"),
                // Une couche a EXACTEMENT la taille de la surface : un pixel de l'image est un
                // pixel de l'écran. Interpoler n'aurait rien à interpoler, mais flouterait au
                // moindre écart d'un demi-pixel — et la charte veut « net à quasi 100 % ».
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            dessous: None,
            dessus: None,
        }
    }

    fn pipeline(
        peripherique: &wgpu::Device,
        module: &wgpu::ShaderModule,
        disposition: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        melange: Option<wgpu::BlendState>,
    ) -> wgpu::RenderPipeline {
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("couches"),
            bind_group_layouts: &[Some(disposition)],
            immediate_size: 0,
        });
        peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("couches"),
            layout: Some(&agencement),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: melange,
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

    /// Téléverse une couche, en refaisant sa texture seulement si la taille a changé.
    fn accorder(
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        disposition: &wgpu::BindGroupLayout,
        filtre: &wgpu::Sampler,
        place: &mut Option<Portee>,
        source: &Pixmap,
    ) {
        let taille = (source.width(), source.height());
        let refaire = place.as_ref().is_none_or(|p| p.taille != taille);
        if refaire {
            let dim = wgpu::Extent3d {
                width: taille.0,
                height: taille.1,
                depth_or_array_layers: 1,
            };
            let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
                label: Some("couche"),
                size: dim,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let vue = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("couche"),
                layout: disposition,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&vue),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(filtre),
                    },
                ],
            });
            *place = Some(Portee {
                texture,
                liaison,
                taille,
            });
        }
        let Some(portee) = place.as_ref() else {
            return;
        };
        file.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &portee.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            source.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(taille.0 * 4),
                rows_per_image: Some(taille.1),
            },
            wgpu::Extent3d {
                width: taille.0,
                height: taille.1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Téléverse les deux couches du processeur. À faire avant d'ouvrir la passe.
    pub fn televerser(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        dessous: &Pixmap,
        dessus: &Pixmap,
    ) {
        Self::accorder(
            peripherique,
            file,
            &self.disposition,
            &self.echantillonneur,
            &mut self.dessous,
            dessous,
        );
        Self::accorder(
            peripherique,
            file,
            &self.disposition,
            &self.echantillonneur,
            &mut self.dessus,
            dessus,
        );
    }

    /// Pose la couche du dessous : elle **remplace**, puisqu'elle porte le fond.
    pub fn poser_le_dessous(&self, passe: &mut wgpu::RenderPass<'_>) {
        let Some(portee) = self.dessous.as_ref() else {
            return;
        };
        passe.set_pipeline(&self.remplace);
        passe.set_bind_group(0, &portee.liaison, &[]);
        passe.draw(0..3, 0..1);
    }

    /// Pose la couche du dessus : elle **se compose**, puisqu'elle est transparente partout où
    /// les photos doivent se voir.
    pub fn poser_le_dessus(&self, passe: &mut wgpu::RenderPass<'_>) {
        let Some(portee) = self.dessus.as_ref() else {
            return;
        };
        passe.set_pipeline(&self.compose);
        passe.set_bind_group(0, &portee.liaison, &[]);
        passe.draw(0..3, 0..1);
    }
}

/// **Encode la passe des trois temps** : le dessous, les photos, le dessus.
///
/// L'ordre est tout : le dessous REMPLACE, puisqu'il porte le fond ; le dessus se COMPOSE,
/// puisqu'il est transparent partout ou les photos doivent se voir. Les intervertir mettrait
/// le fond par-dessus tout, et une membrane par-dessus la photo qu'elle contient.
pub fn composer(
    encodeur: &mut wgpu::CommandEncoder,
    cible: &wgpu::TextureView,
    couches: &Couches,
    scene: &super::scene_gpu::SceneGpu,
    retenues: &[String],
) {
    let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("glucose-couches"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: cible,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                // Rien a effacer : la couche du dessous porte le fond et couvre tout.
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    couches.poser_le_dessous(&mut passe);
    scene.poser(&mut passe, retenues);
    couches.poser_le_dessus(&mut passe);
}
