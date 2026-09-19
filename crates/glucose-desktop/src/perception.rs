//! Ce que l'œil peut voir de ce qui bouge — et donc ce qu'on a le **droit** d'abîmer.
//!
//! # La confusion que ce module défait
//!
//! La finesse du rendu était décidée par un **budget de temps** : si l'image dépasse, on
//! dégrade. C'était confondre deux grandeurs sans rapport.
//!
//! Le budget dit ce dont on a **besoin**. Il ne dit rien de ce qui est **licite**. Et
//! l'utilisateur a nommé la conséquence exactement : « le fait qu'on a un smooth qui nous
//! ralentit peu à peu, et bien la pixelisation continue ». Évidemment : l'amortissement
//! s'éteint, la scène redevient lisible à l'œil, mais le budget reste tendu — donc la
//! dégradation reste.
//!
//! Ce module apporte la seconde condition. **On ne dégrade que si l'on en a besoin _et_ si
//! l'œil ne peut pas le voir.**
//!
//! # La loi, et elle n'est pas de nous
//!
//! La psychophysique de la poursuite oculaire donne deux nombres, et ce sont les seuls que ce
//! module contienne :
//!
//! * **l'œil poursuit** ce qui bouge, avec un gain de 0,9 à 1,0 jusqu'à environ vingt degrés
//!   par seconde. En dessous de cette vitesse, il **annule** le mouvement : le contenu est
//!   stabilisé sur sa rétine, et il le voit aussi net qu'à l'arrêt ;
//! * **au-delà**, la poursuite décroche et il reste un glissement rétinien. L'acuité chute
//!   dès deux degrés par seconde de glissement, et les hautes fréquences spatiales sont
//!   perdues vers cinq ou six.
//!
//! D'où le calcul, sans un seul seuil choisi :
//!
//! ```text
//!     glissement = max(0, vitesse_apparente − POURSUITE_MAX)
//!     facteur admissible = max(1, glissement / PERTE_D_ACUITE)
//! ```
//!
//! Et les trois propriétés qui manquaient tombent d'elles-mêmes :
//!
//! * **la netteté revient en même temps que le mouvement s'éteint**, continûment, sans seuil —
//!   parce que la vitesse tend vers zéro et le facteur admissible vers un ;
//! * à l'arrêt, le facteur vaut un : la netteté est intégrale, comme la charte l'exige ;
//! * la règle est la même sur toute machine, puisqu'elle s'exprime en **degrés vus**, pas en
//!   pixels.
//!
//! # Pourquoi l'hypothèse la plus prudente est la bonne
//!
//! Supposer que l'œil poursuit est **conservateur** : c'est le cas où il voit le mieux, donc
//! celui qui autorise le moins. On préfère ne pas dégrader alors qu'on aurait pu, plutôt que
//! l'inverse — une image trop nette coûte du temps, une image abîmée à tort ne se rattrape
//! pas. C'est la même règle de sûreté que la salissure : on ne réduit le travail que quand on
//! peut le prouver.

/// Ce qu'un pixel **logique** sous-tend à l'œil, en degrés.
///
/// Un pixel logique est défini par la convention d'affichage — un quatre-vingt-seizième de
/// pouce vu à la distance de lecture de référence — et c'est précisément le travail que le
/// système fait déjà pour nous en annonçant son facteur d'échelle : sur un écran deux fois
/// plus dense, il vaut deux pixels physiques, pour le même angle apparent.
///
/// On n'a donc ni densité ni distance de vision à deviner. Un pixel logique, c'est cet angle,
/// sur toute machine.
const DEGRES_PAR_PIXEL_LOGIQUE: f64 = 0.0213;

/// La vitesse jusqu'à laquelle l'œil poursuit ce qui bouge, en degrés par seconde.
///
/// En dessous, il stabilise le contenu sur sa rétine et le voit **net**. Rien n'y est donc
/// dégradable, quelle que soit la vitesse en pixels.
const POURSUITE_MAX: f64 = 20.0;

/// Le glissement rétinien, en degrés par seconde, à partir duquel l'acuité chute.
///
/// Sert d'unité au facteur admissible : deux degrés de glissement de plus, et l'œil résout
/// deux fois moins fin.
const PERTE_D_ACUITE: f64 = 2.0;

/// Ce que la vue peut se permettre d'abîmer, à cette vitesse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perception {
    /// Le plus grand facteur de réduction que l'œil ne verrait pas.
    facteur: u32,
}

impl Perception {
    /// À l'arrêt : rien n'est dégradable.
    pub fn nette() -> Self {
        Self { facteur: 1 }
    }

    /// Ce que l'œil tolère quand la vue se déplace de tant de **pixels physiques** par seconde.
    ///
    /// `echelle` est le facteur d'échelle annoncé par le système pour cet écran — le pont
    /// entre les pixels physiques qu'on mesure et les pixels logiques qui portent l'angle.
    pub fn a_la_vitesse(pixels_par_seconde: f64, echelle: f64) -> Self {
        let logiques = pixels_par_seconde / echelle.max(f64::MIN_POSITIVE);
        let degres = logiques.max(0.0) * DEGRES_PAR_PIXEL_LOGIQUE;
        let glissement = (degres - POURSUITE_MAX).max(0.0);
        // Un facteur admissible fractionnaire n'a pas de sens pour un rendu : on ne réduit
        // que par paliers dyadiques. `floor` sur la puissance de deux inférieure est la seule
        // lecture sûre — arrondir au-dessus abîmerait plus que l'œil ne le tolère.
        let admissible = (glissement / PERTE_D_ACUITE).max(1.0);
        Self {
            facteur: palier_en_dessous(admissible),
        }
    }

    /// Le plus grand facteur de réduction que l'œil ne verrait pas. Vaut 1 à l'arrêt.
    pub fn facteur_admissible(self) -> u32 {
        self.facteur
    }

    /// A-t-on le **droit** d'abîmer l'image en ce moment ?
    ///
    /// Distinct de « en a-t-on besoin », qui est la question du budget. La dégradation demande
    /// les deux, et c'est l'absence de cette seconde condition qui faisait persister la
    /// pixelisation pendant que le mouvement s'éteignait.
    pub fn autorise_a_degrader(self) -> bool {
        self.facteur > 1
    }
}

impl Default for Perception {
    fn default() -> Self {
        Self::nette()
    }
}

/// La plus grande puissance de deux qui ne dépasse pas `valeur`.
fn palier_en_dessous(valeur: f64) -> u32 {
    if !valeur.is_finite() || valeur < 2.0 {
        return 1;
    }
    // `log2` puis `exp2` referait un flottant ; le décalage entier est exact et borné.
    let entier = valeur.min(f64::from(u32::MAX >> 1)) as u32;
    1 << (u32::BITS - 1 - entier.leading_zeros())
}

#[cfg(test)]
mod tests;
