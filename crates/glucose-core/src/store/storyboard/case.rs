//! **Ce qu'une case dit pour la lecture** (schéma 4, fiche 67) : ses textes, sa durée, son
//! plan — et ce que le storyboard règle pour toutes : la durée d'une case qui n'a pas la sienne,
//! le nom de chaque champ.
//!
//! Chaque réglage est **un geste**, un `Ctrl+Z`. Un réglage qui ne change rien n'écrit rien :
//! ici, on photographie avant et après ; c'est le journal qui jette ce qui n'a pas changé
//! (`Journal::record`), une fois pour tous — une seconde comparaison ici serait une garde
//! qu'aucune épreuve ne peut faire tomber (fiche 68).
//!
//! # Les champs sont ceux du storyboard
//!
//! Comme les légendes de Toon Boom (« les champs se règlent pour le projet : un champ ajouté
//! apparaît vide dans chaque case ; en retirer un retire son texte de toutes ») et les
//! commentaires de Krita. Une case garde le texte du champ de rang `i` à son rang `i` : retirer
//! un champ retire ce rang de chaque case **dans le même geste**, et `Ctrl+Z` rend les deux.

use super::super::journal::{Edit, Slot};
use super::super::Store;
use super::{Plan, Storyboard};
use crate::types::StoryboardPanel;

/// **Un texte d'une case** : sa description — la légende sous son cadre —, ou l'un des champs
/// du storyboard, par son rang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Texte {
    Description,
    Champ(usize),
}

impl Texte {
    /// Ce que la case dit à ce texte.
    pub fn lire(self, case: &StoryboardPanel) -> &str {
        match self {
            Self::Description => &case.description,
            Self::Champ(rang) => case.texte(rang),
        }
    }
}

impl Store {
    /// **Écrit l'un des textes de cette case.**
    pub fn ecrire_dans_la_case(&mut self, board_id: &str, id: &str, texte: Texte, ecrit: &str) {
        self.changer_les_cases(board_id, |c| {
            if c.id == id {
                match texte {
                    Texte::Description => c.description = ecrit.to_string(),
                    Texte::Champ(rang) => c.ecrire(rang, ecrit),
                }
            }
        });
    }

    /// **Règle la durée de cette case**, en millisecondes — `None` : celle du storyboard. Une
    /// durée nulle ne montrerait pas la case : elle vaut au moins une milliseconde.
    pub fn regler_la_duree(&mut self, board_id: &str, id: &str, duree_ms: Option<u32>) {
        self.changer_les_cases(board_id, |c| {
            if c.id == id {
                c.duree_ms = duree_ms.map(|ms| ms.max(1));
            }
        });
    }

    /// **Dit le plan de cette case** — `None` : aucun.
    pub fn choisir_le_plan(&mut self, board_id: &str, id: &str, plan: Option<Plan>) {
        self.changer_les_cases(board_id, |c| {
            if c.id == id {
                c.plan = plan;
            }
        });
    }

    /// **Règle la durée d'une case qui n'a pas la sienne**, pour tout ce storyboard.
    pub fn regler_la_duree_du_storyboard(&mut self, (board_id, sb): (&str, &str), duree_ms: u32) {
        let edits = self.regler((board_id, sb), |s| s.duree_ms = duree_ms.max(1));
        self.record_as_one_gesture(edits);
    }

    /// **Renomme le champ de rang `rang`** de ce storyboard.
    pub fn nommer_le_champ(&mut self, (board_id, sb): (&str, &str), rang: usize, nom: &str) {
        let edits = self.regler((board_id, sb), |s| {
            if let Some(champ) = s.champs.get_mut(rang) {
                *champ = nom.to_string();
            }
        });
        self.record_as_one_gesture(edits);
    }

    /// **Ajoute un champ au bout** de ce storyboard : chacune de ses cases l'a, vide. Rend son
    /// rang.
    pub fn ajouter_un_champ(&mut self, (board_id, sb): (&str, &str), nom: &str) -> Option<usize> {
        let rang = self.storyboard(board_id, sb)?.champs.len();
        let edits = self.regler((board_id, sb), |s| s.champs.push(nom.to_string()));
        self.record_as_one_gesture(edits);
        Some(rang)
    }

    /// **Retire le champ de rang `rang`** de ce storyboard, et son texte de chacune de ses
    /// cases — en un geste.
    pub fn retirer_le_champ(&mut self, (board_id, sb): (&str, &str), rang: usize) {
        if self
            .storyboard(board_id, sb)
            .is_none_or(|s| rang >= s.champs.len())
        {
            return;
        }
        let mut edits = self.regler((board_id, sb), |s| {
            s.champs.remove(rang);
        });
        edits.extend(self.changer(board_id, |c| {
            if c.storyboard == sb && rang < c.textes.len() {
                c.textes.remove(rang);
                while c.textes.last().is_some_and(String::is_empty) {
                    c.textes.pop();
                }
            }
        }));
        self.record_as_one_gesture(edits);
    }

    /// Change ce storyboard par `f`, et rend son avant et son après.
    fn regler(
        &mut self,
        (board_id, sb): (&str, &str),
        f: impl FnOnce(&mut Storyboard),
    ) -> Vec<Edit> {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let Some(i) = b.storyboards.iter().position(|s| s.id == sb) else {
            return Vec::new();
        };
        let avant = b.storyboards[i].clone();
        f(&mut b.storyboards[i]);
        vec![Edit::Storyboard {
            board: board_id.to_string(),
            slot: Slot::changed(i, avant, b.storyboards[i].clone()),
        }]
    }

    /// Change les cases par `f`, une à une, et l'écrit en un geste.
    fn changer_les_cases(&mut self, board_id: &str, f: impl FnMut(&mut StoryboardPanel)) {
        let edits = self.changer(board_id, f);
        self.record_as_one_gesture(edits);
    }

    /// Change les cases par `f`, et rend l'avant et l'après de chacune.
    fn changer(&mut self, board_id: &str, mut f: impl FnMut(&mut StoryboardPanel)) -> Vec<Edit> {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let mut edits = Vec::new();
        for (i, case) in b.panels.iter_mut().enumerate() {
            let avant = case.clone();
            f(case);
            edits.push(Edit::Panel {
                board: board_id.to_string(),
                slot: Slot::changed(i, avant, case.clone()),
            });
        }
        edits
    }
}

#[cfg(test)]
mod tests;
