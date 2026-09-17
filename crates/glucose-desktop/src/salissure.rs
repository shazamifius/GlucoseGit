//! Ce qui a changé depuis la dernière image, et doit donc être redessiné (A.1).
//!
//! # La règle de sûreté, avant tout le reste
//!
//! **On ne réduit le travail que quand on peut le prouver.** Un doute vaut `Tout` : redessiner
//! l'écran entier coûte ce qu'il coûtait hier, tandis qu'oublier une zone laisse à l'écran un
//! morceau d'image périmé — le pire défaut possible, parce qu'il se voit et qu'on ne sait pas
//! d'où il vient.
//!
//! C'est pourquoi le type par défaut est `Tout` et non `Rien`, et pourquoi un appelant qui ne
//! sait pas ce qu'il a changé n'a rien à déclarer : il salit tout, comme avant.
//!
//! # En coordonnées monde, et pas écran
//!
//! Un geste connaît le monde : il déplace un nœud dont le rectangle est en unités du document.
//! La conversion vers l'écran dépend de la vue, qui peut changer entre le moment où la
//! salissure est notée et celui où l'image se dessine — et si la vue change, tout l'écran est
//! sale de toute façon.
//!
//! # Un seul rectangle, et pourquoi c'est assez pour commencer
//!
//! Deux zones éloignées donnent un englobant plus grand que leur somme. Une liste de
//! rectangles serait plus fine, mais elle demande de décider combien on en garde — et ce
//! nombre serait une constante arbitraire, à moins de le dériver du coût relatif d'un pixel
//! repeint et d'un rectangle géré, ce qui suppose de mesurer les deux.
//!
//! On commence donc par l'englobant, qui couvre exactement le cas dominant — **un nœud qu'on
//! déplace** — et la liste viendra si la mesure la réclame, avec son critère mesuré.

use glucose_core::geometry::Rect;
use glucose_core::types::Viewport;

/// Ce qui doit être redessiné à la prochaine image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Salissure {
    /// Rien n'a changé : l'image précédente est encore exacte.
    Rien,
    /// Cette zone du **monde** a changé, et elle seule.
    Zone(Rect),
    /// Quelque chose a changé qu'on ne sait pas localiser — la vue, la fenêtre, le thème — ou
    /// qu'aucun appelant n'a pris la peine de décrire. Tout se redessine.
    Tout,
}

impl Default for Salissure {
    /// `Tout`, et non `Rien` : voir la règle de sûreté en tête de module.
    fn default() -> Self {
        Self::Tout
    }
}

impl Salissure {
    /// N'y a-t-il rien à redessiner ?
    pub fn est_propre(self) -> bool {
        self == Self::Rien
    }

    /// Ajoute une zone du monde à ce qui est déjà sale.
    ///
    /// `Tout` absorbe n'importe quoi — c'est ce qui rend l'opération sûre quel que soit l'ordre
    /// des déclarations : une zone précise ne peut jamais *réduire* une salissure existante.
    pub fn avec(self, zone: Rect) -> Self {
        match self {
            Self::Tout => Self::Tout,
            Self::Rien => Self::Zone(zone),
            Self::Zone(deja) => Self::Zone(englobant(deja, zone)),
        }
    }

    /// Réunit deux salissures.
    pub fn union(self, autre: Self) -> Self {
        match (self, autre) {
            (Self::Tout, _) | (_, Self::Tout) => Self::Tout,
            (Self::Rien, a) | (a, Self::Rien) => a,
            (Self::Zone(a), Self::Zone(b)) => Self::Zone(englobant(a, b)),
        }
    }

    /// La région d'écran à redessiner : `(x, y, largeur, hauteur)` en pixels entiers.
    ///
    /// `None` veut dire « toute la fenêtre » — soit parce que la salissure est `Tout`, soit
    /// parce que la zone en couvre déjà l'essentiel et qu'un rendu partiel coûterait plus cher
    /// que le rendu complet qu'il remplace.
    ///
    /// # La marge n'est pas un réglage
    ///
    /// `marge` est la portée de ce qu'une passe peut dessiner **au-delà** du rectangle d'un
    /// nœud : le flou des halos, essentiellement. `bench_zone` l'a mesurée — l'écart entre un
    /// rendu par région et un rendu complet s'éteint entre seize et quarante-huit pixels, ce
    /// qui est exactement la portée du flou à l'échelle 1. L'appelant la calcule depuis le
    /// modèle de halo et l'échelle courante ; elle ne se choisit nulle part.
    pub fn region(self, vp: &Viewport, fenetre: (u32, u32), marge: f32) -> Option<Region> {
        let Self::Zone(zone) = self else {
            // `Rien` n'a pas de région : l'appelant doit avoir traité ce cas avant, puisqu'il
            // ne dessine alors rien du tout.
            return None;
        };
        let (fw, fh) = (f64::from(fenetre.0), f64::from(fenetre.1));
        let marge = f64::from(marge);

        let x0 = (zone.left * vp.scale + vp.x - marge).floor().max(0.0);
        let y0 = (zone.top * vp.scale + vp.y - marge).floor().max(0.0);
        let x1 = ((zone.left + zone.width) * vp.scale + vp.x + marge)
            .ceil()
            .min(fw);
        let y1 = ((zone.top + zone.height) * vp.scale + vp.y + marge)
            .ceil()
            .min(fh);

        if x1 <= x0 || y1 <= y0 {
            // La zone sale est entièrement hors de la fenêtre : rien à redessiner.
            return Some(Region::VIDE);
        }

        let region = Region {
            x: x0 as u32,
            y: y0 as u32,
            largeur: (x1 - x0) as u32,
            hauteur: (y1 - y0) as u32,
        };
        // Au-delà de la moitié de la fenêtre, le détour ne paie plus : rendre à part puis
        // reporter coûte l'aire deux fois, donc il faut que l'aire épargnée dépasse l'aire
        // reportée. Le seuil est le point où les deux s'égalent — encore un point fixe, et
        // non un nombre choisi.
        if region.aire() as f64 * 2.0 >= fw * fh {
            return None;
        }
        Some(region)
    }
}

/// Une région de la fenêtre, en pixels entiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub largeur: u32,
    pub hauteur: u32,
}

impl Region {
    /// Rien à redessiner — la zone sale est hors de la fenêtre.
    pub const VIDE: Self = Self {
        x: 0,
        y: 0,
        largeur: 0,
        hauteur: 0,
    };

    pub fn est_vide(self) -> bool {
        self.largeur == 0 || self.hauteur == 0
    }

    pub fn aire(self) -> u64 {
        u64::from(self.largeur) * u64::from(self.hauteur)
    }

    /// Cette région touche-t-elle la bande `[0, hauteur[` en haut de la fenêtre ?
    ///
    /// Sert à savoir si la chrome est concernée : elle se place en coordonnées écran, donc
    /// elle ne peut pas se rendre dans une région décalée. Si la zone sale la touche, il faut
    /// la redessiner entière — et donc redessiner la scène sous elle.
    pub fn touche_le_haut(self, hauteur: f32) -> bool {
        (self.y as f32) < hauteur
    }
}

/// Le plus petit rectangle qui contient les deux.
fn englobant(a: Rect, b: Rect) -> Rect {
    let left = a.left.min(b.left);
    let top = a.top.min(b.top);
    let right = (a.left + a.width).max(b.left + b.width);
    let bottom = (a.top + a.height).max(b.top + b.height);
    Rect {
        left,
        top,
        width: right - left,
        height: bottom - top,
    }
}

#[cfg(test)]
mod tests;
