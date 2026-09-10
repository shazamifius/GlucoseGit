//! Moteur de rendu typographique vectoriel anti-aliasé haute performance via fontdue.
//! Embarque directement les polices TTF pour garantir 0 dépendance système externe.
//! Dispose d'un cache de glyphes (atlas mémoire) et d'un mélange alpha prémultiplié (R-26, R-27).

use fontdue::{Font, FontSettings, Metrics};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tiny_skia::{Color, PixmapMut};

#[derive(Clone)]
pub struct GlyphEntry {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
}

pub struct Typography {
    pub regular: Font,
    pub bold: Font,
    glyph_cache: RefCell<HashMap<(bool, char, u16), Arc<GlyphEntry>>>,
}

impl Typography {
    pub fn new() -> Self {
        let reg_bytes = include_bytes!("../assets/font.ttf");
        let bold_bytes = include_bytes!("../assets/font_bold.ttf");
        let regular =
            Font::from_bytes(reg_bytes as &[u8], FontSettings::default()).expect("Failed to load regular font");
        let bold =
            Font::from_bytes(bold_bytes as &[u8], FontSettings::default()).expect("Failed to load bold font");
        Self {
            regular,
            bold,
            glyph_cache: RefCell::new(HashMap::with_capacity(512)),
        }
    }

    /// Récupère ou rastérise un glyphe avec mise en cache (R-26).
    pub fn get_glyph(&self, ch: char, size: f32, bold: bool) -> (char, Arc<GlyphEntry>) {
        let font = if bold { &self.bold } else { &self.regular };
        let safe_ch = normalize_char(font, ch);
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let key = (bold, safe_ch, size_key);

        if let Some(entry) = self.glyph_cache.borrow().get(&key) {
            return (safe_ch, entry.clone());
        }

        let (metrics, bitmap) = font.rasterize(safe_ch, size);
        let entry = Arc::new(GlyphEntry { metrics, bitmap });

        let mut cache = self.glyph_cache.borrow_mut();
        if cache.len() > 4096 {
            // Éviction progressive : garder la moitié au lieu d'une table rase complète (R-40)
            let mut count = 0;
            cache.retain(|_, _| {
                count += 1;
                count % 2 == 0
            });
        }
        cache.insert(key, entry.clone());

        (safe_ch, entry)
    }

    /// Nombre de glyphes actuellement en cache
    pub fn cached_glyph_count(&self) -> usize {
        self.glyph_cache.borrow().len()
    }

    /// Dessine une chaîne de caractères vectorielle avec antialiasing et cache de glyphes (R-26, R-27)
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
        let r = (color.red() * 255.0) as f32;
        let g = (color.green() * 255.0) as f32;
        let b = (color.blue() * 255.0) as f32;
        let a = color.alpha();

        let w = pixmap.width() as i32;
        let h = pixmap.height() as i32;
        let data = pixmap.data_mut();

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let (_, entry) = self.get_glyph(ch, size, bold);
            let metrics = &entry.metrics;
            let bitmap = &entry.bitmap;

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
                            let src_a = a * coverage;
                            let inv_src_a = 1.0 - src_a;

                            let dr = data[idx] as f32;
                            let dg = data[idx + 1] as f32;
                            let db = data[idx + 2] as f32;
                            let da = data[idx + 3] as f32;

                            // R-27: Correction du mélange alpha prémultiplié sans écraser l'alpha
                            data[idx] = (r * src_a + dr * inv_src_a).min(255.0) as u8;
                            data[idx + 1] = (g * src_a + dg * inv_src_a).min(255.0) as u8;
                            data[idx + 2] = (b * src_a + db * inv_src_a).min(255.0) as u8;
                            data[idx + 3] = (src_a * 255.0 + da * inv_src_a).min(255.0) as u8;
                        }
                    }
                }
            }
            x += metrics.advance_width;
        }
        x
    }

    /// Dessine une chaîne avec un contour net ou une ombre en une seule consultation de glyphes (R-26).
    pub fn draw_text_with_outline(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        mut x: f32,
        y: f32,
        size: f32,
        color: Color,
        outline_color: Color,
        bold: bool,
    ) -> f32 {
        let shadow_offsets: [(f32, f32); 8] = [
            (-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5),
            (-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0),
        ];

        let out_r = (outline_color.red() * 255.0) as f32;
        let out_g = (outline_color.green() * 255.0) as f32;
        let out_b = (outline_color.blue() * 255.0) as f32;
        let out_a = outline_color.alpha();

        let r = (color.red() * 255.0) as f32;
        let g = (color.green() * 255.0) as f32;
        let b = (color.blue() * 255.0) as f32;
        let a = color.alpha();

        let w = pixmap.width() as i32;
        let h = pixmap.height() as i32;
        let data = pixmap.data_mut();

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let (_, entry) = self.get_glyph(ch, size, bold);
            let metrics = &entry.metrics;
            let bitmap = &entry.bitmap;

            let base_gx = x + metrics.xmin as f32;
            let base_gy = y + size - metrics.ymin as f32 - metrics.height as f32;

            // 1. Passe contour (8 offsets avec l'outline)
            for &(ox, oy) in &shadow_offsets {
                let gx = base_gx + ox;
                let gy = base_gy + oy;
                for row in 0..metrics.height {
                    for col in 0..metrics.width {
                        let coverage = bitmap[row * metrics.width + col] as f32 / 255.0;
                        if coverage > 0.0 {
                            let px = (gx + col as f32) as i32;
                            let py = (gy + row as f32) as i32;
                            if px >= 0 && px < w && py >= 0 && py < h {
                                let idx = ((py as usize) * (w as usize) + (px as usize)) * 4;
                                let src_a = out_a * coverage;
                                let inv_src_a = 1.0 - src_a;

                                let dr = data[idx] as f32;
                                let dg = data[idx + 1] as f32;
                                let db = data[idx + 2] as f32;
                                let da = data[idx + 3] as f32;

                                data[idx] = (out_r * src_a + dr * inv_src_a).min(255.0) as u8;
                                data[idx + 1] = (out_g * src_a + dg * inv_src_a).min(255.0) as u8;
                                data[idx + 2] = (out_b * src_a + db * inv_src_a).min(255.0) as u8;
                                data[idx + 3] = (src_a * 255.0 + da * inv_src_a).min(255.0) as u8;
                            }
                        }
                    }
                }
            }

            // 2. Passe principale
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col] as f32 / 255.0;
                    if coverage > 0.0 {
                        let px = (base_gx + col as f32) as i32;
                        let py = (base_gy + row as f32) as i32;
                        if px >= 0 && px < w && py >= 0 && py < h {
                            let idx = ((py as usize) * (w as usize) + (px as usize)) * 4;
                            let src_a = a * coverage;
                            let inv_src_a = 1.0 - src_a;

                            let dr = data[idx] as f32;
                            let dg = data[idx + 1] as f32;
                            let db = data[idx + 2] as f32;
                            let da = data[idx + 3] as f32;

                            data[idx] = (r * src_a + dr * inv_src_a).min(255.0) as u8;
                            data[idx + 1] = (g * src_a + dg * inv_src_a).min(255.0) as u8;
                            data[idx + 2] = (b * src_a + db * inv_src_a).min(255.0) as u8;
                            data[idx + 3] = (src_a * 255.0 + da * inv_src_a).min(255.0) as u8;
                        }
                    }
                }
            }

            x += metrics.advance_width;
        }
        x
    }

    /// Mesure la largeur et hauteur d'un texte via le cache de glyphes et métriques de police réelles (R-40)
    pub fn measure_text(&self, text: &str, size: f32, bold: bool) -> (f32, f32) {
        let font = if bold { &self.bold } else { &self.regular };
        let height = font.horizontal_line_metrics(size).map(|m| m.new_line_size).unwrap_or(size * 1.2);
        let mut width = 0.0;
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let (_, entry) = self.get_glyph(ch, size, bold);
            width += entry.metrics.advance_width;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn test_glyph_cache_reuses_entries() {
        let typo = Typography::new();
        assert_eq!(typo.cached_glyph_count(), 0);

        let mut pixmap = Pixmap::new(200, 100).unwrap();
        let color = Color::from_rgba8(255, 255, 255, 255);
        typo.draw_text(&mut pixmap.as_mut(), "Hello", 10.0, 20.0, 14.0, color, false);
        let count_after_first = typo.cached_glyph_count();
        assert!(count_after_first > 0);

        // Réutiliser le texte ne doit pas augmenter le nombre de glyphes rastérisés
        typo.draw_text(&mut pixmap.as_mut(), "Hello", 10.0, 50.0, 14.0, color, false);
        assert_eq!(typo.cached_glyph_count(), count_after_first);
    }

    #[test]
    fn test_alpha_blending_preserves_destination_alpha() {
        let typo = Typography::new();
        let mut pixmap = Pixmap::new(100, 50).unwrap();
        let color = Color::from_rgba8(255, 255, 255, 128);
        typo.draw_text(&mut pixmap.as_mut(), "A", 10.0, 10.0, 16.0, color, false);

        // Les pixels hors de la lettre doivent conserver un alpha transparent (0)
        let data = pixmap.data();
        assert_eq!(data[3], 0);
    }

    #[test]
    fn test_draw_text_with_outline() {
        let typo = Typography::new();
        let mut pixmap = Pixmap::new(200, 100).unwrap();
        let text_color = Color::from_rgba8(255, 255, 255, 255);
        let outline_color = Color::from_rgba8(0, 0, 0, 255);
        let next_x = typo.draw_text_with_outline(
            &mut pixmap.as_mut(),
            "Membrane 1",
            10.0,
            20.0,
            16.0,
            text_color,
            outline_color,
            true,
        );
        assert!(next_x > 10.0);
    }
}
