//! **Les membranes sur la carte** : la loi du noyau, évaluée en chaque pixel (MEMB-FORME-1).
//!
//! # Pourquoi cette passe existe
//!
//! La chronique du 25/09 : une membrane qui remplit l'écran coûtait **23 ms en médiane** au
//! processeur (trois remplissages anticrénelés de l'écran entier), et sa seule présence
//! faisait exister la couche du dessous — quinze mébioctets effacés puis envoyés à chaque
//! image. Le fond et les lueurs avaient déjà quitté cette couche pour la carte (fiche 22) ; la
//! forme des membranes les y rejoint, et la couche du dessous ne porte plus que leurs titres,
//! leurs réglettes et leurs poignées.
//!
//! # Le nuanceur est la loi, pas une approximation de la loi
//!
//! [`glucose_core::membrane_forme`] écrit la membrane comme **un seul champ d'opacité** —
//! quatre couches d'une même teinte se composent en `A = 1 − Π (1 − αₖ·couvertureₖ)` — sur
//! des rectangles aux coins circulaires dont la distance signée est exacte. Le nuanceur
//! recopie ces fonctions ligne pour ligne, en `f32` comme le noyau, dans le même ordre : ce
//! qui sépare les deux voies est l'arrondi de l'écriture, et l'épreuve le borne.
//!
//! Une membrane est un quad, celui de son enveloppe ; aucun tampon de sommets.

use glucose_core::membrane_forme::Membrane;

/// Ce qu'une membrane occupe dans le tampon : treize `vec4`.
const OCTETS_MEMBRANE: usize = 13 * 16;

/// Écrit une membrane dans l'ordre des champs du nuanceur.
fn ecrire(m: &Membrane, dans: &mut Vec<u8>) {
    let mut pousser = |v: f32| dans.extend_from_slice(&v.to_le_bytes());
    let formes = [
        m.remplissages[0].0,
        m.remplissages[1].0,
        m.remplissages[2].0,
        m.bord.forme,
    ];
    for f in &formes {
        for v in [f.gauche, f.haut, f.droite, f.bas] {
            pousser(v);
        }
    }
    for f in &formes {
        pousser(f.rayon);
    }
    for v in [
        m.remplissages[0].1,
        m.remplissages[1].1,
        m.remplissages[2].1,
        m.bord.alpha,
    ] {
        pousser(v);
    }
    let tiret = m.bord.pointille.map_or(0.0, |p| p.tiret);
    for v in [m.bord.demi_largeur, tiret, 0.0, 0.0] {
        pousser(v);
    }
    let pointille = m
        .bord
        .pointille
        .unwrap_or(glucose_core::membrane_forme::Pointille {
            tiret: 0.0,
            phases: [0.0; 8],
            restes: [0.0; 8],
        });
    for v in pointille.phases.into_iter().chain(pointille.restes) {
        pousser(v);
    }
    for v in [m.teinte[0], m.teinte[1], m.teinte[2], 0.0] {
        pousser(v);
    }
    let e = m.enveloppe();
    for v in [e.gauche, e.haut, e.droite, e.bas] {
        pousser(v);
    }
}

const NUANCEUR: &str = include_str!("membranes_gpu/membranes.wgsl");

/// La passe des membranes : un quad par membrane, un nuanceur, aucun tampon de sommets.
pub struct Membranes {
    pipeline: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    liaison: wgpu::BindGroup,
    tampon: wgpu::Buffer,
    ecran: wgpu::Buffer,
    /// Combien de membranes le tampon peut porter avant d'être refait.
    capacite: usize,
    /// Combien [`Membranes::preparer`] en a retenues pour cette image.
    posees: usize,
}

/// Combien de membranes le tampon porte au départ — il grandit si l'écran en demande plus.
const MEMBRANES_AU_DEPART: usize = 32;

impl Membranes {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = Self::disposition(peripherique);
        let pipeline = Self::pipeline(peripherique, format, &disposition);
        let tampon = Self::tampon(peripherique, MEMBRANES_AU_DEPART);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("membranes: ecran"),
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
            capacite: MEMBRANES_AU_DEPART,
            posees: 0,
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
        peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("membranes"),
            entries: &[
                entree(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                entree(
                    1,
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
            label: Some("membranes"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("membranes"),
            bind_group_layouts: &[Some(disposition)],
            immediate_size: 0,
        });
        peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("membranes"),
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
                    // Source-over en prémultiplié, dans l'ordre des instances : deux
                    // membranes imbriquées se composent comme le processeur les peint.
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
            label: Some("membranes: formes"),
            size: (combien.max(1) * OCTETS_MEMBRANE) as u64,
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
            label: Some("membranes"),
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

    /// **Écrit les membranes dans le tampon.** À appeler avant d'ouvrir la passe.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        membranes: &[Membrane],
    ) {
        self.posees = membranes.len();
        if membranes.is_empty() {
            return;
        }
        if membranes.len() > self.capacite {
            // Par doublements, comme les lueurs : il cesse de grandir en quelques images.
            self.capacite = membranes.len().next_power_of_two();
            self.tampon = Self::tampon(peripherique, self.capacite);
            self.liaison =
                Self::liaison(peripherique, &self.disposition, &self.tampon, &self.ecran);
        }
        let mut octets = Vec::with_capacity(membranes.len() * OCTETS_MEMBRANE);
        for m in membranes {
            ecrire(m, &mut octets);
        }
        file.write_buffer(&self.tampon, 0, &octets);
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }

    /// **Pose dans la passe en cours** les membranes que [`Membranes::preparer`] a reçues.
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

#[cfg(test)]
mod tests;
