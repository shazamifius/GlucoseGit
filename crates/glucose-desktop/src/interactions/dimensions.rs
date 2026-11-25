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
//!
//! **La taille dite est celle de la photo droite** (EXIF-1, fiche 165) : une photo de téléphone tenue
//! en portrait écrit ses pixels en paysage, et sa balise EXIF « orientation » dit de les tourner — le
//! décodage les tourne (`renderer::photo::orientation`), le nœud naît donc en portrait.

use image::metadata::Orientation;
use image::{ImageDecoder, ImageError, ImageFormat, ImageReader, ImageResult};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// **La largeur et la hauteur d'une image sur le disque**, l'en-tête seul lu pour un JPEG. C'est
/// la signature du fichier qui dit son format, jamais son extension.
pub(crate) fn dimensions_du_fichier(chemin: &Path) -> ImageResult<(u32, u32)> {
    let lecteur = ImageReader::new(BufReader::new(std::fs::File::open(chemin)?))
        .with_guessed_format()
        .map_err(ImageError::IoError)?;
    if lecteur.format() != Some(ImageFormat::Jpeg) {
        let mut decodeur = lecteur.into_decoder()?;
        let orientation = decodeur.orientation()?;
        return Ok(droite(decodeur.dimensions(), orientation));
    }
    let mut fichier = lecteur.into_inner();
    if let Some(taille) = dimensions_jpeg(&mut fichier) {
        return Ok(taille);
    }
    // Un JPEG que la marche ne comprend pas (une hauteur différée au marqueur DNL, un en-tête
    // abîmé) : le décodeur entier tranche, comme avant.
    fichier.seek(SeekFrom::Start(0))?;
    let mut decodeur = ImageReader::with_format(fichier, ImageFormat::Jpeg).into_decoder()?;
    let orientation = decodeur.orientation()?;
    Ok(droite(decodeur.dimensions(), orientation))
}

/// La taille d'une photo une fois tournée comme elle le demande : un quart de tour échange les côtés.
fn droite((largeur, hauteur): (u32, u32), orientation: Orientation) -> (u32, u32) {
    match orientation {
        Orientation::Rotate90
        | Orientation::Rotate270
        | Orientation::Rotate90FlipH
        | Orientation::Rotate270FlipH => (hauteur, largeur),
        _ => (largeur, hauteur),
    }
}

/// **La taille d'un JPEG, de segment en segment jusqu'à son `SOFn`** (ITU T.81, annexe B) : chaque
/// segment dit sa longueur, et sa charge se saute sans être lue — sauf l'EXIF (`APP1`), où se lit son
/// orientation : la taille rendue est celle de la photo droite. `None` si le flux n'est pas un JPEG,
/// s'il finit ou commence ses données (`SOS`) avant d'avoir dit sa taille.
pub(crate) fn dimensions_jpeg<R: BufRead + Seek>(r: &mut R) -> Option<(u32, u32)> {
    let mut deux = [0u8; 2];
    r.read_exact(&mut deux).ok()?;
    if deux != [0xFF, 0xD8] {
        return None;
    }
    let mut orientation = Orientation::NoTransforms;
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
            return (largeur > 0 && hauteur > 0).then_some(droite((largeur, hauteur), orientation));
        }
        // La longueur compte ses deux propres octets.
        let mut charge = u64::from(longueur.checked_sub(2)?);
        if marqueur == 0xE1 {
            let (lue, dite) = orientation_de_l_exif(r, charge)?;
            charge -= lue;
            orientation = dite.unwrap_or(orientation);
        }
        r.seek_relative(i64::try_from(charge).ok()?).ok()?;
    }
}

/// **L'orientation que dit un segment `APP1`**, lue dans le seul IFD0 de son EXIF : l'en-tête TIFF, ses
/// entrées, rien d'autre — la vignette et le reste se sautent. Rend ce qu'elle a lu de la charge, et
/// l'orientation si elle y est ; `None` seulement si le flux finit.
fn orientation_de_l_exif<R: BufRead + Seek>(
    r: &mut R,
    charge: u64,
) -> Option<(u64, Option<Orientation>)> {
    let mut tete = [0u8; 14];
    if charge < 14 {
        return Some((0, None));
    }
    r.read_exact(&mut tete).ok()?;
    let gros = match tete[6..10] {
        [b'I', b'I', 42, 0] if tete[..6] == *b"Exif\0\0" => false,
        [b'M', b'M', 0, 42] if tete[..6] == *b"Exif\0\0" => true,
        _ => return Some((14, None)),
    };
    let u16_de = |o: [u8; 2]| {
        if gros {
            u16::from_be_bytes(o)
        } else {
            u16::from_le_bytes(o)
        }
    };
    let quatre = [tete[10], tete[11], tete[12], tete[13]];
    let ifd = u64::from(if gros {
        u32::from_be_bytes(quatre)
    } else {
        u32::from_le_bytes(quatre)
    });
    // L'IFD0 se compte depuis le début du TIFF, six octets après celui de la charge.
    let Some(avant) = ifd.checked_sub(8).filter(|a| 14 + a + 2 <= charge) else {
        return Some((14, None));
    };
    r.seek_relative(i64::try_from(avant).ok()?).ok()?;
    let mut deux = [0u8; 2];
    r.read_exact(&mut deux).ok()?;
    let mut lue = 14 + avant + 2;
    for _ in 0..u16_de(deux) {
        if lue + 12 > charge {
            break;
        }
        let mut entree = [0u8; 12];
        r.read_exact(&mut entree).ok()?;
        lue += 12;
        if u16_de([entree[0], entree[1]]) == 0x0112 {
            let valeur = u16_de([entree[8], entree[9]]);
            return Some((
                lue,
                u8::try_from(valeur).ok().and_then(Orientation::from_exif),
            ));
        }
    }
    Some((lue, None))
}

fn octet<R: Read>(r: &mut R) -> Option<u8> {
    let mut un = [0u8; 1];
    r.read_exact(&mut un).ok()?;
    Some(un[0])
}

#[cfg(test)]
mod tests;
