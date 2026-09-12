//! L'index spatial de l'arène : une grille en tableaux, sans allocation par nœud.
//!
//! # Pourquoi le balayage linéaire ne suffit pas
//!
//! Mesuré par `bench_arena` : trouver ce qui croise l'écran sur dix millions de nœuds coûte
//! **8,9 ms** en balayant l'arène. C'est sous le budget de 10 ms par frame — et ça ne laisse
//! rien pour dessiner. La cause n'est pas le calcul mais la mémoire : cinq tableaux de dix
//! millions d'entrées, soit 170 Mo à lire, à la vitesse où la machine sait les lire.
//!
//! Un écran ne montre jamais dix millions de nœuds. L'index existe pour ne lire que ceux-là.
//!
//! # La taille de cellule ne se règle pas : elle se déduit
//!
//! Une grille a besoin d'une taille de cellule, et c'est d'ordinaire une constante qu'on ajuste
//! à la main jusqu'à ce que « ça aille ». Ici elle est **calculée à la construction**, à partir
//! de deux contraintes que les données portent elles-mêmes :
//!
//! ```text
//!     densité : une cellule contient en moyenne un nœud   →   c ≥ √(W × H / n)
//!     taille  : presque tout nœud tient dans une cellule  →   c ≥ 99e centile de max(w, h)
//! ```
//!
//! La cellule est la plus petite puissance de deux qui satisfait les deux.
//!
//! > **La première version n'avait que la première contrainte**, et deux tests l'ont démentie
//! > tout de suite : sur un amas dense, la cellule déduite de la densité était plus petite que
//! > les nœuds eux-mêmes, et *tout le document* se retrouvait dans la liste des grands. Sur un
//! > document dont les nœuds sont tous au même point, la cellule tombait à un 256e de pixel.
//! > Une règle qui ne regarde que les positions ne peut pas décider d'une taille.
//!
//! Le résultat règle trois problèmes d'un coup :
//!
//! - **La densité** : ni des cellules vides par millions, ni des cellules qui contiennent la
//!   moitié du document.
//! - **Le remplissage de la liste des grands**, borné à 1 % des nœuds par construction — c'est
//!   ce que « 99e centile » veut dire, et [`Grid::large`] le donne pour vérification.
//! - **La taille du tableau de cellules**, qui est le piège des grilles denses. Une cellule
//!   fixée à 512 px sur un document occupant tout le domaine demanderait 10⁹ cellules, soit
//!   4 Go rien que pour l'index. Ici le nombre de cellules suit le nombre de nœuds, quelle que
//!   soit l'étendue du document : deux nœuds aux antipodes donnent une grille de deux cellules.
//!
//! Le centile reste une décision — la seule du module. Une grille **hiérarchique**, où chaque
//! nœud irait dans la grille dont la cellule est juste plus grande que lui, la ferait
//! disparaître tout à fait et n'aurait aucun « grand ». Elle coûte un index par niveau et des
//! cellules creuses à parcourir par recherche binaire. Ce qui décidera n'est pas ce texte mais
//! le banc : tant que les grands se comptent en milliers et que la requête tient son budget, la
//! grille simple est la bonne réponse.
//!
//! # Un nœud, une cellule — et les grands à part
//!
//! Chaque nœud est inscrit **une seule fois**, dans la cellule de son coin haut-gauche. Aucune
//! duplication : l'index pèse 4 octets par nœud, pas 8 ou 16 comme une grille qui inscrirait
//! chaque nœud dans toutes les cellules qu'il touche.
//!
//! La contrepartie est qu'un nœud déborde de sa cellule. Tant qu'il tient dans *une* cellule,
//! il suffit d'élargir la requête d'une cellule vers la gauche et vers le haut — et c'est
//! exact, sans marge arbitraire. Les nœuds trop grands pour une cellule — une membrane qui
//! couvre un quartier — vont dans une liste à part, balayée en entier à chaque requête.
//!
//! Cette liste est le point faible de la structure, alors elle est mesurée et non supposée :
//! [`Grid::large`] dit combien il y en a, et le banc dit ce qu'ils coûtent. S'ils devenaient
//! nombreux, c'est une grille hiérarchique qu'il faudrait, et la mesure le dira avant.
//!
//! # Ce que l'index n'est pas
//!
//! Un filtre, pas un oracle : [`Grid::query`] revérifie la boîte exacte de chaque candidat. Le
//! test `test_la_grille_rend_exactement_ce_que_rend_le_balayage` le compare au balayage sur des
//! milliers de vues — l'index doit rendre le même résultat, jamais « à peu près ».

use super::{Arena, Box2, NodeId};
use crate::fixed::Fx;

/// Une grille de cellules carrées, en tableaux contigus.
///
/// `starts` a une entrée de plus que le nombre de cellules : les nœuds de la cellule `c` sont
/// `items[starts[c]..starts[c + 1]]`. C'est la disposition d'une matrice creuse compressée —
/// aucune allocation par cellule, aucune indirection.
#[derive(Debug, Clone)]
pub struct Grid {
    /// Le côté d'une cellule est `1 << shift` unités de coordonnée.
    shift: u32,
    min_i: i32,
    min_j: i32,
    cols: u32,
    rows: u32,
    starts: Vec<u32>,
    items: Vec<NodeId>,
    large: Vec<NodeId>,
}

impl Default for Grid {
    /// Une grille sans cellule — mais avec sa borne : l'invariant GRD-1 veut une borne de plus
    /// que de cellules, et il ne souffre pas d'exception pour le cas vide.
    fn default() -> Self {
        Self {
            shift: 0,
            min_i: 0,
            min_j: 0,
            cols: 0,
            rows: 0,
            starts: vec![0],
            items: Vec::new(),
            large: Vec::new(),
        }
    }
}

impl Grid {
    /// Construit l'index d'une arène. Coût linéaire : deux passes de comptage, aucune
    /// comparaison, aucun tri.
    pub fn build(a: &Arena) -> Self {
        let Some((min, max)) = bounds(a) else {
            return Self::default();
        };
        let vivants = a.len().max(1);
        let shift = deduce_shift(
            max.0.raw() as i64 - min.0.raw() as i64,
            max.1.raw() as i64 - min.1.raw() as i64,
            vivants,
            &size_histogram(a),
        );
        let side = 1i64 << shift;
        // Le nombre de colonnes se compte en CELLULES, pas en unités : deux coordonnées
        // distantes d'une demi-cellule peuvent tomber de part et d'autre d'une frontière, et
        // une division de leur écart l'ignorerait.
        let (min_i, min_j) = (cell_of(min.0, shift), cell_of(min.1, shift));
        let cols = (cell_of(max.0, shift) - min_i + 1) as u32;
        let rows = (cell_of(max.1, shift) - min_j + 1) as u32;

        let mut g = Self {
            shift,
            min_i,
            min_j,
            cols,
            rows,
            starts: vec![0; cols as usize * rows as usize + 1],
            items: Vec::with_capacity(vivants),
            large: Vec::new(),
        };

        // Passe 1 : compter, et mettre de côté ce qui ne tient pas dans une cellule.
        for id in a.iter_alive() {
            let b = a.box_of(id).expect("iter_alive ne rend que des vivants");
            if b.w.raw() as i64 > side || b.h.raw() as i64 > side {
                g.large.push(id);
                continue;
            }
            let c = g.cell_index(b.x, b.y);
            g.starts[c + 1] += 1;
        }
        // Passe 2 : somme préfixe, puis placement.
        for c in 1..g.starts.len() {
            g.starts[c] += g.starts[c - 1];
        }
        g.items.resize(*g.starts.last().unwrap_or(&0) as usize, NodeId::NONE);
        let mut curseur = g.starts.clone();
        for id in a.iter_alive() {
            let b = a.box_of(id).expect("iter_alive ne rend que des vivants");
            if b.w.raw() as i64 > side || b.h.raw() as i64 > side {
                continue;
            }
            let c = g.cell_index(b.x, b.y);
            g.items[curseur[c] as usize] = id;
            curseur[c] += 1;
        }
        g
    }

    /// Le côté d'une cellule, en pixels monde.
    pub fn cell_size(&self) -> f64 {
        Fx::from_raw(1 << self.shift.min(30)).to_f64()
    }

    /// Le nombre de cellules.
    pub fn cells(&self) -> usize {
        self.cols as usize * self.rows as usize
    }

    /// Le nombre de nœuds trop grands pour une cellule, balayés à chaque requête.
    pub fn large(&self) -> usize {
        self.large.len()
    }

    /// Le nombre de nœuds rangés dans des cellules.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Vrai si l'index ne contient aucun nœud.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.large.is_empty()
    }

    /// Les octets occupés par l'index.
    pub fn bytes(&self) -> usize {
        self.starts.len() * 4 + (self.items.len() + self.large.len()) * 4
    }

    /// La colonne ou la ligne d'une coordonnée.
    fn cell_of(v: Fx, shift: u32) -> i32 {
        cell_of(v, shift)
    }

    /// Le rang d'une cellule dans `starts`, rabattu dans la grille.
    fn cell_index(&self, x: Fx, y: Fx) -> usize {
        let i = (Self::cell_of(x, self.shift) - self.min_i).clamp(0, self.cols as i32 - 1);
        let j = (Self::cell_of(y, self.shift) - self.min_j).clamp(0, self.rows as i32 - 1);
        j as usize * self.cols as usize + i as usize
    }

    /// Les nœuds vivants dont la boîte croise `view`, dans l'ordre des cellules puis des
    /// identifiants.
    ///
    /// La boîte exacte de chaque candidat est revérifiée : l'index filtre, il ne décide pas.
    pub fn query(&self, a: &Arena, view: Box2, out: &mut Vec<NodeId>) {
        out.clear();
        for &id in &self.large {
            if a.box_of(id).is_some_and(|b| b.overlaps(view)) {
                out.push(id);
            }
        }
        if self.cells() == 0 {
            return;
        }
        // Un nœud inscrit en (i, j) tient dans une cellule : il ne peut déborder que vers la
        // droite et vers le bas. Élargir la recherche d'une cellule vers la gauche et vers le
        // haut suffit donc, et c'est exact — aucune marge choisie.
        let i0 = (Self::cell_of(view.x, self.shift) - self.min_i - 1).clamp(0, self.cols as i32 - 1);
        let j0 = (Self::cell_of(view.y, self.shift) - self.min_j - 1).clamp(0, self.rows as i32 - 1);
        let i1 = (Self::cell_of(view.right(), self.shift) - self.min_i).clamp(0, self.cols as i32 - 1);
        let j1 = (Self::cell_of(view.bottom(), self.shift) - self.min_j).clamp(0, self.rows as i32 - 1);

        for j in j0..=j1 {
            let base = j as usize * self.cols as usize;
            // Les cellules d'une même ligne sont contiguës dans `items` : une ligne entière se
            // lit d'un seul tenant, sans revenir dans `starts` à chaque colonne.
            let debut = self.starts[base + i0 as usize] as usize;
            let fin = self.starts[base + i1 as usize + 1] as usize;
            for &id in &self.items[debut..fin] {
                if a.box_of(id).is_some_and(|b| b.overlaps(view)) {
                    out.push(id);
                }
            }
        }
    }

    /// Vérifie l'invariant GRD-1 : les bornes de cellules sont croissantes, couvrent exactement
    /// `items`, et chaque nœud est rangé dans la cellule de son coin.
    pub fn check(&self, a: &Arena) -> Result<(), String> {
        if self.starts.len() != self.cells() + 1 {
            return Err(format!(
                "GRD-1 : {} bornes pour {} cellules",
                self.starts.len(),
                self.cells()
            ));
        }
        for c in 1..self.starts.len() {
            if self.starts[c] < self.starts[c - 1] {
                return Err(format!("GRD-1 : les bornes reculent en {c}"));
            }
        }
        if *self.starts.last().unwrap_or(&0) as usize != self.items.len() {
            return Err(format!(
                "GRD-1 : la dernière borne vaut {:?} pour {} entrées",
                self.starts.last(),
                self.items.len()
            ));
        }
        for c in 0..self.cells() {
            for &id in &self.items[self.starts[c] as usize..self.starts[c + 1] as usize] {
                let b = a
                    .box_of(id)
                    .ok_or_else(|| format!("GRD-1 : le nœud {} de la cellule {c} n'est pas vivant", id.index()))?;
                if self.cell_index(b.x, b.y) != c {
                    return Err(format!(
                        "GRD-1 : le nœud {} est rangé en {c} et appartient à {}",
                        id.index(),
                        self.cell_index(b.x, b.y)
                    ));
                }
            }
        }
        Ok(())
    }
}

/// La colonne ou la ligne d'une coordonnée, pour un côté de cellule de `1 << shift` unités.
///
/// Un décalage arithmétique, pas une division : il arrondit vers le bas y compris pour les
/// coordonnées négatives, ce qu'une division entière ne fait pas.
fn cell_of(v: Fx, shift: u32) -> i32 {
    v.raw() >> shift.min(31)
}

/// Le nombre de bits d'une taille : le plus petit `L` tel que `2^L >= taille`.
///
/// Exact et sans flottant — 256 donne 8, et 257 donne 9.
fn size_bits(taille: i32) -> u32 {
    if taille <= 1 {
        return 0;
    }
    32 - (taille - 1).leading_zeros()
}

/// Combien de nœuds ont une taille de chaque puissance de deux.
///
/// Un histogramme plutôt qu'un tri : le centile d'une puissance de deux se lit en une passe
/// linéaire, et c'est exactement la précision dont la cellule a besoin, puisqu'elle est
/// elle-même une puissance de deux.
fn size_histogram(a: &Arena) -> [usize; 33] {
    let mut hist = [0usize; 33];
    for id in a.iter_alive() {
        if let Some(b) = a.box_of(id) {
            hist[size_bits(b.w.raw().max(b.h.raw())) as usize] += 1;
        }
    }
    hist
}

/// La part des nœuds qu'on accepte de voir déborder de leur cellule, et qui iront donc dans la
/// liste des grands : un centième.
///
/// C'est la seule décision du module. Voir l'en-tête pour ce qui la ferait disparaître.
const PART_DES_GRANDS: f64 = 0.01;

/// Le côté de cellule, en bits, satisfaisant les deux contraintes de l'en-tête.
///
/// Les cas dégénérés sont couverts par la seconde : un document ponctuel a une étendue nulle,
/// donc aucune contrainte de densité, mais ses nœuds ont une taille — et c'est elle qui décide.
/// La grille n'a alors qu'une cellule, et la balayer est le balayage du document, ce qui est
/// exactement ce qu'il faut faire quand tout est au même endroit.
fn deduce_shift(w: i64, h: i64, n: usize, hist: &[usize; 33]) -> u32 {
    let aire = (w.max(1) as f64) * (h.max(1) as f64) / n as f64;
    let par_densite = (aire.sqrt().max(1.0).log2().ceil() as i64).clamp(0, 31) as u32;

    // Le plus petit côté qui laisse au plus PART_DES_GRANDS des nœuds déborder.
    let toleres = (n as f64 * PART_DES_GRANDS) as usize;
    let mut au_dessus: usize = hist.iter().sum();
    let mut par_taille = 0u32;
    for (bits, compte) in hist.iter().enumerate() {
        au_dessus -= compte;
        if au_dessus <= toleres {
            par_taille = bits as u32;
            break;
        }
    }
    par_densite.max(par_taille).min(31)
}

/// Les coins du plus petit rectangle contenant tous les nœuds vivants.
fn bounds(a: &Arena) -> Option<((Fx, Fx), (Fx, Fx))> {
    let mut min = (Fx::MAX, Fx::MAX);
    let mut max = (Fx::MIN, Fx::MIN);
    let mut vu = false;
    for id in a.iter_alive() {
        let Some(b) = a.box_of(id) else { continue };
        vu = true;
        min = (min.0.min(b.x), min.1.min(b.y));
        max = (max.0.max(b.x), max.1.max(b.y));
    }
    vu.then_some((min, max))
}

#[cfg(test)]
mod tests;
