//! Tracé vectoriel ultra-précis des icônes de Glucose via tiny-skia.
//! Reproduit fidèlement les tracés SVG de Toolbar.tsx avec anti-aliasing et subpixel rendering.

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
    Storyboard,
    Magnet,
    TransDomain,
    Collab,
    Export,
    Plugins,
    Preset,
    Domains,
}

/// Dessine un rectangle à coins arrondis dans un PathBuilder
fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
        IconType::Select => {
            // <path d="M2 2L6.5 12L8 8L12 6.5L2 2Z" strokeWidth="1.5" strokeLinejoin="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 2.0);
            pb.line_to(6.5, 12.0);
            pb.line_to(8.0, 8.0);
            pb.line_to(12.0, 6.5);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Pan => {
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Text => {
            // <path d="M2 3h10M7 3v8" strokeWidth="1.5" strokeLinecap="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 3.0);
            pb.line_to(12.0, 3.0);
            pb.move_to(7.0, 3.0);
            pb.line_to(7.0, 11.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Folder => {
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Membrane => {
            // <rect x="1.5" y="1.5" width="11" height="11" rx="3" strokeWidth="1.3" strokeDasharray="3 2"/>
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 11.0, 11.0, 3.0);
            if let Some(path) = pb.finish() {
                let dashed_stroke = Stroke {
                    dash: StrokeDash::new(vec![3.0, 2.0], 0.0),
                    ..stroke
                };
                pixmap.stroke_path(&path, &paint, &dashed_stroke, ts, None);
            }
        }
        IconType::Plus => {
            // <path d="M8 2v12M2 8h12" strokeWidth="2" strokeLinecap="round"/>
            let mut pb = PathBuilder::new();
            pb.move_to(8.0, 2.0);
            pb.line_to(8.0, 14.0);
            pb.move_to(2.0, 8.0);
            pb.line_to(14.0, 8.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Organize => {
            // 4 rects rx="1": (1,1,6,4), (9,1,6,4), (1,8,6,4), (9,8,6,4)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 9.0, 1.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 1.5, 8.5, 5.5, 4.5, 1.0);
            push_rounded_rect(&mut pb, 9.0, 8.5, 5.5, 4.5, 1.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Timer => {
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Storyboard => {
            // 4 rects rx="1": (1,2,6,5), (9,2,6,5), (1,9,6,5), (9,9,6,5)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 2.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 9.0, 2.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 1.5, 9.0, 5.5, 5.0, 1.0);
            push_rounded_rect(&mut pb, 9.0, 9.0, 5.5, 5.0, 1.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Magnet => {
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
                    ((color.alpha() * 255.0) as f32 * 0.45) as u8,
                );
                faint_paint.set_color(col);
                faint_paint.anti_alias = true;
                let thin_stroke = Stroke {
                    width: s_width * 0.8,
                    ..stroke
                };
                pixmap.stroke_path(&path, &faint_paint, &thin_stroke, ts, None);
            }

            let mut pb_rect = PathBuilder::new();
            push_rounded_rect(&mut pb_rect, 3.0, 3.0, 10.0, 10.0, 1.5);
            if let Some(path) = pb_rect.finish() {
                let dashed = Stroke {
                    dash: StrokeDash::new(vec![2.5, 1.5], 0.0),
                    ..stroke
                };
                pixmap.stroke_path(&path, &paint, &dashed, ts, None);
            }
        }
        IconType::TransDomain => {
            // <circle cx="3.5" cy="8" r="2" strokeWidth="1.3"/>
            // <circle cx="12.5" cy="8" r="2" strokeWidth="1.3"/>
            // <path d="M5.5 8h5" strokeWidth="1.3" strokeDasharray="1.5 1.5"/>
            let mut pb = PathBuilder::new();
            pb.push_circle(3.5, 8.0, 2.0);
            pb.push_circle(12.5, 8.0, 2.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
            let mut pb_line = PathBuilder::new();
            pb_line.move_to(5.5, 8.0);
            pb_line.line_to(10.5, 8.0);
            if let Some(path) = pb_line.finish() {
                let dashed = Stroke {
                    dash: StrokeDash::new(vec![1.8, 1.8], 0.0),
                    ..stroke
                };
                pixmap.stroke_path(&path, &paint, &dashed, ts, None);
            }
        }
        IconType::Collab => {
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Plugins => {
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
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Preset => {
            // 4 carrés rx="1": (1,1,6,6), (9,1,6,6), (1,9,6,6), (9,9,6,6)
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, 1.5, 1.5, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 9.0, 1.5, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 1.5, 9.0, 5.5, 5.5, 1.2);
            push_rounded_rect(&mut pb, 9.0, 9.0, 5.5, 5.5, 1.2);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Domains => {
            // 3 cercles: (5, 6, r=3), (11, 6, r=3), (8, 11, r=3)
            let mut pb = PathBuilder::new();
            pb.push_circle(5.0, 6.0, 3.0);
            pb.push_circle(11.0, 6.0, 3.0);
            pb.push_circle(8.0, 11.0, 3.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
    }
}
