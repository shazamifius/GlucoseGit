//! **Ces octets sont-ils une vidéo ?** À leur début seulement (VIDEO-1, fiche 169) : MP4 et
//! QuickTime (ce que filment les téléphones), Matroska et WebM, AVI.
//!
//! Le piège : un HEIC ou un AVIF est aussi une boîte `ftyp` (ISO BMFF, fiche 161). Ce qui les
//! sépare est la marque qui suit : une marque d'image fixe ou de suite d'images n'est pas une
//! vidéo — ces fichiers restent aux photos.

/// Les marques ISO BMFF d'une image (HEIF, HEIC, AVIF), jamais d'une vidéo.
const MARQUES_D_IMAGE: [&[u8; 4]; 12] = [
    b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx", b"hevm", b"hevs", b"mif1", b"msf1",
    b"avif", b"avis",
];

/// Les premières boîtes d'un vieux QuickTime, écrit avant que `ftyp` n'existe.
const BOITES_QUICKTIME: [&[u8; 4]; 5] = [b"moov", b"mdat", b"wide", b"free", b"skip"];

/// **Les extensions du dialogue d'import** (fiche 172 § 6) : les conteneurs que [`est_video`]
/// reconnaît, et rien d'autre — un MPEG-TS ou un Ogg choisi y deviendrait un lanceur. Un filtre
/// seulement : la signature décide, jamais le nom. Le téléphone a son sélecteur de photos.
#[cfg(not(target_os = "android"))]
pub(crate) const EXTENSIONS: [&str; 7] = ["mp4", "m4v", "mov", "3gp", "mkv", "webm", "avi"];

/// Les octets qu'il faut pour le dire.
pub(crate) const LONGUEUR: usize = 12;

/// **Vrai si ce début est celui d'une vidéo.**
pub(crate) fn est_video(debut: &[u8]) -> bool {
    let boite = debut.get(4..8);
    if boite == Some(b"ftyp") {
        return debut
            .get(8..12)
            .is_some_and(|marque| !MARQUES_D_IMAGE.iter().any(|m| m == &marque));
    }
    if boite.is_some_and(|b| BOITES_QUICKTIME.iter().any(|m| m == &b)) {
        return true;
    }
    debut.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
        || (debut.starts_with(b"RIFF") && debut.get(8..12) == Some(b"AVI "))
}

/// **L'extension que ce début de vidéo porte** (RECUE-1, fiche 172 § 8) : le nom d'une vidéo
/// reçue, qu'on enregistre ailleurs, doit dire ce qu'elle est — `partage` sans extension ne
/// s'ouvrait nulle part. `None` si ce n'est pas une vidéo. Matroska et WebM ne se distinguent
/// qu'au-delà de ces douze octets : `mkv`, que tout lecteur de WebM lit aussi.
pub(crate) fn extension(debut: &[u8]) -> Option<&'static str> {
    if !est_video(debut) {
        return None;
    }
    Some(match (debut.get(4..8), debut.get(8..11)) {
        (Some(b"ftyp"), Some(b"qt ")) => "mov",
        (Some(b"ftyp"), Some(b"M4V")) => "m4v",
        (Some(b"ftyp"), Some(b"3gp" | b"3g2")) => "3gp",
        (Some(b"ftyp"), _) => "mp4",
        _ if debut.starts_with(b"RIFF") => "avi",
        _ if debut.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) => "mkv",
        _ => "mov",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ftyp(marque: &[u8; 4]) -> Vec<u8> {
        [&[0, 0, 0, 24][..], b"ftyp", marque].concat()
    }

    #[test]
    fn video_1_les_videos_de_telephone_et_du_web_se_reconnaissent() {
        for marque in [b"isom", b"mp42", b"qt  ", b"M4V ", b"3gp5", b"avc1"] {
            assert!(est_video(&ftyp(marque)), "{marque:?}");
        }
        assert!(est_video(&[0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81]));
        assert!(est_video(b"RIFF\x10\0\0\0AVI LIST"));
        assert!(est_video(b"\0\0\0\x08wide\0\0\0\0"));
        let fixture = include_bytes!("../../../tests/donnees/rouge-64x48.mp4");
        assert!(est_video(&fixture[..LONGUEUR]));
    }

    #[test]
    fn recue_1_l_extension_suit_la_signature() {
        let cas: [(Vec<u8>, &str); 7] = [
            (ftyp(b"isom"), "mp4"),
            (ftyp(b"qt  "), "mov"),
            (ftyp(b"M4V "), "m4v"),
            (ftyp(b"3gp5"), "3gp"),
            (b"RIFF\x10\0\0\0AVI LIST".to_vec(), "avi"),
            (vec![0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81], "mkv"),
            (b"\0\0\0\x08wide\0\0\0\0".to_vec(), "mov"),
        ];
        for (debut, attendue) in cas {
            assert_eq!(extension(&debut), Some(attendue), "{debut:?}");
        }
        assert_eq!(extension(&ftyp(b"heic")), None, "une photo n'en a pas");
    }

    #[test]
    fn video_1_un_heic_un_avif_un_webp_restent_des_photos() {
        for marque in [b"heic", b"mif1", b"avif", b"avis", b"hevc"] {
            assert!(!est_video(&ftyp(marque)), "{marque:?}");
        }
        assert!(!est_video(b"RIFF\x10\0\0\0WEBPVP8 "));
        assert!(!est_video(b"GIF89a"));
        assert!(!est_video(b"\x89PNG\r\n\x1a\n"));
        assert!(!est_video(b"\0\0"));
    }
}
