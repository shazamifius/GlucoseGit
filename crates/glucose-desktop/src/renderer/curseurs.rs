//! **Les curseurs des autres** (fiche 117) : dans un canevas partagé, la souris de chaque autre
//! participant, à sa place dans le monde, de sa couleur, son nom à côté — on voit où il travaille
//! avant même qu'il ait posé quoi que ce soit (Figma, tldraw ; le `PeerCursorsLayer` de Glucose
//! Tauri). La couleur **dit qui** : c'est la sienne, choisie parmi les dix de Tauri.
//!
//! Par-dessus tout, comme un message de l'interface : un curseur n'est pas dans le document. Il
//! garde sa taille d'écran à tout zoom.

use super::PaintKit;
use crate::canvas::world_to_screen;
use crate::params::CurseurDuPair;
use crate::typography::{Face, TextStyle};
use glucose_core::types::Viewport;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// La flèche d'un pointeur, sa pointe à l'origine, en points.
const FLECHE: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 17.0),
    (4.5, 13.0),
    (8.0, 20.0),
    (10.5, 19.0),
    (7.0, 12.0),
    (12.5, 12.0),
];

/// Pose le curseur de chaque autre participant.
pub(super) fn dessiner_les_curseurs(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    curseurs: &[CurseurDuPair],
    (vp, echelle_ui): (&Viewport, f32),
) {
    let s = crate::theme::clamp_ui_scale(echelle_ui);
    for c in curseurs {
        let (sx, sy) = world_to_screen(c.monde.0, c.monde.1, vp);
        let (x, y) = (sx as f32, sy as f32);
        let [r, g, b] = c.couleur;
        let mut sienne = Paint::default();
        sienne.set_color(Color::from_rgba8(r, g, b, 255));
        sienne.anti_alias = true;
        let mut sombre = Paint::default();
        sombre.set_color(kit.theme.bg_canvas);
        sombre.anti_alias = true;
        if let Some(fleche) = fleche(x, y, s) {
            pixmap.fill_path(
                &fleche,
                &sienne,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
            let trait_ = Stroke {
                width: 1.2 * s,
                ..Default::default()
            };
            pixmap.stroke_path(&fleche, &sombre, &trait_, Transform::identity(), None);
        }
        let corps = 12.0 * s;
        let (tw, _) = kit.typography.measure_text(&c.nom, corps, Face::Regular);
        let (w, h) = (tw + 14.0 * s, 20.0 * s);
        let (px, py) = (x + 10.0 * s, y + 20.0 * s);
        if let Some(pilule) = crate::ui::toast::pilule(px, py, w, h, h / 2.0) {
            pixmap.fill_path(
                &pilule,
                &sienne,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
        kit.typography.draw_text(
            pixmap,
            &c.nom,
            px + 7.0 * s,
            py + (h - corps) / 2.0,
            TextStyle {
                size: corps,
                color: kit.theme.bg_canvas,
                face: Face::Regular,
            },
        );
    }
}

fn fleche(x: f32, y: f32, s: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let [(x0, y0), reste @ ..] = FLECHE;
    pb.move_to(x + x0 * s, y + y0 * s);
    for (dx, dy) in reste {
        pb.line_to(x + dx * s, y + dy * s);
    }
    pb.close();
    pb.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Un curseur se pose là où est la souris de l'autre**, de sa couleur : la pointe au point du
    /// monde, et sa pastille dessous.
    #[test]
    fn un_curseur_se_pose_a_sa_place_de_sa_couleur() {
        let mut pixmap = tiny_skia::Pixmap::new(300, 200).expect("pixmap");
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
        let typography = crate::typography::Typography::new();
        let math = crate::renderer::math::MathRenderer::new();
        let tints = crate::renderer::DomainTints::default();
        let theme = crate::theme::Theme::dark();
        let kit = PaintKit {
            typography: &typography,
            math: &math,
            mises_en_page: &Default::default(),
            tints: &tints,
            theme: &theme,
            sens: &Default::default(),
        };
        let vp = Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
        let curseurs = [CurseurDuPair {
            monde: (100.0, 50.0),
            couleur: [0xfb, 0x92, 0x3c],
            nom: "Chat Orange".into(),
        }];
        dessiner_les_curseurs(kit, &mut pixmap.as_mut(), &curseurs, (&vp, 1.0));
        let orange = |x: u32, y: u32| {
            let p = pixmap.pixel(x, y).expect("dans l'image");
            (p.red(), p.green(), p.blue()) == (0xfb, 0x92, 0x3c)
        };
        assert!(orange(103, 60), "la flèche, juste sous la pointe");
        assert!(orange(112, 80), "la pastille du nom");
        assert!(!orange(90, 40), "rien au-dessus et à gauche de la pointe");
    }
}
