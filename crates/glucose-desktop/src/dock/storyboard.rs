//! Le panneau STORYBOARD — une **façade honnête** (fiche 09 § 8).
//!
//! Le format, la largeur, les colonnes, l'espacement, la grille de cellules et le bouton
//! `Activer` sont ceux de la référence. Aucun ne touche encore au canevas : le modèle porte
//! `StoryboardPanel`, le store sait ajouter et mettre à jour des panneaux, et rien ne les
//! dessine. Le bouton le dit au clic, et ne reste pas allumé (fiche 12, chantier 3.G).

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};

pub mod paint;

/// Les cinq cadrages de la référence (fiche 10 § 5.3).
pub const FORMATS: [&str; 5] = [
    "16:9 — Cinéma HD",
    "4:3 — Classique",
    "2.35:1 — Scope",
    "1:1 — Carré",
    "9:16 — Vertical",
];

/// La grille de cellules montrée : quatre colonnes sur deux rangées.
const GRID: (usize, usize) = (4, 2);

#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardState {
    pub format_idx: usize,
    pub panel_width: f64,
    pub cols: usize,
    pub gap: f64,
    /// Le storyboard est « activé » — ce qui, tant que rien ne le dessine, ne dure pas : le
    /// clic qui l'allume l'éteint aussitôt en disant pourquoi.
    pub active: bool,
}

impl Default for StoryboardState {
    fn default() -> Self {
        Self {
            format_idx: 0,
            panel_width: 280.0,
            cols: 4,
            gap: 24.0,
            active: false,
        }
    }
}

impl StoryboardState {
    pub fn format_label(&self) -> &'static str {
        FORMATS[self.format_idx.min(FORMATS.len() - 1)]
    }
}

#[derive(Debug, Clone)]
pub struct StoryboardPanelLayout {
    pub format_button: WidgetRect,
    pub width_input: WidgetRect,
    pub cols_input: WidgetRect,
    pub gap_input: WidgetRect,
    pub cells: Vec<WidgetRect>,
    pub activate_button: WidgetRect,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_storyboard_panel(frame: ScaledRect) -> StoryboardPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw, ph) = (frame.x, frame.y, frame.w, frame.h);
    let pad_x = 14.0 * s;
    let inner_w = pw - 28.0 * s;
    let format_button = WidgetRect::new(px + pad_x, py + 52.0 * s, inner_w, 24.0 * s);

    let cy = py + (84.0 + 12.0) * s;
    let col_w = (inner_w - 12.0 * s) / 3.0;
    let column = |i: f32| px + pad_x + i * (col_w + 6.0 * s);
    let width_input = WidgetRect::new(column(0.0), cy, col_w, 20.0 * s);
    let cols_input = WidgetRect::new(column(1.0), cy, col_w, 20.0 * s);
    let gap_input = WidgetRect::new(column(2.0), cy, col_w, 20.0 * s);

    let (gcols, grows) = GRID;
    let cell_y = cy + 30.0 * s;
    let cell_w = (inner_w - (gcols - 1) as f32 * 3.0 * s) / gcols as f32;
    let cell_h = 24.0 * s;
    let cells = (0..grows)
        .flat_map(|r| (0..gcols).map(move |c| (r, c)))
        .map(|(r, c)| {
            WidgetRect::new(
                px + pad_x + c as f32 * (cell_w + 3.0 * s),
                cell_y + r as f32 * (cell_h + 3.0 * s),
                cell_w,
                cell_h,
            )
        })
        .collect();

    let activate_button = WidgetRect::new(px + pad_x, py + ph - 38.0 * s, inner_w, 28.0 * s);

    StoryboardPanelLayout {
        format_button,
        width_input,
        cols_input,
        gap_input,
        cells,
        activate_button,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryboardClick {
    Handled,
    /// Le format a changé — cyclé au clic, comme la référence.
    FormatSelected(usize),
    /// `Activer` — que l'application refuse tant que rien ne dessine un storyboard.
    ToggleRequested,
}

pub fn click_storyboard_panel(
    state: &mut StoryboardState,
    layout: &StoryboardPanelLayout,
    pointer: Pointer,
) -> StoryboardClick {
    let hit = |r: WidgetRect| r.contains(pointer.x, pointer.y);
    if hit(layout.format_button) {
        state.format_idx = (state.format_idx + 1) % FORMATS.len();
        return StoryboardClick::FormatSelected(state.format_idx);
    }
    if hit(layout.activate_button) {
        state.active = !state.active;
        return StoryboardClick::ToggleRequested;
    }
    StoryboardClick::Handled
}
