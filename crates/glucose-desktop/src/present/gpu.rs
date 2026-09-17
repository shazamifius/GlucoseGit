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

use super::Presenter;
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
    /// La texture qui porte l'image, et la taille pour laquelle elle a été faite.
    texture: Option<(wgpu::Texture, wgpu::BindGroup, (u32, u32))>,
    /// Le nom de l'adaptateur retenu, pour que l'application puisse le dire.
    adaptateur: String,
    /// Le format de la texture qui porte l'image (GAMMA-1).
    format_image: wgpu::TextureFormat,
}

/// Un format de surface qui n'impose **aucune** conversion, s'il en existe un.
///
/// # Le défaut que cette fonction répare
///
/// `get_default_config` retient volontiers `Bgra8UnormSrgb`. Écrire dedans les octets de
/// `tiny-skia` — qui sont déjà du sRGB — en les ayant déclarés linéaires fait appliquer une
/// conversion linéaire → sRGB de trop, et **toute l'interface pâlit**. C'est exactement ce
/// qu'on a vu : un fond censé être presque noir rendu en gris moyen, et les lueurs délavées.
///
/// Les deux réponses possibles étaient de déclarer la texture en sRGB — le sampler
/// reconvertit alors dans l'autre sens, et les deux conversions s'annulent — ou de refuser
/// l'espace sRGB des deux côtés. La seconde est meilleure : elle ne compense pas une
/// conversion par une autre, elle n'en fait aucune. C'est aussi ce que ce module promet.
fn format_sans_conversion(proposes: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    proposes.iter().copied().find(|f| !f.is_srgb())
}

impl GpuPresenter {
    /// Ouvre une présentation graphique sur cette fenêtre, ou dit pourquoi elle ne peut pas.
    ///
    /// Aucune de ces erreurs n'est anormale : une machine virtuelle, un bureau distant ou un
    /// pilote absent sont des cas de tous les jours. L'appelant retombe alors sur le chemin
    /// processeur, et le dit.
    pub fn new(window: Arc<Window>, width: NonZeroU32, height: NonZeroU32) -> DesktopResult<Self> {
        Self::avec_cadence(window, width, height, cadence_demandee())
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
    ) -> DesktopResult<Self> {
        let echec = |quoi: &str, e: &dyn std::fmt::Display| {
            DesktopError::WindowError(format!("présentation graphique — {quoi} : {e}"))
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .map_err(|e| echec("surface", &e))?;

        let (adapter, device, queue, adaptateur) = ouvrir(&instance, &surface, width, height)?;

        let config = surface
            .get_default_config(&adapter, width.get(), height.get())
            .ok_or_else(|| {
                DesktopError::WindowError(
                    "présentation graphique : la surface n'accepte aucun format".into(),
                )
            })?;
        let mut config = config;
        config.format = format_sans_conversion(&surface.get_capabilities(&adapter).formats)
            .unwrap_or(config.format);
        let config = cadencer(&surface, &adapter, config, cadence);
        let (pipeline, layout, sampler) = atelier(&device, config.format);
        // La texture porte les octets tels quels quand la surface est linéaire. Si aucun
        // format non-sRGB n'était disponible, elle se déclare sRGB pour que le sampler
        // défasse ce que la sortie refera — les deux conversions s'annulent alors.
        let format_image = if config.format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            sampler,
            layout,
            texture: None,
            adaptateur,
            format_image,
        })
    }

    /// Le nom de la carte retenue.
    pub fn adaptateur(&self) -> &str {
        &self.adaptateur
    }

    /// Comment les images se succèdent — ce qui décide si présenter attend ou non.
    pub fn cadence(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    /// Téléverse l'image dans la texture, sans rien convertir.
    ///
    /// C'est la moitié du travail que la carte graphique fait à la place du processeur, et la
    /// seule qui lui coûte vraiment : mesurée à 0,79 ms sur 2560 × 1600, contre 3,00 ms pour
    /// convertir puis recopier du côté processeur.
    fn televerser(&mut self, pixmap: &Pixmap) {
        let (w, h) = (pixmap.width(), pixmap.height());
        let (texture, _, _) = self.texture_pour(w, h);
        let texture = texture.clone();

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
        crate::perf::stage("blit");
    }

    /// La texture à cette taille, refaite seulement si la taille a changé.
    fn texture_pour(&mut self, w: u32, h: u32) -> &(wgpu::Texture, wgpu::BindGroup, (u32, u32)) {
        if self.texture.as_ref().map(|(_, _, t)| *t) != Some((w, h)) {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("glucose-image"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format_image,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("glucose-image"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.texture = Some((texture, bind, (w, h)));
        }
        self.texture
            .as_ref()
            .expect("la texture vient d'être faite")
    }
}

/// Choisit un adaptateur, ouvre un périphérique, et refuse proprement ce qu'il ne peut pas.
///
/// Le garde-fou de taille n'est pas décoratif : demander une surface plus grande que la
/// texture maximale de l'adaptateur **fait paniquer** la couche graphique au fond de la pile,
/// et la première version du module le faisait dès qu'un écran dépassait le 1080p.
fn ouvrir(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'static>,
    width: NonZeroU32,
    height: NonZeroU32,
) -> DesktopResult<(wgpu::Adapter, wgpu::Device, wgpu::Queue, String)> {
    let echec = |quoi: &str, e: &dyn std::fmt::Display| {
        DesktopError::WindowError(format!("présentation graphique — {quoi} : {e}"))
    };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: Some(surface),
        ..Default::default()
    }))
    .map_err(|e| echec("adaptateur", &e))?;
    let adaptateur = adapter.get_info().name;

    // Les limites de l'adaptateur, et non les limites « de base » : celles-ci plafonnent
    // les textures à 2048 pixels, ce qui refuse d'emblée tout écran au-delà du 1080p.
    // C'est la faute qui a fait paniquer la première version sur une fenêtre de 2160 de
    // large — une garantie de portabilité transformée en refus de fonctionner.
    let limites = adapter.limits();
    let plafond = limites.max_texture_dimension_2d;
    if width.get() > plafond || height.get() > plafond {
        return Err(DesktopError::WindowError(format!(
            "présentation graphique : une fenêtre de {}×{} dépasse la texture maximale de                  cet adaptateur ({plafond})",
            width.get(),
            height.get()
        )));
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("glucose"),
        required_features: wgpu::Features::empty(),
        required_limits: limites,
        memory_hints: wgpu::MemoryHints::Performance,
        ..Default::default()
    }))
    .map_err(|e| echec("périphérique", &e))?;

    // Sans cela, la moindre erreur de validation tue l'application par un `panic!` au
    // fond de la pile graphique. Une erreur de pilote n'est pas un bogue de Glucose : elle
    // se dit, et l'image suivante réessaie.
    device.on_uncaptured_error(Arc::new(|e| {
        eprintln!("[Glucose] la couche graphique a refusé une commande : {e}");
    }));

    Ok((adapter, device, queue, adaptateur))
}

/// La façon de présenter demandée par l'environnement, s'il en demande une.
///
/// # Pourquoi ce réglage existe
///
/// La chronique a montré que `present` coûte 28 ms sur un canevas **vide**, soit 68 % de
/// l'image. Or `get_current_texture` **bloque** en mode `Fifo` : il attend que l'écran ait
/// fini de balayer. Une durée seule ne distingue donc pas un travail lent d'une attente, et
/// c'est exactement l'ambiguïté qui a déjà fait chercher au mauvais endroit cette semaine.
///
/// `GLUCOSE_PRESENT=immediate` supprime l'attente : ce qui reste est le travail réel. La
/// comparaison des deux tranche la question au lieu de la raisonner.
///
/// * `immediate` — aucune attente, l'image part tout de suite (déchirure possible) ;
/// * `mailbox` — sans attente ni déchirure, quand la carte le propose ;
/// * `fifo` — le défaut : l'image attend le balayage.
fn cadence_demandee() -> Option<wgpu::PresentMode> {
    match std::env::var("GLUCOSE_PRESENT").ok()?.trim().to_lowercase().as_str() {
        "immediate" => Some(wgpu::PresentMode::Immediate),
        "mailbox" => Some(wgpu::PresentMode::Mailbox),
        "fifo" => Some(wgpu::PresentMode::Fifo),
        autre => {
            eprintln!("[Glucose] GLUCOSE_PRESENT={autre} inconnu (immediate, mailbox, fifo)");
            None
        }
    }
}

/// Impose la cadence demandée si la surface l'accepte, et le dit sinon.
///
/// C'est ce réglage qui décide si présenter **attend** le balayage de l'écran. `Fifo`, le
/// défaut, est ce qu'on veut à l'usage : pas de déchirure, et le fil dort au lieu de produire
/// des images que personne ne verra. `Immediate` ne sert qu'à mesurer le travail seul — sans
/// lui, le banc chiffrait la période de l'écran et concluait de travers.
fn cadencer(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    mut config: wgpu::SurfaceConfiguration,
    cadence: Option<wgpu::PresentMode>,
) -> wgpu::SurfaceConfiguration {
    if let Some(voulue) = cadence {
        let possibles = surface.get_capabilities(adapter).present_modes;
        if possibles.contains(&voulue) {
            config.present_mode = voulue;
        } else {
            eprintln!(
                "[Glucose] cadence {voulue:?} indisponible, on garde {:?}",
                config.present_mode
            );
        }
    }
    config
}

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
        if self.config.width == width.get() && self.config.height == height.get() {
            return Ok(());
        }
        self.config.width = width.get();
        self.config.height = height.get();
        self.surface.configure(&self.device, &self.config);
        Ok(())
    }

    fn present(&mut self, pixmap: &Pixmap) -> DesktopResult<()> {
        self.televerser(pixmap);

        // Les trois temps de la présentation, séparés parce qu'ils n'ont pas la même nature :
        // **acquerir** peut attendre que l'écran rende une image du carrousel, **encoder** est
        // du travail de processeur, **soumettre** confie le tout à la carte. Mesurés ensemble,
        // ils annonçaient 28 ms sur un canevas vide sans dire lequel les portait.
        use wgpu::CurrentSurfaceTexture as Etat;
        let frame = match self.surface.get_current_texture() {
            Etat::Success(frame) => frame,
            // La surface tient encore, mais elle ne correspond plus tout à fait à la fenêtre.
            // On affiche quand même — sauter une image se verrait — et on la réaccorde pour
            // la suivante.
            Etat::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.config);
                frame
            }
            // Rien à afficher, et ce n'est pas une panne : la fenêtre est réduite, cachée
            // derrière une autre, ou le compositeur a mis du temps à rendre la main. Dessiner
            // dans le vide serait du travail perdu, et le signaler comme une erreur ferait
            // crier l'application à chaque fois qu'on la minimise.
            Etat::Timeout | Etat::Occluded => return Ok(()),
            // La surface a vieilli ou s'est perdue : on la réaccorde, et l'image suivante —
            // dans quelques millisecondes — repartira sur des bases saines.
            autre => {
                self.surface.configure(&self.device, &self.config);
                return Err(DesktopError::WindowError(format!(
                    "la surface graphique doit être refaite : {autre:?}"
                )));
            }
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
            let bind = &self
                .texture
                .as_ref()
                .expect("la texture existe à ce point")
                .1;
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
        self.queue.submit(Some(encodeur.finish()));
        // La vue sur l'image de la surface doit être relâchée avant de la rendre au
        // compositeur : elle l'emprunte.
        drop(cible);
        self.queue.present(frame);
        crate::perf::stage("present");
        Ok(())
    }

    fn nom(&self) -> &'static str {
        "carte graphique"
    }
}
