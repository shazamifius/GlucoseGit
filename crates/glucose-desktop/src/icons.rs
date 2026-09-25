//! Tracé vectoriel ultra-précis des icônes de Glucose via tiny-skia.
//! Reproduit fidèlement les tracés SVG de Toolbar.tsx avec anti-aliasing et subpixel rendering.

use crate::renderer::push_rounded_rect;
use tiny_skia::{
    Color, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, StrokeDash, Transform,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    Select,
    Pan,
    Text,
    Sticky,
    Arrow,
    Folder,
    Membrane,
    Plus,
    Organize,
    Timer,
    /// Une horloge qu'une flèche fait revenir en arrière — la Time Machine.
    Histoire,
    Storyboard,
    Magnet,
    TransDomain,
    Collab,
    Export,
    Plugins,
    Preset,
    Domains,
    /// Un cadenas fermé — l'état verrouillé de la barre d'action (fiche 10).
    Lock,
    /// Le même, anse ouverte : l'action « verrouiller », qui ne l'est pas encore.
    Unlock,
    /// Une corbeille — l'action « supprimer ».
    Trash,
}

#[allow(dead_code)]
pub fn draw_icon(
    pixmap: &mut PixmapMut,
    icon: IconType,
    x: f32,
    y: f32,
    color: Color,
    stroke_width: f32,
) {
    draw_icon_scaled(pixmap, icon, x, y, 14.0, color, stroke_width);
}

pub fn draw_icon_scaled(
    pixmap: &mut PixmapMut,
    icon: IconType,
    x: f32,
    y: f32,
    target_size: f32,
    color: Color,
    stroke_width: f32,
) {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let base_stroke = Stroke {
        width: stroke_width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };

    // Chaque icône a sa viewBox d'origine selon Toolbar.tsx
    let (src_size, s_width) = match icon {
        IconType::Pan => (20.0, stroke_width * (20.0 / target_size)),
        IconType::Organize
        | IconType::Timer
        | IconType::Histoire
        | IconType::Storyboard
        | IconType::Magnet
        | IconType::TransDomain
        | IconType::Collab
        | IconType::Export
        | IconType::Plugins
        | IconType::Preset
        | IconType::Domains
        | IconType::Plus => (16.0, stroke_width * (16.0 / target_size)),
        _ => (14.0, stroke_width * (14.0 / target_size)),
    };

    let scale = target_size / src_size;
    let ts = Transform::from_scale(scale, scale).post_translate(x, y);

    let stroke = Stroke {
        width: s_width,
        ..base_stroke
    };

    match icon {
        IconType::Select
        | IconType::Pan
        | IconType::Text
        | IconType::Sticky
        | IconType::Arrow
        | IconType::Folder
        | IconType::Membrane
        | IconType::Plus => draw_tool_icon(pixmap, icon, &paint, &stroke, ts),
        IconType::Lock | IconType::Unlock | IconType::Trash => {
            draw_action_icon(pixmap, icon, &paint, &stroke, ts)
        }
        _ => draw_panel_icon(pixmap, icon, &paint, &stroke, ts, color, s_width),
    }
}

/// Les outils de la barre : ce qui se pose sur le canevas.
fn draw_tool_icon(
    pixmap: &mut PixmapMut,
    icon: IconType,
    paint: &Paint,
    stroke: &Stroke,
    ts: Transform,
) {
    match icon {
        IconType::Select => {
            // <path d="M2 2L6.5 12L8 8L12 6.5L2 2Z" strokeWidth="1.5" strokeLinejoin="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 2.0);
            pb.line_to(6.5, 12.0);
            pb.line_to(8.0, 8.0);
            pb.line_to(12.0, 6.5);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Pan => draw_pan_icon(pixmap, paint, stroke, ts),
        IconType::Text => {
            // <path d="M2 3h10M7 3v8" strokeWidth="1.5" strokeLinecap="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 3.0);
            pb.line_to(12.0, 3.0);
            pb.move_to(7.0, 3.0);
            pb.line_to(7.0, 11.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Sticky => {
            // <rect x="1.5" y="1.5" width="11" height="11" rx="1" strokeWidth="1.3"/>
            // <path d="M4 5h6M4 7.5h4" strokeWidth="1.3" strokeLinecap="round"/>
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 11.0, 11.0, 1.5);
            pb.move_to(4.0, 5.0);
            pb.line_to(10.0, 5.0);
            pb.move_to(4.0, 7.5);
            pb.line_to(8.0, 7.5);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Arrow => {
            // <path d="M2 12L12 2M12 2H7M12 2V7" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 12.0);
            pb.line_to(12.0, 2.0);
            pb.move_to(7.0, 2.0);
            pb.line_to(12.0, 2.0);
            pb.line_to(12.0, 7.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Folder => draw_folder_icon(pixmap, paint, stroke, ts),
        IconType::Membrane => {
            // <rect x="1.5" y="1.5" width="11" height="11" rx="3" strokeWidth="1.3" strokeDasharray="3 2"/>
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 11.0, 11.0, 3.0);
            if let Some(path) = pb.finish() {
                let dashed_stroke = Stroke {
                    dash: StrokeDash::new(vec![3.0, 2.0], 0.0),
                    ..stroke.clone()
                };
                pixmap.stroke_path(&path, paint, &dashed_stroke, ts, None);
            }
        }
        IconType::Plus => draw_plus_icon(pixmap, paint, stroke, ts),
        _ => {}
    }
}

/// Les panneaux et les menus de la barre supérieure.
fn draw_panel_icon(
    pixmap: &mut PixmapMut,
    icon: IconType,
    paint: &Paint,
    stroke: &Stroke,
    ts: Transform,
    color: Color,
    s_width: f32,
) {
    match icon {
        IconType::Organize => {
            // 4 rects rx="1": (1,1,6,4), (9,1,6,4), (1,8,6,4), (9,8,6,4)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 9.0, 1.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 1.5, 8.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 9.0, 8.5, 5.5, 4.5, 1.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Timer => draw_timer_icon(pixmap, paint, stroke, ts),
        IconType::Histoire => draw_history_icon(pixmap, paint, stroke, ts),
        IconType::Storyboard => {
            // 4 rects rx="1": (1,2,6,5), (9,2,6,5), (1,9,6,5), (9,9,6,5)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 2.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 9.0, 2.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 1.5, 9.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 9.0, 9.0, 5.5, 5.0, 1.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Magnet => draw_magnet_icon(pixmap, paint, stroke, ts, color, s_width),
        IconType::TransDomain => draw_trans_domain_icon(pixmap, paint, stroke, ts),
        IconType::Collab => draw_collab_icon(pixmap, paint, stroke, ts),
        IconType::Export => {
            // <path d="M8 2v8M5 7l3 3 3-3M2 13h12" strokeWidth="1.5"/>
            let mut pb = PathBuilder::new();
            pb.move_to(8.0, 2.0);
            pb.line_to(8.0, 10.0);
            pb.move_to(5.0, 7.0);
            pb.line_to(8.0, 10.0);
            pb.line_to(11.0, 7.0);
            pb.move_to(2.0, 13.0);
            pb.line_to(14.0, 13.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Plugins => draw_plugins_icon(pixmap, paint, stroke, ts),
        IconType::Preset => {
            // 4 carrés rx="1": (1,1,6,6), (9,1,6,6), (1,9,6,6), (9,9,6,6)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 9.0, 1.5, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 1.5, 9.0, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 9.0, 9.0, 5.5, 5.5, 1.2);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        IconType::Domains => draw_domains_icon(pixmap, paint, stroke, ts),
        // <rect x="2" y="6" width="10" height="7" rx="1"/> + l'anse
        // <path d="M4.5 6V4.5a2.5 2.5 0 015 0V6"/> — fermée — ou sans le « V6 », ouverte.
        _ => {}
    }
}

/// Les actions d'une sélection : verrouiller, supprimer (fiche 10 § 3).
fn draw_action_icon(
    pixmap: &mut PixmapMut,
    icon: IconType,
    paint: &Paint,
    stroke: &Stroke,
    ts: Transform,
) {
    match icon {
        IconType::Lock | IconType::Unlock => {
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 2.0, 6.0, 10.0, 7.0, 1.0);
            pb.move_to(4.5, 6.0);
            pb.line_to(4.5, 4.5);
            // Le demi-cercle de l'anse, en deux quarts : l'approximation d'un quart de
            // cercle par une cubique est exacte à moins d'un millième du rayon, soit
            // invisible sur une icône de dix pixels.
            push_half_circle(&mut pb, 7.0, 4.5, 2.5);
            if icon == IconType::Lock {
                pb.line_to(9.5, 6.0);
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        // <path d="M2 4h10M5 4V2.5h4V4M5.5 6.5v4M8.5 6.5v4M3 4l.5 8h7L11 4"/>
        IconType::Trash => {
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 4.0);
            pb.line_to(12.0, 4.0);
            pb.move_to(5.0, 4.0);
            pb.line_to(5.0, 2.5);
            pb.line_to(9.0, 2.5);
            pb.line_to(9.0, 4.0);
            pb.move_to(5.5, 6.5);
            pb.line_to(5.5, 10.5);
            pb.move_to(8.5, 6.5);
            pb.line_to(8.5, 10.5);
            pb.move_to(3.0, 4.0);
            pb.line_to(3.5, 12.0);
            pb.line_to(10.5, 12.0);
            pb.line_to(11.0, 4.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, paint, stroke, ts, None);
            }
        }
        _ => {}
    }
}

/// Le demi-cercle supérieur de centre `(cx, cy)` et de rayon `r`, du point gauche au droit.
///
/// Deux cubiques d'un quart de tour chacune. La constante `K` est celle de l'approximation
/// d'un quart de cercle par une cubique — $rac{4}{3}(\sqrt{2} - 1)$, et non un nombre
/// choisi : c'est la valeur qui annule l'erreur au point médian de l'arc.
fn push_half_circle(pb: &mut PathBuilder, cx: f32, cy: f32, r: f32) {
    const K: f32 = 0.552_284_8;
    let k = r * K;
    pb.cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
    pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
}

/// L'aimant : un fer à cheval, ses deux pôles et ses ondes.
fn draw_magnet_icon(
    pixmap: &mut PixmapMut,
    paint: &Paint,
    stroke: &Stroke,
    ts: Transform,
    color: Color,
    s_width: f32,
) {
    // <path d="M2 2l12 12M14 2L2 14" strokeWidth="1" strokeOpacity="0.4"/>
    // <rect x="3" y="3" width="10" height="10" rx="1" strokeWidth="1.3" strokeDasharray="2 1"/>
    let mut pb_cross = PathBuilder::new();
    pb_cross.move_to(2.0, 2.0);
    pb_cross.line_to(14.0, 14.0);
    pb_cross.move_to(14.0, 2.0);
    pb_cross.line_to(2.0, 14.0);
    if let Some(path) = pb_cross.finish() {
        let mut faint_paint = Paint::default();
        let col = Color::from_rgba8(
            (color.red() * 255.0) as u8,
            (color.green() * 255.0) as u8,
            (color.blue() * 255.0) as u8,
            (color.alpha() * 255.0 * 0.45) as u8,
        );
        faint_paint.set_color(col);
        faint_paint.anti_alias = true;
        let thin_stroke = Stroke {
            width: s_width * 0.8,
            ..stroke.clone()
        };
        pixmap.stroke_path(&path, &faint_paint, &thin_stroke, ts, None);
    }

    let mut pb_rect = PathBuilder::new();
    push_rounded_rect(&mut pb_rect, 3.0, 3.0, 10.0, 10.0, 1.5);
    if let Some(path) = pb_rect.finish() {
        let dashed = Stroke {
            dash: StrokeDash::new(vec![2.5, 1.5], 0.0),
            ..stroke.clone()
        };
        pixmap.stroke_path(&path, paint, &dashed, ts, None);
    }
}

/// Les domaines : trois cercles en triangle.
fn draw_domains_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // 3 cercles: (5, 6, r=3), (11, 6, r=3), (8, 11, r=3)
    let mut pb = PathBuilder::new();
    pb.push_circle(5.0, 6.0, 3.0);
    pb.push_circle(11.0, 6.0, 3.0);
    pb.push_circle(8.0, 11.0, 3.0);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// Les extensions : une pièce de puzzle.
fn draw_plugins_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // Puzzle icon
    let mut pb = PathBuilder::new();
    pb.move_to(6.0, 1.5);
    pb.line_to(10.0, 1.5);
    pb.line_to(10.0, 3.7);
    pb.line_to(12.3, 4.6);
    pb.line_to(14.5, 4.6);
    pb.line_to(14.5, 8.6);
    pb.line_to(12.3, 8.6);
    pb.line_to(10.0, 9.5);
    pb.line_to(10.0, 14.5);
    pb.line_to(4.0, 14.5);
    pb.line_to(4.0, 12.3);
    pb.line_to(1.7, 11.4);
    pb.line_to(1.7, 7.4);
    pb.line_to(4.0, 7.4);
    pb.line_to(6.0, 6.5);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// Le plus : ajouter.
fn draw_plus_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <path d="M8 2v12M2 8h12" strokeWidth="2" strokeLinecap="round"/>
    let mut pb = PathBuilder::new();
    pb.move_to(8.0, 2.0);
    pb.line_to(8.0, 14.0);
    pb.move_to(2.0, 8.0);
    pb.line_to(14.0, 8.0);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// Les trans-domaines : deux anneaux liés.
fn draw_trans_domain_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <circle cx="3.5" cy="8" r="2" strokeWidth="1.3"/>
    // <circle cx="12.5" cy="8" r="2" strokeWidth="1.3"/>
    // <path d="M5.5 8h5" strokeWidth="1.3" strokeDasharray="1.5 1.5"/>
    let mut pb = PathBuilder::new();
    pb.push_circle(3.5, 8.0, 2.0);
    pb.push_circle(12.5, 8.0, 2.0);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
    let mut pb_line = PathBuilder::new();
    pb_line.move_to(5.5, 8.0);
    pb_line.line_to(10.5, 8.0);
    if let Some(path) = pb_line.finish() {
        let dashed = Stroke {
            dash: StrokeDash::new(vec![1.8, 1.8], 0.0),
            ..stroke.clone()
        };
        pixmap.stroke_path(&path, paint, &dashed, ts, None);
    }
}

/// La collaboration : deux silhouettes.
fn draw_collab_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <circle cx="8" cy="8" r="6.5" strokeWidth="1.2"/>
    // <path d="M1.5 8h13M8 1.5c2 2 2 11 0 13M8 1.5c-2 2-2 11 0 13"/>
    let mut pb = PathBuilder::new();
    pb.push_circle(8.0, 8.0, 6.2);
    pb.move_to(1.8, 8.0);
    pb.line_to(14.2, 8.0);
    pb.move_to(8.0, 1.8);
    pb.cubic_to(10.2, 3.8, 10.2, 12.2, 8.0, 14.2);
    pb.move_to(8.0, 1.8);
    pb.cubic_to(5.8, 3.8, 5.8, 12.2, 8.0, 14.2);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// La main : déplacer la vue.
fn draw_pan_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <circle cx="10" cy="10" r="3" strokeWidth="1.5"/>
    // <path d="M10 2v3M10 15v3M2 10h3M15 10h3" strokeWidth="1.5" strokeLinecap="round"/>
    let mut pb = PathBuilder::new();
    pb.push_circle(10.0, 10.0, 3.0);
    pb.move_to(10.0, 2.0);
    pb.line_to(10.0, 5.0);
    pb.move_to(10.0, 15.0);
    pb.line_to(10.0, 18.0);
    pb.move_to(2.0, 10.0);
    pb.line_to(5.0, 10.0);
    pb.move_to(15.0, 10.0);
    pb.line_to(18.0, 10.0);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// Le dossier : une chemise à onglet.
fn draw_folder_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <path d="M1 4.5V11.5a1 1 0 001 1h10a1 1 0 001-1V5.5a1 1 0 00-1-1H7L5.5 2.5H2a1 1 0 00-1 2z" strokeWidth="1.2"/>
    let mut pb = PathBuilder::new();
    pb.move_to(1.5, 4.5);
    pb.line_to(1.5, 11.5);
    pb.line_to(12.5, 11.5);
    pb.line_to(12.5, 5.5);
    pb.line_to(7.0, 5.5);
    pb.line_to(5.5, 2.5);
    pb.line_to(2.5, 2.5);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// Le minuteur : un cadran et son aiguille.
fn draw_timer_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    // <circle cx="8" cy="9" r="6" strokeWidth="1.3"/>
    // <path d="M8 6v3.5l2 1.5"/> <path d="M6 2h4"/>
    let mut pb = PathBuilder::new();
    pb.push_circle(8.0, 9.0, 5.5);
    pb.move_to(8.0, 6.0);
    pb.line_to(8.0, 9.2);
    pb.line_to(10.0, 10.7);
    pb.move_to(6.0, 2.0);
    pb.line_to(10.0, 2.0);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}

/// L'icône de la Time Machine, sur la grille de 16 : un cercle presque complet qui revient
/// vers la gauche, la pointe de la flèche, et les aiguilles — la forme « history » de Lucide.
fn draw_history_icon(pixmap: &mut PixmapMut, paint: &Paint, stroke: &Stroke, ts: Transform) {
    let (cx, cy, r) = (8.0f32, 8.0f32, 6.0f32);
    let mut pb = PathBuilder::new();
    // De la gauche, en passant par le bas, la droite et le haut, jusqu'en haut à gauche.
    let (debut, fin) = (std::f32::consts::PI, -0.75 * std::f32::consts::PI);
    let pas = 32;
    for k in 0..=pas {
        let a = debut + (fin - debut) * k as f32 / pas as f32;
        let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
        if k == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.move_to(2.0, 2.5);
    pb.line_to(2.0, 5.5);
    pb.line_to(5.0, 5.5);
    pb.move_to(8.0, 5.0);
    pb.line_to(8.0, 8.0);
    pb.line_to(10.5, 9.3);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, ts, None);
    }
}
