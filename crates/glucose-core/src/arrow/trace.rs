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
//! regardent les voisins : `p₁ + (p₂ − p₀)/6` et `p₂ − (p₃ − p₁)/6`. Aux deux bouts, le
//! voisin manquant est le **reflet** du point intérieur (`2·p₀ − p₁`), ce qui fait partir et
//! arriver la courbe dans l'axe de son premier et de son dernier tronçon. Ce sont les
//! nombres de Tauri : une flèche de ses documents se dessine ici à l'identique.
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
    (0..n - 1)
        .map(|i| {
            let i = i as isize;
            let (p0, p1, p2, p3) = (voisin(i - 1), voisin(i), voisin(i + 1), voisin(i + 2));
            Morceau::Courbe {
                de: p1,
                c1: (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0),
                c2: (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0),
                a: p2,
            }
        })
        .collect()
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
