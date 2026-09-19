//! Le tempo : combien de balayages chaque image occupe, et **quand** la soumettre pour que ce
//! soit exactement cela.
//!
//! # Le défaut que ce module corrige, chiffré sur le terrain
//!
//! Sur la machine de l'utilisateur, une image coûte 4,87 ms en médiane pour un écran qui bat
//! toutes les 4,17. Une image sur sept rate donc son balayage et en occupe deux : 86 % des
//! images sur un balayage, 13 % sur deux, et une fidélité la plus basse de **0,51** — le
//! rapport exact de un à deux. Le contenu avance du pas d'une image et reste affiché la durée
//! de la suivante, alors sa vitesse apparente est divisée par deux, une image sur sept, sans
//! aucun rapport avec ce qu'on dessine : « que ce soit avec un seul petit texte ou huit cents
//! images ».
//!
//! C'est la limite que [`crate::horloge`] annonçait par un test : tant que l'intervalle
//! d'affichage varie d'une image à l'autre, la vitesse apparente varie encore. Ce module rend
//! cet intervalle **constant**.
//!
//! # Le principe, et il n'a rien d'original
//!
//! Tous les moteurs qui tiennent une cadence le font : on choisit un nombre entier de
//! balayages par image, `k`, et on **soumet** chaque image exactement `k` périodes après la
//! précédente — quitte à attendre quand elle est prête trop tôt. Deux cent quarante images par
//! seconde irrégulières se voient ; cent vingt régulières ne se voient pas.
//!
//! ```text
//!     cible(n) = soumission(n−1) + k · P
//!     prête avant la cible  →  on attend, puis on soumet à la cible
//!     prête après la cible  →  on a raté : on soumet, et k monte d'un cran
//! ```
//!
//! `k` redescend quand le rendu a tenu, avec sa marge, dans `k − 1` périodes depuis assez
//! longtemps pour qu'une oscillation soit un événement rare et non un tremblement.
//!
//! # Ce que l'attente coûte, et ce qu'elle rapporte
//!
//! Elle ajoute de la latence : à `k = 2` et 4,87 ms de rendu, l'image attend 3,5 ms avant de
//! partir. La charte l'assume — c'est « cent images par seconde **constant** », pas « le plus
//! d'images possible ». Et ce temps n'est pas perdu : c'est le temps libre de chaque image, où
//! le travail de fond avance (CASCADE-1), et où l'arbre de possibilités préparera bientôt les
//! tuiles de la trajectoire (fiche 18, étape 3).
//!
//! # Les deux constantes, et pourquoi elles restent
//!
//! La **marge** est celle de [`crate::cadence`] : l'air dont la présentation a besoin. Elle
//! n'est pas redéclarée.
//!
//! L'**horizon** avant lequel `k` ne redescend pas vaut une seconde. Une oscillation entre `k`
//! et `k − 1` coûte deux images irrégulières par cycle ; à un cycle par seconde c'est un
//! événement, à dix c'est un tremblement. Et la cadence se compte en images par **seconde** :
//! c'est l'unité dans laquelle on la juge, donc celle sur laquelle on la stabilise.

use std::time::{Duration, Instant};

/// Le temps pendant lequel `k` ne redescend pas après un balayage raté.
const HORIZON: Duration = Duration::from_secs(1);

/// Le rythme de soumission des images.
#[derive(Debug, Clone, Copy)]
pub struct Tempo {
    /// La période de l'écran. Nulle tant qu'il n'a rien annoncé.
    periode: Duration,
    /// Combien de balayages chaque image occupe. Jamais moins d'un.
    balayages: u32,
    /// L'instant où la dernière image a été soumise.
    derniere_soumission: Option<Instant>,
    /// L'instant visé pour l'image en cours, fixé quand elle est prête.
    ///
    /// Retenu plutôt que recalculé : `k` peut redescendre entre le moment où l'image est prête
    /// et celui où elle part, et la cible de CETTE image ne doit pas bouger avec lui.
    cible: Option<Instant>,
    /// Le début de la fenêtre d'observation en cours, et le rendu le plus long qu'elle a vu.
    ///
    /// La descente de `k` se décide **une fois par horizon**, sur le pire rendu de l'horizon :
    /// c'est lui qui raterait. Une première version gardait le pire depuis le dernier raté ;
    /// il ne s'oubliait jamais, et `k` ne redescendait jamais.
    fenetre: Option<Instant>,
    pire_rendu: Duration,
}

impl Default for Tempo {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Tempo {
    pub fn nouveau() -> Self {
        Self {
            periode: Duration::ZERO,
            balayages: 1,
            derniere_soumission: None,
            cible: None,
            fenetre: None,
            pire_rendu: Duration::ZERO,
        }
    }

    /// Accorde le tempo à l'écran.
    pub fn accorder(&mut self, periode: Duration) {
        self.periode = periode;
    }

    /// Combien de balayages chaque image occupe en ce moment.
    pub fn balayages(&self) -> u32 {
        self.balayages
    }

    /// La durée pendant laquelle une image reste à l'écran, au tempo courant.
    pub fn intervalle(&self) -> Duration {
        self.periode.saturating_mul(self.balayages)
    }

    /// L'image est prête : dit combien attendre avant de la soumettre, et ajuste `k`.
    ///
    /// `rendu` est ce qu'elle a coûté, du début du rendu à maintenant. Rend zéro quand il faut
    /// soumettre tout de suite — parce qu'on est à l'heure, ou parce qu'on est en retard.
    pub fn attente_avant_de_soumettre(&mut self, maintenant: Instant, rendu: Duration) -> Duration {
        let (Some(derniere), false) = (self.derniere_soumission, self.periode.is_zero()) else {
            return Duration::ZERO;
        };
        let cible = derniere + self.intervalle();
        self.cible = Some(cible);
        // **Le balayage est raté** : l'image sera vue une période de plus que prévu, et la
        // suivante vise un cran plus loin pour que cela ne se reproduise pas.
        if maintenant > cible + crate::cadence::MARGE {
            self.balayages += 1;
            // Un raté ouvre une fenêtre neuve : la descente attendra un horizon entier.
            self.fenetre = Some(maintenant);
            self.pire_rendu = Duration::ZERO;
            return Duration::ZERO;
        }
        self.observer_le_rendu(maintenant, rendu);
        cible.saturating_duration_since(maintenant)
    }

    /// Note ce rendu dans la fenêtre en cours ; à la fin de l'horizon, décide si `k` descend.
    ///
    /// `k` redescend d'un cran si le **pire** rendu de l'horizon tient, avec sa marge, dans
    /// `k − 1` périodes. Une décision par horizon : une oscillation est alors un événement,
    /// pas un tremblement.
    fn observer_le_rendu(&mut self, maintenant: Instant, rendu: Duration) {
        let debut = *self.fenetre.get_or_insert(maintenant);
        self.pire_rendu = self.pire_rendu.max(rendu);
        if maintenant.saturating_duration_since(debut) < HORIZON {
            return;
        }
        let en_dessous = self
            .periode
            .saturating_mul(self.balayages.saturating_sub(1));
        if self.balayages > 1 && self.pire_rendu + crate::cadence::MARGE <= en_dessous {
            self.balayages -= 1;
        }
        self.fenetre = Some(maintenant);
        self.pire_rendu = Duration::ZERO;
    }

    /// L'image vient d'être soumise.
    ///
    /// L'instant retenu est la **cible** quand on l'a atteinte, et non « maintenant » : la
    /// grille des soumissions ne dérive alors pas d'un retard d'horloge à chaque image. En
    /// retard, c'est l'instant réel, et la grille se recale dessus.
    pub fn soumise(&mut self, maintenant: Instant) {
        self.derniere_soumission = Some(match self.cible.take() {
            Some(c) if maintenant <= c + crate::cadence::MARGE => c,
            _ => maintenant,
        });
    }

    /// Un repos vient de se terminer : la grille repart de la prochaine soumission.
    pub fn oublier(&mut self) {
        self.derniere_soumission = None;
        self.cible = None;
    }
}

#[cfg(test)]
mod tests;
