// Le champ d'une fleche, evalue en chaque pixel : la loi de `glucose_core::arrow::champ`,
// recopiee fonction pour fonction, en f32 et dans le meme ordre.
//
// Un quad par segment, un par disque. Chaque pixel n'est ecrit que par UN quad de sa fleche :
// celui du segment le plus proche s'il est a portee, sinon celui du premier disque qui le
// couvre. Un coude, couvert par les quads de ses deux segments, n'est donc compose qu'une fois.

struct Disque {
    // centre x, y, rayon, demi-epaisseur du contour
    geometrie: vec4<f32>,
    // fond rgb, present (1) ou absent (0)
    fond: vec4<f32>,
    // contour rgb, portee du disque
    contour: vec4<f32>,
};

struct Fleche {
    axe: vec4<f32>,
    t0: vec4<f32>,
    t1: vec4<f32>,
    t2: vec4<f32>,
    // demi-largeur et opacite du halo, demi-largeur et opacite du trait
    trait_: vec4<f32>,
    // trait blanc (1) ou non (0), portee d'un segment
    reglage: vec4<f32>,
    // premier segment, combien de segments
    plage: vec4<u32>,
    disques: array<Disque, 2>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> fleches: array<Fleche>;
@group(0) @binding(1) var<storage, read> segments: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> instances: array<vec2<u32>>;
@group(0) @binding(3) var<uniform> ecran: Ecran;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) fleche: u32,
    @location(1) @interpolate(flat) morceau: u32,
};

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    var coins = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let inst = instances[i];
    let c = coins[v];
    let combien = fleches[inst.x].plage.y;
    var p: vec2<f32>;
    if inst.y < combien {
        // Le segment, allonge et epaissi de la portee : il couvre tout pixel a portee.
        let s = segments[fleches[inst.x].plage.x + inst.y];
        let d = s.zw - s.xy;
        let l = length(d);
        var u = vec2<f32>(1.0, 0.0);
        if l > 0.0 {
            u = d / l;
        }
        let n = vec2<f32>(-u.y, u.x);
        let r = fleches[inst.x].reglage.y;
        p = mix(s.xy - u * r, s.zw + u * r, c.x * 0.5 + 0.5) + n * (r * c.y);
    } else {
        let g = fleches[inst.x].disques[inst.y - combien];
        p = g.geometrie.xy + c * g.contour.w;
    }
    var s: Sortie;
    s.position = vec4<f32>(
        p.x / ecran.taille.x * 2.0 - 1.0,
        1.0 - p.y / ecran.taille.y * 2.0,
        0.0,
        1.0,
    );
    s.fleche = inst.x;
    s.morceau = inst.y;
    return s;
}

// `distance_au_segment`.
fn distance_au_segment(s: vec4<f32>, x: f32, y: f32) -> f32 {
    let dx = s.z - s.x;
    let dy = s.w - s.y;
    let px = x - s.x;
    let py = y - s.y;
    let l2 = dx * dx + dy * dy;
    var t = 0.0;
    if l2 > 0.0 {
        t = clamp((px * dx + py * dy) / l2, 0.0, 1.0);
    }
    let qx = px - dx * t;
    let qy = py - dy * t;
    return sqrt(qx * qx + qy * qy);
}

// `membrane_forme::couverture_d_un_plein` et `couverture_d_un_trait`.
fn couverture_d_un_plein(d: f32) -> f32 {
    return clamp(0.5 - d, 0.0, 1.0);
}

fn couverture_d_un_trait(d: f32, h: f32) -> f32 {
    return max(min(d + 0.5, h) - max(d - 0.5, -h), 0.0);
}

// `Champ::teinte`.
fn teinte(i: u32, x: f32, y: f32) -> vec3<f32> {
    let a = fleches[i].axe;
    let dx = a.z - a.x;
    let dy = a.w - a.y;
    let l2 = dx * dx + dy * dy;
    var t = 0.5;
    if l2 > 0.0 {
        t = clamp(((x - a.x) * dx + (y - a.y) * dy) / l2, 0.0, 1.0);
    }
    var c0 = fleches[i].t0.xyz;
    var c1 = fleches[i].t1.xyz;
    var k = t * 2.0;
    if t >= 0.5 {
        c0 = fleches[i].t1.xyz;
        c1 = fleches[i].t2.xyz;
        k = t * 2.0 - 1.0;
    }
    return c0 + (c1 - c0) * k;
}

// `poser` : une couleur c d'opacite a sur un premultiplie.
fn poser(o: vec4<f32>, c: vec3<f32>, a: f32) -> vec4<f32> {
    return vec4<f32>(c * a + o.xyz * (1.0 - a), a + o.w * (1.0 - a));
}

fn poser_le_disque(o: vec4<f32>, g: Disque, x: f32, y: f32) -> vec4<f32> {
    if g.fond.w < 0.5 {
        return o;
    }
    let vx = x - g.geometrie.x;
    let vy = y - g.geometrie.y;
    let r = sqrt(vx * vx + vy * vy) - g.geometrie.z;
    let avec_fond = poser(o, g.fond.xyz, couverture_d_un_plein(r));
    return poser(avec_fond, g.contour.xyz, couverture_d_un_trait(abs(r), g.geometrie.w));
}

// Le point est-il strictement dans la portee de ce disque, s'il existe ?
fn dans_le_disque(g: Disque, x: f32, y: f32) -> bool {
    let vx = x - g.geometrie.x;
    let vy = y - g.geometrie.y;
    return g.fond.w > 0.5 && sqrt(vx * vx + vy * vy) < g.contour.w;
}

// `Champ::couleur_parmi`, la distance au trace deja mesuree.
fn couleur(i: u32, x: f32, y: f32, d: f32) -> vec4<f32> {
    let c = teinte(i, x, y);
    let tr = fleches[i].trait_;
    var o = poser(vec4<f32>(0.0), c, tr.y * couverture_d_un_trait(d, tr.x));
    var ame = c;
    if fleches[i].reglage.x > 0.5 {
        ame = vec3<f32>(1.0);
    }
    o = poser(o, ame, tr.w * couverture_d_un_trait(d, tr.z));
    o = poser_le_disque(o, fleches[i].disques[0], x, y);
    o = poser_le_disque(o, fleches[i].disques[1], x, y);
    return o;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    let x = e.position.x;
    let y = e.position.y;
    let premier = fleches[e.fleche].plage.x;
    let combien = fleches[e.fleche].plage.y;
    var dmin = 1.0e30;
    var plus_proche = 0xffffffffu;
    for (var k = 0u; k < combien; k = k + 1u) {
        let d = distance_au_segment(segments[premier + k], x, y);
        if d < dmin {
            dmin = d;
            plus_proche = k;
        }
    }
    // La propriete se decide par des distances STRICTES : un point a moins de la portee d'un
    // segment est strictement dans son quad, et un point strictement dans le cercle d'un
    // disque strictement dans son carre -- tous deux sont donc toujours rasterises. Un test
    // large laisserait un pixel pose sur un bord a un quad que la carte ne dessine pas.
    let portee = fleches[e.fleche].reglage.y;
    if e.morceau < combien {
        // Un segment n'ecrit que les pixels dont il est le plus proche, a portee.
        if plus_proche != e.morceau || !(dmin < portee) {
            discard;
        }
    } else {
        // Un disque n'ecrit que ce qu'aucun segment ne possede, dans son cercle ; le second
        // laisse au premier ce qui est dans le cercle du premier.
        if dmin < portee || !dans_le_disque(fleches[e.fleche].disques[e.morceau - combien], x, y) {
            discard;
        }
        if e.morceau == combien + 1u && dans_le_disque(fleches[e.fleche].disques[0], x, y) {
            discard;
        }
    }
    return couleur(e.fleche, x, y, dmin);
}
