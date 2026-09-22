//! **ARBITRE-1** : quelle carte graphique porte Glucose, décidé en la regardant travailler.
//!
//! # Le problème, et il a coûté une session entière à l'utilisateur
//!
//! Cette machine porte deux cartes : un Intel Arc intégré et une RTX dédiée. `wgpu` retient
//! l'économe par défaut, et sur un portable hybride l'écran est souvent câblé sur la dédiée :
//! chaque image rendue sur l'intégrée traverse alors le bus pour être composée, et **ce
//! chemin gèle**. Mesuré sur le terrain, trois sessions de suite :
//!
//! ```text
//!     Intel Arc      present  median 1,22 ms   p99 65,54 ms   pire 487 ms
//!     RTX 5070       present  median 0,72 ms   p99  1,72 ms   pire   2,35 ms
//! ```
//!
//! Les douze images les plus lentes d'une session sur l'Arc sont **toutes** dominées par
//! `present`, de deux cents à quatre cent quatre-vingt-sept millisecondes — pendant le repos,
//! la sélection, le glissement, indifféremment. `GLUCOSE_CARTE=rapide` réglait la question,
//! mais il fallait y penser à chaque lancement, et le jour où on l'oublie le logiciel devient
//! inutilisable sans qu'on comprenne pourquoi.
//!
//! # Ce que la charte impose, et qui interdit la solution facile
//!
//! Prendre `HighPerformance` d'office serait choisir par une **étiquette**, pas par une
//! mesure — et cela contredirait deux fois la charte : *« on n'interroge pas le matériel, on
//! observe son débit »*, et *« se mettre là où il y a de la place, pour surtout jamais gêner
//! l'utilisateur »*. Sur une machine dont la carte dédiée fait tourner Blender, la bonne
//! réponse est l'intégrée.
//!
//! **L'arbitre essaie, regarde, et garde la meilleure.** Il commence par l'économe — c'est
//! elle qui gêne le moins — et ne change que si elle se montre incapable.
//!
//! # Aucune constante nouvelle, et c'est ce qui rend la règle sûre
//!
//! Trois nombres décident, et les trois existaient déjà :
//!
//! * **ce qu'est un gel** : une image qui dépasse le plancher de la charte
//!   ([`crate::cadence::BUDGET_TOTAL`]) ;
//! * **combien de gels sont de trop** : la part que le tempo tolère avant de monter d'un cran,
//!   et que le verdict emploie pour dire qu'une cadence cloche ;
//! * **au bout de combien de temps juger** : une seconde, et elle se compte en images que
//!   l'écran a montrées — sa cadence est **lue**, jamais supposée.
//!
//! # Pourquoi ce n'est pas le cercle vicieux que ce dépôt a déjà écrit cinq fois
//!
//! La forme d'un cercle est toujours la même : *un mécanisme qui s'adapte à un coût qu'il ne
//! commande pas*. Ici l'arbitre commande exactement ce qu'il mesure — c'est la carte qui
//! produit ces gels, et en changer change la cause.
//!
//! Le garde-fou qui ferme la porte pour de bon : **une carte n'est essayée qu'une fois**.
//! Quand il n'en reste plus à essayer, l'arbitre revient à la meilleure constatée et ne
//! bouge plus. Il ne peut donc pas osciller, quelle que soit la charge de la machine.

use std::time::Duration;

/// Ce qu'une carte a montré pendant qu'on la regardait.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bilan {
    /// Combien d'images ont été présentées et regardées.
    pub observees: u64,
    /// Combien d'entre elles ont dépassé le plancher de la charte **dans `present` seul**.
    pub gels: u64,
}

impl Bilan {
    /// La part d'images que la présentation a fait geler.
    fn part_de_gels(self) -> f64 {
        if self.observees == 0 {
            return 0.0;
        }
        self.gels as f64 / self.observees as f64
    }

    /// Ce bilan est-il meilleur que celui-là ?
    ///
    /// Moins de gels d'abord : une carte qui gèle une fois sur cent est pire qu'une carte
    /// lente et régulière, parce qu'un gel se voit et qu'une milliseconde de plus, non.
    fn vaut_mieux_que(self, autre: Self) -> bool {
        self.part_de_gels() < autre.part_de_gels()
    }
}

/// Ce que l'arbitre demande à l'application de faire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Rien à faire : on continue avec la carte en place.
    Continuer,
    /// Cette carte gèle et une autre n'a pas été essayée : l'ouvrir.
    Essayer(Preference),
    /// Toutes ont été essayées ; celle-ci s'est montrée la meilleure.
    Revenir(Preference),
}

/// Laquelle des cartes de la machine on demande à la couche graphique.
///
/// Un type à nous plutôt que celui de `wgpu` : l'arbitre décide, il ne dessine pas, et rien de
/// ce qui le teste n'a besoin d'une carte graphique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preference {
    /// La plus économe — celle qui gêne le moins les autres logiciels, donc le départ.
    Econome,
    /// La plus puissante.
    Rapide,
}

impl Preference {
    /// L'autre.
    fn autre(self) -> Self {
        match self {
            Self::Econome => Self::Rapide,
            Self::Rapide => Self::Econome,
        }
    }

    /// Le nom qu'en dit la chronique.
    pub fn nom(self) -> &'static str {
        match self {
            Self::Econome => "econome",
            Self::Rapide => "rapide",
        }
    }
}

/// L'arbitre : il regarde la carte en place, et dit quand en essayer une autre (ARBITRE-1).
#[derive(Debug)]
pub struct Arbitre {
    /// La carte en place, et ce qu'elle a montré depuis qu'elle l'est.
    courante: (Preference, Bilan),
    /// Ce que l'autre a montré, si on l'a déjà essayée.
    essayee: Option<(Preference, Bilan)>,
    /// Combien d'images observer avant de juger — une seconde de la cadence lue.
    fenetre: u64,
    /// Une fois la décision prise, elle ne se refait plus.
    tranche: bool,
}

impl Arbitre {
    /// Un arbitre qui commence par cette carte, sur un écran de cette période.
    ///
    /// La fenêtre d'observation vaut **une seconde de cet écran** : à 240 Hz elle compte deux
    /// cent quarante images, à 60 Hz soixante. Elle n'est donc pas choisie, elle se déduit —
    /// et sur une machine lente, où les images sont rares, on attend naturellement plus
    /// longtemps avant de juger.
    pub fn nouveau(depart: Preference, periode: Duration) -> Self {
        let par_seconde = if periode.is_zero() {
            1
        } else {
            (1.0 / periode.as_secs_f64()).round().max(1.0) as u64
        };
        Self {
            courante: (depart, Bilan::default()),
            essayee: None,
            fenetre: par_seconde,
            tranche: false,
        }
    }

    /// La carte en place.
    pub fn courante(&self) -> Preference {
        self.courante.0
    }

    /// Ce que la carte en place a montré jusqu'ici.
    pub fn bilan(&self) -> Bilan {
        self.courante.1
    }

    /// **Note ce que la présentation vient de coûter**, et dit ce qu'il faut en faire.
    ///
    /// `present_us` est le poste `present` seul, pas la durée de l'image : c'est lui que le
    /// chemin hybride fait geler, et une image lente pour une autre raison n'a rien à dire
    /// du choix de la carte.
    pub fn observer(&mut self, present_us: u32) -> Verdict {
        if self.tranche {
            return Verdict::Continuer;
        }
        let plancher = crate::cadence::BUDGET_TOTAL.as_micros() as u32;
        self.courante.1.observees += 1;
        if present_us > plancher {
            self.courante.1.gels += 1;
        }
        if self.courante.1.observees < self.fenetre {
            return Verdict::Continuer;
        }
        self.juger()
    }

    /// La fenêtre est pleine : cette carte tient-elle, et sinon que faire ?
    fn juger(&mut self) -> Verdict {
        // Le seuil du tempo et du verdict, et pas un de plus : une carte qui gèle moins
        // souvent que ce que la cadence tolère n'a aucune raison d'être remplacée.
        if self.courante.1.part_de_gels() <= crate::cadence::PART_TOLEREE {
            self.tranche = true;
            return Verdict::Continuer;
        }
        match self.essayee {
            // L'autre n'a jamais été vue : on l'essaie. La fenêtre repart à zéro pour elle.
            None => {
                let suivante = self.courante.0.autre();
                self.essayee = Some(self.courante);
                self.courante = (suivante, Bilan::default());
                Verdict::Essayer(suivante)
            }
            // Les deux ont été vues : on garde la meilleure, et on ne bouge plus jamais.
            Some(precedente) => {
                self.tranche = true;
                if precedente.1.vaut_mieux_que(self.courante.1) {
                    self.courante = precedente;
                    Verdict::Revenir(precedente.0)
                } else {
                    Verdict::Continuer
                }
            }
        }
    }

    /// La décision est-elle prise pour de bon ?
    pub fn a_tranche(&self) -> bool {
        self.tranche
    }
}

#[cfg(test)]
mod tests;
