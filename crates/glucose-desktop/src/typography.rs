//! Moteur de rendu typographique vectoriel anti-aliasé haute performance via fontdue.
//! Embarque directement les polices TTF pour garantir 0 dépendance système externe.
//! Dispose d'un cache de glyphes (atlas mémoire) et d'un mélange alpha prémultiplié (R-26, R-27).
//!
//! # FONT-1 — la police embarquée couvre le français, et un test le prouve
//!
//! La police d'origine était `KaTeX_SansSerif`, copiée telle quelle depuis les dépendances
//! JavaScript de l'ancienne version : une police de **formules**, 121 points de code, pas un
//! seul accent (**R-51**). Chaque « é » de l'interface passait par une table de repli qui le
//! remplaçait par « e », et tout ce que l'utilisateur tapait avec un accent était amputé —
//! un moodboard en français ne pouvait pas contenir de français.
//!
//! La police est désormais **Inter** (Regular et SemiBold, SIL Open Font License 1.1, texte
//! dans `assets/LICENSE-Inter.txt`) : 2 505 points de code, accents français complets,
//! ligatures, guillemets, tirets, flèches. Il n'y a plus de table de repli : un caractère
//! absent de la police se dessine en `.notdef`, visiblement, plutôt que d'être remplacé en
//! silence par un autre. Et `typography/coverage.rs` lit la table `cmap` de chaque police
//! embarquée pour affirmer la présence d'un ensemble nommé de caractères, dont **tous les
//! caractères non-ASCII des chaînes littérales de ce crate** : changer de police sans
//! couvrir le français casse la build, au lieu de casser l'écran.
//!
//! # GLYPH-1 — un glyphe se pose à sa vraie place, pas à la place entière la plus proche
//!
//! La position calculée d'un glyphe est fractionnaire : la plume avance de
//! `advance_width`, qui ne tombe jamais sur un entier. Le code d'origine écrivait
//! `(gx + col as f32) as i32`, c'est-à-dire une **troncature**, indépendante pour chaque
//! glyphe. Trois conséquences, toutes visibles (**R-46**) :
//!
//! 1. l'espacement entre deux lettres d'un même mot était faux de 0 à 1 px, au hasard de
//!    l'endroit où chacune tombait ;
//! 2. le texte tremblait pendant un déplacement, chaque glyphe franchissant son seuil
//!    d'arrondi à un instant différent ;
//! 3. le rendu paraissait pixelisé alors que la rastérisation, elle, était bonne.
//!
//! Ici, la partie fractionnaire de la position devient une **phase** : le glyphe est
//! rastérisé une fois par `fontdue`, puis décalé d'une fraction de pixel par interpolation
//! bilinéaire, et la variante obtenue est mise en cache avec sa phase dans la clé. Le blit
//! reste une boucle entière, donc le coût par frame ne change pas ; ce qu'on paie est une
//! interpolation par variante, une seule fois, et quelques entrées de cache de plus.
//!
//! [`SUBPIXEL_PHASES`] positions par axe bornent l'erreur résiduelle à un huitième de pixel.

#[cfg(test)]
mod coverage;
mod glyph;

use fontdue::{Font, FontSettings};
use glyph::{
    blend_glyph, evict_if_full, shifted_glyph, split_position, CachedGlyph, GlyphKey,
    PHASE_ORIGIN, SUBPIXEL_PHASES,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::{Color, PixmapMut};

pub use glyph::GlyphEntry;

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

/// La police d'interface, graisse normale (FONT-1).
pub(crate) const REGULAR_FONT_BYTES: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");
/// La police d'interface, graisse forte (FONT-1).
pub(crate) const BOLD_FONT_BYTES: &[u8] = include_bytes!("../assets/Inter-SemiBold.ttf");

pub struct Typography {
    pub regular: Font,
    pub bold: Font,
    glyph_cache: RefCell<HashMap<GlyphKey, CachedGlyph>>,
    access_counter: Cell<u64>,
}

impl Typography {
    pub fn new() -> Self {
        let regular = Font::from_bytes(REGULAR_FONT_BYTES, FontSettings::default())
            .expect("police intégrée par include_bytes! — corruption du binaire si ceci échoue");
        let bold = Font::from_bytes(BOLD_FONT_BYTES, FontSettings::default())
            .expect("police intégrée par include_bytes! — corruption du binaire si ceci échoue");
        Self {
            regular,
            bold,
            glyph_cache: RefCell::new(HashMap::with_capacity(512)),
            access_counter: Cell::new(0),
        }
    }

    /// Récupère ou rastérise un glyphe **non décalé** (R-26, R-40).
    pub fn get_glyph(&self, ch: char, size: f32, bold: bool) -> Rc<GlyphEntry> {
        let size = clamp_font_size(size);
        self.glyph_variant(ch, size, bold, PHASE_ORIGIN)
    }

    /// Récupère la variante de `ch` décalée de `phase` (GLYPH-1).
    ///
    /// Une variante décalée se dérive de la variante d'origine plutôt que d'une nouvelle
    /// rastérisation : `fontdue` n'est donc appelé qu'**une fois par (caractère, taille,
    /// graisse)**, quel que soit le nombre de phases. La lecture de l'origine met à jour
    /// son horodatage, ce qui la garde en cache tant que l'une de ses phases sert.
    fn glyph_variant(&self, ch: char, size: f32, bold: bool, phase: u8) -> Rc<GlyphEntry> {
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let access = self.access_counter.get().wrapping_add(1);
        self.access_counter.set(access);

        let mut cache = self.glyph_cache.borrow_mut();
        if let Some((entry, last_access)) = cache.get_mut(&(bold, ch, size_key, phase)) {
            *last_access = access;
            return entry.clone();
        }

        let origin = match cache.get_mut(&(bold, ch, size_key, PHASE_ORIGIN)) {
            Some((entry, last_access)) => {
                *last_access = access;
                entry.clone()
            }
            None => {
                let font = if bold { &self.bold } else { &self.regular };
                let (metrics, bitmap) = font.rasterize(ch, size);
                let (width, height) = (metrics.width, metrics.height);
                let entry = Rc::new(GlyphEntry { metrics, bitmap, width, height });
                evict_if_full(&mut cache);
                cache.insert((bold, ch, size_key, PHASE_ORIGIN), (entry.clone(), access));
                entry
            }
        };
        if phase == PHASE_ORIGIN {
            return origin;
        }

        let shifted = Rc::new(shifted_glyph(&origin, phase));
        evict_if_full(&mut cache);
        cache.insert((bold, ch, size_key, phase), (shifted.clone(), access));
        shifted
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

        // La partie fractionnaire verticale est la même pour toute la ligne : `ymin` et
        // `height` sont entiers, donc seule l'ordonnée de base porte une phase (GLYPH-1).
        let (cell_y, phase_y) = split_position(y + size);

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let (cell_x, phase_x) = split_position(x);
            let entry = self.glyph_variant(ch, size, bold, phase_y * SUBPIXEL_PHASES + phase_x);
            let metrics = &entry.metrics;

            let gx = cell_x + metrics.xmin;
            let gy = cell_y - metrics.ymin - metrics.height as i32;
            blend_glyph(data, (w, h), &entry, (gx, gy), (r, g, b, a));
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
            let advance = {
                // 1. Passe contour : chaque décalage a sa propre phase, comme n'importe
                //    quelle position. Les huit offsets n'en produisent que quatre distinctes.
                for &(ox, oy) in &shadow_offsets {
                    self.blend_positioned(
                        data,
                        (w, h),
                        (ch, size, bold),
                        (x + ox, y + size + oy),
                        (out_r, out_g, out_b, out_a),
                    );
                }
                // 2. Passe principale
                self.blend_positioned(
                    data,
                    (w, h),
                    (ch, size, bold),
                    (x, y + size),
                    (r, g, b, a),
                )
            };
            x += advance;
        }
        x
    }

    /// Compose un glyphe à une position fractionnaire et rend son avance.
    fn blend_positioned(
        &self,
        data: &mut [u8],
        bounds: (i32, i32),
        glyph: (char, f32, bool),
        at: (f32, f32),
        color: (f32, f32, f32, f32),
    ) -> f32 {
        let (ch, size, bold) = glyph;
        let (cell_x, phase_x) = split_position(at.0);
        let (cell_y, phase_y) = split_position(at.1);
        let entry = self.glyph_variant(ch, size, bold, phase_y * SUBPIXEL_PHASES + phase_x);
        let gx = cell_x + entry.metrics.xmin;
        let gy = cell_y - entry.metrics.ymin - entry.metrics.height as i32;
        blend_glyph(data, bounds, &entry, (gx, gy), color);
        entry.metrics.advance_width
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
            width += self.get_glyph(ch, size, bold).metrics.advance_width;
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

#[cfg(test)]
mod tests {
    use super::glyph::GLYPH_CACHE_CAPACITY;
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

    /// Abscisse du centre de gravité de l'encre, pondérée par la couverture.
    ///
    /// C'est une mesure **sous-pixel** : elle voit un déplacement d'un quart de pixel là où
    /// une boîte englobante ne verrait rien.
    fn ink_centroid_x(pixmap: &Pixmap) -> f32 {
        let w = pixmap.width() as usize;
        let (mut weighted, mut total) = (0.0_f64, 0.0_f64);
        let (chunks, _) = pixmap.data().as_chunks::<4>();
        for (i, px) in chunks.iter().enumerate() {
            let alpha = px[3] as f64;
            weighted += alpha * (i % w) as f64;
            total += alpha;
        }
        assert!(total > 0.0, "aucune encre a mesurer");
        (weighted / total) as f32
    }

    fn draw_probe(typo: &Typography, x: f32) -> Pixmap {
        let mut pixmap = Pixmap::new(120, 60).expect("pixmap");
        let color = Color::from_rgba8(255, 255, 255, 255);
        typo.draw_text(&mut pixmap.as_mut(), "H", x, 10.0, TextStyle { size: 20.0, color, bold: false });
        pixmap
    }

    #[test]
    fn test_glyph_1_a_fraction_of_a_pixel_moves_the_glyph_by_that_fraction() {
        // Le cœur de R-46. Avec la troncature d'origine, les cinq décalages ci-dessous
        // donnaient tous exactement le MÊME centre de gravité : le glyphe ne bougeait qu'au
        // franchissement de l'entier, d'où le tremblement et l'espacement irrégulier.
        let typo = Typography::new();
        let reference = ink_centroid_x(&draw_probe(&typo, 30.0));
        for offset in [0.25_f32, 0.5, 0.75, 0.3, 0.9] {
            let observed = ink_centroid_x(&draw_probe(&typo, 30.0 + offset)) - reference;
            // La phase arrondit au huitième de pixel ; l'arrondi de la couverture sur 8 bits
            // ajoute un bruit du même ordre.
            assert!(
                (observed - offset).abs() < 0.2,
                "decalage demande {offset}, obtenu {observed}"
            );
        }
    }

    #[test]
    fn test_glyph_1_letter_spacing_does_not_drift_along_a_word() {
        // L'avance de la plume est fractionnaire : au bout de dix lettres, une troncature
        // par glyphe avait accumule jusqu'a un pixel d'ecart entre deux paires voisines.
        let typo = Typography::new();
        let color = Color::from_rgba8(255, 255, 255, 255);
        let style = TextStyle { size: 18.0, color, bold: false };
        let advance = typo.measure_text("i", 18.0, false).0;

        let mut centres = Vec::new();
        for n in 0..10 {
            let mut pixmap = Pixmap::new(400, 60).expect("pixmap");
            // Une seule lettre par image, posee la ou la plume l'aurait laissee.
            typo.draw_text(&mut pixmap.as_mut(), "i", 20.0 + advance * n as f32, 10.0, style);
            centres.push(ink_centroid_x(&pixmap));
        }
        // Chaque lettre est mesuree par rapport a la premiere, pas a sa voisine : l'erreur
        // d'une phase (un huitieme de pixel) ne s'accumule pas, alors qu'une troncature
        // par glyphe derivait jusqu'a un pixel entier au bout du mot. Comparer deux
        // voisines cumulerait les deux arrondis et ne verrait rien quand l'avance est
        // proche d'un entier.
        for (n, centre) in centres.iter().enumerate() {
            let observed = centre - centres[0];
            let expected = advance * n as f32;
            assert!(
                (observed - expected).abs() < 0.2,
                "lettre {n} a {observed} au lieu de {expected} — l'espacement derive"
            );
        }
    }

    #[test]
    fn test_glyph_1_the_phase_belongs_to_the_cache_key() {
        // Servir le bitmap d'une phase pour une autre annulerait tout le correctif.
        let typo = Typography::new();
        let origin = typo.get_glyph('A', 16.0, false);
        let shifted = typo.glyph_variant('A', 16.0, false, 2);
        assert_ne!(origin.bitmap, shifted.bitmap, "deux phases doivent differer");
        assert_eq!(typo.cached_glyph_count(), 2, "les deux variantes coexistent en cache");
        // Redemander la meme phase ne rastérise rien de neuf.
        let again = typo.glyph_variant('A', 16.0, false, 2);
        assert_eq!(again.bitmap, shifted.bitmap);
        assert_eq!(typo.cached_glyph_count(), 2);
    }

    #[test]
    fn test_glyph_1_the_cache_stays_bounded_across_every_phase() {
        // Un déplacement continu traverse les seize phases de chaque glyphe : le cache doit
        // rester borné, sinon le correctif de R-46 achète la fidélité avec une fuite.
        let typo = Typography::new();
        let color = Color::from_rgba8(255, 255, 255, 255);
        let mut pixmap = Pixmap::new(600, 40).expect("pixmap");
        for step in 0..600 {
            let offset = step as f32 * 0.07;
            for size in [11.0_f32, 13.0, 14.0, 16.0, 18.0] {
                typo.draw_text(
                    &mut pixmap.as_mut(),
                    "Glucose 0123 — fidelite",
                    10.0 + offset,
                    5.0 + offset,
                    TextStyle { size, color, bold: false },
                );
            }
        }
        let count = typo.cached_glyph_count();
        assert!(count <= GLYPH_CACHE_CAPACITY, "cache de {count} variantes");
        println!("[R-46] variantes en cache apres un balayage complet : {count}");
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
