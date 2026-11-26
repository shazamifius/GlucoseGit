//! **Une image BGRA en rangées, rendue en RGBA serré** : ce que rendent les lecteurs de Windows (un
//! bitmap WIC) et du Mac (un `CVPixelBuffer`) — chacun ses rangées, plus larges que l'image
//! quand le système les aligne.

/// **L'image de `largeur` × `hauteur` dont chaque rangée commence tous les `pas` octets**, en
/// RGBA serré et opaque (une vidéo n'a pas de transparence) ; `None` si les octets manquent.
pub(crate) fn en_rgba(bgra: &[u8], pas: usize, largeur: u32, hauteur: u32) -> Option<Vec<u8>> {
    let (ligne, hauteur) = (largeur as usize * 4, hauteur as usize);
    let utile = pas.checked_mul(hauteur.checked_sub(1)?)? + ligne;
    if pas < ligne || bgra.len() < utile {
        return None;
    }
    let mut rgba = Vec::with_capacity(ligne * hauteur);
    for rangee in bgra.chunks(pas).take(hauteur) {
        for p in rangee[..ligne].as_chunks::<4>().0 {
            rgba.extend_from_slice(&[p[2], p[1], p[0], 255]);
        }
    }
    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::en_rgba;

    /// Deux pixels par rangée, deux rangées alignées sur 12 octets : le bleu et le rouge
    /// s'échangent, l'alpha devient plein, la marge de chaque rangée tombe.
    #[test]
    fn video_windows_1_le_bgra_aligne_devient_du_rgba_serre() {
        let bgra = [
            1, 2, 3, 0, 4, 5, 6, 0, 9, 9, 9, 9, // première rangée, puis sa marge
            7, 8, 9, 0, 10, 11, 12, 0, // la dernière n'a pas besoin de la sienne
        ];
        let rgba = en_rgba(&bgra, 12, 2, 2).expect("assez d'octets");
        assert_eq!(
            rgba,
            [3, 2, 1, 255, 6, 5, 4, 255, 9, 8, 7, 255, 12, 11, 10, 255]
        );
        assert_eq!(en_rgba(&bgra[..19], 12, 2, 2), None, "un octet manque");
        assert_eq!(
            en_rgba(&bgra, 4, 2, 2),
            None,
            "des rangées plus courtes que l'image"
        );
        assert_eq!(en_rgba(&bgra, 12, 2, 0), None, "aucune rangée");
    }
}
