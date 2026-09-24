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
