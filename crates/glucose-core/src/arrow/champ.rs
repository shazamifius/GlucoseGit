//! **Le champ d'une flèche** — ce que chaque pixel en reçoit (FLECHE-2).
//!
//! # Pourquoi une loi par pixel
//!
//! Dessinée par le traceur générique de `tiny-skia` — un halo et un trait en dégradé,
//! anticrénelés —, une flèche coûtait **0,43 ms** : quatre-vingts flèches, 34 ms par image au
//! processeur (`bench_fleches`), cinq fois l'ancien trait gris. Rogner l'aspect aurait été une
//! rustine. Une flèche est en réalité un **champ** : en chaque pixel, sa distance au tracé,
//! et sa place le long du dégradé. Ce champ s'évalue en quelques opérations, seulement là où
//! il peut laisser de l'encre, et il s'évalue **de la même façon** au processeur et sur la
//! carte graphique — c'est ce qu'avait fait MEMB-FORME-1 pour les membranes.
//!
//! # La loi
//!
//! Tout est en pixels d'écran. `d` est la distance du centre du pixel au tracé : la plus
//! courte à l'un de ses segments, ce qui décrit exactement un trait aux bouts et aux coudes
//! ronds. La couleur `c` est celle du dégradé à la projection du pixel sur l'axe qui joint
//! les deux bouts (`gradientUnits="userSpaceOnUse"`), bornée aux deux extrémités.
//!
//! Sur du transparent, dans l'ordre :
//!
//! 1. le **halo** : `c` à `αₕ · couverture(d, hₕ)` ;
//! 2. le **trait** : `c` — ou le blanc d'une flèche sélectionnée — à `αₜ · couverture(d, hₜ)` ;
//! 3. chaque **disque** : son fond à `couverture_d_un_plein(r − ρ)`, puis son contour à
//!    `couverture(|r − ρ|, e)`, où `r` est la distance à son centre et `ρ` son rayon.
//!
//! Les couvertures sont celles des membranes : un filtre-boîte d'un pixel, perpendiculaire au
//! bord. Un pixel reçoit donc **une** couleur par flèche, composée une fois : aux coudes, rien
//! n'est couvert deux fois, par construction.

use crate::membrane_forme::{couverture_d_un_plein, couverture_d_un_trait};
use crate::report::Pixel;

/// Une flèche porte au plus deux disques : à son départ et à son arrivée.
pub const DISQUES: usize = 2;

/// **La seule marge du champ** : la demi-largeur du filtre-boîte. Un pixel ne reçoit rien d'un
/// bord à plus d'un demi-pixel de son centre — c'est ce que disent les deux couvertures. Toutes
/// les bornes s'en déduisent, exactement : une marge de prudence de plus masquerait le pixel
/// qu'une borne trop courte oublierait, et l'épreuve ne le verrait pas.
const DEMI_PIXEL: f32 = 0.5;

impl Disque {
    /// Au-delà, ni son fond ni son contour ne touchent un pixel.
    pub fn portee(&self) -> f32 {
        self.rayon + self.demi_contour + DEMI_PIXEL
    }

    /// Le point est-il dans le carré de sa portée ? Hors de ce carré, il est hors de portée.
    fn touche(&self, x: f32, y: f32) -> bool {
        let p = self.portee();
        (x - self.centre[0]).abs() < p && (y - self.centre[1]).abs() < p
    }
}

/// Un disque de la flèche, en pixels d'écran.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Disque {
    pub centre: [f32; 2],
    pub rayon: f32,
    /// La demi-épaisseur de son contour.
    pub demi_contour: f32,
    /// Son fond et son contour, de 0 à 1.
    pub fond: [f32; 3],
    pub contour: [f32; 3],
}

/// Le champ d'une flèche, prêt à peindre : tout en pixels d'écran.
#[derive(Debug, Clone, PartialEq)]
pub struct Champ {
    /// Le tracé : des segments `(ax, ay, bx, by)`, découpés au bord de l'écran.
    pub segments: Vec<[f32; 4]>,
    /// L'axe du dégradé : du départ à l'arrivée de la flèche, `(ax, ay, bx, by)`.
    pub axe: [f32; 4],
    /// Les couleurs du départ, du milieu et de l'arrivée, de 0 à 1.
    pub teintes: [[f32; 3]; 3],
    /// Le halo : sa demi-largeur et son opacité.
    pub halo: [f32; 2],
    /// Le trait : sa demi-largeur et son opacité.
    pub ame: [f32; 2],
    /// Le trait est-il blanc (la flèche est sélectionnée) ?
    pub ame_blanche: bool,
    pub disques: [Option<Disque>; DISQUES],
}

/// La distance d'un point à un segment — à un point, si le segment est réduit à un point.
///
/// Le nuanceur l'écrit dans le même ordre, en `f32`.
pub fn distance_au_segment(s: [f32; 4], x: f32, y: f32) -> f32 {
    let (dx, dy) = (s[2] - s[0], s[3] - s[1]);
    let (px, py) = (x - s[0], y - s[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        ((px * dx + py * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (qx, qy) = (px - dx * t, py - dy * t);
    (qx * qx + qy * qy).sqrt()
}

/// Pose une couleur `c` d'opacité `a` sur un prémultiplié.
fn poser(o: &mut [f32; 4], c: [f32; 3], a: f32) {
    for i in 0..3 {
        o[i] = c[i] * a + o[i] * (1.0 - a);
    }
    o[3] = a + o[3] * (1.0 - a);
}

impl Champ {
    /// La portée d'un segment : au-delà, ni le halo ni le trait ne touchent un pixel.
    pub fn portee(&self) -> f32 {
        self.halo[0].max(self.ame[0]) + DEMI_PIXEL
    }

    /// La couleur du dégradé en ce point.
    pub fn teinte(&self, x: f32, y: f32) -> [f32; 3] {
        let [ax, ay, bx, by] = self.axe;
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 {
            (((x - ax) * dx + (y - ay) * dy) / l2).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let [t0, t1, t2] = self.teintes;
        let (c0, c1, k) = if t < 0.5 {
            (t0, t1, t * 2.0)
        } else {
            (t1, t2, t * 2.0 - 1.0)
        };
        [
            c0[0] + (c1[0] - c0[0]) * k,
            c0[1] + (c1[1] - c0[1]) * k,
            c0[2] + (c1[2] - c0[2]) * k,
        ]
    }

    /// **Ce que la flèche pose en ce point**, en RGBA prémultiplié de 0 à 1.
    pub fn couleur(&self, x: f32, y: f32) -> [f32; 4] {
        let d = self
            .segments
            .iter()
            .map(|&s| distance_au_segment(s, x, y))
            .fold(f32::INFINITY, f32::min);
        self.couleur_a_distance(x, y, d)
    }

    /// La même, la distance au tracé déjà mesurée.
    ///
    /// Une distance prise sur une partie seulement des segments est juste dès que les
    /// segments écartés sont hors de portée : ils n'y couvrent rien, et la couleur est la même,
    /// au bit près.
    fn couleur_a_distance(&self, x: f32, y: f32, d: f32) -> [f32; 4] {
        // **Deux raccourcis exacts.** Hors de portée, une couche couvre zéro, et poser une
        // couleur à zéro laisse le prémultiplié tel quel, au bit près : un pixel qu'aucun
        // segment ni aucun disque n'atteint est transparent sans rien calculer, et un disque
        // lointain ne se mesure pas. C'est ce qui fait qu'une flèche coûte ce qu'elle couvre.
        let proches = self.disques.map(|g| g.filter(|g| g.touche(x, y)));
        // `NaN` n'est pas à portée : la comparaison est écrite pour le dire.
        let a_portee = d < self.portee();
        if !a_portee && proches.iter().all(Option::is_none) {
            return [0.0; 4];
        }
        let c = self.teinte(x, y);
        let mut o = [0.0; 4];
        poser(
            &mut o,
            c,
            self.halo[1] * couverture_d_un_trait(d, self.halo[0]),
        );
        let ame = if self.ame_blanche { [1.0; 3] } else { c };
        poser(
            &mut o,
            ame,
            self.ame[1] * couverture_d_un_trait(d, self.ame[0]),
        );
        for disque in proches.iter().flatten() {
            let (vx, vy) = (x - disque.centre[0], y - disque.centre[1]);
            let r = (vx * vx + vy * vy).sqrt() - disque.rayon;
            poser(&mut o, disque.fond, couverture_d_un_plein(r));
            poser(
                &mut o,
                disque.contour,
                couverture_d_un_trait(r.abs(), disque.demi_contour),
            );
        }
        o
    }

    /// Ce que la flèche peut toucher : `(gauche, haut, droite, bas)`.
    pub fn enveloppe(&self) -> [f32; 4] {
        let mut e = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        let mut couvrir = |x0: f32, y0: f32, x1: f32, y1: f32| {
            e = [e[0].min(x0), e[1].min(y0), e[2].max(x1), e[3].max(y1)];
        };
        let r = self.portee();
        for s in &self.segments {
            couvrir(
                s[0].min(s[2]) - r,
                s[1].min(s[3]) - r,
                s[0].max(s[2]) + r,
                s[1].max(s[3]) + r,
            );
        }
        for d in self.disques.iter().flatten() {
            let r = d.portee();
            couvrir(
                d.centre[0] - r,
                d.centre[1] - r,
                d.centre[0] + r,
                d.centre[1] + r,
            );
        }
        e
    }

    /// **Compose la flèche dans l'image**, et rend vrai si elle y a posé de l'encre.
    ///
    /// L'image est en RGBA prémultiplié, `largeur × hauteur` pixels. Seules les lignes de
    /// l'enveloppe sont visitées, et sur chacune seulement les intervalles où un segment ou
    /// un disque peut porter : une flèche coûte ce qu'elle couvre, pas sa boîte.
    pub fn peindre(&self, image: &mut [Pixel], largeur: u32, hauteur: u32) -> bool {
        let (l, h) = (largeur as usize, hauteur as usize);
        if l == 0 || image.len() != l * h {
            return false;
        }
        // Les lignes dont le centre tombe dans l'enveloppe, et elles seules.
        let e = self.enveloppe();
        let premiere = (e[1] - 0.5).ceil().max(0.0);
        let derniere = ((e[3] - 0.5).floor() + 1.0).min(hauteur as f32);
        if premiere.partial_cmp(&derniere) != Some(std::cmp::Ordering::Less) {
            return false;
        }
        let mut encre = false;
        let mut actifs = Vec::new();
        let mut intervalles = Vec::new();
        // La distance au tracé de chaque pixel de la ligne, que chaque segment n'abaisse que
        // sur son propre intervalle : un pixel n'est mesuré que contre les segments qui peuvent
        // l'atteindre — un ou deux, là où une courbe aplatie en compte des dizaines sur les
        // mêmes lignes.
        let mut distances = vec![f32::INFINITY; l];
        for y in premiere as usize..derniere as usize {
            let yc = y as f32 + 0.5;
            self.relever(yc, &mut actifs, &mut intervalles);
            for &(s, (a, b)) in &actifs {
                for (x, d) in distances
                    .iter_mut()
                    .enumerate()
                    .take(fin(b, l))
                    .skip(debut(a))
                {
                    *d = d.min(distance_au_segment(s, x as f32 + 0.5, yc));
                }
            }
            let ligne = &mut image[y * l..(y + 1) * l];
            for &(a, b) in &intervalles {
                for x in debut(a)..fin(b, l) {
                    let d = std::mem::replace(&mut distances[x], f32::INFINITY);
                    let o = self.couleur_a_distance(x as f32 + 0.5, yc, d);
                    if o[3] > 0.0 {
                        composer(&mut ligne[x], o);
                        encre = true;
                    }
                }
            }
        }
        encre
    }

    /// Les segments proches de la ligne `yc`, et les intervalles de la ligne où quelque chose
    /// peut porter, triés et fondus — de sorte qu'aucun pixel ne soit posé deux fois.
    fn relever(
        &self,
        yc: f32,
        actifs: &mut Vec<([f32; 4], (f32, f32))>,
        intervalles: &mut Vec<(f32, f32)>,
    ) {
        actifs.clear();
        intervalles.clear();
        let r = self.portee();
        for &s in &self.segments {
            if let Some(i) = intervalle_d_un_segment(s, yc, r) {
                actifs.push((s, i));
                intervalles.push(i);
            }
        }
        for d in self.disques.iter().flatten() {
            let r = d.portee();
            let dy = yc - d.centre[1];
            if dy.abs() <= r {
                let demi = (r * r - dy * dy).sqrt();
                intervalles.push((d.centre[0] - demi, d.centre[0] + demi));
            }
        }
        intervalles.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut fondus: Vec<(f32, f32)> = Vec::with_capacity(intervalles.len());
        for &(a, b) in intervalles.iter() {
            match fondus.last_mut() {
                // Deux intervalles disjoints ne partagent aucun centre de pixel : seuls ceux
                // qui se chevauchent se fondent.
                Some(dernier) if a <= dernier.1 => dernier.1 = dernier.1.max(b),
                _ => fondus.push((a, b)),
            }
        }
        *intervalles = fondus;
    }
}

/// Le premier pixel dont le centre tombe à droite de `a`.
fn debut(a: f32) -> usize {
    (a - 0.5).ceil().max(0.0) as usize
}

/// Un pixel après le dernier dont le centre tombe à gauche de `b`, sans dépasser la ligne.
fn fin(b: f32, largeur: usize) -> usize {
    ((b - 0.5).floor() + 1.0).clamp(0.0, largeur as f32) as usize
}

/// Les abscisses de la ligne `yc` qui peuvent être à moins de `r` du segment — un
/// sur-ensemble, calculé sans racine : les paramètres du segment à moins de `r` de la ligne
/// forment un intervalle, et ses deux bouts donnent les abscisses extrêmes, élargies de `r`.
fn intervalle_d_un_segment(s: [f32; 4], yc: f32, r: f32) -> Option<(f32, f32)> {
    let [ax, ay, bx, by] = s;
    if yc < ay.min(by) - r || yc > ay.max(by) + r {
        return None;
    }
    let dy = by - ay;
    let (s0, s1) = if dy == 0.0 {
        (0.0, 1.0)
    } else {
        let (u, v) = ((yc - r - ay) / dy, (yc + r - ay) / dy);
        (u.min(v).clamp(0.0, 1.0), u.max(v).clamp(0.0, 1.0))
    };
    let (x0, x1) = (ax + (bx - ax) * s0, ax + (bx - ax) * s1);
    Some((x0.min(x1) - r, x0.max(x1) + r))
}

/// Compose un prémultiplié sur un pixel : `source + destination × (1 − a)`, en virgule fixe
/// sur seize bits, au plus proche — la loi que la carte applique en mélangeant.
fn composer(pixel: &mut Pixel, o: [f32; 4]) {
    const UN: f32 = 65_536.0;
    let fixe = |v: f32| (v * UN + 0.5) as u32;
    let source = [
        fixe(255.0 * o[0]),
        fixe(255.0 * o[1]),
        fixe(255.0 * o[2]),
        fixe(255.0 * o[3]),
    ];
    let garde = fixe(1.0 - o[3]);
    for (octet, s) in pixel.iter_mut().zip(source) {
        let v = (s + u32::from(*octet) * garde + 0x8000) >> 16;
        *octet = v.min(255) as u8;
    }
}

/// **Découpe des segments au cadre**, en `f64`, avant de les passer en `f32`.
///
/// À ×40, une flèche longue de quelques cartes s'étend sur des centaines de milliers de
/// pixels : en `f32`, ses coordonnées perdraient le dixième de pixel, et le trait
/// tremblerait. Ce qui sort du cadre ne se voit pas ; ce qui reste tient dans l'écran, où le
/// `f32` est exact au millième. L'algorithme est celui de Liang et Barsky.
pub fn decouper(segments: &[[f64; 4]], cadre: [f64; 4]) -> Vec<[f32; 4]> {
    let [gauche, haut, droite, bas] = cadre;
    segments
        .iter()
        .filter_map(|s| {
            let [ax, ay, bx, by] = *s;
            let (dx, dy) = (bx - ax, by - ay);
            let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
            for (p, q) in [
                (-dx, ax - gauche),
                (dx, droite - ax),
                (-dy, ay - haut),
                (dy, bas - ay),
            ] {
                if p == 0.0 {
                    if q < 0.0 {
                        return None;
                    }
                } else {
                    let t = q / p;
                    if p < 0.0 {
                        t0 = t0.max(t);
                    } else {
                        t1 = t1.min(t);
                    }
                }
            }
            (t0 <= t1).then_some([
                (ax + dx * t0) as f32,
                (ay + dy * t0) as f32,
                (ax + dx * t1) as f32,
                (ay + dy * t1) as f32,
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests;
