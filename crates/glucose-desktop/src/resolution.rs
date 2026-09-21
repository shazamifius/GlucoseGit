//! Combien de pixels la scène mérite pendant qu'on la déplace, et pourquoi jamais plus.
//!
//! # Le constat qui rend ce module nécessaire
//!
//! Un **seul** bloc de texte, zoomé jusqu'à déborder de l'écran, coûtait cent vingt
//! millisecondes par image. Le nombre de nœuds n'y était pour rien : le fond, la grille, le
//! halo, l'aura de la carte et son cadre repeignent chacun les quatre millions de pixels de
//! l'écran. Cinq fois la surface, pour un seul objet.
//!
//! Aucune optimisation passe par passe ne répond à cela, parce que le coût ne vient pas d'une
//! passe : il vient de la **surface**. Et la surface, on peut la choisir.
//!
//! # La loi, et elle est exacte
//!
//! Une passe qui couvre l'écran coûte proportionnellement au nombre de pixels qu'elle écrit.
//! Rendre la scène dans une image `f` fois plus petite coûte donc `f²` fois moins, et
//! l'agrandir à la présentation ne coûte qu'un report — la primitive la moins chère du
//! programme.
//!
//! C'est mot pour mot ce que demande la charte : « rappelle TOUJOURS avoir 100 fps minimum ;
//! si on est en dessous alors go pixeliser tout ». Ici, *tout* veut dire tout : les formes,
//! les textes, les halos, pas seulement les photos.
//!
//! # Ce qui suit la surface, et ce qui n'en dépend pas
//!
//! **La première version de ce module divisait tout, et c'était faux.** Elle appliquait la loi
//! de la surface à la durée *entière* d'une image, alors que l'interface, les panneaux, le
//! téléversement vers la carte et l'agrandissement lui-même n'en dépendent pas du tout. Le
//! calcul divisait donc une part qui ne bougeait pas, constatait que le budget n'était toujours
//! pas tenu, et divisait encore — jusqu'au dernier palier, où il restait bloqué. La scène
//! devenait illisible pour un gain nul.
//!
//! Une image se lit donc en **deux termes** :
//!
//! ```text
//!     T(f) = fixe + scène / f²
//! ```
//!
//! Les deux se mesurent séparément — la scène est chronométrée pour elle-même — donc rien
//! n'est à deviner ni à inverser. Et cela dit une chose que l'ancien modèle ne pouvait pas
//! dire : **quand le terme fixe dépasse à lui seul le budget, aucune réduction ne le tiendra**.
//! Abîmer l'image dans ce cas est perdant deux fois, et le modèle répond alors « pleine
//! résolution », ce qui est à la fois le bon rendu et le bon diagnostic.
//!
//! # Pourquoi cela ne peut pas osciller
//!
//! L'écueil d'une résolution adaptative est l'oscillation : on réduit, l'image devient rapide,
//! on rétablit, elle redevient lente. Il faut donc une grandeur qui ne dépende **pas** du
//! facteur choisi — le coût de la scène à pleine résolution — et décider d'après elle.
//!
//! ## Et cette grandeur, on la MESURE. On l'a longtemps devinée, et c'était faux.
//!
//! Ce module la calculait par `scène × f²`, en écrivant que la loi était « exacte ». Elle ne
//! l'est plus depuis TUILE-1, et pour une raison qui n'a rien d'un détail : **rendre la scène
//! réduite désactive le cache de tuiles** — `Cadrage::reduit` impose le régime direct. Une
//! scène réduite n'est donc pas la même scène divisée par `f²` : c'est un *autre* travail,
//! plus cher par pixel, qui repeint ce que la grille aurait repris.
//!
//! D'où un cercle, et c'est le troisième de la même famille trouvé le même jour. On réduit ;
//! réduire perd les tuiles ; la scène réduite coûte donc presque autant qu'entière ;
//! multipliée par `f²`, elle paraît énorme ; on réduit encore. Sur une session réelle :
//! **47 % des images rendues à un facteur moyen 3,42** — un pixel d'écran pour onze du
//! canevas — pendant que la scène nette, elle, tenait dans le budget.
//!
//! ## La référence est donc le minimum de deux choses, et aucune n'est supposée
//!
//! `scène × f²` reste utile, mais pour ce qu'elle est vraiment : un **majorant**. Une scène
//! réduite coûte *plus* que le quotient exact, donc son produit par `f²` dépasse ce que la
//! scène nette coûterait. Sûr, mais parfois quatre fois trop haut.
//!
//! **Ce que la scène a coûté la dernière fois qu'elle s'est rendue nette** est exact, mais
//! vieillit : tant que la main bouge, le facteur ne revient pas à un et rien ne la rafraîchit.
//!
//! S'y fier *seule* a été mon erreur, et la session suivante l'a montrée sans appel : le
//! facteur se figeait sur le pic qui l'avait déclenché, parce que le chemin du retour passait
//! justement par le majorant. Le minimum des deux garde ce que chacune apporte — le majorant
//! suit l'image courante et fait revenir la netteté **pendant** le mouvement, la mesure le
//! borne et l'empêche de s'emballer.
//!
//! # Et la netteté revient sans à-coup
//!
//! Quand la main s'arrête, rétablir la pleine résolution d'un coup coûterait précisément
//! l'image à cent vingt millisecondes qu'on venait d'éviter — juste au moment où l'œil se
//! pose. Le facteur redescend donc par moitiés : quelques images de plus, aucune qui pique.

use std::time::Duration;

/// Le facteur de réduction ne prend que des puissances de deux.
///
/// Trois raisons qui n'en font qu'une : l'agrandissement au plus proche par un entier donne
/// des blocs exacts sans aucun rééchantillonnage, le tampon ne se réalloue qu'aux rares
/// changements de palier, et la remontée par moitiés retombe toujours sur un palier existant.
const PALIERS: [u32; 4] = [1, 2, 4, 8];

/// Le facteur de réduction courant, et ce qui le décide.
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    facteur: u32,
    /// Ce que la scène a coûté la dernière fois qu'elle s'est rendue **nette**.
    ///
    /// `None` tant qu'aucune image pleine n'a été mesurée dans cette session.
    nette: Option<Duration>,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::nette()
    }
}

impl Resolution {
    /// Pleine résolution : ce qu'on montre dès que la main ne demande plus rien.
    pub fn nette() -> Self {
        Self {
            facteur: 1,
            nette: None,
        }
    }

    /// De combien la scène est réduite. `1` veut dire « pas du tout ».
    pub fn facteur(&self) -> u32 {
        self.facteur
    }

    /// La scène est-elle rendue plus petite que la fenêtre ?
    pub fn reduite(&self) -> bool {
        self.facteur > 1
    }

    /// Ce que la dernière image a coûté, décomposé, et ce que la main est en train de faire.
    ///
    /// `image` est la durée totale et `scene` la part qui suit la surface — chronométrée pour
    /// elle-même, pas déduite. La différence est le terme fixe, que réduire ne touche pas.
    /// `plafond` est ce que l'œil tolère à la vitesse courante ([`crate::perception`]) : le
    /// budget dit ce dont on a **besoin**, le plafond ce qui est **licite**. Dégrader demande
    /// les deux, et c'est l'absence de la seconde condition qui faisait persister les gros
    /// blocs pendant que l'amortissement s'éteignait.
    pub fn observer(&mut self, mesure: Mesure, budget: Duration, en_mouvement: bool, plafond: u32) {
        // **La seule mesure qui vaille référence est celle d'une image nette.** Une image
        // réduite ne dit rien du coût de la scène entière : elle a été rendue autrement, sans
        // les tuiles (voir l'en-tête du module).
        if self.facteur == 1 {
            self.nette = Some(mesure.scene);
        }
        // L'œil ne tolère rien à cette vitesse : il n'y a plus de décision à prendre, quel
        // que soit le budget. C'est le cas de l'arrêt, et celui de toute la fin d'un
        // amortissement — donc celui où la netteté doit revenir sans qu'on la lui demande.
        if !en_mouvement || plafond <= 1 {
            // Par moitiés : revenir d'un coup rendrait l'image chère juste au moment où
            // l'œil se pose dessus.
            self.facteur = (self.facteur / 2).max(1);
            return;
        }
        let budget = budget.as_secs_f64();
        let fixe = (mesure.image.as_secs_f64() - mesure.scene.as_secs_f64()).max(0.0);
        // Ce qui reste à la scène une fois le fixe payé. S'il ne reste rien, aucune réduction
        // ne tiendra le budget : la dégrader serait perdre la netteté ET la cadence.
        let disponible = budget - fixe;
        if disponible <= 0.0 {
            self.facteur = 1;
            return;
        }
        // **Deux estimations du coût de la scène nette, et on prend la plus basse.**
        //
        // `scène × f²` en est un **majorant**, et c'est tout ce qu'on peut en dire : une scène
        // réduite a perdu les tuiles, donc elle coûte *plus* que le quotient exact, donc son
        // produit par `f²` dépasse ce que la scène nette coûterait vraiment. Majorant sûr,
        // mais qui peut être quatre fois trop haut — c'est lui qui emballait la réduction.
        //
        // La mesure d'une image nette, elle, est exacte. Mais elle **vieillit** : tant que la
        // main bouge, le facteur ne revient pas à un, donc rien ne la rafraîchit. S'y fier
        // seule fige le facteur sur le pic qui l'a déclenché — c'est ce que j'ai fait, et la
        // session suivante l'a montré : le chemin du retour avait disparu.
        //
        // Le minimum des deux garde ce que chacune apporte. Le majorant suit l'image courante,
        // donc il redescend dès que la scène réduite devient rapide, et la netteté revient
        // **pendant** le mouvement. La mesure borne le majorant, donc il ne peut plus
        // s'emballer. Aucune n'est supposée : l'une est mesurée, l'autre est une inégalité.
        let majorant = mesure.scene.as_secs_f64() * f64::from(self.facteur).powi(2);
        let a_pleine_resolution = self
            .nette
            .map_or(majorant, |nette| nette.as_secs_f64().min(majorant));
        // `g ≥ √(coût / disponible)` : la surface se divise par le rapport des durées, donc
        // le côté par sa racine. Puis le plafond de l'œil, qui a le dernier mot : on ne
        // dégrade jamais plus que ce que la vitesse rend invisible.
        let voulu = palier_au_dessus((a_pleine_resolution / disponible).sqrt());
        self.facteur = voulu.min(plafond);
    }
}

/// Ce qu'une image a coûté, et la part de ce coût qui suit la surface de la scène.
///
/// Les deux voyagent ensemble parce que ni l'une ni l'autre ne décide seule : une image lente
/// dont la scène ne coûte rien ne se répare pas en rapetissant la scène.
#[derive(Debug, Clone, Copy)]
pub struct Mesure {
    /// La durée entière de l'image.
    pub image: Duration,
    /// Ce que la scène y a pris, elle seule.
    pub scene: Duration,
}

/// Le plus petit palier qui atteint au moins `voulu`.
///
/// Un `voulu` insensé — une durée absurde, une division qui déborde — retombe sur le dernier
/// palier plutôt que sur un facteur qu'aucun tampon ne pourrait porter.
fn palier_au_dessus(voulu: f64) -> u32 {
    if !voulu.is_finite() {
        return PALIERS[PALIERS.len() - 1];
    }
    PALIERS
        .iter()
        .copied()
        .find(|p| f64::from(*p) >= voulu)
        .unwrap_or(PALIERS[PALIERS.len() - 1])
}

#[cfg(test)]
mod tests;
