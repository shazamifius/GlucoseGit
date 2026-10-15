//! **Ranger une forme comme la carte la lit** (P8, fiche 73) : ses courbes, ses bandes.
//!
//! # Les courbes
//!
//! Une courbe tient dans deux cases de quatre nombres : la première porte son début et son
//! contrôle, la seconde commence par sa fin. Les courbes d'un contour étant jointes, la fin de
//! l'une est le début de la suivante : un contour de `k` courbes tient en `k + 1` cases, la
//! dernière ne portant que le point où il se referme. C'est le rangement de Slug.
//!
//! # BANDES-1 — autant de bandes que de courbes
//!
//! Un pixel lit toutes les courbes de sa bande. Si une forme a `C` courbes et `n` bandes d'égale
//! épaisseur, une courbe de hauteur `h` touche au plus `h·n/H + 2` bandes, donc un pixel en lit
//! en moyenne au plus `Σh/H + 2C/n` — la première part est ce que son rayon **croise vraiment**
//! (rien ne la réduit), la seconde est le prix du découpage. Avec **`n = C`**, ce prix est d'au
//! plus deux courbes : aucun nombre n'est choisi, et la mémoire reste de l'ordre de la forme
//! (deux entiers par bande). Les courbes strictement horizontales ne croisent aucun rayon
//! horizontal : elles n'entrent dans aucune bande horizontale (de même pour les verticales).
//!
//! # BANDES-2 — une courbe est dans chaque bande que la carte peut calculer pour elle
//!
//! Le nuanceur calcule la bande d'un pixel par `tronquer(coordonnée × échelle + décalage)`, en
//! `f32`. Une courbe qu'un rayon croise a ce rayon entre le plus bas et le plus haut de ses trois
//! points ; l'arithmétique arrondie étant monotone, la bande du rayon est entre celles de ces deux
//! bornes — **calculées ici par la même formule, en `f32`**. Une carte graphique peut fondre la
//! multiplication et l'addition en une seule opération (et l'arrondi diffère alors d'un cran) :
//! les deux façons sont calculées, et la courbe va dans toutes les bandes de l'une ou de l'autre.
//! C'est la marge de Slug (« environ 1/1024 d'em ») sans le nombre : la précision même du calcul.

use super::Quadratique;

/// **Une forme rangée** : ses courbes, ses bandes, et ce qu'il faut pour les lire.
///
/// Les positions sont **relatives** : une courbe se nomme par son rang dans [`Self::courbes`],
/// une bande par son rang dans [`Self::bandes`]. La carte, qui met bout à bout les formes de tout
/// un écran, y ajoute où celle-ci commence.
#[derive(Debug, Clone, PartialEq)]
pub struct Emballage {
    /// Les cases des courbes, quatre nombres chacune.
    pub courbes: Vec<[f32; 4]>,
    /// Les bandes : pour chaque bande horizontale puis verticale, `(combien, où)` ; puis les
    /// listes, chaque entrée le rang d'une courbe.
    pub bandes: Vec<u32>,
    /// La boîte de la forme : gauche, bas, droite, haut — les extrêmes de tous ses points.
    pub boite: [f32; 4],
    /// Combien de bandes horizontales, puis verticales.
    pub nombre_de_bandes: [u32; 2],
    /// L'échelle en x et en y, puis le décalage en x et en y : la bande d'un pixel est
    /// `tronquer(coordonnée × échelle + décalage)`, bornée à ses bandes.
    pub transformation: [f32; 4],
}

/// La bande d'une coordonnée, par les deux arithmétiques qu'une carte peut employer
/// (BANDES-2) : la plus basse et la plus haute.
pub fn bandes_de(v: f32, echelle: f32, decalage: f32, derniere: u32) -> (u32, u32) {
    let borner = |x: f32| (x as i32).clamp(0, derniere as i32) as u32;
    let separees = borner(v * echelle + decalage);
    let fondue = borner(v.mul_add(echelle, decalage));
    (separees.min(fondue), separees.max(fondue))
}

/// **Range ces contours** comme la carte les lit, ou `None` s'ils n'ont aucune courbe.
pub fn emballer(contours: &[Vec<Quadratique>]) -> Option<Emballage> {
    let (courbes, rangs) = ranger_les_courbes(contours);
    if rangs.is_empty() {
        return None;
    }
    let boite = boite_de(&courbes, &rangs);
    let lire = |r: u32| -> [[f32; 2]; 3] {
        let (a, b) = (courbes[r as usize], courbes[r as usize + 1]);
        [[a[0], a[1]], [a[2], a[3]], [b[0], b[1]]]
    };
    let mut axes = [Vec::new(), Vec::new()];
    for (axe, liste) in axes.iter_mut().enumerate() {
        // L'axe 0 : les bandes horizontales, que coupe y ; l'axe 1 : les verticales, que coupe x.
        let coupe = 1 - axe;
        *liste = rangs
            .iter()
            .copied()
            .filter(|&r| {
                let p = lire(r);
                !(p[0][coupe] == p[1][coupe] && p[1][coupe] == p[2][coupe])
            })
            .collect::<Vec<u32>>();
    }
    let nombre = [axes[0].len().max(1) as u32, axes[1].len().max(1) as u32];
    let (bas, haut) = ([boite[0], boite[1]], [boite[2], boite[3]]);
    // Les bandes couvrent la boîte d'égale épaisseur ; une boîte plate n'en a qu'une.
    let echelle = |a: usize, n: u32| {
        let etendue = haut[a] - bas[a];
        if etendue > 0.0 {
            n as f32 / etendue
        } else {
            0.0
        }
    };
    // `nombre[0]` bandes horizontales s'étagent en y (axe 1 de la boîte), `nombre[1]`
    // verticales en x.
    let (ex, ey) = (echelle(0, nombre[1]), echelle(1, nombre[0]));
    let transformation = [ex, ey, -bas[0] * ex, -bas[1] * ey];
    let bandes = ecrire_les_bandes(&axes, nombre, transformation, &lire);
    Some(Emballage {
        courbes,
        bandes,
        boite,
        nombre_de_bandes: nombre,
        transformation,
    })
}

/// Les cases des courbes, contour après contour, et le rang de chaque courbe.
fn ranger_les_courbes(contours: &[Vec<Quadratique>]) -> (Vec<[f32; 4]>, Vec<u32>) {
    let mut courbes = Vec::new();
    let mut rangs = Vec::new();
    let f = |p: [f64; 2]| [p[0] as f32, p[1] as f32];
    for contour in contours.iter().filter(|c| !c.is_empty()) {
        for q in contour {
            let (d, c) = (f(q.debut), f(q.controle));
            rangs.push(courbes.len() as u32);
            courbes.push([d[0], d[1], c[0], c[1]]);
        }
        // La case qui referme : la fin de la dernière courbe, qui est le début de la première.
        let fin = f(contour[contour.len() - 1].fin);
        courbes.push([fin[0], fin[1], 0.0, 0.0]);
    }
    (courbes, rangs)
}

/// Les extrêmes de tous les points des courbes : gauche, bas, droite, haut.
fn boite_de(courbes: &[[f32; 4]], rangs: &[u32]) -> [f32; 4] {
    let mut b = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for &r in rangs {
        let (a, s) = (courbes[r as usize], courbes[r as usize + 1]);
        for (x, y) in [(a[0], a[1]), (a[2], a[3]), (s[0], s[1])] {
            b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
        }
    }
    b
}

/// L'en-tête puis les listes : chaque bande horizontale, puis chaque verticale.
fn ecrire_les_bandes(
    axes: &[Vec<u32>; 2],
    nombre: [u32; 2],
    [ex, ey, dx, dy]: [f32; 4],
    lire: &dyn Fn(u32) -> [[f32; 2]; 3],
) -> Vec<u32> {
    let total = (nombre[0] + nombre[1]) as usize;
    let mut listes: Vec<Vec<u32>> = vec![Vec::new(); total];
    for (axe, candidates) in axes.iter().enumerate() {
        // Une bande horizontale se repère par y, et se trie par le plus grand x.
        let (coupe, tri) = (1 - axe, axe);
        let (echelle, decalage) = if axe == 0 { (ey, dy) } else { (ex, dx) };
        let premiere = if axe == 0 { 0 } else { nombre[0] as usize };
        let derniere = nombre[axe] - 1;
        for &r in candidates {
            let p = lire(r);
            let (lo, hi) = extremes(p.map(|q| q[coupe]));
            let (de, _) = bandes_de(lo, echelle, decalage, derniere);
            let (_, a) = bandes_de(hi, echelle, decalage, derniere);
            for b in de..=a {
                listes[premiere + b as usize].push(r);
            }
        }
        for liste in &mut listes[premiere..premiere + nombre[axe] as usize] {
            liste.sort_by(|&a, &b| {
                let max = |r: u32| extremes(lire(r).map(|q| q[tri])).1;
                max(b).total_cmp(&max(a))
            });
        }
    }
    let mut bandes = Vec::with_capacity(2 * total + listes.iter().map(Vec::len).sum::<usize>());
    let mut ou = 2 * total as u32;
    for liste in &listes {
        bandes.extend([liste.len() as u32, ou]);
        ou += liste.len() as u32;
    }
    for liste in listes {
        bandes.extend(liste);
    }
    bandes
}

fn extremes(v: [f32; 3]) -> (f32, f32) {
    (v[0].min(v[1]).min(v[2]), v[0].max(v[1]).max(v[2]))
}
