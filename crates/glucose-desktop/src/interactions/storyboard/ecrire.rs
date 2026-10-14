//! **Écrire dans une case, nommer un champ** (STORYBOARD-1, fiche 67) : une question dessinée,
//! un champ que le clavier remplit — au téléphone aussi, par le miroir (CLAVIER-1).

use crate::app::GlucoseApp;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use glucose_core::store::storyboard::Texte;

impl GlucoseApp {
    /// **Tab dans un texte d'une case** : il s'enregistre, et le même texte de la suivante
    /// s'écrit — Maj+Tab, la précédente. Légender un storyboard d'une traite, comme dans Boords
    /// ou un tableur (fiche 65 § 9). Après la dernière, la question se referme. Rend vrai si la
    /// question posée écrivait dans une case.
    pub(crate) fn decrire_la_voisine(&mut self, arriere: bool) -> bool {
        let Some((_, Suite::EcrireLaCase { board, case, texte })) = &self.ui.question else {
            return false;
        };
        let texte = *texte;
        let cases = self.store.cases(board);
        let rang = cases.iter().position(|c| c.id == *case);
        let voisine = match (rang, arriere) {
            (Some(r), true) => r.checked_sub(1),
            (Some(r), false) => Some(r + 1),
            (None, _) => None,
        };
        let voisine = voisine.and_then(|r| cases.get(r)).map(|c| c.id.clone());
        self.repondre_a_la_question(Some(Reponse::Oui));
        if let Some(id) = voisine {
            self.ecrire_la_case(&id, texte);
        }
        self.mark_dirty();
        true
    }

    /// **Décrit une case** — sa légende, sous son cadre.
    pub(crate) fn decrire_la_case(&mut self, id: &str) {
        self.ecrire_la_case(id, Texte::Description);
    }

    /// **Écrit l'un des textes d'une case** : sa description, ou un champ du storyboard — la
    /// question porte son nom. Tout est choisi : la première lettre tapée remplace l'ancien.
    pub(crate) fn ecrire_la_case(&mut self, id: &str, texte: Texte) {
        let board = self.store.project.active_board_id.clone();
        let Some(case) = self
            .store
            .cases(&board)
            .into_iter()
            .find(|c| c.id == id)
            .cloned()
        else {
            return;
        };
        let (titre, aide) = match texte {
            Texte::Description => (
                format!("Case {}", case.order + 1),
                "Ce qui s'y passe : l'action, le son, une note.".to_string(),
            ),
            Texte::Champ(rang) => {
                let Some(nom) = self
                    .store
                    .storyboard(&board)
                    .and_then(|s| s.champs.get(rang))
                else {
                    return;
                };
                (format!("Case {} — {nom}", case.order + 1), String::new())
            }
        };
        let ecrit = texte.lire(&case).to_string();
        let question = Question {
            titre,
            texte: aide,
            champ: Some(Champ {
                selection: glucose_core::text::Selection::all(&ecrit),
                texte: ecrit,
            }),
            choix: vec![
                ("Écrire".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        let suite = Suite::EcrireLaCase {
            board,
            case: case.id,
            texte,
        };
        self.demander(question, suite);
    }

    /// **Nomme un champ du storyboard** — `rang` : le renommer, ou le retirer ; `None` : en
    /// ajouter un. Retirer efface son texte de chaque case : la question le dit.
    pub(crate) fn nommer_le_champ(&mut self, rang: Option<usize>) {
        let board = self.store.project.active_board_id.clone();
        let Some(storyboard) = self.store.storyboard(&board) else {
            return;
        };
        let nom = rang
            .and_then(|r| storyboard.champs.get(r))
            .cloned()
            .unwrap_or_default();
        let (titre, texte, mut choix) = match rang {
            Some(_) => (
                format!("Le champ « {nom} »"),
                "Son nom, pour toutes les cases. Le retirer efface ce qu'il dit dans chacune."
                    .to_string(),
                vec![
                    ("Renommer".to_string(), Reponse::Oui),
                    ("Retirer le champ".to_string(), Reponse::Non),
                ],
            ),
            None => (
                "Un champ de plus".to_string(),
                "Son nom : chaque case l'aura, vide.".to_string(),
                vec![("Ajouter".to_string(), Reponse::Oui)],
            ),
        };
        choix.push(("Annuler".into(), Reponse::Annuler));
        let question = Question {
            titre,
            texte,
            champ: Some(Champ {
                selection: glucose_core::text::Selection::all(&nom),
                texte: nom,
            }),
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::NommerLeChamp { board, rang });
    }

    /// **La réponse à « nommer un champ »** : un nom vide ne nomme rien.
    pub(crate) fn repondre_au_champ(
        &mut self,
        (board, rang): (&str, Option<usize>),
        reponse: Reponse,
        nom: Option<String>,
    ) {
        let nom = nom.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
        match (reponse, rang, nom) {
            (Reponse::Non, Some(r), _) => self.store.retirer_le_champ(board, r),
            (Reponse::Oui, Some(r), Some(nom)) => self.store.nommer_le_champ(board, r, &nom),
            (Reponse::Oui, None, Some(nom)) => {
                self.store.ajouter_un_champ(board, &nom);
            }
            _ => return,
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
