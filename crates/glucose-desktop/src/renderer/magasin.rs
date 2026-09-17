//! Le magasin d'images : ce qui est décodé, ce qui est en chemin, ce qui ne viendra jamais.
//!
//! # Pourquoi ce type existe
//!
//! Quatre choses répondaient jusqu'ici séparément à la même question — « puis-je poser cette
//! image, et sous quelle forme ? » : le cache des pyramides, celui des vignettes, la liste des
//! fichiers illisibles, et l'atelier de décodage. Elles voyageaient en quatre paramètres, et
//! la passe des images en comptait huit à elle seule.
//!
//! Ce n'est pas qu'un compte : quatre paramètres qui vont toujours ensemble **sont** un type,
//! et tant qu'ils n'en forment pas un, chaque nouvel appelant doit se souvenir de l'ordre et
//! de ce qu'il faut emprunter en écriture. Le compilateur ne peut rien pour lui.
//!
//! # Les champs restent publics, et c'est délibéré
//!
//! Poser une image demande la pyramide **et** la vignette au même instant, l'une pour lire,
//! l'autre pour écrire. Passer par des méthodes emprunterait le magasin entier et rendrait la
//! chose impossible ; des champs distincts s'empruntent séparément, ce que le compilateur sait
//! vérifier. Le type gagne la cohésion sans rien perdre.

use super::atelier::Atelier;
use super::photo::Pyramide;
use super::vignette::Vignettes;
use std::collections::{HashMap, HashSet};

/// Tout ce qu'il faut pour poser une image sur le canevas.
#[derive(Default)]
pub struct Magasin {
    /// Les images décodées, par chemin de fichier — plusieurs nœuds partagent la même.
    pub cache: HashMap<String, Pyramide>,
    /// Les images prêtes à reporter sans transformation, par nœud (MIP-2).
    pub vignettes: Vignettes,
    /// Les fichiers dont on sait qu'ils ne donneront rien : absents, corrompus, trop grands.
    /// Cache négatif (R-29) — on ne les redemande jamais.
    pub echecs: HashSet<String>,
    /// Les fils qui décodent pendant que la scène continue de se dessiner (DECODE-1).
    pub atelier: Atelier,
}

impl Magasin {
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// L'image déjà décodée pour ce chemin, ou rien — et une demande partie à l'atelier.
    ///
    /// # INVARIANT DECODE-1 — le rendu n'attend jamais un décodage
    ///
    /// Cette méthode ne décode pas. Elle consulte, et si l'image manque elle la **demande**
    /// (une seule fois : l'atelier refuse les doublons) puis rend `None` immédiatement.
    /// L'appelant dessine alors l'image comme ce qu'elle est à cet instant — en chemin.
    ///
    /// Mesuré avant que ce soit vrai : une photo de douze mégapixels coûtait 351 ms dans la
    /// boucle de rendu, soit cent quarante images perdues pour une seule photo. C'est ce que
    /// l'utilisateur décrivait par « importer une image fige tout », et c'était exact.
    pub fn pyramide(&mut self, src: &str) -> Option<&mut Pyramide> {
        if self.echecs.contains(src) {
            return None;
        }
        if self.cache.contains_key(src) {
            return self.cache.get_mut(src);
        }
        self.atelier.demander(src);
        None
    }

    /// Verse dans les caches tout ce que l'atelier a fini de décoder.
    ///
    /// Appelée au début de chaque image, et là seulement : le rendu voit ainsi un cache qui ne
    /// bouge pas sous ses pieds pendant qu'il dessine.
    pub fn recolter(&mut self) {
        for (src, image) in self.atelier.recolter() {
            match image {
                // La pyramide arrive **faite** : il ne reste ici qu'un déplacement de
                // pointeurs. La construire ici coûtait 73 ms en pleine image (DECODE-1).
                Some(pyramide) => {
                    self.cache.insert(src, pyramide);
                }
                // Absent, illisible, ou trop grand pour tenir en mémoire : les trois se
                // constatent de la même façon, et aucun ne se redemande.
                None => {
                    self.echecs.insert(src);
                }
            }
        }
    }

    /// Combien d'images sont encore en cours de décodage.
    pub fn en_travail(&self) -> usize {
        self.atelier.en_travail()
    }

    /// Attend que tout le chantier soit rentré.
    ///
    /// # Le seul endroit où attendre un décodage est juste
    ///
    /// Les **témoins visuels et les bancs**, qui produisent une image de référence et non une
    /// cadence : un témoin rendu avant l'arrivée des photos ne montrerait que des cadres, et
    /// ne prouverait rien. Partout ailleurs, l'invariant DECODE-1 l'interdit — le rendu
    /// n'attend jamais, il dessine ce qu'il a.
    ///
    /// Ne coûte rien quand le chantier est vide, ce qui est le cas de toutes les images sauf
    /// la première. La borne de temps n'est pas un réglage : c'est un garde-fou pour qu'un
    /// fil perdu fasse échouer un test au lieu de le faire tourner sans fin.
    pub fn attendre_le_chantier(&mut self) {
        let depart = std::time::Instant::now();
        while self.en_travail() > 0 && depart.elapsed() < std::time::Duration::from_secs(60) {
            self.recolter();
            std::thread::yield_now();
        }
        self.recolter();
    }

    /// Les octets que les images décodées occupent.
    pub fn octets(&self) -> usize {
        self.cache.values().map(|p| p.octets()).sum()
    }
}

#[cfg(test)]
mod tests;
