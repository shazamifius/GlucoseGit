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

use crate::params::Pen;
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
    (
        "KaTeX_AMS-Regular",
        include_bytes!("../../assets/katex/KaTeX_AMS-Regular.ttf"),
    ),
    (
        "KaTeX_Caligraphic-Bold",
        include_bytes!("../../assets/katex/KaTeX_Caligraphic-Bold.ttf"),
    ),
    (
        "KaTeX_Caligraphic-Regular",
        include_bytes!("../../assets/katex/KaTeX_Caligraphic-Regular.ttf"),
    ),
    (
        "KaTeX_Fraktur-Bold",
        include_bytes!("../../assets/katex/KaTeX_Fraktur-Bold.ttf"),
    ),
    (
        "KaTeX_Fraktur-Regular",
        include_bytes!("../../assets/katex/KaTeX_Fraktur-Regular.ttf"),
    ),
    (
        "KaTeX_Main-Bold",
        include_bytes!("../../assets/katex/KaTeX_Main-Bold.ttf"),
    ),
    (
        "KaTeX_Main-BoldItalic",
        include_bytes!("../../assets/katex/KaTeX_Main-BoldItalic.ttf"),
    ),
    (
        "KaTeX_Main-Italic",
        include_bytes!("../../assets/katex/KaTeX_Main-Italic.ttf"),
    ),
    (
        "KaTeX_Main-Regular",
        include_bytes!("../../assets/katex/KaTeX_Main-Regular.ttf"),
    ),
    (
        "KaTeX_Math-BoldItalic",
        include_bytes!("../../assets/katex/KaTeX_Math-BoldItalic.ttf"),
    ),
    (
        "KaTeX_Math-Italic",
        include_bytes!("../../assets/katex/KaTeX_Math-Italic.ttf"),
    ),
    (
        "KaTeX_SansSerif-Bold",
        include_bytes!("../../assets/katex/KaTeX_SansSerif-Bold.ttf"),
    ),
    (
        "KaTeX_SansSerif-Italic",
        include_bytes!("../../assets/katex/KaTeX_SansSerif-Italic.ttf"),
    ),
    (
        "KaTeX_SansSerif-Regular",
        include_bytes!("../../assets/katex/KaTeX_SansSerif-Regular.ttf"),
    ),
    (
        "KaTeX_Script-Regular",
        include_bytes!("../../assets/katex/KaTeX_Script-Regular.ttf"),
    ),
    (
        "KaTeX_Size1-Regular",
        include_bytes!("../../assets/katex/KaTeX_Size1-Regular.ttf"),
    ),
    (
        "KaTeX_Size2-Regular",
        include_bytes!("../../assets/katex/KaTeX_Size2-Regular.ttf"),
    ),
    (
        "KaTeX_Size3-Regular",
        include_bytes!("../../assets/katex/KaTeX_Size3-Regular.ttf"),
    ),
    (
        "KaTeX_Size4-Regular",
        include_bytes!("../../assets/katex/KaTeX_Size4-Regular.ttf"),
    ),
    (
        "KaTeX_Typewriter-Regular",
        include_bytes!("../../assets/katex/KaTeX_Typewriter-Regular.ttf"),
    ),
];

/// Le nom de fichier d'une famille et d'un style, tel qu'il est embarqué — le nom même que les
/// métriques de KaTeX portent (`Family::nom_de_fonte`), pour que la mesure et le dessin ne
/// puissent pas choisir deux fontes différentes.
fn nom_de_fonte(family: Family, style: Style) -> String {
    format!("KaTeX_{}", family.nom_de_fonte(style))
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
        Self {
            fonts,
            cache: RefCell::new(HashMap::new()),
        }
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
        let calcule = glucose_math::layout_avec(source, mode, &|c, f, s| self.avance(c, f, s));
        self.cache
            .borrow_mut()
            .insert((source.to_string(), mode), calcule.clone());
        calcule
    }

    /// **L'avance d'un caractère dans la fonte qui le dessinera**, en `em` — ce que le navigateur
    /// de Glucose Tauri employait, et que les métriques de KaTeX ne donnent pas toujours
    /// (`∬`, `°` : [`glucose_math::layout_avec`]). Rien si la fonte ne porte pas le caractère.
    fn avance(&self, c: char, family: Family, style: Style) -> Option<f64> {
        let font = self.fonts.get(nom_de_fonte(family, style).as_str())?;
        let corps = font.units_per_em();
        (font.lookup_glyph_index(c) != 0)
            .then(|| f64::from(font.metrics(c, corps).advance_width) / f64::from(corps))
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
        Some((
            l.width as f32 * em,
            l.height as f32 * em,
            l.depth as f32 * em,
        ))
    }

    /// Dessine une formule dont la **ligne de base** commence à la plume, à son corps.
    ///
    /// Rend `false` si la source est fausse : l'appelant décide alors quoi montrer — dans une
    /// carte, la source elle-même, en rouge.
    pub fn draw(
        &self,
        pixmap: &mut PixmapMut,
        source: &str,
        mode: Mode,
        pen: Pen,
        color: Color,
    ) -> bool {
        let Ok(layout) = self.layout(source, mode) else {
            return false;
        };
        let Pen { x, y, font_size } = pen;

        for item in &layout.items {
            match item {
                MathItem::Glyph {
                    text,
                    x: gx,
                    y: gy,
                    size,
                    family,
                    style,
                } => {
                    let nom = nom_de_fonte(*family, *style);
                    let Some(font) = self.fonts.get(nom.as_str()) else {
                        continue;
                    };
                    let plume = Pen {
                        x: x + *gx as f32 * font_size,
                        // `y` croît vers le haut côté mathématiques, vers le bas à l'écran.
                        y: y - *gy as f32 * font_size,
                        font_size: *size as f32 * font_size,
                    };
                    dessine_glyphe(pixmap, font, text, plume, color);
                }
                MathItem::Rule {
                    x: rx,
                    y: ry,
                    width,
                    height,
                } => dessine_filet(pixmap, (*rx, *ry, *width, *height), pen, color),
                MathItem::Path {
                    x: px,
                    y: py,
                    width,
                    height,
                    forme: Some(forme),
                    ..
                } => {
                    // La boîte de la forme : sa ligne de base en bas, sa hauteur au-dessus.
                    // Une boîte sans surface ne se construit pas, donc ne se dessine pas.
                    let (w, h) = (*width as f32 * font_size, *height as f32 * font_size);
                    let bas = y - *py as f32 * font_size;
                    if let Some(boite) = Rect::from_xywh(x + *px as f32 * font_size, bas - h, w, h)
                    {
                        let trait_px = forme.epaisseur.map(|e| e as f32 * font_size);
                        dessine_forme(pixmap, forme, boite, (color, trait_px));
                    }
                }
                // Un chemin inconnu ne se dessine pas, plutôt que de travers.
                MathItem::Path { forme: None, .. } => {}
            }
        }
        true
    }
}

/// Dessine un filet `(x, y, largeur, épaisseur)`, en `em` depuis la plume, `y` au bas du filet.
fn dessine_filet(
    pixmap: &mut PixmapMut,
    (rx, ry, width, height): (f64, f64, f64, f64),
    Pen { x, y, font_size }: Pen,
    color: Color,
) {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(color);
    // Un filet d'épaisseur inférieure au pixel disparaîtrait ; on lui en donne un, sans quoi une
    // barre de fraction s'évanouit au dézoom.
    let h = (height as f32 * font_size).max(1.0);
    if let Some(r) = Rect::from_xywh(
        x + rx as f32 * font_size,
        y - ry as f32 * font_size - h,
        (width as f32 * font_size).max(1.0),
        h,
    ) {
        pixmap.fill_rect(r, &paint, Transform::identity(), None);
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
    pen: Pen,
    couleur: Color,
) {
    let Pen {
        x,
        y,
        font_size: taille,
    } = pen;
    if taille < 0.5 {
        return;
    }
    let mut plume = x;
    for c in texte.chars() {
        let (metrics, bitmap) = font.rasterize(c, taille);
        if metrics.width > 0 && metrics.height > 0 {
            let gx = (plume + metrics.xmin as f32).round() as i32;
            let gy = (y - metrics.height as f32 - metrics.ymin as f32).round() as i32;
            compose(
                pixmap,
                &bitmap,
                metrics.width,
                metrics.height,
                gx,
                gy,
                couleur,
            );
        }
        plume += metrics.advance_width;
    }
}

/// **Dessine une forme de KaTeX dans sa boîte** — la ligne de base en est le bas — par son tracé
/// même (FORMULE-1).
///
/// Le radical était **construit** ici — trois segments reconnaissables de loin, faux de près —,
/// et aucune autre forme ne se dessinait : ni flèche longue, ni accolade, ni chapeau large.
/// KaTeX décrit chacune par un chemin, une boîte de vue et une règle d'ajustement ; la
/// transformation est celle de SVG ([`glucose_math::Forme::transformation`]). La forme se peint
/// dans une image de la taille de sa boîte, qui la rogne comme le `overflow: hidden` de KaTeX :
/// un radical de 400 000 unités ne montre que la longueur de ce qu'il couvre.
///
/// Un trait (`<line>`, les ratures de `\cancel`) se trace à l'épaisseur qu'il porte, en pixels :
/// elle ne s'étire pas avec la boîte.
fn dessine_forme(
    pixmap: &mut PixmapMut,
    forme: &glucose_math::Forme,
    boite: Rect,
    (couleur, trait_px): (Color, Option<f32>),
) {
    use glucose_math::Commande;
    let (ox, oy) = (boite.left().floor(), boite.top().floor());
    let (l, h) = (
        (boite.right() - ox).ceil() as u32,
        (boite.bottom() - oy).ceil() as u32,
    );
    let Some(mut image) = tiny_skia::Pixmap::new(l.max(1), h.max(1)) else {
        return;
    };
    let (sx, sy, dx, dy) =
        forme.transformation((f64::from(boite.width()), f64::from(boite.height())));
    let (fx, fy) = (f64::from(boite.left() - ox), f64::from(boite.top() - oy));
    let point = |x: f64, y: f64| ((fx + dx + x * sx) as f32, (fy + dy + y * sy) as f32);
    let mut pb = tiny_skia::PathBuilder::new();
    for c in &forme.commandes {
        match *c {
            Commande::Aller(x, y) => {
                let (x, y) = point(x, y);
                pb.move_to(x, y);
            }
            Commande::Ligne(x, y) => {
                let (x, y) = point(x, y);
                pb.line_to(x, y);
            }
            Commande::Cubique(a, b, c2, d, e, f) => {
                let ((a, b), (c2, d), (e, f)) = (point(a, b), point(c2, d), point(e, f));
                pb.cubic_to(a, b, c2, d, e, f);
            }
            Commande::Fermer => pb.close(),
        }
    }
    let Some(chemin) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(couleur);
    match trait_px {
        Some(largeur) => {
            let trait_ = tiny_skia::Stroke {
                width: largeur,
                ..Default::default()
            };
            image.stroke_path(&chemin, &paint, &trait_, Transform::identity(), None);
        }
        None => image.fill_path(
            &chemin,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        ),
    }
    pixmap.draw_pixmap(
        ox as i32,
        oy as i32,
        image.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );
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
