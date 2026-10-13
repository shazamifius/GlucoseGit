//! **Écrire dans le champ d'une question, au clavier** (CHAMP-QUESTION-1, fiche 65).
//!
//! Le champ n'était rempli que par le clavier du téléphone (le miroir, CLAVIER-1) : au PC, la
//! question jetait toute touche autre que Tab, les flèches, Entrée et Échap — *« le bouton
//! Décrire ne fonctionne absolument pas, la zone de texte n'écrit rien »* (son retour du
//! 13/10). Et l'épreuve qui la gardait écrivait dans le champ à la main, sans passer par une
//! seule touche.
//!
//! Une frappe s'y traduit désormais par **la même loi que dans un nœud**
//! ([`Command::of`](crate::interactions::text_edit::keys::Command::of)) — une seule traduction
//! du clavier, accents des touches mortes compris (le texte composé, pas la touche). Entrée,
//! Échap, Tab et les flèches verticales restent à la question : répondre, renoncer, passer
//! d'une réponse à l'autre.

use crate::app::GlucoseApp;
use crate::interactions::text_edit::keys::Command;
use crate::ui::question::Champ;
use glucose_core::text::selection::{delete, move_offset, replace, Motion, Selection};
use winit::keyboard::{Key, NamedKey};

impl GlucoseApp {
    /// **Une frappe pour le champ de la question posée** ; `texte`, ce que la composition du
    /// clavier a produit. Rend vrai si le champ l'a prise.
    pub(crate) fn frapper_dans_la_question(&mut self, key: &Key, texte: Option<&str>) -> bool {
        let a_la_question = matches!(
            key,
            Key::Named(
                NamedKey::Enter
                    | NamedKey::Escape
                    | NamedKey::BrowserBack
                    | NamedKey::Tab
                    | NamedKey::ArrowUp
                    | NamedKey::ArrowDown
            )
        );
        let a_un_champ = self
            .ui
            .question
            .as_ref()
            .is_some_and(|(q, _)| q.champ.is_some());
        if a_la_question || !a_un_champ {
            return false;
        }
        let Some((commande, etendre)) = Command::of(key, &self.modifiers, texte) else {
            return false;
        };
        match commande {
            Command::Copy | Command::Cut => self.copier_le_champ(commande == Command::Cut),
            Command::Paste => {
                if let Some(colle) = self.clipboard_text() {
                    self.modifier_le_champ(|c| {
                        c.selection = replace(&mut c.texte, c.selection, &colle)
                    });
                }
            }
            autre => self.modifier_le_champ(|c| appliquer(c, autre, etendre)),
        }
        self.mark_dirty();
        true
    }

    /// Ce que le champ de la question posée devient.
    fn modifier_le_champ(&mut self, f: impl FnOnce(&mut Champ)) {
        if let Some(champ) = self
            .ui
            .question
            .as_mut()
            .and_then(|(q, _)| q.champ.as_mut())
        {
            f(champ);
        }
    }

    /// Copie ce qui est choisi dans le champ ; `couper` l'efface, une fois la copie faite.
    fn copier_le_champ(&mut self, couper: bool) {
        let Some(champ) = self
            .ui
            .question
            .as_ref()
            .and_then(|(q, _)| q.champ.as_ref())
        else {
            return;
        };
        let pris = champ.selection.slice(&champ.texte).to_string();
        if pris.is_empty() || !self.ecrire_presse_papiers(pris) {
            return;
        }
        if couper {
            self.modifier_le_champ(|c| c.selection = replace(&mut c.texte, c.selection, ""));
        }
    }
}

/// **Une commande d'écriture sur le champ** — d'une seule ligne : monter, descendre, le début
/// et la fin d'une ligne vont à ses bords.
fn appliquer(champ: &mut Champ, commande: Command, etendre: bool) {
    let sel = champ.selection;
    let tete = match commande {
        Command::Insert(t) => {
            champ.selection = replace(&mut champ.texte, sel, &t);
            return;
        }
        Command::Delete(motion, dir) => {
            champ.selection = delete(&mut champ.texte, sel, motion, dir);
            return;
        }
        Command::SelectAll => {
            champ.selection = Selection::all(&champ.texte);
            return;
        }
        // Sans `Maj`, une flèche sur une sélection retombe de son côté (SEL-1).
        Command::Move(Motion::Char, dir) if !etendre && !sel.is_empty() => sel.collapsed(dir).head,
        Command::Move(motion, dir) => move_offset(&champ.texte, sel.head, motion, dir),
        Command::MoveLine(dir) | Command::MoveVisualEdge(dir) => {
            move_offset(&champ.texte, sel.head, Motion::DocEdge, dir)
        }
        Command::Copy | Command::Cut | Command::Paste => return,
        Command::Commit | Command::Undo | Command::Redo => return,
    };
    champ.selection.head = tete;
    if !etendre {
        champ.selection.anchor = tete;
    }
}
