//! Raccourcis clavier globaux et sélecteur d'outils (PureRef-style).

use crate::app::GlucoseApp;
use crate::ui::ActiveTool;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowLevel;

impl GlucoseApp {
    /// Traite les raccourcis clavier hors session d'édition de texte.
    pub fn handle_keyboard_shortcut(&mut self, event: &KeyEvent) {
        if event.logical_key == Key::Named(NamedKey::Space) {
            self.space_pressed = event.state == ElementState::Pressed;
            if !self.space_pressed && !self.right_or_middle_down && self.ui.active_tool != ActiveTool::Pan {
                self.is_panning = false;
            }
            self.update_cursor();
            self.mark_dirty();
            return;
        }

        if event.state != ElementState::Pressed {
            return;
        }

        let ctrl = self.modifiers.control_key();
        let active_bid = self.store.project.active_board_id.clone();

        match event.logical_key {
            Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                self.store.delete_selected(&active_bid);
                self.ui.show_toast("🗑 Supprimé");
                self.mark_dirty();
            }
            Key::Character(ref c) => match c.as_str() {
                "h" | "H" => {
                    if !ctrl {
                        self.ui.active_tool = ActiveTool::Pan;
                        self.update_cursor();
                        self.mark_dirty();
                    }
                }
                "v" | "V" => {
                    if ctrl {
                        self.paste_from_clipboard();
                    } else {
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.mark_dirty();
                    }
                }
                "z" | "Z" => {
                    if ctrl {
                        if self.modifiers.shift_key() {
                            if self.store.redo() {
                                self.ui.show_toast("🔁 Rétablir");
                            }
                        } else if self.store.undo() {
                            self.ui.show_toast("↩️ Annuler");
                        }
                        self.mark_dirty();
                    }
                }
                "y" | "Y" => {
                    if ctrl {
                        if self.store.redo() {
                            self.ui.show_toast("🔁 Rétablir");
                        }
                        self.mark_dirty();
                    }
                }
                "d" | "D" => {
                    if ctrl {
                        self.store.duplicate_selected(&active_bid);
                        self.ui.show_toast("📑 Dupliqué");
                        self.mark_dirty();
                    }
                }
                "a" | "A" => {
                    if ctrl {
                        if let Some(b) = self.store.active_board() {
                            let img_ids = b.images.iter().map(|i| i.id.clone()).collect();
                            let ann_ids = b.annotations.iter().map(|a| a.id().to_string()).collect();
                            self.store.set_selected_image_ids(img_ids);
                            self.store.set_selected_annotation_ids(ann_ids);
                        }
                        self.mark_dirty();
                    } else {
                        self.ui.active_tool = ActiveTool::Arrow;
                        self.update_cursor();
                        self.mark_dirty();
                    }
                }
                "t" | "T" => {
                    if self.modifiers.alt_key() {
                        self.always_on_top = !self.always_on_top;
                        if let Some(w) = &self.window {
                            w.set_window_level(if self.always_on_top {
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
                    } else {
                        self.ui.active_tool = ActiveTool::Text;
                        self.update_cursor();
                        self.mark_dirty();
                    }
                }
                "n" | "N" => {
                    self.ui.active_tool = ActiveTool::Sticky;
                    self.update_cursor();
                    self.mark_dirty();
                }
                "m" | "M" => {
                    self.ui.active_tool = ActiveTool::Membrane;
                    self.update_cursor();
                    self.mark_dirty();
                }
                "f" | "F" => {
                    // Fit view / recentrer la caméra PureRef
                    if let Some(b) = self.store.active_board_mut() {
                        b.viewport.x = 0.0;
                        b.viewport.y = 0.0;
                        b.viewport.scale = 1.0;
                    }
                    self.ui.show_toast("🎯 Vue recentrée");
                    self.mark_dirty();
                }
                "o" | "O" => {
                    if ctrl {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                            .pick_files()
                        {
                            self.import_image_files(&files);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
