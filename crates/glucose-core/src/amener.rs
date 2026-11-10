//! **LIEN-3 — amener près d'un nœud tout ce qui lui est lié** (fiche 102) : la disposition de Bring
//! & Go (Moscovich, Chevalier, Henry, Pietriga et Fekete, *Topology-Aware Navigation in Large
//! Networks*, CHI 2009).
//!
//! Sa parole du 9/11 : *« une fenêtre temporaire qui rassemble le nœud et tous ses liens, les blocs
//! de texte placés à côté, automatiquement, même s'ils sont à des kilomètres »*. L'article : les
//! voisins *« sont placés sur des cercles concentriques autour du nœud choisi […] triés par distance,
//! le plus proche d'abord ; chacun cherche, depuis le cercle intérieur, le premier où il se pose
//! sans chevaucher ceux déjà posés, en gardant son angle »* — ce qui **garde la direction de sa vraie
//! place**, et donne le sens de sa distance relative.
//!
//! # La loi, sans cercles à espacer
//!
//! Des cercles concentriques demandent un pas entre deux cercles : une constante. Ici chaque voisin
//! glisse **sur la demi-droite** qui part du centre vers sa vraie place, et s'arrête **au plus près du
//! centre où il ne chevauche rien** de ce qui est déjà posé (le centre compris), un air donné entre
//! deux boîtes. C'est la limite continue des cercles de l'article : la direction exacte, le plus
//! proche d'abord, et aucun pas à choisir.
//!
//! Le calcul est exact. Une boîte de taille `w × h` qui glisse le long de la demi-droite `c + r·u`
//! chevauche une boîte posée `P` exactement quand son centre est dans `P` élargie de `w/2 + air` et
//! `h/2 + air` (la somme de Minkowski de deux rectangles) : la demi-droite y entre et en sort en deux
//! valeurs de `r` (la méthode des tranches). Le premier `r ≥ 0` hors de tous ces intervalles est la
//! place.

use crate::geometry::Rect;

/// **Un voisin à amener** : sa boîte telle qu'elle est, le point de cette boîte qui glisse sur la
/// demi-droite (son nœud, quand la boîte l'entoure d'une membrane dont le titre est au-dessus), et
/// la vraie place de son nœud, qui donne la direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voisin {
    pub boite: Rect,
    pub accroche: (f64, f64),
    pub vraie_place: (f64, f64),
}

/// **Ce que chaque boîte amenée devient** : sa place, dans l'ordre des voisins donnés.
///
/// `centre` est la boîte de ce qu'on montre, et `origine` le point d'où partent les demi-droites
/// (son nœud) ; `air`, l'écart qui sépare deux boîtes posées. Chaque voisin glisse sur la
/// demi-droite qui part de l'origine vers sa vraie place, son point d'accroche dessus.
pub fn disposer(centre: Rect, origine: (f64, f64), voisins: &[Voisin], air: f64) -> Vec<Rect> {
    let c = origine;
    let mut ordre: Vec<usize> = (0..voisins.len()).collect();
    // Le plus proche d'abord ; à distance égale, l'ordre donné — la loi est déterministe.
    ordre.sort_by(|&a, &b| {
        distance(c, voisins[a].vraie_place)
            .total_cmp(&distance(c, voisins[b].vraie_place))
            .then(a.cmp(&b))
    });
    let mut poses = vec![centre];
    let mut places = vec![Rect::ZERO; voisins.len()];
    for i in ordre {
        let v = voisins[i];
        let u = direction(c, v.vraie_place);
        // Ce qui dépasse de la boîte de part et d'autre du point d'accroche.
        let avant = (v.accroche.0 - v.boite.left, v.accroche.1 - v.boite.top);
        let r = premier_libre(c, u, (v.boite, avant), &poses, air);
        let place = Rect::new(
            c.0 + r * u.0 - avant.0,
            c.1 + r * u.1 - avant.1,
            v.boite.width,
            v.boite.height,
        );
        poses.push(place);
        places[i] = place;
    }
    places
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

/// La direction de `a` vers `b` ; un voisin posé exactement au centre part vers la droite — il faut
/// une direction, et aucune n'est plus vraie qu'une autre.
fn direction(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let d = distance(a, b);
    if d > 0.0 && d.is_finite() {
        ((b.0 - a.0) / d, (b.1 - a.1) / d)
    } else {
        (1.0, 0.0)
    }
}

/// **Le premier `r ≥ 0` où une boîte dont le point d'accroche est en `c + r·u` ne chevauche
/// aucune boîte posée** (l'air compris). La boîte s'étend de `avant` à gauche et en haut de son
/// point d'accroche, du reste à droite et en bas.
fn premier_libre(
    c: (f64, f64),
    u: (f64, f64),
    (boite, avant): (Rect, (f64, f64)),
    poses: &[Rect],
    air: f64,
) -> f64 {
    let apres = (boite.width - avant.0, boite.height - avant.1);
    let mut interdits: Vec<(f64, f64)> = poses
        .iter()
        .filter_map(|p| {
            // Les points d'accroche où la boîte chevaucherait `p` : `p` élargie de l'air, et de
            // ce qui dépasse de la boîte de chaque côté (la somme de Minkowski).
            let zone = Rect::new(
                p.left - air - apres.0,
                p.top - air - apres.1,
                p.width + 2.0 * air + boite.width,
                p.height + 2.0 * air + boite.height,
            );
            traversee(c, u, zone)
        })
        .collect();
    interdits.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut r = 0.0_f64;
    for (entree, sortie) in interdits {
        // Un intervalle ouvert : toucher son bord, c'est être à l'air exact, et c'est permis.
        if entree < r && r < sortie {
            r = sortie;
        } else if entree >= r {
            // Les intervalles suivants commencent plus loin : `r` est libre.
            break;
        }
    }
    r
}

/// **Les `r` où la demi-droite `c + r·u` est à l'intérieur de `m`** — un intervalle ouvert, ou
/// rien. La méthode des tranches : l'intersection des intervalles de chaque axe.
fn traversee(c: (f64, f64), u: (f64, f64), m: Rect) -> Option<(f64, f64)> {
    let mut bas = f64::NEG_INFINITY;
    let mut haut = f64::INFINITY;
    for (o, d, min, max) in [
        (c.0, u.0, m.left, m.left + m.width),
        (c.1, u.1, m.top, m.top + m.height),
    ] {
        if d == 0.0 {
            if o <= min || o >= max {
                return None;
            }
        } else {
            let (t1, t2) = ((min - o) / d, (max - o) / d);
            bas = bas.max(t1.min(t2));
            haut = haut.min(t1.max(t2));
        }
    }
    (bas < haut).then_some((bas, haut))
}

#[cfg(test)]
mod tests;
