// La forme d'une membrane, evaluee en chaque pixel : la loi de `glucose_core::membrane_forme`,
// recopiee fonction pour fonction, en f32 et dans le meme ordre.

struct Membrane {
    // Les trois remplissages (deux halos, le fond) puis le bord : gauche, haut, droite, bas.
    boites: array<vec4<f32>, 4>,
    // Leurs rayons, dans le meme ordre.
    rayons: vec4<f32>,
    // Leurs opacites ; celle du bord en dernier.
    alphas: vec4<f32>,
    // La demi-largeur du trait, la longueur d'un tiret (0 : trait plein), et une reserve.
    trait_: vec4<f32>,
    // La phase du pointille au debut de chacun des huit morceaux du bord.
    phases: array<vec4<f32>, 2>,
    // Ce qui reste du perimetre depuis le debut de chaque morceau.
    restes: array<vec4<f32>, 2>,
    teinte: vec4<f32>,
    // Ce que la membrane peut toucher : le quad qu'on dessine.
    enveloppe: vec4<f32>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> membranes: array<Membrane>;
@group(0) @binding(1) var<uniform> ecran: Ecran;

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
    let e = membranes[i].enveloppe;
    let p = mix(e.xy, e.zw, coins[v]);
    var s: Sortie;
    s.position = vec4<f32>(
        p.x / ecran.taille.x * 2.0 - 1.0,
        1.0 - p.y / ecran.taille.y * 2.0,
        0.0,
        1.0,
    );
    s.rang = i;
    return s;
}

// `Arrondi::distance` : depuis les BORDS, pour rester juste quand le centre est tres loin.
fn distance(b: vec4<f32>, r: f32, p: vec2<f32>) -> f32 {
    let qx = max(b.x - p.x, p.x - b.z) + r;
    let qy = max(b.y - p.y, p.y - b.w) + r;
    return length(max(vec2<f32>(qx, qy), vec2<f32>(0.0))) + min(max(qx, qy), 0.0) - r;
}

// `Arrondi::sur_le_bord` : le morceau du bord le plus proche, et l'abscisse depuis son debut.
fn sur_le_bord(b: vec4<f32>, r: f32, p: vec2<f32>) -> vec2<f32> {
    let a_gauche = b.x - p.x > p.x - b.z;
    let en_haut = b.y - p.y > p.y - b.w;
    let qx = max(b.x - p.x, p.x - b.z) + r;
    let qy = max(b.y - p.y, p.y - b.w) + r;
    if qx > 0.0 && qy > 0.0 {
        let c = vec2<f32>(select(b.z - r, b.x + r, a_gauche), select(b.w - r, b.y + r, en_haut));
        let v = p - c;
        var morceau = 7.0;
        var angle = atan2(-v.y, -v.x);
        if !a_gauche && en_haut {
            morceau = 1.0;
            angle = atan2(v.x, -v.y);
        } else if !a_gauche && !en_haut {
            morceau = 3.0;
            angle = atan2(v.y, v.x);
        } else if a_gauche && !en_haut {
            morceau = 5.0;
            angle = atan2(-v.x, v.y);
        }
        return vec2<f32>(morceau, r * clamp(angle, 0.0, 1.57079632679));
    }
    if qx > qy {
        if a_gauche {
            return vec2<f32>(6.0, (b.w - r) - p.y);
        }
        return vec2<f32>(2.0, p.y - (b.y + r));
    }
    if en_haut {
        return vec2<f32>(0.0, p.x - (b.x + r));
    }
    return vec2<f32>(4.0, (b.z - r) - p.x);
}

// `Pointille::ecart` : la distance, le long du bord, jusqu'au tiret le plus proche.
//
// Tout se lit dans le tampon, par le rang de la membrane : indexer un tableau porte par une
// VALEUR avec un indice calcule n'est pas garanti sur toutes les couches graphiques.
fn ecart(i: u32, morceau: u32, abscisse: f32) -> f32 {
    let tiret = membranes[i].trait_.y;
    let periode = 2.0 * tiret;
    let phase = membranes[i].phases[morceau / 4u][morceau % 4u];
    let reste = membranes[i].restes[morceau / 4u][morceau % 4u];
    // `rem_euclid` : le reste tronque, ramene dans [0, periode).
    var t = (phase + abscisse) % periode;
    if t < 0.0 {
        t = t + periode;
    }
    if t < tiret {
        return 0.0;
    }
    return min(min(t - tiret, periode - t), max(reste - abscisse, 0.0));
}

fn couverture_d_un_plein(d: f32) -> f32 {
    return clamp(0.5 - d, 0.0, 1.0);
}

fn couverture_d_un_trait(d: f32, h: f32) -> f32 {
    return max(min(d + 0.5, h) - max(d - 0.5, -h), 0.0);
}

// `Membrane::alpha` : le produit des transparences de ses couches, dans l'ordre.
fn alpha(i: u32, p: vec2<f32>) -> f32 {
    var transparence = 1.0;
    for (var k = 0u; k < 3u; k = k + 1u) {
        let a = membranes[i].alphas[k];
        if a > 0.0 {
            let d = distance(membranes[i].boites[k], membranes[i].rayons[k], p);
            transparence = transparence * (1.0 - a * couverture_d_un_plein(d));
        }
    }
    let bord = membranes[i].boites[3];
    let rayon = membranes[i].rayons[3];
    let d = distance(bord, rayon, p);
    var e = 0.0;
    if membranes[i].trait_.y > 0.0 {
        let lieu = sur_le_bord(bord, rayon, p);
        e = ecart(i, u32(lieu.x), lieu.y);
    }
    let couverture = couverture_d_un_trait(length(vec2<f32>(d, e)), membranes[i].trait_.x);
    transparence = transparence * (1.0 - membranes[i].alphas[3] * couverture);
    return 1.0 - transparence;
}

@fragment
fn fs(e: Sortie) -> @location(0) vec4<f32> {
    let a = alpha(e.rang, e.position.xy);
    // Premultiplie : la teinte multipliee par son opacite, comme le processeur la compose.
    return vec4<f32>(membranes[e.rang].teinte.rgb * a, a);
}
