//! Les primitives de dessin des panneaux : une seule façon de remplir, de cerner, d'écrire.
//!
//! Chaque panneau dessinait ses rectangles arrondis en quinze lignes — un `Paint`, un
//! `PathBuilder`, un `push_rounded_rect`, un `finish`, un `fill_path`, puis la même chose
//! pour le contour — et ses boutons en trente. Cinq panneaux, la même chose cinq fois, avec
//! les mêmes tailles et les mêmes couleurs recopiées. Ce module est l'endroit unique.
//!
//! Le [`Brush`] garde ce qu'un dessin de panneau a constant : la typographie, le thème,
//! l'échelle, et la position du curseur — pour que le survol se lise d'un mot.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Ajoute un rectangle à coins arrondis dans un `PathBuilder`.
pub fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

/// L'aspect d'un bouton : ce qui, hors survol, décide de sa couleur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ButtonLook {
    /// Le bouton porte l'état courant (un tri choisi, une disposition retenue).
    pub active: bool,
    /// Le bouton fait quelque chose. Grisé sinon — jamais un toast qui simule (fiche 05
    /// § 5.4).
    pub disabled: bool,
    /// Le bouton est transparent au repos, et ne se colore qu'au survol ou actif.
    pub quiet: bool,
}

/// Ce que le dessin d'un panneau garde constant.
pub struct Brush<'a> {
    pub typo: &'a Typography,
    pub theme: &'a Theme,
    /// L'échelle d'interface, déjà bornée.
    pub s: f32,
    pub pointer: Pointer,
}

impl Brush<'_> {
    /// Une longueur de la fiche, en pixels d'écran.
    pub fn px(&self, logical: f32) -> f32 {
        logical * self.s
    }

    pub fn hovered(&self, rect: WidgetRect) -> bool {
        rect.contains(self.pointer.x, self.pointer.y)
    }

    pub fn fill(&self, pixmap: &mut PixmapMut, rect: WidgetRect, radius: f32, color: Color) {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return;
        }
        let mut pb = PathBuilder::new();
        push_rounded_rect(&mut pb, rect.x, rect.y, rect.w, rect.h, radius);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &paint(color),
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn stroke(
        &self,
        pixmap: &mut PixmapMut,
        rect: WidgetRect,
        radius: f32,
        color: Color,
        width: f32,
    ) {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return;
        }
        let mut pb = PathBuilder::new();
        push_rounded_rect(&mut pb, rect.x, rect.y, rect.w, rect.h, radius);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(
                &path,
                &paint(color),
                &line(width),
                Transform::identity(),
                None,
            );
        }
    }

    pub fn circle(&self, pixmap: &mut PixmapMut, center: (f32, f32), radius: f32, color: Color) {
        if radius <= 0.0 {
            return;
        }
        let mut pb = PathBuilder::new();
        pb.push_circle(center.0, center.1, radius);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &paint(color),
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn ring(
        &self,
        pixmap: &mut PixmapMut,
        center: (f32, f32),
        radius: f32,
        color: Color,
        width: f32,
    ) {
        if radius <= 0.0 {
            return;
        }
        let mut pb = PathBuilder::new();
        pb.push_circle(center.0, center.1, radius);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(
                &path,
                &paint(color),
                &line(width),
                Transform::identity(),
                None,
            );
        }
    }

    /// Écrit un texte ; `size` est un corps de la fiche, mis à l'échelle ici. Rend la plume.
    pub fn text(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        at: (f32, f32),
        size: f32,
        color: Color,
        face: Face,
    ) -> f32 {
        let style = TextStyle {
            size: self.px(size),
            color,
            face,
        };
        self.typo.draw_text(pixmap, text, at.0, at.1, style)
    }

    /// Écrit un texte centré dans un rectangle.
    pub fn text_centered(
        &self,
        pixmap: &mut PixmapMut,
        rect: WidgetRect,
        text: &str,
        size: f32,
        color: Color,
        face: Face,
    ) {
        let (w, line) = self.typo.measure_text(text, self.px(size), face);
        let at = (rect.x + (rect.w - w) / 2.0, rect.y + (rect.h - line) / 2.0);
        self.text(pixmap, text, at, size, color, face);
    }

    /// Le titre d'un panneau, en haut à gauche de son cadre.
    pub fn title(&self, pixmap: &mut PixmapMut, frame: ScaledRect, text: &str) {
        let at = (frame.x + self.px(14.0), frame.y + self.px(18.0));
        self.text(pixmap, text, at, 12.0, self.theme.text_primary, Face::Bold);
    }

    /// Un libellé de section, en capitales grises.
    pub fn caption(&self, pixmap: &mut PixmapMut, at: (f32, f32), text: &str) {
        self.text(pixmap, text, at, 9.5, self.theme.text_muted, Face::Bold);
    }

    /// Un bouton standard : fond, contour et libellé centré, selon son aspect et le survol.
    pub fn button(
        &self,
        pixmap: &mut PixmapMut,
        rect: WidgetRect,
        label: &str,
        size: f32,
        look: ButtonLook,
    ) {
        let theme = self.theme;
        let hovered = !look.disabled && self.hovered(rect);
        let bg = match (look.active, hovered, look.quiet) {
            (true, _, _) => Some(theme.bg_active),
            (false, true, _) => Some(theme.bg_hover),
            (false, false, true) => None,
            (false, false, false) => Some(theme.btn_bg),
        };
        if let Some(bg) = bg {
            self.fill(pixmap, rect, self.px(3.0), bg);
        }
        let border = if look.active {
            theme.border_accent
        } else {
            theme.btn_border
        };
        self.stroke(pixmap, rect, self.px(3.0), border, self.px(1.0));
        let ink = match (look.disabled, look.active) {
            (true, _) => theme.text_muted,
            (false, true) => theme.text_primary,
            (false, false) => theme.text_secondary,
        };
        self.text_centered(
            pixmap,
            rect,
            label,
            size,
            ink,
            if look.active {
                Face::Bold
            } else {
                Face::Regular
            },
        );
    }

    /// Un champ de valeur : un fond de saisie et sa valeur, alignée à gauche.
    pub fn field(&self, pixmap: &mut PixmapMut, rect: WidgetRect, value: &str) {
        self.fill(pixmap, rect, self.px(3.0), self.theme.input_bg);
        let at = (
            rect.x + self.px(8.0),
            rect.y + (rect.h - self.px(11.0) * 1.2) / 2.0,
        );
        self.text(
            pixmap,
            value,
            at,
            11.0,
            self.theme.text_primary,
            Face::Regular,
        );
    }

    /// Un bouton radio : un anneau, plein quand il est coché.
    pub fn radio(&self, pixmap: &mut PixmapMut, center: (f32, f32), checked: bool) {
        let theme = self.theme;
        let color = if checked {
            theme.text_primary
        } else {
            theme.border_medium
        };
        self.ring(pixmap, center, self.px(4.5), color, self.px(1.2));
        if checked {
            self.circle(pixmap, center, self.px(2.2), color);
        }
    }

    /// La poignée `⠿⠿` d'un panneau : deux colonnes de trois points, centrées.
    pub fn grip(&self, pixmap: &mut PixmapMut, center: (f32, f32), active: bool) {
        let color = if active {
            self.theme.dock_grip_active
        } else {
            self.theme.dock_grip_inactive
        };
        let (cx, cy) = center;
        let mut pb = PathBuilder::new();
        for dx in [-6.0, -2.0, 3.0, 7.0] {
            for dy in [-3.0, 0.0, 3.0] {
                pb.push_circle(cx + self.px(dx), cy + self.px(dy), self.px(0.9));
            }
        }
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &paint(color),
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
}

fn paint(color: Color) -> Paint<'static> {
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(color);
    paint
}

fn line(width: f32) -> Stroke {
    Stroke {
        width: width.max(0.5),
        ..Default::default()
    }
}
