//! **Les commandes d'une vidéo sur son nœud** (VIDEO-1, fiche 169) : ▶ au milieu tant qu'elle ne
//! joue pas, et l'icône du son en bas à gauche dès qu'elle est lancée — coupé par défaut, barré
//! (sa réponse du 25/11). Une seule géométrie pour les dessiner et pour les prendre : ce qu'on voit
//! est ce qu'on touche.
//!
//! Elles se posent par-dessus tout, sur les deux voies, comme les ornements des photos
//! (ORNEMENTS-1) ; un nœud trop petit à l'écran n'en porte pas.

use crate::canvas::{screen_to_world, world_to_screen};
use crate::params::ViewPass;
use crate::renderer::magasin::Magasin;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Viewport};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Sous ce côté (en pixels d'écran), un nœud vidéo ne porte aucune commande : elles le
/// couvriraient.
const COTE_MIN: f32 = 48.0;

/// Ce qu'un clic touche sur un nœud vidéo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Touche {
    Lecture,
    Son,
}

/// **Où sont les commandes** d'un nœud dont la boîte à l'écran est `(x, y, l, h)`, non tournée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Commandes {
    centre: (f32, f32),
    rayon: f32,
    /// Le carré du son : son coin haut gauche et son côté.
    son: (f32, f32, f32),
}

impl Commandes {
    pub(crate) fn de((x, y, l, h): (f32, f32, f32, f32)) -> Option<Self> {
        let cote = l.min(h);
        if cote < COTE_MIN {
            return None;
        }
        let rayon = (cote * 0.12).clamp(14.0, 36.0);
        let s = (cote * 0.1).clamp(18.0, 30.0);
        let marge = s / 3.0;
        Some(Self {
            centre: (x + l / 2.0, y + h / 2.0),
            rayon,
            son: (x + marge, y + h - marge - s, s),
        })
    }

    /// Ce que touche ce point de l'écran (dans le repère du nœud non tourné), si elle est lancée.
    pub(crate) fn touche(&self, (px, py): (f32, f32), lancee: bool) -> Option<Touche> {
        let (sx, sy, s) = self.son;
        if lancee && px >= sx && px <= sx + s && py >= sy && py <= sy + s {
            return Some(Touche::Son);
        }
        let (dx, dy) = (px - self.centre.0, py - self.centre.1);
        (dx * dx + dy * dy <= self.rayon * self.rayon).then_some(Touche::Lecture)
    }
}

/// La boîte d'une image à l'écran (non tournée), et la rotation qui la tourne autour de son centre.
fn boite_a_l_ecran(img: &BoardImage, vp: &Viewport) -> (f32, f32, f32, f32) {
    let (x, y) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, vp);
    let (l, h) = (img.width * vp.scale, img.height * vp.scale);
    (x as f32, y as f32, l as f32, h as f32)
}

/// **Ce qu'un clic à ce point de l'écran touche sur ce nœud vidéo.** Le point est ramené dans le
/// repère du nœud : un nœud tourné se touche là où il se voit.
pub(crate) fn touche(
    img: &BoardImage,
    (px, py): (f64, f64),
    vp: &Viewport,
    lancee: bool,
) -> Option<Touche> {
    let boite = boite_a_l_ecran(img, vp);
    let commandes = Commandes::de(boite)?;
    let (cx, cy) = (boite.0 + boite.2 / 2.0, boite.1 + boite.3 / 2.0);
    let (s, c) = (-img.rotation).sin_cos();
    let (dx, dy) = (px - f64::from(cx), py - f64::from(cy));
    let local = (
        dx * c - dy * s + f64::from(cx),
        dx * s + dy * c + f64::from(cy),
    );
    commandes.touche((local.0 as f32, local.1 as f32), lancee)
}

/// **Pose les commandes des vidéos visibles.**
pub(crate) fn poser(
    pixmap: &mut PixmapMut,
    (store, pass): (&Store, ViewPass<'_>),
    magasin: &Magasin,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let images = Visibles::nouvelles(pass.visibles, board).images();
    for img in images.filter(|i| i.is_video) {
        let etat = img.src.as_deref().and_then(|s| magasin.etat_video(s));
        let boite = boite_a_l_ecran(img, &pass.vp);
        let Some(commandes) = Commandes::de(boite) else {
            continue;
        };
        let (cx, cy) = (boite.0 + boite.2 / 2.0, boite.1 + boite.3 / 2.0);
        let tourner = Transform::from_rotate_at(img.rotation.to_degrees() as f32, cx, cy);
        // Une commande que recouvre un autre nœud ne se dessine pas : elle ne se prend pas non
        // plus (le clic va au nœud du dessus).
        let libre = |(x, y): (f32, f32)| {
            let mut p = tiny_skia::Point::from_xy(x, y);
            tourner.map_points(std::slice::from_mut(&mut p));
            let monde = screen_to_world(f64::from(p.x), f64::from(p.y), &pass.vp);
            !board.couvre(
                &img.id,
                glucose_core::geometry::Point::new(monde.0, monde.1),
            )
        };
        if !etat.is_some_and(|e| e.joue) && libre(commandes.centre) {
            poser_le_bouton(pixmap, &commandes, tourner);
        }
        let (sx, sy, s) = commandes.son;
        if let Some(e) = etat.filter(|_| libre((sx + s / 2.0, sy + s / 2.0))) {
            poser_le_son(pixmap, &commandes, e.son, tourner);
        }
    }
}

fn peinture(r: u8, g: u8, b: u8, a: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(Color::from_rgba8(r, g, b, a));
    p.anti_alias = true;
    p
}

/// ▶ : un disque sombre, un triangle blanc — lisible sur n'importe quelle image.
fn poser_le_bouton(pixmap: &mut PixmapMut, c: &Commandes, tourner: Transform) {
    let ((x, y), r) = (c.centre, c.rayon);
    if let Some(disque) = PathBuilder::from_circle(x, y, r) {
        pixmap.fill_path(
            &disque,
            &peinture(0, 0, 0, 150),
            FillRule::Winding,
            tourner,
            None,
        );
    }
    let mut t = PathBuilder::new();
    t.move_to(x - r * 0.32, y - r * 0.48);
    t.line_to(x - r * 0.32, y + r * 0.48);
    t.line_to(x + r * 0.52, y);
    t.close();
    if let Some(t) = t.finish() {
        pixmap.fill_path(
            &t,
            &peinture(255, 255, 255, 235),
            FillRule::Winding,
            tourner,
            None,
        );
    }
}

/// Le haut-parleur : deux ondes quand le son joue, une croix quand il est coupé.
fn poser_le_son(pixmap: &mut PixmapMut, c: &Commandes, son: bool, tourner: Transform) {
    let (x, y, s) = c.son;
    if let Some(fond) = tiny_skia::Rect::from_xywh(x, y, s, s).map(PathBuilder::from_rect) {
        pixmap.fill_path(
            &fond,
            &peinture(0, 0, 0, 150),
            FillRule::Winding,
            tourner,
            None,
        );
    }
    let blanc = peinture(255, 255, 255, 235);
    let mut hp = PathBuilder::new();
    hp.move_to(x + 0.18 * s, y + 0.4 * s);
    hp.line_to(x + 0.34 * s, y + 0.4 * s);
    hp.line_to(x + 0.54 * s, y + 0.22 * s);
    hp.line_to(x + 0.54 * s, y + 0.78 * s);
    hp.line_to(x + 0.34 * s, y + 0.6 * s);
    hp.line_to(x + 0.18 * s, y + 0.6 * s);
    hp.close();
    if let Some(hp) = hp.finish() {
        pixmap.fill_path(&hp, &blanc, FillRule::Winding, tourner, None);
    }
    let trait_ = Stroke {
        width: (s * 0.08).max(1.5),
        line_cap: tiny_skia::LineCap::Round,
        ..Stroke::default()
    };
    let mut marque = PathBuilder::new();
    if son {
        for (r, h) in [(0.12, 0.14), (0.22, 0.24)] {
            marque.move_to(x + (0.6 + r * 0.4) * s, y + (0.5 - h) * s);
            marque.quad_to(
                x + (0.6 + r) * s,
                y + 0.5 * s,
                x + (0.6 + r * 0.4) * s,
                y + (0.5 + h) * s,
            );
        }
    } else {
        marque.move_to(x + 0.64 * s, y + 0.38 * s);
        marque.line_to(x + 0.86 * s, y + 0.62 * s);
        marque.move_to(x + 0.86 * s, y + 0.38 * s);
        marque.line_to(x + 0.64 * s, y + 0.62 * s);
    }
    if let Some(m) = marque.finish() {
        pixmap.stroke_path(&m, &blanc, &trait_, tourner, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_1_le_bouton_au_milieu_le_son_en_bas_a_gauche_une_fois_lancee() {
        let c = Commandes::de((100.0, 100.0, 400.0, 300.0)).expect("assez grand");
        assert_eq!(c.touche((300.0, 250.0), false), Some(Touche::Lecture));
        assert_eq!(c.touche((300.0, 250.0), true), Some(Touche::Lecture));
        assert_eq!(
            c.touche((120.0, 380.0), false),
            None,
            "pas lancée : pas de son"
        );
        assert_eq!(c.touche((120.0, 380.0), true), Some(Touche::Son));
        assert_eq!(c.touche((480.0, 120.0), true), None, "le reste est au nœud");
        assert!(
            Commandes::de((0.0, 0.0, 40.0, 300.0)).is_none(),
            "trop petit"
        );
    }

    #[test]
    fn video_1_un_noeud_tourne_se_touche_la_ou_il_se_voit() {
        let mut img = BoardImage::new("v", 0.0, 0.0, 400.0, 200.0);
        img.is_video = true;
        img.rotation = std::f64::consts::FRAC_PI_2;
        let vp = Viewport {
            x: 400.0,
            y: 300.0,
            scale: 1.0,
        };
        // Le centre de l'écran est celui du nœud : ▶ y reste.
        assert_eq!(
            touche(&img, (400.0, 300.0), &vp, true),
            Some(Touche::Lecture)
        );
        // Tourné d'un quart, son coin bas gauche passe en haut à gauche de l'écran.
        assert_eq!(touche(&img, (310.0, 115.0), &vp, true), Some(Touche::Son));
        assert_eq!(touche(&img, (215.0, 390.0), &vp, true), None);
    }
}
