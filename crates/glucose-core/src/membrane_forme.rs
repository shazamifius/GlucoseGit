//! **La forme d'une membrane : une loi, deux instruments** (MEMB-FORME-1).
//!
//! # Ce que la mesure a montré
//!
//! La chronique du 25/09 (`sortie-chronique-2026-09-25-essai2.txt`), sur un document de cent
//! trente-neuf nœuds : dès qu'une membrane remplit l'écran, le poste `membranes` coûte
//! **23 ms en médiane et 48 au pire**, et la cadence tombe à 26 images par seconde. La marque
//! précédente est `cull` : le poste compte exactement le dessin des membranes, rien d'autre.
//!
//! La cause est arithmétique. Chaque membrane remplissait **trois fois** son rectangle arrondi
//! au processeur — deux halos, puis le fond translucide —, anticrénelés, avant son pointillé.
//! Zoomée jusqu'à remplir l'écran, cela fait près de neuf millions de pixels par image. Et sur
//! la voie graphique, sa seule présence faisait exister la couche du dessous tout entière :
//! quinze mébioctets effacés (`effacer`, 2,4 ms) puis envoyés à la carte (`blit`, 9,7 ms).
//!
//! # La loi
//!
//! Les quatre couches d'une membrane portent **la même teinte**. Composer une teinte sur
//! elle-même ne change que son opacité :
//!
//! ```text
//!     A(x, y) = 1 − Π (1 − αₖ · couvertureₖ(x, y))
//! ```
//!
//! Une membrane n'est donc pas quatre dessins mais **un seul champ scalaire** : sa teinte
//! multipliée par `A`. Et ce champ est **constant** partout où aucune couche n'a de bord —
//! c'est-à-dire sur presque toute sa surface.
//!
//! Chaque couche est un rectangle aux coins **circulaires**, le `rx` du `<rect>` que dessinait
//! Glucose Tauri (`SvgAnnotationLayer.tsx`). Le traceur du bureau (`push_rounded_rect`) en
//! faisait des paraboles — des Bézier quadratiques dont le point de contrôle est le coin —,
//! qui bombaient de six pour cent du rayon au milieu du coin : un accident du traceur, pas une
//! décision, et un écart de dizaines de pixels avec la référence au fort zoom. Toutes les
//! autres formes arrondies de l'application l'empruntent ; il trace maintenant des arcs
//! (ARC-1), et les cartes ont les mêmes coins que les membranes.
//!
//! La couverture d'un pixel est celle d'un **filtre-boîte d'un pixel** posé perpendiculairement
//! au bord, lue sur la distance signée, qui est exacte. Le pointillé se lit sur l'**abscisse
//! curviligne** du point du bord le plus proche ; un tiret a des bouts ronds.
//!
//! # Deux instruments, la même fonction
//!
//! La carte évalue [`Membrane::alpha`] en chaque pixel (`present::membranes_gpu` du bureau).
//! Le processeur ([`peindre`]) découpe chaque ligne en segments : là où aucune couche n'a de
//! bord, `A` est constant et le segment se compose d'une seule couleur ; ailleurs, pixel par
//! pixel. Les deux calculent en `f32`, la même formule dans le même ordre : ce qui les sépare
//! est l'arrondi de l'écriture, et une épreuve le borne.

mod peindre;

pub use peindre::peindre;

use std::f32::consts::FRAC_PI_2;

/// Un rectangle aux coins circulaires, en pixels d'écran : ses quatre bords et son rayon.
///
/// # Pourquoi les bords, et pas un centre et une demi-taille
///
/// La distance signée s'écrit d'ordinaire depuis le centre. Mais zoomé très fort sur le coin
/// d'une grande membrane, son centre est à des millions de pixels : `|p − c| − b` soustrait
/// deux nombres immenses, et en `f32` le bord tremble d'un quart de pixel. Depuis les bords,
/// la soustraction qui compte oppose deux nombres **proches de l'écran** — celui qu'on voit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrondi {
    pub gauche: f32,
    pub haut: f32,
    pub droite: f32,
    pub bas: f32,
    pub rayon: f32,
}

/// Les huit morceaux du bord, dans l'ordre du parcours : il part du haut, juste après le coin
/// haut-gauche, et tourne dans le sens des aiguilles d'une montre — le départ et le sens du
/// traceur d'avant, pour que le pointillé garde sa place.
const MORCEAUX: usize = 8;

impl Arrondi {
    /// Le rectangle `(x, y, largeur, hauteur)` aux coins de `rayon`, borné comme tout rectangle
    /// arrondi : jamais plus que la moitié du plus petit côté.
    pub fn nouveau(x: f32, y: f32, largeur: f32, hauteur: f32, rayon: f32) -> Self {
        let (largeur, hauteur) = (largeur.max(0.0), hauteur.max(0.0));
        Self {
            gauche: x,
            haut: y,
            droite: x + largeur,
            bas: y + hauteur,
            rayon: rayon.min(largeur / 2.0).min(hauteur / 2.0).max(0.0),
        }
    }

    /// **La distance signée du point au bord** : négative dedans, exacte.
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let r = self.rayon;
        let qx = (self.gauche - x).max(x - self.droite) + r;
        let qy = (self.haut - y).max(y - self.bas) + r;
        longueur(qx.max(0.0), qy.max(0.0)) + qx.max(qy).min(0.0) - r
    }

    /// Le même rectangle, son bord repoussé de `d` — rentré si `d` est négatif.
    ///
    /// C'est **exact** : l'ensemble des points à distance au plus `d` du bord d'un rectangle
    /// arrondi est un rectangle arrondi, de rayon `r + d`. Rentré de plus que son rayon, il
    /// devient un rectangle aux coins vifs : son rayon tombe à zéro, et c'est encore exact.
    pub fn dilate(&self, d: f32) -> Self {
        Self {
            gauche: self.gauche - d,
            haut: self.haut - d,
            droite: self.droite + d,
            bas: self.bas + d,
            rayon: (self.rayon + d).max(0.0),
        }
    }

    /// **L'étendue `[x0, x1]` de la forme sur la ligne `y`** — là où la distance est ≤ 0 —, ou
    /// rien si la ligne ne la traverse pas.
    pub fn etendue(&self, y: f32) -> Option<(f32, f32)> {
        if !(y >= self.haut && y <= self.bas) || self.droite < self.gauche {
            return None;
        }
        let r = self.rayon;
        let dy = (self.haut + r - y).max(y - (self.bas - r)).max(0.0);
        let retrait = r - (r * r - dy * dy).max(0.0).sqrt();
        Some((self.gauche + retrait, self.droite - retrait))
    }

    /// **Là où le bord le plus proche est le côté haut ou bas, sur la ligne `y`** : l'étendue
    /// `[x0, x1]` où la distance ne dépend que de la ligne.
    ///
    /// C'est la partie droite du bord, entre les deux coins — rétrécie de ce dont la ligne est
    /// sous le haut du rectangle intérieur, là où un côté vertical devient plus proche.
    pub fn droit_horizontal(&self, y: f32) -> Option<(f32, f32)> {
        let r = self.rayon;
        let retrait = ((self.haut - y).max(y - self.bas) + r).min(0.0);
        let (x0, x1) = (self.gauche + r - retrait, self.droite - r + retrait);
        (x0 <= x1).then_some((x0, x1))
    }

    /// La longueur de chaque morceau du bord, dans l'ordre du parcours.
    fn longueurs(&self) -> [f64; MORCEAUX] {
        let r = f64::from(self.rayon);
        let droit_x = (f64::from(self.droite) - f64::from(self.gauche) - 2.0 * r).max(0.0);
        let droit_y = (f64::from(self.bas) - f64::from(self.haut) - 2.0 * r).max(0.0);
        let arc = r * std::f64::consts::FRAC_PI_2;
        [droit_x, arc, droit_y, arc, droit_x, arc, droit_y, arc]
    }

    /// **Le morceau du bord le plus proche du point, et l'abscisse depuis son début.**
    ///
    /// Les huit régions se lisent sur les mêmes `qx`, `qy` que la distance : un point au-delà
    /// du rectangle intérieur dans les deux sens est face à un coin, sinon face au côté dont il
    /// est le plus près.
    pub fn sur_le_bord(&self, x: f32, y: f32) -> (usize, f32) {
        let r = self.rayon;
        let a_gauche = self.gauche - x > x - self.droite;
        let en_haut = self.haut - y > y - self.bas;
        let qx = (self.gauche - x).max(x - self.droite) + r;
        let qy = (self.haut - y).max(y - self.bas) + r;
        if qx > 0.0 && qy > 0.0 {
            return self.sur_un_coin(x, y, a_gauche, en_haut);
        }
        match (qx > qy, a_gauche, en_haut) {
            (true, false, _) => (2, y - (self.haut + r)),
            (true, true, _) => (6, (self.bas - r) - y),
            (false, _, true) => (0, x - (self.gauche + r)),
            (false, _, false) => (4, (self.droite - r) - x),
        }
    }

    /// L'abscisse sur un arc : le rayon fois l'angle balayé depuis le début du morceau.
    fn sur_un_coin(&self, x: f32, y: f32, a_gauche: bool, en_haut: bool) -> (usize, f32) {
        let r = self.rayon;
        let cx = if a_gauche {
            self.gauche + r
        } else {
            self.droite - r
        };
        let cy = if en_haut { self.haut + r } else { self.bas - r };
        let (vx, vy) = (x - cx, y - cy);
        // `atan2(v · fin, v · début)` : l'angle entre la direction où l'arc commence et le
        // point, dans le sens du parcours.
        let (morceau, angle) = match (a_gauche, en_haut) {
            (false, true) => (1, vx.atan2(-vy)),
            (false, false) => (3, vy.atan2(vx)),
            (true, false) => (5, (-vx).atan2(vy)),
            (true, true) => (7, (-vy).atan2(-vx)),
        };
        (morceau, r * angle.clamp(0.0, FRAC_PI_2))
    }

    /// **Le pointillé de ce bord** : un tiret de `tiret`, autant de vide, depuis le départ.
    ///
    /// Les phases s'accumulent en `f64` et se ramènent dans la période : une abscisse de
    /// plusieurs millions de pixels — un grand cadre vu de très près — garderait sinon en
    /// `f32` une erreur de plusieurs pixels sur la place de chaque tiret.
    pub fn pointille(&self, tiret: f32) -> Pointille {
        let periode = 2.0 * f64::from(tiret);
        let longueurs = self.longueurs();
        let perimetre: f64 = longueurs.iter().sum();
        let mut phases = [0.0; MORCEAUX];
        let mut restes = [0.0; MORCEAUX];
        let mut parcouru = 0.0_f64;
        for (k, longueur) in longueurs.iter().enumerate() {
            phases[k] = parcouru.rem_euclid(periode) as f32;
            restes[k] = (perimetre - parcouru) as f32;
            parcouru += longueur;
        }
        Pointille {
            tiret,
            phases,
            restes,
        }
    }
}

/// Le motif d'un pointillé, prêt à être lu en un point du bord.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointille {
    /// La longueur d'un tiret, et celle du vide qui le suit.
    pub tiret: f32,
    /// Où en est le motif au début de chaque morceau du bord, dans `[0, 2·tiret)`.
    pub phases: [f32; MORCEAUX],
    /// Ce qui reste du périmètre depuis le début de chaque morceau.
    pub restes: [f32; MORCEAUX],
}

impl Pointille {
    /// **La distance, le long du bord, jusqu'au tiret le plus proche** : nulle sur un tiret.
    ///
    /// Au bout du tour, le prochain tiret est le premier — le motif ne tombe pas juste sur le
    /// périmètre, et le dernier tiret se soude au premier au lieu de laisser deux bouts ronds
    /// se chevaucher.
    pub fn ecart(&self, morceau: usize, abscisse: f32) -> f32 {
        let periode = 2.0 * self.tiret;
        let t = reste(self.phases[morceau] + abscisse, periode);
        if t < self.tiret {
            return 0.0;
        }
        let jusqu_au_bout = (self.restes[morceau] - abscisse).max(0.0);
        (t - self.tiret).min(periode - t).min(jusqu_au_bout)
    }
}

/// La longueur d'un vecteur, `√(x² + y²)` — le `length` du nuanceur.
///
/// Et non `hypot`, qui se garde des débordements au prix d'un appel de bibliothèque : ici les
/// longueurs sont des pixels, et leur carré tient en `f32` jusqu'à 10¹⁹ pixels.
fn longueur(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

/// Le reste de `x` modulo `p`, dans `[0, p)` : `x − p·trunc(x / p)`, ramené positif.
///
/// C'est **exactement** le `%` du nuanceur, et non le `fmod` exact de la bibliothèque : les deux
/// voies calculent ainsi la même chose — et celui-ci ne coûte qu'une division, là où `fmod`
/// coûtait le plus cher de toute la loi, pixel après pixel du pointillé.
fn reste(x: f32, p: f32) -> f32 {
    let r = x - p * (x / p).trunc();
    if r < 0.0 {
        r + p
    } else {
        r
    }
}

/// Le trait qui borde une membrane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bord {
    pub forme: Arrondi,
    pub demi_largeur: f32,
    /// Le pointillé, ou `None` pour un trait plein.
    pub pointille: Option<Pointille>,
    /// L'opacité du trait, de 0 à 1.
    pub alpha: f32,
}

/// **Une membrane telle que l'écran la montre** : ses remplissages, son bord, sa teinte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Membrane {
    /// Du dessous vers le dessus : les deux halos, puis le fond. Une opacité nulle est une
    /// couche absente — les halos d'une membrane vue de très loin.
    pub remplissages: [(Arrondi, f32); 3],
    pub bord: Bord,
    /// La teinte, de 0 à 1 par canal.
    pub teinte: [f32; 3],
}

impl Membrane {
    /// **L'opacité de la membrane au point `(x, y)`** — le centre d'un pixel.
    ///
    /// Le produit des transparences de ses couches, dans l'ordre : c'est ce que donnent quatre
    /// compositions successives d'une même teinte, en une seule.
    pub fn alpha(&self, x: f32, y: f32) -> f32 {
        let mut transparence = 1.0;
        for (forme, alpha) in &self.remplissages {
            if *alpha > 0.0 {
                transparence *= 1.0 - alpha * couverture_d_un_plein(forme.distance(x, y));
            }
        }
        transparence *= 1.0 - self.bord.alpha * self.couverture_du_bord(x, y);
        1.0 - transparence
    }

    /// La part du pixel que le trait couvre.
    fn couverture_du_bord(&self, x: f32, y: f32) -> f32 {
        let b = &self.bord;
        let d = b.forme.distance(x, y);
        let ecart = b.pointille.map_or(0.0, |p| {
            let (morceau, abscisse) = b.forme.sur_le_bord(x, y);
            p.ecart(morceau, abscisse)
        });
        couverture_d_un_trait(longueur(d, ecart), b.demi_largeur)
    }

    /// Ce que ses couches peuvent toucher : la plus large, et son demi-pixel d'anticrénelage.
    pub fn enveloppe(&self) -> Arrondi {
        let bord = self.bord.forme.dilate(self.bord.demi_largeur + 0.5);
        self.remplissages
            .iter()
            .filter(|(_, alpha)| *alpha > 0.0)
            .map(|(forme, _)| forme.dilate(0.5))
            .fold(bord, |a, b| Arrondi {
                gauche: a.gauche.min(b.gauche),
                haut: a.haut.min(b.haut),
                droite: a.droite.max(b.droite),
                bas: a.bas.max(b.bas),
                rayon: 0.0,
            })
    }
}

/// La couverture d'un plein, pour un pixel dont le centre est à `d` de son bord : un
/// filtre-boîte d'un pixel, perpendiculaire au bord.
pub fn couverture_d_un_plein(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// La couverture d'un trait de demi-largeur `h`, pour un pixel dont le centre est à `d ≥ 0`
/// de son milieu : le recouvrement de `[d − ½, d + ½]` et de `[−h, h]`.
///
/// Un trait plus fin qu'un pixel ne couvre jamais le pixel entier : sa couverture plafonne à
/// `2h`, ce que le filtre donne sans cas particulier.
pub fn couverture_d_un_trait(d: f32, h: f32) -> f32 {
    ((d + 0.5).min(h) - (d - 0.5).max(-h)).max(0.0)
}

#[cfg(test)]
mod tests;
