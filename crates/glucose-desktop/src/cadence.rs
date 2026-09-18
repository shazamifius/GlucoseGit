//! La cadence visée, et le temps qu'elle laisse au travail de fond (CADENCE-1).
//!
//! # La règle que ce module porte
//!
//! Elle n'est pas « chaque action doit tenir dans le budget ». Elle est : **on doit toujours
//! pouvoir se déplacer dans Glucose à la cadence de l'écran, quoi qu'il se passe par
//! ailleurs.** Une action lourde a le droit de durer — elle se fait en tranches qui tiennent
//! dans le temps libre de chaque image, et le rendu ne l'attend jamais.
//!
//! D'où deux grandeurs, et non une :
//!
//! * **le budget du rendu** — ce qu'une image a le droit de coûter. On le veut court, bien
//!   plus court que la période de l'écran, parce que c'est ce qui reste ensuite qui permet au
//!   travail de fond d'avancer sans se voir ;
//! * **le temps libre** — ce qui reste de la période une fois l'image rendue. C'est le
//!   crédit que l'ordonnanceur dépense, et il se recalcule à chaque image.
//!
//! # Pourquoi la fréquence se lit et ne se suppose pas
//!
//! Tout le projet a longtemps raisonné sur « 100 fps, donc 10 ms ». C'était faux sur l'écran
//! où le défaut a été constaté : **240 Hz, donc 4,16 ms**. Un banc qui annonce « tenu » à
//! 9 ms sur une telle machine ment, et fait optimiser dans la mauvaise direction.
//!
//! La fréquence vient donc de l'écran. Quand le système ne la donne pas — c'est courant sur
//! un bureau distant ou une machine virtuelle — on retient 60 Hz, qui est le minimum absolu
//! de la charte : mieux vaut viser trop bas et tenir que l'inverse.

use std::time::Duration;

/// Ce qu'une image a le droit de coûter, en règle générale.
///
/// Quatre cents images par seconde. Ce n'est pas une cadence d'affichage — aucun écran
/// courant ne la demande — c'est une **cible de coût** : une image rendue en deux
/// millisecondes et demie laisse tout le reste de la période au travail de fond, et c'est ce
/// qui permet à une action lourde de s'étaler sans jamais se voir.
pub const BUDGET_RENDU: Duration = Duration::from_micros(2_500);

/// La cadence la plus basse que la charte admette, toutes machines confondues.
pub const FPS_PLANCHER: f64 = 60.0;

/// La part de la période qu'on s'autorise pour une image, quand la période est courte.
///
/// Sur un écran très rapide, [`BUDGET_RENDU`] peut dépasser ce qui est raisonnable : à
/// 500 Hz la période entière fait deux millisecondes. Le rendu ne prend alors jamais plus
/// des deux tiers de la période, pour qu'il reste toujours de quoi présenter et souffler.
const PART_MAX_DU_RENDU: f64 = 2.0 / 3.0;

/// Une marge gardée en fin de période, jamais offerte au travail de fond.
///
/// La présentation, le système et l'ordonnanceur lui-même ont besoin d'un peu d'air. Sans
/// elle, une tranche estimée au plus juste ferait rater l'image — et rater une image se voit,
/// là où retarder une tranche ne se voit pas.
const MARGE: Duration = Duration::from_micros(300);

/// La cadence d'un écran, et ce qu'elle autorise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cadence {
    /// La durée d'une image à l'écran — ce qu'on ne peut pas dépasser sans en sauter une.
    periode: Duration,
}

impl Default for Cadence {
    fn default() -> Self {
        Self::inconnue()
    }
}

impl Cadence {
    /// La cadence déduite de la fréquence annoncée par l'écran, en millihertz.
    ///
    /// C'est l'unité de `winit` ; la convertir ici plutôt que chez l'appelant évite qu'un
    /// facteur mille se perde en route.
    pub fn depuis_millihertz(millihertz: u32) -> Self {
        if millihertz == 0 {
            return Self::inconnue();
        }
        let hz = f64::from(millihertz) / 1_000.0;
        // Un écran plus lent que le plancher de la charte ne change rien à ce qu'on vise :
        // c'est lui qui impose sa période, et on la respecte.
        Self {
            periode: Duration::from_secs_f64(1.0 / hz.max(1.0)),
        }
    }

    /// La cadence retenue quand le système ne dit rien.
    pub fn inconnue() -> Self {
        Self {
            periode: Duration::from_secs_f64(1.0 / FPS_PLANCHER),
        }
    }

    /// La durée d'une image à l'écran.
    pub fn periode(&self) -> Duration {
        self.periode
    }

    /// La cadence en images par seconde.
    pub fn fps(&self) -> f64 {
        1.0 / self.periode.as_secs_f64()
    }

    /// Ce qu'une image a le droit de coûter sur cet écran.
    pub fn budget_rendu(&self) -> Duration {
        let part = self.periode.mul_f64(PART_MAX_DU_RENDU);
        BUDGET_RENDU.min(part)
    }

    /// Ce que le travail de fond peut prendre après une image qui a coûté `rendu`.
    ///
    /// # Deux régimes, et le second est celui qui compte
    ///
    /// * **L'image a tenu dans la période** — le fond prend ce qui reste, et rien ne se voit.
    /// * **L'image a dépassé la période** — la cadence est *déjà* perdue, et le travail de fond
    ///   est précisément ce qui la fera revenir. Lui refuser sa tranche enfermerait la machine
    ///   dans son régime dégradé : les images resteraient chères parce que le travail
    ///   n'avance pas, et le travail n'avancerait pas parce que les images sont chères.
    ///
    /// Mesuré sur le banc d'occlusion : vingt-sept photos coûtent 39,62 ms par le chemin
    /// général, et **0,90 ms** une fois leurs vignettes faites. Ne jamais les faire, pour
    /// protéger une cadence qu'on a déjà perdue, revient à garder quarante fois le prix.
    ///
    /// Dans ce second cas on accorde donc **une période**. Ce n'est pas un réglage : c'est la
    /// seule durée de référence que l'écran donne, et elle borne le dépassement au double
    /// d'une image déjà ratée, en échange d'une sortie en un nombre d'images borné.
    pub fn tranche_de_fond(&self, rendu: Duration) -> Duration {
        self.temps_libre(rendu).unwrap_or(self.periode)
    }

    /// Ce qui reste pour le travail de fond après une image qui a coûté `rendu`.
    ///
    /// Rend `None` quand il ne reste rien : l'image a mangé sa période, et faire avancer une
    /// tranche par-dessus ne ferait que creuser le retard.
    pub fn temps_libre(&self, rendu: Duration) -> Option<Duration> {
        self.periode
            .checked_sub(rendu)?
            .checked_sub(MARGE)
            .filter(|reste| !reste.is_zero())
    }
}

#[cfg(test)]
mod tests;
