//! **Les commandes qui attendent des pixels** (ETAGES-1, ORDONNER-2).
//!
//! Un `Ctrl+B` attend les originaux des images qu'il recadre ; un rangement par la couleur,
//! la couleur de chacune. Ni l'une ni l'autre ne fige l'écran : elles réclament ce qui leur
//! manque, l'écran continue de vivre, et chacune s'applique d'un bloc — une seule entrée
//! d'annulation — à l'image où ce qu'elle attendait est là. Elles vivent ensemble parce que
//! la boucle les relance au même instant, avant la peinture, et que le réveil les attend
//! pour la même raison.
//!
//! **L'avis de l'IA sur les textes** (PARENTE-2, fiche 106) entre au même instant : un fil le
//! calcule quand les textes changent, et il se pose ici, avant la peinture, pour que l'image qui
//! suit le montre — une seule.

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
    /// **Un export en image**, sur son fil (EXPORT-PNG-1). Il ne compte pas dans
    /// [`Attentes::aucune`] : il réveille la boucle lui-même à chaque bande, et rien n'est à
    /// redessiner entre deux.
    pub export: Option<crate::persist::export::png::EnCours>,
    /// **Les brouillons tenus au lancement** (LIBERE-1, fiche 72), attendus chacun sur son
    /// fil ; comme l'export, ils réveillent la boucle eux-mêmes.
    pub liberes: Option<crate::persist::reprise::Liberes>,
    /// **Un détourage** (DETOURER-1, fiche 121) : tant qu'il attend ses originaux, la boucle
    /// repasse ; son fil, lui, la réveille quand il a fini.
    pub detourage: Option<crate::interactions::detourer::Detourage>,
    /// **Le document où les nœuds couchés ont été cherchés** (EXIF-1) : son identité, pour les
    /// chercher encore dans un autre qu'on ouvre.
    pub redresse_pour: String,
}

impl Attentes {
    /// Rien n'attend : le réveil n'a pas à repasser pour elles.
    pub fn aucune(&self) -> bool {
        self.bordures.is_none()
            && self.rangement.is_none()
            && self.detourage.as_ref().is_none_or(|d| d.au_travail())
    }
}

impl GlucoseApp {
    /// Chaque commande en attente redemande ce qui lui manque, ou s'applique si tout est là —
    /// avant la peinture, pour que ce qu'elle change se voie dans l'image qui suit.
    pub(crate) fn poursuivre_les_attentes(&mut self) {
        self.lire_le_miroir_ou_l_on_entre();
        self.poursuivre_les_bordures();
        self.poursuivre_le_rangement();
        self.poursuivre_le_detourage();
        self.suivre_la_parente();
        self.redresser_les_photos();
    }

    /// **Une photo tournée attend que ses nœuds couchés se redressent** (EXIF-1) — pas pendant un
    /// geste : il attend qu'il se ferme, et une frappe de texte n'a pas à réveiller la boucle.
    pub(crate) fn redressement_en_attente(&self) -> bool {
        self.renderer.magasin.tournees_neuves && !self.store.in_live_edit()
    }

    /// **Les nœuds posés quand leur photo arrivait couchée se redressent** (EXIF-1, fiche 165
    /// § 3.3) : quand une photo tournée arrive, et dans un autre document qu'on ouvre sur celles
    /// déjà là. Un seul geste, signé « Glucose » (`Store::redresser_les_photos`). Le passé qu'on
    /// regarde est un autre document : ce qui s'y redresse ne s'écrit pas, et le présent se
    /// redresse au retour.
    fn redresser_les_photos(&mut self) {
        let autre_document = self.attentes.redresse_pour != self.store.identite;
        let magasin = &mut self.renderer.magasin;
        if !(magasin.tournees_neuves || autre_document) {
            return;
        }
        let Some(redresses) = self.store.redresser_les_photos(&magasin.tournees) else {
            return;
        };
        magasin.tournees_neuves = false;
        self.attentes.redresse_pour.clone_from(&self.store.identite);
        if redresses > 0 {
            self.mark_dirty();
        }
    }

    /// **L'avis de l'IA**, s'il est neuf, passe au rendu ; toute l'image se refait — chaque halo,
    /// chaque ornement de texte peut avoir changé de couleur.
    fn suivre_la_parente(&mut self) {
        if self.renderer.sens_des_textes.suivre(&self.store) {
            self.mark_dirty();
        }
    }
}

#[cfg(test)]
mod tests;
