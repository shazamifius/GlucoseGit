//! Le dessin des formules : de la géométrie de [`glucose_math`] aux pixels.
//!
//! # Le partage des rôles
//!
//! [`glucose_math`] dit *quel caractère, dans quelle famille, à quelle taille, à quel endroit*,
//! en `em` et sans toucher un pixel. Ce module-ci apporte les fontes, la conversion vers
//! l'écran, et le cache — c'est-à-dire tout ce qui ne peut pas se tester sans dessiner.
//!
//! # Pourquoi un cache, et sur quelle clé
//!
//! Mettre une formule en page coûte de l'ordre de la milliseconde : KaTeX analyse la source,
//! expanse les macros et calcule les métriques. À soixante images par seconde, une seule
//! formule à l'écran mangerait donc six pour cent du budget — pour un résultat identique à
//! chaque image.
//!
//! La mise en page ne dépend que de deux choses : **la source et le mode**. Elle ne dépend ni du
//! zoom ni de la position, puisqu'elle est exprimée en `em` — c'est tout l'intérêt de la
//! frontière. La clé est donc exactement `(source, mode)`, et le cache survit au déplacement
//! comme au zoom.
//!
//! # Le repère
//!
//! `glucose_math` compte `y` vers le **haut**, comme TeX ; l'écran compte vers le bas. La
//! conversion est une soustraction, faite une fois ici.

use crate::typography::Typography;
use glucose_math::layout::{MathItem, MathLayout};
use glucose_math::{Family, MathError, Mode, Style};
use std::cell::RefCell;
use std::collections::HashMap;
use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};

/// Les vingt fontes de KaTeX, embarquées. Elles viennent du dépôt de KaTeX (licence MIT, texte
/// dans `assets/katex/LICENSE-KaTeX.txt`) et pèsent 540 Ko au total.
///
/// Elles ne sont pas un choix esthétique : ce sont **celles dont KaTeX donne les métriques**.
/// Dessiner sa mise en page avec d'autres fontes produirait des positions justes et des glyphes
/// de la mauvaise largeur — c'est-à-dire un résultat faux qui aurait l'air presque bon.
const FONTS: &[(&str, &[u8])] = &[
    ("KaTeX_AMS-Regular", include_bytes!("../../assets/katex/KaTeX_AMS-Regular.ttf")),
    ("KaTeX_Caligraphic-Bold", include_bytes!("../../assets/katex/KaTeX_Caligraphic-Bold.ttf")),
    ("KaTeX_Caligraphic-Regular", include_bytes!("../../assets/katex/KaTeX_Caligraphic-Regular.ttf")),
    ("KaTeX_Fraktur-Bold", include_bytes!("../../assets/katex/KaTeX_Fraktur-Bold.ttf")),
    ("KaTeX_Fraktur-Regular", include_bytes!("../../assets/katex/KaTeX_Fraktur-Regular.ttf")),
    ("KaTeX_Main-Bold", include_bytes!("../../assets/katex/KaTeX_Main-Bold.ttf")),
    ("KaTeX_Main-BoldItalic", include_bytes!("../../assets/katex/KaTeX_Main-BoldItalic.ttf")),
    ("KaTeX_Main-Italic", include_bytes!("../../assets/katex/KaTeX_Main-Italic.ttf")),
    ("KaTeX_Main-Regular", include_bytes!("../../assets/katex/KaTeX_Main-Regular.ttf")),
    ("KaTeX_Math-BoldItalic", include_bytes!("../../assets/katex/KaTeX_Math-BoldItalic.ttf")),
    ("KaTeX_Math-Italic", include_bytes!("../../assets/katex/KaTeX_Math-Italic.ttf")),
    ("KaTeX_SansSerif-Bold", include_bytes!("../../assets/katex/KaTeX_SansSerif-Bold.ttf")),
    ("KaTeX_SansSerif-Italic", include_bytes!("../../assets/katex/KaTeX_SansSerif-Italic.ttf")),
    ("KaTeX_SansSerif-Regular", include_bytes!("../../assets/katex/KaTeX_SansSerif-Regular.ttf")),
    ("KaTeX_Script-Regular", include_bytes!("../../assets/katex/KaTeX_Script-Regular.ttf")),
    ("KaTeX_Size1-Regular", include_bytes!("../../assets/katex/KaTeX_Size1-Regular.ttf")),
    ("KaTeX_Size2-Regular", include_bytes!("../../assets/katex/KaTeX_Size2-Regular.ttf")),
    ("KaTeX_Size3-Regular", include_bytes!("../../assets/katex/KaTeX_Size3-Regular.ttf")),
    ("KaTeX_Size4-Regular", include_bytes!("../../assets/katex/KaTeX_Size4-Regular.ttf")),
    ("KaTeX_Typewriter-Regular", include_bytes!("../../assets/katex/KaTeX_Typewriter-Regular.ttf")),
];

/// Le nom de fichier d'une famille et d'un style, tel qu'il est embarqué.
///
/// Toutes les combinaisons n'existent pas : les fontes de taille (`Size1` à `Size4`) et les
/// symboles AMS n'ont qu'une variante. Demander un italique qui n'existe pas rend la variante
/// disponible plutôt que rien — un glyphe droit vaut mieux qu'un glyphe absent.
fn nom_de_fonte(family: Family, style: Style) -> String {
    let base = family.font_name();
    let variante = match family {
        Family::Size1 | Family::Size2 | Family::Size3 | Family::Size4 => "Regular",
        Family::Ams | Family::Script | Family::Typewriter => "Regular",
        Family::Math => {
            // KaTeX_Math n'existe qu'en italique et gras italique.
            if style.bold {
                "BoldItalic"
            } else {
                "Italic"
            }
        }
        Family::Caligraphic | Family::Fraktur => {
            if style.bold {
                "Bold"
            } else {
                "Regular"
            }
        }
        // `KaTeX_SansSerif` n'a pas de gras italique : KaTeX ne demande jamais la combinaison,
        // mais un repli explicite vaut mieux qu'un glyphe absent si elle arrivait un jour.
        Family::SansSerif if style.bold && style.italic => "Bold",
        Family::Main | Family::SansSerif => style.suffix(),
    };
    format!("{base}-{variante}")
}

/// Les fontes de KaTeX, chargées une fois, plus le souvenir des formules déjà mises en page.
///
/// # Pourquoi le cache est derrière un `RefCell`
///
/// Mesurer une formule et la dessiner sont, pour l'appelant, des questions **sans effet** : la
/// carte demande « quelle place prend cette formule » et « dessine-la ici ». Que la réponse soit
/// retenue est un détail interne.
///
/// L'exposer par un `&mut` obligerait toute la chaîne de rendu — la passe, la carte, le corps —
/// à porter une référence mutable de plus, uniquement pour une mémoire. Le `RefCell` garde cette
/// mémoire là où elle est, sans rien imposer au-dessus. C'est exactement la situation pour
/// laquelle il existe, et le rendu est mono-thread.
pub struct MathRenderer {
    fonts: HashMap<&'static str, fontdue::Font>,
    /// `(source, mode)` → mise en page, ou l'erreur de KaTeX si la source est fausse.
    cache: RefCell<HashMap<(String, Mode), Result<MathLayout, MathError>>>,
}

impl Default for MathRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MathRenderer {
    pub fn new() -> Self {
        let fonts = FONTS
            .iter()
            .filter_map(|(nom, octets)| {
                fontdue::Font::from_bytes(*octets, fontdue::FontSettings::default())
                    .ok()
                    .map(|f| (*nom, f))
            })
            .collect();
        Self { fonts, cache: RefCell::new(HashMap::new()) }
    }

    /// Le nombre de fontes effectivement chargées — vingt si tout va bien.
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// La mise en page d'une formule, calculée une fois puis retenue.
    ///
    /// Le résultat est **copié** plutôt que prêté : une formule tient en quelques dizaines
    /// d'éléments, et la copie coûte sans commune mesure avec l'analyse qu'elle évite. Le prêt
    /// obligerait l'appelant à tenir un emprunt du cache pendant tout son dessin.
    pub fn layout(&self, source: &str, mode: Mode) -> Result<MathLayout, MathError> {
        if let Some(connu) = self.cache.borrow().get(&(source.to_string(), mode)) {
            return connu.clone();
        }
        let calcule = glucose_math::layout(source, mode);
        self.cache.borrow_mut().insert((source.to_string(), mode), calcule.clone());
        calcule
    }

    /// Combien de formules distinctes sont en mémoire.
    pub fn cached(&self) -> usize {
        self.cache.borrow().len()
    }

    /// La taille d'une formule à une taille de police donnée, en pixels : `(largeur, hauteur
    /// au-dessus de la ligne de base, profondeur en dessous)`.
    ///
    /// C'est ce dont la mise en page d'une carte a besoin pour réserver la place, **avant** de
    /// dessiner quoi que ce soit.
    pub fn measure(&self, source: &str, mode: Mode, font_size: f32) -> Option<(f32, f32, f32)> {
        let l = self.layout(source, mode).ok()?;
        let em = font_size;
        Some((l.width as f32 * em, l.height as f32 * em, l.depth as f32 * em))
    }

    /// Dessine une formule dont la **ligne de base** commence en `(x, y)` à l'écran.
    ///
    /// Rend `false` si la source est fausse : l'appelant décide alors quoi montrer — dans une
    /// carte, la source elle-même, en rouge.
    pub fn draw(
        &self,
        pixmap: &mut PixmapMut,
        typography: &Typography,
        source: &str,
        mode: Mode,
        x: f32,
        y: f32,
        font_size: f32,
        color: Color,
    ) -> bool {
        let Ok(layout) = self.layout(source, mode) else {
            return false;
        };

        for item in &layout.items {
            match item {
                MathItem::Glyph { text, x: gx, y: gy, size, family, style } => {
                    let nom = nom_de_fonte(*family, *style);
                    let Some(font) = self.fonts.get(nom.as_str()) else { continue };
                    dessine_glyphe(
                        pixmap,
                        font,
                        text,
                        x + *gx as f32 * font_size,
                        // `y` croît vers le haut côté mathématiques, vers le bas à l'écran.
                        y - *gy as f32 * font_size,
                        *size as f32 * font_size,
                        color,
                    );
                }
                MathItem::Rule { x: rx, y: ry, width, height } => {
                    let mut paint = Paint { anti_alias: true, ..Default::default() };
                    paint.set_color(color);
                    // Un filet d'épaisseur inférieure au pixel disparaîtrait ; on lui en donne
                    // un, sans quoi une barre de fraction s'évanouit au dézoom.
                    let h = (*height as f32 * font_size).max(1.0);
                    if let Some(r) = Rect::from_xywh(
                        x + *rx as f32 * font_size,
                        y - *ry as f32 * font_size - h,
                        (*width as f32 * font_size).max(1.0),
                        h,
                    ) {
                        pixmap.fill_rect(r, &paint, Transform::identity(), None);
                    }
                }
                MathItem::Path { name, x: px, y: py, width, height } => {
                    dessine_forme(
                        pixmap,
                        typography,
                        name,
                        x + *px as f32 * font_size,
                        y - *py as f32 * font_size,
                        *width as f32 * font_size,
                        *height as f32 * font_size,
                        color,
                    );
                }
            }
        }
        true
    }
}

/// Dessine un caractère avec une fonte de KaTeX, à la taille voulue.
///
/// `fontdue` rastérise le glyphe puis on le compose en alpha prémultiplié, comme le fait déjà
/// [`Typography`] pour le texte ordinaire. La différence est qu'ici la fonte change d'un glyphe
/// à l'autre, ce qui interdit de réutiliser son cache tel quel.
fn dessine_glyphe(
    pixmap: &mut PixmapMut,
    font: &fontdue::Font,
    texte: &str,
    x: f32,
    y: f32,
    taille: f32,
    couleur: Color,
) {
    if taille < 0.5 {
        return;
    }
    let mut plume = x;
    for c in texte.chars() {
        let (metrics, bitmap) = font.rasterize(c, taille);
        if metrics.width > 0 && metrics.height > 0 {
            let gx = (plume + metrics.xmin as f32).round() as i32;
            let gy = (y - metrics.height as f32 - metrics.ymin as f32).round() as i32;
            compose(pixmap, &bitmap, metrics.width, metrics.height, gx, gy, couleur);
        }
        plume += metrics.advance_width;
    }
}

/// Dessine une forme étirable de KaTeX dans sa boîte.
///
/// # Ce que ceci fait, et ce qu'il reste à faire
///
/// KaTeX décrit ces formes par des chemins vectoriels — une quinzaine en tout, radicaux,
/// accolades et flèches longues. Les dessiner exactement demande de lire ces chemins ; en
/// attendant, un radical est **construit** : la diagonale montante et le trait horizontal qui
/// couvre le contenu. C'est la partie qui se voit, et elle se pose exactement dans la boîte que
/// KaTeX a calculée.
///
/// Les autres formes ne sont pas dessinées plutôt que mal dessinées. Le nom est là, la boîte
/// est là : ce qui manque est le tracé, pas l'information.
fn dessine_forme(
    pixmap: &mut PixmapMut,
    _typography: &Typography,
    nom: &str,
    x: f32,
    y: f32,
    largeur: f32,
    hauteur: f32,
    couleur: Color,
) {
    if !nom.starts_with("sqrt") || largeur <= 0.0 || hauteur <= 0.0 {
        return;
    }
    let epaisseur = (hauteur * 0.045).max(1.0);
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(couleur);
    let stroke = tiny_skia::Stroke { width: epaisseur, ..Default::default() };

    let mut pb = tiny_skia::PathBuilder::new();
    // La jambe du radical : du creux en bas à gauche jusqu'au sommet, puis le trait qui
    // surplombe le contenu.
    pb.move_to(x, y - hauteur * 0.45);
    pb.line_to(x + largeur * 0.28, y - hauteur * 0.06);
    pb.line_to(x + largeur * 0.62, y - hauteur * 0.96);
    pb.line_to(x + largeur, y - hauteur * 0.96);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

/// Compose un bitmap de couverture sur le pixmap, en alpha prémultiplié.
fn compose(
    pixmap: &mut PixmapMut,
    bitmap: &[u8],
    bw: usize,
    bh: usize,
    x: i32,
    y: i32,
    couleur: Color,
) {
    let (pw, ph) = (pixmap.width() as i32, pixmap.height() as i32);
    let (cr, cg, cb) = (
        (couleur.red() * 255.0) as u32,
        (couleur.green() * 255.0) as u32,
        (couleur.blue() * 255.0) as u32,
    );
    let ca = (couleur.alpha() * 255.0) as u32;
    let data = pixmap.data_mut();

    for row in 0..bh {
        let py = y + row as i32;
        if py < 0 || py >= ph {
            continue;
        }
        for col in 0..bw {
            let px = x + col as i32;
            if px < 0 || px >= pw {
                continue;
            }
            let couverture = bitmap[row * bw + col] as u32;
            if couverture == 0 {
                continue;
            }
            let a = couverture * ca / 255;
            let i = ((py * pw + px) * 4) as usize;
            for (k, c) in [cr, cg, cb].into_iter().enumerate() {
                let fond = data[i + k] as u32;
                data[i + k] = ((c * a + fond * (255 - a)) / 255) as u8;
            }
            let fond_a = data[i + 3] as u32;
            data[i + 3] = (a + fond_a * (255 - a) / 255) as u8;
        }
    }
}

#[cfg(test)]
mod tests;
