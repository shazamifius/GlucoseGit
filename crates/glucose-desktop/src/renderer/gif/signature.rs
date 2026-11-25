//! **Une image qui bouge, à sa signature** (fiche 159 § 9) : un GIF, un WebP animé, un PNG animé
//! (APNG) — reconnus sans les lire entiers : une image rouverte vient de son aperçu et ne passe
//! jamais par le décodeur (GIF-2).

/// La signature d'un PNG, avant ses morceaux.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";

/// **Le verdict sur le début d'une image** : `Ok(animée)`, ou `Err(n)` — il faut ses `n` premiers
/// octets pour le dire (un PNG dont les morceaux d'avant les pixels sont longs, un profil de
/// couleur par exemple). Un début plus court que demandé est celui d'un fichier qui finit.
pub(crate) fn animee(debut: &[u8]) -> Result<bool, usize> {
    if debut.starts_with(b"GIF8") {
        return Ok(true);
    }
    if debut.starts_with(PNG) {
        return png_anime(debut);
    }
    // Un WebP : `RIFF`, sa taille, `WEBP` ; puis `VP8X`, dont le premier octet de données porte
    // les drapeaux — celui de l'animation est le bit 1.
    let Some(tete) = debut.get(..21) else {
        return Err(21);
    };
    Ok(&tete[..4] == b"RIFF" && &tete[8..16] == b"WEBPVP8X" && tete[20] & 0x02 != 0)
}

/// Un PNG est animé si son morceau `acTL` vient avant ses premiers pixels (`IDAT`).
fn png_anime(debut: &[u8]) -> Result<bool, usize> {
    let mut i = PNG.len();
    loop {
        let Some(fin) = i.checked_add(8) else {
            return Ok(false);
        };
        let Some(tete) = debut.get(i..fin) else {
            return Err(fin);
        };
        match &tete[4..] {
            b"acTL" => return Ok(true),
            b"IDAT" | b"IEND" => return Ok(false),
            _ => {}
        }
        let longueur = u32::from_be_bytes([tete[0], tete[1], tete[2], tete[3]]);
        // Sa longueur, son nom, ses données, son contrôle.
        let longueur = usize::try_from(longueur).unwrap_or(usize::MAX);
        i = i.saturating_add(12).saturating_add(longueur);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gif_4_les_signatures_des_images_qui_bougent() {
        assert_eq!(animee(b"GIF89a"), Ok(true));
        let mut webp = b"RIFF\0\0\0\0WEBPVP8X\x0a\0\0\0\x02".to_vec();
        assert_eq!(animee(&webp), Ok(true), "le drapeau de l'animation");
        webp[20] = 0x10;
        assert_eq!(animee(&webp), Ok(false), "un WebP fixe, avec transparence");
        assert_eq!(animee(b"RIFF\0\0\0\0WEBPVP8L\0\0\0\0\x02"), Ok(false));
        assert_eq!(animee(b"RIFF"), Err(21), "trop court pour le dire");
        assert_eq!(animee(b"\xff\xd8\xff\xe0 un JPEG assez long"), Ok(false));
    }

    #[test]
    fn gif_4_un_png_anime_l_est_avant_ses_pixels() {
        let morceau = |nom: &[u8], donnees: usize| {
            let mut m = u32::try_from(donnees).unwrap().to_be_bytes().to_vec();
            m.extend_from_slice(nom);
            m.extend(std::iter::repeat_n(0, donnees + 4));
            m
        };
        let png = |noms: &[(&[u8], usize)]| {
            let mut p = PNG.to_vec();
            noms.iter().for_each(|(nom, n)| p.extend(morceau(nom, *n)));
            p
        };
        let apng = png(&[(b"IHDR", 13), (b"iCCP", 3000), (b"acTL", 8), (b"IDAT", 1)]);
        assert_eq!(
            animee(&apng[..64]),
            Err(PNG.len() + 25 + 3012 + 8),
            "le profil est long"
        );
        assert_eq!(animee(&apng), Ok(true));
        let fixe = png(&[(b"IHDR", 13), (b"IDAT", 1), (b"acTL", 8)]);
        assert_eq!(
            animee(&fixe),
            Ok(false),
            "après les pixels, acTL ne compte pas"
        );
    }
}
