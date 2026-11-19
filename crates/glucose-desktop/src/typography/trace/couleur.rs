//! **EMOJI-2 — un émoji se dessine en couleur, couche par couche** (fiche 125, `decisions/18`).
//!
//! Noto Emoji, monochrome (EMOJI-1, `decisions/10`), dessinait 🟢 et 🔴 du même rond rayé — or
//! une IA s'en sert comme de signaux, « fait » et « reste à faire » (fiche 122 § 6). La police des
//! émojis est désormais **Twemoji 17 en COLRv0** (`outils/emojis/construire.sh`) : un émoji est une
//! **pile de glyphes ordinaires**, chacun d'une couleur de sa palette (tables `COLR` et `CPAL`),
//! lue par `ttf-parser` — la lecture même des contours (`decisions/13`).
//!
//! # Deux lectures d'un même glyphe
//!
//! * **Dessiné**, un émoji est ses couches ([`Couches`]), chacune posée de sa couleur : les trois
//!   dessins (pixels, carte graphique, SVG) savent déjà poser un glyphe d'une couleur.
//! * **Lu d'une seule encre** — le gris d'un texte vu de loin (GRIS-1), le halo d'un titre
//!   (TITRE-2), la boîte d'une ligne (TITRE-3) —, il est sa **silhouette**, l'union de ses
//!   couches ([`Contours::rasteriser`]). Sa base n'est qu'un trait sans surface, que nanoemoji y
//!   pose pour dire son étendue : lue seule, elle ne laisserait aucune encre, et un émoji
//!   disparaîtrait de loin.
//!
//! # Pourquoi par le contour, et pas par `fontdue`
//!
//! `fontdue` ne prépare au chargement que les glyphes que la table des caractères atteint
//! (`Font::from_bytes`) : les couches n'y sont pas, il les verrait vides. Elles se rastérisent
//! donc par leur contour, avec le rastériseur des grandes lettres (GRAND-1), et entrent dans le
//! même cache, avec les mêmes métriques, que n'importe quelle lettre.

use super::Contours;
use fontdue::{Metrics, OutlineBounds};
use std::rc::Rc;
use tiny_skia::{Color, FillRule, Paint, Pixmap, Rect, Transform};
use ttf_parser::colr::{ClipBox, CompositeMode, Painter};
use ttf_parser::{GlyphId, RgbaColor};

/// Un glyphe : le rang de sa police dans [`super::super::toutes_les_polices`], et son indice.
pub type Glyphe = (usize, u16);

/// **Une couche** : un glyphe et sa couleur — `None`, l'encre du texte (une lettre, ou une couche
/// que la police peint « de la couleur du texte », l'indice de palette 0xFFFF).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Couche {
    pub glyphe: Glyphe,
    pub couleur: Option<Color>,
}

impl Couche {
    /// **La couleur où cette couche se pose**, dans un texte de couleur `encre` : la sienne, qui
    /// pâlit avec le texte (l'estompe d'une lentille, FRISE-2 et DOMAINE-1, passe par l'alpha) —
    /// ou l'encre même.
    pub fn sur(&self, encre: Color) -> Color {
        match self.couleur {
            None => encre,
            Some(c) => {
                let mut c = c;
                c.set_alpha(c.alpha() * encre.alpha());
                c
            }
        }
    }
}

/// **Les couches d'un glyphe**, dans l'ordre où elles se posent : une seule, à l'encre du texte,
/// pour tout ce qui n'est pas en couleur.
#[derive(Debug, Clone)]
pub enum Couches {
    Une(Glyphe),
    Pile(Rc<[Couche]>),
}

impl Couches {
    pub fn iter(&self) -> impl Iterator<Item = Couche> + '_ {
        let (une, pile): (Option<Couche>, &[Couche]) = match self {
            Self::Une(glyphe) => (
                Some(Couche {
                    glyphe: *glyphe,
                    couleur: None,
                }),
                &[],
            ),
            Self::Pile(pile) => (None, pile),
        };
        une.into_iter().chain(pile.iter().copied())
    }
}

/// Ce que la peinture d'un glyphe en couleur annonce : pour chaque couche, son glyphe, puis sa
/// couleur. COLRv0 ne fait que cela ; les dégradés, transformations et découpes de COLRv1 n'ont
/// pas cours dans la police de Glucose (`outils/emojis/construire.py` écrit du COLRv0).
#[derive(Default)]
struct Releve {
    glyphe: u16,
    couches: Vec<(u16, RgbaColor)>,
}

impl<'a> Painter<'a> for Releve {
    fn outline_glyph(&mut self, glyph_id: GlyphId) {
        self.glyphe = glyph_id.0;
    }
    fn paint(&mut self, paint: ttf_parser::colr::Paint<'a>) {
        if let ttf_parser::colr::Paint::Solid(c) = paint {
            self.couches.push((self.glyphe, c));
        }
    }
    fn push_clip(&mut self) {}
    fn push_clip_box(&mut self, _: ClipBox) {}
    fn pop_clip(&mut self) {}
    fn push_layer(&mut self, _: CompositeMode) {}
    fn pop_layer(&mut self) {}
    fn push_transform(&mut self, _: ttf_parser::Transform) {}
    fn pop_transform(&mut self) {}
}

impl Contours {
    /// **L'indice de `ch` dans cette police**, s'il y est.
    pub fn indice(&self, police: usize, ch: char) -> Option<u16> {
        self.polices[police]
            .glyph_index(ch)
            .map(|g| g.0)
            .filter(|&i| i != 0)
    }

    /// **Les couches de ce glyphe**, lues une fois et gardées.
    ///
    /// Une couche « de la couleur du texte » ne se distingue pas à sa couleur : `ttf-parser` la
    /// remplace par celle qu'on lui donne. La peinture se fait donc **deux fois**, d'une encre noire
    /// transparente puis d'une encre blanche opaque : la couche dont la couleur suit l'encre est
    /// celle du texte — exactement, sans couleur sentinelle qu'une palette pourrait contenir.
    pub fn couches(&self, (police, indice): Glyphe) -> Couches {
        let face = &self.polices[police];
        if !face.is_color_glyph(GlyphId(indice)) {
            return Couches::Une((police, indice));
        }
        let pile = self
            .piles
            .borrow_mut()
            .entry((police, indice))
            .or_insert_with(|| {
                let peindre = |encre| {
                    let mut releve = Releve::default();
                    face.paint_color_glyph(GlyphId(indice), 0, encre, &mut releve);
                    releve.couches
                };
                let noire = peindre(RgbaColor::new(0, 0, 0, 0));
                let blanche = peindre(RgbaColor::new(255, 255, 255, 255));
                noire
                    .iter()
                    .zip(&blanche)
                    .map(|(&(glyphe, a), &(_, b))| Couche {
                        glyphe: (police, glyphe),
                        couleur: (a == b)
                            .then(|| Color::from_rgba8(a.red, a.green, a.blue, a.alpha)),
                    })
                    .collect()
            })
            .clone();
        Couches::Pile(pile)
    }

    /// **De combien la plume avance** après ce glyphe, à ce corps — l'em en pixels, la règle de
    /// `fontdue` (l'avance de la police, mise à l'échelle).
    pub fn avance(&self, (police, indice): Glyphe, corps: f32) -> f32 {
        let face = &self.polices[police];
        let s = corps / f32::from(face.units_per_em());
        face.glyph_hor_advance(GlyphId(indice))
            .map_or(0.0, |a| f32::from(a) * s)
    }

    /// Les formes que ce glyphe pose d'une seule encre : son contour, ou ceux de ses couches.
    fn formes(&self, glyphe: Glyphe) -> Vec<Rc<tiny_skia::Path>> {
        self.couches(glyphe)
            .iter()
            .filter_map(|c| self.contour(c.glyphe.0, c.glyphe.1))
            .collect()
    }

    /// **Les métriques de ce glyphe**, comme `fontdue` les donne : la boîte de son encre en pixels
    /// entiers depuis la plume et la ligne de base (l'ordonnée vers le haut), et son avance.
    pub fn metriques(&self, glyphe: Glyphe, corps: f32) -> Metrics {
        let s = corps / f32::from(self.polices[glyphe.0].units_per_em());
        cadre(&self.formes(glyphe), s, self.avance(glyphe, corps))
    }

    /// **L'image de ce glyphe**, comme `fontdue` la rend : ses métriques, et sa couverture rangée
    /// par rangée depuis le haut — celle d'un émoji est sa **silhouette**, l'union de ses couches.
    pub fn rasteriser(&self, glyphe: Glyphe, corps: f32) -> (Metrics, Vec<u8>) {
        let formes = self.formes(glyphe);
        let s = corps / f32::from(self.polices[glyphe.0].units_per_em());
        let m = cadre(&formes, s, self.avance(glyphe, corps));
        let Some(mut image) = Pixmap::new(m.width as u32, m.height as u32) else {
            return (
                Metrics {
                    width: 0,
                    height: 0,
                    ..m
                },
                Vec::new(),
            );
        };
        let mut paint = Paint::default();
        paint.set_color(Color::WHITE);
        paint.anti_alias = true;
        // L'em en pixels, l'axe des ordonnées retourné : le haut de la boîte sur la rangée zéro.
        let haut = (m.ymin + m.height as i32) as f32;
        let pose = Transform::from_row(s, 0.0, 0.0, -s, -m.xmin as f32, haut);
        for forme in &formes {
            image.fill_path(forme, &paint, FillRule::Winding, pose, None);
        }
        let couverture = image.pixels().iter().map(|p| p.alpha()).collect();
        (m, couverture)
    }
}

/// La boîte de ces formes, à l'échelle `s`, en pixels entiers qui la contiennent.
fn cadre(formes: &[Rc<tiny_skia::Path>], s: f32, avance: f32) -> Metrics {
    let vide = Metrics {
        advance_width: avance,
        ..Metrics::default()
    };
    let Some(b) = formes
        .iter()
        .map(|f| f.bounds())
        .reduce(|a, b| union(a, b).unwrap_or(a))
    else {
        return vide;
    };
    // Les contours sont dans les unités de la police, l'ordonnée vers le haut : `top` est le bas.
    let (x0, y0) = ((b.left() * s).floor(), (b.top() * s).floor());
    let (x1, y1) = ((b.right() * s).ceil(), (b.bottom() * s).ceil());
    Metrics {
        xmin: x0 as i32,
        ymin: y0 as i32,
        width: (x1 - x0) as usize,
        height: (y1 - y0) as usize,
        advance_width: avance,
        advance_height: 0.0,
        bounds: OutlineBounds {
            xmin: b.left() * s,
            ymin: b.top() * s,
            width: b.width() * s,
            height: b.height() * s,
        },
    }
}

fn union(a: Rect, b: Rect) -> Option<Rect> {
    Rect::from_ltrb(
        a.left().min(b.left()),
        a.top().min(b.top()),
        a.right().max(b.right()),
        a.bottom().max(b.bottom()),
    )
}

#[cfg(test)]
mod tests;
