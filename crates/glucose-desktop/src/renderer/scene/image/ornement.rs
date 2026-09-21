//! Ce qu'une image porte **en plus** de ses pixels : son cadre de sélection, ses poignées, et
//! le carré qui la remplace tant que ses octets ne sont pas là.
//!
//! # Pourquoi c'est à part, et pourquoi cela compte pour les tuiles
//!
//! Aucun de ces ornements n'appartient au **document**. La sélection est un état de
//! l'interface ; l'absence d'octets est un état du magasin. Une tuile, elle, ne connaît que
//! le document — c'est ce qui la rend réutilisable. Y peindre un cadre de sélection ferait
//! qu'une image resterait cerclée après avoir été désélectionnée, tant que sa tuile survit.
//!
//! Les ornements se dessinent donc **par-dessus** les tuiles, à chaque image, en direct.

use super::super::super::handles::draw_rotated_handles;
use super::super::super::scale::WorldScale;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::resize::Handle;
use tiny_skia::{BlendMode, Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Comment reporter une image sur le canevas : en remplaçant, ou en composant.
///
/// Le remplacement est plus rapide, mais il écrit **tous** les pixels de la zone couverte. Il
/// n'est donc licite qu'à deux conditions réunies :
///
/// * l'image est opaque — sinon le fond devrait transparaître à travers elle ;
/// * elle n'est pas tournée — sinon la zone couverte est un parallélogramme, et les coins du
///   rectangle qui l'entoure seraient remplacés par du vide, laissant quatre trous.
///
/// Les deux se constatent, aucune ne s'estime.
pub(super) fn mode_de_report(opaque: bool, rotation: f64) -> BlendMode {
    if opaque && rotation == 0.0 {
        BlendMode::Source
    } else {
        BlendMode::SourceOver
    }
}

/// Ce qu'une image **sélectionnée** porte en plus : son cadre, et ses prises.
///
/// Une image verrouillée se signale par la couleur de son cadre et par l'absence de ses
/// poignées (fiche 06 § 4.3) : les deux disent le même fait, l'un de loin, l'autre au moment
/// où la main cherche une prise. Les deux suivent la rotation du nœud, comme lui.
pub(super) fn draw_image_adornments(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    img: &glucose_core::types::BoardImage,
    screen_box: (f32, f32, f32, f32),
) {
    let (sx, sy, sw, sh) = screen_box;
    let ink = if img.locked {
        theme.alert
    } else {
        theme.selection_frame
    };
    draw_image_selection(pixmap, ink, (sx, sy), (sw, sh), img.rotation);
    if !img.locked {
        draw_rotated_handles(pixmap, theme, scale, screen_box, &Handle::ALL, img.rotation);
    }
}

/// Une image dont les octets ne sont pas (encore) là : un cadre gris de la chrome, son
/// identifiant dedans. Monochrome — ce n'est pas du contenu, c'est son absence.
pub(in crate::renderer) fn draw_missing_image(
    typography: &Typography,
    theme: &Theme,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    size: (f32, f32),
    id: &str,
    rotation: f64,
) {
    let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) else {
        return;
    };
    // Le carré de remplacement tourne comme tournerait la texture : sans cela, une image
    // introuvable et penchée se dessinerait droite dans un cadre incliné.
    let ts = rotation_at(rotation, at, size);
    let path = PathBuilder::from_rect(rect);
    let mut fill = Paint {
        anti_alias: true,
        ..Default::default()
    };
    fill.set_color(theme.bg_hover);
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, ts, None);

    let mut border = Paint {
        anti_alias: true,
        ..Default::default()
    };
    border.set_color(theme.border_accent);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, ts, None);

    typography.draw_text(
        pixmap,
        &format!("Image [{id}]"),
        at.0 + 10.0,
        at.1 + size.1 / 2.0 - 6.0,
        TextStyle {
            size: 12.0,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
}

/// La transformation qui fait tourner une boîte écran autour de son propre centre.
///
/// Un seul endroit où l'angle devient une matrice : la texture, son carré de remplacement et
/// son cadre de sélection tournent donc exactement pareil, par construction.
fn rotation_at(rotation: f64, at: (f32, f32), size: (f32, f32)) -> Transform {
    if rotation == 0.0 {
        return Transform::identity();
    }
    Transform::from_rotate_at(
        rotation.to_degrees() as f32,
        at.0 + size.0 / 2.0,
        at.1 + size.1 / 2.0,
    )
}

/// Débord du cadre de sélection autour de la texture, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_INSET: f32 = 3.0;
/// Épaisseur du cadre de sélection, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_STROKE: f32 = 1.25;

/// Fiche 06 § 4.2 — cadre hairline blanc pur à 0,80, débordant de 3 px, épais de 1,25 px à
/// l'écran quel que soit le zoom. Aucun néon : le contour se lit sur une image claire par le
/// liseré des poignées, sur le fond noir par le blanc.
fn draw_image_selection(
    pixmap: &mut PixmapMut,
    ink: Color,
    at: (f32, f32),
    size: (f32, f32),
    rotation: f64,
) {
    let d = IMAGE_SELECTION_INSET;
    let Some(rect) = Rect::from_xywh(at.0 - d, at.1 - d, size.0 + 2.0 * d, size.1 + 2.0 * d) else {
        return;
    };
    // Le cadre épouse le nœud : il tourne avec lui, autour du même centre.
    let ts = rotation_at(
        rotation,
        (at.0 - d, at.1 - d),
        (size.0 + 2.0 * d, size.1 + 2.0 * d),
    );
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(ink);
    let stroke = Stroke {
        width: IMAGE_SELECTION_STROKE,
        ..Default::default()
    };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &paint, &stroke, ts, None);
}
