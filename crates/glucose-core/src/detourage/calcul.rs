//! **L'exécution d'un modèle de détourage** (DETOURER-1) : le graphe parcouru dans l'ordre, chaque
//! tenseur rendu dès sa dernière lecture. Les opérations sont celles d'ONNX, à la lettre : l'étalon
//! (`outils/detourage/reference.py`) les écrit en numpy, et les épreuves comparent.

use super::conv;
use super::modele::{Modele, Sorte};

/// **Un tenseur** : `c` plans de `h × l` valeurs, rangés plan après plan, ligne après ligne.
#[derive(Debug, Clone, PartialEq)]
pub struct Tenseur {
    pub c: usize,
    pub h: usize,
    pub l: usize,
    pub v: Vec<f32>,
}

impl Tenseur {
    pub fn plan(&self, c: usize) -> &[f32] {
        let n = self.h * self.l;
        &self.v[c * n..(c + 1) * n]
    }
}

/// **Le masque brut du modèle** pour une entrée déjà normalisée (`3 × taille × taille`), calculé
/// sur `fils` fils.
pub fn executer(modele: &Modele, entree: Tenseur, fils: usize) -> Tenseur {
    let mut t: Vec<Option<Tenseur>> = vec![None; modele.tenseurs];
    t[0] = Some(entree);
    for (rang, op) in modele.operations.iter().enumerate() {
        let lu = |i: usize| t[op.entrees[i]].as_ref().expect("vérifié à la lecture");
        let mut y = match &op.sorte {
            Sorte::Conv(c) => conv::conv(lu(0), c, op.relu, fils),
            Sorte::MaxPool {
                noyau,
                pas,
                plafond,
            } => maxpool(lu(0), *noyau, *pas, *plafond),
            Sorte::Agrandir { hauteur, largeur } => agrandir(lu(0), *hauteur, *largeur),
            Sorte::Concat => concat(
                &op.entrees
                    .iter()
                    .map(|&e| t[e].as_ref().expect("vérifié"))
                    .collect::<Vec<_>>(),
            ),
            Sorte::Ajouter => ajouter(lu(0), lu(1)),
            Sorte::Sigmoide => appliquer(lu(0).clone(), |x| 1.0 / (1.0 + (-x).exp())),
        };
        if op.relu && !matches!(op.sorte, Sorte::Conv(_)) {
            y = appliquer(y, |x| x.max(0.0));
        }
        for &e in &op.entrees {
            if modele.derniere_lecture[e] == rang && e != modele.sortie {
                t[e] = None;
            }
        }
        t[op.sortie] = Some(y);
    }
    t[modele.sortie].take().expect("vérifié à la lecture")
}

fn appliquer(mut x: Tenseur, f: impl Fn(f32) -> f32) -> Tenseur {
    x.v.iter_mut().for_each(|v| *v = f(*v));
    x
}

/// Le plus grand de chaque fenêtre ; avec `plafond`, la dernière fenêtre peut déborder du bord
/// (elle ne lit que ce qui est dedans).
pub fn maxpool(x: &Tenseur, noyau: usize, pas: usize, plafond: bool) -> Tenseur {
    let n = |v: usize| {
        if plafond {
            (v - noyau).div_ceil(pas) + 1
        } else {
            (v - noyau) / pas + 1
        }
    };
    let (ho, lo) = (n(x.h), n(x.l));
    let mut v = Vec::with_capacity(x.c * ho * lo);
    for c in 0..x.c {
        let plan = x.plan(c);
        for yo in 0..ho {
            let lignes = yo * pas..(yo * pas + noyau).min(x.h);
            for xo in 0..lo {
                let colonnes = xo * pas..(xo * pas + noyau).min(x.l);
                let m = lignes
                    .clone()
                    .flat_map(|y| plan[y * x.l + colonnes.start..y * x.l + colonnes.end].iter())
                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                v.push(m);
            }
        }
    }
    Tenseur {
        c: x.c,
        h: ho,
        l: lo,
        v,
    }
}

/// **L'agrandissement bilinéaire d'ONNX, repère « demi-pixel »** : le centre du pixel de sortie
/// `(i + ½)` tombe en `(i + ½)·n/m − ½` dans l'entrée, borné à ses bords.
pub fn agrandir(x: &Tenseur, h: usize, l: usize) -> Tenseur {
    let (ys, xs) = (axe(x.h, h), axe(x.l, l));
    let mut v = Vec::with_capacity(x.c * h * l);
    for c in 0..x.c {
        let plan = x.plan(c);
        for &(y0, y1, fy) in &ys {
            let (haut, bas) = (
                &plan[y0 * x.l..(y0 + 1) * x.l],
                &plan[y1 * x.l..(y1 + 1) * x.l],
            );
            for &(x0, x1, fx) in &xs {
                let a = haut[x0] * (1.0 - fx) + haut[x1] * fx;
                let b = bas[x0] * (1.0 - fx) + bas[x1] * fx;
                v.push(a * (1.0 - fy) + b * fy);
            }
        }
    }
    Tenseur { c: x.c, h, l, v }
}

/// Pour chaque position de sortie, les deux voisines d'entrée et le poids de la seconde.
pub(super) fn axe(n: usize, m: usize) -> Vec<(usize, usize, f32)> {
    let echelle = n as f32 / m as f32;
    (0..m)
        .map(|i| {
            let s = ((i as f32 + 0.5) * echelle - 0.5).clamp(0.0, (n - 1) as f32);
            let i0 = s.floor() as usize;
            (i0, (i0 + 1).min(n - 1), s - i0 as f32)
        })
        .collect()
}

fn concat(xs: &[&Tenseur]) -> Tenseur {
    let (h, l) = (xs[0].h, xs[0].l);
    let mut v = Vec::with_capacity(xs.iter().map(|x| x.v.len()).sum());
    xs.iter().for_each(|x| v.extend_from_slice(&x.v));
    Tenseur {
        c: xs.iter().map(|x| x.c).sum(),
        h,
        l,
        v,
    }
}

fn ajouter(a: &Tenseur, b: &Tenseur) -> Tenseur {
    let v = a.v.iter().zip(&b.v).map(|(x, y)| x + y).collect();
    Tenseur { v, ..*a }
}
