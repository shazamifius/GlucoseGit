//! **Le corps des zones sur la carte** (PRESET-1, fiche 62) : les rectangles que le socle a
//! dressés ([`crate::renderer::zone::rectangles`]), posés juste après le fond et avant la forme
//! des membranes — le premier rang du monde, comme dans Glucose Tauri.
//!
//! Le processeur remplit les mêmes rectangles au même rang : la liste est **une**, déjà sur la
//! grille de pixels, et la carte n'a rien à décider. Ce qui sépare les deux voies est
//! l'arrondi du mélange, que l'épreuve des deux voies borne.
//!
//! Un rectangle est un quad ; aucun tampon de sommets.

use crate::renderer::zone::Rectangle;

/// Ce qu'un rectangle occupe dans le tampon : deux `vec4`.
const OCTETS_RECTANGLE: usize = 2 * 16;
/// Combien de rectangles le tampon porte au départ — il grandit si l'écran en demande plus.
const RECTANGLES_AU_DEPART: usize = 256;

const NUANCEUR: &str = include_str!("zones_gpu/zones.wgsl");

/// Écrit un rectangle dans l'ordre des champs du nuanceur : sa boîte, puis sa couleur
/// prémultipliée.
fn ecrire(r: &Rectangle, dans: &mut Vec<u8>) {
    let c = r.couleur.premultiply();
    for v in [
        r.rect.left(),
        r.rect.top(),
        r.rect.right(),
        r.rect.bottom(),
        c.red(),
        c.green(),
        c.blue(),
        c.alpha(),
    ] {
        dans.extend_from_slice(&v.to_le_bytes());
    }
}

/// La passe des zones : un quad par rectangle, un nuanceur.
pub struct Zones {
    pipeline: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    liaison: wgpu::BindGroup,
    tampon: wgpu::Buffer,
    ecran: wgpu::Buffer,
    capacite: usize,
    posees: usize,
}

impl Zones {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = Self::disposition(peripherique);
        let pipeline = Self::pipeline(peripherique, format, &disposition);
        let tampon = Self::tampon(peripherique, RECTANGLES_AU_DEPART);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zones: ecran"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let liaison = Self::liaison(peripherique, &disposition, &tampon, &ecran);
        Self {
            pipeline,
            disposition,
            liaison,
            tampon,
            ecran,
            capacite: RECTANGLES_AU_DEPART,
            posees: 0,
        }
    }

    fn disposition(peripherique: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entree = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zones"),
            entries: &[
                entree(0, wgpu::BufferBindingType::Storage { read_only: true }),
                entree(1, wgpu::BufferBindingType::Uniform),
            ],
        })
    }

    fn pipeline(
        peripherique: &wgpu::Device,
        format: wgpu::TextureFormat,
        disposition: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zones"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zones"),
            bind_group_layouts: &[Some(disposition)],
            immediate_size: 0,
        });
        peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zones"),
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
                    // Source-over en prémultiplié, dans l'ordre de la liste : le fond d'une
                    // zone, puis son bord, son bandeau, son filet, comme le processeur.
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

    fn tampon(peripherique: &wgpu::Device, combien: usize) -> wgpu::Buffer {
        peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zones: rectangles"),
            size: (combien.max(1) * OCTETS_RECTANGLE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn liaison(
        peripherique: &wgpu::Device,
        disposition: &wgpu::BindGroupLayout,
        tampon: &wgpu::Buffer,
        ecran: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zones"),
            layout: disposition,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tampon.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ecran.as_entire_binding(),
                },
            ],
        })
    }

    /// **Écrit les rectangles dans le tampon.** À appeler avant d'ouvrir la passe.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        rectangles: &[Rectangle],
    ) {
        self.posees = rectangles.len();
        if rectangles.is_empty() {
            return;
        }
        if rectangles.len() > self.capacite {
            self.capacite = rectangles.len().next_power_of_two();
            self.tampon = Self::tampon(peripherique, self.capacite);
            self.liaison =
                Self::liaison(peripherique, &self.disposition, &self.tampon, &self.ecran);
        }
        let mut octets = Vec::with_capacity(rectangles.len() * OCTETS_RECTANGLE);
        for r in rectangles {
            ecrire(r, &mut octets);
        }
        file.write_buffer(&self.tampon, 0, &octets);
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }

    /// **Pose dans la passe en cours** les rectangles que [`Zones::preparer`] a reçus.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>) {
        let Ok(combien) = u32::try_from(self.posees) else {
            return;
        };
        if combien == 0 {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.liaison, &[]);
        passe.draw(0..6, 0..combien);
    }
}
