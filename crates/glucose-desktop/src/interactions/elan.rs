//! L'élan de la caméra : ce que la main demande, et ce qu'il en reste quand elle lâche.
//!
//! # Les deux défauts que cette version corrige, et ils avaient la même cause
//!
//! L'utilisateur en a nommé deux, sans savoir qu'ils n'en faisaient qu'un :
//!
//! * « **des sauts d'image comme si on avait 15 fps**, alors que les logs afficheront 102 » ;
//! * « quatre fois en deux minutes, **je pars dans la direction opposée et l'algo me remet
//!   dans la direction précédente** ».
//!
//! La cause tenait à une confusion de **repère temporel**. Un pavé tactile émet une centaine
//! d'événements par seconde ; l'écran en affiche deux cent quarante. Plus d'une image sur deux
//! ne reçoit donc **rien**, et les événements arrivent par rafales quand le système en a
//! retenu plusieurs.
//!
//! La version précédente convertissait « ce qui est arrivé pendant cette image » en vitesse en
//! divisant par la durée de l'**image**. Or ce déplacement couvre la durée écoulée depuis
//! l'**événement** précédent — deux fois et demie plus longue en moyenne, et bien davantage
//! après une rafale. La vitesse déduite était donc surestimée, et surtout **erratique** : elle
//! dépendait du hasard du groupement. D'où les sauts.
//!
//! Et comme elle basculait entre deux régimes à chaque événement — « la main pousse » /
//! « la main a lâché » — sa fenêtre de mesure se vidait des dizaines de fois par seconde et ne
//! contenait jamais qu'un seul échantillon. Le frein reposait sur une mesure d'un point.
//!
//! # Ce qui remplace tout cela : un **reste à parcourir**
//!
//! La caméra ne poursuit plus une vitesse. Elle a une **dette** — ce que la main a demandé et
//! que l'écran n'a pas encore montré — et chaque image en rembourse une fraction :
//!
//! ```text
//!     montré = reste × (1 − e^(−dt/τ))        puis     reste −= montré
//! ```
//!
//! C'est l'intégrale exacte de l'amortissement sur la durée de l'image, donc le trajet ne
//! dépend pas de la cadence. Et trois propriétés tombent d'elles-mêmes :
//!
//! * **une rafale ne saute plus** : trois événements arrivés ensemble s'ajoutent à la dette,
//!   qui se rembourse lissée sur les images suivantes ;
//! * **un silence ne gèle plus** : la dette continue de se rembourser même sans événement ;
//! * **le frein est exact et gratuit** : repartir en sens inverse **soustrait** de la dette.
//!   Il n'y a plus d'état à basculer ni de fenêtre à vider — deux directions opposées
//!   s'annulent parce que ce sont des nombres, et qu'on les additionne.
//!
//! # Tout en même temps, parce que c'est ainsi qu'on navigue
//!
//! « À la fois on va à droite, à la fois on va en haut, et en plus on zoome et on dézoome et
//! on part de l'autre côté. » La dette est un **vecteur** : deux composantes de déplacement et
//! une d'échelle, chacune remboursée avec sa propre constante de temps. Aucune ne parle aux
//! autres, donc une diagonale reste une diagonale et un zoom simultané ne la perturbe pas.
//!
//! # Le rythme de la source s'observe, il ne se suppose pas
//!
//! Pendant que la main pousse, la constante de temps vaut l'**intervalle d'émission mesuré** :
//! la dette se rembourse à peu près aussi vite qu'elle se contracte, donc le contenu suit la
//! main de près tout en lissant les rafales. Quand la main lâche, elle passe aux constantes de
//! glissade, bien plus longues.
//!
//! Et « la main a lâché » ne se décide pas non plus : c'est un silence plus long que le pire
//! intervalle que cette source ait montré récemment. Aucune durée n'a été choisie.

use std::time::{Duration, Instant};

/// Ce qu'une glissade de déplacement met à s'éteindre.
///
/// Après cette durée il reste 37 % du chemin, et 5 % après trois fois plus. Lâcher à mille
/// pixels par seconde emporte donc quatre cent cinquante pixels — c'est du ressenti, et c'est
/// le seul nombre de ce module qui se juge à la main.
const TAU_LIBRE_PAN: f64 = 0.45;

/// Ce qu'une glissade de zoom met à s'éteindre.
///
/// Plus courte que celle du déplacement : l'échelle change vite d'ordre de grandeur, et une
/// glissade longue y devient un vol plané dont on ne sait plus où il s'arrête.
const TAU_LIBRE_ZOOM: f64 = 0.28;

/// Le temps que met la vue à rattraper la main pendant qu'elle pousse.
///
/// # Pourquoi il est **constant**, et ce que coûtait de le calculer
///
/// La version précédente le déduisait du rythme de la source, à chaque image : l'horizon
/// divisé par le nombre d'événements qu'il contenait. Ce nombre oscille — cinq, puis douze,
/// puis huit, selon ce que le pilote a livré — donc la constante de temps oscillait avec lui,
/// entre six et vingt millisecondes. **La fraction remboursée variait d'un facteur trois d'une
/// image à l'autre, pour une main parfaitement régulière.** C'est une saccade, et c'est ce que
/// l'utilisateur voyait.
///
/// La preuve était sous les yeux : le vol de caméra, qui emploie la même équation avec une
/// constante **fixe**, a toujours été jugé fluide — « quand on fait F ou Ctrl+1, il n'y a
/// presque aucun problème ».
///
/// Cinquante millisecondes absorbent les deux irrégularités de la livraison — les rafales, qui
/// durent quelques millisecondes, et les trous, quelques dizaines — tout en restant bien sous
/// le seuil où un retard se perçoit, qui est la latence de la poursuite oculaire.
const TAU_CONDUITE: f64 = 0.05;

/// Le plus long qu'une image puisse durer sans que l'élan franchisse d'un coup ce qu'elle a
/// manqué.
///
/// Une image de trois secondes — un dialogue natif ouvert, une fenêtre réduite — rembourserait
/// sinon la dette entière en une fois, c'est-à-dire par une téléportation.
const PAS_MAX: Duration = Duration::from_millis(100);

/// Combien d'intervalles d'émission on garde pour connaître le rythme de la source.
///
/// Seize : assez pour que le pire ne soit pas un accident isolé, assez peu pour qu'un
/// changement de source — la souris après le pavé — se voie en un sixième de seconde.
const INTERVALLES: usize = 16;

/// Ce qu'une image doit montrer du mouvement de la caméra.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mouvement {
    /// Le déplacement, en pixels d'écran.
    pub pan: (f64, f64),
    /// Le changement d'échelle, en octaves : `+1` double.
    pub octaves: f64,
    /// Le point d'écran autour duquel l'échelle change.
    ///
    /// Une ancre est un **lieu**, pas une quantité : elle ne s'additionne pas et ne se
    /// consomme pas. La confondre avec le reste faisait tourner tout le zoom autour du coin de
    /// l'écran dès que la main lâchait.
    pub ancre: (f64, f64),
}

impl Mouvement {
    fn existe(&self) -> bool {
        self.pan != (0.0, 0.0) || self.octaves != 0.0
    }

    /// Ce qui se verrait de ce mouvement, en pixels, sur un écran de cette diagonale.
    ///
    /// Le déplacement et le zoom se comparent ainsi dans la **seule unité** où « ça se voit »
    /// veut dire quelque chose : un dixième d'octave ne dit rien tant qu'on ne sait pas sur
    /// quelle surface il s'applique.
    fn en_pixels(&self, diagonale: f64) -> f64 {
        let deplacement = self.pan.0.hypot(self.pan.1);
        let echelle = diagonale / 2.0 * (self.octaves.exp2() - 1.0).abs();
        deplacement.max(echelle)
    }
}

/// L'horizon sur lequel une vitesse de geste se mesure.
///
/// # Pourquoi cette durée, et pourquoi elle n'est pas arbitraire
///
/// C'est la latence de la poursuite oculaire : entre cent et cent trente millisecondes
/// séparent le début d'un mouvement du moment où l'œil le suit. C'est donc l'horizon sur
/// lequel **l'œil lui-même** intègre un déplacement — estimer la vitesse sur moins de temps
/// mesurerait quelque chose que personne ne perçoit comme une vitesse.
const HORIZON: Duration = Duration::from_millis(100);

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
struct Source {
    /// `(quand, ce qui est arrivé)`, le plus récent en dernier.
    entrees: [(Option<Instant>, Mouvement); INTERVALLES],
    curseur: usize,
    remplies: usize,
}

impl Source {
    /// Note ce qu'un événement apporte, et quand il est arrivé.
    fn noter(&mut self, maintenant: Instant, apport: Mouvement) {
        self.entrees[self.curseur] = (Some(maintenant), apport);
        self.curseur = (self.curseur + 1) % INTERVALLES;
        self.remplies = (self.remplies + 1).min(INTERVALLES);
    }

    fn vider(&mut self) {
        self.remplies = 0;
        self.curseur = 0;
        self.entrees = [(None, Mouvement::default()); INTERVALLES];
    }

    fn dernier(&self) -> Option<Instant> {
        let i = (self.curseur + INTERVALLES - 1) % INTERVALLES;
        self.entrees[i].0
    }

    /// Depuis combien de temps la source se tait.
    fn silence(&self, maintenant: Instant) -> f64 {
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
    /// # Le plancher est [`TAU_CONDUITE`], et surtout pas l'intervalle typique
    ///
    /// La version précédente y mettait l'intervalle typique — donc le **nombre** d'événements
    /// reçus. C'était le défaut de la constante de temps, déplacé d'un cran : le même geste
    /// livré en cinq morceaux ou en vingt ne concluait pas au lâcher au même moment, et
    /// montrait donc six pour cent de plus dans un cas que dans l'autre.
    ///
    /// `TAU_CONDUITE` ne dépend de rien, et il a un sens ici : conclure au lâcher plus vite
    /// que le temps qu'on met à rattraper la main n'aurait de toute façon aucun effet visible.
    /// Rend `None` tant qu'il n'y a pas **deux** événements : une vitesse est un rapport, et
    /// un point isolé n'en porte aucune. Un cran de molette seul ne mérite pas de glissade —
    /// il s'applique en douceur, et c'est tout ce qu'on peut savoir de lui.
    fn silence_qui_termine(&self, maintenant: Instant) -> Option<f64> {
        if self.remplies < 2 {
            return None;
        }
        let mut pire: f64 = TAU_CONDUITE;
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
    fn vitesse(&self, maintenant: Instant) -> Mouvement {
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

/// Ce que la main a demandé et que l'écran n'a pas encore montré.
#[derive(Debug, Default)]
pub struct Elan {
    /// La dette : ce qui reste à montrer.
    reste: Mouvement,
    ancre: (f64, f64),
    source: Source,
    dernier_pas: Option<Instant>,
    /// La main a-t-elle lâché ?
    libre: bool,
}

impl Elan {
    /// La main demande un déplacement de tant de pixels d'écran.
    ///
    /// # L'instant est une **donnée**, et ce n'est pas un détail
    ///
    /// Ce module lit le temps entre les **événements**, pas entre les images : c'est tout son
    /// objet. Aller chercher l'horloge ici le rendrait intestable — un test ne pourrait jouer
    /// ni une rafale, ni un rythme régulier, ni un silence, c'est-à-dire précisément les trois
    /// situations où les défauts vivaient. La fiche 13 en a fait une règle après le double-clic.
    pub fn pousser_pan(&mut self, dx: f64, dy: f64, maintenant: Instant) {
        self.reprendre_la_main();
        self.reste.pan.0 += dx;
        self.reste.pan.1 += dy;
        self.source.noter(
            maintenant,
            Mouvement {
                pan: (dx, dy),
                ..Mouvement::default()
            },
        );
    }

    /// La main demande un changement d'échelle, autour de ce point.
    pub fn pousser_zoom(&mut self, octaves: f64, ancre: (f64, f64), maintenant: Instant) {
        self.reprendre_la_main();
        self.ancre = ancre;
        self.reste.octaves += octaves;
        self.source.noter(
            maintenant,
            Mouvement {
                octaves,
                ..Mouvement::default()
            },
        );
    }

    /// Reprendre la main **tue la glissade en cours**, et c'est le frein.
    ///
    /// L'inertie était une prédiction de ce que la main aurait fait ; la main vient de dire
    /// autre chose. La garder ferait exactement ce que l'utilisateur décrit — « je pars dans
    /// la direction opposée et l'algo me remet dans la direction précédente ».
    ///
    /// Une fois la main revenue, plus rien n'a besoin d'être jeté : les apports s'ajoutent à la
    /// dette, donc deux directions opposées s'annulent d'elles-mêmes.
    fn reprendre_la_main(&mut self) {
        if self.libre {
            self.libre = false;
            self.reste = Mouvement::default();
            self.source.vider();
        }
    }

    /// Reste-t-il quelque chose à montrer ?
    pub fn en_cours(&self) -> bool {
        self.reste.existe()
    }

    /// Ce que cette image doit montrer, ou rien s'il n'y a plus de dette.
    pub fn avancer(&mut self, maintenant: Instant, diagonale: f64) -> Option<Mouvement> {
        let dt = self
            .dernier_pas
            .replace(maintenant)
            .map(|t| maintenant.saturating_duration_since(t).min(PAS_MAX))
            .unwrap_or(PAS_MAX)
            .as_secs_f64();

        self.constater_le_lacher(maintenant);

        // Moins d'un demi-pixel à montrer : on solde la dette plutôt que de s'en approcher
        // indéfiniment sans jamais l'atteindre.
        if self.reste.en_pixels(diagonale) < 0.5 {
            self.reste = Mouvement::default();
            return None;
        }

        let (tau_pan, tau_zoom) = self.constantes();
        let part = |tau: f64| 1.0 - (-dt / tau.max(1e-6)).exp();
        let montre = Mouvement {
            pan: (
                self.reste.pan.0 * part(tau_pan),
                self.reste.pan.1 * part(tau_pan),
            ),
            octaves: self.reste.octaves * part(tau_zoom),
            ancre: self.ancre,
        };
        self.reste.pan.0 -= montre.pan.0;
        self.reste.pan.1 -= montre.pan.1;
        self.reste.octaves -= montre.octaves;
        montre.existe().then_some(montre)
    }

    /// La main a-t-elle lâché ? Si oui, l'inertie entre dans la dette.
    ///
    /// Le silence qui le dit est celui que **cette source** a montré de pire récemment : un
    /// pavé tactile, une molette et un pilote qui hoquette n'ont pas le même rythme, et aucun
    /// nombre écrit ici ne saurait valoir pour les trois.
    fn constater_le_lacher(&mut self, maintenant: Instant) {
        if self.libre {
            return;
        }
        let Some(pire) = self.source.silence_qui_termine(maintenant) else {
            return;
        };
        if self.source.silence(maintenant) <= pire {
            return;
        }
        self.libre = true;
        // **La vitesse se lit à l'instant du dernier événement**, et non à celui où l'on
        // constate le lâcher. Les deux diffèrent d'une durée d'image, qui dépend de l'écran :
        // les confondre faisait glisser de mille deux cent soixante pixels à 240 Hz contre
        // neuf cent vingt-quatre à 30 Hz, pour le même geste. La vitesse d'un geste est celle
        // qu'il avait quand il s'est terminé, pas quand on s'en est aperçu.
        let fin = self.source.dernier().unwrap_or(maintenant);
        // Ce qu'une glissade amortie parcourt depuis cette vitesse : `v · τ`. L'inertie est
        // donc une dette de plus, et non un régime à part — c'est ce qui permet au frein de
        // l'annuler d'une soustraction.
        let v = self.source.vitesse(fin);
        self.reste.pan.0 += v.pan.0 * TAU_LIBRE_PAN;
        self.reste.pan.1 += v.pan.1 * TAU_LIBRE_PAN;
        self.reste.octaves += v.octaves * TAU_LIBRE_ZOOM;
    }

    /// Les constantes de temps du régime courant.
    ///
    /// Pendant que la main pousse, la vue la rattrape en [`TAU_CONDUITE`] — assez pour lisser
    /// les irrégularités de livraison, assez peu pour que le retard ne se perçoive pas. Quand
    /// la main a lâché, ce sont les constantes de glissade.
    ///
    /// **Aucune des trois ne dépend de l'instant**, et c'est tout l'objet de cette version :
    /// une constante de temps qui change d'une image à l'autre fait varier ce qui est montré
    /// alors que le geste, lui, n'a pas changé.
    fn constantes(&self) -> (f64, f64) {
        if self.libre {
            return (TAU_LIBRE_PAN, TAU_LIBRE_ZOOM);
        }
        (TAU_CONDUITE, TAU_CONDUITE)
    }
}

#[cfg(test)]
mod tests;
