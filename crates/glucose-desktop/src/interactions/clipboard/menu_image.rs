//! **Clic droit sur une image** (fiche 51 § 3) : la copier comme une image, que Discord, un
//! navigateur ou Photoshop collent ; l'enregistrer telle qu'elle a été posée.
//!
//! # Ce que « copier l'image » pose, et pourquoi
//!
//! Ce que pose « Copier l'image » d'un navigateur : un **PNG** sous le format enregistré « PNG »
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
use crate::interactions::presse_papiers;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;

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

    /// **Copier l'image** : ses octets se lisent et se préparent sur un fil à part.
    pub(crate) fn copier_l_image(&mut self) {
        let Some(cle) = self.image_seule_choisie().and_then(|i| i.src) else {
            return;
        };
        let objets = Arc::clone(&self.disque.objets);
        let systeme = presse_papiers::est_celui_du_systeme();
        let (envoi, recu) = channel();
        std::thread::spawn(move || {
            let prete = objets
                .lire_en_attendant(&cle)
                .ok_or_else(|| "ses octets sont introuvables".to_string())
                .and_then(|o| preparer(&o));
            // Le presse-papiers du système se remplit d'ici ; celui d'une épreuve, qui vit sur
            // le fil qui l'a ouvert, se remplit au retour.
            let rendu = match prete {
                Ok(image) if systeme => {
                    crate::plateforme::presse_papiers::ecrire_une_image(&image).map(|()| None)
                }
                Ok(image) => Ok(Some(image)),
                Err(e) => Err(e),
            };
            let _ = envoi.send(rendu);
        });
        self.echanges.image = Some(recu);
    }

    /// Ce que le fil de « Copier l'image » a rendu.
    pub(super) fn finir_l_image(&mut self, rendu: Option<Result<Option<ImagePosee>, String>>) {
        let rendu = rendu.unwrap_or_else(|| Err("la copie n'a pas abouti".into()));
        let ecrit = rendu.and_then(|image| match image {
            Some(image) => presse_papiers::ouvrir().and_then(|mut a| a.ecrire_une_image(image)),
            None => Ok(()),
        });
        match ecrit {
            Ok(()) => self.ui.show_toast("Image copiée"),
            Err(e) => self.echec_presse_papiers(e),
        }
        self.mark_dirty();
    }

    /// **Enregistrer l'image sous…** : demande où, puis écrit ses octets d'origine, et dit ce
    /// qui a eu lieu — une seule phrase pour les deux issues.
    pub(crate) fn enregistrer_l_image_sous(&mut self) {
        if let Some(message) = self.demander_et_enregistrer() {
            self.ui.show_toast(message);
        }
    }

    /// Rend ce qu'il faut dire, ou `None` si l'on a renoncé.
    fn demander_et_enregistrer(&mut self) -> Option<String> {
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
        let choisi = self.sous_un_dialogue(|ancre| {
            crate::dialogue::fichier(ancre)
                .filtre(nom, &[extension])
                .nom(format!("{base}.{extension}"))
                .enregistrer()
        });
        let mut chemin = choisi?;
        if chemin.extension().is_none() {
            chemin.set_extension(extension);
        }
        Some(match ecrire_l_image(&chemin, &octets) {
            Ok(()) => format!(
                "Image enregistrée — {}",
                chemin.file_name().and_then(|n| n.to_str()).unwrap_or("")
            ),
            Err(e) => e,
        })
    }
}

/// Écrit ces octets, tels quels, à ce chemin.
pub fn ecrire_l_image(chemin: &Path, octets: &[u8]) -> Result<(), String> {
    crate::persist::atomic::write_atomic(chemin, octets).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
