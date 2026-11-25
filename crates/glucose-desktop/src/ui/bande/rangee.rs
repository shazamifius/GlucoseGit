//! **La rangée des onglets** (sortie de `bande.rs`, 25/11) : les onglets, le « + », le champ de
//! renommage, et le témoin du partage (SYNCHRO-1).

use super::super::onglets::{self, TabButtonLayout};
use super::super::{Synchro, UiState};
use crate::icons::{draw_icon_scaled, IconType};
use crate::renderer::scale::fill_crisp;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{PixmapMut, Rect};

pub(super) fn render_board_tabs(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    width: f32,
    onglets: &[TabButtonLayout],
    survole: Option<usize>,
) {
    let s = ui.scale();
    let y_start = ui.topbar_height();
    let tabs_h = ui.tabs_height();

    if let Some(rect) = Rect::from_xywh(0.0, y_start, width, tabs_h) {
        fill_crisp(pixmap, rect, theme.bg_canvas);
    }
    if let Some(rect) = Rect::from_xywh(0.0, y_start + tabs_h - 1.0, width, 1.0) {
        fill_crisp(pixmap, rect, theme.border_subtle);
    }

    let etat = &ui.onglets;
    let glisse = etat
        .tenu
        .as_ref()
        .filter(|t| t.glisse)
        .map(|t| t.id.as_str());
    for (i, tab) in onglets.iter().enumerate() {
        let is_hover = survole == Some(i);
        // Pendant un glisser, l'onglet survolé est la place d'arrivée : il porte le trait.
        let arrivee = glisse.is_some_and(|g| is_hover && !tab.is_plus && tab.board_id != g);
        if tab.is_plus {
            if is_hover {
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 5.0 * s, 24.0 * s, 24.0 * s) {
                    fill_crisp(pixmap, rect, theme.bg_hover);
                }
            }
            draw_icon_scaled(
                pixmap,
                IconType::Plus,
                tab.x + 5.0 * s,
                y_start + 10.0 * s,
                14.0 * s,
                theme.text_muted,
                1.5 * s,
            );
        } else {
            let tenu = glisse == Some(tab.board_id.as_str());
            dessiner_un_onglet(
                pixmap,
                typo,
                theme,
                tab,
                (is_hover, tenu),
                (s, y_start, tabs_h),
            );
            if let Some((_, entree)) = etat.renomme.as_ref().filter(|(id, _)| *id == tab.board_id) {
                dessiner_le_champ(pixmap, typo, theme, tab, entree, (s, y_start, tabs_h));
            }
            if arrivee {
                if let Some(r) =
                    Rect::from_xywh(tab.x, y_start + tabs_h - 2.0 * s, tab.width, 2.0 * s)
                {
                    fill_crisp(pixmap, r, theme.text_muted);
                }
            }
        }
    }
    dessiner_le_temoin(pixmap, ui, typo, theme, width);
}

/// **Le témoin du partage** (SYNCHRO-1), au bout de la rangée : un point et un mot. Coupé, ou
/// sous un code qui ne fait que regarder, il prend la couleur du danger, et dit combien de gestes
/// attendent — ce qu'on écrit alors reste ici, et part au retour du lien. Son fond couvre un
/// onglet qui irait jusque-là : il se lit toujours.
fn dessiner_le_temoin(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    largeur: f32,
) {
    let Some(synchro) = ui.synchro else {
        return;
    };
    let (mot, couleur, plein) = match synchro {
        Synchro::Reliee => ("synchro".to_string(), theme.text_muted, true),
        Synchro::EnRoute => ("synchro…".to_string(), theme.text_muted, false),
        Synchro::Regarde => (
            "lecture seule : tes gestes restent ici".into(),
            theme.danger,
            false,
        ),
        Synchro::Coupee(0) => ("synchro coupée".into(), theme.danger, false),
        Synchro::Coupee(n) => (
            format!("synchro coupée · {n} en attente"),
            theme.danger,
            false,
        ),
    };
    let (s, y, h) = (ui.scale(), ui.topbar_height(), ui.tabs_height());
    let corps = onglets::CORPS * s;
    let x = largeur - 14.0 * s - typo.measure_text(&mot, corps, Face::Regular).0;
    let (cx, cy) = (x - 10.0 * s, y + h / 2.0);
    if let Some(fond) = Rect::from_xywh(cx - 10.0 * s, y, largeur - cx + 10.0 * s, h - 1.0) {
        fill_crisp(pixmap, fond, theme.bg_canvas);
    }
    let mut peinture = tiny_skia::Paint::default();
    peinture.set_color(couleur);
    peinture.anti_alias = true;
    if let Some(rond) = tiny_skia::PathBuilder::from_circle(cx, cy, 3.5 * s) {
        let t = tiny_skia::Transform::identity();
        if plein {
            pixmap.fill_path(&rond, &peinture, tiny_skia::FillRule::Winding, t, None);
        } else {
            let trait_ = tiny_skia::Stroke {
                width: 1.2 * s,
                ..Default::default()
            };
            pixmap.stroke_path(&rond, &peinture, &trait_, t, None);
        }
    }
    let style = TextStyle {
        size: corps,
        color: couleur,
        face: Face::Regular,
    };
    typo.draw_text(pixmap, &mot, x, y + 10.0 * s, style);
}

/// **Le champ de renommage**, à la place du nom : le fond et le filet du champ de Glucose
/// Tauri, le texte tapé, et le curseur.
fn dessiner_le_champ(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    tab: &TabButtonLayout,
    entree: &crate::interactions::text_entry::TextEntry,
    (s, y_start, tabs_h): (f32, f32, f32),
) {
    let (x, h) = (tab.x + 10.0 * s, tabs_h - 14.0 * s);
    let largeur = onglets::CHAMP * s + 8.0 * s;
    if let Some(r) = Rect::from_xywh(x, y_start + 7.0 * s, largeur, h) {
        fill_crisp(pixmap, r, theme.bg_active);
    }
    for (bx, by, bw, bh) in [
        (x, y_start + 7.0 * s, largeur, 1.0),
        (x, y_start + 7.0 * s + h - 1.0, largeur, 1.0),
        (x, y_start + 7.0 * s, 1.0, h),
        (x + largeur - 1.0, y_start + 7.0 * s, 1.0, h),
    ] {
        if let Some(r) = Rect::from_xywh(bx, by, bw, bh) {
            fill_crisp(pixmap, r, theme.border_medium);
        }
    }
    let corps = onglets::CORPS * s;
    let style = TextStyle {
        size: corps,
        color: theme.text_primary,
        face: Face::Regular,
    };
    typo.draw_text(
        pixmap,
        entree.text(),
        x + 4.0 * s,
        y_start + 10.0 * s,
        style,
    );
    let (avant, _) = typo.measure_text(entree.before_cursor(), corps, Face::Regular);
    if let Some(r) = Rect::from_xywh(x + 4.0 * s + avant, y_start + 10.0 * s, 1.0, corps * 1.2) {
        fill_crisp(pixmap, r, theme.text_primary);
    }
}

/// Un onglet nommé : son survol, son nom, et le trait de l'onglet actif.
fn dessiner_un_onglet(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    tab: &TabButtonLayout,
    (is_hover, tenu): (bool, bool),
    (s, y_start, tabs_h): (f32, f32, f32),
) {
    if is_hover && !tab.is_active && !tenu {
        if let Some(rect) = Rect::from_xywh(tab.x, y_start + 4.0 * s, tab.width, tabs_h - 6.0 * s) {
            fill_crisp(pixmap, rect, theme.bg_hover);
        }
    }
    // L'onglet qu'on glisse s'efface à moitié, comme dans Glucose Tauri (`opacity: 0.5`).
    let text_color = if tenu {
        theme.text_muted
    } else if tab.is_active {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    if let Some((cx, cy, cote)) = tab.fermer {
        typo.draw_text(
            pixmap,
            "×",
            cx + cote * 0.2,
            cy - cote * 0.05,
            TextStyle {
                size: cote,
                color: if is_hover {
                    theme.text_secondary
                } else {
                    theme.text_muted
                },
                face: Face::Regular,
            },
        );
    }
    typo.draw_text(
        pixmap,
        &tab.name,
        tab.x + 14.0 * s,
        y_start + 10.0 * s,
        TextStyle {
            size: 12.0 * s,
            color: text_color,
            face: if tab.is_active {
                Face::Bold
            } else {
                Face::Regular
            },
        },
    );
    // Fiche 06 § 10.2 : l'onglet actif porte une bordure inférieure de 2 px, blanc
    // pur — pas l'accent, qui n'est jamais décoratif.
    if tab.is_active {
        if let Some(rect) = Rect::from_xywh(tab.x, y_start + tabs_h - 2.0 * s, tab.width, 2.0 * s) {
            fill_crisp(pixmap, rect, theme.text_accent);
        }
    }
}
