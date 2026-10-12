//! **Les signets de vue** (fiche 53 ; ACCES-1, fiche 63) : neuf emplacements par tableau, où la
//! vue revient en volant, enregistrés avec le document.
//!
//! Au clavier, `Ctrl+1` à `Ctrl+9` posent et `1` à `9` ramènent ([`super::shortcuts`]). Sans
//! clavier — au téléphone, à la souris seule —, rien ne les atteignait : l'entrée « Signets… »
//! du menu les montre dans une question dessinée — ceux qui sont posés, où voler d'un toucher, et
//! de quoi en poser un ici, ou en remplacer un quand les neuf sont pris.

use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use glucose_core::types::Viewport;

/// Les neuf emplacements, dans l'ordre des touches.
const CLES: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

impl GlucoseApp {
    /// **La vue où ce signet ramène**, dans le canevas d'aujourd'hui.
    ///
    /// # SIGNET-2 — un signet retient la vue par son **centre**
    ///
    /// Le signet retenait la vue telle quelle : le décalage du coin haut-gauche du monde à
    /// l'écran. Rappelé dans une fenêtre plus petite — le mode référence, sans bande, réduit à
    /// une vignette —, ce qui était au centre partait sur le côté, et l'on « tombait dans le
    /// vide » (son essai du 07/10). Il retient désormais le décalage **par rapport au centre du
    /// canevas**, et se rappelle autour du centre du canevas d'aujourd'hui : même lieu, même
    /// échelle, quelle que soit la fenêtre. Le format du fichier ne change pas ; seuls les
    /// signets posés avant se rappellent décalés d'une demi-fenêtre.
    pub(crate) fn vue_du_signet(&self, cle: char) -> Option<Viewport> {
        let tableau = &self.store.project.active_board_id;
        let vue = self.store.bookmark(tableau, &cle.to_string())?;
        let (cx, cy) = self.centre_du_canevas();
        Some(Viewport {
            x: vue.x + cx,
            y: vue.y + cy,
            ..vue
        })
    }

    /// Pose ce signet sur la vue d'ici, et dit lequel.
    pub(crate) fn poser_le_signet(&mut self, cle: char) -> Option<String> {
        let tableau = self.store.project.active_board_id.clone();
        let (cx, cy) = self.centre_du_canevas();
        let vue = self.store.viewport();
        let vue = Viewport {
            x: vue.x - cx,
            y: vue.y - cy,
            ..vue
        };
        self.store.set_bookmark(&tableau, &cle.to_string(), vue);
        Some(format!("Signet {cle} posé ici"))
    }

    /// Vole vers ce signet ; s'il est vide, dit comment le poser.
    pub(crate) fn voler_vers_le_signet(&mut self, cle: char) -> Option<String> {
        let Some(vue) = self.vue_du_signet(cle) else {
            return Some(format!("Signet {cle} vide -- Ctrl+{cle} le pose ici"));
        };
        self.vol.voler_vers(vue);
        self.mark_dirty();
        None
    }

    /// **Le seul message des signets**, pour le clavier comme pour le menu : poser ne se voit
    /// pas, un signet vide non plus. Un vol **réussi**, lui, se voit — donc il se tait.
    pub(crate) fn annoncer_le_signet(&mut self, message: Option<String>) {
        if let Some(message) = message {
            self.ui.show_toast(message);
        }
    }

    /// **« Signets… »** (ACCES-1) : ceux qui sont posés, puis « Poser ici ».
    pub(crate) fn montrer_les_signets(&mut self) {
        let poses: Vec<usize> = (0..CLES.len())
            .filter(|&i| self.vue_du_signet(CLES[i]).is_some())
            .collect();
        let mut choix: Vec<(String, Reponse)> = poses
            .iter()
            .map(|&i| (format!("Signet {}", CLES[i]), Reponse::Choix(i)))
            .collect();
        choix.push(("Poser un signet ici".into(), Reponse::Oui));
        choix.push(("Annuler".into(), Reponse::Annuler));
        let texte = if poses.is_empty() {
            "Aucun signet. Posé ici, il ramènera à cette vue, d'où que l'on soit."
        } else {
            "Un toucher y vole."
        };
        let question = Question {
            titre: "Signets".into(),
            texte: texte.into(),
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::Signets);
    }

    /// La réponse à « Signets… » : voler, ou poser au premier emplacement libre — et, les neuf
    /// pris, demander lequel remplacer.
    pub(crate) fn suivre_les_signets(&mut self, reponse: Reponse) {
        match reponse {
            Reponse::Choix(i) => {
                if let Some(&cle) = CLES.get(i) {
                    let message = self.voler_vers_le_signet(cle);
                    self.annoncer_le_signet(message);
                }
            }
            Reponse::Oui => match CLES.into_iter().find(|&c| self.vue_du_signet(c).is_none()) {
                Some(cle) => {
                    let message = self.poser_le_signet(cle);
                    self.annoncer_le_signet(message);
                }
                None => self.demander_le_signet_a_remplacer(),
            },
            _ => {}
        }
    }

    /// Les neuf sont pris : lequel prend la vue d'ici ?
    fn demander_le_signet_a_remplacer(&mut self) {
        let mut choix: Vec<(String, Reponse)> = (0..CLES.len())
            .map(|i| (format!("Signet {}", CLES[i]), Reponse::Choix(i)))
            .collect();
        choix.push(("Annuler".into(), Reponse::Annuler));
        let question = Question {
            titre: "Remplacer un signet".into(),
            texte: "Les neuf sont posés : le signet choisi ramènera ici.".into(),
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::RemplacerUnSignet);
    }

    /// La réponse : ce signet prend la vue d'ici.
    pub(crate) fn remplacer_le_signet(&mut self, reponse: Reponse) {
        if let Reponse::Choix(i) = reponse {
            if let Some(&cle) = CLES.get(i) {
                let message = self.poser_le_signet(cle);
                self.annoncer_le_signet(message);
            }
        }
    }
}

#[cfg(test)]
mod tests;
