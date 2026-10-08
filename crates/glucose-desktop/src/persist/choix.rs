//! **Ce qu'on choisit dans un sélecteur de fichiers, et ce qui suit** (DIAL-2, fiche 58).
//!
//! Le sélecteur s'ouvre sur un fil à lui ([`crate::dialogue`]) ; Glucose continue de se
//! dessiner. Le choix revient plus tard, avec sa **demande** — ce qu'il doit déclencher. C'est
//! la forme des questions dessinées (POPUP-1) : une suite, pas un retour. Renoncer ne déclenche
//! rien, sinon le report d'une mise à jour qui attendait l'enregistrement.

use crate::app::GlucoseApp;
use crate::persist::close::{Apres, Ensuite, Puis};
use std::path::PathBuf;

/// **Ce que le choix déclenchera.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demande {
    /// Ouvrir ce document à la place de celui qu'on regarde (`Ctrl+O`) — le travail sans nom
    /// qu'on quitte pose d'abord sa question (BROUILLON-1).
    Ouvrir,
    /// Ajouter ce document dans de nouveaux onglets.
    AjouterUnDocument,
    /// Poser ces images.
    ImporterDesImages,
    /// Enregistrer le document ici, puis ce qui suit.
    Enregistrer(Puis),
    /// Écrire ces octets d'image ici, sous cette extension s'il n'en a pas.
    EnregistrerLImage(Vec<u8>, &'static str),
    /// Exporter le tableau ici.
    Exporter,
}

impl GlucoseApp {
    /// **Un choix revient** : sa demande part — ou, si l'on a renoncé, rien.
    pub(crate) fn suivre_le_choix(&mut self, demande: Demande, choisi: Option<Vec<PathBuf>>) {
        let Some(chemins) = choisi.filter(|c| !c.is_empty()) else {
            // La mise à jour attendait l'enregistrement : elle est reportée, et le dit.
            if matches!(
                demande,
                Demande::Enregistrer(Puis::Fermer(Apres::Installer(_)))
            ) {
                self.reporter_la_mise_a_jour();
            }
            return;
        };
        let premier = chemins[0].clone();
        match demande {
            Demande::Ouvrir => self.laisser_puis(Ensuite::Ouvrir(premier)),
            Demande::AjouterUnDocument => self.ajouter_un_document(&premier),
            Demande::ImporterDesImages => self.import_image_files(&chemins),
            Demande::Enregistrer(puis) => {
                self.save_to(crate::persist::with_glucose_extension(premier));
                self.apres_l_enregistrement(puis);
            }
            Demande::EnregistrerLImage(octets, extension) => {
                self.ecrire_l_image_choisie(premier, &octets, extension);
            }
            Demande::Exporter => self.exporter_vers(premier),
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
