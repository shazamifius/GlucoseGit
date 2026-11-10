//! **Le rythme auquel la main émet**, et ce qu'il dit de son geste : la vitesse, rapportée à
//! l'horizon où l'œil intègre un déplacement, et le silence qui dit qu'elle a lâché.
//!
//! Extrait de [`super`] quand la souris y a pris ses pixels entiers (fiche 100) : six cent sept
//! lignes là où la fiche 05 en admet six cents. La coupure tombe là où la question change —
//! là-bas ce que la vue doit montrer, ici ce que la main vient de faire.

use super::Mouvement;
use std::time::{Duration, Instant};

/// Le plus petit silence dont on puisse conclure que la main a lâché.
///
/// # Pourquoi ce n'est plus [`TAU_CONDUITE`], et ce que leur confusion coûtait
///
/// Un seul nombre tenait les deux rôles : la vitesse à laquelle la vue rattrape la main, et
/// le silence au-delà duquel on la déclare partie. Rien ne les lie, et les baisser ensemble
/// a immédiatement cassé `le_rythme_de_livraison_ne_change_pas_ce_qui_est_montre` : une
/// source à cinq événements par dixième de seconde se voyait déclarée lâchée entre deux
/// événements, quand une source à vingt ne l'était pas. **Le pilote décidait de la fin d'un
/// geste.**
///
/// Cinquante millisecondes restent donc ici, où elles n'ont jamais gêné personne, pendant
/// que le lissage descend. Ce seuil n'est de toute façon qu'un **plancher** : le vrai critère
/// est le pire écart que cette source vient de montrer, et il le dépasse dès qu'elle
/// hoquette.
const SILENCE_MINIMAL: f64 = 0.05;

/// Combien d'intervalles d'émission on garde pour connaître le rythme de la source.
///
/// Seize : assez pour que le pire ne soit pas un accident isolé, assez peu pour qu'un
/// changement de source — la souris après le pavé — se voie en un sixième de seconde.
const INTERVALLES: usize = 16;

/// L'horizon sur lequel une vitesse de geste se mesure.
///
/// # Pourquoi cette durée, et pourquoi elle n'est pas arbitraire
///
/// C'est la latence de la poursuite oculaire : entre cent et cent trente millisecondes
/// séparent le début d'un mouvement du moment où l'œil le suit. C'est donc l'horizon sur
/// lequel **l'œil lui-même** intègre un déplacement — estimer la vitesse sur moins de temps
/// mesurerait quelque chose que personne ne perçoit comme une vitesse.
pub(super) const HORIZON: Duration = Duration::from_millis(100);

/// Le rythme auquel la source émet, et ce qu'elle a demandé récemment.
///
/// # Le piège des rafales, et il a fallu un test pour le voir
///
/// Windows livre parfois plusieurs événements dans la même milliseconde : il les avait retenus.
/// Ces événements représentent pourtant un geste étalé dans le temps — leur **livraison** est
/// groupée, pas leur **cause**.
///
/// Diviser leur somme par l'écart entre le premier et le dernier donne donc une vitesse
/// absurde. Le test l'a chiffré : quarante-quatre mille pixels de glissade là où le même geste,
/// livré régulièrement, en donnait cinq cent quarante-cinq.
///
/// La sortie est de ne jamais diviser par la durée de livraison, mais **par l'horizon**, qui
/// est connu : ce qui est arrivé pendant les cent dernières millisecondes, rapporté à cent
/// millisecondes. Un geste régulier et le même geste livré en rafale donnent alors exactement
/// la même vitesse — c'est la propriété qu'on cherchait, et elle est exacte.
#[derive(Debug, Default)]
pub(super) struct Source {
    /// `(quand, ce qui est arrivé)`, le plus récent en dernier.
    entrees: [(Option<Instant>, Mouvement); INTERVALLES],
    curseur: usize,
    remplies: usize,
}

impl Source {
    /// Note ce qu'un événement apporte, et quand il est arrivé.
    pub(super) fn noter(&mut self, maintenant: Instant, apport: Mouvement) {
        self.entrees[self.curseur] = (Some(maintenant), apport);
        self.curseur = (self.curseur + 1) % INTERVALLES;
        self.remplies = (self.remplies + 1).min(INTERVALLES);
    }

    pub(super) fn vider(&mut self) {
        self.remplies = 0;
        self.curseur = 0;
        self.entrees = [(None, Mouvement::default()); INTERVALLES];
    }

    pub(super) fn dernier(&self) -> Option<Instant> {
        let i = (self.curseur + INTERVALLES - 1) % INTERVALLES;
        self.entrees[i].0
    }

    /// Depuis combien de temps la source se tait.
    pub(super) fn silence(&self, maintenant: Instant) -> f64 {
        self.dernier()
            .map(|t| maintenant.saturating_duration_since(t).as_secs_f64())
            .unwrap_or(f64::INFINITY)
    }

    /// Les événements arrivés dans l'horizon qui précède `maintenant`.
    fn recentes(&self, maintenant: Instant) -> impl Iterator<Item = &(Option<Instant>, Mouvement)> {
        (0..self.remplies)
            .map(move |n| {
                let i = (self.curseur + INTERVALLES - 1 - n) % INTERVALLES;
                &self.entrees[i]
            })
            .take_while(move |(quand, _)| {
                quand.is_some_and(|t| maintenant.saturating_duration_since(t) <= HORIZON)
            })
    }

    /// Le silence au-delà duquel on peut conclure que la main a lâché.
    ///
    /// Le plus grand écart que cette source s'est permis récemment, jamais moins que le temps
    /// de rattrapage : un pavé tactile hoquette, et conclure au lâcher à chaque trou ferait
    /// basculer de régime des dizaines de fois par seconde.
    ///
    /// # Le plancher est [`SILENCE_MINIMAL`], et surtout pas l'intervalle typique
    ///
    /// La version précédente y mettait l'intervalle typique — donc le **nombre** d'événements
    /// reçus. C'était le défaut de la constante de temps, déplacé d'un cran : le même geste
    /// livré en cinq morceaux ou en vingt ne concluait pas au lâcher au même moment, et
    /// montrait donc six pour cent de plus dans un cas que dans l'autre.
    ///
    /// [`SILENCE_MINIMAL`] ne dépend de rien, et il a un sens ici : sous cette durée, un
    /// silence n'est pas une fin de geste mais un hoquet de livraison.
    /// Rend `None` tant qu'il n'y a pas **deux** événements : une vitesse est un rapport, et
    /// un point isolé n'en porte aucune. Un cran de molette seul ne mérite pas de glissade —
    /// il s'applique en douceur, et c'est tout ce qu'on peut savoir de lui.
    pub(super) fn silence_qui_termine(&self, maintenant: Instant) -> Option<f64> {
        if self.remplies < 2 {
            return None;
        }
        let mut pire: f64 = SILENCE_MINIMAL;
        let mut precedent: Option<Instant> = None;
        for (quand, _) in self.recentes(maintenant) {
            if let (Some(t), Some(p)) = (*quand, precedent) {
                pire = pire.max(p.saturating_duration_since(t).as_secs_f64());
            }
            precedent = *quand;
        }
        Some(pire)
    }

    /// La vitesse du geste : ce qui est arrivé dans l'horizon, **rapporté à l'horizon**.
    ///
    /// Jamais à la durée de livraison — c'est toute la leçon des rafales. Deux apports opposés
    /// s'y annulent, donc un demi-tour se voit à l'instant où il se fait.
    pub(super) fn vitesse(&self, maintenant: Instant) -> Mouvement {
        let mut cumul = Mouvement::default();
        for (_, m) in self.recentes(maintenant) {
            cumul.pan.0 += m.pan.0;
            cumul.pan.1 += m.pan.1;
            cumul.octaves += m.octaves;
        }
        let horizon = HORIZON.as_secs_f64();
        Mouvement {
            pan: (cumul.pan.0 / horizon, cumul.pan.1 / horizon),
            octaves: cumul.octaves / horizon,
            ancre: (0.0, 0.0),
        }
    }
}
