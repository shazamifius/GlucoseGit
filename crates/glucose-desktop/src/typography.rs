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
//!
//! # EMOJI-1 — un emoji se dessine, par la police de repli (fiche 58)
//!
//! Inter n'a aucun emoji : celui que Gboard propose à chaque phrase s'affichait comme une
//! case vide. **Noto Emoji**, monochrome (`decisions/10`), sert de repli, et seulement de
//! repli : un caractère que le visage demandé dessine reste à lui ; un caractère qu'il n'a pas,
//! et que Noto Emoji a, se dessine en Noto Emoji ([`Typography::police`]). Aucune table écrite
//! à la main : la `cmap` des deux polices décide, caractère par caractère. La mesure et le
//! dessin passent par le même choix, et ne peuvent donc pas diverger.

#[cfg(test)]
pub(crate) mod coverage;
mod detoure;
mod encre;
pub use encre::{forme_grise, peindre_exact, LETTRE_LISIBLE};
pub mod glyph;
pub(crate) mod trace;

pub use detoure::DETOURAGE;

use fontdue::{Font, FontSettings};
use glyph::{
    blend_glyph, shifted_glyph, split_position, CachedGlyph, GlyphKey, PHASE_ORIGIN,
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

/// **La police de repli des emojis** (EMOJI-1) : Noto Emoji 3.002, monochrome, instance fixe
/// de graisse 400, sous SIL Open Font License 1.1 (`assets/LICENSE-NotoEmoji.txt`,
/// `decisions/10`).
const EMOJI_BYTES: &[u8] = include_bytes!("../assets/NotoEmoji-Regular.ttf");

/// Le nom de chaque police embarquée, dans l'ordre de [`FACE_BYTES`].
const FACE_NOMS: [&str; 5] = [
    "Inter-Regular",
    "Inter-SemiBold",
    "Inter-Italic",
    "Inter-SemiBoldItalic",
    "JetBrainsMonoNL-Regular",
];

/// **Une police embarquée, nommée** : ce que le peintre vectoriel lit pour écrire les contours de
/// ses glyphes (P5, fiche 70). Le nom distingue deux polices, les octets donnent leurs tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Police {
    pub nom: &'static str,
    pub octets: &'static [u8],
}

/// **Toutes les polices embarquées** : les cinq visages, puis le repli des emojis — tout ce
/// qu'une lettre de Glucose peut dessiner.
pub(crate) fn toutes_les_polices() -> [Police; 6] {
    let mut polices = [Police {
        nom: "NotoEmoji-Regular",
        octets: EMOJI_BYTES,
    }; 6];
    for (k, police) in polices.iter_mut().take(5).enumerate() {
        *police = Police {
            nom: FACE_NOMS[k],
            octets: FACE_BYTES[k],
        };
    }
    polices
}

pub struct Typography {
    faces: [Font; 5],
    /// Le repli des emojis (EMOJI-1).
    emoji: Font,
    glyph_cache: RefCell<HashMap<GlyphKey, CachedGlyph>>,
    /// **L'image en cours** : chaque lettre retient la dernière qui l'a demandée, et une image
    /// qui commence oublie celles que la précédente n'a pas demandées
    /// ([`Typography::nouvelle_image`]).
    image: Cell<u64>,
    /// Les contours des mêmes polices, dans le même ordre, le repli des emojis en dernier :
    /// une grande lettre se trace (GRAND-1, fiche 72).
    contours: trace::Contours,
    /// L'encre de chaque lettre, lue une fois (GRIS-1, fiche 95) : [`Typography::encre`].
    encres: RefCell<HashMap<(Face, char), f32>>,
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
        let emoji = Font::from_bytes(EMOJI_BYTES, FontSettings::default())
            .expect("police intégrée par include_bytes! — corruption du binaire si ceci échoue");
        let mut octets = FACE_BYTES.to_vec();
        octets.push(EMOJI_BYTES);
        Self {
            faces,
            emoji,
            glyph_cache: RefCell::new(HashMap::with_capacity(512)),
            image: Cell::new(0),
            contours: trace::Contours::de(&octets),
            encres: RefCell::new(HashMap::new()),
        }
    }

    /// **Trace ce caractère par son contour** (GRAND-1) — la police que le choix d'EMOJI-1 lui
    /// donne, numérotée comme [`Self::contours`] —, et rend son avance.
    fn tracer(
        &self,
        pixmap: &mut PixmapMut,
        (ch, size, face): (char, f32, Face),
        plume: (f32, f32),
        couleur: Color,
    ) -> f32 {
        let (choisi, indice) = self.choix(ch, face);
        let police = choisi.map_or(Face::ALL.len(), Face::index);
        self.contours
            .remplir(pixmap, (police, indice), plume, size, couleur);
        self.advance(ch, size, face)
    }

    /// La police d'un visage.
    pub fn font(&self, face: Face) -> &Font {
        &self.faces[face.index()]
    }

    /// **La police qui dessine `ch` dans ce visage**, et l'indice de son glyphe (EMOJI-1) : le
    /// visage s'il l'a, sinon Noto Emoji s'il l'a, sinon le `.notdef` du visage — un caractère
    /// que personne ne dessine se voit, plutôt que de disparaître.
    fn police(&self, ch: char, face: Face) -> (&Font, u16) {
        match self.choix(ch, face) {
            (Some(face), indice) => (self.font(face), indice),
            (None, indice) => (&self.emoji, indice),
        }
    }

    /// **La police nommée qui dessine `ch` dans ce visage**, et l'indice de son glyphe — le
    /// même choix que le tracé ([`Self::police`]) : le SVG écrit le glyphe même que le canevas
    /// pose (P5, fiche 70).
    pub fn source(&self, ch: char, face: Face) -> (Police, u16) {
        let (choisi, indice) = self.choix(ch, face);
        let rang = choisi.map_or(Face::ALL.len(), Face::index);
        (toutes_les_polices()[rang], indice)
    }

    /// Le choix d'EMOJI-1 : le visage (`Some`) ou le repli des emojis (`None`), et l'indice.
    fn choix(&self, ch: char, face: Face) -> (Option<Face>, u16) {
        let indice = self.font(face).lookup_glyph_index(ch);
        if indice != 0 {
            return (Some(face), indice);
        }
        match self.emoji.lookup_glyph_index(ch) {
            0 => (Some(face), 0),
            repli => (None, repli),
        }
    }

    /// Récupère ou rastérise l'image d'un glyphe **non décalé** (R-26, R-40) — `None` au-delà
    /// d'un demi-carré : à cette taille, une lettre se trace et n'a pas d'image (GRAND-1).
    pub fn get_glyph(&self, ch: char, size: f32, face: Face) -> Option<Rc<GlyphEntry>> {
        let size = clamp_font_size(size);
        (!trace::se_trace(size)).then(|| self.glyph_variant(ch, size, face, PHASE_ORIGIN))
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
        let (font, indice) = self.police(ch, face);
        font.metrics_indexed(indice, clamp_font_size(size))
            .advance_width
    }

    /// Récupère la variante de `ch` décalée de `phase` (GLYPH-1).
    ///
    /// Une variante décalée se dérive de la variante d'origine plutôt que d'une nouvelle
    /// rastérisation : `fontdue` n'est donc appelé qu'**une fois par (caractère, taille,
    /// graisse)**, quel que soit le nombre de phases. La lecture de l'origine la marque
    /// de l'image en cours, ce qui la garde en cache tant que l'une de ses phases sert.
    fn glyph_variant(&self, ch: char, size: f32, face: Face, phase: u8) -> Rc<GlyphEntry> {
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let image = self.image.get();
        let mut cache = self.glyph_cache.borrow_mut();
        if let Some((entry, vue)) = cache.get_mut(&(face, ch, size_key, phase)) {
            *vue = image;
            return entry.clone();
        }

        let origin = match cache.get_mut(&(face, ch, size_key, PHASE_ORIGIN)) {
            Some((entry, vue)) => {
                *vue = image;
                entry.clone()
            }
            None => {
                let (font, indice) = self.police(ch, face);
                let (metrics, bitmap) = font.rasterize_indexed(indice, size);
                let (width, height) = (metrics.width, metrics.height);
                let entry = Rc::new(GlyphEntry {
                    metrics,
                    bitmap,
                    width,
                    height,
                });
                cache.insert((face, ch, size_key, PHASE_ORIGIN), (entry.clone(), image));
                entry
            }
        };
        if phase == PHASE_ORIGIN {
            return origin;
        }

        let shifted = Rc::new(shifted_glyph(&origin, phase));
        cache.insert((face, ch, size_key, phase), (shifted.clone(), image));
        shifted
    }

    /// **Une image commence** : les lettres que la précédente n'a pas demandées s'oublient — la
    /// loi des mises en page (P8 bis) et des textures, sans capacité choisie (fiche 104).
    ///
    /// # Pourquoi plus de plafond
    ///
    /// Le cache gardait 4 096 variantes et évinçait le quart le plus ancien quand il était plein
    /// (R-40) : mesuré à 531 pour une image d'un document ordinaire, cinq fois la marge. Puis
    /// chaque titre de membrane a pris la taille de sa région (TITRE-1) : sur la carte de
    /// Wikipédia, une image vue de loin demande **5 654** variantes, halos compris. Un cache plus
    /// petit que ce qu'une image demande ne garde rien — chaque image chassait ce que la suivante
    /// redemandait : 7 727 lettres refaites par image, les titres à 15 ms sur 25 (le vrai Glucose
    /// au pilote, la souris seule sur la carte). Sans éviction du tout, le même cache passe de 10 000
    /// à 42 000 variantes après trois zooms : chaque image d'un zoom pose des tailles neuves.
    ///
    /// Ce qu'une image demande, la suivante le redemande presque toujours (la vue a peu bougé) ; ce
    /// qu'elle n'a pas demandé ne se voit plus. Le cache tient donc exactement une image : une vue
    /// immobile ne refait aucune lettre, un zoom ne garde pas ses tailles d'hier.
    pub fn nouvelle_image(&self) {
        let image = self.image.get();
        self.glyph_cache
            .borrow_mut()
            .retain(|_, (_, vue)| *vue == image);
        self.image.set(image + 1);
    }

    /// Nombre de glyphes actuellement en cache
    pub fn cached_glyph_count(&self) -> usize {
        self.glyph_cache.borrow().len()
    }

    /// Les octets que les bitmaps du cache occupent.
    pub fn octets_en_cache(&self) -> usize {
        self.glyph_cache
            .borrow()
            .values()
            .map(|(g, _)| g.bitmap.capacity())
            .sum()
    }

    /// Dessine une chaîne de caractères vectorielle avec antialiasing et cache de glyphes (R-26, R-27)
    pub fn draw_text(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        x: f32,
        y: f32,
        style: TextStyle,
    ) -> f32 {
        self.draw_text_offset(pixmap, text, x, y, (0, 0), style)
    }

    /// Écrit un texte dont la **place** est décalée d'un nombre entier de pixels (GLYPH-1).
    ///
    /// # Pourquoi le décalage ne peut pas se faire sur les coordonnées
    ///
    /// Un panneau mis en cache se dessine dans un tampon à lui, donc son texte doit être
    /// déplacé de l'origine de ce tampon. Le faire en soustrayant cette origine de `x` avant
    /// d'appeler le tracé **paraît** équivalent — l'origine est entière, et une phase
    /// sous-pixel ne dépend que de la partie fractionnaire.
    ///
    /// Elle ne l'est pas. `x - origine` est une soustraction en virgule flottante : son
    /// résultat n'est pas toujours représentable, et la fraction ressort parfois décalée du
    /// dernier bit. Quand `x` tombe près d'une frontière de phase, ce bit suffit à basculer le
    /// glyphe sur la variante voisine. Mesuré sur le panneau PRESETS : trente-six pixels d'un
    /// libellé, jusqu'à 23/255 d'écart avec le rendu direct.
    ///
    /// Le décalage porte donc sur la **cellule entière**, après que la phase a été lue sur la
    /// position d'écran. La phase est alors exactement celle du rendu direct, et la
    /// translation est exacte puisqu'elle n'opère que sur des entiers.
    pub fn draw_text_offset(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        mut x: f32,
        y: f32,
        offset: (i32, i32),
        style: TextStyle,
    ) -> f32 {
        // GRIS-2 : une lettre qui ne se lit plus montre le gris de son encre.
        if style.size < encre::LETTRE_LISIBLE {
            let coin = (x + offset.0 as f32, y + offset.1 as f32);
            return self.griser(pixmap, text, coin, style) - offset.0 as f32;
        }
        let TextStyle { color, face, .. } = style;
        let size = clamp_font_size(style.size);
        let r = color.red() * 255.0;
        let g = color.green() * 255.0;
        let b = color.blue() * 255.0;
        let a = color.alpha();

        let w = pixmap.width() as i32;
        let h = pixmap.height() as i32;

        // La partie fractionnaire verticale est la même pour toute la ligne : `ymin` et
        // `height` sont entiers, donc seule l'ordonnée de base porte une phase (GLYPH-1).
        let (cell_y, phase_y) = split_position(y + size);
        let cell_y = cell_y + offset.1;

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            if trace::se_trace(size) {
                let plume = (x + offset.0 as f32, y + size + offset.1 as f32);
                x += self.tracer(pixmap, (ch, size, face), plume, color);
                continue;
            }
            let data = pixmap.data_mut();
            let (cell_x, phase_x) = split_position(x);
            let entry = self.glyph_variant(ch, size, face, phase_y * SUBPIXEL_PHASES + phase_x);
            let metrics = &entry.metrics;

            let gx = cell_x + offset.0 + metrics.xmin;
            let gy = cell_y - metrics.ymin - metrics.height as i32;
            blend_glyph(data, (w, h), &entry, (gx, gy), (r, g, b, a));
            x += metrics.advance_width;
        }
        x
    }

    /// Compose un glyphe à une position fractionnaire et rend son avance — tracé s'il est plus
    /// grand qu'un demi-carré (GRAND-1).
    fn blend_positioned(
        &self,
        pixmap: &mut PixmapMut,
        glyph: (char, f32, Face),
        at: (f32, f32),
        (couleur, color): (Color, (f32, f32, f32, f32)),
    ) -> f32 {
        let (ch, size, face) = glyph;
        if trace::se_trace(size) {
            return self.tracer(pixmap, glyph, at, couleur);
        }
        let bounds = (pixmap.width() as i32, pixmap.height() as i32);
        let data = pixmap.data_mut();
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

/// Normalise une taille de police : `NaN` devient 1, et rien ne descend sous un pixel.
///
/// # Ce qui a disparu, et pourquoi (GRAND-1, fiche 72)
///
/// Une borne haute, `MAX_FONT_SIZE` (512 px), protégeait le rastériseur : une lettre se
/// rastérisait entière, et une taille aberrante (échelle corrompue, argument inversé) aurait
/// coûté des centaines de millions de pixels. Elle **réduisait** donc les lettres de près —
/// au-delà de ×32, un texte se dessinait plus petit que sa boîte. Depuis GRAND-1, une lettre
/// plus grande qu'un carré se trace par son contour, dont le coût est borné par le tampon où
/// l'on dessine, quelle que soit sa taille : la garde n'a plus rien à garder, elle disparaît.
pub fn clamp_font_size(size: f32) -> f32 {
    if size.is_nan() {
        return 1.0;
    }
    size.max(1.0)
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
        let origin = typo.get_glyph('A', 16.0, Face::Regular).expect("une image");
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

    /// **Un balayage ne fait pas grandir le cache** : un déplacement continu traverse les seize
    /// phases de chaque glyphe, une image par pas ; le cache ne garde que ce que la dernière image a
    /// demandé — sinon le positionnement sous-pixel (R-46) achèterait la fidélité avec une fuite.
    #[test]
    fn test_glyph_1_the_cache_stays_bounded_across_every_phase() {
        let typo = Typography::new();
        let color = Color::from_rgba8(255, 255, 255, 255);
        let mut pixmap = Pixmap::new(600, 40).expect("pixmap");
        let mut par_image = 0;
        for step in 0..600 {
            // Juste après le début d'une image, le cache tient ce que la précédente a demandé.
            typo.nouvelle_image();
            par_image = par_image.max(typo.cached_glyph_count());
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
        // Une image : cinq tailles, les lettres distinctes de la phrase, une phase chacune, et
        // l'origine de chaque variante décalée — jamais plus de deux variantes par lettre posée.
        let lettres = "Glucose 0123 — fidelite".chars().count();
        assert!(
            par_image <= 5 * lettres * 2,
            "{par_image} variantes pour une image"
        );
        crate::dire!("[R-46] variantes en cache au plus, image par image : {par_image}");
    }

    /// **Ce qu'une image demande, la suivante le retrouve — même au-delà de l'ancien plafond**
    /// (fiche 104) : sur la carte de Wikipédia, une image vue de loin demande 5 654 variantes, et
    /// un cache de 4 096 les refaisait à chaque image. Ici 6 000 ; la seconde image n'en refait
    /// aucune (les mêmes images, pas des copies). **Et ce qu'elle n'a pas demandé s'oublie.**
    #[test]
    fn test_lettres_1_une_image_garde_ce_qu_elle_demande_et_oublie_le_reste() {
        let typo = Typography::new();
        let demander = |n: usize| -> Vec<std::rc::Rc<glyph::GlyphEntry>> {
            (0..n)
                .map(|i| {
                    let ch = ['A', 'C', 'D', 'E', 'F', 'G'][i / 1000];
                    let size = 10.0 + ((i % 1000) as f32) * 0.1;
                    typo.get_glyph(ch, size, Face::Regular).expect("une image")
                })
                .collect()
        };
        typo.nouvelle_image();
        let premiere = demander(6000);
        assert_eq!(
            typo.cached_glyph_count(),
            6000,
            "rien ne s'évince pendant l'image"
        );
        typo.nouvelle_image();
        let seconde = demander(6000);
        assert!(
            premiere
                .iter()
                .zip(&seconde)
                .all(|(a, b)| std::rc::Rc::ptr_eq(a, b)),
            "la seconde image retrouve chaque lettre de la première"
        );
        typo.nouvelle_image();
        let troisieme = demander(6000);
        assert!(
            premiere
                .iter()
                .zip(&troisieme)
                .all(|(a, b)| std::rc::Rc::ptr_eq(a, b)),
            "redemandée, une lettre est de l'image qui la redemande : elle traverse les images"
        );
        typo.nouvelle_image();
        typo.get_glyph('B', 12.0, Face::Regular).expect("une image");
        typo.nouvelle_image();
        assert_eq!(
            typo.cached_glyph_count(),
            1,
            "seule la lettre de l'image d'avant reste"
        );
    }

    /// **L'origine d'une lettre décalée vit tant que l'une de ses phases sert** (GLYPH-1, fiche
    /// 104) : une phase neuve se dérive de l'origine, qui est alors de cette image — sans quoi
    /// chaque phase neuve rastériserait la lettre à nouveau.
    #[test]
    fn test_lettres_2_l_origine_suit_ses_phases() {
        let typo = Typography::new();
        typo.nouvelle_image();
        typo.glyph_variant('A', 14.0, Face::Regular, 1);
        typo.nouvelle_image();
        typo.glyph_variant('A', 14.0, Face::Regular, 2);
        typo.nouvelle_image();
        let cles: Vec<u8> = typo
            .glyph_cache
            .borrow()
            .keys()
            .map(|(_, _, _, phase)| *phase)
            .collect();
        assert!(cles.contains(&PHASE_ORIGIN), "l'origine reste : {cles:?}");
        assert!(cles.contains(&2), "la phase demandée reste : {cles:?}");
        assert!(!cles.contains(&1), "la phase d'avant s'oublie : {cles:?}");
    }

    /// **Le halo d'un titre traverse les images comme ses lettres** (TITRE-2, fiche 104) : un
    /// titre immobile ne recompose pas ses huit copies à chaque image.
    #[test]
    fn test_lettres_3_le_halo_traverse_les_images() {
        let typo = Typography::new();
        let halo = || typo.halo_variant('W', 30.0, Face::Bold, PHASE_ORIGIN);
        typo.nouvelle_image();
        let premier = halo();
        for _ in 0..3 {
            typo.nouvelle_image();
            assert!(std::rc::Rc::ptr_eq(&premier, &halo()), "le même halo");
        }
    }

    /// **TITRE-2 — le halo composé une fois vaut les huit copies** (fiche 95). Aux seize phases
    /// d'une lettre, sur un fond opaque : le texte détouré en deux passes, et le même en neuf
    /// (huit copies du contour, puis la lettre). Chaque copie d'avant tronquait sa composition à
    /// l'octet : l'écart est d'au plus un niveau par copie, huit — et aucun pixel n'est encré
    /// d'un côté seulement.
    #[test]
    fn test_titre_2_le_halo_vaut_les_huit_copies() {
        let typo = Typography::new();
        let (texte, corps) = ("Eau — Wikipédia", 40.0);
        let style = TextStyle {
            size: corps,
            color: Color::from_rgba8(96, 165, 250, 255),
            face: Face::Bold,
        };
        let fond = Color::from_rgba8(17, 19, 24, 255);
        let mut pire = 0u8;
        for phase in 0..16 {
            let (x, y) = (
                10.0 + (phase % 4) as f32 / 4.0,
                10.0 + (phase / 4) as f32 / 4.0,
            );
            let mut neuve = Pixmap::new(420, 80).expect("tampon");
            neuve.fill(fond);
            typo.draw_text_with_outline(&mut neuve.as_mut(), texte, x, y, style, Color::BLACK);
            let mut ancienne = Pixmap::new(420, 80).expect("tampon");
            ancienne.fill(fond);
            let (noir, bleu) = ((0.0, 0.0, 0.0, 1.0), (96.0, 165.0, 250.0, 1.0));
            let mut plume = x;
            for ch in texte.chars() {
                let lettre = (ch, corps, Face::Bold);
                for (ox, oy) in DETOURAGE {
                    let a = (plume + ox, y + corps + oy);
                    typo.blend_positioned(&mut ancienne.as_mut(), lettre, a, (Color::BLACK, noir));
                }
                let a = (plume, y + corps);
                plume +=
                    typo.blend_positioned(&mut ancienne.as_mut(), lettre, a, (style.color, bleu));
            }
            for (n, a) in neuve.data().iter().zip(ancienne.data()) {
                pire = pire.max(n.abs_diff(*a));
            }
        }
        assert!(
            pire <= 8,
            "le halo s'écarte de {pire} niveaux des huit copies"
        );
    }
}
