//! **Les lueurs sur la carte** : une ombre de boîte est un produit de deux profils, donc un
//! nuanceur de six lignes.
//!
//! # Pourquoi cette passe existe, et pourquoi elle est la première après les photos
//!
//! Sur la session de terrain du 21/09, une fois les photos passées sur la carte, `halos` est
//! le **premier poste réel** de tous les gestes dont l'utilisateur se plaint :
//!
//! ```text
//!     repos               4,10 ms   (premier)
//!     deplacer la vue     4,10 ms   (premier apres l'attente du tempo)
//!     glisser un noeud    5,00 ms   (premier)
//!     selectionner        4,63 ms   (premier)
//! ```
//!
//! La cause est géométrique, et [`crate::renderer::halo`] la nomme : une ombre déborde de sa
//! carte de la portée du flou, donc elle touche bien plus de pixels qu'elle n'en désigne —
//! 109 000 pour une carte de 240 × 60. Vingt cartes, et la passe couvre l'écran plusieurs
//! fois. Seize fils l'ont fait tomber de 8,86 à 4,43 ms, et c'est là que le processeur
//! s'arrête : le découpage est horizontal, et des cartes alignées verticalement n'occupent
//! que trois bandes sur seize.
//!
//! # Ce que la carte fait de mieux, et ce n'est pas « aller plus vite »
//!
//! Le remplissage est **exactement** ce pour quoi une carte est faite : des lots homogènes,
//! sans branche, sans dépendance entre pixels. La fiche 21 le dit dans ces termes, et c'est
//! la raison de cette passe — pas un repli parce que le processeur échouerait.
//!
//! Le processeur, lui, garde ce qu'il fait de mieux et que la carte ne saurait pas faire : le
//! **culling**, le calcul de la teinte symbiotique (qui dépend du voisinage), et la décision
//! de ce qui se dessine. C'est le socle de la fiche 21, et rien n'en descend ici.
//!
//! # LUEUR-GPU-1 — le nuanceur est la formule, pas une approximation de la formule
//!
//! [`crate::renderer::halo`] établit qu'une ombre de boîte floutée par une gaussienne est
//! **séparable**, et que c'est exact :
//!
//! ```text
//!     α(x, y) = A · P(x ; gauche, droite) · P(y ; haut, bas)
//!     P(t ; a, b) = Φ(t − a) − Φ(t − b)
//! ```
//!
//! Le processeur échantillonne `Φ` dans une table, parce qu'un masque pré-calculé doit être
//! discret. La carte n'a pas cette contrainte : elle évalue `Φ` **analytiquement**, par
//! `Φ(t) = ½(1 + erf(t / σ√2))`, avec l'approximation d'Abramowitz & Stegun 7.1.26 dont
//! l'erreur maximale vaut 1,5·10⁻⁷ — soit **un millième du demi-niveau** de huit bits.
//!
//! Ce n'est donc pas une voie qui abîme pour aller vite : c'est la même loi, écrite pour un
//! matériel qui sait l'évaluer par million. L'écart mesuré entre les deux voies est celui de
//! la table discrète du processeur, et un test le borne.

/// Où une lueur se pose, et de quelle couleur.
///
/// Les longueurs sont en **pixels d'écran**, comme la [`crate::renderer::halo::HaloBox`] dont
/// elle vient : c'est le socle qui a déjà mis la vue à l'échelle et fait le culling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lueur {
    /// La boîte **dilatée** de la carte — celle que le flou étale.
    pub gauche: f32,
    pub haut: f32,
    pub droite: f32,
    pub bas: f32,
    /// L'écart-type de la gaussienne, en pixels d'écran.
    pub sigma: f32,
    /// L'opacité au plateau, de 0 à 1.
    pub alpha: f32,
    /// La distance au-delà de laquelle la lueur ne peut plus changer un pixel (HALO-2).
    pub portee: f32,
    /// La teinte symbiotique, de 0 à 1 par canal.
    pub rouge: f32,
    pub vert: f32,
    pub bleu: f32,
}

/// Ce qu'une lueur occupe dans le tampon : trois `vec4`, l'alignement d'un élément de tableau.
const OCTETS_LUEUR: usize = 48;

impl Lueur {
    fn ecrire(&self, dans: &mut Vec<u8>) {
        for v in [
            self.gauche,
            self.haut,
            self.droite,
            self.bas,
            self.sigma,
            self.alpha,
            self.portee,
            0.0,
            self.rouge,
            self.vert,
            self.bleu,
            0.0,
        ] {
            dans.extend_from_slice(&v.to_le_bytes());
        }
    }
}

const NUANCEUR: &str = r#"
struct Lueur {
    // gauche, haut, droite, bas de la boite dilatee, en pixels d'ecran.
    boite: vec4<f32>,
    // sigma, alpha, portee, et une reserve : un element s'aligne sur seize octets.
    reglage: vec4<f32>,
    // La teinte symbiotique, et une reserve.
    couleur: vec4<f32>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> lueurs: array<Lueur>;
@group(0) @binding(1) var<uniform> ecran: Ecran;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) rang: u32,
};

// Le quad d'une lueur : sa boite dilatee, etendue de la PORTEE du flou de chaque cote.
// Au-dela, la lueur ne peut plus changer un pixel -- c'est HALO-2, et le socle l'a mesure.
@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    let coins = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let l = lueurs[i];
    let p = l.reglage.z;
    let e = mix(
        vec2<f32>(l.boite.x - p, l.boite.y - p),
        vec2<f32>(l.boite.z + p, l.boite.w + p),
        coins[v],
    );
    var s: Sortie;
    s.position = vec4<f32>(
        e.x / ecran.taille.x * 2.0 - 1.0,
        1.0 - e.y / ecran.taille.y * 2.0,
        0.0,
        1.0,
    );
    s.rang = i;
    return s;
}

// erf par Abramowitz & Stegun 7.1.26 : erreur maximale 1,5e-7, soit un milliieme du
// demi-niveau de huit bits. Ce n'est donc pas une approximation VISIBLE, c'est la formule.
fn erf(x: f32) -> f32 {
    let a = abs(x);
    let t = 1.0 / (1.0 + 0.3275911 * a);
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741
        + t * (-1.453152027 + t * 1.061405429))));
    return sign(x) * (1.0 - poly * exp(-a * a));
}

// La gaussienne cumulee : la fraction de la lueur qui tombe a gauche de la distance `t`.
fn phi(t: f32, inv: f32) -> f32 {
    return 0.5 * (1.0 + erf(t * inv));
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    let l = lueurs[e.rang];
    // 1 / (sigma * racine de deux). Le plancher evite la division par zero d'une carte
    // vue de tres loin ; a cette valeur, phi est deja la marche d'un bord net, ce qui est
    // exactement ce que le processeur produit avec un noyau d'un seul echantillon.
    let inv = 1.0 / (max(l.reglage.x, 1e-4) * 1.41421356);
    let p = e.position.xy;
    let px = phi(p.x - l.boite.x, inv) - phi(p.x - l.boite.z, inv);
    let py = phi(p.y - l.boite.y, inv) - phi(p.y - l.boite.w, inv);
    let a = l.reglage.y * px * py;
    // Premultiplie : la meme loi que `blend_pixel` du processeur, qui ecrit
    // `(c x a + d x (255 - a)) / 255` par canal.
    return vec4<f32>(l.couleur.rgb * a, a);
}
"#;

/// La passe des lueurs : un quad par carte, un nuanceur, aucun tampon de sommets.
pub struct Lueurs {
    pipeline: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    liaison: wgpu::BindGroup,
    tampon: wgpu::Buffer,
    ecran: wgpu::Buffer,
    /// Combien de lueurs le tampon peut porter avant d'être refait.
    capacite: usize,
    /// Combien [`Lueurs::preparer`] en a retenues pour cette image.
    posees: usize,
}

/// Combien de lueurs le tampon porte au départ — il grandit si l'écran en demande plus.
const LUEURS_AU_DEPART: usize = 256;

impl Lueurs {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = Self::disposition(peripherique);
        let pipeline = Self::pipeline(peripherique, format, &disposition);
        let tampon = Self::tampon(peripherique, LUEURS_AU_DEPART);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lueurs: ecran"),
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
            capacite: LUEURS_AU_DEPART,
            posees: 0,
        }
    }

    fn disposition(peripherique: &wgpu::Device) -> wgpu::BindGroupLayout {
        peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lueurs"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            ],
        })
    }

    fn pipeline(
        peripherique: &wgpu::Device,
        format: wgpu::TextureFormat,
        disposition: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lueurs"),
            source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
        });
        let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lueurs"),
            bind_group_layouts: &[Some(disposition)],
            immediate_size: 0,
        });
        peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lueurs"),
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
                    // Source-over en premultiplie : deux lueurs qui se chevauchent se
                    // composent dans l'ordre des instances, comme le processeur les peint
                    // l'une apres l'autre.
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
            label: Some("lueurs: poses"),
            size: (combien.max(1) * OCTETS_LUEUR) as u64,
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
            label: Some("lueurs"),
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

    /// **Écrit les lueurs dans le tampon.** À appeler avant d'ouvrir la passe : un tampon ne
    /// s'écrit pas pendant qu'on dessine.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        lueurs: &[Lueur],
    ) {
        self.posees = lueurs.len();
        if lueurs.is_empty() {
            return;
        }
        if lueurs.len() > self.capacite {
            // Le tampon suit ce que l'ecran demande : il grandit par doublements, donc il
            // cesse de grandir en quelques images. Aucun nombre n'a eu a etre choisi.
            self.capacite = lueurs.len().next_power_of_two();
            self.tampon = Self::tampon(peripherique, self.capacite);
            self.liaison =
                Self::liaison(peripherique, &self.disposition, &self.tampon, &self.ecran);
        }
        let mut octets = Vec::with_capacity(lueurs.len() * OCTETS_LUEUR);
        for lueur in lueurs {
            lueur.ecrire(&mut octets);
        }
        file.write_buffer(&self.tampon, 0, &octets);
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }

    /// **Pose dans la passe en cours** les lueurs que [`Lueurs::preparer`] a reçues.
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
