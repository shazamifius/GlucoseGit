//! **Les nœuds dessinés de près par la carte, depuis leurs formes** (P8, fiche 73).
//!
//! # Ce qu'il a vu, et pourquoi
//!
//! De près, un nœud de texte devenait flou, en gros pixels, et ne redevenait net qu'au bout d'un
//! moment : la carte le posait comme une image que le processeur avait peinte, découpée en carrés
//! repeints un à un, l'image entière agrandie dix ou vingt fois en attendant (DE-PRES-1, fiche 72).
//!
//! Ici, la carte le dessine **depuis sa description vectorielle** — celle de l'export SVG
//! ([`crate::renderer::vectoriel`]) : chaque lettre par **Slug**, depuis son contour, chaque fond,
//! plaque, filet et puce par la **loi des membranes**. Rien n'est jamais repeint : net à tout zoom,
//! dès la première image, et un zoom ne coûte rien de plus qu'un déplacement.
//!
//! # Quand
//!
//! Exactement là où le découpage commençait : un nœud **plus grand que l'écran**. En deçà, sa
//! texture reste — rendue une fois, posée pour quatre nombres, ce qui vaut mieux que mille glyphes
//! quand on voit tout un tableau de loin. Aucune frontière n'est ajoutée : c'est la même.
//!
//! # Une passe, deux lois, l'ordre des instances
//!
//! Un arrondi et des courbes sont deux genres d'une même primitive, dans un même tampon : la carte
//! les compose dans l'ordre où ils viennent, comme le processeur les peint — la plaque sous le
//! texte, le texte sous son trait. Aucune passe ne s'intercale au milieu d'un nœud.

pub mod atlas;
pub use atlas::Source;

use atlas::Atlas;
use glucose_core::membrane_forme::Arrondi;

/// **Une primitive** que la carte dessine : un genre, une couleur, ce qui la rogne.
#[derive(Debug, Clone)]
pub struct Primitive {
    pub genre: Genre,
    /// La couleur, prémultipliée.
    pub couleur: [f32; 4],
    /// Ce qui la rogne, à l'écran : gauche, haut, droite, bas — l'infini quand rien ne la rogne.
    pub rogne: [f32; 4],
}

/// Ce qui ne rogne rien : le plus grand nombre fini, et non l'infini, qu'un pilote peut
/// traiter à sa façon.
pub const PARTOUT: [f32; 4] = [-f32::MAX, -f32::MAX, f32::MAX, f32::MAX];

#[derive(Debug, Clone)]
pub enum Genre {
    /// **Un rectangle arrondi à l'écran** — plein si `demi` est nul, sinon le trait de cette
    /// demi-largeur centré sur son bord : la loi des membranes (`membrane_forme`).
    Arrondi { forme: Arrondi, demi: f32 },
    /// **Une forme en courbes**, portée à l'écran par `matrice` — `(a, b, c, d, e, f)` : un point
    /// `(x, y)` de la forme va en `(a x + c y + e, b x + d y + f)` (Slug).
    Courbes { source: Source, matrice: [f32; 6] },
}

/// **Ce que la carte dessine en formes à cette image** : les primitives, et les suites qui les
/// rangent entre les textures — une suite par nœud, à son rang parmi les cartes.
#[derive(Debug, Clone, Default)]
pub struct Formes {
    pub primitives: Vec<Primitive>,
    pub suites: Vec<Suite>,
}

/// **Une suite** : des primitives qui se posent d'un coup, dans leur tranche, juste avant la
/// texture de rang `avant` de cette tranche — après toutes, si `avant` vaut leur nombre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suite {
    pub tranche: Tranche,
    pub avant: usize,
    pub primitives: std::ops::Range<usize>,
}

/// **Les deux tranches où une suite se pose** : parmi les photos (une photo en chemin), ou parmi
/// les cartes de texte — les lueurs passent entre les deux (LUEUR-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tranche {
    Photos,
    Cartes,
}

impl Formes {
    /// Ajoute ces formes au bout de celles-ci.
    pub fn etendre(&mut self, autres: Formes) {
        let decalage = self.primitives.len();
        self.primitives.extend(autres.primitives);
        self.suites.extend(autres.suites.into_iter().map(|s| Suite {
            primitives: s.primitives.start + decalage..s.primitives.end + decalage,
            ..s
        }));
    }
}

/// Ce qu'une primitive occupe dans le tampon : neuf `vec4`.
const OCTETS_PRIMITIVE: usize = 9 * 16;

/// **L'inverse de la partie linéaire d'une matrice** `(a, b, c, d, e, f)` : ce qui ramène un
/// point de l'écran, une fois la translation ôtée, dans l'unité de la forme (RACINES-1).
///
/// Calculé ici, une fois, et non par la carte : WGSL tolère deux ulps et demi d'erreur sur une
/// division, et le point de la forme sous un pixel décide **par son signe** si une racine compte.
/// Sur un trait horizontal posé pile à la hauteur d'un centre de pixel, un ulp faisait sauter la
/// couverture de cinquante-deux niveaux (fiche 73). Des deux côtés, le point se calcule ensuite
/// par les mêmes multiplications, que l'une et l'autre arrondissent correctement.
pub fn inverse([a, b, c, d, _, _]: [f32; 6]) -> [f32; 4] {
    let det = a * d - c * b;
    [d / det, -b / det, -c / det, a / det]
}

/// La loi des rectangles arrondis, commune aux membranes, puis les formes.
const NUANCEUR: &str = concat!(
    include_str!("arrondi.wgsl"),
    include_str!("formes_gpu/formes.wgsl")
);

/// Un tampon de lecture pour les nuanceurs, qui grandit par doublements.
struct Tampon {
    tampon: wgpu::Buffer,
    /// Combien d'octets il porte.
    capacite: u64,
    nom: &'static str,
}

impl Tampon {
    fn nouveau(peripherique: &wgpu::Device, nom: &'static str, octets: u64) -> Self {
        // Une liaison de stockage n'accepte pas un tampon vide.
        let capacite = octets.max(16).next_power_of_two();
        let tampon = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some(nom),
            size: capacite,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            tampon,
            capacite,
            nom,
        }
    }

    /// Assez grand pour `octets` ? Sinon refait, et le dit : tout est alors à renvoyer.
    fn tenir(&mut self, peripherique: &wgpu::Device, octets: u64) -> bool {
        if octets <= self.capacite {
            return false;
        }
        *self = Self::nouveau(peripherique, self.nom, octets);
        true
    }
}

/// La passe des formes : un quad par primitive, un nuanceur, l'atlas des formes.
pub struct FormesGpu {
    pipeline: wgpu::RenderPipeline,
    disposition: wgpu::BindGroupLayout,
    liaison: wgpu::BindGroup,
    primitives: Tampon,
    courbes: Tampon,
    bandes: Tampon,
    ecran: wgpu::Buffer,
    atlas: Atlas,
    /// Pour chaque suite de l'image, les instances écrites dans le tampon.
    posees: Vec<std::ops::Range<u32>>,
}

impl FormesGpu {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let disposition = disposition(peripherique);
        let pipeline = pipeline(peripherique, format, &disposition);
        let primitives = Tampon::nouveau(peripherique, "formes: primitives", 0);
        let courbes = Tampon::nouveau(peripherique, "formes: courbes", 0);
        let bandes = Tampon::nouveau(peripherique, "formes: bandes", 0);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("formes: ecran"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let liaison = liaison(
            peripherique,
            &disposition,
            [&primitives, &courbes, &bandes],
            &ecran,
        );
        Self {
            pipeline,
            disposition,
            liaison,
            primitives,
            courbes,
            bandes,
            ecran,
            atlas: Atlas::default(),
            posees: Vec::new(),
        }
    }

    /// **Écrit les formes de cette image.** À appeler avant d'ouvrir la passe.
    pub fn preparer(
        &mut self,
        (peripherique, file): (&wgpu::Device, &wgpu::Queue),
        ecran: (f32, f32),
        formes: &Formes,
    ) {
        // Le resserrement lit ce que l'image d'avant a demandé : il se fait avant d'écrire
        // celle-ci, dont les instances nomment les places du nouvel atlas.
        self.atlas.resserrer();
        self.atlas.ouvrir();
        self.posees.clear();
        let mut octets = Vec::with_capacity(formes.primitives.len() * OCTETS_PRIMITIVE);
        let mut ecrites = 0u32;
        for suite in &formes.suites {
            let debut = ecrites;
            for p in &formes.primitives[suite.primitives.clone()] {
                if self.ecrire(p, &mut octets) {
                    ecrites += 1;
                }
            }
            self.posees.push(debut..ecrites);
        }
        self.envoyer((peripherique, file), &octets);
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }

    /// Écrit une primitive dans l'ordre des champs du nuanceur ; `false` si elle ne dessine rien
    /// (un glyphe sans contour).
    fn ecrire(&mut self, p: &Primitive, dans: &mut Vec<u8>) -> bool {
        let mut champs = [[0u32; 4]; 9];
        let f = |v: [f32; 4]| v.map(f32::to_bits);
        match &p.genre {
            Genre::Arrondi { forme, demi } => {
                champs[0] = f([forme.gauche, forme.haut, forme.droite, forme.bas]);
                champs[1] = f([forme.rayon, *demi, 0.0, 0.0]);
                champs[7] = [1, 1, 0, 0];
            }
            Genre::Courbes { source, matrice: m } => {
                let Some(r) = self.atlas.ranger(source) else {
                    return false;
                };
                champs[0] = f([m[0], m[1], m[2], m[3]]);
                champs[1] = f([m[4], m[5], 0.0, 0.0]);
                champs[2] = f(r.boite);
                champs[3] = f(r.transformation);
                champs[6] = [1, r.courbes, r.bandes, 0];
                champs[7] = [r.nombre_de_bandes[0], r.nombre_de_bandes[1], 0, 0];
                champs[8] = f(inverse(*m));
            }
        }
        champs[4] = f(p.couleur);
        champs[5] = f(p.rogne);
        for v in champs.iter().flatten() {
            dans.extend_from_slice(&v.to_le_bytes());
        }
        true
    }

    /// Envoie les instances, et ce que l'atlas a gagné depuis le dernier envoi.
    fn envoyer(&mut self, (peripherique, file): (&wgpu::Device, &wgpu::Queue), instances: &[u8]) {
        let atlas = &mut self.atlas;
        let mut refaite = self.primitives.tenir(peripherique, instances.len() as u64);
        if self
            .courbes
            .tenir(peripherique, (atlas.courbes.len() * 16) as u64)
        {
            atlas.envoyees.0 = 0;
            refaite = true;
        }
        if self
            .bandes
            .tenir(peripherique, (atlas.bandes.len() * 4) as u64)
        {
            atlas.envoyees.1 = 0;
            refaite = true;
        }
        if refaite {
            let tampons = [&self.primitives, &self.courbes, &self.bandes];
            self.liaison = liaison(peripherique, &self.disposition, tampons, &self.ecran);
        }
        let (c, b) = atlas.envoyees;
        if c < atlas.courbes.len() {
            let octets: Vec<u8> = atlas.courbes[c..]
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            file.write_buffer(&self.courbes.tampon, (c * 16) as u64, &octets);
        }
        if b < atlas.bandes.len() {
            let octets: Vec<u8> = atlas.bandes[b..]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            file.write_buffer(&self.bandes.tampon, (b * 4) as u64, &octets);
        }
        atlas.envoyees = (atlas.courbes.len(), atlas.bandes.len());
        if !instances.is_empty() {
            file.write_buffer(&self.primitives.tampon, 0, instances);
        }
    }

    /// **Pose dans la passe en cours** la suite de rang `suite`.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>, suite: usize) {
        let Some(instances) = self.posees.get(suite).filter(|r| !r.is_empty()) else {
            return;
        };
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.liaison, &[]);
        passe.draw(0..6, instances.clone());
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
    peripherique.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("formes"),
        entries: &[
            entree(0, wgpu::ShaderStages::VERTEX_FRAGMENT, lecture),
            entree(
                1,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
            ),
            entree(2, wgpu::ShaderStages::FRAGMENT, lecture),
            entree(3, wgpu::ShaderStages::FRAGMENT, lecture),
        ],
    })
}

fn pipeline(
    peripherique: &wgpu::Device,
    format: wgpu::TextureFormat,
    disposition: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let module = peripherique.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("formes"),
        source: wgpu::ShaderSource::Wgsl(NUANCEUR.into()),
    });
    let agencement = peripherique.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("formes"),
        bind_group_layouts: &[Some(disposition)],
        immediate_size: 0,
    });
    peripherique.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("formes"),
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
                // Source-over en prémultiplié, dans l'ordre des instances : la plaque sous le
                // texte, le texte sous son trait, comme le processeur les peint.
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

fn liaison(
    peripherique: &wgpu::Device,
    disposition: &wgpu::BindGroupLayout,
    [primitives, courbes, bandes]: [&Tampon; 3],
    ecran: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let tampons = [&primitives.tampon, ecran, &courbes.tampon, &bandes.tampon];
    let entrees: Vec<wgpu::BindGroupEntry<'_>> = (0u32..)
        .zip(tampons)
        .map(|(binding, tampon)| wgpu::BindGroupEntry {
            binding,
            resource: tampon.as_entire_binding(),
        })
        .collect();
    peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("formes"),
        layout: disposition,
        entries: &entrees,
    })
}

#[cfg(test)]
mod tests;
