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
//! `GLUCOSE_CARTE=rapide` réglait la question, mais il fallait y penser à chaque lancement, et
//! le jour où on l'oublie le logiciel devient inutilisable sans qu'on comprenne pourquoi.
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
//! # ARBITRE-2 — ce que la première version comptait, et pourquoi elle n'a rien fait
//!
//! Lancée sans `GLUCOSE_CARTE` sur l'Arc, une session de quarante-huit secondes a gelé quatre
//! fois dans `present` — **478, 221, 194 et 156 millisecondes** — et l'arbitre n'a pas bougé.
//! Il obéissait pourtant à sa propre loi. **Deux défauts s'y cachaient, et chacun suffisait à
//! l'aveugler.**
//!
//! ## Premier défaut : la mauvaise grandeur
//!
//! Il comptait des **images gelées** : quatre sur deux mille cinq cent quarante-six font
//! 0,16 %, sous le pour cent que la cadence tolère. C'était reprendre le seuil du tempo sans
//! reprendre sa grandeur — bonne intention, mauvaise unité. Le tempo compte des **ratés**,
//! qui coûtent un balayage chacun ; un gel de 480 ms n'est pas un raté, c'est **quarante-sept
//! images entièrement perdues**.
//!
//! Ce qui se compte ici est donc le **temps perdu au-delà du plancher**, ramené en images :
//!
//! ```text
//!     images perdues = Σ max(0, present − plancher) / plancher
//! ```
//!
//! Sur la même session : 101 images perdues pour 2 546 observées, soit **3,97 %** — vingt-cinq
//! fois ce que l'ancienne lecture annonçait, et franchement au-dessus de la tolérance.
//!
//! ## Second défaut, et c'est lui qui décidait : la porte se fermait au bout d'une seconde
//!
//! L'arbitre jugeait **une** fois, sur la première seconde, puis déclarait la question close.
//! Les quatre gels de la session sont arrivés aux quinzième, vingt-deuxième, vingt-septième et
//! trente-quatrième secondes : **la porte était fermée depuis douze secondes quand le premier
//! est tombé.** Même avec la bonne grandeur, il n'aurait rien vu.
//!
//! Il observe donc en continu, par échantillons renouvelés, exactement comme le tempo — et il
//! ne clôt la question que lorsqu'il n'a plus de carte à essayer.
//!
//! # Aucune constante nouvelle, et une de moins
//!
//! Trois nombres décident, et les trois viennent de [`crate::cadence`], où le tempo les avait
//! déjà établis :
//!
//! * **ce qu'est du temps perdu** : ce qu'une image dépasse le plancher de la charte
//!   ([`crate::cadence::BUDGET_TOTAL`]) ;
//! * **combien d'images perdues sont de trop** : [`crate::cadence::UNE_FREQUENCE`], ce que la
//!   tolérance permet sur un échantillon entier ;
//! * **sur combien d'images juger** : [`crate::cadence::ECHANTILLON`], le plus petit nombre où
//!   un pour cent soit mesurable.
//!
//! La fenêtre « une seconde de l'écran » que la première version s'était donnée **disparaît** :
//! elle jugeait la même tolérance sur un autre échantillon que le tempo, et la charte demande
//! qu'une constante qui peut disparaître disparaisse.
//!
//! # Le premier échantillon ne compte pas, et ce n'est pas une faveur
//!
//! Le gel de démarrage est daté et décomposé depuis la fiche 19 § 5.1 : environ une seconde, à
//! la première seconde, dont deux reconfigurations de surface et le premier téléversement ; et
//! la fiche 22 § 6 mesure `present` à **30 ms sur les seize premières images**. Une mesure
//! prise pendant qu'un pilote s'initialise ne mesure pas un régime. Le premier échantillon est
//! donc un échauffement — deux cents images, douze fois ce que le démarrage occupe — et il en
//! va de même après un changement de carte, qui reconstruit tout ce que la carte détenait.
//!
//! # Pourquoi ce n'est pas le cercle vicieux que ce dépôt a déjà écrit cinq fois
//!
//! La forme d'un cercle est toujours la même : *un mécanisme qui s'adapte à un coût qu'il ne
//! commande pas*. Ici l'arbitre commande exactement ce qu'il mesure — c'est la carte qui
//! produit ces gels, et en changer change la cause.
//!
//! Le garde-fou qui ferme la porte pour de bon : **une carte n'est essayée qu'une fois**.
//! Quand il n'en reste plus à essayer, l'arbitre revient à la meilleure constatée et ne bouge
//! plus. Deux bascules au maximum dans la vie du processus, quelle que soit la charge.
//!
//! # Ce que cette loi accepte, et il faut le dire
//!
//! Trois images perdues sur deux cents suffisent : un hoquet isolé de quarante millisecondes
//! dans `present` fera essayer l'autre carte, une fois. C'est le même arbitrage que le tempo,
//! qui monte d'un cran sur trois ratés, et le coût en est borné — un hoquet de
//! reconfiguration, puis l'arbitre garde la meilleure des deux. L'inverse, lui, a été mesuré :
//! une machine enfermée sur une carte qui gèle, pendant toute une session.

/// Ce qu'une carte a montré pendant qu'on la regardait.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bilan {
    /// Combien d'images ont été présentées et regardées.
    pub observees: u64,
    /// **Le temps que la présentation a fait perdre**, au-delà du plancher, en microsecondes.
    ///
    /// Ce n'est pas ce que `present` a coûté : c'est ce qu'il a coûté **de trop**. Une
    /// présentation qui tient dans le plancher n'y ajoute rien, quelle que soit sa durée.
    pub perdu_us: u64,
}

impl Bilan {
    /// Le plancher de la charte, en microsecondes — la durée d'une image perdue.
    fn plancher_us() -> f64 {
        crate::cadence::BUDGET_TOTAL.as_micros() as f64
    }

    /// **Combien d'images entières la carte a fait perdre**, gel compris.
    ///
    /// Un réel, et non un compte : un gel de 194 ms vaut 18,4 images, pas dix-huit. Arrondir
    /// ici reviendrait à jeter ce que la grandeur a de plus précieux — la gravité.
    fn images_perdues(self) -> f64 {
        self.perdu_us as f64 / Self::plancher_us()
    }

    /// La part des images observées que la présentation a entièrement mangée.
    fn part_perdue(self) -> f64 {
        if self.observees == 0 {
            return 0.0;
        }
        self.images_perdues() / self.observees as f64
    }

    /// Ce bilan est-il meilleur que celui-là ?
    ///
    /// Moins de temps perdu d'abord : une carte qui gèle est pire qu'une carte lente et
    /// régulière, parce qu'un gel se voit et qu'une milliseconde de plus, non.
    fn vaut_mieux_que(self, autre: Self) -> bool {
        self.part_perdue() < autre.part_perdue()
    }

    /// Note ce que cette présentation a coûté de trop.
    fn noter(&mut self, present_us: u32) {
        self.observees += 1;
        self.perdu_us += u64::from(present_us).saturating_sub(Self::plancher_us() as u64);
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
    /// L'échantillon en cours : combien d'images il a vues, et ce qu'elles ont fait perdre.
    ///
    /// Le bilan de la carte cumule toute sa vie — c'est lui qui départage les deux à la fin ;
    /// l'échantillon, lui, se renouvelle, et c'est sur lui que la décision se prend.
    echantillon: Bilan,
    /// Combien d'images d'échauffement restent à jeter avant de commencer à juger.
    echauffement: u32,
    /// Une fois la décision prise, elle ne se refait plus.
    tranche: bool,
}

impl Arbitre {
    /// Un arbitre qui commence par cette carte.
    ///
    /// Il ne demande plus la cadence de l'écran : son échantillon se compte en images, et
    /// [`crate::cadence::ECHANTILLON`] le dit pour tout le projet.
    pub fn nouveau(depart: Preference) -> Self {
        Self {
            courante: (depart, Bilan::default()),
            essayee: None,
            echantillon: Bilan::default(),
            echauffement: crate::cadence::ECHANTILLON,
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
    /// chemin hybride fait geler, et une image lente pour une autre raison n'a rien à dire du
    /// choix de la carte.
    pub fn observer(&mut self, present_us: u32) -> Verdict {
        if self.tranche {
            return Verdict::Continuer;
        }
        if self.echauffement > 0 {
            self.echauffement -= 1;
            return Verdict::Continuer;
        }
        self.courante.1.noter(present_us);
        self.echantillon.noter(present_us);
        // Conclure n'attend pas la fin de l'échantillon : dès que les images perdues dépassent
        // ce que la tolérance permet sur l'échantillon entier, la conclusion est acquise, et
        // attendre ne ferait que prolonger les gels. C'est la règle du tempo, mot pour mot.
        if self.echantillon.images_perdues() > f64::from(crate::cadence::UNE_FREQUENCE) {
            return self.juger();
        }
        if self.echantillon.observees >= u64::from(crate::cadence::ECHANTILLON) {
            self.echantillon = Bilan::default();
        }
        Verdict::Continuer
    }

    /// Cette carte perd plus que toléré : que faire ?
    fn juger(&mut self) -> Verdict {
        match self.essayee {
            // L'autre n'a jamais été vue : on l'essaie. Tout repart à zéro pour elle, et elle
            // a droit au même échauffement — rouvrir une carte lui fait tout reconstruire.
            None => {
                let suivante = self.courante.0.autre();
                self.essayee = Some(self.courante);
                self.courante = (suivante, Bilan::default());
                self.echantillon = Bilan::default();
                self.echauffement = crate::cadence::ECHANTILLON;
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
    ///
    /// Elle ne l'est que lorsqu'il n'y a plus rien à essayer. Sur une machine saine, l'arbitre
    /// ne tranche jamais et ne fait jamais rien : il observe, pour deux additions par image, et
    /// c'est ce qui lui permet de voir un gel qui n'arrive qu'à la trentième seconde.
    pub fn a_tranche(&self) -> bool {
        self.tranche
    }
}

#[cfg(test)]
mod tests;
