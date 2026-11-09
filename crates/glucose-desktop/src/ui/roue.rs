//! **ROUE-1 — la roue de couleur** (fiche 99) : au-dessus de la barre de couleur, un disque
//! teinte × saturation et un curseur de clarté — le sélecteur de Blender, dans Oklch
//! ([`glucose_core::couleur::roue`]). Sa parole du 9/11/2026 : *« le même système de palette que
//! Blender, avec la roue — elle est extrêmement ergonomique »*.
//!
//! # La mise en page est une fonction pure
//!
//! [`placer`] rend les rectangles sans rien dessiner ; le dessin et le clic lisent la même mise en
//! page (loi L4), comme la barre sous elle.

use super::action_bar::{dans, fond_arrondi, pastille};
use crate::interactions::roue::RoueOuverte;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::couleur::roue::{point_de, PlaceDansLaRoue};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, PixmapPaint, Stroke, Transform};

/// Les mesures de la roue, à l'échelle un.
const MARGE: f32 = 12.0;
const DIAMETRE: f32 = 168.0;
const ECART: f32 = 12.0;
const CURSEUR: f32 = 18.0;
const APERCU: f32 = 24.0;
/// L'écart entre la roue et la barre de couleur qu'elle surmonte.
const AU_DESSUS: f32 = 8.0;

/// La roue, une fois placée : rectangles `(x, y, largeur, hauteur)` à l'écran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoueAPlat {
    pub rect: (f32, f32, f32, f32),
    /// Le disque : son coin haut-gauche et son diamètre (un nombre entier de pixels).
    pub disque: (f32, f32, f32),
    pub curseur: (f32, f32, f32, f32),
    /// La couleur d'avant la roue, et celle qu'elle pose.
    pub avant: (f32, f32, f32, f32),
    pub apres: (f32, f32, f32, f32),
    /// Où s'écrit la couleur, en `#rrggbb`.
    pub ecrit: (f32, f32),
}

/// **Le diamètre du disque**, en pixels entiers : sa texture ne se pose qu'au pixel.
pub fn diametre(scale: f32) -> u32 {
    (DIAMETRE * crate::theme::clamp_ui_scale(scale)).round() as u32
}

/// **La roue au-dessus de la barre `sous`**, centrée sur elle et tenue dans l'écran.
pub fn placer(sous: (f32, f32, f32, f32), screen: (f32, f32), scale: f32) -> RoueAPlat {
    let s = crate::theme::clamp_ui_scale(scale);
    let d = diametre(scale) as f32;
    let largeur = MARGE * s * 2.0 + d + ECART * s + CURSEUR * s;
    let hauteur = MARGE * s * 2.0 + d + ECART * s + APERCU * s;
    let x = (sous.0 + sous.2 / 2.0 - largeur / 2.0).clamp(0.0, (screen.0 - largeur).max(0.0));
    let y = (sous.1 - AU_DESSUS * s - hauteur).max(0.0);
    let (gauche, haut) = ((x + MARGE * s).round(), (y + MARGE * s).round());
    let bas = haut + d + ECART * s;
    let apercu = APERCU * s;
    RoueAPlat {
        rect: (x, y, largeur, hauteur),
        disque: (gauche, haut, d),
        curseur: (gauche + d + ECART * s, haut, CURSEUR * s, d),
        avant: (gauche, bas, apercu * 1.5, apercu),
        apres: (gauche + apercu * 1.5, bas, apercu * 1.5, apercu),
        ecrit: (gauche + apercu * 3.0 + ECART * s, bas),
    }
}

/// Ce qu'un point de l'écran touche dans la roue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endroit {
    /// Le disque : le point, depuis son centre, en fractions de son rayon.
    Disque(f64, f64),
    /// Le curseur : la clarté sous le point, de 0 (en bas) à 1 (en haut).
    Clarte(f64),
    /// La roue, ailleurs que sur ces deux-là.
    Ailleurs,
}

/// **Ce qu'un clic en `(px, py)` touche**, ou `None` hors de la roue.
pub fn endroit(roue: &RoueAPlat, px: f32, py: f32) -> Option<Endroit> {
    if !dans(roue.rect, px, py) {
        return None;
    }
    let (x, y, d) = roue.disque;
    let r = d / 2.0;
    let (dx, dy) = (f64::from((px - x - r) / r), f64::from((py - y - r) / r));
    if dx.hypot(dy) <= 1.0 {
        return Some(Endroit::Disque(dx, dy));
    }
    if dans(roue.curseur, px, py) {
        let (_, cy, _, ch) = roue.curseur;
        return Some(Endroit::Clarte(
            f64::from(1.0 - (py - cy) / ch).clamp(0.0, 1.0),
        ));
    }
    Some(Endroit::Ailleurs)
}

/// **Dessine la roue** ouverte au-dessus de la barre `sous`.
pub fn draw_roue(
    pixmap: &mut PixmapMut,
    roue: &RoueOuverte,
    sous: (f32, f32, f32, f32),
    (typography, theme): (&Typography, &Theme),
    (screen, scale): ((f32, f32), f32),
) {
    let s = crate::theme::clamp_ui_scale(scale);
    let plat = placer(sous, screen, scale);
    pastille(pixmap, plat.rect, s, theme);
    let (x, y, d) = plat.disque;
    pixmap.draw_pixmap(
        x as i32,
        y as i32,
        roue.disque.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    let (px, py) = point_de(roue.place);
    let r = d / 2.0;
    anneau(pixmap, (x + r + px as f32 * r, y + r + py as f32 * r), s);
    dessiner_le_curseur(pixmap, plat.curseur, roue.place, s);
    apercu(pixmap, plat.avant, roue.avant, theme);
    apercu(pixmap, plat.apres, roue.apres, theme);
    let ecrit = roue.apres.map_or_else(
        || super::couleurs::PAR_DEFAUT.to_string(),
        glucose_core::couleur::roue::hex,
    );
    let corps = 12.0 * s;
    let (ex, ey) = plat.ecrit;
    let style = TextStyle {
        size: corps,
        color: theme.text_secondary,
        face: Face::Regular,
    };
    typography.draw_text(pixmap, &ecrit, ex, ey + (APERCU * s - corps) / 2.0, style);
}

/// Le curseur de clarté : la couleur de la place à chaque clarté, de 1 en haut à 0 en bas, et
/// l'encoche de la clarté choisie.
fn dessiner_le_curseur(
    pixmap: &mut PixmapMut,
    (cx, cy, cw, ch): (f32, f32, f32, f32),
    place: PlaceDansLaRoue,
    s: f32,
) {
    let lignes = ch.round().max(1.0) as u32;
    for i in 0..lignes {
        let clarte = 1.0 - (f64::from(i) + 0.5) / f64::from(lignes);
        let (r, g, b) = PlaceDansLaRoue { clarte, ..place }.couleur().octets();
        let rect = tiny_skia::Rect::from_xywh(cx, cy + i as f32, cw, 1.0);
        if let Some(rect) = rect {
            crate::renderer::scale::fill_crisp(pixmap, rect, Color::from_rgba8(r, g, b, 255));
        }
    }
    let y = cy + (1.0 - place.clarte as f32) * ch;
    for (epaisseur, couleur) in [(3.0 * s, (20, 20, 20)), (1.0 * s, (240, 240, 240))] {
        let rect = tiny_skia::Rect::from_xywh(cx - s, y - epaisseur / 2.0, cw + 2.0 * s, epaisseur);
        if let Some(rect) = rect {
            let (r, g, b) = couleur;
            crate::renderer::scale::fill_crisp(pixmap, rect, Color::from_rgba8(r, g, b, 255));
        }
    }
}

/// L'anneau qui marque la place sur le disque : clair, bordé de sombre, lisible sur toute teinte.
fn anneau(pixmap: &mut PixmapMut, (x, y): (f32, f32), s: f32) {
    let Some(cercle) = PathBuilder::from_circle(x, y, 5.0 * s) else {
        return;
    };
    for (largeur, (r, g, b)) in [(3.5 * s, (20, 20, 20)), (1.5 * s, (240, 240, 240))] {
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, 255);
        paint.anti_alias = true;
        let trait_ = Stroke {
            width: largeur,
            ..Stroke::default()
        };
        pixmap.stroke_path(&cercle, &paint, &trait_, Transform::identity(), None);
    }
}

/// Un aperçu : la couleur, ou « par défaut » — le panneau, barré.
fn apercu(
    pixmap: &mut PixmapMut,
    rect: (f32, f32, f32, f32),
    couleur: Option<(u8, u8, u8)>,
    theme: &Theme,
) {
    match couleur {
        Some((r, g, b)) => fond_arrondi(pixmap, rect, 3.0, Color::from_rgba8(r, g, b, 255), None),
        None => fond_arrondi(pixmap, rect, 3.0, theme.bg_panel, Some(theme.text_muted)),
    }
}
