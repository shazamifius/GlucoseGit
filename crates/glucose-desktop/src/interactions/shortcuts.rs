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

/// Ce que `Maj` fait à un déplacement au clavier : dix pas d'un coup.
///
/// La base décimale, pas une longueur choisie — l'unité fine est celle du monde, et ceci en
/// est la dizaine.
const NUDGE_DECADE: f64 = 10.0;
use glucose_core::store::StackMove;
use glucose_core::types::Viewport;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowLevel;

impl GlucoseApp {
    /// Une touche arrive de la fenêtre. Trois preneurs, dans l'ordre : la saisie d'un nom de
    /// domaine, l'édition d'une annotation, puis les raccourcis globaux. Chacun rend `false`
    /// quand la touche ne le concerne pas ; aucun ne contient de logique (§ 1.7).
    pub fn handle_key(&mut self, event: &KeyEvent) {
        if self.handle_domain_rename_key(event) || self.handle_text_key(event) {
            return;
        }
        self.handle_keyboard_shortcut(event);
    }

    /// Traite les raccourcis clavier hors session d'édition de texte.
    pub fn handle_keyboard_shortcut(&mut self, event: &KeyEvent) {
        self.handle_shortcut_input(&event.logical_key, event.state);
    }

    /// Le corps de [`GlucoseApp::handle_keyboard_shortcut`], sans le `KeyEvent` de winit,
    /// qui ne se construit pas hors de la boucle d'événements : tout ce qui décide se teste
    /// ici, sans fenêtre (§ 7.1).
    pub fn handle_shortcut_input(&mut self, logical_key: &Key, state: ElementState) {
        if *logical_key == Key::Named(NamedKey::Space) {
            self.handle_space_pan(state == ElementState::Pressed);
            return;
        }

        if state != ElementState::Pressed {
            return;
        }

        match logical_key {
            Key::Named(NamedKey::Escape) => self.escape_gesture(),
            Key::Named(NamedKey::ArrowLeft) => self.nudge(-1.0, 0.0),
            Key::Named(NamedKey::ArrowRight) => self.nudge(1.0, 0.0),
            Key::Named(NamedKey::ArrowUp) => self.nudge(0.0, -1.0),
            Key::Named(NamedKey::ArrowDown) => self.nudge(0.0, 1.0),
            Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                self.delete_selection();
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

    /// `Échap` annule le geste en cours. Aujourd'hui, le seul geste annulable hors édition est
    /// un redimensionnement, qui reprend sa taille de départ ; la fiche 03 § 19.6 en attend
    /// davantage (tout geste courant), et c'est ici que les autres viendront.
    fn escape_gesture(&mut self) {
        // Un menu ouvert est ce qu'on annule en premier : c'est le geste le plus récent, et
        // celui qui attend une décision.
        if self.ui.context_menu_at.take().is_some() {
            self.mark_dirty();
            return;
        }
        // Sans toast : la boîte reprend sa taille de départ sous les yeux de celui qui
        // vient d'appuyer. Un message qui décrit ce que l'œil enregistre est du bruit — la
        // même règle que pour l'ordre d'empilement.
        self.cancel_resize();
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
        match key {
            "v" | "V" => self.paste_from_clipboard(),
            "z" | "Z" => {
                if self.modifiers.shift_key() {
                    let done = self.store.redo();
                    self.toast_if(done, "Rétablir");
                } else {
                    let done = self.store.undo();
                    self.toast_if(done, "Annuler");
                }
            }
            "y" | "Y" => {
                let done = self.store.redo();
                self.toast_if(done, "Rétablir");
            }
            "d" | "D" => self.duplicate_selection(),
            "a" | "A" => self.select_all(),
            "]" => self.restack(StackMove::Front),
            "[" => self.restack(StackMove::Back),
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
            "l" | "L" => {
                self.toggle_lock();
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

    /// Déplace la sélection d'un cran au clavier, pour l'ajustement que la souris ne sait pas
    /// faire.
    ///
    /// Le pas est **l'unité du monde**, et `Maj` le multiplie par dix : deux gestes, aucune
    /// longueur à choisir. Une image verrouillée ne bouge pas — c'est `move_selected` qui le
    /// tient, et le clavier n'a pas à le savoir.
    fn nudge(&mut self, dx: f64, dy: f64) {
        let pas = if self.modifiers.shift_key() {
            NUDGE_DECADE
        } else {
            1.0
        };
        let board = self.store.project.active_board_id.clone();
        self.store.move_selected(&board, dx * pas, dy * pas);
        self.mark_dirty();
    }

    /// Porte la sélection au premier ou au dernier plan de sa couche.
    ///
    /// Sans toast : le nœud passe devant, ou derrière, et cela **se voit**. Un message qui
    /// décrit ce que l'œil vient d'enregistrer est du bruit, pas une confirmation.
    fn restack(&mut self, mv: StackMove) {
        let board = self.store.project.active_board_id.clone();
        if self.store.move_selection_in_stack(&board, mv) > 0 {
            self.mark_dirty();
        }
    }

    /// Duplique la sélection entière.
    pub(crate) fn duplicate_selection(&mut self) {
        let board = self.store.project.active_board_id.clone();
        self.store.duplicate_selected(&board);
        self.ui.show_toast("Dupliqué");
        self.mark_dirty();
    }

    /// Supprime la sélection entière.
    ///
    /// Un geste, un endroit : la touche `Suppr` et le bouton de la barre d'action appellent
    /// la même fonction. Deux chemins vers un même geste finissent toujours par diverger —
    /// l'un oublie le toast, l'autre le `mark_dirty`.
    pub(crate) fn delete_selection(&mut self) {
        let board = self.store.project.active_board_id.clone();
        self.store.delete_selected(&board);
        self.ui.show_toast("Supprimé");
        self.mark_dirty();
    }

    /// Bascule le verrou des images sélectionnées (fiche 08 § 1.3).
    pub(crate) fn toggle_lock(&mut self) {
        let board = self.store.project.active_board_id.clone();
        let Some(locked) = self.store.toggle_lock_selection(&board) else {
            return;
        };
        // La fiche 08 § 1.3 cite le libellé de la référence, cadenas compris. Il tourne dans
        // un navigateur, qui a une police d'emoji ; le natif n'en embarque pas, et le test
        // FONT-1 refuse tout caractère qu'aucun visage ne sait dessiner. Le mot suffit — et
        // c'est le cadre rouge qui dit la chose à l'œil, pas le toast.
        self.ui.show_toast(if locked {
            "Images verrouillées"
        } else {
            "Images déverrouillées"
        });
        self.mark_dirty();
    }

    pub(crate) fn select_all(&mut self) {
        let Some(board) = self.store.active_board() else {
            return;
        };
        let img_ids = board.images.iter().map(|i| i.id.clone()).collect();
        let ann_ids = board
            .annotations
            .iter()
            .map(|a| a.id().to_string())
            .collect();
        self.store.set_selected_image_ids(img_ids);
        self.store.set_selected_annotation_ids(ann_ids);
    }

    /// Recentre la caméra PureRef sur l'origine, à l'échelle 1.
    fn reset_view(&mut self) {
        let board = self.store.project.active_board_id.clone();
        self.store.set_viewport(&board, Viewport::default());
        self.ui.show_toast("Vue recentrée");
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
            "Toujours au premier plan"
        } else {
            "Fenêtre normale"
        });
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
