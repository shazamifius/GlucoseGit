//! Moteur de rendu typographique vectoriel anti-aliasé haute performance via fontdue.
//! Embarque directement les polices TTF pour garantir 0 dépendance système externe.

use fontdue::{Font, FontSettings};
use tiny_skia::{Color, PixmapMut};

pub struct Typography {
    pub regular: Font,
    pub bold: Font,
}

impl Typography {
    pub fn new() -> Self {
        let reg_bytes = include_bytes!("../assets/font.ttf");
        let bold_bytes = include_bytes!("../assets/font_bold.ttf");
        let regular =
            Font::from_bytes(reg_bytes as &[u8], FontSettings::default()).expect("Failed to load regular font");
        let bold =
            Font::from_bytes(bold_bytes as &[u8], FontSettings::default()).expect("Failed to load bold font");
        Self { regular, bold }
    }

    /// Dessine une chaîne de caractères vectorielle avec antialiasing
    pub fn draw_text(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        mut x: f32,
        y: f32,
        size: f32,
        color: Color,
        bold: bool,
    ) -> f32 {
        let font = if bold { &self.bold } else { &self.regular };
        let r = (color.red() * 255.0) as f32;
        let g = (color.green() * 255.0) as f32;
        let b = (color.blue() * 255.0) as f32;
        let a = color.alpha();

        let w = pixmap.width() as i32;
        let h = pixmap.height() as i32;

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let safe_ch = normalize_char(font, ch);
            let (metrics, bitmap) = font.rasterize(safe_ch, size);
            let gx = x + metrics.xmin as f32;
            let gy = y + size - metrics.ymin as f32 - metrics.height as f32;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col] as f32 / 255.0;
                    if coverage > 0.0 {
                        let px = (gx + col as f32) as i32;
                        let py = (gy + row as f32) as i32;
                        if px >= 0 && px < w && py >= 0 && py < h {
                            let idx = ((py as usize) * (w as usize) + (px as usize)) * 4;
                            let data = pixmap.data_mut();
                            let effective_alpha = a * coverage;
                            let inv_alpha = 1.0 - effective_alpha;

                            let dr = data[idx] as f32;
                            let dg = data[idx + 1] as f32;
                            let db = data[idx + 2] as f32;

                            data[idx] = (r * effective_alpha + dr * inv_alpha).min(255.0) as u8;
                            data[idx + 1] = (g * effective_alpha + dg * inv_alpha).min(255.0) as u8;
                            data[idx + 2] = (b * effective_alpha + db * inv_alpha).min(255.0) as u8;
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
            x += metrics.advance_width;
        }
        x
    }

    /// Mesure la largeur et hauteur d'un texte
    pub fn measure_text(&self, text: &str, size: f32, bold: bool) -> (f32, f32) {
        let font = if bold { &self.bold } else { &self.regular };
        let mut width = 0.0;
        let height = size;
        for ch in text.chars() {
            let safe_ch = normalize_char(font, ch);
            let metrics = font.metrics(safe_ch, size);
            width += metrics.advance_width;
        }
        (width, height)
    }
}

fn normalize_char(font: &Font, ch: char) -> char {
    if font.lookup_glyph_index(ch) != 0 {
        return ch;
    }
    match ch {
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'à' | 'â' | 'ä' => 'a',
        'À' | 'Â' | 'Ä' => 'A',
        'î' | 'ï' => 'i',
        'Î' | 'Ï' => 'I',
        'ô' | 'ö' => 'o',
        'Ô' | 'Ö' => 'O',
        'ù' | 'û' | 'ü' => 'u',
        'Ù' | 'Û' | 'Ü' => 'U',
        'ç' => 'c',
        'Ç' => 'C',
        'œ' => 'o',
        '’' => '\'',
        '—' | '–' => '-',
        '→' => '>',
        '←' => '<',
        '↺' => 'R',
        _ => ch,
    }
}
