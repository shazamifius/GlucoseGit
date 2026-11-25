//! **Le tracé d'une flèche** : les morceaux qu'elle dessine, droits ou courbes (FLECHE-1).
//!
//! # Une seule description, pour l'œil et pour la main
//!
//! Une flèche « courbe » de Glucose Tauri passe par ses points avec une spline de
//! Catmull-Rom, écrite en cubiques de Bézier (`ArrowSvgLayer.tsx`). Mais Tauri la **vise**
//! le long de sa ligne brisée : sur une courbe ample, on clique à côté de ce qu'on voit.
//! Ici, le dessin et le clic lisent les mêmes [`Morceau`]s — la loi L4, appliquée à la
//! courbe.
//!
//! # La spline
//!
//! Chaque morceau va d'un point `p₁` au suivant `p₂`, et ses deux points de contrôle
//! regardent les voisins. Aux deux bouts, le voisin manquant est le **reflet** du point
//! intérieur (`2·p₀ − p₁`), ce qui fait partir et arriver la courbe dans l'axe de son premier
//! et de son dernier tronçon.
//!
//! Tauri donnait à chaque tronçon **la même durée** — la spline « uniforme » : `p₁ + (p₂ − p₀)/6`
//! et `p₂ − (p₃ − p₁)/6`. Elle ne tient que si les points sont à peu près également espacés. Le
//! contournement (FLECHE-5) en pose de très inégaux : à la sortie de sa carte, un tronçon de
//! 38 unités jusqu'au coin du nœud du dessous, puis un de 1 950. La tangente au coin, la moitié
//! du grand, tirait le petit en une boucle qui remontait à travers le texte de sa source (son
//! nœud « URGENT IMPORTANT », 25/11 ; fiche 152). Ici, chaque tronçon dure **la racine de sa
//! longueur** — la paramétrisation **centripète**, la seule de la famille sans rebroussement
//! ni boucle dans un tronçon (Yuksel, Schaefer et Keyser, *Parameterization and applications of
//! Catmull–Rom curves*, Computer-Aided Design 43, 2011). Des points également espacés gardent
//! exactement les nombres de Tauri : une flèche de ses documents se dessine ici à l'identique
//! tant qu'elle ne bouclait pas.
//!
//! # Aplatir sans choisir de pas — la formule de Wang
//!
//! Pour la viser, une cubique se remplace par une ligne brisée. Combien de tronçons ? Le
//! théorème de Wang le dit, pour une tolérance donnée : `n = ⌈√(¾ · M / ε)⌉`, où `M` est la
//! plus grande des deux différences secondes des points de contrôle. Au-delà de ce `n`, la
//! ligne brisée ne s'écarte **nulle part** de plus de `ε` de la courbe. Aucun nombre de
//! tronçons n'est donc choisi : il se déduit de la précision voulue, et la précision voulue
//! se déduit de l'écran (un quart de pixel).

/// Un point du monde.
pub type Point = (f64, f64);

/// Un morceau du tracé d'une flèche, en coordonnées du monde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Morceau {
    /// Un segment de droite.
    Droit { de: Point, a: Point },
    /// Une cubique de Bézier : son départ, ses deux points de contrôle, son arrivée.
    Courbe {
        de: Point,
        c1: Point,
        c2: Point,
        a: Point,
    },
}

impl Morceau {
    /// Le point du morceau au paramètre `t` ∈ [0, 1].
    pub fn point(&self, t: f64) -> Point {
        match *self {
            Self::Droit { de, a } => (de.0 + (a.0 - de.0) * t, de.1 + (a.1 - de.1) * t),
            Self::Courbe { de, c1, c2, a } => {
                let u = 1.0 - t;
                let (k0, k1, k2, k3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                (
                    k0 * de.0 + k1 * c1.0 + k2 * c2.0 + k3 * a.0,
                    k0 * de.1 + k1 * c1.1 + k2 * c2.1 + k3 * a.1,
                )
            }
        }
    }

    /// Le milieu du morceau, **sur** le morceau : là où se posent l'étiquette et la poignée
    /// qui insère un coude. Sur un segment, c'est le milieu de Tauri ; sur une courbe, c'est
    /// un point de la courbe, et non le milieu d'une corde qui tomberait à côté du trait.
    pub fn milieu(&self) -> Point {
        self.point(0.5)
    }

    /// Le départ du morceau.
    pub fn depart(&self) -> Point {
        match *self {
            Self::Droit { de, .. } | Self::Courbe { de, .. } => de,
        }
    }

    /// L'arrivée du morceau.
    pub fn arrivee(&self) -> Point {
        match *self {
            Self::Droit { a, .. } | Self::Courbe { a, .. } => a,
        }
    }

    /// Combien de tronçons droits suffisent pour ne s'écarter nulle part de plus de
    /// `tolerance` — la formule de Wang pour une cubique. Un segment n'en demande qu'un.
    pub fn troncons(&self, tolerance: f64) -> usize {
        let Self::Courbe { de, c1, c2, a } = *self else {
            return 1;
        };
        let seconde =
            |p: Point, q: Point, r: Point| (p.0 - 2.0 * q.0 + r.0).hypot(p.1 - 2.0 * q.1 + r.1);
        let m = seconde(de, c1, c2).max(seconde(c1, c2, a));
        // Une tolérance nulle, négative ou `NaN` ne demande rien : un tronçon.
        let mesurable = tolerance > 0.0 && m.is_finite();
        if !mesurable {
            return 1;
        }
        // `0.75 = d (d − 1) / 8` pour le degré d = 3.
        ((0.75 * m / tolerance).sqrt().ceil() as usize).max(1)
    }
}

/// Les morceaux d'une flèche qui passe par ces points, droite ou courbe.
///
/// Deux points font toujours un segment, courbe ou non : il n'y a rien à courber sans
/// voisin. C'est aussi ce que fait Tauri (`!curved || n === 2`).
pub fn morceaux(points: &[Point], courbe: bool) -> Vec<Morceau> {
    if !courbe || points.len() <= 2 {
        return points
            .windows(2)
            .map(|p| Morceau::Droit { de: p[0], a: p[1] })
            .collect();
    }
    let n = points.len();
    let reflet = |p: Point, q: Point| (2.0 * p.0 - q.0, 2.0 * p.1 - q.1);
    let avant = reflet(points[0], points[1]);
    let apres = reflet(points[n - 1], points[n - 2]);
    let voisin = |i: isize| -> Point {
        if i < 0 {
            avant
        } else if i as usize >= n {
            apres
        } else {
            points[i as usize]
        }
    };
    let tangente = |i: isize| tangente(voisin(i - 1), voisin(i), voisin(i + 1));
    (0..n - 1)
        .map(|i| {
            let i = i as isize;
            let (p1, p2) = (voisin(i), voisin(i + 1));
            let (m1, m2) = (tangente(i), tangente(i + 1));
            // Une cubique d'Hermite en Bézier : un tiers de la durée du tronçon.
            let tiers = duree(p1, p2) / 3.0;
            Morceau::Courbe {
                de: p1,
                c1: (p1.0 + m1.0 * tiers, p1.1 + m1.1 * tiers),
                c2: (p2.0 - m2.0 * tiers, p2.1 - m2.1 * tiers),
                a: p2,
            }
        })
        .collect()
}

/// **La durée d'un tronçon** : la racine de sa longueur — la paramétrisation centripète.
fn duree(p: Point, q: Point) -> f64 {
    (q.0 - p.0).hypot(q.1 - p.1).sqrt()
}

/// La vitesse moyenne de `p` à `q` en `duree`. Nulle pour un tronçon nul : c'est sa limite, la
/// corde s'annulant plus vite que la racine de sa longueur — deux points confondus ne comptent
/// pour rien, sans aucun seuil.
fn vitesse(p: Point, q: Point, duree: f64) -> Point {
    if duree == 0.0 {
        return (0.0, 0.0);
    }
    ((q.0 - p.0) / duree, (q.1 - p.1) / duree)
}

/// **La tangente de la spline en `p1`**, entre ses voisins `p0` et `p2` (la pyramide de Barry et
/// Goldman dérivée en `p1`) : les vitesses des deux tronçons, moins celle qui les enjambe. Pour
/// deux tronçons de même durée, c'est `(p2 − p0) / 2` par unité de durée : les nombres de Tauri.
fn tangente(p0: Point, p1: Point, p2: Point) -> Point {
    let (d0, d1) = (duree(p0, p1), duree(p1, p2));
    let (avant, enjambee, apres) = (
        vitesse(p0, p1, d0),
        vitesse(p0, p2, d0 + d1),
        vitesse(p1, p2, d1),
    );
    (
        avant.0 - enjambee.0 + apres.0,
        avant.1 - enjambee.1 + apres.1,
    )
}

/// La ligne brisée qui suit ces morceaux à `tolerance` près — pour viser, jamais pour
/// dessiner : le dessin garde les vraies courbes.
pub fn aplatir(morceaux: &[Morceau], tolerance: f64) -> Vec<Point> {
    let mut points = Vec::new();
    if let Some(premier) = morceaux.first() {
        points.push(premier.depart());
    }
    for m in morceaux {
        let n = m.troncons(tolerance);
        points.extend((1..=n).map(|k| m.point(k as f64 / n as f64)));
    }
    points
}

#[cfg(test)]
mod tests;
