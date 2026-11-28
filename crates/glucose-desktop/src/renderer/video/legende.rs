//! **La légende d'une affiche** (fiche 195) : une vidéo gardée à côté dont le fichier ne se lit
//! pas ici montre sa première image — et dit pourquoi elle ne joue pas, en bas de son nœud :
//! « Vidéo restée chez Alice », ou « Vidéo introuvable ici : v.mp4 ». Sans elle, l'affiche
//! cacherait ce que le cadre rouge d'ABSENT-1 disait.
//!
//! Elle se pose par-dessus, sur les deux voies, comme le ▶ ([`super::commandes`]) ; un nœud trop
//! petit pour elle, ou tourné, n'en porte pas — le menu « Retrouver le fichier… » reste.
use crate::params::ViewPass;
use crate::renderer::magasin::Magasin;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};

/// La taille de la légende, en pixels de l'écran, et la marge autour d'elle.
const TAILLE: f32 = 12.0;
const MARGE: f32 = 6.0;

/// **Pose la légende des affiches visibles.**
pub(crate) fn poser(
    pixmap: &mut PixmapMut,
    (store, pass): (&Store, ViewPass<'_>),
    magasin: &Magasin,
    typo: &Typography,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let images = Visibles::nouvelles(pass.visibles, board).images();
    for img in images.filter(|i| i.is_video && i.rotation == 0.0) {
        let Some(texte) = magasin.legende_de_l_affiche(img) else {
            continue;
        };
        let (x, y) = crate::canvas::world_to_screen(
            img.x - img.width / 2.0,
            img.y - img.height / 2.0,
            &pass.vp,
        );
        let (l, h) = (img.width * pass.vp.scale, img.height * pass.vp.scale);
        let (largeur, _) = typo.measure_text(&texte, TAILLE, Face::Regular);
        let bande = TAILLE + 2.0 * MARGE;
        if largeur + 2.0 * MARGE > l as f32 || 2.0 * bande > h as f32 {
            continue;
        }
        let haut = (y + h) as f32 - bande;
        if let Some(fond) = Rect::from_xywh(x as f32, haut, l as f32, bande) {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(0, 0, 0, 160));
            pixmap.fill_rect(fond, &p, Transform::identity(), None);
        }
        let style = TextStyle {
            size: TAILLE,
            color: Color::from_rgba8(255, 255, 255, 235),
            face: Face::Regular,
        };
        typo.draw_text(pixmap, &texte, x as f32 + MARGE, haut + MARGE, style);
    }
}
