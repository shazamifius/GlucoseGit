//! **FLECHE-5 — une flèche contourne ce qu'elle traverserait.**
//!
//! # Ce qu'il a demandé
//!
//! *« Tout le système que possède Tauri pour éviter que la flèche traverse une image n'existe
//! pas ; on devrait avoir un algorithme super bien fait pour éviter les zones de texte, les
//! autres images, ou TOUT autre élément de Glucose. »*
//!
//! # Ce que Tauri faisait
//!
//! `getDynamicRoute` (`ArrowSvgLayer.tsx`) : à chaque rendu et pour chaque flèche sans coude, la
//! boîte la plus proche que le segment traverse, contournée par celui de douze chemins de coins
//! qui est le plus court, puis la même chose sur chaque morceau, récursivement — dix niveaux au
//! plus. C'est glouton : le premier obstacle choisi décide du détour, et deux détours voisins
//! s'ignorent. Et chaque flèche parcourt **tous** les nœuds, à chaque image.
//!
//! # Le plus court chemin passe par des coins
//!
//! Parmi des obstacles polygonaux, le plus court chemin entre deux points est une ligne brisée
//! dont chaque sommet intérieur est un sommet d'obstacle (Lozano-Pérez et Wesley, 1979). On le
//! trouve dans le **graphe de visibilité** : ses sommets sont les deux bouts et les coins, ses
//! arêtes les paires qui se voient. Ici, les obstacles sont les boîtes des nœuds gonflées de
//! l'[`ECART`] : le chemin les longe à cette distance. Un A* le parcourt, guidé par la distance à
//! vol d'oiseau, qui ne surestime jamais — le chemin trouvé est donc le plus court.
//!
//! # Un chemin tendu
//!
//! Un plus court chemin ne passe jamais tout droit par un coin, et ne tourne jamais du côté
//! opposé à sa boîte : à chaque coin, il **s'enroule** autour d'elle, comme un fil tendu. Sans
//! cette règle, une flèche qui longe une colonne de deux cents cartes voyait l'A* essayer
//! chacun des deux cents coins alignés, tous presque aussi prometteurs — des millisecondes pour
//! une seule flèche. La transition qui n'est pas tendue ne s'explore pas ; le chemin droit qui
//! la remplace, lui, s'explore depuis le sommet d'avant.
//!
//! # Ne regarder que ce que le chemin touche
//!
//! Le plus court chemin qui évite **une partie** des obstacles, s'il n'en traverse aucun autre,
//! est le plus court qui les évite tous : ajouter un obstacle ne raccourcit jamais un chemin. La
//! recherche part donc du segment droit, demande à l'index ce qu'il traverse, cherche le plus
//! court chemin qui l'évite, demande ce que **ce** chemin traverse — et s'arrête au premier chemin
//! libre. Seul le voisinage du chemin est jamais lu : dix millions de nœuds ailleurs ne coûtent
//! rien, et une flèche que rien ne gêne — presque toutes — coûte une requête le long d'un segment.
//!
//! # Ce qui ne se contourne pas
//!
//! * Un obstacle qui **contient** un bout de la flèche : on ne sort pas d'une boîte où l'on est
//!   déjà. Une carte posée sur une photo se relie à travers elle, sans détour absurde.
//! * Un bout **enfermé** : aucun chemin n'existe, la flèche va droit.
//! * Au-delà du [`BUDGET`] : un dédale n'est pas un détour. La flèche va droit, et son coût reste
//!   borné quoi que porte le tableau.
//!
//! La source et la cible sont des obstacles comme les autres, sauf pour le tronçon qui part de
//! l'une ou arrive à l'autre : une flèche ne traverse pas sa propre carte pour la contourner.

mod grille;

use super::trace;
use crate::geometry::Rect;
use grille::Grille;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Un point du monde.
pub type Point = (f64, f64);

/// **L'écart qu'une flèche garde avec ce qu'elle longe** : celui qui décolle déjà sa pointe du
/// bloc qu'elle vise (`ANCHOR_MARGIN`, la marge de Tauri). Une seule distance dit « une flèche
/// ne colle pas à un bloc », qu'elle y arrive ou qu'elle passe à côté.
pub const ECART: f64 = crate::arrow_anchor::ANCHOR_MARGIN;

/// **Le travail qu'un itinéraire peut coûter**, compté en tests élémentaires — un segment contre
/// une boîte, un coin contre une boîte, une boîte rendue par l'index.
///
/// C'est un budget de temps, pas une distance : environ un million de tests, de l'ordre de la
/// milliseconde sur la machine de mesure (fiche 42), payée une fois — l'itinéraire est retenu
/// tant que le document ne change pas. Un détour qui en demanderait davantage traverse un
/// dédale ; la flèche va droit.
pub const BUDGET: usize = 1 << 20;

/// Ce qui répond « quels obstacles dans cette zone ? » : la boîte de chacun, **non gonflée**.
pub type Requete<'r> = &'r mut dyn FnMut(Rect, &mut Vec<Rect>);

/// Un bout de flèche : le point qu'elle vise, et la boîte du nœud qui le porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bout {
    pub point: Point,
    pub boite: Option<Rect>,
}

/// **Les étapes par lesquelles une flèche de `depart` à `arrivee` passe pour ne rien
/// traverser** — vide si elle va droit.
pub fn itineraire(depart: Bout, arrivee: Bout, requete: Requete) -> Vec<Point> {
    let mut recherche = Recherche::nouvelle(depart, arrivee, requete);
    let mut chemin = vec![depart.point, arrivee.point];
    loop {
        match recherche.decouvrir(&chemin) {
            Some(false) => return chemin[1..chemin.len() - 1].to_vec(),
            Some(true) => {}
            None => return Vec::new(),
        }
        match recherche.a_etoile() {
            Some(nouveau) => chemin = nouveau,
            None => return Vec::new(),
        }
    }
}

/// Une boîte fermée `[x0, x1] × [y0, y1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Boite {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Boite {
    fn de(r: Rect, ecart: f64) -> Self {
        Self {
            x0: r.left - ecart,
            y0: r.top - ecart,
            x1: r.right() + ecart,
            y1: r.bottom() + ecart,
        }
    }

    /// Le bruit du calcul à l'échelle des coordonnées : ce qui longe un bord n'entre pas.
    fn bruit(&self) -> f64 {
        1e-9 * [self.x0, self.y0, self.x1, self.y1, 1.0]
            .iter()
            .fold(0.0_f64, |m, v| m.max(v.abs()))
    }

    /// Le point est-il **strictement** à l'intérieur ?
    fn contient(&self, p: Point) -> bool {
        let e = self.bruit();
        p.0 > self.x0 + e && p.0 < self.x1 - e && p.1 > self.y0 + e && p.1 < self.y1 - e
    }

    /// **Le segment passe-t-il par l'intérieur ?** Longer un bord, toucher un coin n'est pas
    /// traverser : c'est ainsi que le chemin contourne.
    ///
    /// La part du segment dans la boîte fermée (Liang et Barsky) est convexe : ou bien elle
    /// tient sur un bord, et son milieu y est aussi ; ou bien elle passe par l'intérieur, et son
    /// milieu y est. Le milieu décide.
    fn traversee(&self, a: Point, b: Point) -> bool {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        let bords = [
            (-dx, a.0 - self.x0),
            (dx, self.x1 - a.0),
            (-dy, a.1 - self.y0),
            (dy, self.y1 - a.1),
        ];
        for (p, q) in bords {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else if p < 0.0 {
                t0 = t0.max(q / p);
            } else {
                t1 = t1.min(q / p);
            }
        }
        let t = (t0 + t1) / 2.0;
        t0 <= t1 && self.contient((a.0 + dx * t, a.1 + dy * t))
    }

    /// Les quatre coins, chacun avec la direction qui entre dans la boîte depuis lui.
    fn coins(&self) -> [(Point, Point); 4] {
        [
            ((self.x0, self.y0), (1.0, 1.0)),
            ((self.x1, self.y0), (-1.0, 1.0)),
            ((self.x1, self.y1), (-1.0, -1.0)),
            ((self.x0, self.y1), (1.0, -1.0)),
        ]
    }
}

/// **Le chemin `p → u → v` est-il tendu au coin `u` ?** — la boîte du coin étant du côté `q`.
///
/// Il l'est si les deux tronçons longent la boîte sans y entrer (chacun garde la boîte d'un
/// seul côté de sa droite) et si le chemin tourne **vers** elle : il s'y enroule. Passer tout
/// droit n'est pas tendu — le segment `p → v` fait le même chemin sans le coin.
fn tendu(p: Point, u: Point, q: Point, v: Point) -> bool {
    let d1 = (u.0 - p.0, u.1 - p.1);
    if d1 == (0.0, 0.0) {
        // Deux sommets confondus : le premier ne dit rien de la direction.
        return true;
    }
    let d2 = (v.0 - u.0, v.1 - u.1);
    let longe = |d: Point| (d.0 * q.0) * (d.1 * q.1) <= 0.0;
    let croix = |a: Point, b: Point| a.0 * b.1 - a.1 * b.0;
    longe(d1) && longe(d2) && croix(d1, d2) * croix(d1, q) > 0.0
}

fn distance(p: Point, q: Point) -> f64 {
    (q.0 - p.0).hypot(q.1 - p.1)
}

/// La zone d'un segment, élargie de `marge` : ce qu'il faut demander à l'index pour trouver
/// toute boîte que le segment pourrait traverser une fois gonflée.
fn zone_du_segment(p: Point, q: Point, marge: f64) -> Rect {
    Rect::new(
        p.0.min(q.0) - marge,
        p.1.min(q.1) - marge,
        (p.0 - q.0).abs() + 2.0 * marge,
        (p.1 - q.1).abs() + 2.0 * marge,
    )
}

/// Un sommet à explorer, rangé par son estimation — la plus petite d'abord, puis le plus petit
/// rang : à égalité, l'exploration ne dépend que du tableau.
#[derive(PartialEq)]
struct Candidat {
    estimation: f64,
    sommet: usize,
}

impl Eq for Candidat {}

impl Ord for Candidat {
    fn cmp(&self, autre: &Self) -> Ordering {
        autre
            .estimation
            .total_cmp(&self.estimation)
            .then(autre.sommet.cmp(&self.sommet))
    }
}

impl PartialOrd for Candidat {
    fn partial_cmp(&self, autre: &Self) -> Option<Ordering> {
        Some(self.cmp(autre))
    }
}

/// L'état de la recherche : les obstacles connus, et le travail dépensé.
struct Recherche<'r> {
    a: Point,
    b: Point,
    /// Les boîtes gonflées de la source et de la cible — un obstacle pour tout tronçon qui ne
    /// part pas de l'une ou n'arrive pas à l'autre.
    propres: [Option<Boite>; 2],
    connues: Vec<Boite>,
    /// Les obstacles connus rangés par cases, refaite à chaque recherche du plus court chemin.
    grille: Grille,
    requete: Requete<'r>,
    tampon: Vec<Rect>,
    travail: usize,
}

impl<'r> Recherche<'r> {
    fn nouvelle(depart: Bout, arrivee: Bout, requete: Requete<'r>) -> Self {
        let (a, b) = (depart.point, arrivee.point);
        // Une boîte propre qui contient l'autre bout ne peut pas être évitée : on l'oublie.
        let propre = |boite: Option<Rect>, autre: Point| {
            boite
                .map(|r| Boite::de(r, ECART))
                .filter(|o| !o.contient(autre))
        };
        Self {
            a,
            b,
            propres: [propre(depart.boite, b), propre(arrivee.boite, a)],
            connues: Vec::new(),
            grille: Grille::de(&[]),
            requete,
            tampon: Vec::new(),
            travail: 0,
        }
    }

    /// Ajoute aux obstacles connus ceux que ce chemin traverse, et dit s'il y en avait — ou
    /// rien, si le budget est épuisé.
    fn decouvrir(&mut self, chemin: &[Point]) -> Option<bool> {
        let mut trouve = false;
        for troncon in chemin.windows(2) {
            let (p, q) = (troncon[0], troncon[1]);
            self.tampon.clear();
            (self.requete)(zone_du_segment(p, q, ECART), &mut self.tampon);
            self.travail += self.tampon.len() + 1;
            for r in self.tampon.drain(..) {
                let o = Boite::de(r, ECART);
                let a_contourner = !o.contient(self.a) && !o.contient(self.b);
                if a_contourner && !self.connues.contains(&o) && o.traversee(p, q) {
                    self.connues.push(o);
                    trouve = true;
                }
            }
            if self.travail > BUDGET {
                return None;
            }
        }
        Some(trouve)
    }

    /// Les sommets du graphe de visibilité : les deux bouts, puis chaque coin qu'aucune boîte ne
    /// recouvre — un coin enfoui dans une voisine n'est pas un passage —, avec la direction de
    /// sa boîte (aucune pour les deux bouts).
    fn sommets(&mut self) -> Vec<(Point, Option<Point>)> {
        let propres: Vec<Boite> = self.propres.iter().flatten().copied().collect();
        let mut sommets = vec![(self.a, None), (self.b, None)];
        for boite in self.connues.iter().chain(&propres) {
            for (coin, dedans) in boite.coins() {
                self.travail += 1;
                let enfoui = self.grille.contient(&self.connues, coin)
                    || propres.iter().any(|o| o.contient(coin));
                if !enfoui {
                    sommets.push((coin, Some(dedans)));
                }
            }
        }
        sommets
    }

    /// Les sommets `i` et `j` se voient-ils ? Le sommet 0 est le départ, le 1 l'arrivée : un
    /// tronçon qui en part traverse librement sa propre boîte.
    fn se_voient(&mut self, sommets: &[Point], i: usize, j: usize) -> bool {
        let (p, q) = (sommets[i], sommets[j]);
        self.travail += 2;
        if self
            .grille
            .traversee(&self.connues, (p, q), &mut self.travail)
        {
            return false;
        }
        let touche = |k: usize| i == k || j == k;
        let [source, cible] = self.propres;
        let bloque = |propre: Option<Boite>, k: usize| {
            propre.is_some_and(|o| !touche(k) && o.traversee(p, q))
        };
        !bloque(source, 0) && !bloque(cible, 1)
    }

    /// **Le plus court chemin qui évite les obstacles connus** (A*), ou rien s'il n'y en a pas
    /// — ou si le budget s'épuise.
    fn a_etoile(&mut self) -> Option<Vec<Point>> {
        self.grille = Grille::de(&self.connues);
        self.travail += self.connues.len();
        let (sommets, cotes): (Vec<Point>, Vec<Option<Point>>) =
            self.sommets().into_iter().unzip();
        let n = sommets.len();
        let b = self.b;
        let reste = |k: usize| distance(sommets[k], b);
        let mut acquis = vec![f64::INFINITY; n];
        let mut parent = vec![usize::MAX; n];
        let mut clos = vec![false; n];
        let mut ouverts = BinaryHeap::new();
        acquis[0] = 0.0;
        ouverts.push(Candidat {
            estimation: reste(0),
            sommet: 0,
        });
        while let Some(Candidat { sommet: u, .. }) = ouverts.pop() {
            if clos[u] {
                continue;
            }
            if u == 1 {
                let mut chemin = vec![sommets[1]];
                let mut k = 1;
                while k != 0 {
                    k = parent[k];
                    chemin.push(sommets[k]);
                }
                chemin.reverse();
                return Some(chemin);
            }
            clos[u] = true;
            let tendu_en_u = |v: usize| match (cotes[u], parent[u]) {
                (Some(q), p) if p != usize::MAX => tendu(sommets[p], sommets[u], q, sommets[v]),
                _ => true,
            };
            for v in 0..n {
                let par_u = acquis[u] + distance(sommets[u], sommets[v]);
                // La visibilité est le test coûteux : on ne le fait que pour un gain, et que
                // pour un chemin tendu.
                let inutile = clos[v] || par_u >= acquis[v] || !tendu_en_u(v);
                if inutile || !self.se_voient(&sommets, u, v) {
                    continue;
                }
                acquis[v] = par_u;
                parent[v] = u;
                ouverts.push(Candidat {
                    estimation: par_u + reste(v),
                    sommet: v,
                });
            }
            if self.travail > BUDGET {
                return None;
            }
        }
        None
    }
}

/// **Resserre une flèche courbe sur son itinéraire**, là où la courbe entrerait dans un obstacle.
///
/// La courbe de Tauri passe par les étapes en Catmull-Rom : entre deux étapes, elle s'écarte de
/// la ligne brisée — que l'itinéraire garde à l'[`ECART`] de tout — et peut mordre un bloc. Là
/// où elle mord, le tronçon fautif et ses deux voisins se coupent en leur milieu : des étapes
/// plus serrées tendent la courbe vers la ligne brisée, qui ne touche rien. `bouts` rend les deux
/// extrémités de la flèche pour ces étapes — elles sortent du bloc du côté de la première et de
/// la dernière.
///
/// La courbe est vérifiée par sa ligne brisée à la moitié de l'écart près, contre les blocs
/// élargis d'autant : ce qui passe ce test ne touche aucun bloc.
pub fn resserrer(
    etapes: &mut Vec<Point>,
    bouts: &dyn Fn(&[Point]) -> (Point, Point),
    exclus: [Point; 2],
    requete: Requete,
) {
    let marge = ECART / 2.0;
    let mut travail = 0;
    let mut tampon = Vec::new();
    while travail <= BUDGET {
        let (debut, fin) = bouts(etapes);
        let points: Vec<Point> = std::iter::once(debut)
            .chain(etapes.iter().copied())
            .chain(std::iter::once(fin))
            .collect();
        let morceaux = trace::morceaux(&points, true);
        let fautif = morceaux.iter().position(|m| {
            mord(m, (marge, exclus), &mut *requete, (&mut tampon, &mut travail))
        });
        let Some(i) = fautif else {
            return;
        };
        // Du dernier au premier : une insertion ne décale pas les tronçons qui la précèdent.
        for k in [i + 1, i, i.wrapping_sub(1)] {
            if k + 1 < points.len() {
                let (p, q) = (points[k], points[k + 1]);
                etapes.insert(k, ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0));
            }
        }
    }
}

/// Ce morceau de courbe entre-t-il dans un bloc ?
fn mord(
    m: &trace::Morceau,
    (marge, exclus): (f64, [Point; 2]),
    requete: Requete,
    (tampon, travail): (&mut Vec<Rect>, &mut usize),
) -> bool {
    let n = m.troncons(marge);
    let brisee: Vec<Point> = (0..=n).map(|k| m.point(k as f64 / n as f64)).collect();
    let (x0, y0, x1, y1) = brisee.iter().fold(
        (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        |(a, b, c, d), p| (a.min(p.0), b.min(p.1), c.max(p.0), d.max(p.1)),
    );
    tampon.clear();
    requete(zone_du_segment((x0, y0), (x1, y1), marge), tampon);
    *travail += tampon.len() * (n + 1) + 1;
    tampon.iter().any(|r| {
        let o = Boite::de(*r, marge);
        !o.contient(exclus[0])
            && !o.contient(exclus[1])
            && brisee.windows(2).any(|s| o.traversee(s[0], s[1]))
    })
}

#[cfg(test)]
mod tests;
