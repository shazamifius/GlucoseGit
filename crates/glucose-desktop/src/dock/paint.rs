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
use crate::renderer::push_rounded_rect;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

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
    /// Le pointeur ne se lit que par [`Brush::hovered`] : c'est ce qui permet de savoir, sans
    /// rien deviner, de quoi le dessin dépend (SURVOL-2).
    pointer: Pointer,
    /// **Les questions de survol que ce dessin a posées**, dans l'ordre (SURVOL-2).
    ///
    /// Un panneau ne dépend du pointeur qu'à travers elles : tant que leurs réponses sont les
    /// mêmes, ses pixels le sont aussi. Le cache les repose au pointeur suivant au lieu de
    /// comparer des positions — la clé gardait la position exacte, et chaque pixel de
    /// mouvement au-dessus d'un panneau le redessinait entier : 76 fois en 35 s, cinq
    /// millisecondes chaque fois, sur la session de l'utilisateur du 24/09.
    questions: std::cell::RefCell<Vec<WidgetRect>>,
    /// Le coin haut-gauche du tampon visé, en coordonnées **écran** (DOCK-CACHE-1).
    ///
    /// Tout le dock se dessine en coordonnées écran — c'est ce qui permet à `hovered` de
    /// comparer un rectangle à la position du pointeur sans conversion. Mais un panneau mis
    /// en cache se dessine dans un tampon à lui, dont le coin n'est pas l'origine de l'écran.
    ///
    /// Plutôt que de convertir les coordonnées dans chaque panneau — six fichiers, des
    /// dizaines de rectangles, et une occasion d'en oublier un — la conversion vit **ici**,
    /// au seul endroit où le dessin touche le tampon. Les panneaux continuent de parler en
    /// coordonnées écran et ne savent pas qu'ils sont mis en cache.
    ///
    /// **Elle est entière, et ce n'est pas un détail** : une translation fractionnaire
    /// décalerait la couverture de l'anti-crénelage, et le panneau ne serait plus rendu au
    /// même pixel près qu'avant le cache. La fraction sous-pixel reste donc dans les
    /// coordonnées, qui ne bougent pas ; seul un entier est retiré.
    pub origin: (f32, f32),
}

impl<'a> Brush<'a> {
    pub fn nouveau(
        (typo, theme): (&'a Typography, &'a Theme),
        s: f32,
        pointer: Pointer,
        origin: (f32, f32),
    ) -> Self {
        Self {
            typo,
            theme,
            s,
            pointer,
            origin,
            questions: Default::default(),
        }
    }

    /// **Les questions de survol posées, et ce que le pointeur y répondait** — et leur oubli.
    pub(super) fn prendre_le_survol(&self) -> Vec<(WidgetRect, bool)> {
        self.questions
            .take()
            .into_iter()
            .map(|r| (r, r.contains(self.pointer.x, self.pointer.y)))
            .collect()
    }

    /// Une longueur de la fiche, en pixels d'écran.
    pub fn px(&self, logical: f32) -> f32 {
        logical * self.s
    }

    /// Le pointeur est-il sur ce rectangle ? La question est notée (SURVOL-2).
    pub fn hovered(&self, rect: WidgetRect) -> bool {
        self.questions.borrow_mut().push(rect);
        rect.contains(self.pointer.x, self.pointer.y)
    }

    /// La translation de l'écran vers le tampon visé (DOCK-CACHE-1).
    fn shift(&self) -> Transform {
        Transform::from_translate(-self.origin.0, -self.origin.1)
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
                self.shift(),
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
            pixmap.stroke_path(&path, &paint(color), &line(width), self.shift(), None);
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
                self.shift(),
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
            pixmap.stroke_path(&path, &paint(color), &line(width), self.shift(), None);
        }
    }

    /// Écrit un texte dont le style est **déjà** à l'échelle. Rend la plume.
    ///
    /// C'est le seul passage du dock vers la typographie, et donc le seul endroit où
    /// l'origine du tampon s'applique au texte (DOCK-CACHE-1). Un panneau qui appellerait
    /// `Typography::draw_text` directement écrirait en coordonnées d'écran dans un tampon
    /// local, et son texte atterrirait ailleurs — le cliquet `cliquets_suite` interdit ce
    /// contournement, parce qu'il a déjà eu lieu.
    pub fn text_styled(
        &self,
        pixmap: &mut PixmapMut,
        text: &str,
        x: f32,
        y: f32,
        style: TextStyle,
    ) -> f32 {
        self.typo.draw_text_offset(
            pixmap,
            text,
            x,
            y,
            (-self.origin.0 as i32, -self.origin.1 as i32),
            style,
        )
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
        self.text_styled(pixmap, text, at.0, at.1, style)
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
                self.shift(),
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
