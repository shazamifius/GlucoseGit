//! Session d'édition de texte in-place avec support UTF-8 (PureRef-style).

use crate::app::GlucoseApp;
use crate::renderer::TextEditSession;
use glucose_core::types::Annotation;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

/// La frappe est-elle une commande de fichier (`Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`) ?
///
/// Une session d'édition avale TOUTES les touches — c'est ce qui permet de taper `s` dans une
/// carte sans déclencher un raccourci. Mais `Ctrl+S` au milieu d'une phrase veut dire
/// « enregistre », pas « ignore-moi » : sans cette exception, enregistrer serait impossible
/// tant qu'un curseur clignote quelque part.
fn is_file_command(modifiers: &ModifiersState, key: &Key) -> bool {
    if !modifiers.control_key() {
        return false;
    }
    matches!(key, Key::Character(c) if matches!(c.as_str(), "s" | "S" | "o" | "O"))
}

impl GlucoseApp {
    /// Initialise une session d'édition in-place pour une annotation.
    pub fn start_text_edit(&mut self, ann_id: String, initial_text: String) {
        let cur_idx = initial_text.len();
        self.editing_session = Some(TextEditSession {
            ann_id,
            buffer: initial_text,
            cursor_idx: cur_idx,
            blink_timer: std::time::Instant::now(),
        });
        self.mark_dirty();
    }

    /// Valide et persiste le texte édité dans le store.
    ///
    /// La saisie entière est **une** entrée d'undo (fiche 09 § 2.2) : `begin_live_edit`
    /// ouvre le geste, le texte puis la hauteur de la carte (TEXT-FIT-1) s'y écrivent, et
    /// `end_live_edit` le referme. Chaque écriture passe par le store, jamais par le board
    /// directement : tant que l'ouverture prenait un cliché du document, une écriture
    /// directe était couverte ; avec le journal d'éditions, elle est invisible — et c'est
    /// ainsi que la saisie de texte a cessé d'être annulable sans qu'aucun test ne le voie.
    pub fn commit_editing(&mut self) {
        let Some(session) = self.editing_session.take() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        let is_empty = session.buffer.trim().is_empty();
        let is_text_card = self
            .store
            .active_board()
            .and_then(|b| b.annotations.iter().find(|a| a.id() == session.ann_id))
            .is_some_and(|a| matches!(a, Annotation::Text { .. }));

        self.store.begin_live_edit();
        if is_empty && is_text_card {
            // Une carte de texte vidée disparaît — une suppression comme une autre.
            self.store.remove_annotations(&board, &[&session.ann_id]);
        } else {
            self.store.update_annotation(&board, &session.ann_id, |ann| match ann {
                Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => {
                    *text = session.buffer.clone();
                }
                Annotation::Membrane { text, .. } => {
                    *text = if is_empty { None } else { Some(session.buffer.clone()) };
                }
                _ => {}
            });
            self.fit_text_card_height(&session.ann_id);
        }
        self.store.end_live_edit();
    }

    /// Traite les touches clavier lors d'une session d'édition active.
    pub fn handle_text_key(&mut self, event: &KeyEvent) -> bool {
        // Enregistrer ou ouvrir pendant une saisie : on valide d'abord le texte en cours,
        // puis on laisse la touche descendre aux raccourcis globaux.
        if event.state == ElementState::Pressed
            && self.editing_session.is_some()
            && is_file_command(&self.modifiers, &event.logical_key)
        {
            self.commit_editing();
            self.mark_dirty();
            return false;
        }

        // Le garde de l'arme `Key::Character` est évalué alors que `session`
        // emprunte déjà `self` : la question « la frappe produit-elle du texte ? »
        // se résout donc AVANT l'emprunt, pas dans le garde.
        let produces_text = !self.modifiers.control_key() && !self.modifiers.alt_key();
        let Some(session) = &mut self.editing_session else {
            return false;
        };

        if event.state != ElementState::Pressed {
            return true;
        }

        match event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.commit_editing();
                self.mark_dirty();
                return true;
            }
            Key::Named(NamedKey::Enter) => {
                if !self.modifiers.shift_key() {
                    self.commit_editing();
                    self.mark_dirty();
                    return true;
                } else {
                    session.buffer.insert(session.cursor_idx, '\n');
                    session.cursor_idx += 1;
                    session.blink_timer = std::time::Instant::now();
                    self.mark_dirty();
                    return true;
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if session.cursor_idx > 0 {
                    let mut prev = session.cursor_idx - 1;
                    while prev > 0 && !session.buffer.is_char_boundary(prev) {
                        prev -= 1;
                    }
                    session.buffer.drain(prev..session.cursor_idx);
                    session.cursor_idx = prev;
                    session.blink_timer = std::time::Instant::now();
                    self.mark_dirty();
                    return true;
                }
            }
            Key::Named(NamedKey::Delete) => {
                if session.cursor_idx < session.buffer.len() {
                    let mut next = session.cursor_idx + 1;
                    while next < session.buffer.len() && !session.buffer.is_char_boundary(next) {
                        next += 1;
                    }
                    session.buffer.drain(session.cursor_idx..next);
                    session.blink_timer = std::time::Instant::now();
                    self.mark_dirty();
                    return true;
                }
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if session.cursor_idx > 0 {
                    let mut prev = session.cursor_idx - 1;
                    while prev > 0 && !session.buffer.is_char_boundary(prev) {
                        prev -= 1;
                    }
                    session.cursor_idx = prev;
                    session.blink_timer = std::time::Instant::now();
                    self.mark_dirty();
                    return true;
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                if session.cursor_idx < session.buffer.len() {
                    let mut next = session.cursor_idx + 1;
                    while next < session.buffer.len() && !session.buffer.is_char_boundary(next) {
                        next += 1;
                    }
                    session.cursor_idx = next;
                    session.blink_timer = std::time::Instant::now();
                    self.mark_dirty();
                    return true;
                }
            }
            Key::Named(NamedKey::Home) => {
                session.cursor_idx = 0;
                session.blink_timer = std::time::Instant::now();
                self.mark_dirty();
                return true;
            }
            Key::Named(NamedKey::End) => {
                session.cursor_idx = session.buffer.len();
                session.blink_timer = std::time::Instant::now();
                self.mark_dirty();
                return true;
            }
            Key::Character(ref c) if produces_text => {
                session.buffer.insert_str(session.cursor_idx, c.as_str());
                session.cursor_idx += c.len();
                session.blink_timer = std::time::Instant::now();
                self.mark_dirty();
                return true;
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::SmolStr;

    fn character(c: &str) -> Key {
        Key::Character(SmolStr::new(c))
    }

    #[test]
    fn test_ctrl_s_and_ctrl_o_escape_a_text_edit_session() {
        let ctrl = ModifiersState::CONTROL;
        assert!(is_file_command(&ctrl, &character("s")));
        assert!(is_file_command(&ctrl, &character("S")));
        assert!(is_file_command(&ctrl, &character("o")));
    }

    #[test]
    fn test_plain_letters_still_belong_to_the_text_being_typed() {
        let none = ModifiersState::empty();
        assert!(!is_file_command(&none, &character("s")));
        assert!(!is_file_command(&none, &character("o")));
        // Ctrl+A reste une sélection de texte, pas une commande de fichier.
        assert!(!is_file_command(&ModifiersState::CONTROL, &character("a")));
        assert!(!is_file_command(&ModifiersState::CONTROL, &Key::Named(NamedKey::Enter)));
    }
}
