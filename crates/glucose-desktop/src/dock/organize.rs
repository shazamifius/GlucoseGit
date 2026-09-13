//! Le panneau ORDONNER : un tri, une disposition, une largeur cible, un espacement — et
//! `Appliquer`, qui réarrange les images du tableau en un geste annulable.
//!
//! # Ce que le panneau dit, et ce qu'il fait
//!
//! Trois tris de la référence — par couleur, du sombre au clair, du clair au sombre — et la
//! disposition « par slot preset » demandent ce que l'application n'a pas encore : les pixels
//! des images pour les premiers, des presets posés pour la seconde. Leurs boutons étaient là,
//! s'allumaient au clic, et retombaient en silence sur « Ordre actuel » et « Rangées
//! compactes ». Ils sont désormais **grisés et inertes** (fiche 05 § 5.4) : ce que
//! [`SortType::implemented`] et [`LayoutMode::implemented`] disent, le panneau le montre.
//!
//! # Une seule géométrie (loi L4)
//!
//! [`layout_organize_panel`] produit la liste des rectangles ; le dessin la lit, le clic la
//! lit. R-37 était exactement ce panneau : deux formules pour la largeur d'un bouton, dont une
//! comptait les octets de « Sombre → Clair » et se trompait de 26 px.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};
use crate::typography::Typography;
use glucose_core::layout::{OrganizeMode, OrganizeSort};
use glucose_core::types::BoardImage;

pub mod paint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortType {
    None,
    Color,
    SizeDesc,
    SizeAsc,
    RatioPort,
    RatioLand,
    LumAsc,
    LumDesc,
}

impl SortType {
    /// Les huit tris de la référence, dans l'ordre du panneau (fiche 10 § 1.2).
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::Color,
        Self::SizeDesc,
        Self::SizeAsc,
        Self::RatioPort,
        Self::RatioLand,
        Self::LumAsc,
        Self::LumDesc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Ordre actuel",
            Self::Color => "Couleur",
            Self::SizeDesc => "Grand → Petit",
            Self::SizeAsc => "Petit → Grand",
            Self::RatioPort => "Portrait",
            Self::RatioLand => "Paysage",
            Self::LumAsc => "Sombre → Clair",
            Self::LumDesc => "Clair → Sombre",
        }
    }

    /// Le tri que le noyau sait faire, s'il le sait. Trier par couleur ou par luminance
    /// demande les pixels des images, que le noyau n'a pas (fiche 12, chantier 3.D).
    pub fn core_sort(self) -> Option<OrganizeSort> {
        match self {
            Self::None => Some(OrganizeSort::None),
            Self::SizeDesc => Some(OrganizeSort::SizeDesc),
            Self::SizeAsc => Some(OrganizeSort::SizeAsc),
            Self::RatioPort => Some(OrganizeSort::RatioPortrait),
            Self::RatioLand => Some(OrganizeSort::RatioLandscape),
            Self::Color | Self::LumAsc | Self::LumDesc => None,
        }
    }

    pub fn implemented(self) -> bool {
        self.core_sort().is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Compact,
    Masonry,
    Grid,
    SameHeight,
    BySlot,
}

impl LayoutMode {
    /// Les cinq dispositions de la référence, dans l'ordre du panneau (fiche 10 § 1.2).
    pub const ALL: [Self; 5] = [
        Self::Compact,
        Self::Masonry,
        Self::Grid,
        Self::SameHeight,
        Self::BySlot,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Compact => "Rangées compactes",
            Self::Masonry => "Masonry (colonnes)",
            Self::Grid => "Grille alignée",
            Self::SameHeight => "Même hauteur",
            Self::BySlot => "Par slot preset",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Compact => "Respecte les ratios, remplit chaque ligne",
            Self::Masonry => "Colonnes indépendantes — Pinterest",
            Self::Grid => "Même largeur par colonne, ratios conservés",
            Self::SameHeight => "Hauteur fixe, largeur proportionnelle",
            Self::BySlot => "Colonnes séparées par catégorie",
        }
    }

    /// La disposition que le noyau sait faire, s'il la sait. « Par slot preset » attend les
    /// presets posés sur un tableau (fiche 12, chantier 3.G).
    pub fn core_mode(self) -> Option<OrganizeMode> {
        match self {
            Self::Compact => Some(OrganizeMode::CompactRows),
            Self::Masonry => Some(OrganizeMode::Masonry),
            Self::Grid => Some(OrganizeMode::Grid),
            Self::SameHeight => Some(OrganizeMode::SameHeight),
            Self::BySlot => None,
        }
    }

    pub fn implemented(self) -> bool {
        self.core_mode().is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizeState {
    pub sort_by: SortType,
    pub layout: LayoutMode,
    pub size: f64,
    pub gap: f64,
    pub cols: usize,
}

impl Default for OrganizeState {
    fn default() -> Self {
        Self {
            sort_by: SortType::None,
            layout: LayoutMode::Compact,
            size: 280.0,
            gap: 16.0,
            cols: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrganizeSortButton {
    pub sort_type: SortType,
    pub rect: WidgetRect,
}

#[derive(Debug, Clone)]
pub struct OrganizeLayoutModeItem {
    pub mode: LayoutMode,
    pub rect: WidgetRect,
}

#[derive(Debug, Clone)]
pub struct OrganizePanelLayout {
    pub target_count_rect: WidgetRect,
    pub sort_buttons: Vec<OrganizeSortButton>,
    pub layout_modes: Vec<OrganizeLayoutModeItem>,
    pub target_size_rect: WidgetRect,
    pub gap_rect: WidgetRect,
    pub apply_rect: WidgetRect,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_organize_panel(frame: ScaledRect, typo: &Typography) -> OrganizePanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw, ph) = (frame.x, frame.y, frame.w, frame.h);
    let pad_x = 14.0 * s;
    let inner_w = pw - 28.0 * s;
    let target_count_rect = WidgetRect::new(px + pad_x, py + 38.0 * s, inner_w, 24.0 * s);
    let mut cy = py + (38.0 + 32.0 + 14.0) * s;

    // Les boutons de tri mesurent leur libellé et passent à la ligne quand il le faut.
    let mut sort_buttons = Vec::with_capacity(SortType::ALL.len());
    let mut sx = px + pad_x;
    for sort_type in SortType::ALL {
        let (tw, _) = typo.measure_text(sort_type.label(), 10.0 * s, false);
        let bw = tw + 14.0 * s;
        if sx + bw > px + pw - pad_x {
            sx = px + pad_x;
            cy += 22.0 * s;
        }
        sort_buttons.push(OrganizeSortButton {
            sort_type,
            rect: WidgetRect::new(sx, cy, bw, 18.0 * s),
        });
        sx += bw + 4.0 * s;
    }
    cy += (28.0 + 14.0) * s;

    let row_h = 30.0 * s;
    let layout_modes = LayoutMode::ALL
        .into_iter()
        .enumerate()
        .map(|(i, mode)| OrganizeLayoutModeItem {
            mode,
            rect: WidgetRect::new(
                px + pad_x,
                cy + i as f32 * (row_h + 2.0 * s),
                inner_w,
                row_h,
            ),
        })
        .collect();
    cy += LayoutMode::ALL.len() as f32 * (row_h + 2.0 * s) + (6.0 + 12.0) * s;

    let col_w = (inner_w - 8.0 * s) / 2.0;
    let target_size_rect = WidgetRect::new(px + pad_x, cy, col_w, 20.0 * s);
    let gap_rect = WidgetRect::new(px + pad_x + col_w + 8.0 * s, cy, col_w, 20.0 * s);
    let apply_rect = WidgetRect::new(px + pad_x, py + ph - 38.0 * s, inner_w, 26.0 * s);

    OrganizePanelLayout {
        target_count_rect,
        sort_buttons,
        layout_modes,
        target_size_rect,
        gap_rect,
        apply_rect,
    }
}

/// Ce qu'un clic dans le panneau demande.
#[derive(Debug, Clone, PartialEq)]
pub enum OrganizeClick {
    /// Le panneau a changé son propre état, ou le clic n'a touché aucun bouton.
    Handled,
    /// `Appliquer` : réarranger les images du tableau avec cet état.
    Apply(OrganizeState),
}

/// Le clic : un bouton inerte ne change rien, un tri ou une disposition change l'état,
/// `Appliquer` demande la disposition.
pub fn click_organize_panel(
    state: &mut OrganizeState,
    layout: &OrganizePanelLayout,
    pointer: Pointer,
) -> OrganizeClick {
    let hit = |r: WidgetRect| r.contains(pointer.x, pointer.y);
    if let Some(btn) = layout.sort_buttons.iter().find(|b| hit(b.rect)) {
        if btn.sort_type.implemented() {
            state.sort_by = btn.sort_type;
        }
        return OrganizeClick::Handled;
    }
    if let Some(item) = layout.layout_modes.iter().find(|i| hit(i.rect)) {
        if item.mode.implemented() {
            state.layout = item.mode;
        }
        return OrganizeClick::Handled;
    }
    if hit(layout.apply_rect) {
        return OrganizeClick::Apply(state.clone());
    }
    OrganizeClick::Handled
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// La disposition calculée par le noyau pour cet état. Un tri ou une disposition que le
/// noyau ne sait pas faire ne peut pas être choisi (voir [`click_organize_panel`]) ; s'il
/// l'était tout de même, on retombe sur l'ordre courant et les rangées compactes.
pub fn apply_organize_layout(images: &[BoardImage], state: &OrganizeState) -> Vec<LayoutResult> {
    let mode = state
        .layout
        .core_mode()
        .unwrap_or(OrganizeMode::CompactRows);
    let sort = state.sort_by.core_sort().unwrap_or(OrganizeSort::None);
    glucose_core::layout::calculate_image_layout(
        images, mode, sort, state.size, state.gap, state.cols,
    )
    .into_iter()
    .map(|r| LayoutResult {
        id: r.id,
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    })
    .collect()
}
