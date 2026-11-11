//! **Les couleurs que le document écrit en texte** (`#rrggbb`, `#rgb`) : les lire, ou dire qu'on ne
//! peut pas. Extrait du moteur de rendu (fiche 104), qui n'a pas à savoir comment une couleur s'écrit.

/// Décode une couleur hexadécimale `#rrggbb` ou `#rgb` ; `None` si ce n'en est pas une.
///
/// Le document range ses couleurs en texte, et un texte peut être faux : l'appelant décide
/// alors de la couleur de repli — en général un jeton du thème — plutôt que d'en recevoir une
/// choisie ici.
pub(crate) fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim_start_matches('#');
    let channel = |from: usize, to: usize| u8::from_str_radix(s.get(from..to)?, 16).ok();
    match s.len() {
        6 => Some((channel(0, 2)?, channel(2, 4)?, channel(4, 6)?)),
        3 => Some((
            channel(0, 1)? * 17,
            channel(1, 2)? * 17,
            channel(2, 3)? * 17,
        )),
        _ => None,
    }
}

/// [`parse_hex_rgb`], avec une couleur de repli.
pub(crate) fn parse_hex_color(
    hex: &str,
    default_r: u8,
    default_g: u8,
    default_b: u8,
) -> (u8, u8, u8) {
    parse_hex_rgb(hex).unwrap_or((default_r, default_g, default_b))
}
