//! **Les flèches sur la carte** : la loi du noyau, évaluée en chaque pixel (FLECHE-2).
//!
//! # Pourquoi cette passe existe
//!
//! Peinte au processeur dans la couche du dessus, une flèche coûte deux fois : le temps de la
//! peindre, et les lignes qu'elle touche — une diagonale en touche beaucoup pour peu de
//! pixels, et chacune part sur le bus (BANDE-1). Ici, elle ne coûte que les pixels qu'elle
//! couvre, et la couche du dessus ne porte plus que ses étiquettes, badges et poignées.
//!
//! # Le nuanceur est la loi
//!
//! [`glucose_core::arrow::champ`] écrit une flèche comme un champ : la distance au tracé, la
//! place le long du dégradé, et la composition du halo, du trait et des disques. Le nuanceur
//! recopie ces fonctions ligne pour ligne, en `f32` comme le noyau, dans le même ordre ; ce
//! qui sépare les deux voies est l'arrondi de l'écriture, et l'épreuve le borne.
//!
//! # Un pixel, un seul quad
//!
//! Une flèche est un quad par segment — le segment allongé et épaissi de sa portée — et un
//! par disque. Aux coudes, deux quads se recouvrent : chacun mesure alors la distance à
//! **tous** les segments de la flèche, et seul celui du plus proche écrit. Un disque n'écrit
//! que ce qu'aucun segment ne possède. Chaque pixel reçoit donc la loi une fois, comme au
//! processeur.

use glucose_core::arrow::champ::{Champ, Disque};

/// Ce qu'une flèche occupe dans son tampon : treize `vec4`.
const OCTETS_FLECHE: usize = 13 * 16;
/// Un segment : un `vec4`.
const OCTETS_SEGMENT: usize = 16;
/// Une instance : deux `u32`.
const OCTETS_INSTANCE: usize = 8;

const NUANCEUR: &str = include_str!("fleches_gpu/fleches.wgsl");

/// Ce que [`FlechesGpu::preparer`] écrit : les flèches, leurs segments, et un quad par segment
/// et par disque.
#[derive(Default)]
struct Ecrit {
    fleches: Vec<u8>,
    segments: Vec<u8>,
    instances: Vec<u8>,
}

impl Ecrit {
    fn de(champs: &[Champ]) -> Self {
        let mut e = Self {
            fleches: Vec::with_capacity(champs.len() * OCTETS_FLECHE),
            ..Self::default()
        };
        for (rang, champ) in champs.iter().enumerate() {
            let premier = (e.segments.len() / OCTETS_SEGMENT) as u32;
            let combien = champ.segments.len() as u32;
            e.fleche(champ, (premier, combien));
            for s in &champ.segments {
                for v in s {
                    e.segments.extend_from_slice(&v.to_le_bytes());
                }
            }
            let quads = (0..combien).chain(
                champ
                    .disques
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| d.is_some())
                    .map(|(j, _)| combien + j as u32),
            );
            for morceau in quads {
                e.instances.extend_from_slice(&(rang as u32).to_le_bytes());
                e.instances.extend_from_slice(&morceau.to_le_bytes());
            }
        }
        e
    }

    /// Écrit une flèche dans l'ordre des champs du nuanceur.
    fn fleche(&mut self, c: &Champ, (premier, combien): (u32, u32)) {
        let mut f = |v: f32| self.fleches.extend_from_slice(&v.to_le_bytes());
        c.axe.into_iter().for_each(&mut f);
        for t in c.teintes {
            [t[0], t[1], t[2], 0.0].into_iter().for_each(&mut f);
        }
        [c.halo[0], c.halo[1], c.ame[0], c.ame[1]]
            .into_iter()
            .for_each(&mut f);
        let blanche = if c.ame_blanche { 1.0 } else { 0.0 };
        [blanche, c.portee(), 0.0, 0.0].into_iter().for_each(&mut f);
        for v in [premier, combien, 0, 0] {
            self.fleches.extend_from_slice(&v.to_le_bytes());
        }
        for d in c.disques {
            let present = if d.is_some() { 1.0 } else { 0.0 };
            let d = d.unwrap_or(Disque::default());
            let mut f = |v: f32| self.fleches.extend_from_slice(&v.to_le_bytes());
            [d.centre[0], d.centre[1], d.rayon, d.demi_contour]
                .into_iter()
                .for_each(&mut f);
            [d.fond[0], d.fond[1], d.fond[2], present]
                .into_iter()
                .for_each(&mut f);
            [d.contour[0], d.contour[1], d.contour[2], d.portee()]
                .into_iter()
                .for_each(&mut f);
        }
    }
}

/// Les tampons de la passe, à leur capacité, et le groupe qui les lie au nuanceur.
struct Tampons {
    fleches: wgpu::Buffer,
    segments: wgpu::Buffer,
    instances: wgpu::Buffer,
    /// Combien d'octets chacun peut porter avant d'être refait.
    capacites: [usize; 3],
    liaison: wgpu::BindGroup,
}

/// La passe des flèches : un quad par segment et par disque, un nuanceur.
pub struct FlechesGpu {
    pipeline: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    ecran: wgpu::Buffer,
    tampons: Tampons,
    /// Combien de quads [`FlechesGpu::preparer`] a retenus pour cette image.
    quads: u32,
}

/// Ce que les tampons portent au départ, en octets — ils grandissent si l'écran en demande
/// plus, par doublements.
const OCTETS_AU_DEPART: usize = 16 * 1024;

impl FlechesGpu {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = disposition(peripherique);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fleches: ecran"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let tampons = Tampons::nouveaux(peripherique, &disposition, &ecran, [OCTETS_AU_DEPART; 3]);
        Self {
            pipeline: pipeline(peripherique, format, &disposition),
            disposition,
            ecran,
            tampons,
            quads: 0,
        }
    }

    /// **Écrit les flèches dans les tampons.** À appeler avant d'ouvrir la passe.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        champs: &[Champ],
    ) {
        let ecrit = Ecrit::de(champs);
        self.quads = (ecrit.instances.len() / OCTETS_INSTANCE) as u32;
        if self.quads == 0 {
            return;
        }
        let voulu = [
            ecrit.fleches.len(),
            ecrit.segments.len(),
            ecrit.instances.len(),
        ];
        if voulu
            .iter()
            .zip(self.tampons.capacites)
            .any(|(v, c)| *v > c)
        {
            let capacites =
                [0, 1, 2].map(|i| voulu[i].next_power_of_two().max(self.tampons.capacites[i]));
            self.tampons =
                Tampons::nouveaux(peripherique, &self.disposition, &self.ecran, capacites);
        }
        file.write_buffer(&self.tampons.fleches, 0, &ecrit.fleches);
        if !ecrit.segments.is_empty() {
            file.write_buffer(&self.tampons.segments, 0, &ecrit.segments);
        }
        file.write_buffer(&self.tampons.instances, 0, &ecrit.instances);
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }

    /// **Pose dans la passe en cours** les flèches que [`FlechesGpu::preparer`] a reçues.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>) {
        if self.quads == 0 {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.tampons.liaison, &[]);
        passe.draw(0..6, 0..self.quads);
    }
}

impl Tampons {
    fn nouveaux(
        peripherique: &wgpu::Device,
        disposition: &wgpu::BindGroupLayout,
        ecran: &wgpu::Buffer,
        capacites: [usize; 3],
    ) -> Self {
        let tampon = |nom, octets: usize| {
            peripherique.create_buffer(&wgpu::BufferDescriptor {
                label: Some(nom),
                size: octets.max(16) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let fleches = tampon("fleches: fleches", capacites[0]);
        let segments = tampon("fleches: segments", capacites[1]);
        let instances = tampon("fleches: instances", capacites[2]);
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fleches"),
            layout: disposition,
            entries: &[
                entree(0, &fleches),
                entree(1, &segments),
                entree(2, &instances),
                entree(3, ecran),
            ],
        });
        Self {
            fleches,
            segments,
            instances,
            capacites,
            liaison,
        }
    }
}

/// Un tampon lié au nuanceur, à sa place.
fn entree(binding: u32, tampon: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: tampon.as_entire_binding(),
    }
}

fn disposition(peripherique: &wgpu::Device) -> wgpu::BindGroupLayout {
    let entree = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let lecture = wgpu::BufferBindingType::Storage { read_only: true };
    let tous = wgpu::ShaderStages::VERTEX_FRAGMENT;
    peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fleches"),
        entries: &[
            entree(0, tous, lecture),
            entree(1, tous, lecture),
            entree(2, wgpu::ShaderStages::VERTEX, lecture),
            entree(
                3,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
            ),
        ],
    })
}

fn pipeline(
    peripherique: &wgpu::Device,
    format: wgpu::TextureFormat,
    disposition: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fleches"),
        source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
    });
    let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fleches"),
        bind_group_layouts: &[Some(disposition)],
        immediate_size: 0,
    });
    peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fleches"),
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
                // Source-over en prémultiplié, dans l'ordre des instances : deux flèches qui se
                // croisent se composent comme le processeur les peint.
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

#[cfg(test)]
mod tests;
