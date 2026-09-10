//! Moteur de rendu de texte bitmap 8x8 pur pour tiny-skia (0 dépendance de police externe).

use tiny_skia::{Color, PixmapMut};

// Police bitmap 8x8 standard pour les caractères ASCII imprimables (32..=126).
// Chaque caractère est représenté par 8 octets, un par ligne de 8 bits.
include!("font_data.rs");

/// Dessine une chaîne de texte sur un Pixmap tiny-skia avec une couleur et un facteur d'échelle.
pub fn draw_text(
    pixmap: &mut PixmapMut,
    text: &str,
    start_x: i32,
    start_y: i32,
    scale: i32,
    color: Color,
) {
    let s = scale.max(1);
    let mut cur_x = start_x;
    let r = (color.red() * 255.0) as u8;
    let g = (color.green() * 255.0) as u8;
    let b = (color.blue() * 255.0) as u8;
    let a = (color.alpha() * 255.0) as u8;

    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        let glyph_idx = if ch.is_ascii() && (ch as usize) >= 32 && (ch as usize) <= 126 {
            (ch as usize) - 32
        } else {
            0 // Espace pour caractères non supportés
        };

        let glyph = FONT_8X8[glyph_idx];
        for row in 0..8 {
            let row_byte = glyph[row];
            for col in 0..8 {
                if (row_byte & (1 << (7 - col))) != 0 {
                    for dy in 0..s {
                        for dx in 0..s {
                            let px = cur_x + col * s + dx;
                            let py = start_y + (row as i32) * s + dy;
                            if px >= 0 && px < width && py >= 0 && py < height {
                                let offset = ((py as usize) * (width as usize) + (px as usize)) * 4;
                                let data = pixmap.data_mut();
                                data[offset] = r;
                                data[offset + 1] = g;
                                data[offset + 2] = b;
                                data[offset + 3] = a;
                            }
                        }
                    }
                }
            }
        }
        cur_x += 8 * s;
    }
}
