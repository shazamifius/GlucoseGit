// Les formes d'un noeud dessine de pres (fiche 73) : des rectangles arrondis par la loi des
// membranes (`arrondi.wgsl`, pose devant ce texte), et des courbes par la loi de Slug
// (`glucose_core::courbes::couverture`, recopiee fonction pour fonction, en f32 et dans le
// meme ordre — elle-meme celle du nuanceur de reference d'Eric Lengyel, licence MIT).

struct Primitive {
    // Un arrondi : gauche, haut, droite, bas, a l'ecran. Des courbes : la matrice (a, b, c, d)
    // qui porte la forme a l'ecran — x' = a x + c y + e, y' = b x + d y + f.
    m: vec4<f32>,
    // Un arrondi : son rayon, la demi-largeur de son trait (0 : un plein). Des courbes : (e, f).
    t: vec4<f32>,
    // La boite de la forme : gauche, bas, droite, haut, dans son unite.
    boite: vec4<f32>,
    // La transformation des bandes : echelle en x et en y, decalage en x et en y.
    bandes: vec4<f32>,
    // La couleur, premultipliee.
    couleur: vec4<f32>,
    // Ce qui rogne la primitive : gauche, haut, droite, bas, a l'ecran.
    rogne: vec4<f32>,
    // Le genre (0 : un arrondi, 1 : des courbes), ou commencent ses courbes et ses bandes.
    n: vec4<u32>,
    // Combien de bandes horizontales, puis verticales.
    nb: vec4<u32>,
    // Des courbes : l'inverse de (a, b, c, d), calcule par le processeur — la carte ne divise pas.
    inverse: vec4<f32>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> primitives: array<Primitive>;
@group(0) @binding(1) var<uniform> ecran: Ecran;
@group(0) @binding(2) var<storage, read> courbes: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> bandes: array<u32>;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) rang: u32,
};

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    let coins = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let p = primitives[i];
    var s: Sortie;
    s.rang = i;
    var ecrit: vec2<f32>;
    if p.n.x == 0u {
        // L'enveloppe d'un arrondi : son bord repousse du trait et d'un demi-pixel, comme
        // `Membrane::enveloppe`.
        let e = p.t.y + 0.5;
        ecrit = mix(p.m.xy - vec2<f32>(e), p.m.zw + vec2<f32>(e), coins[v]);
    } else {
        // La boite de la forme, elargie d'un demi-pixel a l'ecran le long de chacun de ses axes :
        // les pixels que son bord couvre en partie ont leur centre dedans (la dilatation de
        // Slug, pour une transformation sans perspective).
        let d = vec2<f32>(0.5) / pixels_par_unite(p);
        var q = mix(p.boite.xy - d, p.boite.zw + d, coins[v]);
        // Portee sans rotation, la forme se borne a ce qui la rogne, ramene dans son unite : un
        // radical de 400 000 unites ne fait travailler que les pixels de sa boite.
        if p.m.y == 0.0 && p.m.z == 0.0 {
            let a = (p.rogne.xy - vec2<f32>(0.5) - p.t.xy) / vec2<f32>(p.m.x, p.m.w);
            let b = (p.rogne.zw + vec2<f32>(0.5) - p.t.xy) / vec2<f32>(p.m.x, p.m.w);
            q = clamp(q, min(a, b), max(a, b));
        }
        ecrit = vec2<f32>(p.m.x * q.x + p.m.z * q.y + p.t.x, p.m.y * q.x + p.m.w * q.y + p.t.y);
    }
    s.position = vec4<f32>(
        ecrit.x / ecran.taille.x * 2.0 - 1.0,
        1.0 - ecrit.y / ecran.taille.y * 2.0,
        0.0,
        1.0,
    );
    return s;
}

// `code_des_racines` : quelles racines comptent, lu sur les signes des trois ordonnees.
fn code_des_racines(y1: f32, y2: f32, y3: f32) -> u32 {
    let i1 = bitcast<u32>(y1) >> 31u;
    let i2 = bitcast<u32>(y2) >> 30u;
    let i3 = bitcast<u32>(y3) >> 29u;
    var decalage = (i2 & 2u) | (i1 & ~2u);
    decalage = (i3 & 4u) | (decalage & ~4u);
    return (0x2E74u >> decalage) & 0x0101u;
}

// `racines`, l'axe donne : ou la courbe croise `y = 0` (le rayon horizontal) ou `x = 0`.
fn racines(p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, horizontal: bool) -> vec2<f32> {
    // Ramene au cas horizontal : la coordonnee resolue en y, celle qu'on rend en x.
    var q1 = p1;
    var q2 = p2;
    var q3 = p3;
    if !horizontal {
        q1 = p1.yx;
        q2 = p2.yx;
        q3 = p3.yx;
    }
    let a = q1 - q2 * 2.0 + q3;
    let b = q1 - q2;
    let ra = 1.0 / a.y;
    let rb = 0.5 / b.y;
    let d = sqrt(max(b.y * b.y - a.y * q1.y, 0.0));
    var t1 = (b.y - d) * ra;
    var t2 = (b.y + d) * ra;
    if abs(a.y) < 1.0 / 65536.0 {
        t1 = q1.y * rb;
        t2 = t1;
    }
    return vec2<f32>((a.x * t1 - b.x * 2.0) * t1 + q1.x, (a.x * t2 - b.x * 2.0) * t2 + q1.x);
}

// `bandes_de` : la bande d'une coordonnee, bornee a ses bandes.
fn bande_de(v: f32, echelle: f32, decalage: f32, derniere: u32) -> u32 {
    return u32(clamp(i32(v * echelle + decalage), 0, i32(derniere)));
}

// `le_long_d_un_rayon` : la somme et le poids, le long du rayon horizontal ou vertical.
fn le_long_d_un_rayon(i: u32, bande: u32, point: vec2<f32>, ppu: vec2<f32>, horizontal: bool) -> vec2<f32> {
    let en_tete = primitives[i].n.z + 2u * bande;
    let combien = bandes[en_tete];
    let ou = primitives[i].n.z + bandes[en_tete + 1u];
    let base = primitives[i].n.y;
    var somme = 0.0;
    var poids = 0.0;
    var sens = 1.0;
    var mesure = ppu.x;
    if !horizontal {
        sens = -1.0;
        mesure = ppu.y;
    }
    for (var k = 0u; k < combien; k = k + 1u) {
        let r = base + bandes[ou + k];
        let a = courbes[r];
        let s = courbes[r + 1u];
        let p1 = a.xy - point;
        let p2 = a.zw - point;
        let p3 = s.xy - point;
        var loin = max(max(p1.x, p2.x), p3.x);
        var code = code_des_racines(p1.y, p2.y, p3.y);
        if !horizontal {
            loin = max(max(p1.y, p2.y), p3.y);
            code = code_des_racines(p1.x, p2.x, p3.x);
        }
        if loin * mesure < -0.5 {
            break;
        }
        if code == 0u {
            continue;
        }
        let x = racines(p1, p2, p3, horizontal) * mesure;
        if (code & 1u) != 0u {
            somme = somme + sens * clamp(x.x + 0.5, 0.0, 1.0);
            poids = max(poids, clamp(1.0 - abs(x.x) * 2.0, 0.0, 1.0));
        }
        if code > 1u {
            somme = somme - sens * clamp(x.y + 0.5, 0.0, 1.0);
            poids = max(poids, clamp(1.0 - abs(x.y) * 2.0, 0.0, 1.0));
        }
    }
    return vec2<f32>(somme, poids);
}

// `couverture` : les deux rayons, meles selon leur poids ; enroulements non nuls.
fn couverture(i: u32, point: vec2<f32>, ppu: vec2<f32>) -> f32 {
    let t = primitives[i].bandes;
    let nb = primitives[i].nb;
    let horizontale = bande_de(point.y, t.y, t.w, nb.x - 1u);
    let verticale = bande_de(point.x, t.x, t.z, nb.y - 1u);
    let x = le_long_d_un_rayon(i, horizontale, point, ppu, true);
    let y = le_long_d_un_rayon(i, nb.x + verticale, point, ppu, false);
    let melee = abs(x.x * x.y + y.x * y.y) / max(x.y + y.y, 1.0 / 65536.0);
    return clamp(max(melee, min(abs(x.x), abs(y.x))), 0.0, 1.0);
}

// **Le point de la forme sous le centre de ce pixel** : l'inverse de la matrice, applique au
// centre exact. L'interpoler depuis les coins du quad le faisait trembler de pres — sur une
// lettre de 2 400 pixels, la ou un rayon frole la courbe, le croisement bougeait d'un tiers de
// pixel (dix-huit niveaux, fiche 73). L'inverse vient du processeur (RACINES-1) : diviser ici,
// a deux ulps et demi pres, faisait basculer le signe d'un point pose pile sur un trait.
fn point_de_la_forme(p: Primitive, centre: vec2<f32>) -> vec2<f32> {
    let i = p.inverse;
    let x = centre.x - p.t.x;
    let y = centre.y - p.t.y;
    return vec2<f32>(i.x * x + i.z * y, i.y * x + i.w * y);
}

// Combien de pixels vaut une unite de la forme, le long de chacun de ses axes.
fn pixels_par_unite(p: Primitive) -> vec2<f32> {
    return vec2<f32>(length(p.m.xy), length(p.m.zw));
}

// La part du pixel que le rectangle qui rogne laisse voir : le filtre-boite, sur chaque axe.
fn dans_la_rogne(r: vec4<f32>, p: vec2<f32>) -> f32 {
    let x = clamp(min(p.x + 0.5, r.z) - max(p.x - 0.5, r.x), 0.0, 1.0);
    let y = clamp(min(p.y + 0.5, r.w) - max(p.y - 0.5, r.y), 0.0, 1.0);
    return x * y;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    let p = primitives[e.rang];
    var c = 0.0;
    if p.n.x == 0u {
        let d = distance(p.m, p.t.x, e.position.xy);
        if p.t.y > 0.0 {
            c = couverture_d_un_trait(abs(d), p.t.y);
        } else {
            c = couverture_d_un_plein(d);
        }
    } else {
        c = couverture(e.rang, point_de_la_forme(p, e.position.xy), pixels_par_unite(p));
    }
    return p.couleur * (c * dans_la_rogne(p.rogne, e.position.xy));
}
