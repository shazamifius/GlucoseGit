//! Gestion du presse-papiers (collage d'images et texte) et import de fichiers.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::error::DesktopError;
use arboard::Clipboard;
use glucose_core::types::BoardImage;
use std::path::{Path, PathBuf};

/// Extensions proposees par le dialogue d'import d'images.
const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];

impl GlucoseApp {
    /// Ouvre le dialogue natif d'import d'images (Ctrl+I, bouton « Ajouter » de la barre).
    pub fn pick_and_import_images(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("Images", &IMAGE_EXTENSIONS)
            .pick_files()
        {
            self.import_image_files(&files);
        }
    }

    /// Importe une liste de chemins de fichiers image dans le board actif.
    pub fn import_image_files(&mut self, paths: &[PathBuf]) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self
            .store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or_default();
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
                        let filename = path_buf
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("image");
                        let desktop_err = DesktopError::ImageDimensionsFailed {
                            path: filename.to_string(),
                            reason: err.to_string(),
                        };
                        self.ui.show_toast(desktop_err.to_string());
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
            self.ui.show_toast(format!("{} image(s) ajoutée(s)", count));
            self.mark_dirty();
        }
    }

    /// Coller depuis le presse-papiers (Image ou Texte).
    pub fn paste_from_clipboard(&mut self) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self
            .store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or_default();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        match Clipboard::new() {
            Ok(mut clipboard) => {
                // 1. Une image bitmap — navigateur, capture d'écran.
                if let Ok(img_data) = clipboard.get_image() {
                    self.coller_image(&active_bid, &img_data, (wx, wy));
                    return;
                }

                // 2. Tenter de coller du texte ou un chemin de fichier
                if let Ok(text) = clipboard.get_text() {
                    let trimmed = text.trim();
                    let path = Path::new(trimmed);
                    if path.exists() && path.is_file() {
                        self.import_image_files(&[path.to_path_buf()]);
                        return;
                    }

                    // Coller en tant que carte texte, **par la fabrique**.
                    //
                    // Elle construisait la carte à la main, avec 44 unités de haut écrites en
                    // dur. Un texte collé de vingt lignes se dessinait donc sur huit cents
                    // unités et ne se cliquait que sur quarante-quatre : la mise en page
                    // prend le maximum de la hauteur déclarée et de celle du texte, l'arbitre
                    // de clic interroge la boîte du document, et les deux divergeaient d'un
                    // facteur vingt. C'est ce que « les textes trop longs deviennent
                    // impossibles à sélectionner » décrivait.
                    //
                    // La fabrique mesure ; personne d'autre n'a à savoir comment.
                    let aid = self.store.generate_id("text");
                    let ann = crate::interactions::tools::text_card(
                        &self.renderer.typography,
                        &self.renderer.math,
                        aid,
                        wx,
                        wy,
                        trimmed,
                    );
                    self.store.add_annotation(&active_bid, ann);
                    self.ui.show_toast("Texte collé");
                    self.mark_dirty();
                }
            }
            Err(e) => {
                let err = DesktopError::ClipboardError(e.to_string());
                self.ui.show_toast(err.to_string());
            }
        }
    }
}

impl GlucoseApp {
    /// Pose sur le tableau une image venue du presse-papiers.
    ///
    /// Le tampon n'est pas un fichier : il faut l'écrire pour que le cache d'images sache le
    /// relire, et c'est le répertoire temporaire du système qui l'accueille.
    fn coller_image(
        &mut self,
        board: &str,
        img_data: &arboard::ImageData<'_>,
        (wx, wy): (f64, f64),
    ) {
        let (w, h) = (img_data.width, img_data.height);
        let temp_dir = std::env::temp_dir().join("glucose_pasted");
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            self.ui.show_toast(DesktopError::Io(e).to_string());
            return;
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let filename = format!("paste_{nanos}.png");
        let file_path = temp_dir.join(&filename);
        if let Err(e) = image::save_buffer(
            &file_path,
            &img_data.bytes,
            w as u32,
            h as u32,
            image::ExtendedColorType::Rgba8,
        ) {
            let err = DesktopError::ImageDecodeFailed {
                path: filename,
                reason: e.to_string(),
            };
            self.ui.show_toast(err.to_string());
            return;
        }
        // Une image collée naît bornée en largeur, son rapport préservé : un rendu de
        // navigateur peut faire plusieurs milliers de pixels, et naître plus large que le
        // tableau n'aide personne.
        let large = (w as f64).min(600.0);
        let haute = (h as f64) * (large / (w as f64).max(1.0));
        let id = self.store.generate_id("img-paste");
        let mut img = BoardImage::new(id, wx, wy, large, haute);
        img.src = Some(file_path.to_string_lossy().to_string());
        img.original_width = w as f64;
        img.original_height = h as f64;
        self.store.add_image(board, img);
        self.ui.show_toast("Image collée");
        self.mark_dirty();
    }
}

// ── Le presse-papiers **dans** un texte ─────────────────────────────────────
//
// `paste_from_clipboard` colle dans le *canevas* : une image devient une image, un texte
// devient une carte. Pendant une saisie, `Ctrl+V` veut dire tout autre chose — coller **dans**
// le texte, à la place de la sélection — et c'est ce que ces deux méthodes servent.

impl GlucoseApp {
    /// Le texte du presse-papiers, s'il y en a un.
    ///
    /// Les fins de ligne Windows sont ramenées à `\n` : le modèle ne connaît qu'un saut de
    /// ligne, et laisser passer un `\r` ferait apparaître un caractère de contrôle au milieu
    /// d'une carte — invisible à l'écran, bien présent dans le document et dans l'export.
    pub(crate) fn clipboard_text(&mut self) -> Option<String> {
        match Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(texte) => Some(texte.replace("\r\n", "\n").replace('\r', "\n")),
            Err(err) => {
                self.echec_presse_papiers(err);
                None
            }
        }
    }

    /// Copie le texte sélectionné ; `couper` l'efface ensuite.
    ///
    /// Une sélection vide ne copie rien et ne vide pas le presse-papiers : `Ctrl+C` sans
    /// sélection est un geste sans effet, pas un geste destructeur.
    pub(crate) fn copy_selected_text(&mut self, couper: bool) {
        let Some(session) = self.editing_session.as_ref() else {
            return;
        };
        let texte = session.selection.slice(&session.buffer).to_string();
        if texte.is_empty() {
            return;
        }
        if !self.ecrire_presse_papiers(texte) {
            return;
        }
        if couper {
            if let Some(session) = self.editing_session.as_mut() {
                session.selection = glucose_core::text::selection::replace(
                    &mut session.buffer,
                    session.selection,
                    "",
                );
            }
        }
    }
}

// ── Copier ce qui est **sélectionné** ───────────────────────────────────────
//
// `Ctrl+C` n'existait que pendant une saisie. Hors saisie, la table des raccourcis tenait
// `v`, `z`, `y`, `d`, `a`, `]` et `[` — mais ni `c` ni `x`. Sélectionner une carte et la
// copier ne faisait donc rien du tout, et rien ne le disait : pas de toast, pas d'erreur,
// pas de test. Le geste le plus banal d'un ordinateur tombait dans le vide.

impl GlucoseApp {
    /// `Ctrl+C` et `Ctrl+X` hors saisie : la sélection part vers le presse-papiers du système.
    pub(crate) fn copy_selection(&mut self, couper: bool) {
        let compte = self.store.selected_annotation_ids.len() + self.store.selected_image_ids.len();
        let Some(texte) = self.store.selection_as_text() else {
            return;
        };
        if !self.ecrire_presse_papiers(texte) {
            return;
        }
        let verbe = if couper { "coupé" } else { "copié" };
        let pluriel = if compte > 1 { "s" } else { "" };
        self.ui
            .show_toast(format!("{compte} élément{pluriel} {verbe}{pluriel}"));
        if couper {
            self.delete_selection();
        }
        self.mark_dirty();
    }
}

impl GlucoseApp {
    /// Le seul endroit qui dise qu'un échange avec le presse-papiers a échoué.
    ///
    /// Lire et écrire disaient la même phrase chacun de son côté : deux endroits à relire le
    /// jour où elle change, pour une seule chose à dire. Un échec du presse-papiers ne se voit
    /// nulle part — c'est justement le cas où un toast a quelque chose à apprendre.
    fn echec_presse_papiers(&mut self, err: impl std::fmt::Display) {
        self.ui.show_toast(format!("Presse-papiers : {err}"));
    }

    /// Écrit `texte` dans le presse-papiers du système. Rend `false` si l'écriture a échoué.
    ///
    /// Un seul site pour un seul message : trois endroits disaient la même phrase d'échec,
    /// donc trois endroits à relire le jour où elle change. Un échec du presse-papiers ne se
    /// voit nulle part — c'est le cas où un toast a vraiment quelque chose à apprendre.
    fn ecrire_presse_papiers(&mut self, texte: String) -> bool {
        match Clipboard::new().and_then(|mut c| c.set_text(texte)) {
            Ok(()) => true,
            Err(err) => {
                self.echec_presse_papiers(err);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests;
