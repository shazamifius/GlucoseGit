//! **Le chemin d'un vol de caméra** — celui de van Wijk et Nuij, *Smooth and efficient zooming
//! and panning* (IEEE InfoVis 2003).
//!
//! # Ce que l'utilisateur a vu, et pourquoi
//!
//! *« Des fois ça fait des parcours chelous : au lieu de juste zoomer progressivement sur
//! l'image, ça va zoomer très vite et ensuite progressivement aller à l'endroit indiqué — on se
//! retrouve avec un full zoom qui traverse toute la map, qui fait mal aux yeux. »*
//!
//! Le vol amortissait **séparément** le point au centre de l'écran, en unités du monde, et le
//! zoom, en octaves. Les deux convergent au même rythme — mais pas à la même vitesse **à
//! l'écran** : une fois zoomé, la même distance du monde vaut des milliers de pixels, et la
//! carte défilait à toute allure sous l'œil. Il a demandé que le zoom et la translation se
//! rattrapent, et, pour aller loin quand on est déjà proche, de dézoomer puis de rezoomer.
//!
//! # La réponse exacte existe, et c'est celle-ci
//!
//! Van Wijk et Nuij décrivent une vue par son centre `u` et sa largeur `w`, en unités du monde,
//! et mesurent le mouvement **perçu** : glisser d'une largeur de vue et zoomer d'un facteur `e`
//! coûtent des quantités comparables, dans un rapport que règle `ρ`. Le chemin le plus court
//! pour cette mesure a une forme close — des cosinus et des tangentes hyperboliques — et il fait
//! exactement ce que l'utilisateur demandait : zoom et translation avancent ensemble, et quand
//! la destination est loin, il prend de la hauteur, d'autant plus qu'elle est loin, puis
//! redescend. C'est le vol de Google Earth, de Mapbox (`flyTo`) et de d3 (`interpolateZoom`).
//!
//! Avec `r0` et `S` tirés des deux extrémités, pour `s` de `0` à `S` :
//!
//! ```text
//!     w(s) = w0 · cosh(r0) / cosh(ρs + r0)
//!     u(s) = u0 + (u1 − u0) · w0 / (ρ² d) · sinh(ρs) / cosh(ρs + r0)
//! ```
//!
//! La seconde ligne est la forme de l'article **simplifiée** : `cosh(r0)·tanh(x) − sinh(r0)`
//! vaut exactement `sinh(x − r0) / cosh(x)`. La forme d'origine soustrait deux nombres
//! immenses quand on part de très près ; celle-ci n'en soustrait aucun. De même, `r = −asinh(b)`
//! plutôt que `ln(√(b²+1) − b)`, qui perd ses chiffres quand `b` grandit.
//!
//! # Ce que ce module ne fait pas
//!
//! Il donne la **forme** du chemin, parcouru par sa longueur `s`. À quelle allure le parcourir
//! — vite ou lentement, avec quel départ et quelle arrivée — est l'affaire de celui qui vole.

use crate::membrane_focus::ScreenSize;
use crate::types::Viewport;

/// **Le compromis entre zoomer et glisser**, `√2`.
///
/// Ce n'est pas un réglage d'ici : c'est la valeur que l'expérience de van Wijk et Nuij a
/// trouvée préférée de leurs sujets, et celle de d3 (Mapbox prend 1,42). Plus grand, le vol
/// monterait plus haut pour glisser moins ; plus petit, il glisserait davantage à ras.
pub const RHO: f64 = std::f64::consts::SQRT_2;

/// Le chemin entre deux cadrages, et sa longueur perçue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chemin {
    ecran: ScreenSize,
    /// Le centre de départ, en unités du monde.
    centre: (f64, f64),
    /// Le déplacement du centre, du départ à l'arrivée.
    delta: (f64, f64),
    /// La largeur de vue au départ, en unités du monde.
    largeur: f64,
    forme: Forme,
    longueur: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Forme {
    /// Le centre ne bouge pas à l'œil : la largeur change seule, d'un même facteur à chaque
    /// pas — le zoom de la molette.
    Zoom { rapport: f64 },
    /// Le cas général : `r0` et la distance entre les deux centres.
    Courbe { r0: f64, distance: f64 },
}

impl Chemin {
    /// Le chemin de `depart` à `arrivee`, sur cet écran.
    pub fn entre(depart: Viewport, arrivee: Viewport, ecran: ScreenSize) -> Self {
        let (u0, w0) = decomposer(depart, ecran);
        let (u1, w1) = decomposer(arrivee, ecran);
        let delta = (u1.0 - u0.0, u1.1 - u0.1);
        let distance = delta.0.hypot(delta.1);
        // Le centre bouge-t-il à l'œil ? Au plus proche des deux cadrages, un déplacement de
        // moins d'un demi-pixel ne se voit pas — le seuil où un vol se dit arrivé.
        let echelle_max = depart.scale.max(arrivee.scale);
        let (forme, longueur) = if distance * echelle_max < 0.5 {
            let rapport = w1 / w0;
            (Forme::Zoom { rapport }, rapport.ln().abs() / RHO)
        } else {
            let (rho2, rho4) = (RHO * RHO, RHO.powi(4));
            let b0 =
                (w1 * w1 - w0 * w0 + rho4 * distance * distance) / (2.0 * w0 * rho2 * distance);
            let b1 =
                (w1 * w1 - w0 * w0 - rho4 * distance * distance) / (2.0 * w1 * rho2 * distance);
            let (r0, r1) = (-b0.asinh(), -b1.asinh());
            (Forme::Courbe { r0, distance }, (r1 - r0) / RHO)
        };
        Self {
            ecran,
            centre: u0,
            delta,
            largeur: w0,
            forme,
            longueur,
        }
    }

    /// **La longueur perçue du chemin**, `S`.
    ///
    /// Glisser d'une largeur de vue en vaut `ρ` ; zoomer d'un facteur `e`, `1/ρ`. C'est ce qui
    /// permet de voler à une allure constante **pour l'œil**, quel que soit le mélange.
    pub fn longueur(&self) -> f64 {
        self.longueur
    }

    /// Le cadrage à la distance `s` du départ, le long du chemin — `s` ramené dans `[0, S]`.
    pub fn vue_a(&self, s: f64) -> Viewport {
        let s = s.clamp(0.0, self.longueur);
        let (fraction, largeur) = match self.forme {
            Forme::Zoom { rapport } => {
                let t = if self.longueur > 0.0 {
                    s / self.longueur
                } else {
                    1.0
                };
                (t, self.largeur * rapport.powf(t))
            }
            Forme::Courbe { r0, distance } => {
                let x = RHO * s + r0;
                (
                    self.largeur / (RHO * RHO * distance) * (RHO * s).sinh() / x.cosh(),
                    self.largeur * r0.cosh() / x.cosh(),
                )
            }
        };
        let centre = (
            self.centre.0 + self.delta.0 * fraction,
            self.centre.1 + self.delta.1 * fraction,
        );
        composer(centre, largeur, self.ecran)
    }
}

/// Le centre de la vue et sa largeur, en unités du monde.
fn decomposer(vue: Viewport, ecran: ScreenSize) -> ((f64, f64), f64) {
    let centre = (
        (ecran.width / 2.0 - vue.x) / vue.scale,
        (ecran.height / 2.0 - vue.y) / vue.scale,
    );
    (centre, ecran.width / vue.scale)
}

/// L'opération inverse de [`decomposer`].
fn composer(centre: (f64, f64), largeur: f64, ecran: ScreenSize) -> Viewport {
    let scale = ecran.width / largeur;
    Viewport {
        scale,
        x: ecran.width / 2.0 - centre.0 * scale,
        y: ecran.height / 2.0 - centre.1 * scale,
    }
}

#[cfg(test)]
mod tests;
