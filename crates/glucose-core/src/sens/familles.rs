//! **PARENTE-1 — les familles sûres d'un document, et leurs teintes** (fiche 106).
//!
//! Sa règle (VISION § 5) : *des textes qui ont un rapport ont des couleurs proches* ; *si l'IA n'est
//! pas sûre, et qu'ils sont vraiment distants, pas de liaison de couleur*. Et son choix devant
//! l'image, le 11/11/2026 : **par familles**.
//!
//! # Pourquoi des familles, et pas une teinte par texte
//!
//! Le sens d'un texte a 128 dimensions, une teinte une seule. Ramener chaque texte à un angle (les
//! deux axes principaux du document, ou le spectre de son graphe) perdait presque tout : deux résumés
//! du même thème ne tombaient pas plus près en teinte que deux résumés au hasard (fiche 106). Mais sa
//! règle ne demande qu'une direction : un parent sûr rapproche. On rassemble donc d'abord ce qui est
//! sûrement parent, et seulement ensuite on place les familles sur le cercle.
//!
//! # Le hasard d'un document
//!
//! « Sûr » veut dire : plus semblable que deux textes du document pris au hasard. Ce hasard se lit
//! sur toutes les paires du document — mais **pas par leur moyenne** : les vraies parentés en font
//! partie, tirent la moyenne et l'écart-type vers le haut, et le hasard mesuré ressemblait à un
//! document déjà parent. Mesuré sur la carte de Wikipédia (fiche 106) : la sûreté qui en sortait
//! valait la même chose pour un membre de famille et pour un texte seul. Il se lit par **la médiane
//! et l'écart absolu médian** — les estimateurs robustes, que quelques pour cent de vraies parentés
//! ne déplacent pas (Hampel, 1974). L'écart absolu médian est ramené à l'écart-type d'une loi normale
//! par `1 / Φ⁻¹(3/4)`, la constante qui fait de lui un estimateur de cet écart-type : ce n'est pas un
//! réglage.
//!
//! # La loi
//!
//! 1. **Sûr** : plus semblable que le hasard, de **deux écarts** — le seuil de signification d'usage
//!    (une chance sur quarante-quatre qu'une paire quelconque le franchisse). Il se lit sur le
//!    document : il n'est pas réglé.
//! 2. **Les familles** : le regroupement hiérarchique par **liaison moyenne** (deux groupes se
//!    ressemblent comme la moyenne de leurs paires), coupé au seuil — une famille est un groupe dont
//!    les textes se ressemblent, en moyenne, sûrement. Calculé par la chaîne des plus proches voisins
//!    (Murtagh, 1983), en temps quadratique : la liaison moyenne est réductible, l'arbre est le même.
//!    Un texte qui ne rejoint personne reste **seul** : gris, sans liaison.
//! 3. **Les teintes** : les familles rangées sur le cercle par **sériation circulaire spectrale** (les
//!    deux premiers vecteurs propres non triviaux du laplacien normalisé de leurs ressemblances), puis
//!    espacées en proportion de la dissemblance de deux voisines : des familles parentes tombent près.
//! 4. **La sûreté d'un texte** : à quel point l'IA est sûre qu'il a un parent — l'opacité de son
//!    halo (sa parole : *« parfaitement d'accord »*). En deux temps, deux outils d'usage :
//!    * **la chance de son plus proche parent** : chaque texte est comparé à tous les autres, et le
//!      meilleur de trois cent cinquante tirages du hasard est presque toujours « bon ». La chance
//!      que le hasard fasse aussi bien est `1 − Φ(z)ⁿ⁻¹`, la correction de Šidák (1967) ;
//!    * **la part du hasard à sa hauteur** : un texte du pur hasard a une chance uniforme entre 0
//!      et 1, et en peindre l'opacité ferait paraître à moitié sûrs la moitié des textes d'un
//!      document sans aucune parenté. Le **taux local de fausses découvertes** d'Efron (2001) le
//!      dit texte par texte : la densité des chances qu'aurait le seul hasard (uniforme, 1) sur
//!      celle des chances du document, à sa chance. Cette densité s'estime par **Grenander
//!      (1956)** — la pente du plus petit majorant concave de leur répartition, sans aucune fenêtre
//!      à régler —, et la part du hasard se prend à 1, la plus prudente. La sûreté est `1 − 1/g`.
//!
//!    Benjamini et Hochberg (1995), essayés d'abord, disent une part **d'ensemble** : dans un
//!    document riche en vraies parentés, un texte du hasard y héritait de la bonne note des autres
//!    (0,88 sur un mélange planté, contre 0 ici).
//!
//!    Mesuré sur la carte de Wikipédia (fiche 106) : 0,71 pour un membre de famille médian, 0 pour
//!    **chaque** texte seul ; au-dessus de 0,5, le plus proche parent est juste à 97 %. Sur des
//!    textes tirés au hasard : presque 0 dès soixante textes ; sous une quinzaine, l'IA hésite
//!    (0,3 au plus) — elle n'a pas de quoi juger. Sa limite : un texte du hasard qui tombe, une fois
//!    sur quelques centaines, extrêmement près d'un autre paraît sûr.

use std::f64::consts::{PI, TAU};

/// **Ce que l'IA tire des ressemblances d'un document.**
#[derive(Debug, Clone, PartialEq)]
pub struct Parente {
    /// La famille de chaque texte, dans l'ordre reçu ; rien pour un texte sans parent sûr.
    pub famille: Vec<Option<usize>>,
    /// La teinte de chaque famille, en radians, dans `[0, 2π)`.
    pub teintes: Vec<f64>,
    /// La sûreté de chaque texte, de 0 à 1.
    pub surete: Vec<f32>,
}

/// Deux écarts au-dessus du hasard : le seuil de signification d'usage.
const ECARTS_TYPES: f64 = 2.0;

/// L'écart absolu médian d'une loi normale vaut `Φ⁻¹(3/4)` fois son écart-type : son inverse fait de
/// lui un estimateur de l'écart-type (Hampel, 1974).
const MAD_VERS_ECART_TYPE: f64 = 1.482_602_218_505_602;

/// **La parenté d'un document**, depuis le sens de chacun de ses textes (de longueur 1).
pub fn parente(sens: &[Vec<f32>]) -> Parente {
    let n = sens.len();
    let s = Ressemblances::de(sens);
    let mut paires: Vec<f32> = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .map(|(i, j)| s.de_paire(i, j))
        .collect();
    let Some(hasard) = Hasard::de(&mut paires) else {
        return Parente {
            famille: vec![None; n],
            teintes: Vec::new(),
            surete: vec![0.0; n],
        };
    };
    drop(paires);
    let chances: Vec<f64> = (0..n)
        .map(|i| {
            let parent = (0..n)
                .filter(|&j| j != i)
                .map(|j| s.de_paire(i, j))
                .fold(f32::NEG_INFINITY, f32::max);
            hasard.chance(parent, n - 1)
        })
        .collect();
    let surete = efron(&chances);
    let famille = familles(s, hasard.seuil());
    let teintes = teintes(sens, &famille);
    Parente {
        famille,
        teintes,
        surete,
    }
}

impl Parente {
    /// **Le cercle orienté** : la famille du premier texte qui en a une prend la teinte `origine`,
    /// et la première autre famille du document tombe du côté où les angles croissent.
    ///
    /// La sériation ne dit que l'ordre des familles sur le cercle : où il commence, et dans quel
    /// sens il tourne, sont libres — un vecteur propre n'a pas de signe. Les fixer par le hasard du
    /// calcul ferait tourner toutes les couleurs quand un texte s'ajoute ; les fixer par le document
    /// lui-même les garde : le premier texte d'un document y reste le premier, et sa famille sa
    /// teinte.
    pub fn orienter(&mut self, origine: f64) {
        let mut dans_l_ordre = self.famille.iter().flatten();
        let Some(&premiere) = dans_l_ordre.next() else {
            return;
        };
        let depart = self.teintes[premiere];
        let seconde = dans_l_ordre.find(|&&f| f != premiere);
        let sens = match seconde {
            Some(&f) if (self.teintes[f] - depart).rem_euclid(TAU) > PI => -1.0,
            _ => 1.0,
        };
        for t in &mut self.teintes {
            *t = (origine + sens * (*t - depart)).rem_euclid(TAU);
        }
    }
}

/// **Le hasard d'un document** : la loi des ressemblances de deux de ses textes pris au hasard,
/// supposée normale, lue par sa médiane et son écart absolu médian.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Hasard {
    pub(super) mediane: f64,
    pub(super) ecart: f64,
}

impl Hasard {
    /// Le hasard de ces ressemblances (leur ordre est perdu) ; rien sans aucune paire.
    pub(super) fn de(paires: &mut [f32]) -> Option<Self> {
        let mediane = la_mediane(paires)?;
        for p in paires.iter_mut() {
            *p = (f64::from(*p) - mediane).abs() as f32;
        }
        let ecart = la_mediane(paires)? * MAD_VERS_ECART_TYPE;
        Some(Self { mediane, ecart })
    }

    /// Le seuil du sûr : deux écarts au-dessus de la médiane.
    fn seuil(self) -> f32 {
        (self.mediane + ECARTS_TYPES * self.ecart) as f32
    }

    /// **La chance que le hasard fasse aussi bien** qu'un plus proche parent à `parent`, trouvé
    /// parmi `comparaisons` textes : `1 − Φ(z)ᶜ`. Un hasard sans écart (plus de la moitié des paires
    /// exactement pareilles) rend sûr tout ce qui le dépasse, et rien d'autre.
    pub(super) fn chance(self, parent: f32, comparaisons: usize) -> f64 {
        let z = (f64::from(parent) - self.mediane) / self.ecart;
        if z.is_nan() {
            return 1.0;
        }
        // 1 − Φ(z)ᶜ = −expm1(c · ln(1 − Q(z))), Q la queue de la loi normale : exact aussi loin
        // dans la queue que `erfc` l'est, là où `Φ(z)` arrondi à 1 rendrait tout sûr.
        let queue = 0.5 * erfc(z / std::f64::consts::SQRT_2);
        -((comparaisons as f64) * (-queue).ln_1p()).exp_m1()
    }
}

/// **La sûreté de chaque texte** : `1 − 1/g(p)`, où `g` est la densité de Grenander des chances du
/// document — la pente, en `p`, du plus petit majorant concave de leur répartition.
pub(super) fn efron(chances: &[f64]) -> Vec<f32> {
    let n = chances.len() as f64;
    let mut rangees = chances.to_vec();
    rangees.sort_by(f64::total_cmp);
    // La répartition : un point par chance distincte, la part des chances qui ne la dépassent pas ;
    // l'origine reste à part, pour qu'une chance nulle ait une pente infinie.
    let mut points: Vec<(f64, f64)> = vec![(0.0, 0.0)];
    for (k, &x) in rangees.iter().enumerate() {
        let y = (k + 1) as f64 / n;
        match points.last_mut() {
            Some(d) if d.0 == x && d.1 > 0.0 => d.1 = y,
            _ => points.push((x, y)),
        }
    }
    if points.last().is_some_and(|d| d.0 < 1.0) {
        points.push((1.0, 1.0));
    }
    let enveloppe = majorant_concave(points);
    chances
        .iter()
        .map(|&p| {
            let k = enveloppe.partition_point(|e| e.0 < p).max(1);
            let (a, b) = (enveloppe[k - 1], enveloppe[k]);
            let g = if b.0 > a.0 {
                (b.1 - a.1) / (b.0 - a.0)
            } else {
                f64::INFINITY
            };
            (1.0 - 1.0 / g).max(0.0) as f32
        })
        .collect()
}

/// **Le plus petit majorant concave** de points rangés par abscisse : une pile, comme l'enveloppe
/// convexe d'Andrew — un sommet qui tombe sous la corde de ses voisins s'en va.
fn majorant_concave(points: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let mut enveloppe: Vec<(f64, f64)> = Vec::with_capacity(points.len());
    for p in points {
        while let [.., a, b] = enveloppe[..] {
            if (b.1 - a.1) * (p.0 - a.0) <= (p.1 - a.1) * (b.0 - a.0) {
                enveloppe.pop();
            } else {
                break;
            }
        }
        enveloppe.push(p);
    }
    enveloppe
}

/// La médiane de ces valeurs (leur ordre est perdu), sans les trier : `select_nth_unstable`.
fn la_mediane(valeurs: &mut [f32]) -> Option<f64> {
    let n = valeurs.len();
    if n == 0 {
        return None;
    }
    let (bas, &mut haut, _) = valeurs.select_nth_unstable_by(n / 2, f32::total_cmp);
    if n % 2 == 1 {
        return Some(f64::from(haut));
    }
    let sous = bas.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    Some((f64::from(sous) + f64::from(haut)) / 2.0)
}

/// **La fonction d'erreur complémentaire**, à une erreur relative sous 1,2 × 10⁻⁷ partout : le
/// développement de Tchebychev de *Numerical Recipes* (Press et al., § 6.2), qui garde sa précision
/// relative loin dans la queue — c'est là que la sûreté se décide.
pub(super) fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = [
        -1.265_512_23,
        1.000_023_68,
        0.374_091_96,
        0.096_784_18,
        -0.186_288_06,
        0.278_868_07,
        -1.135_203_98,
        1.488_515_87,
        -0.822_152_23,
        0.170_872_77,
    ]
    .iter()
    .rev()
    .fold(0.0, |acc, &c| acc * t + c);
    let r = t * (-z * z + poly).exp();
    if x >= 0.0 {
        r
    } else {
        2.0 - r
    }
}

/// La matrice des ressemblances, pleine.
struct Ressemblances {
    n: usize,
    m: Vec<f32>,
}

impl Ressemblances {
    fn de(sens: &[Vec<f32>]) -> Self {
        let n = sens.len();
        let mut m = vec![0.0; n * n];
        for i in 0..n {
            for j in i + 1..n {
                let r = super::ressemblance(&sens[i], &sens[j]);
                m[i * n + j] = r;
                m[j * n + i] = r;
            }
        }
        Self { n, m }
    }

    fn de_paire(&self, i: usize, j: usize) -> f32 {
        self.m[i * self.n + j]
    }
}

/// **Les familles** : la liaison moyenne coupée au seuil, par la chaîne des plus proches voisins.
/// Une famille a au moins deux textes ; elles sont numérotées dans l'ordre de leur premier texte.
fn familles(s: Ressemblances, seuil: f32) -> Vec<Option<usize>> {
    let n = s.n;
    let mut m = s.m;
    let mut taille = vec![1usize; n];
    let mut actif = vec![true; n];
    let mut chaine: Vec<usize> = Vec::new();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut restants = n;
    while restants > 1 {
        if chaine.is_empty() {
            chaine.extend(actif.iter().position(|&a| a));
        }
        let a = chaine[chaine.len() - 1];
        let precedent = chaine.len().checked_sub(2).map(|k| chaine[k]);
        // Le plus proche voisin de `a` ; à égalité, celui qui le précède dans la chaîne.
        let mut b = precedent.unwrap_or(usize::MAX);
        let mut meilleure = precedent.map_or(f32::NEG_INFINITY, |p| m[a * n + p]);
        for k in (0..n).filter(|&k| actif[k] && k != a) {
            if m[a * n + k] > meilleure {
                (meilleure, b) = (m[a * n + k], k);
            }
        }
        if Some(b) != precedent {
            chaine.push(b);
            continue;
        }
        chaine.truncate(chaine.len() - 2);
        // Lance et Williams, pour la liaison moyenne : `b` rejoint `a`.
        let (ta, tb) = (taille[a] as f32, taille[b] as f32);
        for k in (0..n).filter(|&k| actif[k] && k != a && k != b) {
            let r = (ta * m[a * n + k] + tb * m[b * n + k]) / (ta + tb);
            m[a * n + k] = r;
            m[k * n + a] = r;
        }
        actif[b] = false;
        taille[a] += taille[b];
        restants -= 1;
        // La liaison moyenne est réductible : un nœud de l'arbre est toujours moins haut que ses
        // enfants. Unir ce qui fusionne au-dessus du seuil, dans n'importe quel ordre, coupe l'arbre.
        if meilleure > seuil {
            let (ra, rb) = (racine(&mut parent, a), racine(&mut parent, b));
            parent[rb] = ra;
        }
    }
    numeroter(&mut parent)
}

/// La racine d'un texte dans l'union de ses familles, le chemin compressé au passage.
fn racine(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// Les familles d'au moins deux textes, numérotées dans l'ordre de leur premier texte.
fn numeroter(parent: &mut [usize]) -> Vec<Option<usize>> {
    let n = parent.len();
    let racines: Vec<usize> = (0..n).map(|i| racine(parent, i)).collect();
    let mut compte = vec![0usize; n];
    for &r in &racines {
        compte[r] += 1;
    }
    let mut numero = vec![None; n];
    let mut suivant = 0;
    racines
        .iter()
        .map(|&r| {
            if compte[r] < 2 {
                return None;
            }
            Some(*numero[r].get_or_insert_with(|| {
                suivant += 1;
                suivant - 1
            }))
        })
        .collect()
}

/// **Les teintes des familles** : la sériation circulaire spectrale de leurs centres, chaque écart
/// proportionnel à la dissemblance de deux voisines.
fn teintes(sens: &[Vec<f32>], famille: &[Option<usize>]) -> Vec<f64> {
    let f = famille.iter().flatten().max().map_or(0, |&m| m + 1);
    if f == 0 {
        return Vec::new();
    }
    let d = sens[0].len();
    let mut centres = vec![vec![0.0f64; d]; f];
    for (v, fam) in sens.iter().zip(famille) {
        if let Some(k) = fam {
            for (c, &x) in centres[*k].iter_mut().zip(v) {
                *c += f64::from(x);
            }
        }
    }
    for c in &mut centres {
        let norme = c.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        c.iter_mut().for_each(|x| *x /= norme);
    }
    let w = |i: usize, j: usize| -> f64 {
        if i == j {
            0.0
        } else {
            centres[i]
                .iter()
                .zip(&centres[j])
                .map(|(a, b)| a * b)
                .sum::<f64>()
                .max(0.0)
        }
    };
    let ordre = ordre_circulaire(f, &w);
    let pas: Vec<f64> = (0..f)
        .map(|i| 1.0 - w(ordre[i], ordre[(i + 1) % f]).min(1.0))
        .collect();
    let total: f64 = pas.iter().sum();
    let mut teintes = vec![0.0; f];
    let mut cumul = 0.0;
    for (i, &k) in ordre.iter().enumerate() {
        teintes[k] = if total > 0.0 {
            std::f64::consts::TAU * cumul / total
        } else {
            std::f64::consts::TAU * i as f64 / f as f64
        };
        cumul += pas[i];
    }
    teintes
}

/// **L'ordre des familles sur le cercle** : l'angle de chacune dans le plan des deux premiers
/// vecteurs propres non triviaux du laplacien normalisé `I − D^{−½} W D^{−½}`.
fn ordre_circulaire(f: usize, w: &dyn Fn(usize, usize) -> f64) -> Vec<usize> {
    if f <= 3 {
        return (0..f).collect();
    }
    let degre: Vec<f64> = (0..f).map(|i| (0..f).map(|j| w(i, j)).sum()).collect();
    let inv = |i: usize| {
        if degre[i] > 0.0 {
            degre[i].powf(-0.5)
        } else {
            0.0
        }
    };
    let mut l = vec![0.0; f * f];
    for i in 0..f {
        for j in 0..f {
            let identite = if i == j { 1.0 } else { 0.0 };
            l[i * f + j] = identite - inv(i) * w(i, j) * inv(j);
        }
    }
    let (valeurs, vecteurs) = jacobi(&mut l, f);
    let mut rangs: Vec<usize> = (0..f).collect();
    rangs.sort_by(|&a, &b| valeurs[a].total_cmp(&valeurs[b]));
    let (u, v) = (rangs[1], rangs[2]);
    let angle = |i: usize| vecteurs[i * f + v].atan2(vecteurs[i * f + u]);
    let mut ordre: Vec<usize> = (0..f).collect();
    ordre.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)).then(a.cmp(&b)));
    ordre
}

/// **Les valeurs et vecteurs propres d'une matrice symétrique**, par les rotations de Jacobi : les
/// valeurs sur la diagonale rendue, les vecteurs en colonnes (`vecteurs[i * n + k]`, la composante
/// `i` du `k`-ième).
pub(super) fn jacobi(a: &mut [f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _ in 0..100 {
        let hors: f64 = (0..n)
            .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| a[i * n + j] * a[i * n + j])
            .sum();
        if hors < 1e-22 {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                tourner(a, &mut v, n, (p, q));
            }
        }
    }
    ((0..n).map(|i| a[i * n + i]).collect(), v)
}

/// Une rotation de Jacobi qui annule `a[p][q]`.
fn tourner(a: &mut [f64], v: &mut [f64], n: usize, (p, q): (usize, usize)) {
    let apq = a[p * n + q];
    if apq.abs() < 1e-300 {
        return;
    }
    let theta = (a[q * n + q] - a[p * n + p]) / (2.0 * apq);
    let signe = if theta >= 0.0 { 1.0 } else { -1.0 };
    let t = signe / (theta.abs() + (theta * theta + 1.0).sqrt());
    let c = 1.0 / (t * t + 1.0).sqrt();
    let s = t * c;
    for k in 0..n {
        let (akp, akq) = (a[k * n + p], a[k * n + q]);
        a[k * n + p] = c * akp - s * akq;
        a[k * n + q] = s * akp + c * akq;
    }
    for k in 0..n {
        let (apk, aqk) = (a[p * n + k], a[q * n + k]);
        a[p * n + k] = c * apk - s * aqk;
        a[q * n + k] = s * apk + c * aqk;
    }
    for k in 0..n {
        let (vkp, vkq) = (v[k * n + p], v[k * n + q]);
        v[k * n + p] = c * vkp - s * vkq;
        v[k * n + q] = s * vkp + c * vkq;
    }
}

#[cfg(test)]
mod tests;
