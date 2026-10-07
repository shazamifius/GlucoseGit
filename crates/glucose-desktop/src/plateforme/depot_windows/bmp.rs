//! **Le bitmap qu'une page pose dans le glisser, devenu un fichier BMP** — sorti de
//! [`super`] quand il a dépassé six cents lignes. Pur : aucun COM ici, seulement des
//! octets.

/// Les quatorze octets qui font d'un DIB un fichier BMP.
///
/// L'offset des pixels se **calcule** depuis l'en-tête : sa taille, sa palette, et les trois
/// masques qu'un `BI_BITFIELDS` ajoute. Le supposer à quarante octets marcherait sur le cas
/// courant et donnerait une image décalée sur tous les autres — le genre de faute qui ne se
/// voit que chez quelqu'un d'autre.
pub(super) fn en_fichier_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    const BI_BITFIELDS: u32 = 3;
    let lire =
        |i: usize| -> Option<u32> { Some(u32::from_le_bytes(dib.get(i..i + 4)?.try_into().ok()?)) };
    let taille_entete = lire(0)? as usize;
    if taille_entete < 12 || taille_entete > dib.len() {
        return None;
    }
    // Un en-tête de douze octets est l'ancien `BITMAPCOREHEADER`, dont les champs ne sont pas
    // aux mêmes places. On ne le traite pas : plus aucun navigateur n'en produit, et le
    // traiter à moitié serait pire que de le refuser.
    if taille_entete < 40 {
        return None;
    }
    let bits = u32::from(u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?));
    let compression = lire(16)?;
    let couleurs = lire(32)? as usize;
    let palette = if bits <= 8 {
        let entrees = if couleurs > 0 {
            couleurs
        } else {
            1usize << bits
        };
        entrees * 4
    } else {
        0
    };
    // Les masques ne suivent l'en-tête que pour un `BITMAPINFOHEADER` ; un V4 ou un V5 les
    // porte dans ses propres champs.
    let masques = if compression == BI_BITFIELDS && taille_entete == 40 {
        12
    } else {
        0
    };
    let debut = 14 + taille_entete + palette + masques;
    let total = 14 + dib.len();
    let mut bmp = Vec::with_capacity(total);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(total as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&(debut as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}

#[cfg(test)]
mod tests {
    use super::en_fichier_bmp;

    /// Un DIB tel que Windows le pose : l'en-tête d'un BMP moins ses quatorze premiers octets.
    fn dib_depuis_un_bmp(bmp: &[u8]) -> Vec<u8> {
        bmp[14..].to_vec()
    }

    /// **Un DIB redevient un BMP que le décodeur du projet sait lire**, et l'image en sort
    /// intacte.
    ///
    /// C'est tout ce qu'on demande à ces quatorze octets, et c'est exactement ce qui se casse
    /// sans qu'on le voie : un offset de pixels supposé à quarante marche sur le cas courant et
    /// donne une image décalée dès qu'une palette ou des masques s'intercalent.
    #[test]
    fn test_un_dib_redevient_un_bmp_lisible() {
        for (largeur, hauteur) in [(7u32, 5u32), (64, 1), (1, 64)] {
            let mut source = image::RgbaImage::new(largeur, hauteur);
            for (x, y, p) in source.enumerate_pixels_mut() {
                *p = image::Rgba([(x * 37 % 256) as u8, (y * 91 % 256) as u8, 40, 255]);
            }
            let mut bmp = std::io::Cursor::new(Vec::new());
            source
                .write_to(&mut bmp, image::ImageFormat::Bmp)
                .expect("ecriture BMP");
            let bmp = bmp.into_inner();

            let refait = en_fichier_bmp(&dib_depuis_un_bmp(&bmp)).expect("un DIB se recompose");
            let relu = image::load_from_memory(&refait)
                .expect("le BMP recompose doit se lire")
                .to_rgba8();
            assert_eq!(relu.dimensions(), (largeur, hauteur));
            for (x, y, p) in relu.enumerate_pixels() {
                assert_eq!(
                    p.0[..3],
                    source.get_pixel(x, y).0[..3],
                    "pixel ({x}, {y}) de l'image {largeur} x {hauteur}"
                );
            }
        }
    }

    /// **Ce qui n'est pas un DIB est refusé**, plutôt que de produire un fichier illisible que
    /// le routage du dépôt prendrait pour un lanceur.
    #[test]
    fn test_ce_qui_n_est_pas_un_dib_est_refuse() {
        assert!(en_fichier_bmp(&[]).is_none());
        assert!(
            en_fichier_bmp(&[0, 0, 0, 0]).is_none(),
            "un en-tete de taille nulle"
        );
        // Un `BITMAPCOREHEADER` de douze octets : refusé sciemment, ses champs ne sont pas aux
        // mêmes places et plus aucun navigateur n'en produit.
        let mut core = vec![0u8; 12];
        core[0] = 12;
        assert!(en_fichier_bmp(&core).is_none());
        // Un en-tête qui annonce plus grand que ce qu'il porte.
        let mut menteur = vec![0u8; 40];
        menteur[0] = 200;
        assert!(en_fichier_bmp(&menteur).is_none());
    }
}
