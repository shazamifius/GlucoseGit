//! Le dessin des dossiers de canevas (fiche 06 § 8).
//!
//! # Ce que ce module répare
//!
//! Un dossier est décrit dans la fiche 08 § 5 comme *« un portail vers un univers complet »*.
//! Il existait dans le document, répondait au clic — `hit_priority` connaît son bandeau et son
//! bord depuis longtemps — et **n'avait aucun pixel**. Le renderer ne lisait pas
//! `board.folders`. C'est le constat le plus visible de la relecture du dossier 06–10.
//!
//! # Les proportions
//!
//! Tout vient de la fiche 06 § 8.1, et la seule mesure que le noyau connaissait déjà —
//! [`pick_consts::FOLDER_HEADER`] — reste la source du bandeau : le dessin et le clic ne
//! peuvent pas diverger s'ils lisent la même constante.
//!
//! Le corps est teinté par la couleur du dossier à très faible opacité, comme la référence :
//! c'est un cadre qui contient, pas un objet qui s'affirme. La sélection double cette
//! opacité, remplace le pointillé par un trait plein et ajoute un halo — la même grammaire
//! que les membranes, qui sont l'autre conteneur du canevas.

use crate::renderer::card::Clip;
use crate::renderer::handles::draw_resize_handles;
use crate::renderer::scale::WorldScale;
use crate::renderer::{parse_hex_color, push_rounded_rect, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::typography::TextStyle;
use glucose_core::hit_priority::pick_consts;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use glucose_core::types::CanvasFolder;
use tiny_skia::{Color, LineCap, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Rayon des coins du cadre, en unités monde (fiche 06 § 8.1).
pub(super) const CORNER_RADIUS: f32 = 10.0;
/// Largeur minimale d'un dossier, en unités monde.
pub(super) const MIN_WIDTH: f32 = 180.0;
/// Hauteur minimale d'un dossier, en unités monde.
pub(super) const MIN_HEIGHT: f32 = 120.0;
/// Opacité du corps, sur 255 : 2,5 % au repos, 5 % sélectionné.
const BODY_ALPHA: (u8, u8) = (6, 13);
/// Opacité de la bordure : 14 % au repos, 45 % sélectionnée.
const BORDER_ALPHA: (u8, u8) = (36, 115);
/// Épaisseur de la bordure, en unités monde : 1 px au repos, 1,4 px sélectionnée.
const BORDER_WIDTH: (f32, f32) = (1.0, 1.4);
/// Le pointillé de la bordure au repos : trois unités de trait, cinq de vide.
const BORDER_DASH: (f32, f32) = (3.0, 5.0);
/// Le halo de sélection : débord, rayon supplémentaire, épaisseur, opacité.
const GLOW: (f32, f32, f32, u8) = (3.0, 13.0, 1.5, 89);

/// Position et taille de l'icône d'explorateur, en unités monde.
const ICON: (f32, f32, f32, f32) = (12.0, 11.0, 16.0, 14.0);
/// Opacité de l'icône : 45 % au repos, 70 % sélectionnée.
const ICON_ALPHA: (u8, u8) = (115, 179);
/// Ancre du titre dans le bandeau, et sa taille de police, en unités monde.
const TITLE: (f32, f32, f32) = (34.0, 23.0, 14.0);
/// Longueur au-delà de laquelle le titre est tronqué.
const TITLE_MAX_CHARS: usize = 28;
/// Le badge compteur : retrait depuis le bord droit, ordonnée, largeur, hauteur, rayon.
const BADGE: (f32, f32, f32, f32, f32) = (38.0, 11.0, 30.0, 16.0, 8.0);
/// Opacité du fond du badge (18 %) et de son contour (40 %).
const BADGE_ALPHA: (u8, u8) = (46, 102);
/// Épaisseur du contour du badge et taille de son chiffre, en unités monde.
const BADGE_STROKE: f32 = 0.8;
const BADGE_FONT: f32 = 10.0;

/// Les longueurs d'un dossier à l'échelle d'une passe.
struct Layout {
    width: f32,
    height: f32,
    radius: f32,
    header: f32,
}

impl Layout {
    fn new(f: &CanvasFolder, scale: WorldScale) -> Self {
        // Le minimum appartient au modèle, pas au dessin : un dossier plus petit que 180 × 120
        // serait illisible, et la fiche 06 § 8.1 le pose comme une propriété du cadre.
        let w = (f.width as f32).max(MIN_WIDTH);
        let h = (f.height as f32).max(MIN_HEIGHT);
        Self {
            width: scale.world(w),
            height: scale.world(h),
            radius: scale.world(CORNER_RADIUS),
            header: scale.world(pick_consts::FOLDER_HEADER as f32),
        }
    }
}

/// Dessine tous les dossiers visibles du tableau actif.
///
/// Les dossiers ne passent pas par l'index spatial — il n'indexe que les images et les
/// annotations — donc la visibilité est décidée ici, par le même test de bord que les cartes.
/// Un tableau en compte quelques-uns, jamais des milliers : c'est le bon compromis tant que
/// l'index ne les connaît pas.
pub(super) fn draw_folders(kit: PaintKit<'_>, pixmap: &mut PixmapMut, store: &Store, pass: ViewPass<'_>) {
    let Some(board) = store.active_board() else {
        return;
    };
    if board.folders.is_empty() {
        return;
    }
    let PaintKit { typography, theme, .. } = kit;
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    for f in &board.folders {
        let layout = Layout::new(f, scale);
        let (wx, wy) = world_to_screen(f.x, f.y, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        if clip.rejects(sx, sy, layout.width, layout.height) {
            continue;
        }
        let tint = parse_hex_color(&f.color, 136, 136, 136);
        let selected = store.selected_folder_id.as_deref() == Some(f.id.as_str());

        draw_frame(pixmap, (sx, sy), &layout, tint, (selected, scale));
        if scale.draws_detail() {
            draw_header(typography, pixmap, (sx, sy), &layout, f, (tint, selected, scale));
            let _ = theme;
        }
        if selected {
            draw_resize_handles(pixmap, theme, scale, (sx, sy, layout.width, layout.height), &Handle::ALL);
        }
    }
}

/// Le cadre : halo de sélection, corps teinté, bordure, séparation du bandeau.
fn draw_frame(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &Layout,
    tint: (u8, u8, u8),
    state: (bool, WorldScale),
) {
    let (selected, scale) = state;
    let (r, g, b) = tint;
    let pick = |paire: (u8, u8)| if selected { paire.1 } else { paire.0 };

    if selected && scale.draws_detail() {
        let pad = scale.world(GLOW.0);
        let mut pb = PathBuilder::new();
        push_rounded_rect(
            &mut pb,
            at.0 - pad,
            at.1 - pad,
            layout.width + pad * 2.0,
            layout.height + pad * 2.0,
            scale.world(GLOW.1),
        );
        if let Some(path) = pb.finish() {
            let mut paint = Paint { anti_alias: true, ..Default::default() };
            paint.set_color(Color::from_rgba8(r, g, b, GLOW.3));
            let stroke = Stroke { width: scale.world(GLOW.2), ..Default::default() };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, at.0, at.1, layout.width, layout.height, layout.radius);
    let Some(path) = pb.finish() else {
        return;
    };

    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(Color::from_rgba8(r, g, b, pick(BODY_ALPHA)));
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

    let mut border = Paint { anti_alias: true, ..Default::default() };
    border.set_color(Color::from_rgba8(r, g, b, pick(BORDER_ALPHA)));
    let stroke = Stroke {
        width: scale.world(if selected { BORDER_WIDTH.1 } else { BORDER_WIDTH.0 }),
        dash: if selected {
            None
        } else {
            tiny_skia::StrokeDash::new(
                vec![scale.world(BORDER_DASH.0), scale.world(BORDER_DASH.1)],
                0.0,
            )
        },
        line_cap: LineCap::Butt,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, Transform::identity(), None);

    // Le trait qui sépare le bandeau du corps : c'est lui qui rend le portail lisible comme
    // une fenêtre plutôt que comme un simple rectangle.
    if let Some(line) = Rect::from_xywh(at.0, at.1 + layout.header, layout.width, scale.world(1.0)) {
        let mut sep = Paint { anti_alias: true, ..Default::default() };
        sep.set_color(Color::from_rgba8(r, g, b, pick(BORDER_ALPHA)));
        pixmap.fill_rect(line, &sep, Transform::identity(), None);
    }
}

/// Le bandeau : icône, titre tronqué, badge compteur.
fn draw_header(
    typography: &crate::typography::Typography,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &Layout,
    f: &CanvasFolder,
    state: ((u8, u8, u8), bool, WorldScale),
) {
    let (tint, selected, scale) = state;
    let (r, g, b) = tint;
    let alpha = if selected { ICON_ALPHA.1 } else { ICON_ALPHA.0 };

    // L'icône d'explorateur : un onglet posé sur un corps, réduit à deux rectangles arrondis.
    // Le dessin exact de la référence est vectoriel ; ces deux formes en portent la silhouette
    // et se lisent à toute échelle où le détail est dessiné.
    let (ix, iy) = (at.0 + scale.world(ICON.0), at.1 + scale.world(ICON.1));
    let (iw, ih) = (scale.world(ICON.2), scale.world(ICON.3));
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, ix, iy + ih * 0.25, iw, ih * 0.75, scale.world(2.0));
    push_rounded_rect(&mut pb, ix, iy, iw * 0.45, ih * 0.35, scale.world(1.0));
    if let Some(path) = pb.finish() {
        let mut paint = Paint { anti_alias: true, ..Default::default() };
        paint.set_color(Color::from_rgba8(r, g, b, alpha));
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    let titre = truncate(&f.name, TITLE_MAX_CHARS);
    typography.draw_text(
        pixmap,
        &titre,
        at.0 + scale.world(TITLE.0),
        at.1 + scale.world(TITLE.1),
        TextStyle {
            size: scale.world(TITLE.2),
            color: Color::from_rgba8(r, g, b, if selected { 255 } else { 204 }),
            bold: true,
        },
    );

    draw_badge(typography, pixmap, at, layout, tint, scale);
}

/// Le badge compteur, en haut à droite du bandeau.
///
/// Il annonce ce que le dossier contient. Le compte réel demande d'ouvrir le tableau enfant,
/// ce que le rendu d'une frame ne peut pas faire : le badge montre donc le nombre de nœuds
/// **connus** du dossier, qui est zéro tant que le tableau enfant n'a pas été ouvert. Un
/// compte tenu à jour par le store est le pas suivant, et il appartient au noyau.
fn draw_badge(
    typography: &crate::typography::Typography,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &Layout,
    tint: (u8, u8, u8),
    scale: WorldScale,
) {
    let (r, g, b) = tint;
    let (bx, by) = (at.0 + layout.width - scale.world(BADGE.0), at.1 + scale.world(BADGE.1));
    let (bw, bh) = (scale.world(BADGE.2), scale.world(BADGE.3));
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, bx, by, bw, bh, scale.world(BADGE.4));
    let Some(path) = pb.finish() else {
        return;
    };
    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(Color::from_rgba8(r, g, b, BADGE_ALPHA.0));
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

    let mut border = Paint { anti_alias: true, ..Default::default() };
    border.set_color(Color::from_rgba8(r, g, b, BADGE_ALPHA.1));
    let stroke = Stroke { width: scale.world(BADGE_STROKE), ..Default::default() };
    pixmap.stroke_path(&path, &border, &stroke, Transform::identity(), None);

    typography.draw_text(
        pixmap,
        "0",
        bx + bw * 0.4,
        by + bh * 0.75,
        TextStyle { size: scale.world(BADGE_FONT), color: Color::from_rgba8(r, g, b, 204), bold: true },
    );
}

/// Tronque un titre à `max` caractères, ellipse comprise.
///
/// Le compte porte sur les **caractères** et non sur les octets : un titre accentué serait
/// coupé au milieu d'un caractère par une troncature sur les octets, et la chaîne cesserait
/// d'être du texte.
pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}

#[cfg(test)]
mod tests;
