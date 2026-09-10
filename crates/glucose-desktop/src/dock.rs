//! Module Dock & Panneaux Déroulants (PanelDock) en Rust Natif.
//! Reproduit fidèlement PanelDock.tsx, OrganizePanel.tsx, PomodoroTimer.tsx,
//! StoryboardControls.tsx, PresetPanel.tsx, DomainsPanel.tsx, PluginPanel.tsx.

use crate::typography::Typography;
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

    pub fn default_width(&self) -> f32 {
        match self {
            Self::Organize => 250.0,
            Self::Pomodoro => 170.0,
            Self::Storyboard => 280.0,
            Self::Plugins => 320.0,
            Self::Preset => 280.0,
            Self::Domains => 310.0,
        }
    }

    pub fn default_height(&self) -> f32 {
        match self {
            Self::Organize => 410.0,
            Self::Pomodoro => 210.0,
            Self::Storyboard => 340.0,
            Self::Plugins => 460.0,
            Self::Preset => 460.0,
            Self::Domains => 300.0,
        }
    }
}

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
pub struct DomainItem {
    pub id: String,
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct DomainsState {
    pub domains: Vec<DomainItem>,
}

impl Default for DomainsState {
    fn default() -> Self {
        Self {
            domains: Vec::new(),
        }
    }
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
    pub domains: DomainsState,
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
            domains: DomainsState::default(),
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

    pub fn finish_drag(&mut self) -> Option<TabId> {
        let drag = self.drag.take()?;
        let dy = drag.current_y - drag.start_y;
        let should_dismiss = match drag.tab.anchor() {
            DockAnchor::TopLeft => dy < -60.0,
            DockAnchor::BottomLeft => dy > 60.0,
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
        px >= vx && px < vx + self.width && py >= gy && py < gy + 16.0
    }
}

pub fn compute_panel_layouts(
    dock: &DockManager,
    _screen_w: f32,
    screen_h: f32,
    header_h: f32,
) -> Vec<PanelLayoutBox> {
    let mut layouts = Vec::new();

    // 1. Top-left dock (Plugins, Preset, Domains)
    let top_start_y = header_h + 8.0;
    let mut cur_x = 12.0;

    for &tab in &dock.top_tabs {
        let w = tab.default_width();
        let h = tab.default_height().min(screen_h - top_start_y - 24.0);
        let grip_y = top_start_y + h - 16.0; // Grip at bottom for top-left drawer

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
            is_dragged,
            visual_offset_x: vox,
            visual_offset_y: voy,
        });

        cur_x += w + 10.0;
    }

    // 2. Bottom-left dock (Organize, Pomodoro, Storyboard)
    cur_x = 12.0;
    for &tab in &dock.bottom_tabs {
        let w = tab.default_width();
        let h = tab.default_height().min(screen_h - header_h - 24.0);
        let y = screen_h - h - 12.0;
        let grip_y = y; // Grip at top for bottom-left panel

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
            is_dragged,
            visual_offset_x: vox,
            visual_offset_y: voy,
        });

        cur_x += w + 10.0;
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

fn draw_grip_dots(pixmap: &mut PixmapMut, cx: f32, cy: f32, active: bool) {
    let color = if active {
        Color::from_rgba8(160, 160, 160, 255)
    } else {
        Color::from_rgba8(75, 75, 80, 255)
    };

    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    // Deux colonnes de trois points (⠿⠿ style Glucose)
    let mut pb = PathBuilder::new();
    let col_xs = [cx - 6.0, cx - 2.0, cx + 3.0, cx + 7.0];
    let row_ys = [cy - 3.0, cy, cy + 3.0];

    for &x in &col_xs {
        for &y in &row_ys {
            pb.push_circle(x, y, 0.9);
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

// ── Rendu Global des Docks Déroulants ─────────────────────────────────────

pub fn render_docks(
    pixmap: &mut PixmapMut,
    dock: &DockManager,
    store: &Store,
    typo: &Typography,
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
    mx: f32,
    my: f32,
) {
    let layouts = compute_panel_layouts(dock, screen_w, screen_h, header_h);

    for b in &layouts {
        let px = b.x + b.visual_offset_x;
        let py = b.y + b.visual_offset_y;
        let pw = b.width;
        let ph = b.height;

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
        push_rounded_rect(&mut spb, px - 2.0, py + 3.0, pw + 4.0, ph + 4.0, 8.0);
        if let Some(path) = spb.finish() {
            pixmap.fill_path(&path, &sp, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }

        // 2. Fond du panneau (#111111)
        let mut bg_paint = Paint::default();
        bg_paint.set_color(Color::from_rgba8(17, 17, 17, 255));
        bg_paint.anti_alias = true;
        let mut bpb = PathBuilder::new();
        push_rounded_rect(&mut bpb, px, py, pw, ph, 6.0);
        if let Some(path) = bpb.finish() {
            pixmap.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);

            // Bordure 1px #222222 (ou #444444 si en cours de drag)
            let border_color = if b.is_dragged {
                Color::from_rgba8(68, 68, 68, 255)
            } else {
                Color::from_rgba8(34, 34, 34, 255)
            };
            let mut border_paint = Paint::default();
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;
            let stroke = Stroke { width: 1.0, ..Default::default() };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }

        // 3. Poignée `⠿⠿`
        let gy = b.grip_y + b.visual_offset_y;
        let is_grip_hover = mx >= px && mx < px + pw && my >= gy && my < gy + 16.0;
        draw_grip_dots(pixmap, px + pw / 2.0, gy + 8.0, b.is_dragged || is_grip_hover);

        // 4. Rendu du contenu spécifique du panneau
        match b.tab {
            TabId::Organize => {
                render_organize_content(pixmap, &dock.organize, store, typo, px, py, pw, ph, mx, my);
            }
            TabId::Pomodoro => {
                render_pomodoro_content(pixmap, &dock.pomodoro, typo, px, py, pw, ph, mx, my);
            }
            TabId::Storyboard => {
                render_storyboard_content(pixmap, &dock.storyboard, typo, px, py, pw, ph, mx, my);
            }
            TabId::Plugins => {
                render_plugins_content(pixmap, &dock.plugins, typo, px, py, pw, ph, mx, my);
            }
            TabId::Preset => {
                render_preset_content(pixmap, &dock.presets, typo, px, py, pw, ph, mx, my);
            }
            TabId::Domains => {
                render_domains_content(pixmap, &dock.domains, store, typo, px, py, pw, ph, mx, my);
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
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(
        pixmap,
        "ORDONNER",
        px + 14.0,
        py + 18.0,
        12.0,
        Color::from_rgba8(204, 204, 204, 255),
        true,
    );

    let mut cy = py + 38.0;

    // Badge Cible
    let sel_count = store.selected_image_ids.len();
    let total_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let target_str = if sel_count > 0 {
        format!("{} image{} sélectionnée{}", sel_count, if sel_count > 1 { "s" } else { "" }, if sel_count > 1 { "s" } else { "" })
    } else {
        format!("Toutes les images ({})", total_count)
    };

    let mut tbox_paint = Paint::default();
    tbox_paint.set_color(if sel_count > 0 {
        Color::from_rgba8(35, 30, 0, 255)
    } else {
        Color::from_rgba8(26, 26, 26, 255)
    });
    tbox_paint.anti_alias = true;
    let mut tpb = PathBuilder::new();
    push_rounded_rect(&mut tpb, px + 14.0, cy, pw - 28.0, 24.0, 3.0);
    if let Some(p) = tpb.finish() {
        pixmap.fill_path(&p, &tbox_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(
        pixmap,
        &target_str,
        px + 22.0,
        cy + 6.0,
        11.0,
        if sel_count > 0 { Color::from_rgba8(220, 190, 0, 255) } else { Color::from_rgba8(110, 110, 115, 255) },
        false,
    );
    cy += 32.0;

    // TRIER AVANT DISPOSITION
    typo.draw_text(
        pixmap,
        "TRIER AVANT DISPOSITION",
        px + 14.0,
        cy,
        9.5,
        Color::from_rgba8(85, 85, 85, 255),
        true,
    );
    cy += 14.0;

    let sort_buttons = [
        (SortType::None, "Ordre actuel"),
        (SortType::Color, "Couleur"),
        (SortType::SizeDesc, "Grand → Petit"),
        (SortType::SizeAsc, "Petit → Grand"),
        (SortType::RatioPort, "Portrait"),
        (SortType::RatioLand, "Paysage"),
        (SortType::LumAsc, "Sombre → Clair"),
        (SortType::LumDesc, "Clair → Sombre"),
    ];

    let mut sx = px + 14.0;
    for (st, lbl) in sort_buttons {
        let (tw, _) = typo.measure_text(lbl, 10.0, false);
        let bw = tw + 14.0;
        if sx + bw > px + pw - 14.0 {
            sx = px + 14.0;
            cy += 22.0;
        }

        let is_active = state.sort_by == st;
        let mut bp = Paint::default();
        bp.set_color(if is_active { Color::from_rgba8(45, 45, 45, 255) } else { Color::from_rgba8(26, 26, 26, 255) });
        bp.anti_alias = true;
        let mut bpb = PathBuilder::new();
        push_rounded_rect(&mut bpb, sx, cy, bw, 18.0, 3.0);
        if let Some(p) = bpb.finish() {
            pixmap.fill_path(&p, &bp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            if is_active {
                let mut sp = Paint::default();
                sp.set_color(Color::from_rgba8(68, 68, 68, 255));
                let stroke = Stroke { width: 1.0, ..Default::default() };
                pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
            }
        }
        typo.draw_text(pixmap, lbl, sx + 7.0, cy + 4.0, 10.0, if is_active { Color::from_rgba8(230, 230, 230, 255) } else { Color::from_rgba8(115, 115, 120, 255) }, is_active);

        sx += bw + 4.0;
    }
    cy += 28.0;

    // DISPOSITION
    typo.draw_text(
        pixmap,
        "DISPOSITION",
        px + 14.0,
        cy,
        9.5,
        Color::from_rgba8(85, 85, 85, 255),
        true,
    );
    cy += 14.0;

    let layout_options = [
        (LayoutMode::Compact, "Rangées compactes", "Respecte les ratios, remplit chaque ligne"),
        (LayoutMode::Masonry, "Masonry (colonnes)", "Colonnes indépendantes — Pinterest"),
        (LayoutMode::Grid, "Grille alignée", "Même largeur par colonne, ratios conservés"),
        (LayoutMode::SameHeight, "Même hauteur", "Hauteur fixe, largeur proportionnelle"),
        (LayoutMode::BySlot, "Par slot preset", "Colonnes séparées par catégorie"),
    ];

    for (lm, title, desc) in layout_options {
        let is_active = state.layout == lm;
        let lh = 30.0;
        let mut lp = Paint::default();
        lp.set_color(if is_active { Color::from_rgba8(30, 30, 30, 255) } else { Color::TRANSPARENT });
        lp.anti_alias = true;
        let mut lpb = PathBuilder::new();
        push_rounded_rect(&mut lpb, px + 14.0, cy, pw - 28.0, lh, 4.0);
        if let Some(p) = lpb.finish() {
            if is_active {
                pixmap.fill_path(&p, &lp, tiny_skia::FillRule::Winding, Transform::identity(), None);
                let mut sp = Paint::default();
                sp.set_color(Color::from_rgba8(50, 50, 50, 255));
                let stroke = Stroke { width: 1.0, ..Default::default() };
                pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
            }
        }

        typo.draw_text(pixmap, title, px + 22.0, cy + 4.0, 11.0, if is_active { Color::from_rgba8(255, 255, 255, 255) } else { Color::from_rgba8(150, 150, 155, 255) }, is_active);
        typo.draw_text(pixmap, desc, px + 22.0, cy + 17.0, 9.0, Color::from_rgba8(80, 80, 85, 255), false);
        cy += lh + 2.0;
    }
    cy += 6.0;

    // LARGEUR CIBLE & ESPACEMENT
    let col_w = (pw - 28.0 - 8.0) / 2.0;
    typo.draw_text(pixmap, "LARGEUR CIBLE", px + 14.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    typo.draw_text(pixmap, "ESPACEMENT", px + 14.0 + col_w + 8.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 12.0;

    // Inputs
    let mut in_p = Paint::default();
    in_p.set_color(Color::from_rgba8(26, 26, 26, 255));
    in_p.anti_alias = true;
    let mut in_pb = PathBuilder::new();
    push_rounded_rect(&mut in_pb, px + 14.0, cy, col_w, 20.0, 3.0);
    push_rounded_rect(&mut in_pb, px + 14.0 + col_w + 8.0, cy, col_w, 20.0, 3.0);
    if let Some(p) = in_pb.finish() {
        pixmap.fill_path(&p, &in_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(pixmap, &format!("{}", state.size as i32), px + 22.0, cy + 4.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    typo.draw_text(pixmap, &format!("{}", state.gap as i32), px + 14.0 + col_w + 16.0, cy + 4.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    cy += 28.0;

    // Bouton APPLIQUER
    let mut btn_p = Paint::default();
    btn_p.set_color(Color::from_rgba8(34, 34, 34, 255));
    btn_p.anti_alias = true;
    let mut btn_pb = PathBuilder::new();
    push_rounded_rect(&mut btn_pb, px + 14.0, cy, pw - 28.0, 26.0, 4.0);
    if let Some(p) = btn_pb.finish() {
        pixmap.fill_path(&p, &btn_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(55, 55, 55, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (aw, _) = typo.measure_text("Appliquer", 12.0, true);
    typo.draw_text(pixmap, "Appliquer", px + (pw - aw) / 2.0, cy + 6.5, 12.0, Color::from_rgba8(220, 220, 220, 255), true);
}

// ── 2. Panneau POMODORO ───────────────────────────────────────────────────

fn render_pomodoro_content(
    pixmap: &mut PixmapMut,
    state: &PomodoroState,
    typo: &Typography,
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(
        pixmap,
        "POMODORO",
        px + 14.0,
        py + 18.0,
        10.0,
        Color::from_rgba8(115, 115, 120, 255),
        true,
    );

    let cx = px + pw / 2.0;
    let cy = py + 72.0;
    let r = 32.0;

    // Anneau fond #1e1e1e
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgba8(30, 30, 30, 255));
    bg_paint.anti_alias = true;
    let stroke_bg = Stroke { width: 4.5, ..Default::default() };
    let mut bg_pb = PathBuilder::new();
    bg_pb.push_circle(cx, cy, r);
    if let Some(p) = bg_pb.finish() {
        pixmap.stroke_path(&p, &bg_paint, &stroke_bg, Transform::identity(), None);
    }

    // Anneau de progression
    let progress = if state.total_seconds > 0 {
        1.0 - (state.left_seconds as f32 / state.total_seconds as f32)
    } else {
        1.0
    };

    let ring_color = if state.left_seconds == 0 {
        Color::from_rgba8(74, 222, 128, 255)
    } else {
        Color::from_rgba8(96, 165, 250, 255)
    };

    let mut prog_paint = Paint::default();
    prog_paint.set_color(ring_color);
    prog_paint.anti_alias = true;
    let prog_stroke = Stroke { width: 4.5, line_cap: tiny_skia::LineCap::Round, ..Default::default() };

    let steps = (progress * 60.0).max(1.0) as usize;
    let mut arc_pb = PathBuilder::new();
    for i in 0..=steps {
        let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / 60.0) * std::f32::consts::PI * 2.0;
        let ax = cx + r * angle.cos();
        let ay = cy + r * angle.sin();
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
    let (tw, _) = typo.measure_text(&time_str, 16.0, true);
    typo.draw_text(
        pixmap,
        &time_str,
        cx - tw / 2.0,
        cy - 8.0,
        16.0,
        Color::from_rgba8(230, 230, 235, 255),
        true,
    );

    // Boutons Démarrer / Pause et Reset ↺
    let by = py + 124.0;
    let btn_txt = if state.running { "Pause" } else { "Démarrer" };
    let (btw, _) = typo.measure_text(btn_txt, 11.0, false);
    let bw = btw + 22.0;

    let mut p_start = Paint::default();
    p_start.set_color(Color::from_rgba8(30, 30, 30, 255));
    p_start.anti_alias = true;
    let mut pb_start = PathBuilder::new();
    push_rounded_rect(&mut pb_start, px + 22.0, by, bw, 22.0, 4.0);
    if let Some(p) = pb_start.finish() {
        pixmap.fill_path(&p, &p_start, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(55, 55, 55, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, btn_txt, px + 22.0 + 11.0, by + 5.0, 11.0, Color::from_rgba8(204, 204, 204, 255), false);

    // Bouton ↺
    let rx = px + 22.0 + bw + 6.0;
    let mut pb_reset = PathBuilder::new();
    push_rounded_rect(&mut pb_reset, rx, by, 22.0, 22.0, 4.0);
    if let Some(p) = pb_reset.finish() {
        pixmap.fill_path(&p, &p_start, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(45, 45, 45, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, "↺", rx + 6.0, by + 4.0, 12.0, Color::from_rgba8(115, 115, 120, 255), false);

    // Boutons de présélection : 25 min, 15 min, 5 min
    let py2 = by + 28.0;
    let presets = [("25 min", 25 * 60), ("15 min", 15 * 60), ("5 min", 5 * 60)];
    let pr_w = (pw - 28.0 - 8.0) / 3.0;
    for (i, (lbl, secs)) in presets.into_iter().enumerate() {
        let pr_x = px + 14.0 + i as f32 * (pr_w + 4.0);
        let is_sel = state.total_seconds == secs;
        let mut pp = Paint::default();
        pp.set_color(if is_sel { Color::from_rgba8(42, 42, 42, 255) } else { Color::TRANSPARENT });
        pp.anti_alias = true;
        let mut ppb = PathBuilder::new();
        push_rounded_rect(&mut ppb, pr_x, py2, pr_w, 18.0, 3.0);
        if let Some(p) = ppb.finish() {
            pixmap.fill_path(&p, &pp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            let mut sp = Paint::default();
            sp.set_color(if is_sel { Color::from_rgba8(68, 68, 68, 255) } else { Color::from_rgba8(35, 35, 35, 255) });
            let stroke = Stroke { width: 1.0, ..Default::default() };
            pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
        }
        let (ptw, _) = typo.measure_text(lbl, 9.5, false);
        typo.draw_text(pixmap, lbl, pr_x + (pr_w - ptw) / 2.0, py2 + 4.0, 9.5, if is_sel { Color::from_rgba8(200, 200, 200, 255) } else { Color::from_rgba8(100, 100, 105, 255) }, is_sel);
    }
}

// ── 3. Panneau STORYBOARD ─────────────────────────────────────────────────

fn render_storyboard_content(
    pixmap: &mut PixmapMut,
    state: &StoryboardState,
    typo: &Typography,
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(pixmap, "STORYBOARD", px + 14.0, py + 18.0, 12.0, Color::from_rgba8(204, 204, 204, 255), true);

    let mut cy = py + 38.0;

    // FORMAT
    typo.draw_text(pixmap, "FORMAT", px + 14.0, cy, 9.5, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 14.0;

    let formats = ["16:9 — Cinéma HD", "4:3 — Classique", "2.35:1 — Scope", "1:1 — Carré", "9:16 — Vertical"];
    let fmt_str = formats[state.format_idx.min(formats.len() - 1)];

    let mut f_paint = Paint::default();
    f_paint.set_color(Color::from_rgba8(26, 26, 26, 255));
    f_paint.anti_alias = true;
    let mut fpb = PathBuilder::new();
    push_rounded_rect(&mut fpb, px + 14.0, cy, pw - 28.0, 24.0, 4.0);
    if let Some(p) = fpb.finish() {
        pixmap.fill_path(&p, &f_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(45, 45, 45, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, fmt_str, px + 22.0, cy + 6.0, 11.0, Color::from_rgba8(220, 220, 220, 255), false);
    typo.draw_text(pixmap, "⌄", px + pw - 28.0, cy + 4.0, 12.0, Color::from_rgba8(140, 140, 140, 255), false);
    cy += 32.0;

    // 3 colonnes d'inputs
    let col_w = (pw - 28.0 - 12.0) / 3.0;
    typo.draw_text(pixmap, "LARGEUR", px + 14.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    typo.draw_text(pixmap, "COLONNES", px + 14.0 + col_w + 6.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    typo.draw_text(pixmap, "ESPACEMENT", px + 14.0 + (col_w + 6.0) * 2.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 12.0;

    for i in 0..3 {
        let ix = px + 14.0 + i as f32 * (col_w + 6.0);
        let mut in_pb = PathBuilder::new();
        push_rounded_rect(&mut in_pb, ix, cy, col_w, 20.0, 3.0);
        if let Some(p) = in_pb.finish() {
            pixmap.fill_path(&p, &f_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
    typo.draw_text(pixmap, &format!("{}", state.panel_width as i32), px + 22.0, cy + 4.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    typo.draw_text(pixmap, &format!("{}", state.cols), px + 14.0 + col_w + 14.0, cy + 4.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    typo.draw_text(pixmap, &format!("{}", state.gap as i32), px + 14.0 + (col_w + 6.0) * 2.0 + 14.0, cy + 4.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    cy += 30.0;

    // Grille des cellules 1 à 8
    let grid_w = pw - 28.0;
    let gcols = 4;
    let grows = 2;
    let cell_w = (grid_w - (gcols - 1) as f32 * 3.0) / gcols as f32;
    let cell_h = 24.0;

    let mut cp = Paint::default();
    cp.set_color(Color::from_rgba8(22, 22, 22, 255));
    cp.anti_alias = true;
    let cs = Stroke { width: 0.8, ..Default::default() };
    let mut csp = Paint::default();
    csp.set_color(Color::from_rgba8(45, 45, 45, 255));

    for r in 0..grows {
        for c in 0..gcols {
            let num = r * gcols + c + 1;
            let cx = px + 14.0 + c as f32 * (cell_w + 3.0);
            let cy_cell = cy + r as f32 * (cell_h + 3.0);
            let mut cpb = PathBuilder::new();
            push_rounded_rect(&mut cpb, cx, cy_cell, cell_w, cell_h, 2.0);
            if let Some(p) = cpb.finish() {
                pixmap.fill_path(&p, &cp, tiny_skia::FillRule::Winding, Transform::identity(), None);
                pixmap.stroke_path(&p, &csp, &cs, Transform::identity(), None);
            }
            let num_str = format!("{}", num);
            let (nw, _) = typo.measure_text(&num_str, 9.0, false);
            typo.draw_text(pixmap, &num_str, cx + (cell_w - nw) / 2.0, cy_cell + 7.0, 9.0, Color::from_rgba8(100, 100, 100, 255), false);
        }
    }
    cy += grows as f32 * (cell_h + 3.0) + 16.0;

    // Bouton ACTIVER
    let mut act_p = Paint::default();
    act_p.set_color(if state.active { Color::from_rgba8(40, 40, 40, 255) } else { Color::from_rgba8(28, 28, 28, 255) });
    act_p.anti_alias = true;
    let mut act_pb = PathBuilder::new();
    push_rounded_rect(&mut act_pb, px + 14.0, cy, pw - 28.0, 28.0, 4.0);
    if let Some(p) = act_pb.finish() {
        pixmap.fill_path(&p, &act_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(55, 55, 55, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let btn_lbl = if state.active { "Désactiver" } else { "Activer" };
    let (bw, _) = typo.measure_text(btn_lbl, 12.0, true);
    typo.draw_text(pixmap, btn_lbl, px + (pw - bw) / 2.0, cy + 7.5, 12.0, Color::from_rgba8(220, 220, 220, 255), true);
}

// ── 4. Panneau PLUGINS ────────────────────────────────────────────────────

fn render_plugins_content(
    pixmap: &mut PixmapMut,
    state: &PluginsState,
    typo: &Typography,
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(pixmap, "PLUGINS", px + 14.0, py + 16.0, 13.0, Color::from_rgba8(220, 220, 220, 255), true);

    let mut cy = py + 38.0;

    // IA LOCALE
    typo.draw_text(pixmap, "IA LOCALE", px + 14.0, cy, 9.5, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 14.0;

    // Pastille verte
    let mut dot_paint = Paint::default();
    dot_paint.set_color(Color::from_rgba8(52, 211, 153, 255));
    dot_paint.anti_alias = true;
    let mut dpb = PathBuilder::new();
    dpb.push_circle(px + 18.0, cy + 5.0, 3.5);
    if let Some(p) = dpb.finish() {
        pixmap.fill_path(&p, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    typo.draw_text(pixmap, "Ollama actif", px + 26.0, cy, 11.0, Color::from_rgba8(220, 220, 220, 255), true);
    cy += 16.0;

    typo.draw_text(pixmap, "Ce PC : 32 Go RAM · 12 cœurs · GPU 6 Go", px + 14.0, cy, 10.0, Color::from_rgba8(115, 115, 120, 255), false);
    cy += 14.0;

    typo.draw_text(pixmap, "Modèle conseillé pour ce PC : ", px + 14.0, cy, 10.5, Color::from_rgba8(130, 130, 135, 255), false);
    typo.draw_text(pixmap, "qwen2.5:7b", px + 142.0, cy, 10.5, Color::from_rgba8(255, 255, 255, 255), true);
    cy += 16.0;

    // Bouton Télécharger
    let mut down_p = Paint::default();
    down_p.set_color(Color::from_rgba8(26, 26, 26, 255));
    down_p.anti_alias = true;
    let mut dpb2 = PathBuilder::new();
    push_rounded_rect(&mut dpb2, px + 14.0, cy, pw - 28.0, 24.0, 4.0);
    if let Some(p) = dpb2.finish() {
        pixmap.fill_path(&p, &down_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(45, 45, 45, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (dtw, _) = typo.measure_text("Télécharger qwen2.5:7b", 11.0, false);
    typo.draw_text(pixmap, "Télécharger qwen2.5:7b", px + (pw - dtw) / 2.0, cy + 6.0, 11.0, Color::from_rgba8(200, 200, 200, 255), false);
    cy += 34.0;

    // MOTEUR
    typo.draw_text(pixmap, "MOTEUR", px + 14.0, cy, 9.5, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 14.0;

    let mut card_p = Paint::default();
    card_p.set_color(Color::from_rgba8(20, 22, 25, 255));
    card_p.anti_alias = true;
    let mut cpb = PathBuilder::new();
    push_rounded_rect(&mut cpb, px + 14.0, cy, pw - 28.0, 54.0, 6.0);
    if let Some(p) = cpb.finish() {
        pixmap.fill_path(&p, &card_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(52, 211, 153, 200));
        let stroke = Stroke { width: 1.2, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    typo.draw_text(pixmap, "Cours magistral (intégré)", px + 22.0, cy + 8.0, 11.5, Color::from_rgba8(255, 255, 255, 255), true);
    typo.draw_text(pixmap, "Transforme un texte en carte de concepts avec l'IA", px + 22.0, cy + 23.0, 9.5, Color::from_rgba8(130, 130, 135, 255), false);
    typo.draw_text(pixmap, "locale. Aucun binaire à installer. v1.0", px + 22.0, cy + 35.0, 9.5, Color::from_rgba8(130, 130, 135, 255), false);
    cy += 64.0;

    // RÉGLAGES
    typo.draw_text(pixmap, "RÉGLAGES", px + 14.0, cy, 9.5, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 14.0;

    // Densité
    typo.draw_text(pixmap, "Densité", px + 14.0, cy, 10.5, Color::from_rgba8(180, 180, 180, 255), true);
    cy += 14.0;
    let densities = ["Concis — les idées maîtresses", "Normal — équilibré", "Détaillé — chaque nuance"];
    for (i, d) in densities.into_iter().enumerate() {
        let is_checked = state.density_idx == i;
        draw_radio_dot(pixmap, px + 20.0, cy + 5.0, is_checked);
        typo.draw_text(pixmap, d, px + 30.0, cy, 10.0, if is_checked { Color::from_rgba8(220, 220, 220, 255) } else { Color::from_rgba8(120, 120, 125, 255) }, is_checked);
        cy += 16.0;
    }
    cy += 6.0;

    // Disposition
    typo.draw_text(pixmap, "Disposition", px + 14.0, cy, 10.5, Color::from_rgba8(180, 180, 180, 255), true);
    cy += 14.0;
    let disps = ["Grille — lecture en blocs", "Fil — une section par ligne"];
    for (i, d) in disps.into_iter().enumerate() {
        let is_checked = state.disposition_idx == i;
        draw_radio_dot(pixmap, px + 20.0, cy + 5.0, is_checked);
        typo.draw_text(pixmap, d, px + 30.0, cy, 10.0, if is_checked { Color::from_rgba8(220, 220, 220, 255) } else { Color::from_rgba8(120, 120, 125, 255) }, is_checked);
        cy += 16.0;
    }
    cy += 8.0;

    // TEXTE SOURCE
    typo.draw_text(pixmap, "TEXTE SOURCE", px + 14.0, cy, 9.5, Color::from_rgba8(85, 85, 85, 255), true);
}

fn draw_radio_dot(pixmap: &mut PixmapMut, cx: f32, cy: f32, checked: bool) {
    let mut p = Paint::default();
    p.anti_alias = true;
    p.set_color(if checked { Color::from_rgba8(52, 211, 153, 255) } else { Color::from_rgba8(80, 80, 85, 255) });
    let stroke = Stroke { width: 1.2, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, 4.5);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &p, &stroke, Transform::identity(), None);
    }
    if checked {
        let mut inner_p = Paint::default();
        inner_p.set_color(Color::from_rgba8(52, 211, 153, 255));
        inner_p.anti_alias = true;
        let mut ipb = PathBuilder::new();
        ipb.push_circle(cx, cy, 2.2);
        if let Some(path) = ipb.finish() {
            pixmap.fill_path(&path, &inner_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
}

// ── 5. Panneau PRESETS ────────────────────────────────────────────────────

fn render_preset_content(
    pixmap: &mut PixmapMut,
    _state: &PresetsState,
    typo: &Typography,
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(pixmap, "PRESETS", px + 14.0, py + 16.0, 13.0, Color::from_rgba8(220, 220, 220, 255), true);

    let mut cy = py + 38.0;

    typo.draw_text(pixmap, "CHOISIR UN PRESET POUR \"BOARD PRINCIPAL\"", px + 14.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), true);
    cy += 16.0;

    let preset_items: [(&str, &[(&str, Color)], &str); 4] = [
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
        typo.draw_text(pixmap, title, px + 14.0, cy, 11.5, Color::from_rgba8(220, 220, 220, 255), true);
        cy += 14.0;

        let n = chips.len();
        let gap = 2.0;
        let total_w = pw - 28.0;
        let chip_w = (total_w - (n - 1) as f32 * gap) / n as f32;
        let chip_h = 32.0;

        for (i, (chip_lbl, col)) in chips.iter().enumerate() {
            let cx = px + 14.0 + i as f32 * (chip_w + gap);
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
            push_rounded_rect(&mut cpb, cx, cy, chip_w, chip_h, 2.0);
            if let Some(p) = cpb.finish() {
                pixmap.fill_path(&p, &cp, tiny_skia::FillRule::Winding, Transform::identity(), None);
                let mut sp = Paint::default();
                sp.set_color(*col);
                let stroke = Stroke { width: 0.8, ..Default::default() };
                pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
            }
            let (cw, _) = typo.measure_text(chip_lbl, 7.5, false);
            typo.draw_text(pixmap, chip_lbl, cx + (chip_w - cw) / 2.0, cy + 12.0, 7.5, *col, false);
        }
        cy += chip_h + 3.0;

        typo.draw_text(pixmap, subtitle, px + 14.0, cy, 9.0, Color::from_rgba8(85, 85, 85, 255), false);
        cy += 16.0;
    }

    // Bouton "+ Créer un preset custom"
    let mut custom_p = Paint::default();
    custom_p.set_color(Color::from_rgba8(26, 26, 26, 255));
    custom_p.anti_alias = true;
    let mut cpb = PathBuilder::new();
    push_rounded_rect(&mut cpb, px + 14.0, cy, pw - 28.0, 24.0, 4.0);
    if let Some(p) = cpb.finish() {
        pixmap.fill_path(&p, &custom_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(45, 45, 45, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (ctw, _) = typo.measure_text("+ Créer un preset custom", 10.5, false);
    typo.draw_text(pixmap, "+ Créer un preset custom", px + (pw - ctw) / 2.0, cy + 6.0, 10.5, Color::from_rgba8(160, 160, 160, 255), false);
}

// ── 6. Panneau DOMAINES ───────────────────────────────────────────────────

fn render_domains_content(
    pixmap: &mut PixmapMut,
    state: &DomainsState,
    _store: &Store,
    typo: &Typography,
    px: f32,
    py: f32,
    pw: f32,
    _ph: f32,
    _mx: f32,
    _my: f32,
) {
    typo.draw_text(pixmap, "DOMAINES", px + 14.0, py + 16.0, 13.0, Color::from_rgba8(220, 220, 220, 255), true);

    let mut cy = py + 48.0;

    if state.domains.is_empty() {
        let (tw1, _) = typo.measure_text("Aucun domaine.", 12.0, false);
        typo.draw_text(pixmap, "Aucun domaine.", px + (pw - tw1) / 2.0, cy, 12.0, Color::from_rgba8(115, 115, 120, 255), false);
        cy += 18.0;

        let msg2 = "Crée-en un pour colorer les membranes selon leur";
        let (tw2, _) = typo.measure_text(msg2, 11.0, false);
        typo.draw_text(pixmap, msg2, px + (pw - tw2) / 2.0, cy, 11.0, Color::from_rgba8(95, 95, 100, 255), false);
        cy += 16.0;

        let msg3 = "sémantique.";
        let (tw3, _) = typo.measure_text(msg3, 11.0, false);
        typo.draw_text(pixmap, msg3, px + (pw - tw3) / 2.0, cy, 11.0, Color::from_rgba8(95, 95, 100, 255), false);
        cy += 36.0;
    } else {
        for dom in &state.domains {
            let mut dp = Paint::default();
            dp.set_color(dom.color);
            dp.anti_alias = true;
            let mut dpb = PathBuilder::new();
            dpb.push_circle(px + 22.0, cy + 6.0, 4.0);
            if let Some(p) = dpb.finish() {
                pixmap.fill_path(&p, &dp, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            typo.draw_text(pixmap, &dom.name, px + 34.0, cy, 11.5, Color::from_rgba8(220, 220, 220, 255), false);
            cy += 24.0;
        }
    }

    // Bouton "+ Nouveau domaine"
    let mut btn_p = Paint::default();
    btn_p.set_color(Color::from_rgba8(26, 26, 26, 255));
    btn_p.anti_alias = true;
    let mut bpb = PathBuilder::new();
    push_rounded_rect(&mut bpb, px + 14.0, cy, pw - 28.0, 26.0, 4.0);
    if let Some(p) = bpb.finish() {
        pixmap.fill_path(&p, &btn_p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(45, 45, 45, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&p, &sp, &stroke, Transform::identity(), None);
    }
    let (bw, _) = typo.measure_text("+ Nouveau domaine", 11.5, false);
    typo.draw_text(pixmap, "+ Nouveau domaine", px + (pw - bw) / 2.0, cy + 6.5, 11.5, Color::from_rgba8(180, 180, 185, 255), false);
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
    AddDomain,
}

pub fn handle_dock_click(
    dock: &mut DockManager,
    _store: &Store,
    mx: f32,
    my: f32,
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
) -> Option<PanelClickResult> {
    let layouts = compute_panel_layouts(dock, screen_w, screen_h, header_h);

    for b in layouts {
        if !b.contains_point(mx, my) {
            continue;
        }

        let px = b.x;
        let py = b.y;
        let pw = b.width;

        match b.tab {
            TabId::Organize => {
                let mut cy = py + 38.0 + 32.0 + 14.0;
                let sort_buttons = [
                    (SortType::None, "Ordre actuel"),
                    (SortType::Color, "Couleur"),
                    (SortType::SizeDesc, "Grand → Petit"),
                    (SortType::SizeAsc, "Petit → Grand"),
                    (SortType::RatioPort, "Portrait"),
                    (SortType::RatioLand, "Paysage"),
                    (SortType::LumAsc, "Sombre → Clair"),
                    (SortType::LumDesc, "Clair → Sombre"),
                ];
                let mut sx = px + 14.0;
                for (st, lbl) in sort_buttons {
                    let bw = lbl.len() as f32 * 6.0 + 14.0;
                    if sx + bw > px + pw - 14.0 {
                        sx = px + 14.0;
                        cy += 22.0;
                    }
                    if mx >= sx && mx < sx + bw && my >= cy && my < cy + 18.0 {
                        dock.organize.sort_by = st;
                        return Some(PanelClickResult::Handled);
                    }
                    sx += bw + 4.0;
                }

                cy += 28.0 + 14.0;
                let layout_modes = [
                    LayoutMode::Compact,
                    LayoutMode::Masonry,
                    LayoutMode::Grid,
                    LayoutMode::SameHeight,
                    LayoutMode::BySlot,
                ];
                for lm in layout_modes {
                    if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= cy && my < cy + 30.0 {
                        dock.organize.layout = lm;
                        return Some(PanelClickResult::Handled);
                    }
                    cy += 32.0;
                }

                let apply_y = py + b.height - 38.0;
                if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= apply_y && my < apply_y + 28.0 {
                    return Some(PanelClickResult::ApplyLayout(dock.organize.clone()));
                }

                return Some(PanelClickResult::Handled);
            }
            TabId::Pomodoro => {
                let by = py + 124.0;
                if mx >= px + 22.0 && mx < px + 90.0 && my >= by && my < by + 22.0 {
                    dock.pomodoro.running = !dock.pomodoro.running;
                    dock.pomodoro.last_tick = Instant::now();
                    return Some(PanelClickResult::TogglePomodoro);
                }
                if mx >= px + 96.0 && mx < px + 120.0 && my >= by && my < by + 22.0 {
                    dock.pomodoro.left_seconds = dock.pomodoro.total_seconds;
                    dock.pomodoro.running = false;
                    return Some(PanelClickResult::ResetPomodoro);
                }
                let py2 = by + 28.0;
                let presets = [25 * 60, 15 * 60, 5 * 60];
                let pr_w = (pw - 28.0 - 8.0) / 3.0;
                for (i, secs) in presets.into_iter().enumerate() {
                    let pr_x = px + 14.0 + i as f32 * (pr_w + 4.0);
                    if mx >= pr_x && mx < pr_x + pr_w && my >= py2 && my < py2 + 18.0 {
                        dock.pomodoro.total_seconds = secs;
                        dock.pomodoro.left_seconds = secs;
                        dock.pomodoro.running = false;
                        return Some(PanelClickResult::SetPomodoroTime(secs));
                    }
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Storyboard => {
                if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= py + 52.0 && my < py + 76.0 {
                    dock.storyboard.format_idx = (dock.storyboard.format_idx + 1) % 5;
                    return Some(PanelClickResult::SelectFormat(dock.storyboard.format_idx));
                }
                let act_y = py + b.height - 38.0;
                if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= act_y && my < act_y + 28.0 {
                    dock.storyboard.active = !dock.storyboard.active;
                    return Some(PanelClickResult::ToggleStoryboard);
                }
                return Some(PanelClickResult::Handled);
            }
            TabId::Plugins => {
                let dens_y = py + 38.0 + 14.0 + 16.0 + 14.0 + 16.0 + 24.0 + 34.0 + 14.0 + 54.0 + 14.0 + 14.0;
                for i in 0..3 {
                    let ry = dens_y + i as f32 * 16.0;
                    if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= ry && my < ry + 16.0 {
                        dock.plugins.density_idx = i;
                        return Some(PanelClickResult::SelectDensity(i));
                    }
                }
                let disp_y = dens_y + 3.0 * 16.0 + 20.0;
                for i in 0..2 {
                    let ry = disp_y + i as f32 * 16.0;
                    if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= ry && my < ry + 16.0 {
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
                let btn_y = py + b.height - 40.0;
                if mx >= px + 14.0 && mx < px + pw - 14.0 && my >= btn_y && my < btn_y + 28.0 {
                    let colors = [
                        Color::from_rgba8(96, 165, 250, 255),
                        Color::from_rgba8(52, 211, 153, 255),
                        Color::from_rgba8(244, 114, 182, 255),
                        Color::from_rgba8(251, 191, 36, 255),
                        Color::from_rgba8(167, 139, 250, 255),
                    ];
                    let idx = dock.domains.domains.len() % colors.len();
                    let new_id = format!("domain-{}", dock.domains.domains.len() + 1);
                    let new_name = format!("Domaine {}", dock.domains.domains.len() + 1);
                    dock.domains.domains.push(DomainItem {
                        id: new_id,
                        name: new_name,
                        color: colors[idx],
                    });
                    return Some(PanelClickResult::AddDomain);
                }
                return Some(PanelClickResult::Handled);
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
    if images.is_empty() {
        return Vec::new();
    }

    let mut sorted = images.to_vec();

    match state.sort_by {
        SortType::None => {}
        SortType::SizeDesc => {
            sorted.sort_by(|a, b| (b.width * b.height).partial_cmp(&(a.width * a.height)).unwrap());
        }
        SortType::SizeAsc => {
            sorted.sort_by(|a, b| (a.width * a.height).partial_cmp(&(b.width * b.height)).unwrap());
        }
        SortType::RatioPort => {
            sorted.sort_by(|a, b| (a.width / a.height.max(1.0)).partial_cmp(&(b.width / b.height.max(1.0))).unwrap());
        }
        SortType::RatioLand => {
            sorted.sort_by(|a, b| (b.width / b.height.max(1.0)).partial_cmp(&(a.width / a.height.max(1.0))).unwrap());
        }
        _ => {}
    }

    let avg_x = images.iter().map(|img| img.x).sum::<f64>() / images.len() as f64;
    let avg_y = images.iter().map(|img| img.y).sum::<f64>() / images.len() as f64;
    let start_x = avg_x - (images.len() as f64 * state.size * 0.3);
    let start_y = avg_y - (state.size * 0.5);

    match state.layout {
        LayoutMode::Grid => {
            let cols = if state.cols > 0 { state.cols } else { (images.len() as f64).sqrt().round().max(1.0) as usize };
            let w = state.size;
            let mut results = Vec::new();
            let mut cur_y = start_y;

            for chunk in sorted.chunks(cols) {
                let max_h = chunk.iter().map(|img| {
                    let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                    w / ratio
                }).fold(0.0f64, f64::max);

                for (i, img) in chunk.iter().enumerate() {
                    let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                    let h = w / ratio;
                    results.push(LayoutResult {
                        id: img.id.clone(),
                        x: start_x + i as f64 * (w + state.gap) + w / 2.0,
                        y: cur_y + h / 2.0,
                        width: w,
                        height: h,
                    });
                }
                cur_y += max_h + state.gap;
            }
            results
        }
        LayoutMode::Masonry => {
            let cols = if state.cols > 0 { state.cols } else { 3 };
            let w = state.size;
            let mut col_y = vec![start_y; cols];
            let mut results = Vec::new();

            for img in sorted {
                let mut min_idx = 0;
                let mut min_y = col_y[0];
                for (ci, &cy) in col_y.iter().enumerate() {
                    if cy < min_y {
                        min_y = cy;
                        min_idx = ci;
                    }
                }

                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                let h = w / ratio;
                let x = start_x + min_idx as f64 * (w + state.gap);
                results.push(LayoutResult {
                    id: img.id,
                    x: x + w / 2.0,
                    y: col_y[min_idx] + h / 2.0,
                    width: w,
                    height: h,
                });
                col_y[min_idx] += h + state.gap;
            }
            results
        }
        LayoutMode::SameHeight => {
            let h = state.size;
            let mut cur_x = start_x;
            let mut results = Vec::new();

            for img in sorted {
                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                let w = h * ratio;
                results.push(LayoutResult {
                    id: img.id,
                    x: cur_x + w / 2.0,
                    y: start_y + h / 2.0,
                    width: w,
                    height: h,
                });
                cur_x += w + state.gap;
            }
            results
        }
        _ => {
            // CompactRows par défaut
            let target_h = state.size;
            let max_row_w = target_h * 5.0;
            let mut results = Vec::new();

            let scaled: Vec<_> = sorted.into_iter().map(|img| {
                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                (img.id, target_h * ratio, target_h)
            }).collect();

            let mut rows: Vec<Vec<(String, f64, f64)>> = Vec::new();
            let mut current_row = Vec::new();
            let mut current_w = 0.0;

            for (id, w, h) in scaled {
                if current_w + w > max_row_w && !current_row.is_empty() {
                    rows.push(current_row);
                    current_row = vec![(id, w, h)];
                    current_w = w + state.gap;
                } else {
                    current_row.push((id, w, h));
                    current_w += w + state.gap;
                }
            }
            if !current_row.is_empty() {
                rows.push(current_row);
            }

            let mut cur_y = start_y;
            for row in rows {
                let mut cur_x = start_x;
                for (id, w, h) in row {
                    results.push(LayoutResult {
                        id,
                        x: cur_x + w / 2.0,
                        y: cur_y + h / 2.0,
                        width: w,
                        height: h,
                    });
                    cur_x += w + state.gap;
                }
                cur_y += target_h + state.gap;
            }
            results
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_anchors_and_defaults() {
        assert_eq!(TabId::Organize.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Pomodoro.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Storyboard.anchor(), DockAnchor::BottomLeft);
        assert_eq!(TabId::Plugins.anchor(), DockAnchor::TopLeft);
        assert_eq!(TabId::Preset.anchor(), DockAnchor::TopLeft);
        assert_eq!(TabId::Domains.anchor(), DockAnchor::TopLeft);
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

        dock.update_drag(50.0, 580.0);
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

        let mut state = OrganizeState::default();
        state.layout = LayoutMode::Grid;
        state.cols = 3;
        state.size = 200.0;

        let res = apply_organize_layout(&images, &state);
        assert_eq!(res.len(), 6);
        assert_eq!(res[0].width, 200.0);
    }

    #[test]
    fn test_render_docks_png() {
        let mut pixmap = tiny_skia::Pixmap::new(1440, 900).unwrap();
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
        let store = Store::new("Test");
        let typo = Typography::new();

        // 1. Bottom docks: Organize, Pomodoro, Storyboard
        let mut dock = DockManager::new();
        dock.bottom_tabs = vec![TabId::Organize, TabId::Pomodoro, TabId::Storyboard];
        render_docks(
            &mut pixmap.as_mut(),
            &dock,
            &store,
            &typo,
            1440.0,
            900.0,
            42.0,
            0.0,
            0.0,
        );
        let _ = std::fs::create_dir_all("target");
        let _ = pixmap.save_png("target/dock_bottom.png");

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
            1440.0,
            900.0,
            42.0,
            0.0,
            0.0,
        );
        let _ = pixmap_top.save_png("target/dock_top.png");
    }
}
