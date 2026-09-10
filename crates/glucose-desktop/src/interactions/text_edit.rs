//! Session d'édition de texte in-place avec support UTF-8 (PureRef-style).

use crate::app::GlucoseApp;
use crate::renderer::TextEditSession;
use glucose_core::types::Annotation;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

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
        self.redraw();
    }

    /// Valide et persiste le texte édité dans le store.
    pub fn commit_editing(&mut self) {
        if let Some(session) = self.editing_session.take() {
            let is_empty = session.buffer.trim().is_empty();
            let mut should_delete = false;

            if let Some(b) = self.store.active_board_mut() {
                if let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == session.ann_id) {
                    match ann {
                        Annotation::Text { text, .. } => {
                            if is_empty {
                                should_delete = true;
                            } else {
                                *text = session.buffer.clone();
                            }
                        }
                        Annotation::Sticky { text, .. } => {
                            *text = session.buffer.clone();
                        }
                        Annotation::Membrane { text, .. } => {
                            *text = if is_empty { None } else { Some(session.buffer.clone()) };
                        }
                        _ => {}
                    }
                }
            }

            if should_delete {
                if let Some(b) = self.store.active_board_mut() {
                    b.annotations.retain(|a| a.id() != session.ann_id);
                }
            }
            self.store.push_undo();
        }
    }

    /// Traite les touches clavier lors d'une session d'édition active.
    pub fn handle_text_key(&mut self, event: &KeyEvent) -> bool {
        let Some(session) = &mut self.editing_session else {
            return false;
        };

        if event.state != winit::event::ElementState::Pressed {
            return true;
        }

        match event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.commit_editing();
                self.redraw();
                return true;
            }
            Key::Named(NamedKey::Enter) => {
                if !self.modifiers.shift_key() {
                    self.commit_editing();
                    self.redraw();
                    return true;
                } else {
                    session.buffer.insert(session.cursor_idx, '\n');
                    session.cursor_idx += 1;
                    session.blink_timer = std::time::Instant::now();
                    self.redraw();
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
                    self.redraw();
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
                    self.redraw();
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
                    self.redraw();
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
                    self.redraw();
                    return true;
                }
            }
            Key::Named(NamedKey::Home) => {
                session.cursor_idx = 0;
                session.blink_timer = std::time::Instant::now();
                self.redraw();
                return true;
            }
            Key::Named(NamedKey::End) => {
                session.cursor_idx = session.buffer.len();
                session.blink_timer = std::time::Instant::now();
                self.redraw();
                return true;
            }
            Key::Character(ref c) => {
                if !self.modifiers.control_key() && !self.modifiers.alt_key() {
                    session.buffer.insert_str(session.cursor_idx, c.as_str());
                    session.cursor_idx += c.len();
                    session.blink_timer = std::time::Instant::now();
                    self.redraw();
                    return true;
                }
            }
            _ => {}
        }
        true
    }
}
