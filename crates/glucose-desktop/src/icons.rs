//! Tracé vectoriel des icônes de Glucose via tiny-skia.

use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, Transform};

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

pub fn draw_icon(
    pixmap: &mut PixmapMut,
    icon: IconType,
    x: f32,
    y: f32,
    color: Color,
    stroke_width: f32,
) {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let stroke = Stroke {
        width: stroke_width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };

    let ts = Transform::from_translate(x, y);

    match icon {
        IconType::Select => {
            // Curseur flèche
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
            // Main / Pan
            let mut pb = PathBuilder::new();
            pb.push_circle(7.0, 7.0, 3.0);
            pb.move_to(7.0, 1.0);
            pb.line_to(7.0, 4.0);
            pb.move_to(7.0, 10.0);
            pb.line_to(7.0, 13.0);
            pb.move_to(1.0, 7.0);
            pb.line_to(4.0, 7.0);
            pb.move_to(10.0, 7.0);
            pb.line_to(13.0, 7.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Text => {
            // Lettre T
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 3.0);
            pb.line_to(12.0, 3.0);
            pb.move_to(7.0, 3.0);
            pb.line_to(7.0, 12.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Sticky => {
            // Rectangle sticky avec lignes
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 2.0);
            pb.line_to(12.0, 2.0);
            pb.line_to(12.0, 12.0);
            pb.line_to(2.0, 12.0);
            pb.close();
            pb.move_to(4.0, 5.0);
            pb.line_to(10.0, 5.0);
            pb.move_to(4.0, 8.0);
            pb.line_to(8.0, 8.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Arrow => {
            // Flèche diagonale
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
            // Dossier
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
            // Rectangle arrondi pointillé
            let mut pb = PathBuilder::new();
            pb.move_to(3.0, 2.0);
            pb.line_to(11.0, 2.0);
            pb.line_to(11.0, 12.0);
            pb.line_to(3.0, 12.0);
            pb.close();
            if let Some(path) = pb.finish() {
                let dashed_stroke = Stroke {
                    dash: tiny_skia::StrokeDash::new(vec![3.0, 2.0], 0.0),
                    ..stroke
                };
                pixmap.stroke_path(&path, &paint, &dashed_stroke, ts, None);
            }
        }
        IconType::Plus => {
            // Croix plus
            let mut pb = PathBuilder::new();
            pb.move_to(7.0, 2.0);
            pb.line_to(7.0, 12.0);
            pb.move_to(2.0, 7.0);
            pb.line_to(12.0, 7.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Organize => {
            // 4 cases grille
            let mut pb = PathBuilder::new();
            pb.move_to(1.0, 1.0); pb.line_to(6.0, 1.0); pb.line_to(6.0, 5.0); pb.line_to(1.0, 5.0); pb.close();
            pb.move_to(8.0, 1.0); pb.line_to(13.0, 1.0); pb.line_to(13.0, 5.0); pb.line_to(8.0, 5.0); pb.close();
            pb.move_to(1.0, 7.0); pb.line_to(6.0, 7.0); pb.line_to(6.0, 11.0); pb.line_to(1.0, 11.0); pb.close();
            pb.move_to(8.0, 7.0); pb.line_to(13.0, 7.0); pb.line_to(13.0, 11.0); pb.line_to(8.0, 11.0); pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Timer => {
            // Horloge / sablier
            let mut pb = PathBuilder::new();
            pb.push_circle(7.0, 8.0, 5.5);
            pb.move_to(7.0, 5.0);
            pb.line_to(7.0, 8.0);
            pb.line_to(9.0, 9.5);
            pb.move_to(5.0, 1.5);
            pb.line_to(9.0, 1.5);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Storyboard => {
            // Filmstrip / storyboard
            let mut pb = PathBuilder::new();
            pb.move_to(1.0, 2.0); pb.line_to(6.0, 2.0); pb.line_to(6.0, 6.0); pb.line_to(1.0, 6.0); pb.close();
            pb.move_to(8.0, 2.0); pb.line_to(13.0, 2.0); pb.line_to(13.0, 6.0); pb.line_to(8.0, 6.0); pb.close();
            pb.move_to(1.0, 8.0); pb.line_to(6.0, 8.0); pb.line_to(6.0, 12.0); pb.line_to(1.0, 12.0); pb.close();
            pb.move_to(8.0, 8.0); pb.line_to(13.0, 8.0); pb.line_to(13.0, 12.0); pb.line_to(8.0, 12.0); pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Magnet => {
            // Aimant SNAP-1
            let mut pb = PathBuilder::new();
            pb.move_to(2.0, 2.0);
            pb.line_to(12.0, 12.0);
            pb.move_to(12.0, 2.0);
            pb.line_to(2.0, 12.0);
            pb.move_to(3.0, 3.0);
            pb.line_to(11.0, 3.0);
            pb.line_to(11.0, 11.0);
            pb.line_to(3.0, 11.0);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::TransDomain => {
            // Liens trans-domaines
            let mut pb = PathBuilder::new();
            pb.push_circle(3.5, 7.0, 2.0);
            pb.push_circle(11.5, 7.0, 2.0);
            pb.move_to(5.5, 7.0);
            pb.line_to(9.5, 7.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Collab => {
            // Globe
            let mut pb = PathBuilder::new();
            pb.push_circle(7.0, 7.0, 5.5);
            pb.move_to(1.5, 7.0); pb.line_to(12.5, 7.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Export => {
            // Exporter
            let mut pb = PathBuilder::new();
            pb.move_to(7.0, 2.0);
            pb.line_to(7.0, 9.0);
            pb.move_to(4.5, 6.5);
            pb.line_to(7.0, 9.0);
            pb.line_to(9.5, 6.5);
            pb.move_to(2.0, 12.0);
            pb.line_to(12.0, 12.0);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Plugins => {
            // Puzzle / Plugins
            let mut pb = PathBuilder::new();
            pb.move_to(5.0, 2.0); pb.line_to(9.0, 2.0); pb.line_to(9.0, 5.0);
            pb.line_to(12.0, 5.0); pb.line_to(12.0, 9.0); pb.line_to(9.0, 9.0);
            pb.line_to(9.0, 12.0); pb.line_to(5.0, 12.0); pb.line_to(5.0, 9.0);
            pb.line_to(2.0, 9.0); pb.line_to(2.0, 5.0); pb.line_to(5.0, 5.0);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Preset => {
            // 4 carrés preset
            let mut pb = PathBuilder::new();
            pb.move_to(1.5, 1.5); pb.line_to(6.0, 1.5); pb.line_to(6.0, 6.0); pb.line_to(1.5, 6.0); pb.close();
            pb.move_to(8.0, 1.5); pb.line_to(12.5, 1.5); pb.line_to(12.5, 6.0); pb.line_to(8.0, 6.0); pb.close();
            pb.move_to(1.5, 8.0); pb.line_to(6.0, 8.0); pb.line_to(6.0, 12.5); pb.line_to(1.5, 12.5); pb.close();
            pb.move_to(8.0, 8.0); pb.line_to(12.5, 8.0); pb.line_to(12.5, 12.5); pb.line_to(8.0, 12.5); pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
        IconType::Domains => {
            // 3 cercles domaines
            let mut pb = PathBuilder::new();
            pb.push_circle(4.5, 5.0, 2.5);
            pb.push_circle(9.5, 5.0, 2.5);
            pb.push_circle(7.0, 9.5, 2.5);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, ts, None);
            }
        }
    }
}
