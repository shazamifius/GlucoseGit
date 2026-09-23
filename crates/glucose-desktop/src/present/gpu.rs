//! La présentation par le processeur graphique (PRESENT-1).
//!
//! # Ce que ce module fait, et ce qu'il ne fait pas
//!
//! Il ne dessine rien de la scène. Il prend l'image que `tiny-skia` vient de rendre et la met
//! à l'écran — c'est tout, et c'est déjà le poste le plus cher de l'application dès que la
//! fenêtre est grande.
//!
//! Le pipeline est donc minuscule : une texture, un échantillonneur, et **un seul triangle**
//! qui déborde de l'écran. Trois sommets suffisent à couvrir toute la surface, et ils sont
//! calculés dans le nuanceur à partir de leur propre indice — il n'y a ni tampon de sommets,
//! ni maillage, ni matrice.
//!
//! # Pourquoi aucune conversion
//!
//! `tiny-skia` rend du RGBA à huit bits par canal. `Rgba8Unorm` attend exactement cette
//! disposition. Les octets partent donc **tels quels**, sans être relus un par un : c'est
//! toute la différence avec le chemin processeur, qui doit réécrire chaque pixel dans le
//! `0RGB` que la fenêtre réclame.
//!
//! L'alpha est ignoré à l'affichage — la scène est opaque, et le forcer à un dans le nuanceur
//! évite qu'un fond translucide se mêle à ce qu'il y a derrière la fenêtre.
//!
//! # L'échantillonnage est au plus proche, et c'est une exigence
//!
//! La texture a **exactement** la taille de la surface : un pixel de l'image est un pixel de
//! l'écran. Un filtrage linéaire n'aurait rien à interpoler, mais il flouterait au moindre
//! écart d'un demi-pixel — et la charte demande « net à quasi 100 % à l'arrêt ». Au plus
//! proche, la correspondance est exacte ou elle ne l'est pas, et un test le vérifie.

use super::Issue;
use super::Presenter;
use succession::{cadence_demandee, nom_de_la_cadence};

use crate::error::{DesktopError, DesktopResult};
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::window::Window;

/// Le nuanceur : un triangle plein écran, et la texture telle quelle.
const SHADER: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Trois sommets qui débordent de l'écran : (0,0), (2,0), (0,2) en coordonnées de texture,
// donc (-1,-1), (3,-1), (-1,3) en coordonnées d'écran. Le triangle couvre tout le cadre, et
// ce qui dépasse est découpé — c'est moins de travail qu'un quadrilatère en deux triangles,
// dont la diagonale ferait rastériser deux fois la même ligne de pixels.
@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    out.uv = uv;
    // L'image a son origine en haut à gauche, l'écran en bas à gauche : l'ordonnée s'inverse.
    out.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var au_plus_proche: sampler;

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(image, au_plus_proche, in.uv).rgb, 1.0);
}
"#;

/// La présentation par le processeur graphique.
pub struct GpuPresenter {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    layout: wgpu::BindGroupLayout,
    /// Les textures qui portent l'image, en anneau (voir le module `anneau`).
    anneau: anneau::Anneau,
    /// Le nom de l'adaptateur retenu, pour que l'application puisse le dire.
    adaptateur: String,
    /// Le format de la texture qui porte l'image (GAMMA-1).
    format_image: wgpu::TextureFormat,
    /// Les photos que la carte detient, et de quoi les poser (fiche 21, etape 1).
    scene: super::scene_gpu::SceneGpu,
    /// Les deux couches du processeur, autour des photos.
    couches: super::couches::Couches,
    /// Le fond du canevas et sa grille (fiche 21, etape 3).
    fond: super::fond_gpu::FondGpu,
    /// Les lueurs des cartes (fiche 21, etape 3).
    lueurs: super::lueurs_gpu::Lueurs,
    /// Les bandes de la couche du dessus que la texture détient déjà.
    ///
    /// Cette couche n'a pas d'anneau : sa texture garde son contenu d'une image à l'autre,
    /// ce qui permet de n'y écrire que ce qui a changé. Encore faut-il savoir ce qu'elle
    /// porte, pour effacer sur la carte les bandes que le processeur n'écrit plus.
    bandes_envoyees: super::bandes::Bandes,
    /// La chaîne d'images doit être refaite avant la prochaine acquisition.
    ///
    /// # Le plantage que ce drapeau répare
    ///
    /// Reconfigurer la surface **détruit** la chaîne d'images et la recrée. Le faire pendant
    /// qu'on tient l'image acquise arrachait donc le sol sous ses pieds : la couche graphique
    /// refusait la commande — « the `SurfaceOutput` must be dropped before re-configuring » —
    /// puis la présentation échouait à son tour sur « Surface is not configured for
    /// presentation », et le pilote se retrouvait avec une image qui n'appartenait plus à rien.
    ///
    /// La réparation se **diffère** donc jusqu'au début de la présentation suivante, moment où
    /// aucune image n'est détenue. L'image en cours s'affiche quand même : la sauter se
    /// verrait, alors qu'une chaîne un peu désaccordée ne se voit pas.
    a_reaccorder: bool,
}

mod anneau;
mod cinq_temps;
mod ouverture;
pub mod succession;

impl GpuPresenter {
    /// Ouvre une présentation graphique sur cette fenêtre, ou dit pourquoi elle ne peut pas.
    ///
    /// Aucune de ces erreurs n'est anormale : une machine virtuelle, un bureau distant ou un
    /// pilote absent sont des cas de tous les jours. L'appelant retombe alors sur le chemin
    /// processeur, et le dit.
    pub fn new(window: Arc<Window>, width: NonZeroU32, height: NonZeroU32) -> DesktopResult<Self> {
        Self::sur_la_carte(window, width, height, succession::carte_demandee())
    }

    /// La meme, sur la carte que l'arbitre designe (ARBITRE-1).
    ///
    /// `GLUCOSE_CARTE` garde le dernier mot : quand il est pose, l'utilisateur a tranche et
    /// l'arbitre n'a plus rien a arbitrer.
    pub fn sur_la_carte(
        window: Arc<Window>,
        width: NonZeroU32,
        height: NonZeroU32,
        carte: wgpu::PowerPreference,
    ) -> DesktopResult<Self> {
        Self::avec_cadence(window, width, height, cadence_demandee(), carte)
    }

    /// La même chose, en imposant la façon dont les images se succèdent.
    ///
    /// `None` laisse le choix par défaut, qui est `Fifo` — l'image attend le balayage de
    /// l'écran. C'est ce qu'on veut à l'usage : pas de déchirure, et le fil dort au lieu de
    /// produire des images que personne ne verra.
    ///
    /// Mais c'est aussi ce qui rend la mesure trompeuse, et le banc l'a montré : présenter
    /// semblait coûter 4,74 ms, avec une régularité suspecte — exactement la période d'un
    /// écran à 240 Hz. Ce n'était pas du travail, c'était de l'attente. Pour chiffrer le
    /// travail, il faut demander `Immediate`.
    pub fn avec_cadence(
        window: Arc<Window>,
        width: NonZeroU32,
        height: NonZeroU32,
        cadence: Option<wgpu::PresentMode>,
        carte: wgpu::PowerPreference,
    ) -> DesktopResult<Self> {
        let echec = |quoi: &str, e: &dyn std::fmt::Display| {
            DesktopError::WindowError(format!("présentation graphique — {quoi} : {e}"))
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .map_err(|e| echec("surface", &e))?;

        let (adapter, device, queue, adaptateur) =
            ouverture::ouvrir(&instance, &surface, width, height, carte)?;

        let config = ouverture::accorder_la_surface(&surface, &adapter, (width, height), cadence)?;
        // **La surface se configure ICI, a la naissance.**
        //
        // Elle ne l'etait nulle part : `resize` le faisait, mais il court-circuite quand les
        // dimensions n'ont pas change -- ce qui est le cas juste apres la construction. Au
        // demarrage, ce sont les deux `Resized` que `winit` envoie de lui-meme qui
        // configuraient la surface, par accident (fiche 19 § 5.1 les chiffre a 191 et 37 ms).
        //
        // Une reouverture, elle, ne recoit aucun `Resized` : la premiere image demandait donc
        // une texture a une surface non configuree, et wgpu paniquait -- « Surface is not
        // configured for presentation ». Le defaut dormait depuis toujours et n'attendait que
        // le jour ou une carte se rouvrirait, c'est-a-dire le jour ou ARBITRE-2 a rendu la
        // bascule possible.
        //
        // Un objet naissant pret a servir ne depend d'aucun evenement exterieur.
        surface.configure(&device, &config);
        let (pipeline, layout, sampler) = atelier(&device, config.format);
        // La texture porte les octets tels quels quand la surface est linéaire. Si aucun
        // format non-sRGB n'était disponible, elle se déclare sRGB pour que le sampler
        // défasse ce que la sortie refera — les deux conversions s'annulent alors.
        let format_image = if config.format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        // La voie graphique de la scene se construit ici, ou le peripherique vit : elle pose
        // les photos dans le format de la SURFACE, celui ou tout se compose (fiche 21).
        let scene = super::scene_gpu::SceneGpu::nouvelle(&device, config.format);
        let couches = super::couches::Couches::nouvelles(&device, config.format);
        let fond = super::fond_gpu::FondGpu::nouveau(&device, config.format);
        let lueurs = super::lueurs_gpu::Lueurs::nouvelles(&device, config.format);
        Ok(Self {
            scene,
            couches,
            fond,
            lueurs,
            surface,
            device,
            queue,
            config,
            pipeline,
            sampler,
            layout,
            anneau: anneau::Anneau::nouveau(),
            bandes_envoyees: super::bandes::Bandes::default(),
            adaptateur,
            format_image,
            a_reaccorder: false,
        })
    }

    /// Acquiert l'image de la chaîne, en réparant celle-ci d'abord si une réparation est due.
    ///
    /// Rend `Ok(None)` quand il n'y a **rien à afficher** : la fenêtre est réduite, cachée
    /// derrière une autre, ou le compositeur a mis du temps à rendre la main. Dessiner dans le
    /// vide serait du travail perdu, et le signaler comme une panne ferait crier l'application
    /// à chaque fois qu'on la minimise.
    fn acquerir(&mut self) -> DesktopResult<Result<wgpu::SurfaceTexture, Issue>> {
        // Aucune image n'est détenue ici : c'est le seul instant où reconfigurer est licite.
        if self.a_reaccorder {
            self.surface.configure(&self.device, &self.config);
            self.a_reaccorder = false;
        }

        use wgpu::CurrentSurfaceTexture as Etat;
        match self.surface.get_current_texture() {
            Etat::Success(frame) => Ok(Ok(frame)),
            // La surface tient encore, mais elle ne correspond plus tout à fait à la fenêtre.
            // On affiche quand même — sauter une image se verrait — et la réparation se fait
            // à la prochaine acquisition, quand plus personne ne tiendra cette image.
            Etat::Suboptimal(frame) => {
                self.a_reaccorder = true;
                Ok(Ok(frame))
            }
            // **Deux refus qui ne se traitent pas pareil.** Une fenêtre recouverte n'a besoin
            // de rien ; une attente dépassée est un accident dont la suivante peut se remettre,
            // et l'image doit être redemandée — sans quoi la dernière reste à l'écran.
            Etat::Occluded => Ok(Err(Issue::Cachee)),
            Etat::Timeout => Ok(Err(Issue::Perdue)),
            // **La surface a vieilli : on la répare ICI, et on réessaie tout de suite.**
            //
            // La version précédente posait le drapeau, rendait une erreur, et laissait la
            // réparation à l'acquisition suivante. L'image en cours était donc perdue --
            // rendue pour rien, jamais présentée -- et la session du 21/09 au soir en compte
            // **quatre-vingt-six**, chacune accompagnée d'une ligne sur la sortie d'erreur.
            // Le tempo les voyait comme des gels : 2,5 % des images à trente-deux balayages,
            // exactement la part des images perdues, et un pire gel de 508 ms.
            //
            // Rien n'interdit de reconfigurer à cet instant : `get_current_texture` vient
            // d'échouer, donc aucune image n'est détenue -- c'est la condition que le drapeau
            // [`GpuPresenter::a_reaccorder`] existe pour garantir, et elle est remplie ici.
            //
            // Une surface périmée n'est d'ailleurs pas une panne : c'est un événement normal
            // du cycle de vie d'une fenêtre, que le compositeur provoque en redimensionnant,
            // en changeant de résolution ou en déplaçant la fenêtre d'un écran à l'autre. La
            // signaler comme une erreur faisait crier l'application pour un cas prévu.
            Etat::Outdated | Etat::Lost => {
                // **Combien de fois la surface a dû être refaite.** Reconfigurer n'est pas
                // gratuit -- la chaîne d'images est détruite et recréée -- et remplacer une
                // image perdue par une reconfiguration peut coûter plus cher que le défaut
                // qu'on répare. Sans ce compteur, la chronique montre un `acquerir` lourd
                // sans dire s'il vient de là ou d'une chaîne saturée.
                crate::perf::compteur("surfaces_refaites", 1.0);
                self.surface.configure(&self.device, &self.config);
                self.a_reaccorder = false;
                match self.surface.get_current_texture() {
                    Etat::Success(frame) | Etat::Suboptimal(frame) => Ok(Ok(frame)),
                    // Deux échecs de suite : la fenêtre n'est probablement pas affichable en
                    // ce moment. On saute l'image — mais on en redemande une, parce que rien
                    // ne dit que la suivante échouera aussi, et que ne rien redemander fige le
                    // canevas jusqu'au prochain geste.
                    _ => Ok(Err(Issue::Perdue)),
                }
            }
            // Ce qui reste est une vraie panne -- mémoire épuisée, périphérique perdu -- et
            // doit se dire.
            autre => {
                self.a_reaccorder = true;
                Err(DesktopError::WindowError(format!(
                    "la surface graphique doit être refaite : {autre:?}"
                )))
            }
        }
    }

    /// Le nom de la carte retenue.
    pub fn adaptateur(&self) -> &str {
        &self.adaptateur
    }

    /// Comment les images se succèdent — ce qui décide si présenter attend ou non.
    pub fn cadence(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    /// Combien d'images la chaîne garde en vol.
    ///
    /// Se dit au démarrage, à côté de la cadence : une attente à l'acquisition ne se
    /// comprend pas sans ce nombre, et il n'était écrit nulle part.
    pub fn images_en_vol(&self) -> u32 {
        self.config.desired_maximum_frame_latency
    }

    /// Téléverse l'image dans la texture, sans rien convertir.
    ///
    /// C'est la moitié du travail que la carte graphique fait à la place du processeur, et la
    /// seule qui lui coûte vraiment : mesurée à 0,79 ms sur 2560 × 1600, contre 3,00 ms pour
    /// convertir puis recopier du côté processeur.
    fn televerser(&mut self, pixmap: &Pixmap) {
        let (w, h) = (pixmap.width(), pixmap.height());
        // Les champs s'empruntent separement -- l'anneau en ecriture, le reste en lecture --
        // ce qu'une methode prenant `&self` entier interdirait.
        let Self {
            device,
            layout,
            sampler,
            format_image,
            config,
            anneau,
            ..
        } = self;
        let fabrique = anneau::Fabrique {
            device,
            layout,
            sampler,
            format: *format_image,
            // Une de plus que ce que la chaine garde en vol : pendant que la carte lit les
            // siennes, il en reste exactement une de libre pour l'ecriture.
            combien: config.desired_maximum_frame_latency.max(1) as usize + 1,
        };
        let texture = anneau.pour(&fabrique, w, h);

        // Les octets de `tiny-skia` partent tels quels : c'est ici que la conversion du chemin
        // processeur disparaît, et c'est tout l'intérêt du détour par la carte graphique.
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixmap.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        // Ce qu'on envoie a la carte, en mebioctets. Une image entiere part a CHAQUE frame :
        // a cinquante images par seconde en 4K, cela fait plus d'un gigaoctet par seconde vers
        // une carte qui partage sa memoire avec le processeur. Le savoir, plutot que le
        // supposer, dit si la presentation attend parce qu'elle est saturee.
        crate::perf::compteur("blit_mo", f64::from(w) * f64::from(h) * 4.0 / 1_048_576.0);
        crate::perf::stage("blit");
    }
}

/// montage, pas de la présentation, et le garder dans l'ouverture y mélangeait deux sujets.
/// Le nuanceur, la disposition des liaisons, le pipeline et l'échantillonneur.
///
/// Tout cela se construit une fois et ne dépend que du format de la surface — c'est du
/// montage, pas de la présentation, et le garder dans l'ouverture y mélangeait deux sujets.
fn atelier(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Sampler) {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("glucose-presentation"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("glucose-image"),
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("glucose-presentation"),
        bind_group_layouts: &[Some(&layout)],
        ..Default::default()
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("glucose-presentation"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("glucose-au-plus-proche"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (pipeline, layout, sampler)
}

impl Presenter for GpuPresenter {
    /// Réaccorde la surface — **et seulement si la taille a vraiment changé** (PRESENT-2).
    ///
    /// # Le défaut que cette garde répare, et ce qu'il coûtait
    ///
    /// L'application appelle ceci à chaque image, sans vérifier : c'était sans conséquence
    /// avec `softbuffer`, dont le redimensionnement ne fait rien quand rien ne bouge. Mais
    /// `Surface::configure` est tout autre chose — il détruit la chaîne d'images, la recrée,
    /// et attend que la carte ait fini ce qu'elle avait en cours.
    ///
    /// Le faire soixante fois par seconde rendait l'application **plus lente qu'elle ne l'a
    /// jamais été** : le premier poste d'une image passait de 0,03 ms à 45–97 ms, tout geste
    /// traînait, et tirer le bord de la fenêtre la figeait. Une fonction qu'on croit gratuite
    /// et qui ne l'est pas est pire qu'une fonction lente : personne ne la soupçonne.
    ///
    /// La garde vit ici plutôt que chez l'appelant, parce que c'est cette implémentation-ci
    /// qui sait ce que l'opération coûte.
    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> DesktopResult<()> {
        // La surface est configuree des la construction : a dimensions egales, il n'y a
        // vraiment rien a faire, et ce court-circuit ne cache plus une surface muette.
        if self.config.width == width.get() && self.config.height == height.get() {
            return Ok(());
        }
        self.config.width = width.get();
        self.config.height = height.get();
        // Licite ici : le redimensionnement arrive entre deux images, donc aucune n'est
        // detenue. La chaine etant refaite a neuf, une reparation en attente n'a plus d'objet.
        self.surface.configure(&self.device, &self.config);
        self.a_reaccorder = false;
        Ok(())
    }

    fn present(&mut self, pixmap: &Pixmap) -> DesktopResult<Issue> {
        self.televerser(pixmap);

        // Les trois temps de la présentation, séparés parce qu'ils n'ont pas la même nature :
        // **acquerir** peut attendre que l'écran rende une image du carrousel, **encoder** est
        // du travail de processeur, **soumettre** confie le tout à la carte. Mesurés ensemble,
        // ils annonçaient 28 ms sur un canevas vide sans dire lequel les portait.
        let frame = match self.acquerir()? {
            Ok(frame) => frame,
            Err(issue) => return Ok(issue),
        };
        crate::perf::stage("acquerir");
        let cible = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encodeur = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("glucose-presentation"),
            });
        {
            let bind = self.anneau.courante();
            let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("glucose-presentation"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cible,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        // Rien à effacer : le triangle couvre toute la surface.
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            passe.set_pipeline(&self.pipeline);
            passe.set_bind_group(0, bind, &[]);
            passe.draw(0..3, 0..1);
        }
        crate::perf::stage("encoder");
        // Les deux dernières moitiés se mesurent séparément, parce qu'elles n'attendent pas la
        // même chose : **soumettre** peut buter sur une file de commandes pleine, **présenter**
        // sur le compositeur qui ne rend pas la main. Mesurées ensemble, elles ont annoncé des
        // pics de quatre cent quarante millisecondes sur des images sans une seule photo, sans
        // jamais dire laquelle les portait.
        self.queue.submit(Some(encodeur.finish()));
        crate::perf::stage("soumettre");
        // La vue sur l'image de la surface doit être relâchée avant de la rendre au
        // compositeur : elle l'emprunte.
        drop(cible);
        self.queue.present(frame);
        crate::perf::stage("present");
        Ok(Issue::Presentee)
    }

    fn pose_les_photos(&self) -> bool {
        true
    }

    /// **La scene en cinq temps** : le fond, les lueurs, le dessous, les photos, le dessus.
    ///
    /// Le corps vit dans [`cinq_temps`] : ce fichier decrivait deja la presentation d'une
    /// image finie, et melanger les deux le faisait passer les six cents lignes.
    fn presenter_en_couches(
        &mut self,
        dessous: &Pixmap,
        (confie, budget): (&crate::renderer::Confie, std::time::Duration),
        source: &crate::present::scene_gpu::Source<'_>,
        dessus: &Pixmap,
    ) -> DesktopResult<Issue> {
        cinq_temps::presenter(self, (dessous, dessus), (confie, budget), source)
    }

    fn nom(&self) -> &'static str {
        "carte graphique"
    }

    fn images_en_vol(&self) -> u32 {
        self.config.desired_maximum_frame_latency
    }

    fn rythme(&self) -> &'static str {
        nom_de_la_cadence(self.config.present_mode)
    }
}
