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
//!     prête après la cible  →  on a raté : on soumet, et la grille repart de là
//! ```
//!
//! # La grille est celle des soumissions, et rien d'autre ne la touche
//!
//! La première version recalait la grille sur l'instant où la **présentation** rendait la
//! main. Or présenter, c'est téléverser, acquérir, encoder et remettre au compositeur : de 1,5
//! à 5 ms sur le terrain, jamais moins que la marge. La grille suivait donc la présentation à
//! chaque image, et chaque intervalle en portait la variation — l'écran montrait `k` ou
//! `k + 1` balayages au hasard. La chronique l'a lu comme un décalage d'un balayage entre le
//! tempo visé et les balayages observés.
//!
//! La grille se fixe donc **ici**, au moment où l'image est prête : à l'heure, la prochaine
//! soumission part de la cible ; en retard, elle part de maintenant. Ce que la présentation
//! coûte ensuite ne la regarde pas.
//!
//! # Le typique, et non le pire — sur un échantillon qui sache le dire
//!
//! Un raté isolé est une image irrégulière, et c'est tout : on l'accepte. `k` monte quand les
//! ratés dépassent **un pour cent** des images — le seuil que le verdict de la chronique
//! emploie déjà pour dire qu'un plancher n'en est plus un — et redescend quand ils seraient
//! restés sous ce seuil un cran plus bas. Le même nombre dans les deux sens, et il ne vient
//! pas d'ici.
//!
//! Encore faut-il qu'un pour cent soit **mesurable**. La première version jugeait la descente
//! sur un horizon d'une seconde : à quarante images par seconde, « quatre-vingt-dix-neuf pour
//! cent tiendraient » se lisait « toutes tiendraient » — c'était le critère du pire, sous un
//! autre nom, et le tempo montait vite et redescendait rarement. L'échantillon est donc compté
//! en **images**, et sa taille se déduit de la tolérance : le plus petit nombre d'images où
//! **deux** ratés la valent exactement, un seul n'étant pas une fréquence. Une décision par
//! échantillon est elle-même une image irrégulière ; à une pour deux cents, elle reste sous la
//! tolérance qu'elle sert.
//!
//! Monter n'attend pas la fin de l'échantillon : dès que les ratés dépassent ce que la
//! tolérance permet sur l'échantillon entier, la conclusion est acquise, et attendre ne ferait
//! que prolonger des images irrégulières. Descendre l'attend, parce que « pas de raté » ne se
//! conclut qu'après avoir regardé.
//!
//! # Ce que l'attente coûte, et ce qu'elle rapporte
//!
//! Elle ajoute de la latence : à `k = 2` et 4,87 ms de rendu, l'image attend 3,5 ms avant de
//! partir. La charte l'assume — c'est « cent images par seconde **constant** », pas « le plus
//! d'images possible ». Et ce temps n'est pas perdu : c'est le temps libre de chaque image, où
//! le travail de fond avance (CASCADE-1), et où l'arbre de possibilités préparera bientôt les
//! tuiles de la trajectoire (fiche 18, étape 3).
//!
//! # La seule constante, et pourquoi elle reste
//!
//! La **marge** est celle de [`crate::cadence`] : l'air dont la présentation a besoin. Elle
//! n'est pas redéclarée.

use std::time::{Duration, Instant};

/// La part d'images irrégulières au-delà de laquelle une cadence n'est plus tenue : une sur
/// cent. C'est le seuil du verdict de la chronique, repris et non redéclaré en esprit.
const TOLERANCE_POUR_CENT: u32 = 1;

/// Le nombre de ratés qu'il faut pour parler d'une fréquence : un seul est un événement.
const UNE_FREQUENCE: u32 = 2;

/// Le nombre d'images sur lequel la tolérance se juge : le plus petit où [`UNE_FREQUENCE`]
/// ratés la valent exactement.
const ECHANTILLON: u32 = UNE_FREQUENCE * 100 / TOLERANCE_POUR_CENT;

/// Le rythme de soumission des images.
#[derive(Debug, Clone, Copy)]
pub struct Tempo {
    /// La période de l'écran. Nulle tant qu'il n'a rien annoncé.
    periode: Duration,
    /// Combien de balayages chaque image occupe. Jamais moins d'un.
    balayages: u32,
    /// L'instant d'où part la grille : la dernière soumission, à l'heure ou non.
    derniere_soumission: Option<Instant>,
    /// L'échantillon en cours : combien d'images il a vues, combien ont raté leur balayage,
    /// et combien l'auraient raté un cran plus bas.
    ///
    /// `k` se décide sur ces comptes, jamais sur une image seule.
    vues: u32,
    rates: u32,
    rateraient_en_dessous: u32,
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
            vues: 0,
            rates: 0,
            rateraient_en_dessous: 0,
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

    /// **Ce qu'une image doit coûter pour que la cadence tienne le plancher de la charte.**
    ///
    /// # Pourquoi ce n'est ni dix millisecondes, ni ce que le tempo tient
    ///
    /// Dix millisecondes est le plancher de la charte, mais l'écran ne sait pas montrer une
    /// image pendant dix millisecondes : il la montre un nombre **entier** de balayages. À
    /// 240 Hz, viser dix, c'est viser entre deux crans — deux balayages en donnent 8,33 et
    /// trois en donnent 12,5. Une image à neuf millisecondes ne tient donc pas le plancher,
    /// elle occupe trois balayages, et le viser ainsi laisse une zone morte où l'on dégrade
    /// sans jamais descendre.
    ///
    /// Viser un cran sous ce que le tempo tient est pire encore, et la mesure l'a dit tout
    /// de suite : quand `k` monte, la cible monte avec lui, donc on dégrade moins, donc `k`
    /// monte encore. Une session entière l'a montré — le tempo est monté jusqu'à neuf
    /// balayages, quarante-trois images par seconde, cinquante millisecondes de latence.
    /// **C'était un cercle, pas un asservissement**, et l'écrire ici est moins cher que de
    /// le redécouvrir.
    ///
    /// La cible est donc le **plus grand nombre entier de balayages qui tienne le plancher**.
    /// Elle ne dépend que de la période lue sur l'écran et du plancher de la charte : à
    /// 240 Hz elle vaut deux balayages, à 60 Hz elle vaut le seul balayage que la machine
    /// sait montrer. Rien ne la fait dériver.
    pub fn cible_pour_descendre(&self) -> Duration {
        let Some(crans) = self.crans_sous_le_plancher() else {
            return crate::cadence::BUDGET_TOTAL;
        };
        self.periode.saturating_mul(crans)
    }

    /// Combien de balayages tiennent dans le plancher de la charte — au moins un, puisque
    /// c'est le plus petit intervalle qu'un écran sache montrer.
    fn crans_sous_le_plancher(&self) -> Option<u32> {
        let periode = self.periode.as_nanos();
        (periode > 0).then(|| {
            u32::try_from(crate::cadence::BUDGET_TOTAL.as_nanos() / periode)
                .unwrap_or(u32::MAX)
                .max(1)
        })
    }

    /// L'image est prête : dit combien attendre avant de la soumettre, fixe le départ de la
    /// grille pour la suivante, et ajuste `k`.
    ///
    /// Rend zéro quand il faut soumettre tout de suite — parce qu'on est à l'heure, ou parce
    /// qu'on est en retard. Rien d'autre n'a à être signalé ensuite : la soumission qui suit
    /// est réputée avoir lieu à l'instant que cette fonction a retenu.
    pub fn attente_avant_de_soumettre(&mut self, maintenant: Instant) -> Duration {
        if self.periode.is_zero() {
            return Duration::ZERO;
        }
        let Some(derniere) = self.derniere_soumission else {
            // La première image part tout de suite : c'est elle qui pose la grille.
            self.derniere_soumission = Some(maintenant);
            return Duration::ZERO;
        };
        let cible = derniere + self.intervalle();
        self.vues += 1;
        // **Le balayage est raté** : cette image sera vue une période de plus que prévu.
        let rate = maintenant > cible + crate::cadence::MARGE;
        if rate {
            self.rates += 1;
        }
        // L'aurait-elle été un cran plus bas ? Même règle, une période plus tôt : c'est ce
        // qui permet de redescendre sur ce qui a été VU, et non sur ce qu'on suppose.
        if maintenant > cible - self.periode + crate::cadence::MARGE {
            self.rateraient_en_dessous += 1;
        }
        self.derniere_soumission = Some(if rate { maintenant } else { cible });
        self.decider();
        if rate {
            Duration::ZERO
        } else {
            cible.saturating_duration_since(maintenant)
        }
    }

    /// Monte dès que les ratés dépassent ce que la tolérance permet ; descend à la fin de
    /// l'échantillon si, un cran plus bas, ils seraient restés en dessous.
    fn decider(&mut self) {
        if self.rates > UNE_FREQUENCE {
            self.balayages += 1;
            self.nouvel_echantillon();
            return;
        }
        if self.vues < ECHANTILLON {
            return;
        }
        if self.balayages > 1 && self.rateraient_en_dessous <= UNE_FREQUENCE {
            self.balayages -= 1;
        }
        self.nouvel_echantillon();
    }

    fn nouvel_echantillon(&mut self) {
        self.vues = 0;
        self.rates = 0;
        self.rateraient_en_dessous = 0;
    }

    /// Un repos vient de se terminer : la grille repart de la prochaine soumission.
    pub fn oublier(&mut self) {
        self.derniere_soumission = None;
    }
}

#[cfg(test)]
mod tests;
