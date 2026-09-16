//! Poser une image sur le canevas — et ce que cela coûte selon la façon de la poser.
//!
//! Le sujet a grandi jusqu'à mériter son module : une image ne se dessine pas, elle se
//! **reporte**, depuis un niveau de réduction (MIP-1) ou depuis une vignette déjà à la forme
//! voulue (MIP-2), en composant ou en remplaçant selon ce que son opacité autorise.

use super::super::domain::draw_domain_gauge;
use super::super::handles::draw_rotated_handles;
use super::super::pass::Clip;
use super::super::scale::WorldScale;
use super::super::Renderer;
use super::super::{photo, vignette, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use std::collections::{HashMap, HashSet};
use tiny_skia::{
    BlendMode, Color, FilterQuality, Paint, PathBuilder, PixmapMut, PixmapPaint, Rect, Stroke,
    Transform,
};

pub(in crate::renderer) fn draw_images(
    image_cache: &mut HashMap<String, photo::Pyramide>,
    vignettes: &mut vignette::Vignettes,
    failed_images: &mut HashSet<String>,
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

    for img in Visibles::nouvelles(pass.visibles, board).images() {
        let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        if clip.rejects(sx, sy, sw, sh) {
            continue;
        }

        let drawn = img
            .src
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|src| {
                Renderer::load_image_impl(image_cache, failed_images, src)
                    .map(|pyramide| poser(pyramide, vignettes, pixmap, img, (sx, sy, sw, sh)))
            });
        if drawn.is_none() {
            draw_missing_image(
                typography,
                theme,
                pixmap,
                (sx, sy),
                (sw, sh),
                &img.id,
                img.rotation,
            );
        }
        if store.selected_image_ids.contains(&img.id) {
            draw_image_adornments(pixmap, theme, scale, img, (sx, sy, sw, sh));
        }
        draw_domain_gauge(typography, tints, pixmap, scale, (sx, sy), &img.domains);
    }
}

/// Pose une image sur le canevas, par le chemin le plus économique qu'elle autorise.
///
/// Deux chemins, et le premier n'existe que parce que le rasteriseur n'est rapide qu'à
/// l'échelle 1 posée sur un entier (MIP-2) :
///
/// * la **vignette**, déjà à la taille et à la phase voulues : un report sans transformation ;
/// * le chemin général, qui rééchantillonne depuis le niveau de pyramide adéquat (MIP-1).
///
/// Une image tournée passe toujours par le second : une vignette est un rectangle droit, et la
/// faire tourner redemanderait la transformation qu'elle sert à éviter.
fn poser(
    pyramide: &mut photo::Pyramide,
    vignettes: &mut vignette::Vignettes,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
) {
    let (sx, sy, sw, sh) = ecran;
    let opaque = pyramide.opaque();
    let paint = PixmapPaint {
        quality: FilterQuality::Bilinear,
        blend_mode: mode_de_report(opaque, img.rotation),
        ..Default::default()
    };

    if img.rotation == 0.0 {
        let forme = photo::Forme::posee(sx, sy, sw, sh);
        if let Some(vignette) = vignettes.pour(&img.id, forme, pyramide) {
            // La phase est déjà dans la vignette : il ne reste qu'une position entière, ce qui
            // est le seul cas où le rasteriseur se contente de recopier.
            pixmap.draw_pixmap(
                sx.floor() as i32,
                sy.floor() as i32,
                vignette.as_ref(),
                &paint,
                Transform::identity(),
                None,
            );
            return;
        }
    }

    // MIP-1 : on part du niveau qui couvre encore la taille posée, jamais de la résolution
    // native. Le filtre lit alors des texels voisins au lieu d'en sauter neuf sur dix.
    let loaded = pyramide.niveau_pour(sw);
    let ts = Transform::from_scale(sw / loaded.width() as f32, sh / loaded.height() as f32)
        .post_translate(sx, sy)
        .post_rotate_at(
            img.rotation.to_degrees() as f32,
            sx + sw / 2.0,
            sy + sh / 2.0,
        );
    pixmap.draw_pixmap(0, 0, loaded.as_ref(), &paint, ts, None);
}

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
fn mode_de_report(opaque: bool, rotation: f64) -> BlendMode {
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
fn draw_image_adornments(
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
fn draw_missing_image(
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

// ── Guides et boîte de sélection — taille écran constante ───────────────────
