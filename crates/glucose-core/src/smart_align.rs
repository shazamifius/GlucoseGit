//! SNAP-1 — Alignement intelligent : MOTEUR PUR (0 dépendance).

use crate::types::{Annotation, Board};
use std::collections::HashSet;

/// La boîte que le magnétisme aligne : celle du modèle, et aucune autre. Le nom survit parce
/// que les appelants et les tests le connaissent ; le type est [`crate::geometry::Rect`].
pub type AlignRect = crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignKind {
    Image,
    Text,
    Sticky,
    Membrane,
    Folder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignTarget {
    pub id: String,
    pub kind: AlignKind,
    pub rect: AlignRect,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SnapGuides {
    pub x: Option<Vec<f64>>,
    pub y: Option<Vec<f64>>,
}

pub const SNAP_SCREEN_PX: f64 = 8.0;
#[derive(Debug, Clone, Copy)]
pub struct SnapOptions {
    pub scale: f64,
    pub threshold_px: f64,
    pub axis_x: bool,
    pub axis_y: bool,
}

impl Default for SnapOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            threshold_px: SNAP_SCREEN_PX,
            axis_x: true,
            axis_y: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveSnap {
    pub dx: f64,
    pub dy: f64,
    pub guides: SnapGuides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResizeSnap {
    pub rect: AlignRect,
    pub guides: SnapGuides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointSnap {
    pub x: f64,
    pub y: f64,
    pub guides: SnapGuides,
}

pub fn union_rect(rects: &[AlignRect]) -> Option<AlignRect> {
    if rects.is_empty() {
        return None;
    }
    let mut l = f64::INFINITY;
    let mut t = f64::INFINITY;
    let mut r = f64::NEG_INFINITY;
    let mut b = f64::NEG_INFINITY;
    for rc in rects {
        l = l.min(rc.left);
        t = t.min(rc.top);
        r = r.max(rc.left + rc.width);
        b = b.max(rc.top + rc.height);
    }
    Some(AlignRect {
        left: l,
        top: t,
        width: r - l,
        height: b - t,
    })
}

pub fn collect_align_targets(board: &Board, exclude: &HashSet<String>) -> Vec<AlignTarget> {
    let mut out = Vec::new();

    for img in &board.images {
        if exclude.contains(&img.id) {
            continue;
        }
        out.push(AlignTarget {
            id: img.id.clone(),
            kind: AlignKind::Image,
            rect: img.rect(),
        });
    }

    for ann in &board.annotations {
        if exclude.contains(ann.id()) {
            continue;
        }
        if let Some(rect) = ann.rect() {
            let kind = match ann {
                Annotation::Membrane { .. } => AlignKind::Membrane,
                Annotation::Sticky { .. } => AlignKind::Sticky,
                Annotation::Text { .. } => AlignKind::Text,
                Annotation::Arrow { .. } => unreachable!(),
            };
            out.push(AlignTarget {
                id: ann.id().to_string(),
                kind,
                rect,
            });
        }
    }

    for f in &board.folders {
        if exclude.contains(&f.id) {
            continue;
        }
        out.push(AlignTarget {
            id: f.id.clone(),
            kind: AlignKind::Folder,
            rect: f.rect(),
        });
    }

    out
}

fn target_lines_x(rect: AlignRect) -> [f64; 3] {
    [
        rect.left,
        rect.left + rect.width / 2.0,
        rect.left + rect.width,
    ]
}

fn target_lines_y(rect: AlignRect) -> [f64; 3] {
    [
        rect.top,
        rect.top + rect.height / 2.0,
        rect.top + rect.height,
    ]
}

#[derive(Debug, Clone, Copy)]
struct AxisSnap {
    delta: f64,
    line: f64,
}

/// **Les lignes d'alignement des cibles, triées une fois** (SNAP-2).
///
/// L'aimant comparait le rectangle glissé aux trois lignes de **chaque** cible, et rebâtissait
/// ces lignes à chaque mouvement de la main : trois fois le tableau par mouvement, qui ne
/// tiendrait pas à dix millions de nœuds (fiche 41 § 11). Les cibles ne changent pas pendant un
/// geste : leurs lignes se trient une fois, et chaque mouvement cherche la plus proche par
/// dichotomie — la même réponse, en un logarithme.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Lignes {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl Lignes {
    /// Aucune ligne : l'aimant ne prend rien.
    pub const AUCUNE: Self = Self {
        x: Vec::new(),
        y: Vec::new(),
    };

    /// Les lignes de ces cibles : bords et milieux, sur chaque axe, triés.
    pub fn des(targets: &[AlignTarget]) -> Self {
        let mut x: Vec<f64> = targets
            .iter()
            .flat_map(|t| target_lines_x(t.rect))
            .collect();
        let mut y: Vec<f64> = targets
            .iter()
            .flat_map(|t| target_lines_y(t.rect))
            .collect();
        x.sort_by(f64::total_cmp);
        y.sort_by(f64::total_cmp);
        Self { x, y }
    }
}

/// La ligne la plus proche de `m` parmi des lignes triées — à égalité, la plus petite : la
/// réponse ne dépend pas de l'ordre des nœuds dans le tableau.
fn plus_proche(lignes: &[f64], m: f64) -> Option<f64> {
    let i = lignes.partition_point(|&t| t < m);
    let dessous = i.checked_sub(1).map(|k| lignes[k]);
    match (dessous, lignes.get(i).copied()) {
        (Some(b), Some(h)) => Some(if h - m < m - b { h } else { b }),
        (b, h) => b.or(h),
    }
}

/// Le plus petit écart entre une de mes lignes et une ligne des cibles, sous le seuil — la
/// première des miennes l'emporte à égalité.
fn best_axis_snap(mine: &[f64], theirs: &[f64], threshold: f64) -> Option<AxisSnap> {
    let mut best = None;
    let mut best_dist = threshold;
    for &m in mine {
        let Some(t) = plus_proche(theirs, m) else {
            continue;
        };
        let d = t - m;
        if d.abs() < best_dist {
            best_dist = d.abs();
            best = Some(AxisSnap { delta: d, line: t });
        }
    }
    best
}

fn threshold_of(opts: &SnapOptions) -> f64 {
    let scale = if opts.scale > 0.0 { opts.scale } else { 1.0 };
    opts.threshold_px / scale
}

pub fn snap_move(rect: AlignRect, targets: &[AlignTarget], opts: SnapOptions) -> MoveSnap {
    snap_move_sur(rect, &Lignes::des(targets), opts)
}

/// [`snap_move`] sur des lignes déjà triées : ce qu'un geste appelle à chaque mouvement.
pub fn snap_move_sur(rect: AlignRect, lignes: &Lignes, opts: SnapOptions) -> MoveSnap {
    let threshold = threshold_of(&opts);
    let cx = rect.left + rect.width / 2.0;
    let cy = rect.top + rect.height / 2.0;
    let mine_x = [cx, rect.left, rect.left + rect.width];
    let mine_y = [cy, rect.top, rect.top + rect.height];

    let sx = if opts.axis_x {
        best_axis_snap(&mine_x, &lignes.x, threshold)
    } else {
        None
    };
    let sy = if opts.axis_y {
        best_axis_snap(&mine_y, &lignes.y, threshold)
    } else {
        None
    };

    MoveSnap {
        dx: sx.map_or(0.0, |s| s.delta),
        dy: sy.map_or(0.0, |s| s.delta),
        guides: SnapGuides {
            x: sx.map(|s| vec![s.line]),
            y: sy.map(|s| vec![s.line]),
        },
    }
}

pub fn snap_resize(
    rect: AlignRect,
    handle: &str,
    targets: &[AlignTarget],
    opts: SnapOptions,
    min_width: f64,
    min_height: f64,
) -> ResizeSnap {
    let lignes = Lignes::des(targets);
    snap_resize_sur(rect, handle, &lignes, opts, (min_width, min_height))
}

/// [`snap_resize`] sur des lignes déjà triées : ce qu'un geste appelle à chaque mouvement.
pub fn snap_resize_sur(
    rect: AlignRect,
    handle: &str,
    lignes: &Lignes,
    opts: SnapOptions,
    (min_width, min_height): (f64, f64),
) -> ResizeSnap {
    let threshold = threshold_of(&opts);
    let mut guides = SnapGuides::default();
    let (mut left, mut width) = (rect.left, rect.width);
    if opts.axis_x {
        let tire = (handle.contains('l'), handle.contains('r'));
        let pris = aimanter_un_axe((left, width), tire, &lignes.x, (threshold, min_width));
        if let Some((debut, taille, ligne)) = pris {
            (left, width, guides.x) = (debut, taille, Some(vec![ligne]));
        }
    }
    let (mut top, mut height) = (rect.top, rect.height);
    if opts.axis_y {
        let tire = (handle.contains('t'), handle.contains('b'));
        let pris = aimanter_un_axe((top, height), tire, &lignes.y, (threshold, min_height));
        if let Some((debut, taille, ligne)) = pris {
            (top, height, guides.y) = (debut, taille, Some(vec![ligne]));
        }
    }
    ResizeSnap {
        rect: AlignRect {
            left,
            top,
            width,
            height,
        },
        guides,
    }
}

/// **Aimante un axe d'un redimensionnement** : le bord tiré — le début si `tire.0`, la fin si
/// `tire.1` — va sur la ligne la plus proche sous le seuil, sauf si la taille tomberait sous
/// son minimum. Rend le nouveau début, la nouvelle taille et la ligne, ou rien.
fn aimanter_un_axe(
    (debut, taille): (f64, f64),
    (tire_le_debut, tire_la_fin): (bool, bool),
    lignes: &[f64],
    (seuil, minimum): (f64, f64),
) -> Option<(f64, f64, f64)> {
    let minimum = minimum.max(1.0);
    let mut miennes = Vec::with_capacity(2);
    if tire_le_debut {
        miennes.push(debut);
    }
    if tire_la_fin {
        miennes.push(debut + taille);
    }
    let s = best_axis_snap(&miennes, lignes, seuil)?;
    let (nouveau_debut, nouvelle_taille) = if tire_le_debut {
        (s.line, debut + taille - s.line)
    } else {
        (debut, s.line - debut)
    };
    (nouvelle_taille >= minimum).then_some((nouveau_debut, nouvelle_taille, s.line))
}

pub fn snap_point(x: f64, y: f64, targets: &[AlignTarget], opts: SnapOptions) -> PointSnap {
    let r = snap_move(AlignRect::new(x, y, 0.0, 0.0), targets, opts);
    PointSnap {
        x: x + r.dx,
        y: y + r.dy,
        guides: r.guides,
    }
}

pub fn same_guides(a: Option<&SnapGuides>, b: Option<&SnapGuides>) -> bool {
    let ax = a.and_then(|g| g.x.as_deref()).unwrap_or(&[]);
    let ay = a.and_then(|g| g.y.as_deref()).unwrap_or(&[]);
    let bx = b.and_then(|g| g.x.as_deref()).unwrap_or(&[]);
    let by = b.and_then(|g| g.y.as_deref()).unwrap_or(&[]);
    ax == bx && ay == by
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_target() -> AlignTarget {
        AlignTarget {
            id: "ref".into(),
            kind: AlignKind::Text,
            rect: AlignRect::new(0.0, 0.0, 100.0, 100.0),
        }
    }

    #[test]
    fn test_snap_move_left_edge() {
        let r = snap_move(
            AlignRect::new(3.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert!((r.dx - (-3.0)).abs() < 1e-6);
        assert_eq!(r.guides.x, Some(vec![0.0]));
    }

    #[test]
    fn test_snap_move_centers() {
        // centre cible = 50, boîte left=27 width=40 => centre = 47 => dx = +3
        let r = snap_move(
            AlignRect::new(27.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert!((r.dx - 3.0).abs() < 1e-6);
        assert_eq!(r.guides.x, Some(vec![50.0]));
    }

    #[test]
    fn test_snap_beyond_threshold() {
        let r = snap_move(
            AlignRect::new(40.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert_eq!(r.dx, 0.0);
        assert_eq!(r.guides.x, None);
    }

    #[test]
    fn test_snap_scale_threshold() {
        let t = [ref_target()];
        // scale 0.1 -> threshold = 8 / 0.1 = 80 unités monde -> 20 unités accroche
        let p1 = snap_point(
            20.0,
            500.0,
            &t,
            SnapOptions {
                scale: 0.1,
                ..Default::default()
            },
        );
        assert!((p1.x - 0.0).abs() < 1e-6);

        // scale 4.0 -> threshold = 8 / 4 = 2 unités monde -> 20 unités n'accroche pas
        let p2 = snap_point(
            20.0,
            500.0,
            &t,
            SnapOptions {
                scale: 4.0,
                ..Default::default()
            },
        );
        assert_eq!(p2.x, 20.0);
    }
}
