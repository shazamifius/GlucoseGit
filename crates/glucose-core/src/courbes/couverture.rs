//! **La loi de Slug** (P8, fiche 73) : la part d'un pixel qu'une forme couvre, lue sur ses
//! courbes et ses bandes — celle du nuanceur de référence d'Eric Lengyel (`SlugPixelShader.hlsl`,
//! licence MIT), écrite en Rust, en `f32`, opération pour opération.
//!
//! Le nuanceur de Glucose (`present/formes_gpu/formes.wgsl`) la recopie fonction pour fonction :
//! cette loi est la référence que l'épreuve des deux voies lui oppose.
//!
//! # Ce qu'elle calcule
//!
//! Le pixel est un carré d'un pixel de côté autour de son centre. Un rayon horizontal part du
//! centre vers la droite ; chaque courbe qu'il croise y ajoute ou retire la part du pixel qui est
//! à droite du croisement — `sature(x + ½)`, le croisement `x` compté en pixels depuis le centre,
//! le signe donné par le sens où la courbe monte ou descend. La somme est la couverture vue par
//! ce rayon, filtrée par une boîte d'un pixel le long de lui. Un rayon vertical fait de même ; les
//! deux se mêlent selon leur **poids** — la proximité du croisement le plus proche, puisqu'un
//! rayon ne voit bien que les bords qu'il traverse franchement. C'est la même famille de filtre
//! que celui des membranes (`couverture_d_un_plein`, une boîte perpendiculaire au bord).
//!
//! # L'éligibilité des racines
//!
//! Une courbe ne compte que si le rayon la traverse vraiment. Lengyel le lit sur les **signes**
//! des ordonnées de ses trois points, rapportées au pixel : huit cas, une table de seize bits
//! (`0x2E74`) dit lesquelles des deux racines comptent. Aucune tolérance : un point posé pile sur
//! le rayon tombe d'un côté par son signe, et deux courbes jointes en ce point ne le comptent
//! qu'une fois. C'est ce qui rend Slug sûr sans cas particulier.
//!
//! Les deux nombres de cette loi qui ressemblent à des choix — `1/65536` (une courbe « presque
//! droite », résolue comme une droite) et le même plancher pour la somme des poids — sont ceux de
//! Lengyel, posés pour des coordonnées de l'ordre de l'em : c'est pourquoi une lettre se range
//! **en em** (son unité divisée par les unités par em de sa police), comme chez lui.

use super::emballage::{bandes_de, Emballage};

/// Le seuil de Lengyel : en deçà, le terme du second degré est nul et la courbe une droite.
const PRESQUE_DROITE: f32 = 1.0 / 65536.0;

/// **Quelles racines comptent** pour une courbe dont les ordonnées, rapportées au pixel, sont
/// `y1`, `y2`, `y3` : le bit 0 pour la première, le bit 8 pour la seconde.
pub fn code_des_racines(y1: f32, y2: f32, y3: f32) -> u32 {
    let i1 = y1.to_bits() >> 31;
    let i2 = y2.to_bits() >> 30;
    let i3 = y3.to_bits() >> 29;
    let decalage = (i2 & 2) | (i1 & !2);
    let decalage = (i3 & 4) | (decalage & !4);
    (0x2E74u32 >> decalage) & 0x0101
}

/// **Où la courbe croise l'axe** `y = 0` (`axe = 1`) ou `x = 0` (`axe = 0`) : les deux
/// abscisses (ou ordonnées) des racines de `a t² − 2b t + c`. Un discriminant négatif est
/// ramené à zéro — une racine double au minimum —, et une courbe presque droite se résout comme
/// une droite.
pub fn racines(p: [[f32; 2]; 3], axe: usize) -> [f32; 2] {
    let autre = 1 - axe;
    let a = [
        p[0][0] - p[1][0] * 2.0 + p[2][0],
        p[0][1] - p[1][1] * 2.0 + p[2][1],
    ];
    let b = [p[0][0] - p[1][0], p[0][1] - p[1][1]];
    let ra = 1.0 / a[axe];
    let rb = 0.5 / b[axe];
    let d = (b[axe] * b[axe] - a[axe] * p[0][axe]).max(0.0).sqrt();
    let (mut t1, mut t2) = ((b[axe] - d) * ra, (b[axe] + d) * ra);
    if a[axe].abs() < PRESQUE_DROITE {
        t1 = p[0][axe] * rb;
        t2 = t1;
    }
    let en = |t: f32| (a[autre] * t - b[autre] * 2.0) * t + p[0][autre];
    [en(t1), en(t2)]
}

/// **La couverture vue par un rayon** : la somme et le poids, le long de l'axe `axe` (0 : le
/// rayon horizontal, qui lit les bandes horizontales ; 1 : le vertical).
fn le_long_d_un_rayon(
    forme: &Emballage,
    bande: u32,
    point: [f32; 2],
    pixels_par_unite: [f32; 2],
    axe: usize,
) -> (f32, f32) {
    // Le rayon horizontal mesure en x et lit le signe des ordonnées ; le vertical, l'inverse.
    let (mesure, signe) = if axe == 0 { (0, 1) } else { (1, 0) };
    let en_tete = 2 * (bande as usize);
    let (combien, ou) = (forme.bandes[en_tete], forme.bandes[en_tete + 1]);
    let (mut somme, mut poids) = (0.0f32, 0.0f32);
    for k in 0..combien {
        let r = forme.bandes[(ou + k) as usize] as usize;
        let (a, b) = (forme.courbes[r], forme.courbes[r + 1]);
        let p = [
            [a[0] - point[0], a[1] - point[1]],
            [a[2] - point[0], a[3] - point[1]],
            [b[0] - point[0], b[1] - point[1]],
        ];
        // Triées de la plus loin à la plus près : passée tout entière derrière le pixel, plus
        // aucune ne le touche.
        if p[0][mesure].max(p[1][mesure]).max(p[2][mesure]) * pixels_par_unite[mesure] < -0.5 {
            break;
        }
        let code = code_des_racines(p[0][signe], p[1][signe], p[2][signe]);
        if code == 0 {
            continue;
        }
        let r = racines(p, signe).map(|x| x * pixels_par_unite[mesure]);
        // Le rayon horizontal ajoute la première racine et retire la seconde ; le vertical, qui
        // monte dans l'autre sens du plan, l'inverse.
        let sens = if axe == 0 { 1.0 } else { -1.0 };
        if code & 1 != 0 {
            somme += sens * (r[0] + 0.5).clamp(0.0, 1.0);
            poids = poids.max((1.0 - r[0].abs() * 2.0).clamp(0.0, 1.0));
        }
        if code > 1 {
            somme -= sens * (r[1] + 0.5).clamp(0.0, 1.0);
            poids = poids.max((1.0 - r[1].abs() * 2.0).clamp(0.0, 1.0));
        }
    }
    (somme, poids)
}

/// **La part du pixel de centre `point` que la forme couvre**, `pixels_par_unite` disant
/// combien de pixels vaut une unité de la forme, en x et en y — la règle des enroulements non
/// nuls.
pub fn couverture(forme: &Emballage, point: [f32; 2], pixels_par_unite: [f32; 2]) -> f32 {
    let [ex, ey, dx, dy] = forme.transformation;
    let [nh, nv] = forme.nombre_de_bandes;
    // La bande du pixel : la même formule que l'emballage, la plus basse des deux arithmétiques
    // (BANDES-2 range la courbe dans les deux).
    let (horizontale, _) = bandes_de(point[1], ey, dy, nh - 1);
    let (verticale, _) = bandes_de(point[0], ex, dx, nv - 1);
    let (xc, xp) = le_long_d_un_rayon(forme, horizontale, point, pixels_par_unite, 0);
    let (yc, yp) = le_long_d_un_rayon(forme, nh + verticale, point, pixels_par_unite, 1);
    let melee = (xc * xp + yc * yp).abs() / (xp + yp).max(PRESQUE_DROITE);
    melee.max(xc.abs().min(yc.abs())).clamp(0.0, 1.0)
}
