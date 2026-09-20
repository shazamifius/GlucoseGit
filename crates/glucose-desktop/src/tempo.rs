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
//! # Le typique, et non le pire
//!
//! La première version montait `k` au premier raté et ne le redescendait que si le **pire**
//! rendu de la seconde tenait un cran plus bas. Sur le terrain, un seul pic par seconde — une
//! colonne de tuiles à peindre au zoom, vingt-sept millisecondes — suffisait à le bloquer à
//! sept balayages : quarante-trois images par seconde, régulières, et l'utilisateur les a
//! trouvées « absolument parfaites ». C'est la thèse confirmée, et c'est quand même quatre
//! fois trop lent pour la charte.
//!
//! Un raté isolé est une image irrégulière, et c'est tout : on l'accepte. `k` monte quand les
//! ratés dépassent **un pour cent** des images — le seuil que le verdict de la chronique
//! emploie déjà pour dire qu'un plancher n'en est plus un — et redescend quand **quatre-
//! vingt-dix-neuf pour cent** des rendus auraient tenu un cran plus bas. Le même nombre dans
//! les deux sens, et il ne vient pas d'ici.
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
    /// La fenêtre d'observation en cours : son début, combien d'images elle a vues, combien
    /// ont raté leur balayage, et combien auraient tenu un cran plus bas.
    ///
    /// `k` se décide sur ces comptes, jamais sur une image seule : monter au premier raté et
    /// ne descendre que si le pire tient bloquait `k` en haut au premier pic de la seconde.
    fenetre: Option<Instant>,
    vues: u32,
    rates: u32,
    tiendraient_en_dessous: u32,
}

/// La part d'images irrégulières au-delà de laquelle une cadence n'est plus tenue : une sur
/// cent. C'est le seuil du verdict de la chronique, repris et non redéclaré en esprit.
const TOLERANCE_POUR_CENT: u32 = 1;

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
            vues: 0,
            rates: 0,
            tiendraient_en_dessous: 0,
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
        self.fenetre.get_or_insert(maintenant);
        self.vues += 1;
        let en_dessous = self
            .periode
            .saturating_mul(self.balayages.saturating_sub(1));
        if rendu + crate::cadence::MARGE <= en_dessous {
            self.tiendraient_en_dessous += 1;
        }
        // **Le balayage est raté** : cette image sera vue une période de plus que prévu. Un
        // raté isolé est une image irrégulière, et c'est tout ; c'est leur FREQUENCE qui fait
        // monter `k` — plus d'un pour cent, et jamais sur le premier.
        if maintenant > cible + crate::cadence::MARGE {
            self.rates += 1;
            if self.rates >= 2 && self.rates * 100 > self.vues * TOLERANCE_POUR_CENT {
                self.balayages += 1;
                self.nouvelle_fenetre(maintenant);
            }
            return Duration::ZERO;
        }
        self.decider_a_la_fin_de_l_horizon(maintenant, en_dessous);
        cible.saturating_duration_since(maintenant)
    }

    /// À la fin de l'horizon, `k` redescend si quatre-vingt-dix-neuf pour cent des rendus
    /// auraient tenu un cran plus bas. Une décision par horizon : une oscillation est alors
    /// un événement, pas un tremblement.
    fn decider_a_la_fin_de_l_horizon(&mut self, maintenant: Instant, en_dessous: Duration) {
        let Some(debut) = self.fenetre else {
            return;
        };
        if maintenant.saturating_duration_since(debut) < HORIZON {
            return;
        }
        let assez = self.tiendraient_en_dessous * 100 >= self.vues * (100 - TOLERANCE_POUR_CENT);
        if self.balayages > 1 && !en_dessous.is_zero() && assez {
            self.balayages -= 1;
        }
        self.nouvelle_fenetre(maintenant);
    }

    fn nouvelle_fenetre(&mut self, maintenant: Instant) {
        self.fenetre = Some(maintenant);
        self.vues = 0;
        self.rates = 0;
        self.tiendraient_en_dessous = 0;
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
