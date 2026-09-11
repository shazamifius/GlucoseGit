//! Raccourcis clavier globaux et sélecteur d'outils (PureRef-style).
//!
//! # Ordre de résolution des touches de caractère
//!
//! Trois familles se partagent les mêmes lettres selon les modificateurs (`Ctrl+V` colle,
//! `V` choisit l'outil de sélection). Elles sont donc essayées dans un ordre fixe, chacune
//! rendant `true` si elle a consommé la touche :
//!
//! 1. **Fichier** (`persist`) — `Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`, `Ctrl+I`
//! 2. **Édition** — `Ctrl+V`, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+D`, `Ctrl+A`
//! 3. **Outils** — les lettres nues
//!
//! Le découpage tient la règle § 1.1 : chaque famille est une fonction courte, et ajouter un
//! raccourci n'allonge plus un `match` unique de cent lignes.

use crate::app::GlucoseApp;
use crate::ui::ActiveTool;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowLevel;

impl GlucoseApp {
    /// Traite les raccourcis clavier hors session d'édition de texte.
    pub fn handle_keyboard_shortcut(&mut self, event: &KeyEvent) {
        if event.logical_key == Key::Named(NamedKey::Space) {
            self.handle_space_pan(event.state == ElementState::Pressed);
            return;
        }

        if event.state != ElementState::Pressed {
            return;
        }

        match event.logical_key {
            Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                let active_bid = self.store.project.active_board_id.clone();
                self.store.delete_selected(&active_bid);
                self.ui.show_toast("🗑 Supprimé");
                self.mark_dirty();
            }
            Key::Character(ref c) => {
                let key = c.as_str();
                if self.handle_file_shortcut(key) {
                    return;
                }
                if self.handle_edit_shortcut(key) {
                    return;
                }
                self.handle_tool_shortcut(key);
            }
            _ => {}
        }
    }

    /// Barre d'espace maintenue : pan temporaire, quel que soit l'outil actif.
    fn handle_space_pan(&mut self, pressed: bool) {
        self.space_pressed = pressed;
        if !pressed && !self.right_or_middle_down && self.ui.active_tool != ActiveTool::Pan {
            self.is_panning = false;
        }
        self.update_cursor();
        self.mark_dirty();
    }

    /// Raccourcis d'édition du document. Rend `true` si la touche a été consommée.
    fn handle_edit_shortcut(&mut self, key: &str) -> bool {
        if !self.modifiers.control_key() {
            return false;
        }
        let active_bid = self.store.project.active_board_id.clone();
        match key {
            "v" | "V" => self.paste_from_clipboard(),
            "z" | "Z" => {
                if self.modifiers.shift_key() {
                    let done = self.store.redo();
                    self.toast_if(done, "🔁 Rétablir");
                } else {
                    let done = self.store.undo();
                    self.toast_if(done, "↩️ Annuler");
                }
            }
            "y" | "Y" => {
                let done = self.store.redo();
                self.toast_if(done, "🔁 Rétablir");
            }
            "d" | "D" => {
                self.store.duplicate_selected(&active_bid);
                self.ui.show_toast("📑 Dupliqué");
            }
            "a" | "A" => self.select_all(),
            _ => return false,
        }
        self.mark_dirty();
        true
    }

    /// Choix d'outil et recentrage : les lettres nues, plus `Alt+T`.
    fn handle_tool_shortcut(&mut self, key: &str) {
        if self.modifiers.control_key() {
            return;
        }
        let tool = match key {
            "h" | "H" => ActiveTool::Pan,
            "v" | "V" => ActiveTool::Select,
            "a" | "A" => ActiveTool::Arrow,
            "n" | "N" => ActiveTool::Sticky,
            "m" | "M" => ActiveTool::Membrane,
            "t" | "T" if self.modifiers.alt_key() => {
                self.toggle_always_on_top();
                return;
            }
            "t" | "T" => ActiveTool::Text,
            "f" | "F" => {
                self.reset_view();
                return;
            }
            _ => return,
        };
        self.ui.active_tool = tool;
        self.update_cursor();
        self.mark_dirty();
    }

    fn toast_if(&mut self, happened: bool, message: &str) {
        if happened {
            self.ui.show_toast(message);
        }
    }

    fn select_all(&mut self) {
        let Some(board) = self.store.active_board() else {
            return;
        };
        let img_ids = board.images.iter().map(|i| i.id.clone()).collect();
        let ann_ids = board.annotations.iter().map(|a| a.id().to_string()).collect();
        self.store.set_selected_image_ids(img_ids);
        self.store.set_selected_annotation_ids(ann_ids);
    }

    /// Recentre la caméra PureRef sur l'origine, à l'échelle 1.
    fn reset_view(&mut self) {
        if let Some(board) = self.store.active_board_mut() {
            board.viewport.x = 0.0;
            board.viewport.y = 0.0;
            board.viewport.scale = 1.0;
        }
        self.ui.show_toast("🎯 Vue recentrée");
        self.mark_dirty();
    }

    fn toggle_always_on_top(&mut self) {
        self.always_on_top = !self.always_on_top;
        if let Some(window) = &self.window {
            window.set_window_level(if self.always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });
        }
        self.ui.show_toast(if self.always_on_top {
            "📌 Toujours au premier plan"
        } else {
            "Fenêtre normale"
        });
        self.mark_dirty();
    }
}
