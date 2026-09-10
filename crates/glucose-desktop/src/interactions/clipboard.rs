//! Gestion du presse-papiers (collage d'images et texte) et import de fichiers.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::error::DesktopError;
use arboard::Clipboard;
use glucose_core::types::{Annotation, BoardImage};
use std::path::{Path, PathBuf};

impl GlucoseApp {
    /// Importe une liste de chemins de fichiers image dans le board actif.
    pub fn import_image_files(&mut self, paths: &[PathBuf]) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (mut cur_wx, mut cur_wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        if self.mouse_pos.1 < self.ui.header_height() as f64 {
            cur_wx = 0.0;
            cur_wy = 0.0;
        }

        let mut count = 0;
        for path_buf in paths {
            if let Some(path_str) = path_buf.to_str() {
                let (w, h) = match image::image_dimensions(path_buf) {
                    Ok((w, h)) => (w as f64, h as f64),
                    Err(err) => {
                        let filename = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("image");
                        let desktop_err = DesktopError::ImageDimensionsFailed {
                            path: filename.to_string(),
                            reason: err.to_string(),
                        };
                        self.ui.show_toast(format!("⚠️ {}", desktop_err));
                        continue;
                    }
                };

                let max_dim = 600.0f64;
                let scale = if w > max_dim || h > max_dim {
                    (max_dim / w).min(max_dim / h)
                } else {
                    1.0
                };
                let final_w = (w * scale).max(50.0);
                let final_h = (h * scale).max(50.0);

                let id = self.store.generate_id("img");
                let mut img = BoardImage::new(id, cur_wx, cur_wy, final_w, final_h);
                img.src = Some(path_str.to_string());
                img.original_width = w;
                img.original_height = h;

                self.store.add_image(&active_bid, img);
                cur_wx += final_w + 30.0;
                count += 1;
            }
        }

        if count > 0 {
            self.ui.show_toast(format!("📥 {} image(s) ajoutée(s)", count));
            self.mark_dirty();
        }
    }

    /// Coller depuis le presse-papiers (Image ou Texte).
    pub fn paste_from_clipboard(&mut self) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        match Clipboard::new() {
            Ok(mut clipboard) => {
                // 1. Tenter de coller une image bitmap (Pinterest, navigateur, capture d'écran)
                if let Ok(img_data) = clipboard.get_image() {
                    let w = img_data.width as usize;
                    let h = img_data.height as usize;
                    let temp_dir = std::env::temp_dir().join("glucose_pasted");
                    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                        let err = DesktopError::Io(e);
                        self.ui.show_toast(format!("⚠️ {}", err));
                        return;
                    }
                    let nanos = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();
                    let filename = format!("paste_{}.png", nanos);
                    let file_path = temp_dir.join(&filename);

                    match image::save_buffer(
                        &file_path,
                        &img_data.bytes,
                        w as u32,
                        h as u32,
                        image::ExtendedColorType::Rgba8,
                    ) {
                        Ok(_) => {
                            let id = self.store.generate_id("img-paste");
                            let mut img = BoardImage::new(
                                id,
                                wx,
                                wy,
                                (w as f64).min(600.0),
                                (h as f64) * ((w as f64).min(600.0) / (w as f64).max(1.0)),
                            );
                            img.src = Some(file_path.to_string_lossy().to_string());
                            img.original_width = w as f64;
                            img.original_height = h as f64;

                            self.store.add_image(&active_bid, img);
                            self.ui.show_toast("📥 Image collée");
                            self.mark_dirty();
                            return;
                        }
                        Err(e) => {
                            let err = DesktopError::ImageDecodeFailed {
                                path: filename,
                                reason: e.to_string(),
                            };
                            self.ui.show_toast(format!("⚠️ {}", err));
                            return;
                        }
                    }
                }

                // 2. Tenter de coller du texte ou un chemin de fichier
                if let Ok(text) = clipboard.get_text() {
                    let trimmed = text.trim();
                    let path = Path::new(trimmed);
                    if path.exists() && path.is_file() {
                        self.import_image_files(&[path.to_path_buf()]);
                        return;
                    }

                    // Coller en tant que carte texte
                    let aid = self.store.generate_id("text");
                    let ann = Annotation::Text {
                        id: aid,
                        x: wx,
                        y: wy,
                        width: Some(220.0),
                        height: Some(44.0),
                        text: trimmed.to_string(),
                        font_size: Some(13.0),
                        color: None,
                        cursor_pos: None,
                        source_file: None,
                        membrane_id: None,
                        domains: Vec::new(),
                        mirror_of: None,
                        temporal_anchor: None,
                    };
                    self.store.add_annotation(&active_bid, ann);
                    self.ui.show_toast("📝 Texte collé");
                    self.mark_dirty();
                }
            }
            Err(e) => {
                let err = DesktopError::ClipboardError(e.to_string());
                self.ui.show_toast(format!("⚠️ {}", err));
            }
        }
    }
}
