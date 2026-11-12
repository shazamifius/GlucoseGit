//! **Le texte détouré** : un texte lisible sur ce qu'il chevauche — le titre d'une membrane, le nom
//! d'une zone — par un contour de la teinte du fond (R-26), et ce que la fiche 95 en a fait (TITRE-2) :
//! le halo des huit copies composé une fois, et le trait d'une lettre tracée.

use super::glyph::{self, split_position, GlyphEntry, PHASE_ORIGIN, SUBPIXEL_PHASES};
use super::{clamp_font_size, trace, Face, TextStyle, Typography};
use std::rc::Rc;
use tiny_skia::{Color, PixmapMut};

/// **Le détourage d'un texte** ([`Typography::draw_text_with_outline`]) : les huit copies, de la
/// teinte du contour, qui passent sous le texte — le SVG pose les mêmes (P5, fiche 70).
pub const DETOURAGE: [(f32, f32); 8] = [
    (-1.5, 0.0),
    (1.5, 0.0),
    (0.0, -1.5),
    (0.0, 1.5),
    (-1.0, -1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (1.0, 1.0),
];

impl Typography {
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

        let out_r = outline_color.red() * 255.0;
        let out_g = outline_color.green() * 255.0;
        let out_b = outline_color.blue() * 255.0;
        let out_a = outline_color.alpha();

        let r = color.red() * 255.0;
        let g = color.green() * 255.0;
        let b = color.blue() * 255.0;
        let a = color.alpha();

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            if trace::se_trace(size) {
                let glyphe = (&mut *pixmap, ch, size, face);
                x += self.detourer(glyphe, (x, y + size), (color, outline_color));
                continue;
            }
            let advance = {
                // 1. Le halo : les huit copies du contour, composées une fois (TITRE-2).
                let (cell_x, phase_x) = split_position(x);
                let (cell_y, phase_y) = split_position(y + size);
                let halo = self.halo_variant(ch, size, face, phase_y * SUBPIXEL_PHASES + phase_x);
                let m = &halo.metrics;
                let coin = (cell_x + m.xmin, cell_y - m.ymin - m.height as i32);
                let bornes = (pixmap.width() as i32, pixmap.height() as i32);
                let couleur = (out_r, out_g, out_b, out_a);
                glyph::blend_glyph(pixmap.data_mut(), bornes, &halo, coin, couleur);
                // 2. Passe principale
                self.blend_positioned(
                    pixmap,
                    (ch, size, face),
                    (x, y + size),
                    (color, (r, g, b, a)),
                )
            };
            x += advance;
        }
        x
    }

    /// **Où une ligne détourée peut poser de l'encre** (TITRE-3, fiche 108) : la boîte
    /// `(gauche, haut, droite, bas)` des pixels que [`Self::draw_text_with_outline`] peut
    /// toucher, relative au point où on l'appelle — le haut du corps en `0`, sa ligne de base
    /// en `size`.
    ///
    /// Lue sur les mesures de chaque lettre, plus ce que le dessin y ajoute : le détourage
    /// ([`DETOURAGE`], ou la demi-largeur du trait d'une lettre tracée, qui ne la dépasse pas),
    /// la cellule où une lettre tombe selon sa position (un pixel au plus), et le pixel dont une
    /// lettre décalée de sa phase s'élargit. C'est ce que la texture d'un titre doit contenir
    /// pour ne couper aucune encre.
    pub fn encre_detouree(&self, text: &str, size: f32, face: Face) -> (f32, f32, f32, f32) {
        let size = clamp_font_size(size);
        let (mut g, mut h, mut d, mut b) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut plume = 0.0;
        for ch in text.chars().filter(|&ch| ch != '\n') {
            let (police, indice) = self.police(ch, face);
            let m = police.metrics_indexed(indice, size);
            g = g.min(plume + m.xmin as f32);
            d = d.max(plume + m.xmin as f32 + m.width as f32);
            h = h.min(size - (m.ymin + m.height as i32) as f32);
            b = b.max(size - m.ymin as f32);
            plume += m.advance_width;
        }
        if g > d {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let halo = DETOURAGE
            .iter()
            .fold(0.0_f32, |m, &(x, y)| m.max(x.abs()).max(y.abs()));
        let deborde = halo + 2.0;
        (g - deborde, h - deborde, d + deborde, b + deborde)
    }

    /// **Une lettre tracée, détourée** (TITRE-2) : le trait du contour, puis la lettre — de la
    /// demi-largeur des huit copies d'une petite lettre ([`DETOURAGE`]). Rend son avance.
    fn detourer(
        &self,
        (pixmap, ch, size, face): (&mut PixmapMut<'_>, char, f32, Face),
        plume: (f32, f32),
        couleurs: (Color, Color),
    ) -> f32 {
        let (choisi, indice) = self.choix(ch, face);
        let police = choisi.map_or(Face::ALL.len(), Face::index);
        let demi = DETOURAGE
            .iter()
            .map(|(a, b)| a.hypot(*b))
            .fold(0.0, f32::max);
        self.contours
            .detourer(pixmap, (police, indice), plume, (size, demi), couleurs);
        self.advance(ch, size, face)
    }

    /// **Le halo de `ch` pour une lettre de cette phase** (TITRE-2) : ses huit copies du
    /// détourage ([`DETOURAGE`]), chacune à sa phase comme n'importe quelle position, composées
    /// une fois et gardées dans le même cache que les lettres — la même éviction les borne.
    pub(super) fn halo_variant(
        &self,
        ch: char,
        size: f32,
        face: Face,
        phase: u8,
    ) -> Rc<GlyphEntry> {
        let size_key = (size * 10.0).round().clamp(1.0, 65535.0) as u16;
        let cle = (face, ch, size_key, glyph::HALO | phase);
        let image = self.image.get();
        if let Some((entry, vue)) = self.glyph_cache.borrow_mut().get_mut(&cle) {
            *vue = image;
            return entry.clone();
        }
        let n = i32::from(SUBPIXEL_PHASES);
        let (px, py) = (i32::from(phase) % n, i32::from(phase) / n);
        let copies: Vec<_> = DETOURAGE
            .iter()
            .map(|&(ox, oy)| {
                // Un décalage d'un demi-pixel est un nombre entier de phases : la copie tombe
                // à une cellule et une phase que celles de la lettre déterminent.
                let (qx, qy) = (px + (ox * n as f32) as i32, py + (oy * n as f32) as i32);
                let fine = (qy.rem_euclid(n) * n + qx.rem_euclid(n)) as u8;
                let e = self.glyph_variant(ch, size, face, fine);
                let m = e.metrics;
                let coin = (
                    qx.div_euclid(n) + m.xmin,
                    qy.div_euclid(n) - m.ymin - m.height as i32,
                );
                (e, coin)
            })
            .collect();
        let modele = self.glyph_variant(ch, size, face, PHASE_ORIGIN).metrics;
        let entry = Rc::new(glyph::halo(&copies, &modele));
        self.glyph_cache
            .borrow_mut()
            .insert(cle, (entry.clone(), image));
        entry
    }
}
