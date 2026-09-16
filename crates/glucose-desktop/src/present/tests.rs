//! Ce que la présentation garantit, des deux côtés de la frontière.

use super::pixel_fenetre;

/// La conversion rapide rend **exactement** ce que la recomposition octet par octet rendait.
///
/// Une optimisation qui change la couleur d'un pixel n'est pas une optimisation : c'est un
/// défaut plus rapide. Le test parcourt chaque valeur possible sur chaque canal, l'alpha
/// compris — il n'échantillonne pas, il démontre.
#[test]
fn test_the_fast_conversion_is_the_same_pixel() {
    for v in 0..=255u8 {
        for (i, canal) in [
            [v, 0, 0, 255],
            [0, v, 0, 255],
            [0, 0, v, 255],
            [7, 9, 11, v],
        ]
        .into_iter()
        .enumerate()
        {
            let attendu =
                (u32::from(canal[0]) << 16) | (u32::from(canal[1]) << 8) | u32::from(canal[2]);
            assert_eq!(
                pixel_fenetre(canal),
                attendu,
                "canal {i}, valeur {v} : {canal:?}"
            );
        }
    }
}

/// L'alpha ne doit **jamais** atteindre la fenêtre : elle attend `0RGB`, et un octet de
/// poids fort non nul y serait lu comme une couleur.
#[test]
fn test_the_alpha_never_reaches_the_window() {
    for a in 0..=255u8 {
        assert_eq!(pixel_fenetre([1, 2, 3, a]) >> 24, 0, "alpha {a} a fuité");
    }
}
