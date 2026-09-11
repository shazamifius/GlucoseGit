//! Composants graphiques d'interface de Glucose (TopBar, BoardTabs, Minimap, Toasts).

use crate::icons::{draw_icon_scaled, IconType};
use crate::params::{ButtonState, Pointer, ScaledRect};
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use std::time::{Duration, Instant};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

pub const TOPBAR_HEIGHT: f32 = 44.0;
pub const TABS_HEIGHT: f32 = 34.0;
pub const TOTAL_HEADER_HEIGHT: f32 = TOPBAR_HEIGHT + TABS_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Pan,
    Text,
    Sticky,
    Arrow,
    Folder,
    Membrane,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    SelectTool(ActiveTool),
    AddImages,
    Organize,
    ToggleTimer,
    ToggleStoryboard,
    ToggleMagnet,
    ToggleTransDomain,
    ToggleCollab,
    ExportMenu,
    TogglePlugins,
    TogglePreset,
    ToggleDomains,
    SelectBoard(String),
    AddBoard,
    MinimapPan(f64, f64),
}

pub struct Toast {
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
            duration: Duration::from_millis(2500),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn alpha(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_millis() as f32;
        let total = self.duration.as_millis() as f32;
        if elapsed > total - 400.0 {
            ((total - elapsed) / 400.0).clamp(0.0, 1.0)
        } else {
            (elapsed / 150.0).clamp(0.0, 1.0)
        }
    }
}

pub struct UiState {
    pub active_tool: ActiveTool,
    pub smart_align: bool,
    pub trans_domain: bool,
    pub collab_active: bool,
    #[allow(dead_code)]
    pub hovered_btn: Option<String>,
    pub current_toast: Option<Toast>,
    pub scale_factor: f32,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            active_tool: ActiveTool::Select,
            smart_align: true,
            trans_domain: true,
            collab_active: false,
            hovered_btn: None,
            current_toast: Some(Toast::new("Bienvenue dans Glucose !")),
            scale_factor: 1.0,
        }
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        crate::theme::clamp_ui_scale(self.scale_factor)
    }

    #[inline]
    pub fn topbar_height(&self) -> f32 {
        TOPBAR_HEIGHT * self.scale()
    }

    #[inline]
    pub fn tabs_height(&self) -> f32 {
        TABS_HEIGHT * self.scale()
    }

    #[inline]
    pub fn header_height(&self) -> f32 {
        TOTAL_HEADER_HEIGHT * self.scale()
    }

    pub fn show_toast(&mut self, msg: impl Into<String>) {
        self.current_toast = Some(Toast::new(msg));
    }
}

pub fn render_ui(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
    theme: &Theme,
    pointer: Pointer,
) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;

    // Nettoyage toast expiré
    if let Some(ref t) = ui.current_toast {
        if t.is_expired() {
            ui.current_toast = None;
        }
    }

    // 1. Barre supérieure
    render_topbar(pixmap, store, ui, typo, theme, w, pointer);

    // 2. Barre d'onglets
    render_board_tabs(pixmap, store, ui, typo, theme, w, pointer);

    // 3. Minimap (en bas à droite)
    render_minimap(pixmap, store, theme, w, h, ui.scale_factor);

    // 4. Toast notification (au centre en bas)
    if let Some(ref toast) = ui.current_toast {
        render_toast(pixmap, toast, typo, theme, w, h, ui.scale_factor);
    }
}

#[derive(Debug, Clone)]
pub struct TopbarButtonDef {
    pub action: UiAction,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub icon: IconType,
    pub label: &'static str,
    pub active: bool,
    pub is_tool: bool,
    pub is_collab: bool,
}

pub struct TopbarLayout {
    pub buttons: Vec<TopbarButtonDef>,
    pub separators: Vec<f32>,
    pub img_badge: Option<(f32, String)>,
}

pub fn layout_topbar(
    width: f32,
    ui: &UiState,
    _typo: &Typography,
    board_img_count: usize,
) -> TopbarLayout {
    let mut buttons = Vec::new();
    let mut separators = Vec::new();
    let s = ui.scale();

    // Responsive design :
    // - Mode complet : width >= 1320px * scale
    // - Mode compact : 1050px * scale <= width < 1320px * scale
    // - Mode ultra-compact : width < 1050px * scale
    let is_compact = width < 1320.0 * s;
    let is_ultra = width < 1050.0 * s;

    let topbar_h = ui.topbar_height();
    let tool_size = 30.0 * s;
    let tool_y = (topbar_h - tool_size) / 2.0;

    let mut cur_x = 100.0 * s;

    // 1. Outils de base (Select, Pan)
    buttons.push(TopbarButtonDef {
        action: UiAction::SelectTool(ActiveTool::Select),
        x: cur_x,
        y: tool_y,
        w: tool_size,
        h: tool_size,
        icon: IconType::Select,
        label: "",
        active: ui.active_tool == ActiveTool::Select,
        is_tool: true,
        is_collab: false,
    });
    cur_x += tool_size + 2.0 * s;

    buttons.push(TopbarButtonDef {
        action: UiAction::SelectTool(ActiveTool::Pan),
        x: cur_x,
        y: tool_y,
        w: tool_size,
        h: tool_size,
        icon: IconType::Pan,
        label: "",
        active: ui.active_tool == ActiveTool::Pan,
        is_tool: true,
        is_collab: false,
    });
    cur_x += tool_size + 2.0 * s;

    // Séparateur 1
    separators.push(cur_x + 3.0 * s);
    cur_x += 9.0 * s;

    // 2. Annotations (Text, Sticky, Arrow, Folder, Membrane)
    let ann_tools = [
        (ActiveTool::Text, IconType::Text),
        (ActiveTool::Sticky, IconType::Sticky),
        (ActiveTool::Arrow, IconType::Arrow),
        (ActiveTool::Folder, IconType::Folder),
        (ActiveTool::Membrane, IconType::Membrane),
    ];
    for (t, icon) in ann_tools {
        buttons.push(TopbarButtonDef {
            action: UiAction::SelectTool(t),
            x: cur_x,
            y: tool_y,
            w: tool_size,
            h: tool_size,
            icon,
            label: "",
            active: ui.active_tool == t,
            is_tool: true,
            is_collab: false,
        });
        cur_x += tool_size + 2.0 * s;
    }

    // Séparateur 2
    separators.push(cur_x + 3.0 * s);
    cur_x += 9.0 * s;

    // 3. + Images
    let act_h = 28.0 * s;
    let act_y = (topbar_h - act_h) / 2.0;

    let (img_w, img_label) = if is_ultra { (tool_size, "") } else { (78.0 * s, "Images") };
    buttons.push(TopbarButtonDef {
        action: UiAction::AddImages,
        x: cur_x,
        y: act_y,
        w: img_w,
        h: act_h,
        icon: IconType::Plus,
        label: img_label,
        active: false,
        is_tool: false,
        is_collab: false,
    });
    cur_x += img_w + 6.0 * s;

    // Séparateur 3
    separators.push(cur_x + 1.0 * s);
    cur_x += 7.0 * s;

    // 4. Ordonner, Timer, Storyboard
    let panels = [
        (UiAction::Organize, IconType::Organize, "Ordonner", 84.0 * s, false),
        (UiAction::ToggleTimer, IconType::Timer, "Timer", 66.0 * s, false),
        (UiAction::ToggleStoryboard, IconType::Storyboard, "Storyboard", 96.0 * s, false),
    ];
    for (act, icon, lbl, full_w, active) in panels {
        let (btn_w, btn_lbl) = if is_ultra { (tool_size, "") } else { (full_w, lbl) };
        buttons.push(TopbarButtonDef {
            action: act,
            x: cur_x,
            y: act_y,
            w: btn_w,
            h: act_h,
            icon,
            label: btn_lbl,
            active,
            is_tool: false,
            is_collab: false,
        });
        cur_x += btn_w + 4.0 * s;
    }

    // Séparateur 4
    separators.push(cur_x + 2.0 * s);
    cur_x += 8.0 * s;

    // 5. Aimant, Trans-domaines
    let toggles = [
        (UiAction::ToggleMagnet, IconType::Magnet, "Aimant", 76.0 * s, ui.smart_align),
        (UiAction::ToggleTransDomain, IconType::TransDomain, "Trans-domaines", 118.0 * s, ui.trans_domain),
    ];
    for (act, icon, lbl, full_w, active) in toggles {
        let (btn_w, btn_lbl) = if is_ultra { (tool_size, "") } else { (full_w, lbl) };
        buttons.push(TopbarButtonDef {
            action: act,
            x: cur_x,
            y: act_y,
            w: btn_w,
            h: act_h,
            icon,
            label: btn_lbl,
            active,
            is_tool: false,
            is_collab: false,
        });
        cur_x += btn_w + 4.0 * s;
    }

    let left_end = cur_x;

    // 6. Groupe de Droite
    let (col_w, col_lbl) = if is_ultra { (tool_size, "") } else { (96.0 * s, "Collaborer") };
    let (exp_w, exp_lbl) = if is_ultra { (tool_size, "") } else { (84.0 * s, "Exporter") };
    let (plu_w, plu_lbl) = if is_compact { (tool_size, "") } else { (76.0 * s, "Plugins") };
    let (pre_w, pre_lbl) = if is_compact { (tool_size, "") } else { (72.0 * s, "Preset") };
    let (dom_w, dom_lbl) = if is_compact { (tool_size, "") } else { (88.0 * s, "Domaines") };

    let badge_w = if board_img_count > 0 { 42.0 * s } else { 0.0 };
    let right_total_w = col_w + 8.0 * s + exp_w + 8.0 * s + plu_w + 4.0 * s + pre_w + 4.0 * s + dom_w + badge_w + 16.0 * s;

    let right_start = (width - right_total_w - 12.0 * s).max(left_end + 16.0 * s);
    let mut rx = right_start;

    // Collaborer
    buttons.push(TopbarButtonDef {
        action: UiAction::ToggleCollab,
        x: rx,
        y: act_y,
        w: col_w,
        h: act_h,
        icon: IconType::Collab,
        label: col_lbl,
        active: ui.collab_active,
        is_tool: false,
        is_collab: true,
    });
    rx += col_w + 5.0 * s;

    // Séparateur avant Exporter
    separators.push(rx + 1.0 * s);
    rx += 7.0 * s;

    // Exporter
    buttons.push(TopbarButtonDef {
        action: UiAction::ExportMenu,
        x: rx,
        y: act_y,
        w: exp_w,
        h: act_h,
        icon: IconType::Export,
        label: exp_lbl,
        active: false,
        is_tool: false,
        is_collab: false,
    });
    rx += exp_w + 5.0 * s;

    // Séparateur avant Plugins
    separators.push(rx + 1.0 * s);
    rx += 7.0 * s;

    // Plugins, Preset, Domaines
    let right_actions = [
        (UiAction::TogglePlugins, IconType::Plugins, plu_lbl, plu_w),
        (UiAction::TogglePreset, IconType::Preset, pre_lbl, pre_w),
        (UiAction::ToggleDomains, IconType::Domains, dom_lbl, dom_w),
    ];
    for (act, icon, lbl, bw) in right_actions {
        buttons.push(TopbarButtonDef {
            action: act,
            x: rx,
            y: act_y,
            w: bw,
            h: act_h,
            icon,
            label: lbl,
            active: false,
            is_tool: false,
            is_collab: false,
        });
        rx += bw + 4.0 * s;
    }

    let img_badge = if board_img_count > 0 {
        Some((rx + 4.0 * s, format!("{}img", board_img_count)))
    } else {
        None
    };

    TopbarLayout {
        buttons,
        separators,
        img_badge,
    }
}

fn render_topbar(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    width: f32,
    pointer: Pointer,
) {
    let (mx, my) = (pointer.x, pointer.y);
    let s = ui.scale();
    let topbar_h = ui.topbar_height();

    // Fond header
    let mut bg_paint = Paint::default();
    bg_paint.set_color(theme.bg_header);
    if let Some(rect) = Rect::from_xywh(0.0, 0.0, width, topbar_h) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure inférieure
    let mut border_paint = Paint::default();
    border_paint.set_color(theme.border_subtle);
    if let Some(rect) = Rect::from_xywh(0.0, topbar_h - 1.0, width, 1.0) {
        pixmap.fill_rect(rect, &border_paint, Transform::identity(), None);
    }

    // Logo GLUCOSE
    typo.draw_text(
        pixmap,
        "GLUCOSE",
        12.0 * s,
        (topbar_h - 14.0 * s) / 2.0,
        TextStyle { size: 14.0 * s, color: theme.text_primary, bold: true },
    );

    let img_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let layout = layout_topbar(width, ui, typo, img_count);

    // Séparateurs
    for sep_x in layout.separators {
        draw_separator(pixmap, sep_x, (topbar_h - 20.0 * s) / 2.0, 20.0 * s, theme.border_subtle);
    }

    // Boutons
    for btn in &layout.buttons {
        let is_hover = mx >= btn.x && mx < btn.x + btn.w && my >= btn.y && my < btn.y + btn.h;
        let state = ButtonState { active: btn.active, hover: is_hover };
        if btn.is_tool {
            draw_tool_button(pixmap, theme, box_of(btn, s), btn.icon, state);
        } else {
            draw_action_button(pixmap, typo, theme, box_of(btn, s), btn.icon, btn.label, state);
            if btn.is_collab && ui.collab_active {
                // Pastille verte #10b981
                let mut dot_paint = Paint::default();
                dot_paint.set_color(Color::from_rgba8(16, 185, 129, 255));
                dot_paint.anti_alias = true;
                let mut dot_pb = PathBuilder::new();
                dot_pb.push_circle(btn.x + 18.0 * s, btn.y + 7.0 * s, 3.0 * s);
                if let Some(p) = dot_pb.finish() {
                    pixmap.fill_path(&p, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
        }
    }

    // Badge nombre d'images à droite
    if let Some((badge_x, ref badge_txt)) = layout.img_badge {
        typo.draw_text(
            pixmap,
            badge_txt,
            badge_x,
            (topbar_h - 11.0 * s) / 2.0,
            TextStyle { size: 11.0 * s, color: theme.badge_text, bold: false },
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabButtonLayout {
    pub board_id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_active: bool,
    pub is_plus: bool,
}

pub fn layout_tabs(
    store: &Store,
    typo: &Typography,
    y_start: f32,
    scale: f32,
) -> Vec<TabButtonLayout> {
    let s = crate::theme::clamp_ui_scale(scale);
    let mut layouts = Vec::new();
    let mut tab_x = 8.0 * s;
    let tabs_h = TABS_HEIGHT * s;
    let active_id = &store.project.active_board_id;

    for board in &store.project.boards {
        let is_active = &board.id == active_id;
        let (tw, _) = typo.measure_text(&board.name, 12.0 * s, is_active);
        let tab_w = tw + 28.0 * s;
        layouts.push(TabButtonLayout {
            board_id: board.id.clone(),
            name: board.name.clone(),
            x: tab_x,
            y: y_start,
            width: tab_w,
            height: tabs_h,
            is_active,
            is_plus: false,
        });
        tab_x += tab_w + 4.0 * s;
    }

    // Bouton + (créer un board)
    layouts.push(TabButtonLayout {
        board_id: String::new(),
        name: "+".into(),
        x: tab_x,
        y: y_start,
        width: 30.0 * s,
        height: tabs_h,
        is_active: false,
        is_plus: true,
    });

    layouts
}

fn render_board_tabs(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    width: f32,
    pointer: Pointer,
) {
    let (mx, my) = (pointer.x, pointer.y);
    let s = ui.scale();
    let y_start = ui.topbar_height();
    let tabs_h = ui.tabs_height();

    // Fond tabs
    let mut bg_paint = Paint::default();
    bg_paint.set_color(theme.bg_canvas);
    if let Some(rect) = Rect::from_xywh(0.0, y_start, width, tabs_h) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure inférieure
    let mut border_paint = Paint::default();
    border_paint.set_color(theme.border_subtle);
    if let Some(rect) = Rect::from_xywh(0.0, y_start + tabs_h - 1.0, width, 1.0) {
        pixmap.fill_rect(rect, &border_paint, Transform::identity(), None);
    }

    let tabs = layout_tabs(store, typo, y_start, s);
    for tab in tabs {
        let is_hover = mx >= tab.x && mx < tab.x + tab.width && my >= tab.y && my < tab.y + tab.height;

        if tab.is_plus {
            if is_hover {
                let mut p_paint = Paint::default();
                p_paint.set_color(theme.bg_hover);
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 5.0 * s, 24.0 * s, 24.0 * s) {
                    pixmap.fill_rect(rect, &p_paint, Transform::identity(), None);
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
            if is_hover && !tab.is_active {
                let mut h_paint = Paint::default();
                h_paint.set_color(theme.bg_hover);
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 4.0 * s, tab.width, tabs_h - 6.0 * s) {
                    pixmap.fill_rect(rect, &h_paint, Transform::identity(), None);
                }
            }

            let text_color = if tab.is_active {
                theme.text_primary
            } else {
                theme.text_secondary
            };

            typo.draw_text(
                pixmap,
                &tab.name,
                tab.x + 14.0 * s,
                y_start + 10.0 * s,
                TextStyle { size: 12.0 * s, color: text_color, bold: tab.is_active },
            );

            // Ligne d'accentuation inférieure pour l'onglet actif
            if tab.is_active {
                let mut line_paint = Paint::default();
                line_paint.set_color(theme.accent_primary);
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + tabs_h - 2.0 * s, tab.width, 2.0 * s) {
                    pixmap.fill_rect(rect, &line_paint, Transform::identity(), None);
                }
            }
        }
    }
}

fn draw_separator(pixmap: &mut PixmapMut, x: f32, y: f32, h: f32, color: Color) -> f32 {
    let mut paint = Paint::default();
    paint.set_color(color);
    if let Some(rect) = Rect::from_xywh(x, y, 1.0, h) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
    9.0
}

fn push_ui_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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

/// Rectangle d'un bouton de barre d'outils, échelle UI comprise.
fn box_of(btn: &TopbarButtonDef, scale: f32) -> ScaledRect {
    ScaledRect { x: btn.x, y: btn.y, w: btn.w, h: btn.h, scale }
}

fn draw_tool_button(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(theme.border_accent);
        sp.anti_alias = true;
        let stroke = Stroke { width: 1.0 * scale, ..Default::default() };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x + 0.5 * scale, y + 0.5 * scale, w - 1.0 * scale, h - 1.0 * scale, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let icon_color = if active {
        theme.text_accent
    } else if hover {
        theme.text_primary
    } else {
        theme.text_muted
    };

    let icon_size = 14.0 * scale;
    let icon_x = x + (w - icon_size) / 2.0;
    let icon_y = y + (h - icon_size) / 2.0;
    draw_icon_scaled(pixmap, icon, icon_x, icon_y, icon_size, icon_color, 1.4 * scale);
}

fn draw_action_button(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    label: &str,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(theme.border_accent);
        sp.anti_alias = true;
        let stroke = Stroke { width: 1.0 * scale, ..Default::default() };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x + 0.5 * scale, y + 0.5 * scale, w - 1.0 * scale, h - 1.0 * scale, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let color = if active {
        theme.text_accent
    } else if hover {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let icon_size = 14.0 * scale;
    if label.is_empty() {
        let icon_x = x + (w - icon_size) / 2.0;
        let icon_y = y + (h - icon_size) / 2.0;
        draw_icon_scaled(pixmap, icon, icon_x, icon_y, icon_size, color, 1.3 * scale);
    } else {
        let icon_y = y + (h - icon_size) / 2.0;
        draw_icon_scaled(pixmap, icon, x + 8.0 * scale, icon_y, icon_size, color, 1.3 * scale);
        typo.draw_text(pixmap, label, x + 26.0 * scale, y + (h - 12.0 * scale) / 2.0, TextStyle { size: 12.0 * scale, color, bold: active });
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MinimapBounds {
    pub mm_x: f32,
    pub mm_y: f32,
    pub mm_w: f32,
    pub mm_h: f32,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub span_x: f64,
    pub span_y: f64,
    pub scale: f32,
    pub cam_left: f64,
    pub cam_top: f64,
    pub vp_w: f64,
    pub vp_h: f64,
}

pub fn layout_minimap(
    store: &Store,
    screen_w: f32,
    screen_h: f32,
    scale: f32,
) -> Option<MinimapBounds> {
    let board = store.active_board()?;
    let s = crate::theme::clamp_ui_scale(scale);
    let mm_w = 180.0f32 * s;
    let mm_h = 120.0f32 * s;
    let mm_x = screen_w - mm_w - 16.0 * s;
    let mm_y = screen_h - mm_h - 16.0 * s;
    let header_h = TOTAL_HEADER_HEIGHT * s;

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for img in &board.images {
        min_x = min_x.min(img.x - img.width / 2.0);
        min_y = min_y.min(img.y - img.height / 2.0);
        max_x = max_x.max(img.x + img.width / 2.0);
        max_y = max_y.max(img.y + img.height / 2.0);
    }

    for ann in &board.annotations {
        match ann {
            Annotation::Text { x, y, width, height, .. } => {
                let w = width.unwrap_or(200.0);
                let h = height.unwrap_or(48.0);
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x + w);
                max_y = max_y.max(*y + h);
            }
            Annotation::Sticky { x, y, width, height, .. } => {
                let w = width.unwrap_or(160.0);
                let h = height.unwrap_or(120.0);
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x + w);
                max_y = max_y.max(*y + h);
            }
            Annotation::Membrane { x, y, width, height, .. } => {
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x + *width);
                max_y = max_y.max(*y + *height);
            }
            Annotation::Arrow { x, y, x2, y2, .. } => {
                min_x = min_x.min(*x).min(*x2);
                min_y = min_y.min(*y).min(*y2);
                max_x = max_x.max(*x).max(*x2);
                max_y = max_y.max(*y).max(*y2);
            }
        }
    }

    let vp = &board.viewport;
    let vp_w = screen_w as f64 / vp.scale;
    let vp_h = (screen_h as f64 - header_h as f64) / vp.scale;
    let cam_left = -vp.x / vp.scale;
    let cam_top = -vp.y / vp.scale;

    min_x = min_x.min(cam_left) - 200.0;
    min_y = min_y.min(cam_top) - 200.0;
    max_x = max_x.max(cam_left + vp_w) + 200.0;
    max_y = max_y.max(cam_top + vp_h) + 200.0;

    // Garde-fou : un viewport corrompu (échelle nulle, NaN) ou un board vide
    // laisserait des bornes infinies, puis des coordonnées NaN transmises au
    // rasterizer. On renonce alors à la minimap plutôt que de dessiner du bruit.
    let finite = [min_x, min_y, max_x, max_y, cam_left, cam_top, vp_w, vp_h];
    if finite.iter().any(|v| !v.is_finite()) {
        return None;
    }

    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);

    let scale_x = (mm_w - 12.0 * s) / span_x as f32;
    let scale_y = (mm_h - 12.0 * s) / span_y as f32;
    let minimap_scale = scale_x.min(scale_y);

    Some(MinimapBounds {
        mm_x,
        mm_y,
        mm_w,
        mm_h,
        min_x,
        min_y,
        max_x,
        max_y,
        span_x,
        span_y,
        scale: minimap_scale,
        cam_left,
        cam_top,
        vp_w,
        vp_h,
    })
}

fn render_minimap(pixmap: &mut PixmapMut, store: &Store, theme: &Theme, w: f32, h: f32, scale: f32) {
    let s = crate::theme::clamp_ui_scale(scale);
    let mb = match layout_minimap(store, w, h, s) {
        Some(m) => m,
        None => return,
    };

    let board = match store.active_board() {
        Some(b) => b,
        None => return,
    };

    // Fond minimap
    let mut bg_paint = Paint::default();
    bg_paint.set_color(theme.minimap_bg);
    if let Some(rect) = Rect::from_xywh(mb.mm_x, mb.mm_y, mb.mm_w, mb.mm_h) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure minimap
    let mut border_paint = Paint::default();
    border_paint.set_color(theme.minimap_border);
    let stroke = Stroke { width: 1.0 * s, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.move_to(mb.mm_x, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y + mb.mm_h);
    pb.line_to(mb.mm_x, mb.mm_y + mb.mm_h);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    let pad = 6.0 * s;

    // Dessine miniatures images
    let mut item_paint = Paint::default();
    item_paint.set_color(theme.minimap_element);
    for img in &board.images {
        let ix = mb.mm_x + pad + ((img.x - img.width / 2.0 - mb.min_x) as f32 * mb.scale);
        let iy = mb.mm_y + pad + ((img.y - img.height / 2.0 - mb.min_y) as f32 * mb.scale);
        let iw = (img.width as f32 * mb.scale).max(2.0 * s);
        let ih = (img.height as f32 * mb.scale).max(2.0 * s);
        if let Some(r) = Rect::from_xywh(ix, iy, iw, ih) {
            pixmap.fill_rect(r, &item_paint, Transform::identity(), None);
        }
    }

    // Dessine miniatures annotations & stickies & membranes
    let mut ann_paint = Paint::default();
    ann_paint.set_color(theme.text_muted);
    let mut membrane_paint = Paint::default();
    membrane_paint.set_color(theme.accent_subtle);

    for ann in &board.annotations {
        match ann {
            Annotation::Text { x, y, width, height, .. } => {
                let aw = width.unwrap_or(200.0) as f32 * mb.scale;
                let ah = height.unwrap_or(48.0) as f32 * mb.scale;
                let ax = mb.mm_x + pad + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + pad + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(2.0 * s), ah.max(2.0 * s)) {
                    pixmap.fill_rect(r, &ann_paint, Transform::identity(), None);
                }
            }
            Annotation::Sticky { x, y, width, height, .. } => {
                let aw = width.unwrap_or(160.0) as f32 * mb.scale;
                let ah = height.unwrap_or(120.0) as f32 * mb.scale;
                let ax = mb.mm_x + pad + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + pad + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(2.0 * s), ah.max(2.0 * s)) {
                    pixmap.fill_rect(r, &ann_paint, Transform::identity(), None);
                }
            }
            Annotation::Membrane { x, y, width, height, .. } => {
                let aw = *width as f32 * mb.scale;
                let ah = *height as f32 * mb.scale;
                let ax = mb.mm_x + pad + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + pad + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(4.0 * s), ah.max(4.0 * s)) {
                    pixmap.fill_rect(r, &membrane_paint, Transform::identity(), None);
                }
            }
            _ => {}
        }
    }

    // Rectangle de la caméra
    let cx = mb.mm_x + pad + ((mb.cam_left - mb.min_x) as f32 * mb.scale);
    let cy = mb.mm_y + pad + ((mb.cam_top - mb.min_y) as f32 * mb.scale);
    let cw = (mb.vp_w as f32 * mb.scale).max(4.0 * s);
    let ch = (mb.vp_h as f32 * mb.scale).max(4.0 * s);

    let mut cam_paint = Paint::default();
    cam_paint.set_color(theme.minimap_viewport);
    let cam_stroke = Stroke { width: 1.5 * s, ..Default::default() };
    let mut cam_pb = PathBuilder::new();
    cam_pb.move_to(cx, cy);
    cam_pb.line_to(cx + cw, cy);
    cam_pb.line_to(cx + cw, cy + ch);
    cam_pb.line_to(cx, cy + ch);
    cam_pb.close();
    if let Some(path) = cam_pb.finish() {
        pixmap.stroke_path(&path, &cam_paint, &cam_stroke, Transform::identity(), None);
    }
}

fn render_toast(
    pixmap: &mut PixmapMut,
    toast: &Toast,
    typo: &Typography,
    theme: &Theme,
    w: f32,
    h: f32,
    scale: f32,
) {
    let alpha = toast.alpha();
    if alpha <= 0.01 {
        return;
    }

    let s = crate::theme::clamp_ui_scale(scale);
    let (tw, _) = typo.measure_text(&toast.message, 13.0 * s, false);
    let toast_w = tw + 40.0 * s;
    let toast_h = 36.0 * s;
    let toast_x = (w - toast_w) / 2.0;
    let toast_y = h - 64.0 * s;

    // Fond pilule avec opacité animée
    let mut bg_paint = Paint::default();
    let a_byte = ((theme.toast_bg.alpha() * alpha * 255.0) as u8).max(1);
    bg_paint.set_color(Color::from_rgba8(
        (theme.toast_bg.red() * 255.0) as u8,
        (theme.toast_bg.green() * 255.0) as u8,
        (theme.toast_bg.blue() * 255.0) as u8,
        a_byte,
    ));

    let mut pb = PathBuilder::new();
    let r = 18.0 * s;
    pb.move_to(toast_x + r, toast_y);
    pb.line_to(toast_x + toast_w - r, toast_y);
    pb.quad_to(toast_x + toast_w, toast_y, toast_x + toast_w, toast_y + r);
    pb.line_to(toast_x + toast_w, toast_y + toast_h - r);
    pb.quad_to(toast_x + toast_w, toast_y + toast_h, toast_x + toast_w - r, toast_y + toast_h);
    pb.line_to(toast_x + r, toast_y + toast_h);
    pb.quad_to(toast_x, toast_y + toast_h, toast_x, toast_y + toast_h - r);
    pb.line_to(toast_x, toast_y + r);
    pb.quad_to(toast_x, toast_y, toast_x + r, toast_y);

    let mut border_paint = Paint::default();
    let b_byte = ((theme.toast_border.alpha() * alpha * 255.0) as u8).max(1);
    border_paint.set_color(Color::from_rgba8(
        (theme.toast_border.red() * 255.0) as u8,
        (theme.toast_border.green() * 255.0) as u8,
        (theme.toast_border.blue() * 255.0) as u8,
        b_byte,
    ));
    let stroke = Stroke { width: 1.0 * s, ..Default::default() };

    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // Texte centré
    let text_a = ((theme.toast_text.alpha() * alpha * 255.0) as u8).max(1);
    let text_color = Color::from_rgba8(
        (theme.toast_text.red() * 255.0) as u8,
        (theme.toast_text.green() * 255.0) as u8,
        (theme.toast_text.blue() * 255.0) as u8,
        text_a,
    );
    typo.draw_text(
        pixmap,
        &toast.message,
        toast_x + 20.0 * s,
        toast_y + (toast_h - 13.0 * s) / 2.0,
        TextStyle { size: 13.0 * s, color: text_color, bold: false },
    );
}

/// Détecte si un clic souris se situe sur l'interface et retourne l'action associée
pub fn handle_ui_click(
    x: f32,
    y: f32,
    screen_w: f32,
    screen_h: f32,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
) -> Option<UiAction> {
    let topbar_h = ui.topbar_height();
    let header_h = ui.header_height();
    let s = ui.scale();

    if y < topbar_h {
        let img_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
        let layout = layout_topbar(screen_w, ui, typo, img_count);
        for btn in layout.buttons {
            if x >= btn.x && x < btn.x + btn.w && y >= btn.y && y < btn.y + btn.h {
                match btn.action {
                    UiAction::ToggleMagnet => {
                        ui.smart_align = !ui.smart_align;
                        ui.show_toast(if ui.smart_align { "✨ Aimant activé" } else { "Aimant désactivé" });
                    }
                    UiAction::ToggleCollab => {
                        ui.collab_active = !ui.collab_active;
                        ui.show_toast(if ui.collab_active { "🌐 Collaboration connectée" } else { "Collaboration déconnectée" });
                    }
                    _ => {}
                }
                return Some(btn.action);
            }
        }
    } else if y >= topbar_h && y < header_h {
        // Clic sur la BoardTabs bar via layout_tabs unifié
        let tabs = layout_tabs(store, typo, topbar_h, s);
        for tab in tabs {
            if x >= tab.x && x < tab.x + tab.width && y >= tab.y && y < tab.y + tab.height {
                if tab.is_plus {
                    return Some(UiAction::AddBoard);
                } else {
                    return Some(UiAction::SelectBoard(tab.board_id));
                }
            }
        }
    } else {
        // Clic sur la Minimap via layout_minimap unifié (bornes réelles)
        if let Some(mb) = layout_minimap(store, screen_w, screen_h, s) {
            let pad = 6.0 * s;
            if x >= mb.mm_x && x <= mb.mm_x + mb.mm_w && y >= mb.mm_y && y <= mb.mm_y + mb.mm_h {
                let rel_x = ((x - mb.mm_x - pad) / (mb.mm_w - 2.0 * pad).max(1.0)).clamp(0.0, 1.0) as f64;
                let rel_y = ((y - mb.mm_y - pad) / (mb.mm_h - 2.0 * pad).max(1.0)).clamp(0.0, 1.0) as f64;
                let target_wx = mb.min_x + rel_x * mb.span_x;
                let target_wy = mb.min_y + rel_y * mb.span_y;
                return Some(UiAction::MinimapPan(target_wx, target_wy));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typography::Typography;

    #[test]
    fn test_topbar_no_overlap_across_all_resolutions() {
        let ui = UiState::new();
        let typo = Typography::new();
        let test_widths = [640.0, 800.0, 1024.0, 1280.0, 1440.0, 1920.0, 2560.0, 3840.0];

        for width in test_widths {
            let layout = layout_topbar(width, &ui, &typo, 5);
            assert!(!layout.buttons.is_empty(), "Buttons should not be empty for width {}", width);

            // Vérifier que chaque bouton a une largeur et hauteur positive
            for btn in &layout.buttons {
                assert!(btn.w > 0.0, "Button width must be positive for width {}", width);
                assert!(btn.h > 0.0, "Button height must be positive for width {}", width);
            }

            // Vérifier qu'aucun bouton ne se chevauche
            for i in 0..layout.buttons.len() {
                for j in (i + 1)..layout.buttons.len() {
                    let b1 = &layout.buttons[i];
                    let b2 = &layout.buttons[j];
                    let overlap_x = b1.x < (b2.x + b2.w) && (b1.x + b1.w) > b2.x;
                    let overlap_y = b1.y < (b2.y + b2.h) && (b1.y + b1.h) > b2.y;
                    assert!(
                        !(overlap_x && overlap_y),
                        "Collision detected at screen width {} between button {} and button {} (b1: [{}, {}], b2: [{}, {}])",
                        width, i, j, b1.x, b1.x + b1.w, b2.x, b2.x + b2.w
                    );
                }
            }
        }
    }

    #[test]
    fn test_topbar_responsive_collapse() {
        let ui = UiState::new();
        let typo = Typography::new();

        // Mode ultra-compact (< 1050px) : les boutons d'action doivent être réduits à 30px
        let layout_ultra = layout_topbar(900.0, &ui, &typo, 0);
        for btn in &layout_ultra.buttons {
            if btn.label.is_empty() {
                assert_eq!(btn.w, 30.0);
            }
        }

        // Mode complet (> 1320px) : les boutons secondaires ont leurs labels
        let layout_full = layout_topbar(1600.0, &ui, &typo, 2);
        let plugins_btn = layout_full.buttons.iter().find(|b| b.action == UiAction::TogglePlugins).unwrap();
        assert_eq!(plugins_btn.label, "Plugins");
        assert!(plugins_btn.w > 30.0);
    }

    #[test]
    fn test_click_tabs_selects_correct_board() {
        let mut store = Store::new("Tabs Test");
        let b1 = store.project.active_board_id.clone();
        let b2 = store.add_board("Very Long Custom Board Name That Could Shift Layout");
        let b3 = store.add_board("Short");
        let b4 = store.add_board("Board 4");

        let mut ui = UiState::new();
        let typo = Typography::new();

        // Tester en activant tour à tour chaque onglet pour prouver l'absence de dérive (R-07)
        for target_id in [&b1, &b2, &b3, &b4] {
            store.set_active_board_id(target_id);

            let tabs = layout_tabs(&store, &typo, ui.topbar_height(), ui.scale());
            assert_eq!(tabs.len(), 5); // 4 boards + 1 bouton '+'

            for tab in &tabs {
                let click_x = tab.x + tab.width / 2.0;
                let click_y = tab.y + tab.height / 2.0;

                let action = handle_ui_click(click_x, click_y, 1440.0, 900.0, &store, &mut ui, &typo);
                if tab.is_plus {
                    assert_eq!(action, Some(UiAction::AddBoard));
                } else {
                    assert_eq!(action, Some(UiAction::SelectBoard(tab.board_id.clone())));
                }
            }
        }
    }

    #[test]
    fn test_minimap_click_returns_minimap_pan() {
        let mut store = Store::new("Minimap Test");
        let bid = store.project.active_board_id.clone();
        let img = glucose_core::types::BoardImage::new("img1", 500.0, 300.0, 400.0, 300.0);
        store.add_image(&bid, img);

        let mut ui = UiState::new();
        let typo = Typography::new();

        let mb = layout_minimap(&store, 1440.0, 900.0, ui.scale()).expect("Minimap should have valid layout");

        // Clic au centre de la minimap
        let click_x = mb.mm_x + mb.mm_w / 2.0;
        let click_y = mb.mm_y + mb.mm_h / 2.0;

        let action = handle_ui_click(click_x, click_y, 1440.0, 900.0, &store, &mut ui, &typo);
        assert!(matches!(action, Some(UiAction::MinimapPan(..))), "Minimap click must produce MinimapPan action");
    }

    #[test]
    fn test_ui_dpi_scaling_and_hit_testing_at_150_percent() {
        let mut store = Store::new("DPI Test");
        let b1 = store.project.active_board_id.clone();
        let b2 = store.add_board("Scaled Board");
        store.set_active_board_id(&b2);

        let mut ui = UiState::new();
        ui.scale_factor = 1.5;
        let typo = Typography::new();

        // 1. Topbar à 150 %
        assert_eq!(ui.topbar_height(), 44.0 * 1.5);
        assert_eq!(ui.tabs_height(), 34.0 * 1.5);
        assert_eq!(ui.header_height(), 78.0 * 1.5);

        let layout = layout_topbar(1920.0, &ui, &typo, 1);
        let first_tool = &layout.buttons[0];
        assert_eq!(first_tool.w, 30.0 * 1.5);
        assert_eq!(first_tool.h, 30.0 * 1.5);

        // Clic sur l'outil Pan à 150 %
        let pan_btn = &layout.buttons[1];
        let click_pan_x = pan_btn.x + pan_btn.w / 2.0;
        let click_pan_y = pan_btn.y + pan_btn.h / 2.0;
        let act = handle_ui_click(click_pan_x, click_pan_y, 1920.0, 1080.0, &store, &mut ui, &typo);
        assert_eq!(act, Some(UiAction::SelectTool(ActiveTool::Pan)));

        // 2. Tabs à 150 %
        let tabs = layout_tabs(&store, &typo, ui.topbar_height(), ui.scale());
        assert_eq!(tabs.len(), 3); // b1, b2, plus
        let first_tab = &tabs[0];
        assert_eq!(first_tab.height, 34.0 * 1.5);
        assert_eq!(first_tab.y, 44.0 * 1.5);

        // Clic sur le premier onglet (b1)
        let click_tab_x = first_tab.x + first_tab.width / 2.0;
        let click_tab_y = first_tab.y + first_tab.height / 2.0;
        let tab_act = handle_ui_click(click_tab_x, click_tab_y, 1920.0, 1080.0, &store, &mut ui, &typo);
        assert_eq!(tab_act, Some(UiAction::SelectBoard(b1)));

        // 3. Minimap à 150 %
        let mb = layout_minimap(&store, 1920.0, 1080.0, ui.scale()).expect("Minimap valid layout");
        assert_eq!(mb.mm_w, 180.0 * 1.5);
        assert_eq!(mb.mm_h, 120.0 * 1.5);

        let mm_click_x = mb.mm_x + mb.mm_w / 2.0;
        let mm_click_y = mb.mm_y + mb.mm_h / 2.0;
        let mm_act = handle_ui_click(mm_click_x, mm_click_y, 1920.0, 1080.0, &store, &mut ui, &typo);
        assert!(matches!(mm_act, Some(UiAction::MinimapPan(..))));
    }

    #[test]
    fn test_toast_alpha_phases_and_plateau() {
        let mut toast = Toast::new("Test Toast");
        let base_instant = Instant::now();

        // 1. Début du fondu entrant (0 ms)
        toast.created_at = base_instant;
        assert_eq!(toast.alpha(), 0.0);

        // 2. Mi-course du fondu entrant (75 ms / 150 ms)
        toast.created_at = base_instant - Duration::from_millis(75);
        assert!((toast.alpha() - 0.5).abs() < 0.05);

        // 3. Fin du fondu entrant (150 ms)
        toast.created_at = base_instant - Duration::from_millis(150);
        assert_eq!(toast.alpha(), 1.0);

        // 4. Plateau statique où aucun repaint n'est requis (1000 ms)
        toast.created_at = base_instant - Duration::from_millis(1000);
        assert_eq!(toast.alpha(), 1.0);

        // 5. Début du fondu sortant (2100 ms = 2500 - 400)
        toast.created_at = base_instant - Duration::from_millis(2100);
        assert_eq!(toast.alpha(), 1.0);

        // 6. Mi-course du fondu sortant (2300 ms = 2500 - 200)
        toast.created_at = base_instant - Duration::from_millis(2300);
        assert!((toast.alpha() - 0.5).abs() < 0.05);

        // 7. Expiration (>= 2500 ms)
        toast.created_at = base_instant - Duration::from_millis(2500);
        assert_eq!(toast.alpha(), 0.0);
        assert!(toast.is_expired());
    }
}
