//! **Une image, hors de Glucose** (fiche 51 § 3) : la préparer comme une image, que Discord, un
//! navigateur ou Photoshop collent ; l'enregistrer telle qu'elle a été posée.
//!
//! # Ce que l'image seule d'une copie pose, et pourquoi
//!
//! « Copier l'image » était un geste à part ; `Ctrl+C` le fait désormais lui-même, quand la
//! sélection n'emporte qu'une image (COPIER-1, fiche 59). Ce qu'il pose est ce que pose
//! « Copier l'image » d'un navigateur : un **PNG** sous le format enregistré « PNG »
//! — celui que Chromium lit d'abord, donc Discord et toute page web, et qu'Office préfère —, et
//! les **pixels** en `CF_DIBV5`, que Windows sait donner à tout logiciel qui ne lit que le
//! bitmap (Paint, Word). Quand l'image posée est déjà un PNG, ce sont **ses octets mêmes** :
//! rien n'est réencodé. Un JPEG ou un WebP se décode et s'encode en PNG, sans perte.
//!
//! Un GIF animé ne garde que sa première image : c'est ce que fait un navigateur, et un
//! presse-papiers d'image ne porte pas d'animation.
//!
//! # Ce que « enregistrer l'image sous » écrit
//!
//! Les octets que le document a **scellés**, tels quels, avec l'extension que leur signature dit :
//! le fichier enregistré est identique, octet pour octet, à celui qui avait été posé. Il s'écrit
//! par la seule porte qui pose un fichier à la place d'un autre (cliquet 11).

use crate::app::GlucoseApp;
use std::path::Path;

/// Ce que le presse-papiers reçoit d'une image : un PNG, et ses pixels.
pub struct ImagePosee {
    pub png: Vec<u8>,
    pub largeur: u32,
    pub hauteur: u32,
    /// Rangée par rangée, du haut vers le bas, quatre octets par pixel.
    pub rgba: Vec<u8>,
}

/// **Prépare une image pour le presse-papiers.** Lourd — un décodage, parfois un encodage — :
/// jamais sur le fil qui dessine.
pub fn preparer(octets: &[u8]) -> Result<ImagePosee, String> {
    let pixels = image::load_from_memory(octets)
        .map_err(|e| format!("l'image ne se décode pas : {e}"))?
        .to_rgba8();
    let (largeur, hauteur) = pixels.dimensions();
    let png = if image::guess_format(octets).ok() == Some(image::ImageFormat::Png) {
        octets.to_vec()
    } else {
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&pixels, largeur, hauteur, image::ExtendedColorType::Rgba8)
            .map_err(|e| e.to_string())?;
        png
    };
    Ok(ImagePosee {
        png,
        largeur,
        hauteur,
        rgba: pixels.into_raw(),
    })
}

/// L'extension que la signature des octets dit, et le nom de leur format.
pub fn extension_de(octets: &[u8]) -> (&'static str, &'static str) {
    match image::guess_format(octets) {
        Ok(image::ImageFormat::Png) => ("png", "Image PNG"),
        Ok(image::ImageFormat::Jpeg) => ("jpg", "Image JPEG"),
        Ok(image::ImageFormat::WebP) => ("webp", "Image WebP"),
        Ok(image::ImageFormat::Gif) => ("gif", "Image GIF"),
        Ok(image::ImageFormat::Bmp) => ("bmp", "Image BMP"),
        _ => ("bin", "Fichier"),
    }
}

use image::ImageEncoder;

impl GlucoseApp {
    /// L'image que le menu vise : la seule sélectionnée, et rien d'autre avec elle.
    pub(crate) fn image_seule_choisie(&self) -> Option<glucose_core::types::BoardImage> {
        if self.store.selected_image_ids.len() != 1
            || !self.store.selected_annotation_ids.is_empty()
        {
            return None;
        }
        let board = &self.store.project.active_board_id;
        self.store
            .image(board, &self.store.selected_image_ids[0])
            .cloned()
    }

    /// **Enregistrer l'image sous…** : demande où ; le choix revenu, ses octets d'origine
    /// s'écrivent ([`Self::ecrire_l_image_choisie`]), et ce qui a eu lieu se dit — une seule
    /// phrase pour toutes les issues.
    pub(crate) fn enregistrer_l_image_sous(&mut self) {
        if let Some(message) = self.demander_ou_enregistrer_l_image() {
            self.dire_l_image(message);
        }
    }

    /// Le choix revenu : les octets s'écrivent à ce chemin, sous leur extension s'il n'en a pas.
    pub(crate) fn ecrire_l_image_choisie(
        &mut self,
        mut chemin: std::path::PathBuf,
        octets: &[u8],
        extension: &'static str,
    ) {
        if chemin.extension().is_none() {
            chemin.set_extension(extension);
        }
        let message = match ecrire_l_image(&chemin, octets) {
            Ok(()) => format!(
                "Image enregistrée — {}",
                chemin.file_name().and_then(|n| n.to_str()).unwrap_or("")
            ),
            Err(e) => e,
        };
        self.dire_l_image(message);
    }

    /// La seule voix de l'image enregistrée.
    fn dire_l_image(&mut self, message: String) {
        self.ui.show_toast(message);
    }

    /// Ouvre le sélecteur, ou rend ce qu'il faut dire tout de suite.
    fn demander_ou_enregistrer_l_image(&mut self) -> Option<String> {
        let image = self.image_seule_choisie()?;
        let Some(octets) = image
            .src
            .as_deref()
            .and_then(|c| self.disque.objets.lire_en_attendant(c))
        else {
            return Some("Les octets de cette image sont introuvables".into());
        };
        let (extension, nom) = extension_de(&octets);
        let base = image
            .src
            .as_deref()
            .map(Path::new)
            .filter(|p| p.is_absolute())
            .and_then(Path::file_stem)
            .and_then(|s| s.to_str())
            .unwrap_or("image")
            .to_string();
        let fichier = crate::dialogue::fichier()
            .filtre(nom, &[extension])
            .nom(format!("{base}.{extension}"));
        let demande = crate::persist::choix::Demande::EnregistrerLImage(octets, extension);
        self.demander_un_fichier(fichier, crate::dialogue::Mode::Enregistrer, demande);
        None
    }
}

/// Écrit ces octets, tels quels, à ce chemin.
pub fn ecrire_l_image(chemin: &Path, octets: &[u8]) -> Result<(), String> {
    crate::persist::atomic::write_atomic(chemin, octets).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
