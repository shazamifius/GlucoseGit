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
        let choisis = self.sous_un_dialogue(|fenetre| {
            crate::dialogue::fichier(fenetre)
                .add_filter("Images", &IMAGE_EXTENSIONS)
                .pick_files()
        });
        if let Some(files) = choisis {
            self.import_image_files(&files);
        }
    }

    /// Importe une liste de chemins de fichiers image dans le board actif.
    pub fn import_image_files(&mut self, paths: &[PathBuf]) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.viewport();
        let (mut cur_wx, mut cur_wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        if self.mouse_pos.1 < self.ui.header_height() as f64 {
            cur_wx = 0.0;
            cur_wy = 0.0;
        }

        let mut count = 0;
        for path_buf in paths {
            // Ici, on a promis des images : le dialogue est filtré, le collage a vérifié le
            // chemin. Un échec est donc une faute, et elle se dit.
            match self.place_image_file(&active_bid, path_buf, (cur_wx, cur_wy)) {
                Ok(posee) => {
                    cur_wx += posee + 30.0;
                    count += 1;
                }
                Err(err) => self.ui.show_toast(err.to_string()),
            }
        }

        if count > 0 {
            self.ui.show_toast(format!("{} image(s) ajoutée(s)", count));
            self.mark_dirty();
        }
    }

    /// Pose une image à un point du monde, et rend la **largeur** qu'elle occupe.
    ///
    /// La largeur sert à ce qui pose une suite d'images côte à côte. C'est le seul endroit
    /// qui sait fabriquer une image depuis un chemin — l'import par dialogue, le collage et
    /// le glisser-déposer passent tous par ici plutôt que d'en avoir chacun sa version.
    ///
    /// **Elle ne parle pas.** L'échec est une erreur rendue, jamais un toast : pour
    /// l'import par dialogue un fichier illisible est une faute à signaler, alors que pour
    /// le glisser-déposer c'est simplement la preuve que ce n'était pas une image, et que
    /// le fichier mérite un lanceur. Un toast posé ici mentirait dans le second cas.
    pub(crate) fn place_image_file(
        &mut self,
        board: &str,
        path: &Path,
        (x, y): (f64, f64),
    ) -> Result<f64, DesktopError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| DesktopError::ImageDimensionsFailed {
                path: path.to_string_lossy().into_owned(),
                reason: "chemin illisible".to_string(),
            })?;
        // La **signature** du fichier décide, pas son extension. `image_dimensions` choisit
        // son décodeur d'après l'extension seule : un PNG nommé `.bin` — ou renommé `.jpg`,
        // ce qui est courant sur ce qui vient du web — lui est illisible. `with_guessed_format`
        // lit l'en-tête et tranche pour de bon, ce qui est la seule façon de tenir la
        // promesse du glisser-déposer : c'est le décodeur qui dit ce qui est une image.
        let dimensions = image::ImageReader::open(path)
            .and_then(image::ImageReader::with_guessed_format)
            .map_err(image::ImageError::IoError)
            .and_then(image::ImageReader::into_dimensions);
        let (w, h) = match dimensions {
            Ok((w, h)) => (f64::from(w), f64::from(h)),
            Err(err) => {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("image");
                return Err(DesktopError::ImageDimensionsFailed {
                    path: filename.to_string(),
                    reason: err.to_string(),
                });
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
        let mut img = BoardImage::new(id, x, y, final_w, final_h);
        img.src = Some(path_str.to_string());
        img.original_width = w;
        img.original_height = h;
        self.store.add_image(board, img);
        Ok(final_w)
    }

    /// Coller depuis le presse-papiers (Image ou Texte).
    pub fn paste_from_clipboard(&mut self) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.viewport();
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
