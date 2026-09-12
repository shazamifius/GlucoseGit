//! Les coordonnées du document : entiers à virgule fixe — `std` uniquement, 0 dépendance.
//!
//! # Pourquoi pas `f64` — et ce que la mesure a démenti
//!
//! La loi L7 de la fiche 02 pose « unités monde, `f64`, aucune exception ». L'unicité est
//! juste ; le type ne l'est pas. Mais deux des trois arguments avancés par la fiche 11 § RQ-1
//! ne résistent pas à la mesure, et il faut le dire avant de donner ceux qui tiennent :
//!
//! | Argument de RQ-1 | Ce que la mesure dit |
//! |---|---|
//! | « ~32 octets par nœud au lieu de ~512 » | **Pas grâce à la virgule fixe.** Les quatre coordonnées passent de 32 à 16 octets : 16 octets sur 512, soit **3 %**. L'essentiel du gain vient d'ailleurs — 44 % d'une `Annotation` texte est du remplissage imposé par la variante `Arrow` de l'énumération. |
//! | « l'erreur s'accumule sur les milliers de petits pas d'un glisser long » | **Elle ne s'accumule pas de façon visible.** Mesure : cent mille pas de 0,1 px dérivent de **2,3 × 10⁻⁶ pixel** — soit 1,2 × 10⁻⁴ pixel écran au zoom maximal. Un aller-retour symétrique revient exactement. L'associativité stricte est un fait mathématique, pas un problème observé. |
//! | « plus aucun `< 1e-9` à calibrer nulle part » | **Quatre sur quarante-six.** Le reste des epsilons du dépôt portent sur des *échelles* et des *ratios*, qui restent en `f64` : ils ne disparaîtront pas. |
//!
//! Ce qui justifie réellement la virgule fixe, mesuré et non supposé :
//!
//! - **La mémoire, mais seulement dans un modèle déjà compact.** Le tronc chaud d'un nœud
//!   (position, taille, genre, drapeaux, parent) pèse 22 octets hors coordonnées. Y ajouter
//!   quatre `f64` le porte à 54 ; quatre `Fx`, à 38. Les mêmes 16 octets qui ne valaient 3 %
//!   dans l'ancien modèle en valent **30 % dans le nouveau** — 160 Mo sur dix millions de
//!   nœuds, contre un objectif de 500 Mo. Un gain se juge sur ce qui reste, pas sur ce qu'on
//!   remplace.
//! - **L'indexation exacte**, et c'est l'argument structurant que RQ-1 n'a pas fait. `Ord` et
//!   `Hash` sont dérivables sur `Fx` et ne le sont pas sur `f64`. Une clé de tuile, un rang sur
//!   une courbe de Hilbert, un tri par abscisse, une déduplication de positions : tout cela
//!   devient exact et trivial. L'index spatial et le cache de tuiles en dépendent directement.
//! - **Les bornes anti-crash de la fiche 09 cessent d'être des `clamp` posés à la main** pour
//!   devenir une propriété du type : aucune opération de ce module ne peut sortir du domaine,
//!   donc il n'y a plus de garde à ne pas oublier.
//! - **La distance au carré est exacte** ([`dist2`]) : comparer deux distances ne demande
//!   aucune tolérance. C'est exactement ce dont le picking a besoin, et c'est l'un des quatre
//!   epsilons qui disparaissent vraiment.
//!
//! Le `f64` garde sa place, entière, là où l'on calcule des **pixels** : transform caméra,
//! rastériseur, mise en page du texte, facteurs d'échelle. La frontière est nette — `Fx` décrit
//! où sont les choses dans le document, `f64` décrit où elles tombent à l'écran.
//!
//! # Pourquoi 1/256 de pixel, et pourquoi cette valeur ne pouvait pas être autre chose
//!
//! [`FRAC_BITS`] n'est pas un chiffre rond choisi au jugé : c'est le **seul** entier qui
//! satisfasse les deux exigences du projet à la fois. Avec un `i32`, portée et précision
//! s'échangent exactement — un bit gagné d'un côté est un bit perdu de l'autre :
//!
//! ```text
//!     portée = ±2³¹ / 2^FRAC_BITS  pixels          précision = 2^-FRAC_BITS  pixel
//! ```
//!
//! | Exigence | Inégalité | Conséquence |
//! |---|---|---|
//! | Un canva de 10⁷ nœuds doit tenir dans le document, au maillage confortable de 5 000 px (√10⁷ ≈ 3 163 nœuds de côté, soit 15,8 Mpx de large) | portée ≥ 7,9 Mpx | `FRAC_BITS ≤ 8` |
//! | Au zoom maximal du modèle (×50, fiche 09 § 1), une unité doit rester indiscernable : au plus un quart de pixel écran | 50 / 2^FRAC_BITS ≤ 0,25 | `FRAC_BITS ≥ 8` |
//!
//! Les deux inégalités enferment **8**, et `test_la_precision_est_la_seule_qui_satisfasse_les_deux_exigences`
//! le vérifie en refusant 7 comme 9. La constante ne se règle pas : elle se déduit. Le jour où
//! l'une des deux exigences change, c'est le test qui le dira.
//!
//! > **Ce que cela révèle de la fiche 09, et qui reste à trancher.** À l'échelle minimale du
//! > modèle (0,005), un écran de 1 920 px ne montre que 384 000 px de monde — alors que le
//! > document en fait 16,8 millions de large. **On ne peut donc pas voir un canva Wikipédia en
//! > entier.** Ce n'est pas un défaut de `Fx` : c'est la borne de dézoom qui est calibrée pour
//! > une carte humaine, pas pour dix millions de nœuds. À traiter en fiche 09, pas ici.

use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// Bits de partie fractionnaire d'un [`Fx`] : une unité vaut 2⁻⁸ = 1/256 de pixel monde.
///
/// Déduite, pas choisie — voir la table de l'en-tête du module.
pub const FRAC_BITS: u32 = 8;

/// Une coordonnée du document : un pixel monde divisé en 2^[`FRAC_BITS`] unités.
///
/// `Ord`, `Eq` et `Hash` sont dérivés et **exacts** : c'est tout l'intérêt. Deux positions sont
/// égales ou ne le sont pas, et l'ordre sur `Fx` est celui des réels qu'il représente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fx(i32);

impl Fx {
    /// L'origine.
    pub const ZERO: Self = Self(0);
    /// Un pixel monde.
    pub const ONE: Self = Self(1 << FRAC_BITS);
    /// La plus petite quantité représentable : 1/256 de pixel.
    pub const EPSILON: Self = Self(1);
    /// La borne inférieure du document.
    pub const MIN: Self = Self(i32::MIN);
    /// La borne supérieure du document.
    pub const MAX: Self = Self(i32::MAX);
    /// L'étendue du document dans une direction, en pixels monde : ±8 388 608 px.
    ///
    /// Toute opération de ce module sature ici plutôt que de déborder. C'est la borne
    /// anti-crash de la fiche 09 § 1, devenue une propriété du type : il n'existe aucune
    /// valeur de `Fx` hors du domaine, donc aucun `clamp` à ne pas oublier.
    pub const SPAN: f64 = (1i64 << 31) as f64 / (1i64 << FRAC_BITS) as f64;

    /// Construit depuis un nombre entier de pixels monde, en saturant.
    pub const fn from_px(px: i32) -> Self {
        Self(px.saturating_mul(1 << FRAC_BITS))
    }

    /// Construit depuis des unités brutes de 1/256 px.
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Les unités brutes. C'est ce que la persistance écrit et ce que le SoA range.
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Construit depuis un `f64` de pixels monde, **au plus proche**, en saturant.
    ///
    /// C'est le seul point d'entrée du monde flottant : import d'un fichier, position sous le
    /// curseur, résultat d'un calcul de mise en page. Un `NaN` devient [`Fx::ZERO`] — la même
    /// décision que [`crate::types::Viewport::normalized`], et pour la même raison : une
    /// coordonnée qui n'est pas un nombre rendrait le nœud invisible sans rien faire tomber.
    pub fn from_f64(px: f64) -> Self {
        if px.is_nan() {
            return Self::ZERO;
        }
        let units = (px * (1i64 << FRAC_BITS) as f64).round();
        Self(units.clamp(i32::MIN as f64, i32::MAX as f64) as i32)
    }

    /// Les pixels monde, en `f64`. Exact : tout `i32` se représente sans perte en `f64`.
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / (1i64 << FRAC_BITS) as f64
    }

    /// La valeur absolue, saturée ([`Fx::MIN`] n'a pas d'opposé).
    pub fn abs(self) -> Self {
        Self(self.0.saturating_abs())
    }

    /// Le plus petit des deux.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Le plus grand des deux.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Ramené dans `[lo, hi]`. Contrairement à `f64::clamp`, ne peut pas recevoir de `NaN`.
    pub fn clamp(self, lo: Self, hi: Self) -> Self {
        Self(self.0.clamp(lo.0, hi.0))
    }

    /// Mis à l'échelle par un rationnel exact `num / den`, arrondi au plus proche.
    ///
    /// Le calcul intermédiaire se fait en `i64` : aucun débordement possible avant la
    /// saturation finale. C'est l'opération dont le redimensionnement a besoin — un facteur
    /// d'échelle y est toujours un rapport de deux longueurs, donc un rationnel exact, jamais
    /// un flottant.
    pub fn scaled(self, num: i32, den: i32) -> Self {
        if den == 0 {
            return self;
        }
        let (n, d) = (num as i64, den as i64);
        let prod = self.0 as i64 * n;
        // Arrondi au plus proche, en restant entier : (a + d/2) / d, du bon côté du signe.
        let half = d.abs() / 2;
        let adjusted = if (prod >= 0) == (d > 0) {
            prod + half
        } else {
            prod - half
        };
        Self(saturate(adjusted / d))
    }
}

/// Ramène un `i64` dans le domaine d'un [`Fx`] en saturant.
#[inline]
fn saturate(v: i64) -> i32 {
    v.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

impl Add for Fx {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Fx {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Neg for Fx {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }
}

impl AddAssign for Fx {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Fx {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

/// Le carré de la distance entre deux points, en unités de 1/256 px — **exact, sans condition**.
///
/// Aucune racine, aucune tolérance : comparer deux distances, c'est comparer deux entiers. Le
/// picking (fiche 07 § 3) et le magnétisme (§ 4) n'ont besoin de rien d'autre, et cessent du
/// même coup de dépendre d'un epsilon.
///
/// # Pourquoi `i128` et pas `i64`
///
/// Un écart entre deux coordonnées atteint 2³² unités ; son carré, 2⁶⁴ ; leur somme, 2⁶⁵. Un
/// `i64` **déborde** sur les coins opposés du document — la première version de cette fonction
/// l'affirmait pourtant exacte, et c'est le compilateur qui l'a démentie.
///
/// On aurait pu rétrécir le domaine de [`Fx`] jusqu'à ce que `i64` suffise : il aurait fallu
/// descendre à ±4,19 Mpx, ce qui viole l'exigence de portée du module (±7,9 Mpx pour un canva
/// de 10⁷ nœuds). On aurait pu aussi documenter une condition d'emploi — « exact tant que les
/// points sont proches » : un type exact *sauf si* est un type qui trahit un jour.
///
/// `i128` est exact par construction et ne demande rien à retenir. Sur les architectures 64
/// bits — les seules que Glucose vise — une multiplication 64 × 64 → 128 est une instruction
/// unique, et la comparaison qui suit est négligeable devant l'accès mémoire au nœud.
pub fn dist2(ax: Fx, ay: Fx, bx: Fx, by: Fx) -> i128 {
    let dx = ax.0 as i128 - bx.0 as i128;
    let dy = ay.0 as i128 - by.0 as i128;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests;
