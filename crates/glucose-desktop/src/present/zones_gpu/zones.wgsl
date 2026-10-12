// Le corps des zones d'un gabarit : des rectangles unis, deja sur la grille de pixels, chacun
// avec sa couleur premultipliee. Le socle les a dresses ; la carte ne fait que les poser.

struct Rectangle {
    // Gauche, haut, droite, bas, en pixels d'ecran.
    boite: vec4<f32>,
    // La couleur, premultipliee.
    couleur: vec4<f32>,
};

struct Ecran { taille: vec2<f32>, _r: vec2<f32> };

@group(0) @binding(0) var<storage, read> rectangles: array<Rectangle>;
@group(0) @binding(1) var<uniform> ecran: Ecran;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) couleur: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Sortie {
    let coins = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let b = rectangles[i].boite;
    let p = mix(b.xy, b.zw, coins[v]);
    var s: Sortie;
    s.position = vec4<f32>(
        p.x / ecran.taille.x * 2.0 - 1.0,
        1.0 - p.y / ecran.taille.y * 2.0,
        0.0,
        1.0,
    );
    s.couleur = rectangles[i].couleur;
    return s;
}

// Un rectangle sur la grille couvre ses pixels entiers : le centre d'un pixel y est, ou n'y
// est pas — aucune couverture partielle, comme le remplissage du processeur.
@fragment
fn fs(s: Sortie) -> @location(0) vec4<f32> {
    return s.couleur;
}
