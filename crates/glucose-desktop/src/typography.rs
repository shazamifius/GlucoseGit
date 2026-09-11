//! Moteur de rendu typographique vectoriel anti-aliasé haute performance via fontdue.
//! Embarque directement les polices TTF pour garantir 0 dépendance système externe.
//! Dispose d'un cache de glyphes (atlas mémoire) et d'un mélange alpha prémultiplié (R-26, R-27).

use fontdue::{Font, FontSettings, Metrics};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::{Color, PixmapMut};

#[derive(Clone)]
pub struct GlyphEntry {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
}

/// Clé de cache d'un glyphe : graisse, caractère, taille en dixièmes de point.
type GlyphKey = (bool, char, u16);

/// Valeur de cache : le glyphe partagé et l'horodatage de son dernier accès (LRU, R-40).
type CachedGlyph = (Rc<GlyphEntry>, u64);

/// Style d'un tracé de texte.
///
/// Regroupe les trois paramètres de style pour qu'aucun d'eux — `size` en
/// particulier — ne puisse être confondu avec une coordonnée (R-44).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    pub bold: bool,
}

pub struct Typography {
    pub regular: Font,
    pub bold: Font,
    glyph_cache: RefCell<HashMap<GlyphKey, CachedGlyph>>,
    access_counter: Cell<u64>,
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
            access_counter: Cell::new(0),
        }
    }

    /// Récupère ou rastérise un glyphe avec mise en cache LRU sans atomicité Arc (R-26, R-40).
    pub fn get_glyph(&self, ch: char, size: f32, bold: bool) -> (char, Rc<GlyphEntry>) {
        let size = clamp_font_size(size);
        let font = if bold { &self.bold } else { &self.regular };
        let safe_ch = normalize_char(font, ch);
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let key = (bold, safe_ch, size_key);

        let access = self.access_counter.get().wrapping_add(1);
        self.access_counter.set(access);

        let mut cache = self.glyph_cache.borrow_mut();
        if let Some((entry, last_access)) = cache.get_mut(&key) {
            *last_access = access;
            return (safe_ch, entry.clone());
        }

        let (metrics, bitmap) = font.rasterize(safe_ch, size);
        let entry = Rc::new(GlyphEntry { metrics, bitmap });

        if cache.len() >= 4096 {
            // Éviction LRU (R-40) : évincer les 25% les plus anciens au lieu d'une table rase complète
            let mut accesses: Vec<u64> = cache.values().map(|(_, a)| *a).collect();
            accesses.sort_unstable();
            let cutoff = accesses[accesses.len() / 4];
            cache.retain(|_, (_, a)| *a > cutoff);
        }
        cache.insert(key, (entry.clone(), access));

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
        style: TextStyle,
    ) -> f32 {
        let TextStyle { color, bold, .. } = style;
        let size = clamp_font_size(style.size);
        let r = color.red() * 255.0;
        let g = color.green() * 255.0;
        let b = color.blue() * 255.0;
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
        style: TextStyle,
        outline_color: Color,
    ) -> f32 {
        let TextStyle { color, bold, .. } = style;
        let size = clamp_font_size(style.size);
        let shadow_offsets: [(f32, f32); 8] = [
            (-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5),
            (-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0),
        ];

        let out_r = outline_color.red() * 255.0;
        let out_g = outline_color.green() * 255.0;
        let out_b = outline_color.blue() * 255.0;
        let out_a = outline_color.alpha();

        let r = color.red() * 255.0;
        let g = color.green() * 255.0;
        let b = color.blue() * 255.0;
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
        let size = clamp_font_size(size);
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

/// Taille de glyphe maximale rastérisable.
///
/// Un glyphe est rastérisé dans un bitmap de `size * size` octets puis composé
/// pixel par pixel : sans borne, une taille aberrante (échelle corrompue,
/// NaN, argument inversé) transforme un seul appel de texte en plusieurs
/// centaines de millions d'itérations et gèle l'application. La borne rend ce
/// scénario impossible par construction.
pub const MAX_FONT_SIZE: f32 = 512.0;

/// Normalise une taille de police : NaN et valeurs aberrantes sont ramenées
/// dans une plage rastérisable en temps borné.
pub fn clamp_font_size(size: f32) -> f32 {
    if size.is_nan() {
        return 1.0;
    }
    size.clamp(1.0, MAX_FONT_SIZE)
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
        typo.draw_text(&mut pixmap.as_mut(), "Hello", 10.0, 20.0, TextStyle { size: 14.0, color, bold: false });
        let count_after_first = typo.cached_glyph_count();
        assert!(count_after_first > 0);

        // Réutiliser le texte ne doit pas augmenter le nombre de glyphes rastérisés
        typo.draw_text(&mut pixmap.as_mut(), "Hello", 10.0, 50.0, TextStyle { size: 14.0, color, bold: false });
        assert_eq!(typo.cached_glyph_count(), count_after_first);
    }

    #[test]
    fn test_alpha_blending_preserves_destination_alpha() {
        let typo = Typography::new();
        let mut pixmap = Pixmap::new(100, 50).unwrap();
        let color = Color::from_rgba8(255, 255, 255, 128);
        typo.draw_text(&mut pixmap.as_mut(), "A", 10.0, 10.0, TextStyle { size: 16.0, color, bold: false });

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
            TextStyle { size: 16.0, color: text_color, bold: true },
            outline_color,
        );
        assert!(next_x > 10.0);
    }

    #[test]
    fn test_lru_glyph_cache_eviction_retains_frequent_entries() {
        let typo = Typography::new();
        // Remplir avec 4096 glyphes uniques (différentes tailles)
        for i in 0..4096 {
            let size = 10.0 + (i as f32) * 0.1;
            typo.get_glyph('A', size, false);
        }
        assert_eq!(typo.cached_glyph_count(), 4096);

        // Ré-accéder fréquemment à un glyphe particulier pour qu'il ait un timestamp récent
        typo.get_glyph('A', 10.0, false);

        // Insérer un 4097e glyphe pour déclencher l'éviction LRU
        typo.get_glyph('B', 12.0, false);

        // L'éviction des 25% les plus anciens ramène la taille à ~3072 + 1
        let count = typo.cached_glyph_count();
        assert!(count < 4096);
        assert!(count >= 3072);

        // Le glyphe fréquemment ré-accédé doit toujours être présent en cache sans réallocation
        let pre_count = typo.cached_glyph_count();
        typo.get_glyph('A', 10.0, false);
        assert_eq!(typo.cached_glyph_count(), pre_count);
    }
}
