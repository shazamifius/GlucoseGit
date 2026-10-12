//! **Les commandes qui attendent des pixels** (ETAGES-1, ORDONNER-2).
//!
//! Un `Ctrl+B` attend les originaux des images qu'il recadre ; un rangement par la couleur,
//! la couleur de chacune. Ni l'une ni l'autre ne fige l'écran : elles réclament ce qui leur
//! manque, l'écran continue de vivre, et chacune s'applique d'un bloc — une seule entrée
//! d'annulation — à l'image où ce qu'elle attendait est là. Elles vivent ensemble parce que
//! la boucle les relance au même instant, avant la peinture, et que le réveil les attend
//! pour la même raison.

use super::rangement::RangementEnAttente;
use super::GlucoseApp;
use crate::interactions::recadrage::LotDeBordures;

/// Ce qui attend des pixels pour s'appliquer.
#[derive(Default)]
pub struct Attentes {
    /// Un `Ctrl+B` dont les originaux reviennent de chez le système.
    pub bordures: Option<LotDeBordures>,
    /// Un rangement par la couleur qui attend ses couleurs.
    pub rangement: Option<RangementEnAttente>,
}

impl Attentes {
    /// Rien n'attend : le réveil n'a pas à repasser pour elles.
    pub fn aucune(&self) -> bool {
        self.bordures.is_none() && self.rangement.is_none()
    }
}

impl GlucoseApp {
    /// Chaque commande en attente redemande ce qui lui manque, ou s'applique si tout est là —
    /// avant la peinture, pour que ce qu'elle change se voie dans l'image qui suit.
    pub(crate) fn poursuivre_les_attentes(&mut self) {
        self.poursuivre_les_bordures();
        self.poursuivre_le_rangement();
    }
}
