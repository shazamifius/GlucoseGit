//! **La couleur du premier plan, sans le fond** (DETOURER-2, fiche 121).
//!
//! Un pixel du bord d'un objet mêle deux couleurs : `I = α·F + (1 − α)·B`. Garder `I` sous une
//! opacité `α` y laisse le fond d'origine — le liseré vert qu'on voyait autour de sa boule sur le fond
//! multicolore. Il faut `F`.
//!
//! La méthode de Marco Forte (« Approximate Fast Foreground Colour Estimation », ICIP 2021,
//! doi:10.1109/ICIP42928.2021.9506164), qui approche celle de Germer et al. : on estime la couleur
//! du premier plan et celle du fond **près de chaque pixel** — la moyenne, pondérée par l'opacité,
//! d'une fenêtre autour —, puis on corrige l'estimation pour qu'elle rende exactement `I` :
//!
//! ```text
//! F̄ = flou(F·α) / flou(α)          B̄ = flou(B·(1 − α)) / flou(1 − α)
//! F = F̄ + α · (I − α·F̄ − (1 − α)·B̄)
//! ```
//!
//! Deux passes, comme l'article : une large (rayon 90 pixels, `F = B = I`), qui va chercher des
//! couleurs loin, puis une fine (rayon 6), repartie des estimations de la première. Les rayons sont
//! ceux de l'article — des nombres de ressenti de ses auteurs, pas les nôtres.

/// Les rayons des deux passes (l'article, et le code de ses auteurs).
const RAYONS: [usize; 2] = [90, 6];

/// Ce qui empêche de diviser par une opacité nulle (l'article).
const EPSILON: f32 = 1e-5;

/// **Les couleurs du premier plan** de `rvb` (trois plans de `l × h` valeurs entre 0 et 1) sous le
/// masque `alpha`.
pub fn estimer(rvb: &[Vec<f32>; 3], alpha: &[f32], (l, h): (usize, usize)) -> [Vec<f32>; 3] {
    let (mut f, mut b) = (rvb.clone(), rvb.clone());
    for r in RAYONS {
        let a_flou = flou(alpha, (l, h), r);
        let un_moins: Vec<f32> = alpha.iter().map(|a| 1.0 - a).collect();
        for c in 0..3 {
            let fa: Vec<f32> = f[c].iter().zip(alpha).map(|(f, a)| f * a).collect();
            let b1a: Vec<f32> = b[c].iter().zip(&un_moins).map(|(b, u)| b * u).collect();
            let (fa, b1a) = (flou(&fa, (l, h), r), flou(&b1a, (l, h), r));
            for i in 0..alpha.len() {
                let fm = fa[i] / (a_flou[i] + EPSILON);
                let bm = b1a[i] / ((1.0 - a_flou[i]) + EPSILON);
                let a = alpha[i];
                f[c][i] = (fm + a * (rvb[c][i] - a * fm - (1.0 - a) * bm)).clamp(0.0, 1.0);
                b[c][i] = bm;
            }
        }
    }
    f
}

/// **La moyenne d'une fenêtre carrée** de côté `2r + 1`, en ne comptant que ce qui est dans
/// l'image — deux passes de sommes glissantes, quel que soit le rayon.
pub fn flou(plan: &[f32], (l, h): (usize, usize), r: usize) -> Vec<f32> {
    let mut lignes = vec![0.0; plan.len()];
    for (src, dest) in plan.chunks_exact(l).zip(lignes.chunks_exact_mut(l)) {
        glisser(src, dest, r);
    }
    let mut sortie = vec![0.0; plan.len()];
    let mut colonne = vec![0.0; h];
    let mut moyenne = vec![0.0; h];
    for x in 0..l {
        colonne
            .iter_mut()
            .enumerate()
            .for_each(|(y, c)| *c = lignes[y * l + x]);
        glisser(&colonne, &mut moyenne, r);
        moyenne
            .iter()
            .enumerate()
            .for_each(|(y, m)| sortie[y * l + x] = *m);
    }
    sortie
}

/// La moyenne glissante d'une rangée : `dest[i]` est la moyenne de `src[i − r ..= i + r]`, bornée
/// à la rangée. Les sommes se font en f64 : sur des milliers de pixels, une somme f32 qu'on
/// augmente et diminue tour à tour dérive.
fn glisser(src: &[f32], dest: &mut [f32], r: usize) {
    let n = src.len();
    let mut somme: f64 = src[..r.min(n)].iter().map(|&v| f64::from(v)).sum();
    for i in 0..n {
        if i + r < n {
            somme += f64::from(src[i + r]);
        }
        if i > r {
            somme -= f64::from(src[i - r - 1]);
        }
        let compte = (i + r).min(n - 1) + 1 - i.saturating_sub(r);
        dest[i] = (somme / compte as f64) as f32;
    }
}
