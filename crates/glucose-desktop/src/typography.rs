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
//! La police est désormais **Inter** (SIL Open Font License 1.1, texte dans
//! `assets/LICENSE-Inter.txt`) : 2 505 points de code, accents français complets, ligatures,
//! guillemets, tirets, flèches. Il n'y a plus de table de repli : un caractère absent de la
//! police se dessine en `.notdef`, visiblement, plutôt que d'être remplacé en silence par un
//! autre. Et `typography/coverage.rs` lit la table `cmap` de chaque police embarquée pour
//! affirmer la présence d'un ensemble nommé de caractères, dont **tous les caractères
//! non-ASCII des chaînes littérales de ce crate** : changer de police sans couvrir le
//! français casse la build, au lieu de casser l'écran.
//!
//! Les visages sont au nombre de cinq ([`Face`]) : quatre Inter — droit, gras, italique,
//! gras italique — et une chasse fixe pour le code. C'est le Markdown en ligne qui les
//! demande, et c'est pourquoi aucun `bool` ne traverse plus ce module.
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
pub mod glyph;

use fontdue::{Font, FontSettings};
use glyph::{
    blend_glyph, evict_if_full, shifted_glyph, split_position, CachedGlyph, GlyphKey, PHASE_ORIGIN,
    SUBPIXEL_PHASES,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::{Color, PixmapMut};

pub use glyph::GlyphEntry;

/// Le visage sous lequel un caractère se dessine.
///
/// Un `bool` ne peut plus dire ce que le Markdown demande : `*terme*` veut une italique
/// dessinée, `` `code` `` une chasse fixe, `***les deux***` un quatrième fichier. Les visages
/// sont donc nommés, et c'est ce nom qui traverse la mesure, le cache et le tracé — un
/// appelant ne peut plus écrire `true` sans dire de quoi.
///
/// # FONT-2 — les cinq visages sont la même famille, et leurs métriques le prouvent
///
/// Les quatre Inter viennent d'une seule version (3.19) : mêmes montantes, mêmes descentes,
/// mêmes avances. JetBrains Mono s'y ajoute avec **la même hauteur d'œil et la même hauteur
/// de capitale à taille égale** — mesuré, pas supposé — ce qui est la raison pour laquelle
/// le code en ligne garde la taille du corps au lieu du `0.875em` que les feuilles de style
/// du Web appliquent pour rattraper une monospace système trop grande.
/// `coverage::test_every_face_shares_the_metrics_of_its_family` tient cet accord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Face {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
    /// Chasse fixe, pour `` `code` `` et les blocs de code.
    Mono,
}

impl Face {
    /// Les cinq visages, dans l'ordre de [`Typography::faces`].
    pub const ALL: [Self; 5] = [
        Self::Regular,
        Self::Bold,
        Self::Italic,
        Self::BoldItalic,
        Self::Mono,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

/// Style d'un tracé de texte.
///
/// Regroupe les trois paramètres de style pour qu'aucun d'eux — `size` en
/// particulier — ne puisse être confondu avec une coordonnée (R-44).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    pub face: Face,
}

/// Les cinq polices embarquées, dans l'ordre de [`Face::ALL`] (FONT-1, FONT-2).
///
/// Inter 3.19 sous SIL Open Font License 1.1 (`assets/LICENSE-Inter.txt`), JetBrains Mono
/// 2.304 sous la même licence (`assets/LICENSE-JetBrainsMono.txt`). La variante « NL » de
/// JetBrains Mono est celle **sans ligatures** : le moteur ne fait aucune substitution de
/// glyphes, donc les ligatures seraient du poids mort jamais dessiné.
pub(crate) const FACE_BYTES: [&[u8]; 5] = [
    include_bytes!("../assets/Inter-Regular.ttf"),
    include_bytes!("../assets/Inter-SemiBold.ttf"),
    include_bytes!("../assets/Inter-Italic.otf"),
    include_bytes!("../assets/Inter-SemiBoldItalic.otf"),
    include_bytes!("../assets/JetBrainsMonoNL-Regular.ttf"),
];

pub struct Typography {
    faces: [Font; 5],
    glyph_cache: RefCell<HashMap<GlyphKey, CachedGlyph>>,
    access_counter: Cell<u64>,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for Typography {
    fn default() -> Self {
        Self::new()
    }
}

impl Typography {
    pub fn new() -> Self {
        let faces = FACE_BYTES.map(|bytes| {
            Font::from_bytes(bytes, FontSettings::default())
                .expect("police intégrée par include_bytes! — corruption du binaire si ceci échoue")
        });
        Self {
            faces,
            glyph_cache: RefCell::new(HashMap::with_capacity(512)),
            access_counter: Cell::new(0),
        }
    }

    /// La police d'un visage.
    pub fn font(&self, face: Face) -> &Font {
        &self.faces[face.index()]
    }

    /// Récupère ou rastérise un glyphe **non décalé** (R-26, R-40).
    pub fn get_glyph(&self, ch: char, size: f32, face: Face) -> Rc<GlyphEntry> {
        let size = clamp_font_size(size);
        self.glyph_variant(ch, size, face, PHASE_ORIGIN)
    }

    /// De combien la plume avance après `ch` — **sans toucher au cache de glyphes**.
    ///
    /// # MEASURE-1 — mesurer n'est pas dessiner
    ///
    /// Le reflux d'un paragraphe demande une largeur par caractère, et rien d'autre. Passer
    /// par [`Self::get_glyph`] pour l'obtenir fait payer un hachage, un emprunt de `RefCell`,
    /// un `Rc::clone` et une écriture du compteur LRU — pour lire un seul `f32` déjà présent
    /// dans les tables de la police. Mesuré sur 89 caractères : **3 415 ns par le cache contre
    /// 1 844 ns en direct**, soit presque le double pour le service le plus fréquent du moteur,
    /// puisqu'il tourne à chaque frame sur chaque carte visible.
    ///
    /// Le cache garde ce qui coûte vraiment — les **bitmaps**, que seul le tracé consomme.
    /// `test_measuring_agrees_with_drawing` tient l'égalité des deux chemins : s'ils
    /// divergeaient, le curseur tomberait à côté du texte.
    pub fn advance(&self, ch: char, size: f32, face: Face) -> f32 {
        self.font(face)
            .metrics(ch, clamp_font_size(size))
            .advance_width
    }

    /// Récupère la variante de `ch` décalée de `phase` (GLYPH-1).
    ///
    /// Une variante décalée se dérive de la variante d'origine plutôt que d'une nouvelle
    /// rastérisation : `fontdue` n'est donc appelé qu'**une fois par (caractère, taille,
    /// graisse)**, quel que soit le nombre de phases. La lecture de l'origine met à jour
    /// son horodatage, ce qui la garde en cache tant que l'une de ses phases sert.
    fn glyph_variant(&self, ch: char, size: f32, face: Face, phase: u8) -> Rc<GlyphEntry> {
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let access = self.access_counter.get().wrapping_add(1);
        self.access_counter.set(access);

        let mut cache = self.glyph_cache.borrow_mut();
        if let Some((entry, last_access)) = cache.get_mut(&(face, ch, size_key, phase)) {
            *last_access = access;
            return entry.clone();
        }

        let origin = match cache.get_mut(&(face, ch, size_key, PHASE_ORIGIN)) {
            Some((entry, last_access)) => {
                *last_access = access;
                entry.clone()
            }
            None => {
                let (metrics, bitmap) = self.font(face).rasterize(ch, size);
                let (width, height) = (metrics.width, metrics.height);
                let entry = Rc::new(GlyphEntry {
                    metrics,
                    bitmap,
                    width,
                    height,
                });
                evict_if_full(&mut cache);
                cache.insert((face, ch, size_key, PHASE_ORIGIN), (entry.clone(), access));
                entry
            }
        };
        if phase == PHASE_ORIGIN {
            return origin;
        }

        let shifted = Rc::new(shifted_glyph(&origin, phase));
        evict_if_full(&mut cache);
        cache.insert((face, ch, size_key, phase), (shifted.clone(), access));
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
        let TextStyle { color, face, .. } = style;
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
            let entry = self.glyph_variant(ch, size, face, phase_y * SUBPIXEL_PHASES + phase_x);
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
        let TextStyle { color, face, .. } = style;
        let size = clamp_font_size(style.size);
        let shadow_offsets: [(f32, f32); 8] = [
            (-1.5, 0.0),
            (1.5, 0.0),
            (0.0, -1.5),
            (0.0, 1.5),
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
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
                        (ch, size, face),
                        (x + ox, y + size + oy),
                        (out_r, out_g, out_b, out_a),
                    );
                }
                // 2. Passe principale
                self.blend_positioned(data, (w, h), (ch, size, face), (x, y + size), (r, g, b, a))
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
        glyph: (char, f32, Face),
        at: (f32, f32),
        color: (f32, f32, f32, f32),
    ) -> f32 {
        let (ch, size, face) = glyph;
        let (cell_x, phase_x) = split_position(at.0);
        let (cell_y, phase_y) = split_position(at.1);
        let entry = self.glyph_variant(ch, size, face, phase_y * SUBPIXEL_PHASES + phase_x);
        let gx = cell_x + entry.metrics.xmin;
        let gy = cell_y - entry.metrics.ymin - entry.metrics.height as i32;
        blend_glyph(data, bounds, &entry, (gx, gy), color);
        entry.metrics.advance_width
    }

    /// Mesure la largeur et hauteur d'un texte via le cache de glyphes et métriques de police réelles (R-40)
    pub fn measure_text(&self, text: &str, size: f32, face: Face) -> (f32, f32) {
        let size = clamp_font_size(size);
        let height = self
            .font(face)
            .horizontal_line_metrics(size)
            .map(|m| m.new_line_size)
            .unwrap_or(size * 1.2);
        let mut width = 0.0;
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            width += self.advance(ch, size, face);
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
        typo.draw_text(
            &mut pixmap.as_mut(),
            "Hello",
            10.0,
            20.0,
            TextStyle {
                size: 14.0,
                color,
                face: Face::Regular,
            },
        );
        let count_after_first = typo.cached_glyph_count();
        assert!(count_after_first > 0);

        // Réutiliser le texte ne doit pas augmenter le nombre de glyphes rastérisés
        typo.draw_text(
            &mut pixmap.as_mut(),
            "Hello",
            10.0,
            50.0,
            TextStyle {
                size: 14.0,
                color,
                face: Face::Regular,
            },
        );
        assert_eq!(typo.cached_glyph_count(), count_after_first);
    }

    #[test]
    fn test_alpha_blending_preserves_destination_alpha() {
        let typo = Typography::new();
        let mut pixmap = Pixmap::new(100, 50).unwrap();
        let color = Color::from_rgba8(255, 255, 255, 128);
        typo.draw_text(
            &mut pixmap.as_mut(),
            "A",
            10.0,
            10.0,
            TextStyle {
                size: 16.0,
                color,
                face: Face::Regular,
            },
        );

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
            TextStyle {
                size: 16.0,
                color: text_color,
                face: Face::Bold,
            },
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
        typo.draw_text(
            &mut pixmap.as_mut(),
            "H",
            x,
            10.0,
            TextStyle {
                size: 20.0,
                color,
                face: Face::Regular,
            },
        );
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
        let style = TextStyle {
            size: 18.0,
            color,
            face: Face::Regular,
        };
        let advance = typo.measure_text("i", 18.0, Face::Regular).0;

        let mut centres = Vec::new();
        for n in 0..10 {
            let mut pixmap = Pixmap::new(400, 60).expect("pixmap");
            // Une seule lettre par image, posee la ou la plume l'aurait laissee.
            typo.draw_text(
                &mut pixmap.as_mut(),
                "i",
                20.0 + advance * n as f32,
                10.0,
                style,
            );
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
        let origin = typo.get_glyph('A', 16.0, Face::Regular);
        let shifted = typo.glyph_variant('A', 16.0, Face::Regular, 2);
        assert_ne!(
            origin.bitmap, shifted.bitmap,
            "deux phases doivent differer"
        );
        assert_eq!(
            typo.cached_glyph_count(),
            2,
            "les deux variantes coexistent en cache"
        );
        // Redemander la meme phase ne rastérise rien de neuf.
        let again = typo.glyph_variant('A', 16.0, Face::Regular, 2);
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
                    TextStyle {
                        size,
                        color,
                        face: Face::Regular,
                    },
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
            typo.get_glyph('A', size, Face::Regular);
        }
        assert_eq!(typo.cached_glyph_count(), 4096);

        // Ré-accéder fréquemment à un glyphe particulier pour qu'il ait un timestamp récent
        typo.get_glyph('A', 10.0, Face::Regular);

        // Insérer un 4097e glyphe pour déclencher l'éviction LRU
        typo.get_glyph('B', 12.0, Face::Regular);

        // L'éviction des 25% les plus anciens ramène la taille à ~3072 + 1
        let count = typo.cached_glyph_count();
        assert!(count < 4096);
        assert!(count >= 3072);

        // Le glyphe fréquemment ré-accédé doit toujours être présent en cache sans réallocation
        let pre_count = typo.cached_glyph_count();
        typo.get_glyph('A', 10.0, Face::Regular);
        assert_eq!(typo.cached_glyph_count(), pre_count);
    }
}
