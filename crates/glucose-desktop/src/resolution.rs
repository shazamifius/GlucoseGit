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
//! # Pourquoi cela ne peut pas osciller
//!
//! L'écueil d'une résolution adaptative est l'oscillation : on réduit, l'image devient rapide,
//! on rétablit, elle redevient lente. Il est évité **par construction**, en ne retenant jamais
//! la durée observée mais ce qu'elle dit du coût à pleine résolution : `T × f²`. Cette
//! grandeur-là ne dépend pas du facteur choisi, donc la décision qu'elle dicte est la même
//! qu'on l'ait prise à `f = 1` ou à `f = 8`. Le point fixe est atteint dès la seconde image.
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
}

impl Default for Resolution {
    fn default() -> Self {
        Self::nette()
    }
}

impl Resolution {
    /// Pleine résolution : ce qu'on montre dès que la main ne demande plus rien.
    pub fn nette() -> Self {
        Self { facteur: 1 }
    }

    /// De combien la scène est réduite. `1` veut dire « pas du tout ».
    pub fn facteur(&self) -> u32 {
        self.facteur
    }

    /// La scène est-elle rendue plus petite que la fenêtre ?
    pub fn reduite(&self) -> bool {
        self.facteur > 1
    }

    /// Ce que la dernière image a coûté, et ce que la main est en train de faire.
    ///
    /// `duree` est la durée observée **au facteur courant**, d'où la remise à l'échelle : ce
    /// qu'on veut connaître est le coût de la scène, pas celui du compromis qu'on lui a
    /// imposé.
    pub fn observer(&mut self, duree: Duration, budget: Duration, en_mouvement: bool) {
        if !en_mouvement {
            // La netteté revient par moitiés : d'un coup, elle rendrait l'image chère juste
            // au moment où l'œil se pose dessus.
            self.facteur = (self.facteur / 2).max(1);
            return;
        }
        let budget = budget.as_secs_f64().max(f64::MIN_POSITIVE);
        let a_pleine_resolution = duree.as_secs_f64() * f64::from(self.facteur).powi(2);
        // `g ≥ √(coût / budget)` : la surface doit être divisée par le rapport des durées,
        // donc le côté par sa racine.
        let voulu = (a_pleine_resolution / budget).sqrt();
        self.facteur = palier_au_dessus(voulu);
    }
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
