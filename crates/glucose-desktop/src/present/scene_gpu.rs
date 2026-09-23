//! **La voie graphique : les photos vivent sur la carte, et la vue n'est qu'une matrice.**
//!
//! # Ce que ce module change, et pourquoi il existe
//!
//! Jusqu'ici la carte ne faisait que **poser l'image finie** ([`super::gpu`]). Toute la scène
//! se composait sur le processeur, pixel par pixel, à chaque image — et c'est ce qui obligeait
//! à pixeliser en mouvement : interpoler quatre texels par pixel d'écran coûtait quatre fois
//! le budget d'une image.
//!
//! Ici, une photo décodée devient une **texture téléversée une fois**. Déplacer la vue ne
//! relit plus un seul octet de la source : cela change quatre nombres par photo. Et le
//! filtrage bilinéaire est **câblé dans le silicium** — il ne coûte rien, à aucune échelle.
//!
//! `bench_voie_gpu` le chiffre, sur 429 photos et 2560 × 1600, à une échelle non entière :
//!
//! ```text
//!     voie processeur, seize fils      4,2 a 6,3 ms   et PIXELISEE
//!     Intel Arc 140T (integre)              1,22 ms   et nette
//!     RTX 5070 Laptop (dediee)              0,26 ms   et nette
//! ```
//!
//! # Ce que ce module ne fait pas, et ne fera pas
//!
//! Ni le texte, ni les formes, ni les lueurs : ils restent au processeur, composés autour des
//! photos (fiche 21, étape 1). Il ne décide pas non plus **quand** la voie graphique sert —
//! c'est l'arbitre qui le fera, en observant le débit de chacune, et il n'existe pas encore.
//!
//! # Un dessin par photo, et pourquoi c'est le bon départ
//!
//! Chaque photo a sa texture, donc sa liaison de ressources, donc son dessin. Un atlas ou un
//! tableau de textures les regrouperait en un seul lot — c'est la suite, et elle demande de
//! borner la mémoire d'atlas et de replacer les photos qui entrent et sortent.
//!
//! Le mesurer avant de le compliquer : une carte moderne accepte des milliers de dessins par
//! image, et le **remplissage** domine de toute façon. La complexité d'un atlas ne se prend
//! que si un banc la réclame.

mod pose;
pub use pose::Pose;
use pose::OCTETS_POSE;

use crate::renderer::voies::APoser;
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

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
const NUANCEUR: &str = r#"
struct Pose {
    // x, y, largeur, hauteur, en pixels d'ecran.
    boite: vec4<f32>,
    // L'opacite, l'angle en radians, et deux reserves : un element s'aligne sur seize octets.
    reglage: vec4<f32>,
    // La fenetre de la source que ce quad montre, en fractions : u0, v0, largeur, hauteur.
    // (0, 0, 1, 1) est la texture entiere ; un recadrage la resserre (RECADRAGE-1).
    fenetre: vec4<f32>,
    // Ce que le filtre a le droit de lire : u et v minimaux, puis maximaux (BORDURES-4).
    bornes: vec4<f32>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> poses: array<Pose>;
@group(0) @binding(1) var<uniform> ecran: Ecran;
@group(0) @binding(2) var filtre: sampler;
@group(1) @binding(0) var source: texture_2d<f32>;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) opacite: f32,
    @location(2) @interpolate(flat) bornes: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    let coins = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = coins[v];
    let p = poses[i];
    // La rotation se fait autour du CENTRE de la photo, comme le modele la definit.
    let demi = vec2<f32>(p.boite.z, p.boite.w) * 0.5;
    let centre = vec2<f32>(p.boite.x, p.boite.y) + demi;
    let ecart = (c - vec2<f32>(0.5, 0.5)) * vec2<f32>(p.boite.z, p.boite.w);
    let a = p.reglage.y;
    let tourne = vec2<f32>(
        ecart.x * cos(a) - ecart.y * sin(a),
        ecart.x * sin(a) + ecart.y * cos(a),
    );
    let e = centre + tourne;
    var s: Sortie;
    s.position = vec4<f32>(e.x / ecran.taille.x * 2.0 - 1.0, 1.0 - e.y / ecran.taille.y * 2.0, 0.0, 1.0);
    // Le coin du quad se lit dans la fenetre de la source, et non dans la texture entiere :
    // c'est ainsi qu'un recadrage ne coute rien -- la carte echantillonne un sous-rectangle,
    // et aucun pixel n'est ecrit hors de la boite.
    s.uv = p.fenetre.xy + c * p.fenetre.zw;
    s.opacite = p.reglage.x;
    s.bornes = p.bornes;
    return s;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    // Premultiplie : la meme loi que `Melange::Composer` du noyau, pour que les deux voies
    // composent de la meme facon.
    // Les bords de la fenetre se prolongent, comme ceux de la texture (BORDURES-4) : le
    // filtre ne lit jamais ce que le recadrage a retire.
    let uv = clamp(e.uv, e.bornes.xy, e.bornes.zw);
    return textureSample(source, filtre, uv) * e.opacite;
}
"#;

/// Les photos que la carte détient, et de quoi les poser.
pub struct SceneGpu {
    pipeline: wgpu::RenderPipeline,
    /// Ce qui ne change pas d'une photo à l'autre : les poses, l'écran, le filtre.
    commun: wgpu::BindGroup,
    /// Ce qui change : la texture de la photo courante.
    disposition_photo: wgpu::BindGroupLayout,
    echantillonneur: wgpu::Sampler,
    poses: wgpu::Buffer,
    ecran: wgpu::Buffer,
    /// Combien de poses le tampon peut porter avant d'être refait.
    capacite: usize,
    /// Ce que la carte détient, **indexé par identité et non par clé**.
    ///
    /// Une identité porte au plus une texture : quand un composant change de palier, la
    /// nouvelle remplace l'ancienne, et la mémoire ne double jamais. Tant que la nouvelle
    /// n'est pas rendue, c'est l'ancienne qui se pose — un peu floue, jamais absente.
    photos: std::collections::HashMap<String, Televersee>,
    image: u64,
}

struct Televersee {
    liaison: wgpu::BindGroup,
    /// **Ce que cette texture montre** : la clé qui l'a produite.
    ///
    /// Quand elle diffère de celle que la scène demande, la texture est périmée — elle se
    /// pose quand même, et se refera dès qu'il y aura du temps pour elle.
    cle: String,
    /// La dernière image où cette photo a été posée.
    vue: u64,
}

/// Combien de poses le tampon porte au départ — il grandit si l'écran en demande plus.
///
/// Cinq cent douze : le document de l'utilisateur en compte quatre cent trente, et un écran
/// n'en montre jamais beaucoup plus à la fois puisque le culling les a déjà filtrées.
const POSES_AU_DEPART: usize = 512;

impl SceneGpu {
    /// Construit le pipeline. `format` est celui de la cible où les photos se poseront.
    pub fn nouvelle(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let (commun_layout, disposition_photo) = Self::dispositions(peripherique);
        let pipeline =
            Self::pipeline_des_photos(peripherique, format, &commun_layout, &disposition_photo);
        let echantillonneur = peripherique.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("scene: bilineaire"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let poses = Self::tampon_des_poses(peripherique, POSES_AU_DEPART);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene: ecran"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let commun = Self::liaison_commune(
            peripherique,
            &commun_layout,
            &poses,
            &ecran,
            &echantillonneur,
        );
        Self {
            pipeline,
            commun,
            disposition_photo,
            echantillonneur,
            poses,
            ecran,
            capacite: POSES_AU_DEPART,
            photos: std::collections::HashMap::new(),
            image: 0,
        }
    }

    /// Les deux groupes de ressources : ce qui vaut pour toute l'image, et ce qui change par
    /// photo.
    ///
    /// Les séparer n'est pas cosmétique : une liaison qui change à chaque dessin coûte, et
    /// celle-là ne porte plus qu'une texture.
    fn dispositions(peripherique: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
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

    fn pipeline_des_photos(
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

    fn tampon_des_poses(peripherique: &wgpu::Device, combien: usize) -> wgpu::Buffer {
        peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene: poses"),
            size: (combien * OCTETS_POSE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn liaison_commune(
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

    /// La carte détient-elle **exactement** ce que cette texture montre ?
    ///
    /// Une identité connue dont la clé diffère n'est pas « connue » : elle est périmée, et
    /// elle se refera quand le budget le permettra.
    pub fn connait(&self, identite: &str, cle: &str) -> bool {
        self.photos.get(identite).is_some_and(|t| t.cle == cle)
    }

    /// La carte détient-elle quelque chose à poser pour cette identité, fût-ce périmé ?
    pub fn detient(&self, identite: &str) -> bool {
        self.photos.contains_key(identite)
    }

    /// **Téléverse une photo décodée, une fois pour toutes.**
    ///
    /// C'est le seul moment où ses octets traversent le bus. Tout le reste de sa vie —
    /// déplacements, zooms, changements d'opacité — ne coûte que sa pose.
    pub fn televerser(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        (identite, cle): (&str, &str),
        source: &Pixmap,
    ) {
        let (l, h) = (source.width(), source.height());
        let taille = wgpu::Extent3d {
            width: l,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
            label: Some("photo"),
            size: taille,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // `tiny-skia` produit du RGBA huit bits premultiplie : les octets partent tels
            // quels, comme pour l'image de presentation.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        file.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            source.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(l * 4),
                rows_per_image: Some(h),
            },
            taille,
        );
        let vue = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("photo"),
            layout: &self.disposition_photo,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&vue),
            }],
        });
        let image = self.image;
        // La nouvelle texture **remplace** celle que cette identité portait : une identité
        // n'en a jamais deux, donc la mémoire ne double pas pendant un changement de palier.
        self.photos.insert(
            identite.to_string(),
            Televersee {
                liaison,
                cle: cle.to_string(),
                vue: image,
            },
        );
    }

    /// **Televerse ce que la carte ne connait pas encore**, et rien d'autre.
    ///
    /// `source` n'est appelee que pour les photos absentes : une photo ne traverse le bus
    /// qu'une fois dans sa vie, au premier affichage.
    pub fn assurer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        (a_poser, budget): (&[APoser], Duration),
        source: &dyn Fn(&str) -> Option<Pixmap>,
    ) {
        let debut = Instant::now();
        let mut faites = 0.0_f64;
        let mut reportees = 0.0_f64;
        let mut surface = 0.0_f64;
        // **Deux tours, et leur ordre EST la priorité.** Au premier, ce que la carte n'a pas
        // du tout : sans texture, un composant ne se dessine pas, et un trou se voit plus
        // qu'un flou. Au second, ce qu'elle détient mais qui a vieilli. L'urgent mange donc
        // le budget en premier, et le périmé prend ce qui reste — sans qu'aucun second
        // budget ait eu à être choisi.
        for urgent in [true, false] {
            for t in a_poser {
                if self.connait(&t.identite, &t.cle) {
                    continue;
                }
                if self.detient(&t.identite) == urgent {
                    continue;
                }
                // **Le budget vaut pour les deux tours**, et la première version se trompait
                // ici. Elle n'en exemptait que l'urgent, au motif qu'un trou est pire qu'un
                // flou — vrai pour une carte isolée, faux pour quatre cent quatre-vingts.
                // Le terrain a tranché : les douze images les plus lentes de la session du
                // 21/09 au soir sont toutes des dézooms et des vols de caméra, où toutes les
                // cartes entrent à l'écran ensemble, et `textures` y coûte jusqu'à 57,6 ms.
                // Geler une image d'un vingtième de seconde se voit bien plus que deux cents
                // cartes qui paraissent une image plus tard.
                //
                // **Au moins une par image, toujours.** C'est ce qui garantit qu'une scène
                // finit par se compléter, même quand chaque image dépasse déjà le plancher
                // et que le budget est nul — la même raison qui fait que `tranche_de_fond`
                // ne rend jamais zéro à l'atelier de décodage.
                if faites > 0.0 && debut.elapsed() >= budget {
                    reportees += 1.0;
                    continue;
                }
                if let Some(pixels) = source(&t.cle) {
                    // **Ce qu'une texture pese vraiment**, en kilopixels. Les chroniques du
                    // 21/09 montrent UNE texture a dix-neuf millisecondes, ce qu'aucune carte
                    // de texte ordinaire ne peut couter : il faut donc savoir laquelle, et la
                    // surface est la seule grandeur qui puisse l'expliquer.
                    surface += f64::from(pixels.width()) * f64::from(pixels.height()) / 1000.0;
                    self.televerser(peripherique, file, (&t.identite, &t.cle), &pixels);
                    faites += 1.0;
                }
            }
        }
        crate::perf::compteur("textures_kpx", surface);
        crate::perf::compteur("textures_faites", faites);
        crate::perf::compteur("textures_reportees", reportees);
    }

    /// Ouvre une image : ce qui ne servira pas d'ici à [`SceneGpu::fermer`] sera oublié.
    pub fn ouvrir(&mut self) {
        self.image += 1;
    }

    /// Oublie les photos qui n'ont pas servi à cette image.
    ///
    /// La borne du magasin est donc **ce que l'écran demande**, et rien d'autre : aucun nombre
    /// n'a eu à être choisi. C'est la même loi que le cache de tuiles.
    pub fn fermer(&mut self) {
        let image = self.image;
        self.photos.retain(|_, t| t.vue == image);
    }

    /// **Écrit les poses dans le tampon**, et rend celles que la carte saura dessiner.
    ///
    /// À appeler avant d'ouvrir la passe : un tampon ne s'écrit pas pendant qu'on dessine.
    /// Une photo que la carte ne connaît pas est simplement omise — c'est à l'appelant de la
    /// téléverser avant, et de savoir qu'elle manque.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        photos: &[APoser],
    ) -> Vec<String> {
        if photos.len() > self.capacite {
            // Le tampon suit ce que l'ecran demande, comme le cache de tuiles : il grandit
            // par doublements, donc il cesse de grandir en quelques images.
            self.capacite = photos.len().next_power_of_two();
            self.poses = Self::tampon_des_poses(peripherique, self.capacite);
            let layout = self.pipeline.get_bind_group_layout(0);
            self.commun = Self::liaison_commune(
                peripherique,
                &layout,
                &self.poses,
                &self.ecran,
                &self.echantillonneur,
            );
        }
        let mut octets = Vec::with_capacity(photos.len() * OCTETS_POSE);
        let mut retenues = Vec::with_capacity(photos.len());
        let image = self.image;
        let mut perimees = 0.0_f64;
        for t in photos {
            // **On pose ce que la carte détient pour cette identité**, à jour ou non. Une
            // texture périmée est l'ancien palier : elle se pose à la place demandée, donc
            // au bon endroit et à la bonne taille, et la carte la filtre. C'est un demi-pixel
            // d'adoucissement pendant une image ou deux, contre une image qui gèle.
            let Some(televersee) = self.photos.get_mut(&t.identite) else {
                continue;
            };
            if televersee.cle != t.cle {
                perimees += 1.0;
            }
            televersee.vue = image;
            t.pose.ecrire(&mut octets);
            retenues.push(t.identite.clone());
        }
        crate::perf::compteur("textures_perimees", perimees);
        if !octets.is_empty() {
            file.write_buffer(&self.poses, 0, &octets);
        }
        let mut cadre = Vec::with_capacity(16);
        for v in [ecran.0, ecran.1, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
        retenues
    }

    /// **Pose dans la passe en cours** les photos que [`SceneGpu::preparer`] a retenues.
    pub fn poser(&self, passe: &mut wgpu::RenderPass<'_>, retenues: &[String]) {
        if retenues.is_empty() {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.commun, &[]);
        for (rang, cle) in retenues.iter().enumerate() {
            let Some(televersee) = self.photos.get(cle) else {
                continue;
            };
            passe.set_bind_group(1, &televersee.liaison, &[]);
            let i = rang as u32;
            passe.draw(0..6, i..i + 1);
        }
    }
}

#[cfg(test)]
mod tests;
