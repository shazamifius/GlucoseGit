//! La barre d'action contextuelle (fiche 10 § 3, capture `00_user_production_board`).
//!
//! # Ce qu'elle est
//!
//! Une pastille flottante en bas de l'écran, centrée, qui n'apparaît **que** lorsque quelque
//! chose est sélectionné : le compte de ce qui l'est, puis les gestes qu'on veut faire sur
//! une sélection sans rien apprendre par cœur — verrouiller, supprimer.
//!
//! Elle ne fait rien que le clavier ne fasse déjà (`L`, `Suppr`). C'est exactement sa raison
//! d'être : un raccourci ne se découvre pas, un bouton si. Le libellé du bouton porte
//! d'ailleurs sa touche, comme dans la référence.
//!
//! # La mise en page est une fonction pure
//!
//! [`layout_action_bar`] rend les rectangles sans rien dessiner ; le tracé et le clic lisent
//! la même liste, donc ne peuvent pas viser des endroits différents (loi L4). C'est la
//! discipline de `layout_topbar`, `layout_minimap` et [`super::breadcrumb`].
//!
//! # Ce qui manque encore, et qui est dans la référence
//!
//! Les **tags** d'une image, quand elle est seule sélectionnée : une rangée de pastilles et
//! un champ de saisie, entre le verrou et la suppression. Le modèle porte `tags`, rien ne les
//! montre — c'est un chantier à part, pas un oubli de celui-ci.

use crate::icons::{draw_icon_scaled, IconType};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::store::Store;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

// ── Les mesures des barres du bas, en points (pixels logiques) ─────────────────
//
// Elles suivaient la fiche 10 à la lettre : un corps de 11, des boutons plats hauts de 15, une
// barre de 23. Il les a trouvées **beaucoup trop petites**, et un bouton plat ne se lisait pas
// comme un bouton (« on ne sait même pas que c'est un bouton »). Celles de Glucose Tauri
// faisaient environ 37 points, chaque bouton encadré. Les barres du bas en prennent la
// mesure et l'habit : un corps de 13, des boutons encadrés de 23, une barre de 37 — sous la
// barre du haut (44), dont elles restent les cadettes.

/// Distance entre le bas de la fenêtre et celui de la barre.
const BOTTOM: f32 = 14.0;
/// Marges intérieures de la pastille.
pub(super) const PAD_X: f32 = 10.0;
pub(super) const PAD_Y: f32 = 7.0;
/// Rayon de ses coins.
pub(super) const RADIUS: f32 = 8.0;
/// Écart entre deux éléments de la rangée.
pub(super) const GAP: f32 = 6.0;
/// Corps du texte.
pub(super) const FONT: f32 = 13.0;
/// Retrait entre le compteur et son filet séparateur.
const COUNT_PAD: f32 = 8.0;
/// Marges intérieures d'un bouton.
pub(super) const BTN_PAD_X: f32 = 10.0;
pub(super) const BTN_PAD_Y: f32 = 5.0;
/// Rayon des coins d'un bouton.
pub(super) const BTN_RADIUS: f32 = 5.0;
/// Côté de l'icône d'un bouton.
pub(super) const ICON: f32 = 13.0;
/// Écart entre l'icône et son libellé.
pub(super) const ICON_GAP: f32 = 6.0;
/// Épaisseur du trait d'une icône, rapportée à sa boîte de 14 unités.
pub(super) const ICON_STROKE: f32 = 1.3;

/// Ce qu'un clic sur la barre demande.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionBarClick {
    /// Bascule le verrou des images sélectionnées.
    ToggleLock,
    /// Supprime toute la sélection.
    Delete,
}

/// Un bouton de la barre : sa boîte, son icône, son libellé, et ce qu'il demande.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionButton {
    pub click: ActionBarClick,
    pub rect: (f32, f32, f32, f32),
    pub icon: IconType,
    pub label: &'static str,
    /// Vrai quand le bouton **est** l'état qu'il décrit — « Verrouillé » plutôt que
    /// « Verrouiller ». Il porte alors le rouge de l'interface, fond et filet compris.
    pub on: bool,
}

/// La barre, une fois placée.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionBar {
    /// La pastille entière.
    pub rect: (f32, f32, f32, f32),
    /// « 3 sélectionnés », déjà accordé.
    pub count_label: String,
    /// L'abscisse du filet qui sépare le compte des actions.
    pub separator_x: f32,
    pub buttons: Vec<ActionButton>,
}

/// La barre pour la sélection courante, ou `None` s'il n'y a rien à montrer.
///
/// Fonction pure : elle ne lit que le store et la typographie, et ne dessine rien.
pub fn layout_action_bar(
    store: &Store,
    typography: &Typography,
    screen: (f32, f32),
    scale: f32,
) -> Option<ActionBar> {
    let board = store.active_board()?;
    let images = store.selected_image_ids.len();
    let total = images + store.selected_annotation_ids.len();
    if total == 0 {
        return None;
    }

    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let pluriel = if total > 1 { "s" } else { "" };
    let count_label = format!("{total} sélectionné{pluriel}");
    let (count_w, _) = typography.measure_text(&count_label, font, Face::Regular);

    // Le bouton du verrou n'existe que s'il a des images à fermer : une carte ou un dossier
    // n'en porte pas (fiche 08 § 1.3).
    let mut defs: Vec<(ActionBarClick, IconType, &'static str, bool)> = Vec::with_capacity(2);
    if images > 0 {
        let toutes_fermees = board
            .images
            .iter()
            .filter(|i| store.selected_image_ids.contains(&i.id))
            .all(|i| i.locked);
        let (icon, label) = if toutes_fermees {
            (IconType::Lock, "Verrouillé")
        } else {
            (IconType::Unlock, "Verrouiller")
        };
        defs.push((ActionBarClick::ToggleLock, icon, label, toutes_fermees));
    }
    defs.push((ActionBarClick::Delete, IconType::Trash, "Supprimer", false));

    let bouton_w = |label: &str| {
        let (w, _) = typography.measure_text(label, font, Face::Regular);
        ICON * s + ICON_GAP * s + w + BTN_PAD_X * s * 2.0
    };
    let hauteur_btn = font + BTN_PAD_Y * s * 2.0;
    let hauteur = hauteur_btn + PAD_Y * s * 2.0;

    let largeur = PAD_X * s * 2.0
        + count_w
        + COUNT_PAD * s
        + defs
            .iter()
            .map(|(_, _, label, _)| bouton_w(label) + GAP * s)
            .sum::<f32>();

    let x = (screen.0 - largeur) / 2.0;
    let y = screen.1 - BOTTOM * s - hauteur;

    let mut curseur = x + PAD_X * s + count_w + COUNT_PAD * s;
    let separator_x = curseur;
    let mut buttons = Vec::with_capacity(defs.len());
    for (click, icon, label, on) in defs {
        curseur += GAP * s;
        let w = bouton_w(label);
        buttons.push(ActionButton {
            click,
            rect: (curseur, y + PAD_Y * s, w, hauteur_btn),
            icon,
            label,
            on,
        });
        curseur += w;
    }

    Some(ActionBar {
        rect: (x, y, largeur, hauteur),
        count_label,
        separator_x,
        buttons,
    })
}

/// Ce qu'un clic en `(px, py)` demande à la barre — `None` s'il tombe à côté.
pub fn hit_action_bar(bar: &ActionBar, px: f32, py: f32) -> Option<ActionBarClick> {
    bar.buttons
        .iter()
        .find(|b| dans(b.rect, px, py))
        .map(|b| b.click)
}

/// Le clic tombe-t-il dans la pastille, bouton ou pas ?
///
/// Ce qui tombe dessus ne doit **jamais** atteindre le canevas : sans cela, cliquer à côté
/// d'un bouton désélectionne — donc fait disparaître la barre sous le doigt.
pub fn covers(bar: &ActionBar, px: f32, py: f32) -> bool {
    dans(bar.rect, px, py)
}

pub(super) fn dans((x, y, w, h): (f32, f32, f32, f32), px: f32, py: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// Dessine la barre, si elle a lieu d'être.
pub fn draw_action_bar(
    pixmap: &mut PixmapMut,
    store: &Store,
    typography: &Typography,
    theme: &Theme,
    screen: (f32, f32),
    scale: f32,
) {
    let Some(bar) = layout_action_bar(store, typography, screen, scale) else {
        return;
    };
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;

    pastille(pixmap, bar.rect, s, theme);

    let base = bar.rect.1 + (bar.rect.3 - font) / 2.0;
    typography.draw_text(
        pixmap,
        &bar.count_label,
        bar.rect.0 + PAD_X * s,
        base,
        TextStyle {
            size: font,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
    filet_vertical(
        pixmap,
        bar.separator_x,
        bar.rect.1 + PAD_Y * s,
        bar.rect.3 - PAD_Y * s * 2.0,
        theme.border_medium,
    );

    for btn in &bar.buttons {
        let ink = if btn.on {
            theme.alert
        } else {
            theme.text_secondary
        };
        if btn.on {
            fond_arrondi(
                pixmap,
                btn.rect,
                BTN_RADIUS * s,
                theme.alert_bg,
                Some(theme.alert_border),
            );
        } else {
            bouton(pixmap, btn.rect, (s, false), theme);
        }
        let (bx, by, _, bh) = btn.rect;
        draw_icon_scaled(
            pixmap,
            btn.icon,
            bx + BTN_PAD_X * s,
            by + (bh - ICON * s) / 2.0,
            ICON * s,
            ink,
            ICON_STROKE,
        );
        typography.draw_text(
            pixmap,
            btn.label,
            bx + BTN_PAD_X * s + ICON * s + ICON_GAP * s,
            by + (bh - font) / 2.0,
            TextStyle {
                size: font,
                color: ink,
                face: Face::Regular,
            },
        );
    }
}

/// Un filet vertical d'un pixel, **posé sur la grille de pixels et sans anti-aliasage**.
///
/// Les deux vont ensemble : un trait d'un pixel à une abscisse fractionnaire s'étale sur deux
/// colonnes et devient un gris pâle — c'est R-46 appliqué à un filet plutôt qu'à un glyphe.
/// L'arrondir le garde net, et le sortir du chemin anti-aliasé évite au passage le tracé
/// « hairline » du rastériseur, qui n'aime pas les rectangles plus fins qu'un pixel.
pub(super) fn filet_vertical(pixmap: &mut PixmapMut, x: f32, y: f32, hauteur: f32, color: Color) {
    let Some(rect) = Rect::from_xywh(x.round(), y.round(), 1.0, hauteur.round().max(1.0)) else {
        return;
    };
    let mut paint = Paint {
        anti_alias: false,
        ..Default::default()
    };
    paint.set_color(color);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

/// **La pastille d'une barre du bas** : le panneau sombre de Tauri (`#111`), bordé (`#2a2a2a`).
pub(super) fn pastille(pixmap: &mut PixmapMut, rect: (f32, f32, f32, f32), s: f32, theme: &Theme) {
    fond_arrondi(
        pixmap,
        rect,
        RADIUS * s,
        theme.bg_panel,
        Some(theme.border_medium),
    );
}

/// **Un bouton d'une barre du bas** : une boîte encadrée — ce qui le fait lire comme un
/// bouton. Allumé, il s'éclaire (`#2d2d2d`, filet `#444`), comme l'état actif de Tauri.
pub(super) fn bouton(
    pixmap: &mut PixmapMut,
    rect: (f32, f32, f32, f32),
    (s, allume): (f32, bool),
    theme: &Theme,
) {
    let (fond, filet) = if allume {
        (theme.bg_active, theme.border_accent)
    } else {
        (theme.btn_bg, theme.btn_border)
    };
    fond_arrondi(pixmap, rect, BTN_RADIUS * s, fond, Some(filet));
}

/// Une pastille : son fond, puis son filet.
pub(super) fn fond_arrondi(
    pixmap: &mut PixmapMut,
    (x, y, w, h): (f32, f32, f32, f32),
    radius: f32,
    fill: Color,
    border: Option<Color>,
) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(fill);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
    if let Some(c) = border {
        paint.set_color(c);
        pixmap.stroke_path(
            &path,
            &paint,
            &tiny_skia::Stroke {
                width: 1.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests;
