//! L'horloge de la trajectoire : le temps tel que l'**écran** le montre.
//!
//! # Le raisonnement, et il tient en une égalité
//!
//! La position affichée à l'image `n` vaut `x(n) = x(n−1) + v · pas(n)`. Elle reste sous les
//! yeux pendant `Δ(n)`, l'intervalle qui sépare deux présentations. La vitesse que l'œil
//! mesure est donc :
//!
//! ```text
//!     v_apparente(n) = v · pas(n) / Δ(n)
//! ```
//!
//! Elle vaut `v` — c'est-à-dire que le mouvement paraît régulier — **si et seulement si
//! `pas(n) = Δ(n)`**. Rien d'autre n'entre en jeu : ni le nombre d'images par seconde, ni la
//! latence, ni le coût du rendu.
//!
//! Or `pas` valait le temps mural écoulé entre deux **débuts de rendu**. Ce n'est pas `Δ` : les
//! deux diffèrent exactement de la variation du coût d'une image à l'autre, qui va de 6 à
//! 67 millisecondes sur le terrain. À mille pixels par seconde, le contenu se posait donc
//! jusqu'à soixante pixels à côté de sa trajectoire, pour sept pixels d'avance attendue.
//!
//! **Le modèle du mouvement était exact ; c'est son horloge qui ne l'était pas.** Quatre
//! tentatives ont cherché la cause dans l'élan, et aucune ne pouvait la trouver.
//!
//! # Ce que ce module fait, et les deux choses qu'il corrige
//!
//! **Il se cadence sur les présentations.** C'est le seul instant de la boucle qui corresponde
//! à quelque chose que l'œil reçoive ; le début du rendu et sa fin sont des faits internes.
//! Le pas d'une image est donc l'intervalle d'affichage qui vient d'être **observé**, et non
//! une durée de calcul. Cela décale la trajectoire d'une image — soit une période d'écran,
//! 4,17 ms sur la machine de l'utilisateur — et supprime un écart qui montait à soixante.
//!
//! **Il quantifie ce pas sur la grille de l'écran.** Un écran ne montre pas une image
//! « pendant 6,14 millisecondes » : il balaie à intervalle fixe, et une image y reste un
//! **nombre entier** de balayages. Un pas qui ne l'est pas décrit une durée d'affichage qui
//! n'existe pas.
//!
//! ```text
//!     dette += Δ observé
//!     k      = ⌊dette / P⌋          P = la période de l'écran, LUE
//!     pas    = k · P
//!     dette -= pas
//! ```
//!
//! # Pourquoi une dette, et pas un arrondi
//!
//! Un arrondi dérive. À 1,4 période par image, arrondir à l'inférieur fait avancer la
//! trajectoire d'une période quand 1,4 s'est écoulée : la caméra prend **quarante pour cent de
//! retard permanent**, et une glissade d'une demi-seconde en met huit dixièmes.
//!
//! La fraction non consommée se reporte donc sur le pas suivant. C'est l'accumulateur de
//! Bresenham, transposé au temps : chaque pas reste un multiple de la période, et la somme des
//! pas suit le temps réel à moins d'une période près, **pour toujours**.
//!
//! # Pourquoi aucune borne sur un pas très long
//!
//! Les deux horloges que celle-ci remplace en portaient une, à cent millisecondes, « pour
//! qu'une image très longue ne fasse pas franchir d'un coup ce qu'elle a manqué ». C'était se
//! tromper de problème.
//!
//! Si un dialogue natif a bloqué la boucle trois secondes, la glissade **est** terminée :
//! trois secondes ont réellement passé, et l'amortissement l'a éteinte depuis longtemps. La
//! borne ne supprimait pas le saut — elle le remplaçait par pire, une glissade qui reprend au
//! ralenti pendant trente images pour rattraper un retard que personne n'attendait.
//!
//! La constante disparaît donc, comme la charte l'exige de toute constante qui le peut.
//!
//! # Ce que ce module ne corrige pas, et il faut le dire
//!
//! La vitesse apparente vaut `pas(n) / Δ(n)`, et ce module donne `pas(n) = Δ(n−1)`. **Tant que
//! l'intervalle d'affichage varie d'une image à l'autre, elle varie encore** — simplement
//! décalée d'un cran, et débarrassée de tout ce que le temps de calcul y injectait.
//!
//! La rendre vraiment constante demande de rendre `Δ` constant, c'est-à-dire de faire tomber
//! la variance du coût d'une image : c'est le chantier du cache de tuiles et de la salissure,
//! pas celui d'une horloge. Un test porte cette limite (`test_une_cadence_qui_varie_fait_
//! encore_varier_la_vitesse_apparente`) plutôt qu'un commentaire, parce qu'un commentaire ne
//! casse pas la build le jour où quelqu'un croit le contraire.
//!
//! Ce qui est acquis ici est net et se garde : **la trajectoire ne dépend plus du tout de ce
//! qu'une image coûte.** C'est une erreur systématique en moins, pas la fin du sujet.
//!
//! # Sans période connue, rien n'est quantifié
//!
//! Sur un bureau distant ou une machine virtuelle, le système n'annonce pas toujours sa
//! fréquence. L'horloge rend alors l'intervalle observé tel quel : inventer une période serait
//! inventer une grille qui n'existe pas. La première correction — se cadencer sur les
//! présentations — vaut de toute façon, et c'est la plus grosse des deux.

use std::time::{Duration, Instant};

/// Le temps de la trajectoire, cadencé par l'écran et aligné sur ses balayages.
#[derive(Debug, Clone, Copy, Default)]
pub struct Horloge {
    /// La période de l'écran. Nulle tant qu'il n'a rien annoncé.
    periode: Duration,
    /// L'instant de la présentation précédente.
    precedente: Option<Instant>,
    /// Le temps écoulé qu'aucun balayage n'a encore montré — toujours sous une période.
    dette: Duration,
    /// Ce que la prochaine image doit faire avancer la trajectoire.
    pas: Duration,
}

impl Horloge {
    pub fn nouvelle() -> Self {
        Self::default()
    }

    /// Accorde l'horloge à l'écran. Se lit au démarrage, et se relit si l'écran change.
    pub fn accorder(&mut self, periode: Duration) {
        self.periode = periode;
    }

    /// La période de l'écran, si elle est connue.
    pub fn periode(&self) -> Option<Duration> {
        (!self.periode.is_zero()).then_some(self.periode)
    }

    /// Une image vient d'être présentée : l'horloge en tire le pas de la suivante.
    ///
    /// # Pourquoi le pas se décide ici, et non au moment de rendre
    ///
    /// Ce qu'il faut connaître, c'est la durée pendant laquelle l'image **sera** affichée, et
    /// elle n'existe pas encore quand on la dessine. La seule grandeur du même ordre qui soit
    /// déjà mesurée est celle que l'image précédente vient de vivre — et elle est bien plus
    /// proche de la vérité que le temps de calcul, qui n'en est pas une approximation du tout.
    ///
    /// `attendue` dit si cette image avait été demandée par la précédente. Sinon l'application
    /// dormait : il n'y a aucune trajectoire à rattraper, et un pas de trois secondes ferait
    /// franchir d'un coup au premier geste ce que personne n'a demandé pendant le sommeil.
    pub fn presentee(&mut self, maintenant: Instant, attendue: bool) {
        let Some(avant) = self.precedente.replace(maintenant) else {
            // La première image n'a pas d'intervalle : la trajectoire n'avance pas encore.
            return;
        };
        if !attendue {
            self.dette = Duration::ZERO;
            self.pas = Duration::ZERO;
            return;
        }
        let intervalle = maintenant.saturating_duration_since(avant);
        let Some(periode) = self.periode() else {
            self.pas = intervalle;
            return;
        };
        self.dette = self.dette.saturating_add(intervalle);
        // Le nombre entier de balayages que la dette couvre, et rien de plus : la fraction
        // restante attend le pas suivant, donc rien ne se perd et rien ne s'invente.
        let balayages = self.dette.as_nanos() / periode.as_nanos().max(1);
        self.pas = periode.saturating_mul(u32::try_from(balayages).unwrap_or(u32::MAX));
        self.dette = self.dette.saturating_sub(self.pas);
    }

    /// La boucle a été tenue hors de tout rendu — un dialogue natif, le plus souvent : rien de
    /// ce qui précède ne décrit ce que l'œil a vu, et la trajectoire repart de la prochaine
    /// présentation, sans dette.
    pub fn oublier(&mut self) {
        self.precedente = None;
        self.dette = Duration::ZERO;
        self.pas = Duration::ZERO;
    }

    /// Ce que la trajectoire doit avancer pour l'image qu'on s'apprête à dessiner.
    pub fn pas(&self) -> Duration {
        self.pas
    }

    /// L'instant de la dernière présentation, s'il y en a eu une.
    ///
    /// C'est de lui que se déduit le prochain balayage, donc le moment où la boucle doit se
    /// réveiller pour une animation : ni avant, rien ne serait montré ; ni après, un balayage
    /// serait raté pour rien.
    pub fn derniere_presentation(&self) -> Option<Instant> {
        self.precedente
    }

    /// Le temps écoulé qu'aucun balayage n'a encore montré.
    ///
    /// Toujours strictement sous une période quand l'écran est connu : c'est l'invariant qui
    /// garantit l'absence de dérive, et il se vérifie plutôt qu'il ne se promet.
    pub fn dette(&self) -> Duration {
        self.dette
    }
}

#[cfg(test)]
mod tests;
