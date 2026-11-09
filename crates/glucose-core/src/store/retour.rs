//! **L'histoire de la vue** (RETOUR-1, fiche 99) : les lieux d'où la caméra est partie, pour y
//! revenir (`Alt+Z`), et ceux d'où l'on est revenu, pour y repartir (`Alt+Maj+Z`) — le retour et
//! l'avance d'un navigateur.
//!
//! Elle vit dans le magasin, à côté de la vue de chaque tableau et de la pile des dossiers : c'est
//! de la navigation, qui n'entre jamais dans l'annulation du document ; un document qu'on remplace
//! l'emporte avec lui (chaque document nomme son premier tableau `main` : un lieu de l'ancien
//! s'appliquerait au nouveau). **Aucune borne** : un lieu pèse quelques dizaines d'octets.

use crate::types::Viewport;

/// Un lieu de la caméra : un tableau, et la vue qu'on y avait.
#[derive(Debug, Clone, PartialEq)]
pub struct Lieu {
    pub tableau: String,
    pub vue: Viewport,
}

/// Les lieux d'où l'on est parti, et ceux d'où l'on est revenu.
#[derive(Debug, Clone, Default)]
pub struct HistoireDeLaVue {
    arriere: Vec<Lieu>,
    avant: Vec<Lieu>,
}

impl HistoireDeLaVue {
    /// Un départ quitte `lieu` : il se retient, et l'avant s'efface — on est parti ailleurs.
    pub fn noter(&mut self, lieu: Lieu) {
        if self.arriere.last() != Some(&lieu) {
            self.arriere.push(lieu);
        }
        self.avant.clear();
    }

    /// Le lieu où revenir depuis `ici`, qui se garde pour y repartir.
    pub fn reculer(&mut self, ici: Lieu) -> Option<Lieu> {
        let la = self.arriere.pop()?;
        self.avant.push(ici);
        Some(la)
    }

    /// Le lieu d'où l'on était revenu ; `ici` se retient pour y revenir encore.
    pub fn avancer(&mut self, ici: Lieu) -> Option<Lieu> {
        let la = self.avant.pop()?;
        self.arriere.push(ici);
        Some(la)
    }

    /// Y a-t-il un lieu où revenir ?
    pub fn peut_reculer(&self) -> bool {
        !self.arriere.is_empty()
    }

    /// Tout oublier : le document a été remplacé.
    pub fn oublier(&mut self) {
        self.arriere.clear();
        self.avant.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lieu(x: f64) -> Lieu {
        Lieu {
            tableau: "main".into(),
            vue: Viewport {
                x,
                y: 0.0,
                scale: 1.0,
            },
        }
    }

    /// **Le retour et l'avance d'un navigateur** : on remonte les départs un à un, on repart d'où
    /// l'on était revenu, et un départ neuf efface l'avant ; le même lieu noté deux fois n'en fait
    /// qu'un.
    #[test]
    fn retour_1_l_histoire_se_remonte_comme_celle_d_un_navigateur() {
        let mut h = HistoireDeLaVue::default();
        h.noter(lieu(0.0));
        h.noter(lieu(0.0));
        h.noter(lieu(100.0));
        assert_eq!(h.reculer(lieu(200.0)), Some(lieu(100.0)));
        assert_eq!(h.reculer(lieu(100.0)), Some(lieu(0.0)));
        assert!(
            !h.peut_reculer(),
            "le même lieu noté deux fois n'en fait qu'un"
        );
        assert_eq!(h.avancer(lieu(0.0)), Some(lieu(100.0)));
        h.noter(lieu(150.0));
        assert_eq!(
            h.avancer(lieu(300.0)),
            None,
            "un départ neuf efface l'avant"
        );
        assert_eq!(h.reculer(lieu(300.0)), Some(lieu(150.0)));
    }

    /// **Un document remplacé emporte ses lieux** (le magasin les oublie au chargement).
    #[test]
    fn retour_1_un_document_charge_oublie_les_lieux_de_l_autre() {
        let mut store = crate::store::Store::new("ancien");
        store.histoire_de_la_vue.noter(lieu(0.0));
        let nouveau = crate::store::Store::new("nouveau").project;
        store.load_project(nouveau);
        assert!(!store.histoire_de_la_vue.peut_reculer());
    }
}
