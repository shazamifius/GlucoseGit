//! **RENOMMER-1 — nommer un dossier** (fiche 120) : sa parole du 18/11, *« il n'y a pas de système
//! pour nommer un dossier »*. `F2` — la touche de l'explorateur de Windows et de VS Code —, le menu
//! d'un dossier et la palette ouvrent la question de Glucose, son nom dans le champ, choisi tout
//! entier : on tape, Entrée renomme. La loi vit dans le noyau
//! (`Store::renommer_le_dossier`) : un seul geste, que `Ctrl+Z` défait ; un nom vide ne nomme rien.

use crate::app::GlucoseApp;
use crate::ui::question::{Champ, Question, Reponse, Suite};

impl GlucoseApp {
    /// **Demande le nouveau nom du dossier choisi.**
    pub(crate) fn renommer_le_dossier(&mut self) {
        let Some((id, nom)) = self.store.dossier_choisi_et_son_nom() else {
            return;
        };
        let (id, nom) = (id.to_string(), nom.to_string());
        let question = Question {
            titre: "Renommer le dossier".to_string(),
            texte: "Son nouveau nom :".to_string(),
            champ: Some(Champ {
                selection: glucose_core::text::Selection::all(&nom),
                texte: nom,
            }),
            choix: vec![
                ("Renommer".to_string(), Reponse::Oui),
                ("Annuler".to_string(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        let board = self.store.project.active_board_id.clone();
        self.demander(question, Suite::RenommerLeDossier { board, id });
    }

    /// **La réponse** : « Renommer » nomme le dossier de ce que le champ dit.
    pub(crate) fn repondre_au_dossier(
        &mut self,
        (board, id): (&str, &str),
        reponse: Reponse,
        nom: Option<String>,
    ) {
        if let (Reponse::Oui, Some(nom)) = (reponse, nom) {
            if self.store.renommer_le_dossier(board, id, &nom) {
                self.mark_dirty();
            }
        }
    }
}

#[cfg(test)]
#[path = "renommer/tests.rs"]
mod tests;
