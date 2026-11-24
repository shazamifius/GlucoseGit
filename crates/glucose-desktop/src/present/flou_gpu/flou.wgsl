// Le flou du reste, autour de ce qu'on a choisi (Q12, fiche 137) : la transcription de la loi du
// noyau (`glucose_core::flou`). Chaque lecture tombe aux poids du processeur : une lecture
// bilineaire a un quart de texel rend exactement 1/4 et 3/4, les bords prolonges par
// l'echantillonneur (`ClampToEdge`) comme par `clamp` au processeur.

struct Reglage {
    // .xy : la taille de la source, en texels ; .zw : le pas de la gaussienne, (1, 0) ou (0, 1).
    source: vec4<f32>,
    // Les poids de la gaussienne, du centre vers un bord : poids[d / 4][d % 4].
    poids: array<vec4<f32>, 3>,
    // .x : le rayon de la gaussienne ; .y : de combien la source est plus petite que l'image
    // (1 ou 2) ; .z : la maille du masque.
    divers: vec4<f32>,
};

@group(0) @binding(0) var<uniform> r: Reglage;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var lineaire: sampler;
@group(0) @binding(3) var nette: texture_2d<f32>;
@group(0) @binding(4) var masque: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) v: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

// Reduire de moitie : le filtre [1, 3, 3, 1] / 8 sur chaque axe, en quatre lectures bilineaires a
// trois quarts de texel du centre de la maille -- chacune rend (a + 3 b) / 4.
@fragment
fn fs_reduire(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let centre = p.xy * 2.0;
    var somme = vec4<f32>(0.0);
    for (var j = 0; j < 4; j = j + 1) {
        let d = vec2<f32>(select(-0.75, 0.75, (j & 1) == 1), select(-0.75, 0.75, (j & 2) == 2));
        somme = somme + textureSampleLevel(source, lineaire, (centre + d) / r.source.xy, 0.0);
    }
    return somme * 0.25;
}

fn poids(d: i32) -> f32 {
    return r.poids[d / 4][d % 4];
}

// La gaussienne, sur un axe, par lectures exactes ; les bords prolonges.
@fragment
fn fs_gauss(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let taille = vec2<i32>(r.source.xy);
    let pas = vec2<i32>(r.source.zw);
    let q = vec2<i32>(p.xy);
    var somme = poids(0) * textureLoad(source, q, 0);
    for (var d = 1; d <= i32(r.divers.x); d = d + 1) {
        let avant = clamp(q - d * pas, vec2<i32>(0), taille - 1);
        let apres = clamp(q + d * pas, vec2<i32>(0), taille - 1);
        somme = somme + poids(d) * (textureLoad(source, avant, 0) + textureLoad(source, apres, 0));
    }
    return somme;
}

// Agrandir au double : la grille grossiere, lue sous le centre de chaque pixel fin.
@fragment
fn fs_agrandir(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSampleLevel(source, lineaire, p.xy * 0.5 / r.source.xy, 0.0);
}

// Meler : net x m + flou x (1 - m), le flou lu dans la grille du premier etage, le masque dans
// la plus grossiere -- l'image entiere : la chrome, posee ensuite, couvre la bande.
@fragment
fn fs_meler(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let net = textureLoad(nette, vec2<i32>(p.xy), 0);
    let flou = textureSampleLevel(source, lineaire, p.xy / r.divers.y / r.source.xy, 0.0);
    let grille = vec2<f32>(textureDimensions(masque));
    let m = clamp(textureSampleLevel(masque, lineaire, p.xy / r.divers.z / grille, 0.0).r, 0.0, 1.0);
    return net * m + flou * (1.0 - m);
}
