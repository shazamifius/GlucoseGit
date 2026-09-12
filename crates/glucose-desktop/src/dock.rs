//! Module Dock & Panneaux Déroulants (PanelDock) en Rust Natif.
//!
//! | Sous-module | Panneau |
//! |---|---|
//! | [`domains`] | DOMAINES — une vue du catalogue du document (DOM-UI-1) |

pub mod domains;

use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use std::time::Instant;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    Organize,
    Pomodoro,
    Storyboard,
    Plugins,
    Preset,
    Domains,
}

impl TabId {
    pub fn anchor(&self) -> DockAnchor {
        match self {
            Self::Organize | Self::Pomodoro | Self::Storyboard => DockAnchor::BottomLeft,
            Self::Plugins | Self::Preset | Self::Domains => DockAnchor::TopLeft,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Organize => "ORDONNER",
            Self::Pomodoro => "POMODORO",
            Self::Storyboard => "STORYBOARD",
            Self::Plugins => "PLUGINS",
            Self::Preset => "PRESETS",
            Self::Domains => "DOMAINES",
        }
    }

    /// Largeurs de la fiche 10 § 4 : Domaines 320, Presets 280, Plugins 340 en haut ;
    /// Ordonner 250, Storyboard 260, Pomodoro 160 au minimum en bas.
    pub fn default_width(&self) -> f32 {
        match self {
            Self::Organize => 250.0,
            Self::Pomodoro => 170.0,
            Self::Storyboard => 260.0,
            Self::Plugins => 340.0,
            Self::Preset => 280.0,
            Self::Domains => 320.0,
        }
    }

    pub fn default_height(&self) -> f32 {
        match self {
            Self::Organize => 410.0,
            Self::Pomodoro => 210.0,
            Self::Storyboard => 340.0,
            Self::Plugins => 460.0,
            Self::Preset => 460.0,
            Self::Domains => 430.0,
        }
    }
}

/// Glissement d'un panneau vers sa sortie au-delà duquel il se ferme (fiche 10 § 4 : 80 px).
pub const DISMISS_DRAG_PX: f32 = 80.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockAnchor {
    TopLeft,
    BottomLeft,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Compact,
    Masonry,
    Grid,
    SameHeight,
    BySlot,
}

impl LayoutMode {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Compact => "Rangées compactes",
            Self::Masonry => "Masonry",
            Self::Grid => "Grille alignée",
            Self::SameHeight => "Même hauteur",
            Self::BySlot => "Par slot preset",
        }
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
pub struct PomodoroState {
    pub total_seconds: u32,
    pub left_seconds: u32,
    pub running: bool,
    pub last_tick: Instant,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            total_seconds: 25 * 60,
            left_seconds: 25 * 60,
            running: false,
            last_tick: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoryboardState {
    pub format_idx: usize,
    pub panel_width: f64,
    pub cols: usize,
    pub gap: f64,
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

#[derive(Debug, Clone)]
pub struct PluginsState {
    pub density_idx: usize,
    pub disposition_idx: usize,
    pub source_path: Option<String>,
}

impl Default for PluginsState {
    fn default() -> Self {
        Self {
            density_idx: 1,
            disposition_idx: 0,
            source_path: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PresetsState {
    pub active_preset: Option<String>,
    pub selected_category: usize,
}

#[derive(Debug, Clone)]
pub struct DragSession {
    pub tab: TabId,
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
}

#[derive(Debug, Clone)]
pub struct DockManager {
    pub top_tabs: Vec<TabId>,
    pub bottom_tabs: Vec<TabId>,
    pub drag: Option<DragSession>,
    pub organize: OrganizeState,
    pub pomodoro: PomodoroState,
    pub storyboard: StoryboardState,
    pub plugins: PluginsState,
    pub presets: PresetsState,
    /// L'état d'**interaction** du panneau DOMAINES : ce qui attend une confirmation, ce qui
    /// est en cours de frappe. Aucune donnée de domaine — celles-ci vivent dans le document
    /// (DOM-UI-1, `dock::domains`).
    pub domains: domains::DomainsUi,
}

/// `new()` n'est pas dérivable : l'état initial ouvre deux onglets bas.
impl Default for DockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockManager {
    pub fn new() -> Self {
        Self {
            top_tabs: Vec::new(),
            bottom_tabs: vec![TabId::Organize, TabId::Pomodoro],
            drag: None,
            organize: OrganizeState::default(),
            pomodoro: PomodoroState::default(),
            storyboard: StoryboardState::default(),
            plugins: PluginsState::default(),
            presets: PresetsState::default(),
            domains: domains::DomainsUi::default(),
        }
    }

    pub fn is_open(&self, tab: TabId) -> bool {
        match tab.anchor() {
            DockAnchor::TopLeft => self.top_tabs.contains(&tab),
            DockAnchor::BottomLeft => self.bottom_tabs.contains(&tab),
        }
    }

    pub fn toggle_tab(&mut self, tab: TabId) {
        let anchor = tab.anchor();
        let list = match anchor {
            DockAnchor::TopLeft => &mut self.top_tabs,
            DockAnchor::BottomLeft => &mut self.bottom_tabs,
        };

        if let Some(pos) = list.iter().position(|&t| t == tab) {
            list.remove(pos);
        } else {
            list.push(tab);
        }
    }

    pub fn dismiss_tab(&mut self, tab: TabId) {
        match tab.anchor() {
            DockAnchor::TopLeft => self.top_tabs.retain(|&t| t != tab),
            DockAnchor::BottomLeft => self.bottom_tabs.retain(|&t| t != tab),
        }
    }

    pub fn update_drag(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let (drag_tab, start_x) = match &self.drag {
            Some(d) => (d.tab, d.start_x),
            None => return false,
        };

        if let Some(ref mut d) = self.drag {
            d.current_x = mouse_x;
            d.current_y = mouse_y;
        }

        let anchor = drag_tab.anchor();
        let list = match anchor {
            DockAnchor::TopLeft => &mut self.top_tabs,
            DockAnchor::BottomLeft => &mut self.bottom_tabs,
        };

        let cur_idx = match list.iter().position(|&t| t == drag_tab) {
            Some(i) => i,
            None => return false,
        };

        // Calcule les positions nominales X
        let mut xs = Vec::with_capacity(list.len());
        let mut cur_x = 12.0f32;
        for &t in list.iter() {
            let w = t.default_width();
            xs.push((cur_x, w));
            cur_x += w + 10.0;
        }

        let vox = mouse_x - start_x;
        let dragged_center = xs[cur_idx].0 + xs[cur_idx].1 / 2.0 + vox;

        let mut swapped = false;
        // Swap avec le voisin de gauche si le centre dépasse le centre du voisin
        if cur_idx > 0 {
            let left_center = xs[cur_idx - 1].0 + xs[cur_idx - 1].1 / 2.0;
            if dragged_center < left_center {
                list.swap(cur_idx, cur_idx - 1);
                if let Some(ref mut d) = self.drag {
                    d.start_x -= xs[cur_idx - 1].1 + 10.0;
                }
                swapped = true;
            }
        }
        // Swap avec le voisin de droite si le centre dépasse le centre du voisin
        if !swapped && cur_idx + 1 < list.len() {
            let right_center = xs[cur_idx + 1].0 + xs[cur_idx + 1].1 / 2.0;
            if dragged_center > right_center {
                list.swap(cur_idx, cur_idx + 1);
                if let Some(ref mut d) = self.drag {
                    d.start_x += xs[cur_idx + 1].1 + 10.0;
                }
                swapped = true;
            }
        }

        swapped
    }

    /// Un panneau glissé de plus de [`DISMISS_DRAG_PX`] vers sa sortie — le haut pour le
    /// dock du haut, le bas pour celui du bas — se ferme (fiche 10 § 4).
    pub fn finish_drag(&mut self) -> Option<TabId> {
        let drag = self.drag.take()?;
        let dy = drag.current_y - drag.start_y;
        let should_dismiss = match drag.tab.anchor() {
            DockAnchor::TopLeft => dy < -DISMISS_DRAG_PX,
            DockAnchor::BottomLeft => dy > DISMISS_DRAG_PX,
        };

        if should_dismiss {
            self.dismiss_tab(drag.tab);
            Some(drag.tab)
        } else {
            None
        }
    }

    pub fn tick_pomodoro(&mut self) -> bool {
        if !self.pomodoro.running {
            return false;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.pomodoro.last_tick);
        if elapsed.as_secs() >= 1 {
            let secs = elapsed.as_secs() as u32;
            if self.pomodoro.left_seconds <= secs {
                self.pomodoro.left_seconds = 0;
                self.pomodoro.running = false;
            } else {
                self.pomodoro.left_seconds -= secs;
            }
            self.pomodoro.last_tick = now;
            return true;
        }
        false
    }
}

// ── Calcul de Géométrie et Positions des Panneaux ──────────────────────────

#[derive(Debug, Clone)]
pub struct PanelLayoutBox {
    pub tab: TabId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub grip_y: f32,
    pub grip_height: f32,
    pub is_dragged: bool,
    pub visual_offset_x: f32,
    pub visual_offset_y: f32,
}

impl PanelLayoutBox {
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        let vx = self.x + self.visual_offset_x;
        let vy = self.y + self.visual_offset_y;
        px >= vx && px < vx + self.width && py >= vy && py < vy + self.height
    }

    pub fn grip_contains_point(&self, px: f32, py: f32) -> bool {
        let vx = self.x + self.visual_offset_x;
        let gy = self.grip_y + self.visual_offset_y;
        px >= vx && px < vx + self.width && py >= gy && py < gy + self.grip_height
    }
}

pub fn compute_panel_layouts(
    dock: &DockManager,
    _screen_w: f32,
    screen_h: f32,
    header_h: f32,
    scale: f32,
) -> Vec<PanelLayoutBox> {
    let mut layouts = Vec::new();
    let s = crate::theme::clamp_ui_scale(scale);

    // 1. Top-left dock (Plugins, Preset, Domains)
    let top_start_y = header_h + 8.0 * s;
    let mut cur_x = 12.0 * s;

    for &tab in &dock.top_tabs {
        let w = tab.default_width() * s;
        let h = (tab.default_height() * s).min(screen_h - top_start_y - 24.0 * s);
        let grip_h = 16.0 * s;
        let grip_y = top_start_y + h - grip_h;

        let (is_dragged, vox, voy) = if let Some(ref d) = dock.drag {
            if d.tab == tab {
                (true, d.current_x - d.start_x, d.current_y - d.start_y)
            } else {
                (false, 0.0, 0.0)
            }
        } else {
            (false, 0.0, 0.0)
        };

        layouts.push(PanelLayoutBox {
            tab,
            x: cur_x,
            y: top_start_y,
            width: w,
            height: h,
            grip_y,
            grip_height: grip_h,
            is_dragged,
            visual_offset_x: vox,
            visual_offset_y: voy,
        });

        cur_x += w + 10.0 * s;
    }

    // 2. Bottom-left dock (Organize, Pomodoro, Storyboard)
    cur_x = 12.0 * s;
    for &tab in &dock.bottom_tabs {
        let w = tab.default_width() * s;
        let h = (tab.default_height() * s).min(screen_h - header_h - 24.0 * s);
        let y = screen_h - h - 12.0 * s;
        let grip_h = 16.0 * s;
        let grip_y = y;

        let (is_dragged, vox, voy) = if let Some(ref d) = dock.drag {
            if d.tab == tab {
                (true, d.current_x - d.start_x, d.current_y - d.start_y)
            } else {
                (false, 0.0, 0.0)
            }
        } else {
            (false, 0.0, 0.0)
        };

        layouts.push(PanelLayoutBox {
            tab,
            x: cur_x,
            y,
            width: w,
            height: h,
            grip_y,
            grip_height: grip_h,
            is_dragged,
            visual_offset_x: vox,
            visual_offset_y: voy,
        });

        cur_x += w + 10.0 * s;
    }

    layouts
}

// ── Tracé Vectoriel d'En-tête et Poignée `⠿⠿` ──────────────────────────

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

fn draw_grip_dots(pixmap: &mut PixmapMut, cx: f32, cy: f32, active: bool, theme: &Theme, scale: f32) {
    let color = if active {
        theme.dock_grip_active
    } else {
        theme.dock_grip_inactive
    };

    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    // Deux colonnes de trois points (⠿⠿ style Glucose)
    let s = crate::theme::clamp_ui_scale(scale);
    let mut pb = PathBuilder::new();
    let col_xs = [cx - 6.0 * s, cx - 2.0 * s, cx + 3.0 * s, cx + 7.0 * s];
    let row_ys = [cy - 3.0 * s, cy, cy + 3.0 * s];

    for &x in &col_xs {
        for &y in &row_ys {
            pb.push_circle(x, y, 0.9 * s);
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

// ── Structures de Layout Unifié pour Panneaux (R-37, R-34) ──────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidgetRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl WidgetRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

#[derive(Debug, Clone)]
pub struct OrganizeSortButton {
    pub sort_type: SortType,
    pub label: &'static str,
    pub rect: WidgetRect,
}

#[derive(Debug, Clone)]
pub struct OrganizeLayoutModeItem {
    pub mode: LayoutMode,
    pub title: &'static str,
    pub desc: &'static str,
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

pub fn layout_organize_panel(
    px: f32,
    py: f32,
    pw: f32,
    ph: f32,
    _state: &OrganizeState,
    typo: &Typography,
    scale: f32,
) -> OrganizePanelLayout {
    let s = crate::theme::clamp_ui_scale(scale);
    let pad_x = 14.0 * s;
    let target_count_rect = WidgetRect::new(px + pad_x, py + 38.0 * s, pw - 28.0 * s, 24.0 * s);
    let mut cy = py + (38.0 + 32.0 + 14.0) * s;

    let sort_buttons_def = [
        (SortType::None, "Ordre actuel"),
        (SortType::Color, "Couleur"),
        (SortType::SizeDesc, "Grand → Petit"),
        (SortType::SizeAsc, "Petit → Grand"),
        (SortType::RatioPort, "Portrait"),
        (SortType::RatioLand, "Paysage"),
        (SortType::LumAsc, "Sombre → Clair"),
        (SortType::LumDesc, "Clair → Sombre"),
    ];

    let mut sort_buttons = Vec::with_capacity(sort_buttons_def.len());
    let mut sx = px + pad_x;
    for (st, lbl) in sort_buttons_def {
        let (tw, _) = typo.measure_text(lbl, 10.0 * s, false);
        let bw = tw + 14.0 * s;
        if sx + bw > px + pw - pad_x {
            sx = px + pad_x;
            cy += 22.0 * s;
        }
        let rect = WidgetRect::new(sx, cy, bw, 18.0 * s);
        sort_buttons.push(OrganizeSortButton {
            sort_type: st,
            label: lbl,
            rect,
        });
        sx += bw + 4.0 * s;
    }
    cy += 28.0 * s + 14.0 * s;

    let layout_options_def = [
        (LayoutMode::Compact, "Rangées compactes", "Respecte les ratios, remplit chaque ligne"),
        (LayoutMode::Masonry, "Masonry (colonnes)", "Colonnes indépendantes — Pinterest"),
        (LayoutMode::Grid, "Grille alignée", "Même largeur par colonne, ratios conservés"),
        (LayoutMode::SameHeight, "Même hauteur", "Hauteur fixe, largeur proportionnelle"),
        (LayoutMode::BySlot, "Par slot preset", "Colonnes séparées par catégorie"),
    ];

    let mut layout_modes = Vec::with_capacity(layout_options_def.len());
    let lh = 30.0 * s;
    for (lm, title, desc) in layout_options_def {
        let rect = WidgetRect::new(px + pad_x, cy, pw - 28.0 * s, lh);
        layout_modes.push(OrganizeLayoutModeItem {
            mode: lm,
            title,
            desc,
            rect,
        });
        cy += lh + 2.0 * s;
    }
    cy += 6.0 * s + 12.0 * s;

    let col_w = (pw - 28.0 * s - 8.0 * s) / 2.0;
    let target_size_rect = WidgetRect::new(px + pad_x, cy, col_w, 20.0 * s);
    let gap_rect = WidgetRect::new(px + pad_x + col_w + 8.0 * s, cy, col_w, 20.0 * s);

    let apply_rect = WidgetRect::new(px + pad_x, py + ph - 38.0 * s, pw - 28.0 * s, 26.0 * s);

    OrganizePanelLayout {
        target_count_rect,
        sort_buttons,
        layout_modes,
        target_size_rect,
        gap_rect,
        apply_rect,
    }
}

#[derive(Debug, Clone)]
pub struct PomodoroPresetButton {
    pub label: &'static str,
    pub seconds: u32,
    pub rect: WidgetRect,
}

#[derive(Debug, Clone)]
pub struct PomodoroPanelLayout {
    pub ring_cx: f32,
    pub ring_cy: f32,
    pub ring_radius: f32,
    pub start_button: WidgetRect,
    pub reset_button: WidgetRect,
    pub presets: Vec<PomodoroPresetButton>,
}

pub fn layout_pomodoro_panel(
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    state: &PomodoroState,
    typo: &Typography,
    scale: f32,
) -> PomodoroPanelLayout {
    let s = crate::theme::clamp_ui_scale(scale);
    let cx = px + pw / 2.0;
    let cy = py + 72.0 * s;
    let r = 32.0 * s;

    let by = py + 124.0 * s;
    let btn_txt = if state.running { "Pause" } else { "Démarrer" };
    let (btw, _) = typo.measure_text(btn_txt, 11.0 * s, false);
    let bw = btw + 22.0 * s;

    let start_button = WidgetRect::new(px + 22.0 * s, by, bw, 22.0 * s);
    let reset_button = WidgetRect::new(px + 22.0 * s + bw + 6.0 * s, by, 22.0 * s, 22.0 * s);

    let py2 = by + 28.0 * s;
    let presets_def = [("25 min", 25 * 60), ("15 min", 15 * 60), ("5 min", 5 * 60)];
    let pr_w = (pw - 28.0 * s - 8.0 * s) / 3.0;
    let mut presets = Vec::with_capacity(3);
    for (i, (lbl, secs)) in presets_def.into_iter().enumerate() {
        let pr_x = px + 14.0 * s + i as f32 * (pr_w + 4.0 * s);
        presets.push(PomodoroPresetButton {
            label: lbl,
            seconds: secs,
            rect: WidgetRect::new(pr_x, py2, pr_w, 18.0 * s),
        });
    }

    PomodoroPanelLayout {
        ring_cx: cx,
        ring_cy: cy,
        ring_radius: r,
        start_button,
        reset_button,
        presets,
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

pub fn layout_storyboard_panel(
    px: f32,
    py: f32,
    pw: f32,
    ph: f32,
    scale: f32,
) -> StoryboardPanelLayout {
    let s = crate::theme::clamp_ui_scale(scale);
    let pad_x = 14.0 * s;
    let format_button = WidgetRect::new(px + pad_x, py + 52.0 * s, pw - 28.0 * s, 24.0 * s);

    let cy = py + 84.0 * s + 12.0 * s;
    let col_w = (pw - 28.0 * s - 12.0 * s) / 3.0;
    let width_input = WidgetRect::new(px + pad_x, cy, col_w, 20.0 * s);
    let cols_input = WidgetRect::new(px + pad_x + col_w + 6.0 * s, cy, col_w, 20.0 * s);
    let gap_input = WidgetRect::new(px + pad_x + (col_w + 6.0 * s) * 2.0, cy, col_w, 20.0 * s);

    let cell_y = cy + 30.0 * s;
    let grid_w = pw - 28.0 * s;
    let gcols = 4;
    let grows = 2;
    let cell_w = (grid_w - (gcols - 1) as f32 * 3.0 * s) / gcols as f32;
    let cell_h = 24.0 * s;
    let mut cells = Vec::with_capacity(gcols * grows);
    for r in 0..grows {
        for c in 0..gcols {
            let cx = px + pad_x + c as f32 * (cell_w + 3.0 * s);
            let cy_cell = cell_y + r as f32 * (cell_h + 3.0 * s);
            cells.push(WidgetRect::new(cx, cy_cell, cell_w, cell_h));
        }
    }

    let activate_button = WidgetRect::new(px + pad_x, py + ph - 38.0 * s, pw - 28.0 * s, 28.0 * s);

    StoryboardPanelLayout {
        format_button,
        width_input,
        cols_input,
        gap_input,
        cells,
        activate_button,
    }
}

#[derive(Debug, Clone)]
pub struct PluginsPanelLayout {
    pub download_button: WidgetRect,
    pub card_rect: WidgetRect,
    pub density_options: Vec<WidgetRect>,
    pub disposition_options: Vec<WidgetRect>,
}

pub fn layout_plugins_panel(
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    scale: f32,
) -> PluginsPanelLayout {
    let s = crate::theme::clamp_ui_scale(scale);
    let pad_x = 14.0 * s;
    let download_button = WidgetRect::new(px + pad_x, py + 104.0 * s, pw - 28.0 * s, 24.0 * s);
    let card_rect = WidgetRect::new(px + pad_x, py + 152.0 * s, pw - 28.0 * s, 54.0 * s);

    let dens_y = py + 244.0 * s;
    let mut density_options = Vec::with_capacity(3);
    for i in 0..3 {
        density_options.push(WidgetRect::new(px + pad_x, dens_y + i as f32 * 16.0 * s, pw - 28.0 * s, 16.0 * s));
    }

    let disp_y = dens_y + 3.0 * 16.0 * s + 20.0 * s;
    let mut disposition_options = Vec::with_capacity(2);
    for i in 0..2 {
        disposition_options.push(WidgetRect::new(px + pad_x, disp_y + i as f32 * 16.0 * s, pw - 28.0 * s, 16.0 * s));
    }

    PluginsPanelLayout {
        download_button,
        card_rect,
        density_options,
        disposition_options,
    }
}

// ── Rendu Global des Docks Déroulants ─────────────────────────────────────

pub fn render_docks(
    pixmap: &mut PixmapMut,
    dock: &DockManager,
    store: &Store,
    typo: &Typography,
    theme: &Theme,
    screen: ScreenFrame,
    pointer: Pointer,
) {
    let s = crate::theme::clamp_ui_scale(screen.scale);
    let layouts = compute_panel_layouts(dock, screen.width, screen.height, screen.header_h, s);
    let (mx, my) = (pointer.x, pointer.y);

    for b in &layouts {
        let px = b.x + b.visual_offset_x;
        let py = b.y + b.visual_offset_y;
        let pw = b.width;
        let ph = b.height;
        let frame = ScaledRect { x: px, y: py, w: pw, h: ph, scale: s };

        // 1. Ombre portée douce
        let shadow_color = if b.is_dragged {
            Color::from_rgba8(0, 0, 0, 220)
        } else {
            Color::from_rgba8(0, 0, 0, 160)
        };
        let mut sp = Paint::default();
        sp.set_color(shadow_color);
        sp.anti_alias = true;
        let mut spb = PathBuilder::new();
        push_rounded_rect(&mut spb, px - 2.0 * s, py + 3.0 * s, pw + 4.0 * s, ph + 4.0 * s, 8.0 * s);
        if let Some(path) = spb.finish() {
            pixmap.fill_path(&path, &sp, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }

        // 2. Fond du panneau
        let mut bg_paint = Paint::default();
        bg_paint.set_color(theme.bg_panel);
        bg_paint.anti_alias = true;
        let mut bpb = PathBuilder::new();
        push_rounded_rect(&mut bpb, px, py, pw, ph, 6.0 * s);
        if let Some(path) = bpb.finish() {
            pixmap.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);

            // Bordure
            let border_color = if b.is_dragged {
                theme.border_accent
            } else {
                theme.border_subtle
            };
            let mut border_paint = Paint::default();
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;
            let stroke = Stroke { width: 1.0 * s, ..Default::default() };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }

        // 3. Poignée `⠿⠿`
        let gy = b.grip_y + b.visual_offset_y;
        let is_grip_hover = b.grip_contains_point(mx, my);
        draw_grip_dots(pixmap, px + pw / 2.0, gy + b.grip_height / 2.0, b.is_dragged || is_grip_hover, theme, s);

        // 4. Rendu du contenu spécifique du panneau
        match b.tab {
            TabId::Organize => {
                render_organize_content(pixmap, &dock.organize, store, typo, theme, frame, pointer);
            }
            TabId::Pomodoro => {
                render_pomodoro_content(pixmap, &dock.pomodoro, typo, theme, frame, pointer);
            }
            TabId::Storyboard => {
                render_storyboard_content(pixmap, &dock.storyboard, typo, theme, frame, pointer);
            }
            TabId::Plugins => {
                render_plugins_content(pixmap, &dock.plugins, typo, theme, frame, pointer);
            }
            TabId::Preset => {
                render_preset_content(pixmap, &dock.presets, typo, theme, frame, pointer);
            }
            TabId::Domains => {
                domains::render_domains_panel(pixmap, store, &dock.domains, typo, theme, frame, pointer);
            }
        }
    }
}

// ── 1. Panneau ORDONNER ───────────────────────────────────────────────────

fn render_organize_content(
    pixmap: &mut PixmapMut,
    state: &OrganizeState,
    store: &Store,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let ScaledRect { x: px, y: py, w: pw, h: ph, .. } = frame;
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(frame.scale);
    typo.draw_text(
        pixmap,
        "ORDONNER",
        px + 14.0 * s,
        py + 18.0 * s,
        TextStyle { size: 12.0 * s, color: theme.text_primary, bold: true },
    );

    let layout = layout_organize_panel(px, py, pw, ph, state, typo, s);

    // Badge Cible
    let sel_count = store.selected_image_ids.len();
    let total_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let target_str = if sel_count > 0 {
        format!("{} image{} sélectionnée{}", sel_count, if sel_count > 1 { "s" } else { "" }, if sel_count > 1 { "s" } else { "" })
    } else {
        format!("Toutes les images ({})", total_count)
    };

    let mut tbox_paint = Paint::default();
    tbox_paint.set_color(theme.bg_card);
    tbox_paint.anti_alias = true;
    let mut tpb = PathBuilder::new();
    push_rounded_rect(&mut tpb, layout.target_count_rect.x, layout.target_count_rect.y, layout.target_count_rect.w, layout.target_count_rect.h, 3.0 * s);
    if let Some(p) = tpb.finish() {
        pixmap.fill_path(&p, &tbox_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(
        pixmap,
        &target_str,
        layout.target_count_rect.x + 8.0 * s,
        layout.target_count_rect.y + 6.0 * s,
        TextStyle { size: 11.0 * s, color: if sel_count > 0 { theme.text_accent } else { theme.text_secondary }, bold: false },
    );

    // TRIER AVANT DISPOSITION
    typo.draw_text(
        pixmap,
        "TRIER AVANT DISPOSITION",
        px + 14.0 * s,
        py + 70.0 * s,
        TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true },
    );

    for btn in &layout.sort_buttons {
        let is_active = state.sort_by == btn.sort_type;
        let is_hover = btn.rect.contains(mx, my);
        let mut bp = Paint::default();
        bp.set_color(if is_active { theme.bg_active } else if is_hover { theme.bg_hover } else { theme.btn_bg });
        bp.anti_alias = true;
        let mut bpb = PathBuilder::new();
        push_rounded_rect(&mut bpb, btn.rect.x, btn.rect.y, btn.rect.w, btn.rect.h, 3.0 * s);
        if let Some(p) = bpb.finish() {
            pixmap.fill_path(&p, &bp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            let stroke_color = if is_active { theme.border_accent } else { theme.btn_border };
            let mut sp = Paint::default();
            sp.set_color(stroke_color);
            let stroke = Stroke { width: 1.0 * s, ..Default::default() };
            pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
        }
        let (tw, _) = typo.measure_text(btn.label, 10.0 * s, false);
        typo.draw_text(
            pixmap,
            btn.label,
            btn.rect.x + (btn.rect.w - tw) / 2.0,
            btn.rect.y + 4.0 * s,
            TextStyle { size: 10.0 * s, color: if is_active { theme.text_primary } else { theme.text_secondary }, bold: is_active },
        );
    }

    // DISPOSITION
    if let Some(first_mode) = layout.layout_modes.first() {
        typo.draw_text(
            pixmap,
            "DISPOSITION",
            px + 14.0 * s,
            first_mode.rect.y - 14.0 * s,
            TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true },
        );
    }

    for item in &layout.layout_modes {
        let is_active = state.layout == item.mode;
        let is_hover = item.rect.contains(mx, my);
        let mut lp = Paint::default();
        lp.set_color(if is_active { theme.bg_active } else if is_hover { theme.bg_hover } else { Color::TRANSPARENT });
        lp.anti_alias = true;
        let mut lpb = PathBuilder::new();
        push_rounded_rect(&mut lpb, item.rect.x, item.rect.y, item.rect.w, item.rect.h, 4.0 * s);
        if let Some(p) = lpb.finish() {
            if is_active || is_hover {
                pixmap.fill_path(&p, &lp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            if is_active {
                let mut sp = Paint::default();
                sp.set_color(theme.border_accent);
                let stroke = Stroke { width: 1.0 * s, ..Default::default() };
                pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
            }
        }

        typo.draw_text(
            pixmap,
            item.title,
            item.rect.x + 8.0 * s,
            item.rect.y + 4.0 * s,
            TextStyle { size: 11.0 * s, color: if is_active { theme.text_primary } else { theme.text_secondary }, bold: is_active },
        );
        typo.draw_text(
            pixmap,
            item.desc,
            item.rect.x + 8.0 * s,
            item.rect.y + 17.0 * s,
            TextStyle { size: 9.0 * s, color: theme.text_muted, bold: false },
        );
    }

    // LARGEUR CIBLE & ESPACEMENT
    typo.draw_text(pixmap, "LARGEUR CIBLE", layout.target_size_rect.x, layout.target_size_rect.y - 12.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });
    typo.draw_text(pixmap, "ESPACEMENT", layout.gap_rect.x, layout.gap_rect.y - 12.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });

    // Inputs
    let mut in_p = Paint::default();
    in_p.set_color(theme.input_bg);
    in_p.anti_alias = true;
    let mut in_pb = PathBuilder::new();
    push_rounded_rect(&mut in_pb, layout.target_size_rect.x, layout.target_size_rect.y, layout.target_size_rect.w, layout.target_size_rect.h, 3.0 * s);
    push_rounded_rect(&mut in_pb, layout.gap_rect.x, layout.gap_rect.y, layout.gap_rect.w, layout.gap_rect.h, 3.0 * s);
    if let Some(p) = in_pb.finish() {
        pixmap.fill_path(&p, &in_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(pixmap, &format!("{}", state.size as i32), layout.target_size_rect.x + 8.0 * s, layout.target_size_rect.y + 4.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });
    typo.draw_text(pixmap, &format!("{}", state.gap as i32), layout.gap_rect.x + 8.0 * s, layout.gap_rect.y + 4.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });

    // Bouton APPLIQUER
    let is_apply_hover = layout.apply_rect.contains(mx, my);
    let mut btn_p = Paint::default();
    btn_p.set_color(if is_apply_hover { theme.bg_hover } else { theme.btn_bg });
    btn_p.anti_alias = true;
    let mut btn_pb = PathBuilder::new();
    push_rounded_rect(&mut btn_pb, layout.apply_rect.x, layout.apply_rect.y, layout.apply_rect.w, layout.apply_rect.h, 4.0 * s);
    if let Some(p) = btn_pb.finish() {
        pixmap.fill_path(&p, &btn_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (aw, _) = typo.measure_text("Appliquer", 12.0 * s, true);
    typo.draw_text(pixmap, "Appliquer", layout.apply_rect.x + (layout.apply_rect.w - aw) / 2.0, layout.apply_rect.y + 6.5 * s, TextStyle { size: 12.0 * s, color: theme.text_primary, bold: true });
}

// ── 2. Panneau POMODORO ───────────────────────────────────────────────────

fn render_pomodoro_content(
    pixmap: &mut PixmapMut,
    state: &PomodoroState,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let ScaledRect { x: px, y: py, w: pw, h: ph, .. } = frame;
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(frame.scale);
    typo.draw_text(
        pixmap,
        "POMODORO",
        px + 14.0 * s,
        py + 18.0 * s,
        TextStyle { size: 10.0 * s, color: theme.text_muted, bold: true },
    );

    let layout = layout_pomodoro_panel(px, py, pw, ph, state, typo, s);

    // Anneau fond
    let mut bg_paint = Paint::default();
    bg_paint.set_color(theme.border_subtle);
    bg_paint.anti_alias = true;
    let stroke_bg = Stroke { width: 4.5 * s, ..Default::default() };
    let mut bg_pb = PathBuilder::new();
    bg_pb.push_circle(layout.ring_cx, layout.ring_cy, layout.ring_radius);
    if let Some(p) = bg_pb.finish() {
        pixmap.stroke_path(&p, &bg_paint, &stroke_bg, Transform::identity(), None);
    }

    // Anneau de progression
    let progress = if state.total_seconds > 0 {
        1.0 - (state.left_seconds as f32 / state.total_seconds as f32)
    } else {
        1.0
    };

    // La chrome est monochrome : l'anneau est blanc. La référence le faisait bleu — la dette
    // que style.md avoue — et le port l'avait mis sur l'accent, qui n'est pas pour cela. Le
    // vert de fin est une couleur de contenu (fiche 10 § 1 : « validation Pomodoro »).
    let ring_color = if state.left_seconds == 0 { theme.success } else { theme.text_accent };

    let mut prog_paint = Paint::default();
    prog_paint.set_color(ring_color);
    prog_paint.anti_alias = true;
    let prog_stroke = Stroke { width: 4.5 * s, line_cap: tiny_skia::LineCap::Round, ..Default::default() };

    let steps = (progress * 60.0).max(1.0) as usize;
    let mut arc_pb = PathBuilder::new();
    for i in 0..=steps {
        let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / 60.0) * std::f32::consts::PI * 2.0;
        let ax = layout.ring_cx + layout.ring_radius * angle.cos();
        let ay = layout.ring_cy + layout.ring_radius * angle.sin();
        if i == 0 {
            arc_pb.move_to(ax, ay);
        } else {
            arc_pb.line_to(ax, ay);
        }
    }
    if let Some(p) = arc_pb.finish() {
        pixmap.stroke_path(&p, &prog_paint, &prog_stroke, Transform::identity(), None);
    }

    // Temps "25:00"
    let mm = state.left_seconds / 60;
    let ss = state.left_seconds % 60;
    let time_str = format!("{:02}:{:02}", mm, ss);
    let (tw, _) = typo.measure_text(&time_str, 16.0 * s, true);
    typo.draw_text(
        pixmap,
        &time_str,
        layout.ring_cx - tw / 2.0,
        layout.ring_cy - 8.0 * s,
        TextStyle { size: 16.0 * s, color: theme.text_primary, bold: true },
    );

    // Boutons Démarrer / Pause et Reset ↺
    let btn_txt = if state.running { "Pause" } else { "Démarrer" };
    let is_start_hover = layout.start_button.contains(mx, my);
    let mut p_start = Paint::default();
    p_start.set_color(if is_start_hover { theme.bg_hover } else { theme.btn_bg });
    p_start.anti_alias = true;
    let mut pb_start = PathBuilder::new();
    push_rounded_rect(&mut pb_start, layout.start_button.x, layout.start_button.y, layout.start_button.w, layout.start_button.h, 4.0 * s);
    if let Some(p) = pb_start.finish() {
        pixmap.fill_path(&p, &p_start, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (btw, _) = typo.measure_text(btn_txt, 11.0 * s, false);
    typo.draw_text(pixmap, btn_txt, layout.start_button.x + (layout.start_button.w - btw) / 2.0, layout.start_button.y + 5.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });

    // Bouton ↺
    let is_reset_hover = layout.reset_button.contains(mx, my);
    let mut p_reset = Paint::default();
    p_reset.set_color(if is_reset_hover { theme.bg_hover } else { theme.btn_bg });
    p_reset.anti_alias = true;
    let mut pb_reset = PathBuilder::new();
    push_rounded_rect(&mut pb_reset, layout.reset_button.x, layout.reset_button.y, layout.reset_button.w, layout.reset_button.h, 4.0 * s);
    if let Some(p) = pb_reset.finish() {
        pixmap.fill_path(&p, &p_reset, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (rw, _) = typo.measure_text("↺", 12.0 * s, false);
    typo.draw_text(pixmap, "↺", layout.reset_button.x + (layout.reset_button.w - rw) / 2.0, layout.reset_button.y + 4.0 * s, TextStyle { size: 12.0 * s, color: theme.text_muted, bold: false });

    // Boutons de présélection : 25 min, 15 min, 5 min
    for p_btn in &layout.presets {
        let is_sel = state.total_seconds == p_btn.seconds;
        let is_hover = p_btn.rect.contains(mx, my);
        let mut pp = Paint::default();
        pp.set_color(if is_sel { theme.bg_active } else if is_hover { theme.bg_hover } else { Color::TRANSPARENT });
        pp.anti_alias = true;
        let mut ppb = PathBuilder::new();
        push_rounded_rect(&mut ppb, p_btn.rect.x, p_btn.rect.y, p_btn.rect.w, p_btn.rect.h, 3.0 * s);
        if let Some(p) = ppb.finish() {
            if is_sel || is_hover {
                pixmap.fill_path(&p, &pp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            let mut sp = Paint::default();
            sp.set_color(if is_sel { theme.border_accent } else { theme.btn_border });
            let stroke = Stroke { width: 1.0 * s, ..Default::default() };
            pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
        }
        let (ptw, _) = typo.measure_text(p_btn.label, 9.5 * s, false);
        typo.draw_text(
            pixmap,
            p_btn.label,
            p_btn.rect.x + (p_btn.rect.w - ptw) / 2.0,
            p_btn.rect.y + 4.0 * s,
            TextStyle { size: 9.5 * s, color: if is_sel { theme.text_primary } else { theme.text_muted }, bold: is_sel },
        );
    }
}

// ── 3. Panneau STORYBOARD ─────────────────────────────────────────────────

fn render_storyboard_content(
    pixmap: &mut PixmapMut,
    state: &StoryboardState,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let ScaledRect { x: px, y: py, w: pw, h: ph, .. } = frame;
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(frame.scale);
    typo.draw_text(pixmap, "STORYBOARD", px + 14.0 * s, py + 18.0 * s, TextStyle { size: 12.0 * s, color: theme.text_primary, bold: true });

    let layout = layout_storyboard_panel(px, py, pw, ph, s);

    // FORMAT
    typo.draw_text(pixmap, "FORMAT", px + 14.0 * s, layout.format_button.y - 14.0 * s, TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true });

    let formats = ["16:9 — Cinéma HD", "4:3 — Classique", "2.35:1 — Scope", "1:1 — Carré", "9:16 — Vertical"];
    let fmt_str = formats[state.format_idx.min(formats.len() - 1)];

    let is_fmt_hover = layout.format_button.contains(mx, my);
    let mut f_paint = Paint::default();
    f_paint.set_color(if is_fmt_hover { theme.bg_hover } else { theme.input_bg });
    f_paint.anti_alias = true;
    let mut fpb = PathBuilder::new();
    push_rounded_rect(&mut fpb, layout.format_button.x, layout.format_button.y, layout.format_button.w, layout.format_button.h, 4.0 * s);
    if let Some(p) = fpb.finish() {
        pixmap.fill_path(&p, &f_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, fmt_str, layout.format_button.x + 8.0 * s, layout.format_button.y + 6.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });
    typo.draw_text(pixmap, "˅", layout.format_button.x + layout.format_button.w - 16.0 * s, layout.format_button.y + 4.0 * s, TextStyle { size: 12.0 * s, color: theme.text_muted, bold: false });

    // 3 colonnes d'inputs
    typo.draw_text(pixmap, "LARGEUR", layout.width_input.x, layout.width_input.y - 12.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });
    typo.draw_text(pixmap, "COLONNES", layout.cols_input.x, layout.cols_input.y - 12.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });
    typo.draw_text(pixmap, "ESPACEMENT", layout.gap_input.x, layout.gap_input.y - 12.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });

    for input_rect in [&layout.width_input, &layout.cols_input, &layout.gap_input] {
        let mut in_pb = PathBuilder::new();
        push_rounded_rect(&mut in_pb, input_rect.x, input_rect.y, input_rect.w, input_rect.h, 3.0 * s);
        if let Some(p) = in_pb.finish() {
            pixmap.fill_path(&p, &f_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
    typo.draw_text(pixmap, &format!("{}", state.panel_width as i32), layout.width_input.x + 8.0 * s, layout.width_input.y + 4.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });
    typo.draw_text(pixmap, &format!("{}", state.cols), layout.cols_input.x + 8.0 * s, layout.cols_input.y + 4.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });
    typo.draw_text(pixmap, &format!("{}", state.gap as i32), layout.gap_input.x + 8.0 * s, layout.gap_input.y + 4.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });

    // Grille des cellules
    let mut cp = Paint::default();
    cp.set_color(theme.bg_card);
    cp.anti_alias = true;
    let cs = Stroke { width: 0.8 * s, ..Default::default() };
    let mut csp = Paint::default();
    csp.set_color(theme.border_subtle);

    for (idx, cell) in layout.cells.iter().enumerate() {
        let num = idx + 1;
        let mut cpb = PathBuilder::new();
        push_rounded_rect(&mut cpb, cell.x, cell.y, cell.w, cell.h, 2.0 * s);
        if let Some(p) = cpb.finish() {
            pixmap.fill_path(&p, &cp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&p, &csp, &cs, Transform::identity(), None);
        }
        let num_str = format!("{}", num);
        let (nw, _) = typo.measure_text(&num_str, 9.0 * s, false);
        typo.draw_text(pixmap, &num_str, cell.x + (cell.w - nw) / 2.0, cell.y + 7.0 * s, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: false });
    }

    // Bouton ACTIVER
    let is_act_hover = layout.activate_button.contains(mx, my);
    let mut act_p = Paint::default();
    act_p.set_color(if state.active { theme.bg_active } else if is_act_hover { theme.bg_hover } else { theme.btn_bg });
    act_p.anti_alias = true;
    let mut act_pb = PathBuilder::new();
    push_rounded_rect(&mut act_pb, layout.activate_button.x, layout.activate_button.y, layout.activate_button.w, layout.activate_button.h, 4.0 * s);
    if let Some(p) = act_pb.finish() {
        pixmap.fill_path(&p, &act_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(if state.active { theme.border_accent } else { theme.btn_border });
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let btn_lbl = if state.active { "Désactiver" } else { "Activer" };
    let (bw, _) = typo.measure_text(btn_lbl, 12.0 * s, true);
    typo.draw_text(pixmap, btn_lbl, layout.activate_button.x + (layout.activate_button.w - bw) / 2.0, layout.activate_button.y + 7.5 * s, TextStyle { size: 12.0 * s, color: theme.text_primary, bold: true });
}

// ── 4. Panneau PLUGINS ────────────────────────────────────────────────────

/// L'état affiché d'Ollama tant qu'aucune détection n'existe. Le jour où `localhost:11434`
/// est interrogé, cette constante disparaît au profit du résultat.
pub const OLLAMA_STATUS: &str = "Ollama : non détecté";

fn render_plugins_content(
    pixmap: &mut PixmapMut,
    state: &PluginsState,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let ScaledRect { x: px, y: py, w: pw, h: ph, .. } = frame;
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(frame.scale);
    typo.draw_text(pixmap, "PLUGINS", px + 14.0 * s, py + 16.0 * s, TextStyle { size: 13.0 * s, color: theme.text_primary, bold: true });

    let layout = layout_plugins_panel(px, py, pw, ph, s);

    // IA LOCALE
    typo.draw_text(pixmap, "IA LOCALE", px + 14.0 * s, py + 38.0 * s, TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true });

    // Pastille d'état. Grise tant qu'aucune détection n'existe (fiche 09 § 10.3) : elle
    // était verte et disait « Ollama actif » sans avoir jamais interrogé `localhost:11434`,
    // au-dessus de caractéristiques machine écrites en dur.
    let mut dot_paint = Paint::default();
    dot_paint.set_color(theme.text_muted);
    dot_paint.anti_alias = true;
    let mut dpb = PathBuilder::new();
    dpb.push_circle(px + 18.0 * s, py + 57.0 * s, 3.5 * s);
    if let Some(p) = dpb.finish() {
        pixmap.fill_path(&p, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(pixmap, OLLAMA_STATUS, px + 26.0 * s, py + 52.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: true });

    typo.draw_text(pixmap, "Détection de la machine : à venir", px + 14.0 * s, py + 68.0 * s, TextStyle { size: 10.0 * s, color: theme.text_muted, bold: false });
    // Le nom du modèle suit la plume, pas un décalage fixe calibré sur une police (R-51).
    let after_label = typo.draw_text(pixmap, "Modèle conseillé pour ce PC : ", px + 14.0 * s, py + 82.0 * s, TextStyle { size: 10.5 * s, color: theme.text_secondary, bold: false });
    typo.draw_text(pixmap, "qwen2.5:7b", after_label, py + 82.0 * s, TextStyle { size: 10.5 * s, color: theme.text_primary, bold: true });

    // Bouton Télécharger
    let is_down_hover = layout.download_button.contains(mx, my);
    let mut down_p = Paint::default();
    down_p.set_color(if is_down_hover { theme.bg_hover } else { theme.btn_bg });
    down_p.anti_alias = true;
    let mut dpb2 = PathBuilder::new();
    push_rounded_rect(&mut dpb2, layout.download_button.x, layout.download_button.y, layout.download_button.w, layout.download_button.h, 4.0 * s);
    if let Some(p) = dpb2.finish() {
        pixmap.fill_path(&p, &down_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (dtw, _) = typo.measure_text("Télécharger qwen2.5:7b", 11.0 * s, false);
    typo.draw_text(pixmap, "Télécharger qwen2.5:7b", layout.download_button.x + (layout.download_button.w - dtw) / 2.0, layout.download_button.y + 6.0 * s, TextStyle { size: 11.0 * s, color: theme.text_primary, bold: false });

    // MOTEUR
    typo.draw_text(pixmap, "MOTEUR", px + 14.0 * s, layout.card_rect.y - 14.0 * s, TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true });

    let mut card_p = Paint::default();
    card_p.set_color(theme.bg_card);
    card_p.anti_alias = true;
    let mut cpb = PathBuilder::new();
    push_rounded_rect(&mut cpb, layout.card_rect.x, layout.card_rect.y, layout.card_rect.w, layout.card_rect.h, 6.0 * s);
    if let Some(p) = cpb.finish() {
        pixmap.fill_path(&p, &card_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(52, 211, 153, 200));
        let stroke = Stroke { width: 1.2 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, "Cours magistral (intégré)", layout.card_rect.x + 8.0 * s, layout.card_rect.y + 8.0 * s, TextStyle { size: 11.5 * s, color: theme.text_primary, bold: true });
    typo.draw_text(pixmap, "Transforme un texte en carte de concepts avec l'IA", layout.card_rect.x + 8.0 * s, layout.card_rect.y + 23.0 * s, TextStyle { size: 9.5 * s, color: theme.text_secondary, bold: false });
    typo.draw_text(pixmap, "locale. Aucun binaire à installer. v1.0", layout.card_rect.x + 8.0 * s, layout.card_rect.y + 35.0 * s, TextStyle { size: 9.5 * s, color: theme.text_muted, bold: false });

    // RÉGLAGES
    typo.draw_text(pixmap, "RÉGLAGES", px + 14.0 * s, layout.card_rect.y + layout.card_rect.h + 16.0 * s, TextStyle { size: 9.5 * s, color: theme.text_muted, bold: true });

    // Densité
    typo.draw_text(pixmap, "Densité", px + 14.0 * s, layout.density_options[0].y - 14.0 * s, TextStyle { size: 10.5 * s, color: theme.text_secondary, bold: true });
    let densities = ["Concis — les idées maîtresses", "Normal — équilibré", "Détaillé — chaque nuance"];
    for (i, d) in densities.into_iter().enumerate() {
        let is_checked = state.density_idx == i;
        let opt_rect = &layout.density_options[i];
        draw_radio_dot(pixmap, opt_rect.x + 6.0 * s, opt_rect.y + 5.0 * s, is_checked, theme, s);
        typo.draw_text(pixmap, d, opt_rect.x + 16.0 * s, opt_rect.y, TextStyle { size: 10.0 * s, color: if is_checked { theme.text_primary } else { theme.text_muted }, bold: is_checked });
    }

    // Disposition
    typo.draw_text(pixmap, "Disposition", px + 14.0 * s, layout.disposition_options[0].y - 14.0 * s, TextStyle { size: 10.5 * s, color: theme.text_secondary, bold: true });
    let disps = ["Grille — lecture en blocs", "Fil — une section par ligne"];
    for (i, d) in disps.into_iter().enumerate() {
        let is_checked = state.disposition_idx == i;
        let opt_rect = &layout.disposition_options[i];
        draw_radio_dot(pixmap, opt_rect.x + 6.0 * s, opt_rect.y + 5.0 * s, is_checked, theme, s);
        typo.draw_text(pixmap, d, opt_rect.x + 16.0 * s, opt_rect.y, TextStyle { size: 10.0 * s, color: if is_checked { theme.text_primary } else { theme.text_muted }, bold: is_checked });
    }
}

fn draw_radio_dot(pixmap: &mut PixmapMut, cx: f32, cy: f32, checked: bool, theme: &Theme, scale: f32) {
    let s = crate::theme::clamp_ui_scale(scale);
    let mut p = Paint { anti_alias: true, ..Default::default() };
    p.set_color(if checked { theme.accent_primary } else { theme.border_medium });
    let stroke = Stroke { width: 1.2 * s, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, 4.5 * s);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &p, &stroke, Transform::identity(), None);
    }
    if checked {
        let mut inner_p = Paint::default();
        inner_p.set_color(theme.accent_primary);
        inner_p.anti_alias = true;
        let mut ipb = PathBuilder::new();
        ipb.push_circle(cx, cy, 2.2 * s);
        if let Some(path) = ipb.finish() {
            pixmap.fill_path(&path, &inner_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
}

// ── 5. Panneau PRESETS ────────────────────────────────────────────────────

/// Description d'un preset de board : nom, domaines (libellé + couleur), résumé.
type PresetSpec<'a> = (&'a str, &'a [(&'a str, Color)], &'a str);

fn render_preset_content(
    pixmap: &mut PixmapMut,
    _state: &PresetsState,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let ScaledRect { x: px, y: py, w: pw, .. } = frame;
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(frame.scale);
    typo.draw_text(pixmap, "PRESETS", px + 14.0 * s, py + 16.0 * s, TextStyle { size: 13.0 * s, color: theme.text_primary, bold: true });

    let mut cy = py + 38.0 * s;

    typo.draw_text(pixmap, "CHOISIR UN PRESET POUR \"BOARD PRINCIPAL\"", px + 14.0 * s, cy, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: true });
    cy += 16.0 * s;

    let preset_items: [PresetSpec; 4] = [
        (
            "CharaDesign",
            &[
                ("Réf", Color::from_rgba8(59, 130, 246, 200)),
                ("Sketch", Color::from_rgba8(139, 92, 246, 200)),
                ("Lineart", Color::from_rgba8(168, 85, 247, 200)),
                ("Face", Color::from_rgba8(16, 185, 129, 200)),
                ("3/4", Color::from_rgba8(20, 184, 166, 200)),
                ("Profil", Color::from_rgba8(52, 211, 153, 200)),
                ("Dos", Color::from_rgba8(245, 158, 11, 200)),
                ("Poses", Color::from_rgba8(239, 68, 68, 200)),
                ("Final", Color::from_rgba8(107, 114, 128, 200)),
            ],
            "Workflow complet de création de personnage",
        ),
        (
            "Environment",
            &[
                ("Réf", Color::from_rgba8(59, 130, 246, 200)),
                ("Mood", Color::from_rgba8(99, 102, 241, 200)),
                ("Thumb", Color::from_rgba8(16, 185, 129, 200)),
                ("Layout", Color::from_rgba8(20, 184, 166, 200)),
                ("Détails", Color::from_rgba8(245, 158, 11, 200)),
                ("Lumière", Color::from_rgba8(251, 191, 36, 200)),
                ("Final", Color::from_rgba8(107, 114, 128, 200)),
            ],
            "Conception d'environnement et décors",
        ),
        (
            "Creature Design",
            &[
                ("Réf", Color::from_rgba8(59, 130, 246, 200)),
                ("Silh", Color::from_rgba8(139, 92, 246, 200)),
                ("Anatom", Color::from_rgba8(239, 68, 68, 200)),
                ("Textur", Color::from_rgba8(16, 185, 129, 200)),
                ("Turn", Color::from_rgba8(20, 184, 166, 200)),
                ("Action", Color::from_rgba8(245, 158, 11, 200)),
                ("Final", Color::from_rgba8(107, 114, 128, 200)),
            ],
            "Conception de créature / monstre",
        ),
        (
            "Props & Items",
            &[
                ("Réf", Color::from_rgba8(59, 130, 246, 200)),
                ("Sketch", Color::from_rgba8(139, 92, 246, 200)),
                ("Ortho", Color::from_rgba8(16, 185, 129, 200)),
                ("Détails", Color::from_rgba8(245, 158, 11, 200)),
                ("Final", Color::from_rgba8(107, 114, 128, 200)),
            ],
            "Design d'objets, armes, accessoires",
        ),
    ];

    for (title, chips, subtitle) in preset_items {
        typo.draw_text(pixmap, title, px + 14.0 * s, cy, TextStyle { size: 11.5 * s, color: theme.text_primary, bold: true });
        cy += 14.0 * s;

        let n = chips.len();
        let gap = 2.0 * s;
        let total_w = pw - 28.0 * s;
        let chip_w = (total_w - (n - 1) as f32 * gap) / n as f32;
        let chip_h = 32.0 * s;

        for (i, (chip_lbl, col)) in chips.iter().enumerate() {
            let cx = px + 14.0 * s + i as f32 * (chip_w + gap);
            let mut cp = Paint::default();
            let mut semi_col = *col;
            semi_col = Color::from_rgba8(
                (semi_col.red() * 255.0) as u8,
                (semi_col.green() * 255.0) as u8,
                (semi_col.blue() * 255.0) as u8,
                55,
            );
            cp.set_color(semi_col);
            cp.anti_alias = true;

            let mut cpb = PathBuilder::new();
            push_rounded_rect(&mut cpb, cx, cy, chip_w, chip_h, 2.0 * s);
            if let Some(p) = cpb.finish() {
                pixmap.fill_path(&p, &cp, tiny_skia::FillRule::Winding, Transform::identity(), None);
                let mut sp = Paint::default();
                sp.set_color(*col);
                let stroke = Stroke { width: 0.8 * s, ..Default::default() };
                pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
            }
            let (cw, _) = typo.measure_text(chip_lbl, 7.5 * s, false);
            typo.draw_text(pixmap, chip_lbl, cx + (chip_w - cw) / 2.0, cy + 12.0 * s, TextStyle { size: 7.5 * s, color: *col, bold: false });
        }
        cy += chip_h + 3.0 * s;

        typo.draw_text(pixmap, subtitle, px + 14.0 * s, cy, TextStyle { size: 9.0 * s, color: theme.text_muted, bold: false });
        cy += 16.0 * s;
    }

    // Bouton "+ Créer un preset custom"
    let custom_rect = WidgetRect::new(px + 14.0 * s, cy, pw - 28.0 * s, 24.0 * s);
    let is_custom_hover = custom_rect.contains(mx, my);
    let mut custom_p = Paint::default();
    custom_p.set_color(if is_custom_hover { theme.bg_hover } else { theme.btn_bg });
    custom_p.anti_alias = true;
    let mut cpb = PathBuilder::new();
    push_rounded_rect(&mut cpb, custom_rect.x, custom_rect.y, custom_rect.w, custom_rect.h, 4.0 * s);
    if let Some(p) = cpb.finish() {
        pixmap.fill_path(&p, &custom_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(theme.btn_border);
        let stroke = Stroke { width: 1.0 * s, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (ctw, _) = typo.measure_text("+ Créer un preset custom", 10.5 * s, false);
    typo.draw_text(pixmap, "+ Créer un preset custom", custom_rect.x + (custom_rect.w - ctw) / 2.0, custom_rect.y + 6.0 * s, TextStyle { size: 10.5 * s, color: theme.text_secondary, bold: false });
}

// ── Gestion des Interactions Souris sur les Panneaux Déroulants ────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PanelClickResult {
    Handled,
    ApplyLayout(OrganizeState),
    TogglePomodoro,
    ResetPomodoro,
    SetPomodoroTime(u32),
    ToggleStoryboard,
    SelectFormat(usize),
    SelectDensity(usize),
    SelectDisposition(usize),
    /// Le bouton « Télécharger » du panneau PLUGINS (fiche 09 § 10.3) — sans moteur derrière.
    DownloadModel,
    /// Un geste du panneau DOMAINES, à traduire en commande par `interactions::domains`.
    Domain(domains::DomainIntent),
}

pub fn handle_dock_click(
    dock: &mut DockManager,
    store: &Store,
    typo: &Typography,
    screen: ScreenFrame,
    pointer: Pointer,
) -> Option<PanelClickResult> {
    let (mx, my) = (pointer.x, pointer.y);
    let s = crate::theme::clamp_ui_scale(screen.scale);
    let layouts = compute_panel_layouts(dock, screen.width, screen.height, screen.header_h, s);

    for b in layouts {
        if !b.contains_point(mx, my) {
            continue;
        }

        let px = b.x;
        let py = b.y;
        let pw = b.width;

        match b.tab {
            TabId::Organize => {
                let layout = layout_organize_panel(px, py, pw, b.height, &dock.organize, typo, s);
                for btn in &layout.sort_buttons {
                    if btn.rect.contains(mx, my) {
                        dock.organize.sort_by = btn.sort_type;
                        return Some(PanelClickResult::Handled);
                    }
                }
                for item in &layout.layout_modes {
                    if item.rect.contains(mx, my) {
                        dock.organize.layout = item.mode;
                        return Some(PanelClickResult::Handled);
                    }
                }
                if layout.apply_rect.contains(mx, my) {
                    return Some(PanelClickResult::ApplyLayout(dock.organize.clone()));
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Pomodoro => {
                let layout = layout_pomodoro_panel(px, py, pw, b.height, &dock.pomodoro, typo, s);
                if layout.start_button.contains(mx, my) {
                    dock.pomodoro.running = !dock.pomodoro.running;
                    dock.pomodoro.last_tick = Instant::now();
                    return Some(PanelClickResult::TogglePomodoro);
                }
                if layout.reset_button.contains(mx, my) {
                    dock.pomodoro.left_seconds = dock.pomodoro.total_seconds;
                    dock.pomodoro.running = false;
                    return Some(PanelClickResult::ResetPomodoro);
                }
                for p in &layout.presets {
                    if p.rect.contains(mx, my) {
                        dock.pomodoro.total_seconds = p.seconds;
                        dock.pomodoro.left_seconds = p.seconds;
                        dock.pomodoro.running = false;
                        return Some(PanelClickResult::SetPomodoroTime(p.seconds));
                    }
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Storyboard => {
                let layout = layout_storyboard_panel(px, py, pw, b.height, s);
                if layout.format_button.contains(mx, my) {
                    dock.storyboard.format_idx = (dock.storyboard.format_idx + 1) % 5;
                    return Some(PanelClickResult::SelectFormat(dock.storyboard.format_idx));
                }
                if layout.activate_button.contains(mx, my) {
                    dock.storyboard.active = !dock.storyboard.active;
                    return Some(PanelClickResult::ToggleStoryboard);
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Plugins => {
                let layout = layout_plugins_panel(px, py, pw, b.height, s);
                if layout.download_button.contains(mx, my) {
                    return Some(PanelClickResult::DownloadModel);
                }
                for (i, opt) in layout.density_options.iter().enumerate() {
                    if opt.contains(mx, my) {
                        dock.plugins.density_idx = i;
                        return Some(PanelClickResult::SelectDensity(i));
                    }
                }
                for (i, opt) in layout.disposition_options.iter().enumerate() {
                    if opt.contains(mx, my) {
                        dock.plugins.disposition_idx = i;
                        return Some(PanelClickResult::SelectDisposition(i));
                    }
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Preset => {
                return Some(PanelClickResult::Handled);
            }
            TabId::Domains => {
                let frame = ScaledRect { x: px, y: py, w: pw, h: b.height, scale: s };
                let layout = domains::layout_domains_panel(frame, store, &dock.domains);
                let has_selection = !store.selected_annotation_ids.is_empty()
                    || !store.selected_image_ids.is_empty();
                let intent = domains::hit_domains_panel(&layout, store, pointer, has_selection);
                return Some(match intent {
                    Some(intent) => PanelClickResult::Domain(intent),
                    None => PanelClickResult::Handled,
                });
            }
        }
    }

    None
}

// ── Algorithmes de Réorganisation pour ORDONNER ────────────────────────────

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn apply_organize_layout(images: &[BoardImage], state: &OrganizeState) -> Vec<LayoutResult> {
    let mode = match state.layout {
        LayoutMode::Grid => glucose_core::layout::OrganizeMode::Grid,
        LayoutMode::Masonry => glucose_core::layout::OrganizeMode::Masonry,
        LayoutMode::SameHeight => glucose_core::layout::OrganizeMode::SameHeight,
        LayoutMode::Compact | LayoutMode::BySlot => glucose_core::layout::OrganizeMode::CompactRows,
    };
    let sort = match state.sort_by {
        SortType::None => glucose_core::layout::OrganizeSort::None,
        SortType::SizeDesc => glucose_core::layout::OrganizeSort::SizeDesc,
        SortType::SizeAsc => glucose_core::layout::OrganizeSort::SizeAsc,
        SortType::RatioPort => glucose_core::layout::OrganizeSort::RatioPortrait,
        SortType::RatioLand => glucose_core::layout::OrganizeSort::RatioLandscape,
        _ => glucose_core::layout::OrganizeSort::None,
    };
    let results = glucose_core::layout::calculate_image_layout(images, mode, sort, state.size, state.gap, state.cols);
    results.into_iter().map(|r| LayoutResult {
        id: r.id,
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Fiche 10 § 4 — deux docks : Domaines, Presets, Plugins descendent du haut ; Ordonner,
    /// Storyboard, Pomodoro montent du bas. Largeurs 320 / 280 / 340 et 250 / 260 / ≥ 160.
    /// Fermeture au-delà de 80 px de glissement vers la sortie.
    fn test_dock_anchors_and_defaults() {
        assert_eq!(TabId::Organize.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Pomodoro.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Storyboard.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Plugins.anchor(), DockAnchor::TopLeft);
        assert_eq!(TabId::Preset.anchor(), DockAnchor::TopLeft);
        assert_eq!(TabId::Domains.anchor(), DockAnchor::TopLeft);

        assert_eq!(TabId::Domains.default_width(), 320.0);
        assert_eq!(TabId::Preset.default_width(), 280.0);
        assert_eq!(TabId::Plugins.default_width(), 340.0);
        assert_eq!(TabId::Organize.default_width(), 250.0);
        assert_eq!(TabId::Storyboard.default_width(), 260.0);
        assert!(TabId::Pomodoro.default_width() >= 160.0);
        assert_eq!(DISMISS_DRAG_PX, 80.0);
    }

    #[test]
    fn test_dock_manager_toggle_and_dismiss() {
        let mut dock = DockManager::new();
        assert!(dock.is_open(TabId::Organize));
        assert!(dock.is_open(TabId::Pomodoro));
        assert_eq!(dock.bottom_tabs.len(), 2);

        dock.toggle_tab(TabId::Organize);
        assert!(!dock.is_open(TabId::Organize));
        assert_eq!(dock.bottom_tabs.len(), 1);

        dock.toggle_tab(TabId::Plugins);
        assert!(dock.is_open(TabId::Plugins));
        assert_eq!(dock.top_tabs.len(), 1);

        dock.dismiss_tab(TabId::Plugins);
        assert!(!dock.is_open(TabId::Plugins));
    }

    #[test]
    fn test_dock_drag_swap_and_dismiss() {
        let mut dock = DockManager::new();
        assert_eq!(dock.bottom_tabs, vec![TabId::Organize, TabId::Pomodoro]);

        dock.drag = Some(DragSession {
            tab: TabId::Pomodoro,
            start_x: 300.0,
            start_y: 500.0,
            current_x: 300.0,
            current_y: 500.0,
        });

        let swapped = dock.update_drag(50.0, 500.0);
        assert!(swapped);
        assert_eq!(dock.bottom_tabs, vec![TabId::Pomodoro, TabId::Organize]);

        // 79 px vers le bas : le panneau reste ; 81 px : il se ferme (fiche 10 § 4, 80 px).
        dock.update_drag(50.0, 579.0);
        assert_eq!(dock.finish_drag(), None, "sous le seuil, le panneau reste");
        assert!(dock.is_open(TabId::Pomodoro));
        dock.drag = Some(DragSession { tab: TabId::Pomodoro, start_x: 300.0, start_y: 500.0, current_x: 300.0, current_y: 500.0 });
        dock.update_drag(300.0, 581.0);
        let dismissed = dock.finish_drag();
        assert_eq!(dismissed, Some(TabId::Pomodoro));
        assert!(!dock.is_open(TabId::Pomodoro));
    }

    #[test]
    fn test_organize_layout_generation() {
        let mut images = Vec::new();
        for i in 0..6 {
            let mut img = BoardImage::new(format!("img-{}", i), 0.0, 0.0, 300.0, 200.0);
            img.original_width = 300.0;
            img.original_height = 200.0;
            images.push(img);
        }

        let state = OrganizeState {
            layout: LayoutMode::Grid,
            cols: 3,
            size: 200.0,
            ..Default::default()
        };

        let res = apply_organize_layout(&images, &state);
        assert_eq!(res.len(), 6);
        assert_eq!(res[0].width, 200.0);
    }

    /// Budget de temps maximal accepté pour un rendu complet de docks (debug).
    const DOCK_RENDER_BUDGET_MS: u128 = 2_000;

    /// Non-régression du gel de démarrage : `redraw()` passait la position de la
    /// souris à l'emplacement de l'argument `scale`, ce qui portait l'échelle des
    /// panneaux à 170 et le coût d'une frame à plus de 10 s — la pompe de messages
    /// Windows était affamée et la fenêtre restait blanche et « Ne répond pas ».
    #[test]
    fn test_render_docks_absurd_scale_is_clamped_and_bounded() {
        let mut pixmap = tiny_skia::Pixmap::new(1440, 900).expect("pixmap 1440x900");
        pixmap.fill(Theme::dark().bg_canvas);
        let store = Store::new("Clamp");
        let typo = Typography::new();
        let theme = Theme::dark();
        let dock = DockManager::new();

        let started = Instant::now();
        render_docks(
            &mut pixmap.as_mut(),
            &dock,
            &store,
            &typo,
            &theme,
            ScreenFrame { width: 1440.0, height: 900.0, header_h: 78.0, scale: 170.0 },
            Pointer { x: 0.0, y: 0.0 },
        );
        let elapsed = started.elapsed().as_millis();
        assert!(
            elapsed < DOCK_RENDER_BUDGET_MS,
            "rendu des docks à échelle aberrante : {elapsed} ms (budget {DOCK_RENDER_BUDGET_MS} ms)"
        );

        for b in compute_panel_layouts(&dock, 1440.0, 900.0, 78.0, 170.0) {
            assert!(b.width <= 1440.0, "panneau {:?} plus large que l'écran", b.tab);
            assert!(b.height <= 900.0, "panneau {:?} plus haut que l'écran", b.tab);
        }
    }

    #[test]
    fn test_render_docks_png() {
        let mut pixmap = tiny_skia::Pixmap::new(1440, 900).unwrap();
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
        let store = Store::new("Test");
        let typo = Typography::new();
        let theme = Theme::dark();

        // 1. Bottom docks: Organize, Pomodoro, Storyboard
        let mut dock = DockManager::new();
        dock.bottom_tabs = vec![TabId::Organize, TabId::Pomodoro, TabId::Storyboard];
        render_docks(
            &mut pixmap.as_mut(),
            &dock,
            &store,
            &typo,
            &theme,
            ScreenFrame { width: 1440.0, height: 900.0, header_h: 42.0, scale: 1.0 },
            Pointer { x: 0.0, y: 0.0 },
        );
        if let Err(e) = std::fs::create_dir_all("target") {
            eprintln!("create_dir_all failed: {}", e);
        }
        if let Err(e) = pixmap.save_png("target/dock_bottom.png") {
            eprintln!("save_png failed: {}", e);
        }

        // 2. Top docks: Plugins, Preset, Domains
        let mut pixmap_top = tiny_skia::Pixmap::new(1440, 900).unwrap();
        pixmap_top.fill(Color::from_rgba8(13, 14, 18, 255));
        let mut dock_top = DockManager::new();
        dock_top.top_tabs = vec![TabId::Plugins, TabId::Preset, TabId::Domains];
        render_docks(
            &mut pixmap_top.as_mut(),
            &dock_top,
            &store,
            &typo,
            &theme,
            ScreenFrame { width: 1440.0, height: 900.0, header_h: 42.0, scale: 1.0 },
            Pointer { x: 0.0, y: 0.0 },
        );
        if let Err(e) = pixmap_top.save_png("target/dock_top.png") {
            eprintln!("save_png failed: {}", e);
        }
    }

    #[test]
    fn test_organize_layout_exact_utf8_hit_test() {
        let mut dock = DockManager::new();
        dock.bottom_tabs = vec![TabId::Organize];
        let typo = Typography::new();

        // 1. At scale 1.0
        let layouts = compute_panel_layouts(&dock, 1440.0, 900.0, 78.0, 1.0);
        let org_box = layouts.iter().find(|b| b.tab == TabId::Organize).unwrap();
        let panel_layout = layout_organize_panel(org_box.x, org_box.y, org_box.width, org_box.height, &dock.organize, &typo, 1.0);

        // Find "Sombre → Clair" button
        let lum_asc_btn = panel_layout.sort_buttons.iter().find(|b| b.sort_type == SortType::LumAsc).unwrap();
        let click_x = lum_asc_btn.rect.x + lum_asc_btn.rect.w / 2.0;
        let click_y = lum_asc_btn.rect.y + lum_asc_btn.rect.h / 2.0;

        let screen = ScreenFrame { width: 1440.0, height: 900.0, header_h: 78.0, scale: 1.0 };
        let store = Store::new("Clic");
        let res = handle_dock_click(&mut dock, &store, &typo, screen, Pointer { x: click_x, y: click_y });
        assert_eq!(res, Some(PanelClickResult::Handled));
        assert_eq!(dock.organize.sort_by, SortType::LumAsc);

        // 2. At scale 1.5 (DPI scaling test)
        let mut dock_hi = DockManager::new();
        dock_hi.bottom_tabs = vec![TabId::Organize];
        let layouts_hi = compute_panel_layouts(&dock_hi, 1440.0, 900.0, 78.0 * 1.5, 1.5);
        let org_box_hi = layouts_hi.iter().find(|b| b.tab == TabId::Organize).unwrap();
        assert_eq!(org_box_hi.width, TabId::Organize.default_width() * 1.5);

        let panel_layout_hi = layout_organize_panel(org_box_hi.x, org_box_hi.y, org_box_hi.width, org_box_hi.height, &dock_hi.organize, &typo, 1.5);
        let lum_desc_btn = panel_layout_hi.sort_buttons.iter().find(|b| b.sort_type == SortType::LumDesc).unwrap();
        let click_x_hi = lum_desc_btn.rect.x + lum_desc_btn.rect.w / 2.0;
        let click_y_hi = lum_desc_btn.rect.y + lum_desc_btn.rect.h / 2.0;

        let screen_hi = ScreenFrame { width: 1440.0, height: 900.0, header_h: 78.0 * 1.5, scale: 1.5 };
        let res_hi = handle_dock_click(&mut dock_hi, &store, &typo, screen_hi, Pointer { x: click_x_hi, y: click_y_hi });
        assert_eq!(res_hi, Some(PanelClickResult::Handled));
        assert_eq!(dock_hi.organize.sort_by, SortType::LumDesc);
    }
}
