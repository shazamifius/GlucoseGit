//! Les passes de scène qui ne sont pas des annotations : grille, membranes, images,
//! guides d'alignement et boîte de sélection élastique.
//!
//! Les membranes suivent la même règle que les cartes (SCALE-1) : leur rayon, leurs halos,
//! leur cadre, leur pointillé et leur titre viennent d'une seule mise à l'échelle. Avant
//! R-45, le rayon était `clamp(4, 60)`, le décalage du titre `clamp(8, 20)` et
//! `clamp(12, 24)`, et sa police `clamp(12, 22)` — quatre seuils pour un seul cadre, plus
//! un pointillé de 10 px qui ne suivait pas le zoom du tout.
//!
//! Les guides, les poignées et les points de la grille, eux, gardent une **taille écran
//! constante** : c'est l'exception de SCALE-1. Les poignées elles-mêmes sont dessinées par
//! [`super::handles`], aux positions que le test de clic utilise (RESIZE-1).

pub(super) mod grid;
pub(super) mod image;

pub(super) use image::draw_images;

use super::domain::{draw_domain_gauge, gauge_width};
use super::handles::draw_resize_handles;
use super::pass::{Clip, SELECTION_RING};
use super::scale::WorldScale;
use super::{parse_hex_color, push_rounded_rect, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle};
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::{Color, LineCap, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Rayon des coins d'une membrane, en unités monde.
const MEMBRANE_RADIUS: f32 = 60.0;
/// Débords des deux couches de halo d'une membrane, en unités monde.
const MEMBRANE_GLOW: [(f32, u8); 2] = [(20.0, 8), (10.0, 14)];
/// Décalage horizontal du titre depuis le bord gauche, en unités monde.
const MEMBRANE_LABEL_DX: f32 = 16.0;
/// Élévation du titre au-dessus du bord haut, en unités monde.
const MEMBRANE_LABEL_DY: f32 = 18.0;
/// Corps du titre d'une membrane, en unités monde.
const MEMBRANE_LABEL_FONT: f32 = 16.0;
/// Épaisseur du cadre d'une membrane, en unités monde.
const MEMBRANE_BORDER: f32 = 2.0;
/// Longueur d'un tiret du pointillé, en unités monde.
const MEMBRANE_DASH: f32 = 10.0;

// ── Membranes ───────────────────────────────────────────────────────────────

/// Mise en page d'une membrane. **En unités monde tant que `scaled` n'a pas été appelée.**
#[derive(Clone, Copy)]
struct MembraneLayout {
    width: f32,
    height: f32,
    radius: f32,
    label_dx: f32,
    label_dy: f32,
    label_font: f32,
    border: f32,
    dash: f32,
}

impl MembraneLayout {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            radius: MEMBRANE_RADIUS,
            label_dx: MEMBRANE_LABEL_DX,
            label_dy: MEMBRANE_LABEL_DY,
            label_font: MEMBRANE_LABEL_FONT,
            border: MEMBRANE_BORDER,
            dash: MEMBRANE_DASH,
        }
    }

    /// L'UNIQUE transformation d'échelle de la membrane (SCALE-1).
    fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            radius: s.world(self.radius),
            label_dx: s.world(self.label_dx),
            label_dy: s.world(self.label_dy),
            label_font: s.world(self.label_font),
            border: s.world(self.border),
            dash: s.world(self.dash),
        }
    }
}

pub(super) fn draw_membranes(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let PaintKit {
        typography,
        tints,
        theme,
        ..
    } = kit;
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let Annotation::Membrane {
            id,
            x,
            y,
            width,
            height,
            text,
            color,
            domains,
            ..
        } = ann
        else {
            continue;
        };
        let layout = MembraneLayout::new(*width as f32, *height as f32).scaled(scale);
        let (wx, wy) = world_to_screen(*x, *y, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        if clip.rejects(sx, sy, layout.width, layout.height) {
            continue;
        }

        let tint = color
            .as_deref()
            .map(|c| parse_hex_color(c, 96, 165, 250))
            .unwrap_or((96, 165, 250));
        let selected = store.selected_annotation_ids.contains(id);
        draw_membrane_shape(pixmap, (sx, sy), &layout, tint, (selected, scale));

        if scale.draws_detail() {
            if let Some(label) = text.as_deref().filter(|l| !l.is_empty()) {
                typography.draw_text_with_outline(
                    pixmap,
                    label,
                    sx + layout.label_dx,
                    sy - layout.label_dy,
                    TextStyle {
                        size: layout.label_font,
                        color: Color::from_rgba8(tint.0, tint.1, tint.2, 255),
                        face: Face::Bold,
                    },
                    theme.bg_canvas,
                );
            }
        }
        if selected {
            draw_resize_handles(
                pixmap,
                theme,
                scale,
                (sx, sy, layout.width, layout.height),
                &Handle::ALL,
            );
        }
        // La réglette d'une membrane s'aligne à DROITE de son bord haut : le coin haut-gauche
        // est déjà occupé par le titre protecteur, et deux textes superposés ne se lisent ni
        // l'un ni l'autre.
        let gauge_x = sx + layout.width - gauge_width(scale, domains.len());
        draw_domain_gauge(typography, tints, pixmap, scale, (gauge_x, sy), domains);
    }
}

fn draw_membrane_shape(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &MembraneLayout,
    tint: (u8, u8, u8),
    state: (bool, WorldScale),
) {
    let (selected, scale) = state;
    let (r, g, b) = tint;

    // 1. Halo ultra-discret multicouche — il appartient au cadre, donc il le suit.
    if scale.draws_detail() {
        for (pad_world, alpha) in MEMBRANE_GLOW {
            let pad = scale.world(pad_world);
            let mut pb = PathBuilder::new();
            push_rounded_rect(
                &mut pb,
                at.0 - pad,
                at.1 - pad,
                layout.width + pad * 2.0,
                layout.height + pad * 2.0,
                layout.radius + pad * 0.5,
            );
            if let Some(path) = pb.finish() {
                let mut paint = Paint {
                    anti_alias: true,
                    ..Default::default()
                };
                paint.set_color(Color::from_rgba8(r, g, b, alpha));
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    let mut pb = PathBuilder::new();
    push_rounded_rect(
        &mut pb,
        at.0,
        at.1,
        layout.width,
        layout.height,
        layout.radius,
    );
    let Some(path) = pb.finish() else {
        return;
    };

    // 2. Remplissage translucide.
    let mut fill = Paint {
        anti_alias: true,
        ..Default::default()
    };
    fill.set_color(Color::from_rgba8(r, g, b, 8));
    pixmap.fill_path(
        &path,
        &fill,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );

    draw_membrane_border(
        pixmap,
        &path,
        tint,
        (selected, scale, layout.border, layout.dash),
    );
}

/// Le contour d'une membrane : pointillé au repos, plein quand elle est prise.
///
/// # Le pointillé s'arrête où le pixel s'arrête
///
/// Un tiret plus fin qu'un pixel ne se voit pas comme un tiret : l'œil n'y lit qu'un trait
/// continu, à moitié moins dense puisque la moitié du parcours est vide. On dessine donc
/// exactement cela — un trait plein, d'opacité moitié. Le seuil n'est pas choisi : c'est le
/// pixel, la plus petite chose qu'un écran sache montrer.
///
/// Ce n'est pas qu'une question d'aspect. Le tiret est mis à l'échelle (SCALE-1), donc leur
/// **nombre** ne dépend pas du zoom : une membrane de cinq mille unités de périmètre en porte
/// deux cent cinquante à toute échelle, chacun avec deux bouts arrondis. Au fort dézoom on
/// rastérisait donc cinq cents arcs sous le pixel, par membrane et par image — la moitié du
/// coût d'une image à l'échelle 0,02. La pire image y est divisée par deux (fiche 13, vague B).
fn draw_membrane_border(
    pixmap: &mut PixmapMut,
    path: &tiny_skia::Path,
    tint: (u8, u8, u8),
    state: (bool, WorldScale, f32, f32),
) {
    let (r, g, b) = tint;
    let (selected, scale, border_width, dash) = state;
    let pointille = !selected && dash >= 1.0;
    let opacite = if selected {
        235
    } else if pointille {
        115
    } else {
        115 / 2
    };
    let mut border = Paint {
        anti_alias: true,
        ..Default::default()
    };
    border.set_color(Color::from_rgba8(r, g, b, opacite));
    let stroke = Stroke {
        width: if selected {
            scale.screen(SELECTION_RING)
        } else {
            border_width
        },
        dash: pointille
            .then(|| tiny_skia::StrokeDash::new(vec![dash, dash], 0.0))
            .flatten(),
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(path, &border, &stroke, Transform::identity(), None);
}

pub(super) fn draw_guides(
    theme: &Theme,
    pixmap: &mut PixmapMut,
    guides: &SnapGuides,
    vp: &Viewport,
    size: (u32, u32),
    header_h: f32,
) {
    let mut paint = Paint::default();
    paint.set_color(theme.snap_guide);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    let (w, h) = size;

    for &gx in guides.x.iter().flatten() {
        let (sx, _) = world_to_screen(gx, 0.0, vp);
        let mut pb = PathBuilder::new();
        pb.move_to(sx as f32, header_h);
        pb.line_to(sx as f32, h as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    for &gy in guides.y.iter().flatten() {
        let (_, sy) = world_to_screen(0.0, gy, vp);
        if sy < header_h as f64 {
            continue;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, sy as f32);
        pb.line_to(w as f32, sy as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

/// Fiche 07 § 7.3 — la sélection élastique : contour blanc à 0,50 de 1 px, intérieur blanc
/// à 0,03. Monochrome, comme toute la chrome.
pub(super) fn draw_selection_box(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    a: (f64, f64),
    b: (f64, f64),
) {
    let rect = Rect::from_xywh(
        a.0.min(b.0) as f32,
        a.1.min(b.1) as f32,
        (a.0 - b.0).abs() as f32,
        (a.1 - b.1).abs() as f32,
    );
    let Some(rect) = rect else {
        return;
    };
    let mut fill = Paint::default();
    fill.set_color(theme.rubberband_fill);
    pixmap.fill_rect(rect, &fill, Transform::identity(), None);

    let mut border = Paint::default();
    border.set_color(theme.rubberband_stroke);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(
        &PathBuilder::from_rect(rect),
        &border,
        &stroke,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_1_a_membrane_is_self_similar_at_every_zoom() {
        let world = MembraneLayout::new(800.0, 600.0);
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
            let screen = world.scaled(WorldScale::new(zoom));
            for (on_screen, in_world) in [
                (screen.radius, world.radius),
                (screen.label_dx, world.label_dx),
                (screen.label_dy, world.label_dy),
                (screen.label_font, world.label_font),
                (screen.border, world.border),
                (screen.dash, world.dash),
            ] {
                let expected = in_world / world.width;
                let observed = on_screen / screen.width;
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : rapport {observed} au lieu de {expected}"
                );
            }
        }
    }
}
