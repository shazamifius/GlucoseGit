//! La cadence visée, et le temps qu'elle laisse au travail de fond (CADENCE-1).
//!
//! # La règle que ce module porte
//!
//! Elle n'est pas « chaque action doit tenir dans le budget ». Elle est : **on doit toujours
//! pouvoir se déplacer dans Glucose à la cadence de l'écran, quoi qu'il se passe par
//! ailleurs.** Une action lourde a le droit de durer — elle se fait en tranches qui tiennent
//! dans le temps libre de chaque image, et le rendu ne l'attend jamais.
//!
//! # Une seule grandeur décide, et c'est le plancher
//!
//! Ce module a longtemps porté **deux** budgets : une « cible de coût » à deux millisecondes
//! et demie, et le plancher de la charte à dix. Deux budgets pour une seule question, c'est
//! une question à laquelle on répond deux fois — et la cible a fini par servir de seuil de
//! dégradation, ce qu'elle n'était pas. Rendue à moitié résolution dès qu'elle dépassait
//! 2,5 ms, la scène partait en gros blocs sur 22 % des images d'une session réelle, sans que
//! la cadence y gagne rien.
//!
//! Il n'en reste donc qu'une, [`BUDGET_TOTAL`] : **une image et son travail de fond tiennent
//! ensemble dans dix millisecondes.** Le temps libre, lui, n'est pas un budget mais une
//! constatation — ce qui reste de la période une fois l'image rendue.
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

/// La cadence la plus basse que la charte admette, toutes machines confondues.
pub const FPS_PLANCHER: f64 = 60.0;

/// Ce qu'une image **et son travail de fond** ont le droit de coûter ensemble.
///
/// Cent images par seconde : le plancher que la charte pose sans le négocier — « 100 fps
/// minimum constant, tout le temps, quoi qu'il se passe ; si on est en dessous, alors go
/// pixeliser tout ». C'est donc la seule borne légitime pour un investissement, et elle ne
/// dépend ni de l'écran ni de ce que l'image vient de coûter.
pub const BUDGET_TOTAL: Duration = Duration::from_millis(10);

/// Une marge gardée en fin de période, jamais offerte au travail de fond.
///
/// La présentation, le système et l'ordonnanceur lui-même ont besoin d'un peu d'air. Sans
/// elle, une tranche estimée au plus juste ferait rater l'image — et rater une image se voit,
/// là où retarder une tranche ne se voit pas.
pub const MARGE: Duration = Duration::from_micros(300);

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

    /// Ce que le travail de fond peut prendre après une image qui a coûté `rendu`.
    ///
    /// # Deux régimes, et une borne au-dessus des deux
    ///
    /// * **L'image a tenu dans la période** — le fond prend ce qui reste, et rien ne se voit.
    /// * **L'image a dépassé la période** — la cadence est *déjà* perdue, et le travail de fond
    ///   est précisément ce qui la fera revenir. Lui refuser toute tranche enfermerait la
    ///   machine dans son régime dégradé : les images resteraient chères parce que le travail
    ///   n'avance pas, et le travail n'avancerait pas parce que les images sont chères.
    ///
    /// Mesuré sur le banc d'occlusion : vingt-sept photos coûtent 39,62 ms par le chemin
    /// général, et **0,90 ms** une fois leurs vignettes faites. Ne jamais les faire, pour
    /// protéger une cadence qu'on a déjà perdue, revient à garder quarante fois le prix.
    ///
    /// Dans les deux cas, la tranche s'arrête au **plancher de la charte** : une image et son
    /// travail de fond tiennent ensemble dans [`BUDGET_TOTAL`], ou le fond ne prend rien.
    ///
    /// # Les deux erreurs que cette ligne a traversées, parce qu'aucune n'était évidente
    ///
    /// **La première version accordait une période.** Sur un écran à 240 Hz elle vaut 4,17 ms,
    /// et le chantier n'avançait alors que d'une vignette toutes les trois images pendant que
    /// chacune coûtait soixante-dix millisecondes — donc plus l'écran était RAPIDE, moins le
    /// travail de fond avançait. Rattraper un retard n'a aucune raison de dépendre de la
    /// fréquence d'affichage, et ce diagnostic-là reste juste.
    ///
    /// **La deuxième accordait autant que l'image venait de coûter**, en le justifiant ainsi :
    /// « la durée de l'image mesure exactement ce retard ». C'était une **rétroaction
    /// positive**, et le terrain l'a payée cher : une image chère donnait une grosse tranche,
    /// qui rendait l'image suivante plus chère, qui donnait une tranche plus grosse encore. La
    /// chronique montrait l'atelier prendre la moitié de chaque image lente —
    ///
    /// ```text
    ///    3.8s   71.74ms  repos  428 noeuds  413 photos   file 361
    ///           dont atelier 35.97ms, report 29.32ms
    /// ```
    ///
    /// — pour un rendement de **zéro pour cent** : pas une photo de la session ne s'est posée
    /// depuis une vignette. On empruntait sans jamais rembourser.
    ///
    /// La borne ne peut donc venir ni de l'écran ni du retard. Elle vient de la charte : on
    /// investit dans ce qui sépare l'image du plancher, et rien de plus. Quand le rendu seul
    /// mange déjà les dix millisecondes, le fond se tait — et c'est précisément l'instant où
    /// la pixelisation doit rendre la main, pas l'atelier la prendre.
    pub fn tranche_de_fond(&self, rendu: Duration) -> Duration {
        let dans_la_periode = self.temps_libre(rendu).unwrap_or(Duration::ZERO);
        let sous_le_plancher = BUDGET_TOTAL.saturating_sub(rendu + MARGE);
        dans_la_periode.max(sous_le_plancher)
    }

    /// Ce qui reste de la période après une image qui a coûté `rendu`, s'il reste quelque
    /// chose.
    ///
    /// # Interne, et ce n'est pas un détail
    ///
    /// C'est la moitié optimiste de la question, et l'appeler directement a coûté une journée :
    /// `None` y veut dire « la période est mangée », ce qu'un appelant traduit naturellement en
    /// « ne rien faire » — et la machine reste alors enfermée dans son régime dégradé. Le
    /// dehors passe par [`Cadence::tranche_de_fond`], qui répond aux **deux** régimes.
    fn temps_libre(&self, rendu: Duration) -> Option<Duration> {
        self.periode
            .checked_sub(rendu)?
            .checked_sub(MARGE)
            .filter(|reste| !reste.is_zero())
    }
}

#[cfg(test)]
mod tests;
