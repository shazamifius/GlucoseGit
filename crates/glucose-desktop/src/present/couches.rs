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
//! # Ce que coûte la seconde couche, et ce qui l'a fait disparaître
//!
//! Deux téléversements d'écran au lieu d'un : sur 2560 × 1600, deux fois quinze mébioctets.
//!
//! La première idée pour le réduire était de **ne pas re-téléverser le dessous quand il n'a
//! pas changé**, comme pour la bande du haut. Elle ne vaut rien, et il suffit de la regarder
//! pour le voir : pendant un déplacement ou un zoom — les seuls moments où la cadence est en
//! jeu — le fond se décale et les lueurs suivent leurs cartes. **Tout change à chaque image**,
//! donc un cache de salissure y gagnerait exactement zéro.
//!
//! Ce qui marche est plus simple et définitif : faire descendre le fond et les lueurs sur la
//! carte ([`super::fond_gpu`], [`super::lueurs_gpu`]). La couche du dessous ne porte alors
//! plus que les membranes et les dossiers, et sur un document qui n'en a pas elle est
//! **entièrement vide** — [`Couches::televerser`] la saute alors, et ce téléversement-là
//! n'existe plus du tout. Un coût supprimé vaut mieux qu'un coût mis en cache.

use super::bandes::Bandes;
use super::formes_gpu::Tranche;
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
    /// La couche du dessous porte-t-elle quelque chose pour l'image en cours ?
    dessous_pose: bool,
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
            dessous_pose: false,
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
    /// Fabrique la texture d'une couche et de quoi la lier au nuanceur.
    ///
    /// Une fonction libre plutôt qu'un bloc dans `accorder` : celle-ci décidait **et** fabriquait
    /// **et** téléversait, ce qui lui faisait passer les quatre-vingts lignes que la fiche 05
    /// admet — et le cliquet a eu raison de le dire.
    fn fabriquer(
        peripherique: &wgpu::Device,
        disposition: &wgpu::BindGroupLayout,
        filtre: &wgpu::Sampler,
        taille: (u32, u32),
    ) -> Portee {
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
        Portee {
            texture,
            liaison,
            taille,
        }
    }

    fn accorder(
        peripherique: &wgpu::Device,
        envoi: &mut super::envoi::Envoi,
        disposition: &wgpu::BindGroupLayout,
        filtre: &wgpu::Sampler,
        place: &mut Option<Portee>,
        (source, bandes): (&Pixmap, &Bandes),
    ) -> u32 {
        let taille = (source.width(), source.height());
        let refaire = place.as_ref().is_none_or(|p| p.taille != taille);
        if refaire {
            *place = Some(Self::fabriquer(peripherique, disposition, filtre, taille));
        }
        let Some(portee) = place.as_ref() else {
            return 0;
        };
        // **Une texture neuve ne contient rien de ce qu'on croit** : tout part, une fois.
        // Ensuite elle garde ce qu'on y a mis d'une image à l'autre -- il n'y a pas d'anneau
        // ici, contrairement à l'image principale -- et c'est précisément ce qui permet de
        // n'envoyer que ce qui a changé.
        let a_envoyer = if refaire {
            Bandes::tout(taille.1)
        } else {
            bandes.clone()
        };
        let largeur = taille.0 as usize * 4;
        let mut lignes = 0_u32;
        for bande in a_envoyer.intervalles() {
            let hauteur = bande.end - bande.start;
            if hauteur == 0 {
                continue;
            }
            lignes += hauteur;
            // L'origine porte la bande : le reste de la texture ne bouge pas.
            envoi.texture(
                &portee.texture,
                (0, bande.start),
                (taille.0, hauteur),
                (&source.data()[bande.start as usize * largeur..], largeur),
            );
        }
        lignes
    }

    /// Téléverse les deux couches du processeur. À faire avant d'ouvrir la passe.
    ///
    /// `dessous_utile` dit si la couche du dessous se pose. Quand elle ne porte rien — ni
    /// fond, ni ornement de membrane, ni dossier — **rien ne part** : ni ses bandes, ni le
    /// dessin qui composerait du transparent sur du fond.
    ///
    /// # Les deux couches partent par bandes (BANDE-1, BANDE-2)
    ///
    /// Le dessous partait entier, et la raison était juste tant qu'il portait la forme des
    /// membranes : elles suivent la vue, et un grand cadre touche toutes les lignes. Depuis que
    /// la carte les dessine (MEMB-FORME-1), il n'y reste que des titres, des poignées, des
    /// réglettes et des dossiers — quelques bandes, comme le dessus.
    pub fn televerser(
        &mut self,
        peripherique: &wgpu::Device,
        envoi: &mut super::envoi::Envoi,
        (dessous, dessous_utile, bandes_du_dessous): (&Pixmap, bool, &Bandes),
        (dessus, bandes): (&Pixmap, &Bandes),
    ) {
        self.dessous_pose = dessous_utile;
        let mut lignes = 0;
        if dessous_utile {
            lignes += Self::accorder(
                peripherique,
                envoi,
                &self.disposition,
                &self.echantillonneur,
                &mut self.dessous,
                (dessous, bandes_du_dessous),
            );
        }
        lignes += Self::accorder(
            peripherique,
            envoi,
            &self.disposition,
            &self.echantillonneur,
            &mut self.dessus,
            (dessus, bandes),
        );
        // Les deux couches ensemble : chacune écrasait le compte de l'autre, et le dessous,
        // envoyé le premier, n'apparaissait jamais.
        crate::perf::compteur("blit_lignes", f64::from(lignes));
    }

    /// Pose la couche du dessous, quand elle porte quelque chose.
    ///
    /// Elle **remplace** quand elle porte le fond — la voie processeur — et se **compose**
    /// quand la carte l'a peint elle-même : il ne reste alors que des membranes et des
    /// dossiers sur du transparent, et remplacer effacerait le fond qu'on vient de poser.
    pub fn poser_le_dessous(&self, passe: &mut wgpu::RenderPass<'_>, porte_le_fond: bool) {
        if !self.dessous_pose {
            return;
        }
        let Some(portee) = self.dessous.as_ref() else {
            return;
        };
        passe.set_pipeline(if porte_le_fond {
            &self.remplace
        } else {
            &self.compose
        });
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

/// Les cinq temps d'une image, dans l'ordre du modele.
///
/// Regroupes parce qu'ils voyagent toujours ensemble, et qu'une fonction qui les recevrait un
/// par un aurait cinq arguments dont l'ordre serait la seule protection.
pub struct Temps<'a> {
    pub fond: &'a super::fond_gpu::FondGpu,
    pub lueurs: &'a super::lueurs_gpu::Lueurs,
    /// Le corps des zones d'un gabarit, juste après le fond (PRESET-1).
    pub zones: &'a super::zones_gpu::Zones,
    /// Le rectangle de la vue sur la minimap (CHROME-1), entre le fond de la minimap et le rail.
    pub cadre: &'a super::zones_gpu::Zones,
    pub membranes: &'a super::membranes_gpu::Membranes,
    pub fleches: &'a super::fleches_gpu::FlechesGpu,
    pub lisere: &'a super::lisere_gpu::LisereGpu,
    pub couches: &'a Couches,
    pub scene: &'a super::scene_gpu::SceneGpu,
    pub retenues: &'a super::scene_gpu::Retenues,
    /// Ce qui est plus grand que l'écran, dessiné en formes (fiche 73), et où chaque suite se
    /// pose parmi les textures.
    pub formes: &'a super::formes_gpu::FormesGpu,
    pub suites: &'a [super::formes_gpu::Suite],
    /// Combien des textures demandées sont des photos — les premières : la pose se coupe
    /// après elles pour laisser passer les lueurs (LUEUR-3).
    pub photos: usize,
    /// Combien passent sous la couche du dessus — photos et cartes : les suivantes sont la
    /// chrome et les panneaux, posés après elle (PANNEAUX-1, CHROME-1).
    pub sous_le_dessus: usize,
    /// Combien passent sous le rectangle de la vue : jusqu'au fond de la minimap. Le rail, les
    /// panneaux et ce qui attend une décision viennent après (CHROME-1).
    pub sous_le_cadre: usize,
}

/// **Encode la passe des cinq temps** : le fond, le dessous, les photos, les lueurs et les
/// cartes, le dessus.
///
/// L'ordre est tout, et c'est celui de la voie processeur : le fond REMPLACE puisqu'il est
/// opaque et couvre tout ; la couche du dessous porte les membranes et les dossiers, qui sont
/// des CONTENANTS ; les photos viennent ensuite ; puis les lueurs des cartes, puis les cartes ;
/// et le dessus se compose en dernier, transparent partout ou les photos doivent se voir.
///
/// Les intervertir mettrait le fond par-dessus tout, ou une membrane par-dessus la photo
/// qu'elle contient.
///
/// # LUEUR-3 — la lueur d'une carte passe devant les photos
///
/// Elle se posait juste apres le fond, donc **sous** les photos : une carte posee sur une
/// photo perdait sa lueur, cachee par l'image qu'elle couvrait. Chez Tauri, la lueur est
/// l'ombre CSS d'un bloc HTML pose sur la toile, donc devant toute image. Elle se pose
/// desormais entre les photos et les cartes : devant ce que la carte recouvre, derriere les
/// autres cartes.
/// **Les textures d'une tranche, et ses formes à leur rang** (fiche 73) : chaque suite se pose
/// juste avant la texture qui la suit dans le modèle — `rangs` sont ceux des textures demandées.
fn poser_une_tranche(
    passe: &mut wgpu::RenderPass<'_>,
    temps: &Temps<'_>,
    (tranche, rangs): (Tranche, std::ops::Range<usize>),
) {
    let retenues = temps.retenues;
    let mut rang = rangs.start;
    for (k, suite) in temps.suites.iter().enumerate() {
        if suite.tranche != tranche {
            continue;
        }
        let jusqu_a = (rangs.start + suite.avant).min(rangs.end);
        temps.scene.poser(
            passe,
            retenues,
            retenues.avant(rang)..retenues.avant(jusqu_a),
        );
        temps.formes.poser(passe, k);
        rang = jusqu_a;
    }
    temps.scene.poser(
        passe,
        retenues,
        retenues.avant(rang)..retenues.avant(rangs.end),
    );
}

pub fn composer(encodeur: &mut wgpu::CommandEncoder, cible: &wgpu::TextureView, temps: Temps<'_>) {
    let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("glucose-couches"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: cible,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                // Rien a effacer : le fond couvre tout, qu'il vienne de la carte ou de la
                // couche du dessous.
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    temps.fond.poser(&mut passe);
    // Le corps des zones d'un gabarit : le premier rang du monde, sous les membranes, comme
    // le processeur le peint juste après le fond (PRESET-1).
    temps.zones.poser(&mut passe);
    // La forme des membranes, sous leurs titres qui sont dans la couche du dessous.
    temps.membranes.poser(&mut passe);
    temps
        .couches
        .poser_le_dessous(&mut passe, !temps.fond.a_peindre());
    let retenues = temps.retenues;
    let cartes = retenues.avant(temps.sous_le_dessus);
    poser_une_tranche(&mut passe, &temps, (Tranche::Photos, 0..temps.photos));
    temps.lueurs.poser(&mut passe);
    poser_une_tranche(
        &mut passe,
        &temps,
        (Tranche::Cartes, temps.photos..temps.sous_le_dessus),
    );
    // Les flèches, au-dessus des photos et des cartes, sous leurs étiquettes et leurs
    // poignées qui sont dans la couche du dessus (FLECHE-2).
    temps.fleches.poser(&mut passe);
    temps.couches.poser_le_dessus(&mut passe);
    // La chrome, après tout ce que la couche du dessus porte — la bande, le fil, le fond de la
    // minimap, puis le rectangle de la vue, puis le rail (CHROME-1) —, et les panneaux après
    // elle, comme la voie processeur les compose (PANNEAUX-1).
    let cadre = retenues.avant(temps.sous_le_cadre);
    temps.scene.poser(&mut passe, retenues, cartes..cadre);
    temps.cadre.poser(&mut passe);
    temps
        .scene
        .poser(&mut passe, retenues, cadre..retenues.len());
    // Le liseré du passé, par-dessus tout : il borde la fenêtre entière, chrome comprise.
    temps.lisere.poser(&mut passe);
}
