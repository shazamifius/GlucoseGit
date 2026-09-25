//! **Le liseré du passé sur la carte** : quand la Time Machine montre un état passé, le bord
//! ambré de la fenêtre et son voile se peignent ici, et non plus au processeur.
//!
//! # Ce que la mesure a montré
//!
//! Peint au processeur dans la couche du dessus, le liseré coûtait **11,7 ms par image** au
//! poste `docks` (0,96 sans lui), et ses bandes gauche et droite touchaient toutes les lignes :
//! la couche du dessus repartait entière — 1 350 lignes sur 1 350 — à chaque image, tant qu'on
//! regardait le passé. C'est exactement quand on glisse la réglette.
//!
//! # La loi
//!
//! Quatre dégradés d'une même teinte, chacun de l'opacité du voile au bord jusqu'à zéro à
//! `voile` points du bord ; une même teinte composée sur elle-même n'est qu'une opacité,
//! `A = 1 − Π (1 − aᵢ)`. Par-dessus, un trait ambré de `trait_` points sur le pourtour : la
//! réunion de quatre bandes alignées sur les axes, dont un pixel est couvert de
//! `1 − Π (1 − cₖ)` — la même forme, exacte aux coins intérieurs.

/// Le liseré tel que la carte le peint, en pixels d'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lisere {
    /// La profondeur du voile depuis chaque bord.
    pub voile: f32,
    /// L'épaisseur du trait.
    pub trait_: f32,
    /// Le voile au bord : sa teinte et son opacité, de 0 à 1.
    pub teinte_du_voile: [f32; 4],
    /// Le trait : sa teinte et son opacité.
    pub ambre: [f32; 4],
}

const NUANCEUR: &str = r#"
struct Reglage {
    // largeur, hauteur de l'ecran ; profondeur du voile ; epaisseur du trait.
    ecran: vec4<f32>,
    voile: vec4<f32>,
    ambre: vec4<f32>,
};

@group(0) @binding(0) var<uniform> r: Reglage;

@vertex
fn vs(@builtin(vertex_index) v: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

// L'opacite d'un degrade a `d` pixels de son bord : lineaire, nulle au-dela du voile.
fn degrade(d: f32) -> f32 {
    return r.voile.a * clamp(1.0 - d / r.ecran.z, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let w = r.ecran.x;
    let h = r.ecran.y;
    let bords = vec4<f32>(p.y, h - p.y, p.x, w - p.x);
    var transparence = 1.0;
    var hors_du_trait = 1.0;
    for (var k = 0; k < 4; k = k + 1) {
        let d = bords[k];
        transparence = transparence * (1.0 - degrade(d));
        // Le trait est la reunion de quatre bandes alignees sur les axes : le pixel en est
        // couvert de 1 - produit(1 - c), chaque c etant le recouvrement de [d - 1/2, d + 1/2]
        // et de [0, trait]. Exact, coins interieurs compris.
        let c = max(min(d + 0.5, r.ecran.w) - max(d - 0.5, 0.0), 0.0);
        hors_du_trait = hors_du_trait * (1.0 - c);
    }
    let a_voile = 1.0 - transparence;
    let a_trait = r.ambre.a * (1.0 - hors_du_trait);
    let voile = vec4<f32>(r.voile.rgb * a_voile, a_voile);
    return vec4<f32>(r.ambre.rgb * a_trait, a_trait) + voile * (1.0 - a_trait);
}
"#;

/// La passe du liseré : un triangle qui couvre l'écran, et la distance au bord.
pub struct LisereGpu {
    pipeline: wgpu::RenderPipeline,
    liaison: wgpu::BindGroup,
    reglage: wgpu::Buffer,
    /// Le liseré à poser à cette image, s'il y en a un.
    pose: bool,
}

impl LisereGpu {
    pub fn nouveau(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lisere"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let reglage = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lisere: reglage"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lisere"),
            layout: &disposition,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: reglage.as_entire_binding(),
            }],
        });
        Self {
            pipeline: pipeline(peripherique, format, &disposition),
            liaison,
            reglage,
            pose: false,
        }
    }

    /// **Écrit le réglage de cette image.** À appeler avant d'ouvrir la passe.
    pub fn preparer(&mut self, file: &wgpu::Queue, ecran: (f32, f32), lisere: Option<Lisere>) {
        self.pose = lisere.is_some();
        let Some(l) = lisere else {
            return;
        };
        let mut octets = Vec::with_capacity(48);
        let valeurs = [ecran.0, ecran.1, l.voile, l.trait_]
            .into_iter()
            .chain(l.teinte_du_voile)
            .chain(l.ambre);
        for v in valeurs {
            octets.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.reglage, 0, &octets);
    }

    /// **Pose le liseré** dans la passe en cours, par-dessus tout — s'il y en a un.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>) {
        if !self.pose {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.liaison, &[]);
        passe.draw(0..3, 0..1);
    }
}

fn pipeline(
    peripherique: &wgpu::Device,
    format: wgpu::TextureFormat,
    disposition: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lisere"),
        source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
    });
    let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("lisere"),
        bind_group_layouts: &[Some(disposition)],
        immediate_size: 0,
    });
    peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("lisere"),
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
