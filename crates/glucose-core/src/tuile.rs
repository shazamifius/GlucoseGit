//! TUILE-1 — le découpage du canevas en carrés dyadiques, et l'empreinte de ce qu'ils portent.
//!
//! # Pourquoi les vignettes ne pouvaient pas servir, et pourquoi les tuiles le peuvent
//!
//! Le cache de vignettes gardait, pour chaque nœud, l'image de ce nœud **à la taille et à la
//! phase sous-pixel où il est posé à l'écran**. Sa clé était donc un point d'un espace
//! continu : dès que la vue bougeait d'un millième de pixel, tout périmait. Mesuré sur cinq
//! sessions réelles : **zéro à six pour cent d'utilisation**, et c'est son plafond.
//!
//! Une tuile n'a pas ce défaut, et pour une raison de fond : **sa grille est ancrée au monde,
//! pas à l'écran.** La tuile `(niveau, x, y)` couvre toujours le même rectangle du document, à
//! la même échelle. Déplacer la vue ne change donc rien à son contenu ni à ses pixels — cela
//! change seulement *lesquelles* on regarde. La phase sous-pixel n'existe pas dans le repère
//! de la tuile ; elle réapparaît une seule fois, au moment de poser la grille entière à
//! l'écran, et ne coûte alors qu'un décalage commun.
//!
//! # L'empreinte, et ce qu'elle rend possible
//!
//! Chaque tuile porte l'empreinte de ce qu'elle montre : la liste, dans l'ordre de dessin, de
//! ce qui la traverse, **en coordonnées relatives à son propre coin**.
//!
//! Le « relatives » est tout l'enjeu. Deux régions du canevas qui portent la même chose
//! disposée pareil ont alors la **même empreinte**, où qu'elles soient dans le document — donc
//! un seul rendu pour les deux. C'est la propriété de Hashlife transposée : le travail devient
//! proportionnel au nombre de **contenus distincts**, pas au nombre de tuiles à l'écran.
//!
//! Et le cas le plus fréquent est le meilleur : une tuile vide a une empreinte vide, la même
//! pour toutes. Se déplacer sur une zone sans contenu ne dessine **rien**.
//!
//! # Ce que ce module ne fait pas
//!
//! Il ne dessine pas, il ne garde rien en mémoire, et il ne connaît ni la vue ni l'écran. Il
//! répond à trois questions, toutes géométriques : quelles tuiles couvrent cette région, quel
//! niveau correspond à cette échelle, et qu'est-ce que cette tuile montre. Le cache et le
//! rendu vivent dans `glucose-desktop`, qui a les pixels.

use crate::geometry::Rect;

/// Le côté d'une tuile, en pixels de son niveau.
///
/// # Ce nombre est mesuré, pas choisi
///
/// `bench_tuiles` compare quatre tailles sur trois définitions d'écran. À 2560 × 1600, le
/// temps de composer un écran entier à l'échelle exacte :
///
/// ```text
///   128 px   1,12 ms        512 px   2,58 ms
///   256 px   0,92 ms       1024 px   1,88 ms
/// ```
///
/// Le compromis est celui qu'on attend : trop petite, on paie un appel par carré ; trop
/// grande, une invalidation jette du travail utile et la tuile déborde largement de l'écran.
/// Le creux est à 256, et c'est aussi ce que retiennent les canevas qui tiennent.
pub const COTE: u32 = 256;

/// L'adresse d'une tuile : son niveau dyadique, et sa case dans la grille de ce niveau.
///
/// Les coordonnées sont des entiers **signés et larges** : le canevas est infini dans les
/// quatre directions, et un `i64` de cases de 256 pixels couvre de quoi poser dix millions de
/// nœuds sans jamais approcher le bord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Adresse {
    /// L'exposant de l'échelle : une tuile de niveau `n` se rend à l'échelle `2ⁿ`.
    pub niveau: i32,
    pub x: i64,
    pub y: i64,
}

impl Adresse {
    /// L'échelle d'un niveau : `2ⁿ`.
    pub fn echelle(niveau: i32) -> f64 {
        f64::from(niveau).exp2()
    }

    /// Le niveau dont l'échelle **ne dépasse pas** celle demandée.
    ///
    /// On descend plutôt qu'on ne monte : prendre le niveau au-dessus obligerait à réduire une
    /// tuile, donc à jeter du détail qu'on vient de payer, tandis que l'agrandir d'un facteur
    /// compris entre un et deux reste dans le régime que `bench_tuiles` mesure à 1,7 ms.
    ///
    /// Une échelle nulle ou absurde retombe sur le niveau zéro : aucun appelant n'a alors à
    /// se demander ce qu'il obtient.
    pub fn niveau_pour(echelle: f64) -> i32 {
        if !echelle.is_finite() || echelle <= 0.0 {
            return 0;
        }
        echelle.log2().floor() as i32
    }

    /// Le côté d'une tuile de ce niveau, en unités du **monde**.
    pub fn cote_monde(niveau: i32) -> f64 {
        f64::from(COTE) / Self::echelle(niveau)
    }

    /// Le rectangle du monde que cette tuile couvre.
    pub fn couvre(self) -> Rect {
        let cote = Self::cote_monde(self.niveau);
        Rect::new(self.x as f64 * cote, self.y as f64 * cote, cote, cote)
    }

    /// La tuile de ce niveau qui contient ce point du monde.
    pub fn contenant(niveau: i32, x: f64, y: f64) -> Self {
        let cote = Self::cote_monde(niveau);
        Self {
            niveau,
            x: (x / cote).floor() as i64,
            y: (y / cote).floor() as i64,
        }
    }

    /// Le coin haut-gauche de cette tuile, en pixels de son niveau.
    ///
    /// Sert à ramener une position du monde dans le repère de la tuile — c'est cette
    /// soustraction qui rend deux régions identiques réellement égales.
    pub fn origine_en_pixels(self) -> (f64, f64) {
        (
            self.x as f64 * f64::from(COTE),
            self.y as f64 * f64::from(COTE),
        )
    }
}

/// Toutes les tuiles d'un niveau qui rencontrent ce rectangle du monde.
///
/// L'ordre est celui de la lecture — ligne par ligne, de gauche à droite — pour que deux
/// parcours de la même région donnent la même suite, donc le même ordre de chantier.
///
/// Un rectangle vide ou insensé ne rend aucune tuile plutôt que d'en rendre des milliards :
/// une largeur infinie arriverait sinon jusqu'à la boucle.
pub fn couvrant(niveau: i32, region: Rect) -> impl Iterator<Item = Adresse> {
    let cote = Adresse::cote_monde(niveau);
    let fini = region.left.is_finite()
        && region.top.is_finite()
        && region.width.is_finite()
        && region.height.is_finite();
    let vide = !fini || region.width <= 0.0 || region.height <= 0.0 || cote <= 0.0;

    let (x0, y0, x1, y1) = if vide {
        (0i64, 0i64, -1i64, -1i64)
    } else {
        let premier = Adresse::contenant(niveau, region.left, region.top);
        // Le bord droit est **exclu** : un rectangle qui s'arrête pile sur une frontière ne
        // demande pas la tuile d'après, dont il ne montrerait aucun pixel.
        let dernier = Adresse::contenant(
            niveau,
            region.right() - f64::EPSILON.max(cote * 1e-9),
            region.bottom() - f64::EPSILON.max(cote * 1e-9),
        );
        (
            premier.x,
            premier.y,
            dernier.x.max(premier.x),
            dernier.y.max(premier.y),
        )
    };

    (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| Adresse { niveau, x, y }))
}

/// Ce qu'une tuile montre, résumé en un nombre.
///
/// Deux tuiles de même empreinte portent la même chose disposée pareil : leur rendu est le
/// même, au bit près, et une seule des deux a besoin d'être peinte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Empreinte(u64);

/// L'amorce de FNV-1a sur soixante-quatre bits.
const AMORCE: u64 = 0xcbf2_9ce4_8422_2325;

/// Le multiplicateur de FNV-1a sur soixante-quatre bits.
const PREMIER: u64 = 0x1000_0000_01b3;

impl Empreinte {
    /// L'empreinte d'une tuile **vide** — et c'est la plus utile de toutes.
    ///
    /// Dans un canevas infini, l'immense majorité des tuiles ne portent rien. Elles partagent
    /// donc toutes cette empreinte, donc un seul rendu : se déplacer sur du vide ne dessine
    /// rien, quel que soit le nombre de tuiles traversées.
    pub fn vide() -> Self {
        Self(AMORCE)
    }

    /// Ajoute ce qu'un occupant montre, dans l'ordre où il sera dessiné.
    ///
    /// L'ordre compte et n'est pas commutatif : deux nœuds qui se recouvrent ne donnent pas la
    /// même image selon lequel passe devant. Une empreinte qui les confondrait ferait
    /// réutiliser le mauvais rendu.
    pub fn ajouter(&mut self, occupant: Occupant) {
        for mot in occupant.mots() {
            self.0 = (self.0 ^ mot).wrapping_mul(PREMIER);
        }
    }

    /// L'empreinte de cette suite d'occupants, dans cet ordre.
    pub fn de(occupants: impl IntoIterator<Item = Occupant>) -> Self {
        let mut empreinte = Self::vide();
        for occupant in occupants {
            empreinte.ajouter(occupant);
        }
        empreinte
    }

    /// Le nombre, pour s'en servir comme clé.
    pub fn valeur(self) -> u64 {
        self.0
    }

    /// L'empreinte que porte ce nombre — l'opération inverse de [`Self::valeur`].
    ///
    /// Sert à un cache qui garde les empreintes sous leur forme nue : les retenir comme des
    /// nombres évite d'imposer son type à une table qui n'a que faire de leur sens.
    pub fn depuis(valeur: u64) -> Self {
        Self(valeur)
    }
}

/// Ce qu'un nœud apporte à une tuile — sa place et son aspect, et rien d'autre.
///
/// # Pourquoi les coordonnées sont relatives, et en pixels du niveau
///
/// Relatives, pour que deux régions identiques du canevas donnent la même empreinte où
/// qu'elles soient : c'est ce qui fait qu'un mur de photos posé deux fois ne se peint qu'une.
///
/// En pixels du niveau, parce que c'est l'unité où le rendu a lieu : deux positions qui
/// donnent les mêmes pixels doivent donner la même empreinte, et deux qui n'en donnent pas de
/// différents ne doivent pas s'en distinguer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Occupant {
    /// La boîte, en pixels du niveau, relative au coin de la tuile.
    pub boite: Rect,
    /// Ce qui distingue ce qui sera dessiné : la photo, le texte, la couleur, la rotation.
    ///
    /// L'appelant le compose comme il veut ; le noyau ne fait que le mêler. Deux occupants qui
    /// portent le même nombre **doivent** donner les mêmes pixels, et c'est à l'appelant de le
    /// garantir — c'est la seule règle de ce type.
    pub aspect: u64,
}

impl Occupant {
    /// Les mots à mêler, dans un ordre fixe.
    ///
    /// La boîte passe par ses bits, et non par une valeur arrondie : deux positions qui
    /// diffèrent d'un millième de pixel donnent des pixels différents une fois le filtre
    /// appliqué, et les confondre ferait réutiliser un rendu qui n'est pas le bon.
    fn mots(self) -> [u64; 5] {
        [
            self.boite.left.to_bits(),
            self.boite.top.to_bits(),
            self.boite.width.to_bits(),
            self.boite.height.to_bits(),
            self.aspect,
        ]
    }
}

#[cfg(test)]
mod tests;
