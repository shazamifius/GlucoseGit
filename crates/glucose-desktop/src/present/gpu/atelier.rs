//! **Le montage de la présentation** : le nuanceur qui pose l'image finie, sa disposition,
//! son pipeline et son échantillonneur.
//!
//! # Pourquoi ce fichier existe à part
//!
//! Tout cela se construit une fois et ne dépend que du format de la surface : c'est du
//! montage, pas de la présentation. Le garder dans [`super`] lui faisait passer les six cents
//! lignes que la fiche 05 admet quand la forme des membranes est descendue sur la carte
//! (MEMB-FORME-1), et le cliquet a eu raison de le dire.

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

/// Le nuanceur, la disposition des liaisons, le pipeline et l'échantillonneur.
///
/// Tout cela se construit une fois et ne dépend que du format de la surface — c'est du
/// montage, pas de la présentation, et le garder dans l'ouverture y mélangeait deux sujets.
pub(super) fn atelier(
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
