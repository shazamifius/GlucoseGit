//! **GRILLE-2 — le trait d'une icône se pose sur la grille de l'écran** (fiche 120).
//!
//! Sa capture du 17/11 nommait aussi *« les logos »*. Les icônes des panneaux sont dessinées sur
//! une grille de 16 unités (celle de `Toolbar.tsx`, dans Glucose Tauri) et posées à 14 pixels :
//! chaque coordonnée entière du dessin tombe à un multiple de 0,875 pixel, et leurs traits de
//! 1,3 pixel couvrent toujours deux colonnes à moitié — une icône grise et floue, là où
//! Glucose Tauri, dans un navigateur, profitait de l'ajustement du système.
//!
//! La loi est celle des lettres (GRILLE-1), dite pour un trait : **un trait fait un nombre entier
//! de pixels, et son axe se pose là où ses bords tombent sur des frontières de pixels** — au
//! centre d'un pixel pour un trait impair, sur une frontière pour un trait pair. Chaque point du
//! tracé se pose ainsi à l'écran, puis revient dans les unités du dessin : les pointillés gardent
//! leurs longueurs, les courbes leurs extrémités sur la grille. Un remplissage pose ses points sur
//! les frontières. Aucun nombre choisi : la largeur visée est l'entier le plus proche, jamais zéro.

use tiny_skia::{Paint, Path, PathBuilder, PathSegment, PixmapMut, Point, Stroke, Transform};

/// **Trace `path` sur la grille** : la largeur du trait à l'écran arrondie au pixel (au moins un),
/// chaque point posé pour que les bords du trait tombent sur des frontières.
pub(super) fn tracer(
    pixmap: &mut PixmapMut,
    path: &Path,
    paint: &Paint,
    stroke: &Stroke,
    ts: Transform,
) {
    let k = echelle(ts);
    let largeur = (stroke.width * k).round().max(1.0);
    // Un trait impair a son axe au centre d'un pixel, un trait pair sur une frontière.
    let axe = if largeur % 2.0 == 1.0 { 0.5 } else { 0.0 };
    let Some(pose) = poser(path, ts, axe) else {
        return;
    };
    let sur_la_grille = Stroke {
        width: largeur / k,
        ..stroke.clone()
    };
    pixmap.stroke_path(&pose, paint, &sur_la_grille, ts, None);
}

/// **Remplit `path` sur la grille** : ses points posés sur les frontières des pixels.
pub(super) fn remplir(pixmap: &mut PixmapMut, path: &Path, paint: &Paint, ts: Transform) {
    if let Some(pose) = poser(path, ts, 0.0) {
        pixmap.fill_path(&pose, paint, tiny_skia::FillRule::Winding, ts, None);
    }
}

/// L'échelle du dessin à l'écran : les icônes ne se posent que par une échelle uniforme et une
/// translation.
fn echelle(ts: Transform) -> f32 {
    ts.sx
}

/// Le tracé dont chaque point, porté à l'écran par `ts`, tombe à `axe` d'une frontière de pixel —
/// exprimé de nouveau dans les unités du dessin.
fn poser(path: &Path, ts: Transform, axe: f32) -> Option<Path> {
    let k = echelle(ts);
    let sur = |v: f32, t: f32| ((v * k + t - axe).round() + axe - t) / k;
    let p = |q: Point| (sur(q.x, ts.tx), sur(q.y, ts.ty));
    let mut pb = PathBuilder::new();
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(a) => {
                let a = p(a);
                pb.move_to(a.0, a.1);
            }
            PathSegment::LineTo(a) => {
                let a = p(a);
                pb.line_to(a.0, a.1);
            }
            PathSegment::QuadTo(a, b) => {
                let (a, b) = (p(a), p(b));
                pb.quad_to(a.0, a.1, b.0, b.1);
            }
            PathSegment::CubicTo(a, b, c) => {
                let (a, b, c) = (p(a), p(b), p(c));
                pb.cubic_to(a.0, a.1, b.0, b.1, c.0, c.1);
            }
            PathSegment::Close => pb.close(),
        }
    }
    pb.finish()
}

#[cfg(test)]
#[path = "grille/tests.rs"]
mod tests;
