//! **Le fond sur la carte** : une couleur unie et des points, sans qu'un seul pixel ne passe
//! par le processeur.
//!
//! # Pourquoi cette passe existe, alors qu'elle ne coûte « que » 1,2 ms
//!
//! Pas pour ces 1,2 ms. Pour ce qu'elle **libère** : tant que le fond se peint sur le
//! processeur, la couche du dessous est **opaque**, donc elle existe toujours, donc elle se
//! téléverse toujours — quinze mébioctets par image sur 2560 × 1600.
//!
//! Une fois le fond et les lueurs sur la carte, la couche du dessous ne porte plus que les
//! membranes et les dossiers. Sur un document qui n'en a pas — le cas courant — elle est
//! **entièrement transparente**, et une couche transparente ne se téléverse pas du tout.
//!
//! C'est la différence entre « ne pas re-téléverser ce qui n'a pas changé », qui ne gagne
//! rien pendant un geste puisque tout change, et **faire disparaître le téléversement**.
//! La charte le dit : une constante arbitraire qui peut disparaître doit disparaître, et
//! cela vaut aussi d'un coût.
//!
//! # FOND-GPU-1 — un pixel ne peut toucher qu'un seul point
//!
//! La grille adapte son pas pour qu'il ne descende jamais sous trente-deux pixels d'écran
//! (`GRID_MIN_SCREEN_STEP`), et le rayon d'un point plafonne à deux pixels et demi. Un pixel
//! ne peut donc être touché que par **un** point : celui de sa maille.
//!
//! Le nuanceur n'a alors rien à parcourir. Il calcule le point le plus proche par un arrondi,
//! mesure la distance à son centre, et en déduit la couverture — exactement la formule
//! analytique du processeur, `clamp(r + ½ − d, 0, 1)`, mais **sans la quantification en seize
//! phases** que celui-ci doit s'imposer pour pré-calculer ses masques.
//!
//! L'écart entre les deux voies est donc celui de cette quantification : un huitième de pixel
//! de position, soit au plus un huitième de niveau de couverture. Un test le borne.
//!
//! # Ce qui ne descend pas ici
//!
//! Le **pas** de la grille, son rayon, son opacité et son extinction sont décidés par le
//! socle ([`crate::renderer::scene::grid`]), qui les tient de la fiche 06. La carte ne
//! choisit rien : elle reçoit quatre nombres et remplit.

/// Le fond d'une image : sa couleur, la vue, et ce qu'il faut savoir des points.
///
/// Toutes les longueurs sont en **pixels d'écran** ou en unités monde selon ce que le socle
/// en fait : `pas` est en unités monde, comme la grille le définit ; le reste est à l'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fond {
    /// La couleur du canevas, de 0 à 1 par canal.
    pub rouge: f32,
    pub vert: f32,
    pub bleu: f32,
    /// L'échelle de la vue, et sa translation en pixels d'écran.
    pub echelle: f32,
    pub vue_x: f32,
    pub vue_y: f32,
    /// Le pas de la grille en unités monde, **déjà doublé** autant de fois que le socle l'a
    /// décidé. Nul ou négatif quand la grille est éteinte.
    pub pas: f32,
    /// Le rayon d'un point et son opacité, tels que la fiche 06 les donne.
    pub rayon: f32,
    pub opacite: f32,
    /// Le gris d'un point, de 0 à 1.
    pub gris: f32,
    /// La hauteur du bandeau : aucun point ne se pose au-dessus.
    pub header: f32,
}

/// Ce que le reglage du fond occupe : quatre `vec4`.
const OCTETS_FOND: usize = 64;

impl Fond {
    /// La grille a-t-elle quelque chose à dessiner ?
    fn grille_visible(&self) -> bool {
        self.pas > 0.0 && self.echelle > 0.0 && self.opacite > 0.0 && self.rayon > 0.0
    }

    fn ecrire(&self, dans: &mut Vec<u8>, ecran: (f32, f32)) {
        for v in [
            self.rouge,
            self.vert,
            self.bleu,
            f32::from(u8::from(self.grille_visible())),
            self.echelle,
            self.vue_x,
            self.vue_y,
            self.pas,
            self.rayon,
            self.opacite,
            self.gris,
            self.header,
            ecran.0,
            ecran.1,
            0.0,
            0.0,
        ] {
            dans.extend_from_slice(&v.to_le_bytes());
        }
    }
}

const NUANCEUR: &str = r#"
struct Reglage {
    // La couleur du canevas, et 1 quand la grille se dessine.
    fond: vec4<f32>,
    // L'echelle, la translation de la vue, et le pas de la grille en unites monde.
    vue: vec4<f32>,
    // Le rayon d'un point, son opacite, son gris, et la hauteur du bandeau.
    point: vec4<f32>,
    // La taille de l'ecran, et deux reserves.
    ecran: vec4<f32>,
};

@group(0) @binding(0) var<uniform> r: Reglage;

// Trois sommets couvrent l'ecran, calcules depuis leur propre indice : ni tampon de
// sommets, ni maillage, ni matrice.
@vertex
fn vs(@builtin(vertex_index) v: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let couleur = r.fond.rgb;
    if (r.fond.w < 0.5) {
        return vec4<f32>(couleur, 1.0);
    }
    let p = position.xy;
    if (p.y < r.point.w) {
        return vec4<f32>(couleur, 1.0);
    }
    // Le pas a l'ecran. Un point du monde en `k x pas` se pose en `k x pas x echelle + vue`,
    // donc la grille est PERIODIQUE a l'ecran, et le point le plus proche est un arrondi.
    let pas = r.vue.w * r.vue.x;
    if (!(pas > 0.0)) {
        return vec4<f32>(couleur, 1.0);
    }
    let centre = round((p - r.vue.yz) / pas) * pas + r.vue.yz;
    // Le socle ne pose un point que si son CENTRE tombe dans le cadre, bandeau exclu : un
    // point dont le centre est au-dessus du bandeau ne deborde pas dessous.
    let dedans = centre.y >= r.point.w
        && centre.x >= 0.0
        && centre.x <= r.ecran.x
        && centre.y <= r.ecran.y;
    if (!dedans) {
        return vec4<f32>(couleur, 1.0);
    }
    // La couverture analytique du processeur : la distance signee au bord du disque.
    // FOND-GPU-1 : le pas ne descend jamais sous trente-deux pixels et le rayon plafonne a
    // deux et demi, donc ce point est le SEUL qui puisse toucher ce pixel.
    let d = length(p - centre);
    let couverture = clamp(r.point.x + 0.5 - d, 0.0, 1.0);
    let a = couverture * r.point.y;
    // Premultiplie : `gris x a + fond x (1 - a)`, la meme loi que le processeur.
    return vec4<f32>(mix(couleur, vec3<f32>(r.point.z), a), 1.0);
}
"#;

/// La passe du fond : un triangle, un nuanceur, aucun pixel écrit par le processeur.
pub struct FondGpu {
    pipeline: wgpu::RenderPipeline,
    liaison: wgpu::BindGroup,
    reglage: wgpu::Buffer,
    /// Cette image a-t-elle un fond que la carte doit peindre ?
    a_peindre: bool,
}

impl FondGpu {
    pub fn nouveau(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fond"),
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
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fond"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fond"),
            bind_group_layouts: &[Some(&disposition)],
            immediate_size: 0,
        });
        let pipeline = peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fond"),
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
                    // Le fond REMPLACE : il est opaque et couvre tout, comme le `fill` du
                    // processeur qu'il vient remplacer.
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
        });
        let reglage = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fond: reglage"),
            size: OCTETS_FOND as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fond"),
            layout: &disposition,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: reglage.as_entire_binding(),
            }],
        });
        Self {
            pipeline,
            liaison,
            reglage,
            a_peindre: false,
        }
    }

    /// **Écrit le réglage du fond.** À appeler avant d'ouvrir la passe.
    pub fn preparer(&mut self, file: &wgpu::Queue, ecran: (f32, f32), fond: Option<Fond>) {
        self.a_peindre = fond.is_some();
        let Some(fond) = fond else {
            return;
        };
        let mut octets = Vec::with_capacity(OCTETS_FOND);
        fond.ecrire(&mut octets, ecran);
        file.write_buffer(&self.reglage, 0, &octets);
    }

    /// **Peint le fond dans la passe en cours**, quand cette image en a un.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>) {
        if !self.a_peindre {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.liaison, &[]);
        passe.draw(0..3, 0..1);
    }

    /// Cette image a-t-elle un fond à peindre ?
    ///
    /// La couche du dessous s'en sert : quand le fond ne vient pas de la carte, c'est qu'il
    /// est dans cette couche, donc elle **remplace** au lieu de se composer.
    pub fn a_peindre(&self) -> bool {
        self.a_peindre
    }
}

#[cfg(test)]
mod tests;
