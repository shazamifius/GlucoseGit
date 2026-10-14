//! **Ce que la scène a confié à la carte, et le geste de le rendre** (MEMOIRE-1 et 2, fiche 66).
//!
//! Extrait de [`super`], qui approche des six cents lignes : la question n'est pas celle de la
//! pose, mais celle de la mémoire qu'elle coûte. Sur un téléphone, la carte graphique n'a pas de
//! mémoire à elle — chaque texture est prise sur celle du système, la même que celle des autres
//! applications. Son téléphone s'est figé tout entier sans qu'aucune ligne ne dise ce que
//! Glucose y tenait.

use super::SceneGpu;

impl SceneGpu {
    /// **Ce que la carte détient pour la scène**, en octets : chaque texture, posée ou gardée,
    /// photo ou carte de texte. La comptabilité de Glucose, la même sur toutes les plateformes —
    /// un budget que le système dit (DXGI) ne compte pas ce que le pilote y ajoute, celle-ci non
    /// plus, et c'est dit.
    pub fn octets(&self) -> u64 {
        self.photos.values().map(|t| t.octets).sum()
    }

    /// **Rend à la carte ce qui se reconstruit** quand le système le demande (MEMOIRE-2).
    ///
    /// Au premier plan, ce que l'écran pose reste — le rendre ferait redemander aussitôt ce
    /// qu'on vient de lâcher, et l'image se viderait sous les yeux — et tout le reste part,
    /// cache compris, quel que soit le budget. Derrière une autre application, personne ne
    /// regarde : tout part, et le retour redemande ce que l'écran montre (NETTETE-1 finit
    /// l'image d'elle-même). Rien ne se perd : chaque texture se refait depuis le magasin.
    pub fn rendre(&mut self, arriere_plan: bool) {
        if arriere_plan {
            self.photos.clear();
            self.en_cache = 0;
        } else {
            self.fermer(0);
        }
    }
}
