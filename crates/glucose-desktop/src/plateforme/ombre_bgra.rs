//! **L'ombre telle que Windows la lit** (Q10, fiche 156). `IDragSourceHelper::InitializeFromBitmap`
//! multiplie lui-même chaque couleur par son alpha : des pixels déjà prémultipliés le seraient deux
//! fois, l'ombre assombrie et sa lueur rongée (la documentation de Microsoft : « always pass a
//! bitmap without premultiplied alpha blending ») — Chromium les rend droits pour la même raison
//! (`drag_utils_win.cc`). Et un DIB de 32 bits range ses canaux bleu, vert, rouge, alpha.
//!
//! Écrit hors de Windows, pour s'éprouver partout.

/// Les pixels de l'ombre (rouge, vert, bleu, alpha prémultipliés) en bleu, vert, rouge, alpha
/// droits — chaque couleur divisée par son alpha, à l'arrondi ; un pixel transparent reste nul.
pub fn en_bgra_droit(rgba: &[u8]) -> Vec<u8> {
    let (pixels, _) = rgba.as_chunks::<4>();
    pixels
        .iter()
        .flat_map(|p| {
            let alpha = u32::from(p[3]);
            let droit = |c: u8| match alpha {
                0 => 0,
                a => ((u32::from(c) * 255 + a / 2) / a).min(255) as u8,
            };
            [droit(p[2]), droit(p[1]), droit(p[0]), p[3]]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::en_bgra_droit;

    #[test]
    fn test_un_pixel_opaque_change_seulement_l_ordre_de_ses_canaux() {
        assert_eq!(en_bgra_droit(&[10, 20, 30, 255]), [30, 20, 10, 255]);
    }

    #[test]
    fn test_un_pixel_a_demi_transparent_retrouve_sa_couleur() {
        // Le rouge pur à 50 % : prémultiplié (128, 0, 0, 128) ; droit, (255, 0, 0, 128).
        assert_eq!(en_bgra_droit(&[128, 0, 0, 128]), [0, 0, 255, 128]);
        // Un gris à 20 % : (51, 51, 51, 51) redevient blanc.
        assert_eq!(en_bgra_droit(&[51, 51, 51, 51]), [255, 255, 255, 51]);
    }

    #[test]
    fn test_un_pixel_transparent_reste_nul() {
        assert_eq!(
            en_bgra_droit(&[0, 0, 0, 0, 7, 8, 9, 255]),
            [0, 0, 0, 0, 9, 8, 7, 255]
        );
    }
}
