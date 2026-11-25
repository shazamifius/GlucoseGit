//! **Les dimensions d'une photo, lues dans son en-tête seul** (fiche 164 § 8).
//!
//! `image::ImageReader::into_dimensions` lit **tout** un JPEG pour en dire la taille : son décodeur
//! commence par `read_to_end` (`image` 0.25, `codecs/jpeg/decoder.rs`). Un dossier de 150 photos
//! d'appareil (7,8 Mo chacune) figeait l'image 540 ms à froid, sur le disque NVMe de son poste
//! (mesuré au pilote le 25/11) ; 600 photos, plus de deux secondes, et bien plus sur un disque
//! dur, une clé ou un partage réseau. La taille d'un JPEG est dans son marqueur `SOFn`, avant
//! les données de l'image : on y va de segment en segment, en sautant chaque charge.
//!
//! Les autres formats que Glucose lit ne lisent déjà que leur en-tête : ils passent par `image`.

use image::{ImageError, ImageFormat, ImageReader, ImageResult};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// **La largeur et la hauteur d'une image sur le disque**, l'en-tête seul lu pour un JPEG. C'est
/// la signature du fichier qui dit son format, jamais son extension.
pub(crate) fn dimensions_du_fichier(chemin: &Path) -> ImageResult<(u32, u32)> {
    let lecteur = ImageReader::new(BufReader::new(std::fs::File::open(chemin)?))
        .with_guessed_format()
        .map_err(ImageError::IoError)?;
    if lecteur.format() != Some(ImageFormat::Jpeg) {
        return lecteur.into_dimensions();
    }
    let mut fichier = lecteur.into_inner();
    if let Some(taille) = dimensions_jpeg(&mut fichier) {
        return Ok(taille);
    }
    // Un JPEG que la marche ne comprend pas (une hauteur différée au marqueur DNL, un en-tête
    // abîmé) : le décodeur entier tranche, comme avant.
    fichier.seek(SeekFrom::Start(0))?;
    ImageReader::with_format(fichier, ImageFormat::Jpeg).into_dimensions()
}

/// **La taille d'un JPEG, de segment en segment jusqu'à son `SOFn`** (ITU T.81, annexe B) : chaque
/// segment dit sa longueur, et sa charge se saute sans être lue. `None` si le flux n'est pas un
/// JPEG, s'il finit ou commence ses données (`SOS`) avant d'avoir dit sa taille.
pub(crate) fn dimensions_jpeg<R: BufRead + Seek>(r: &mut R) -> Option<(u32, u32)> {
    let mut deux = [0u8; 2];
    r.read_exact(&mut deux).ok()?;
    if deux != [0xFF, 0xD8] {
        return None;
    }
    loop {
        if octet(r)? != 0xFF {
            return None;
        }
        // Des 0xFF de remplissage peuvent précéder le code du marqueur (B.1.1.2).
        let mut marqueur = octet(r)?;
        while marqueur == 0xFF {
            marqueur = octet(r)?;
        }
        match marqueur {
            // Les marqueurs seuls, sans longueur : TEM, RSTn, un second SOI.
            0x01 | 0xD0..=0xD8 => continue,
            // La fin, ou les données de l'image : la taille aurait dû venir avant.
            0xD9 | 0xDA => return None,
            _ => {}
        }
        r.read_exact(&mut deux).ok()?;
        let longueur = u16::from_be_bytes(deux);
        // SOF0 à SOF15, sauf DHT (C4), JPG (C8) et DAC (CC) qui partagent leur plage.
        if matches!(marqueur, 0xC0..=0xCF) && !matches!(marqueur, 0xC4 | 0xC8 | 0xCC) {
            let mut sof = [0u8; 5];
            r.read_exact(&mut sof).ok()?;
            let hauteur = u32::from(u16::from_be_bytes([sof[1], sof[2]]));
            let largeur = u32::from(u16::from_be_bytes([sof[3], sof[4]]));
            return (largeur > 0 && hauteur > 0).then_some((largeur, hauteur));
        }
        // La longueur compte ses deux propres octets.
        r.seek_relative(i64::from(longueur.checked_sub(2)?)).ok()?;
    }
}

fn octet<R: Read>(r: &mut R) -> Option<u8> {
    let mut un = [0u8; 1];
    r.read_exact(&mut un).ok()?;
    Some(un[0])
}

#[cfg(test)]
mod tests;
