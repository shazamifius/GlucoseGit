//! Composants graphiques d'interface de Glucose (TopBar, BoardTabs, Minimap, Toasts).

use crate::icons::{draw_icon, IconType};
use crate::typography::Typography;
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

    pub fn show_toast(&mut self, msg: impl Into<String>) {
        self.current_toast = Some(Toast::new(msg));
    }
}

pub fn render_ui(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
    mouse_x: f32,
    mouse_y: f32,
) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;

    // Nettoyage toast expiré
    if let Some(ref t) = ui.current_toast {
        if t.is_expired() {
            ui.current_toast = None;
        }
    }

    // 1. Barre supérieure (44px)
    render_topbar(pixmap, store, ui, typo, w, mouse_x, mouse_y);

    // 2. Barre d'onglets (34px sous la topbar)
    render_board_tabs(pixmap, store, ui, typo, w, mouse_x, mouse_y);

    // 3. Minimap (en bas à droite)
    render_minimap(pixmap, store, w, h);

    // 4. Toast notification (au centre en bas)
    if let Some(ref toast) = ui.current_toast {
        render_toast(pixmap, toast, typo, w, h);
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

    // Responsive design :
    // - Mode complet : width >= 1320px
    // - Mode compact : 1050px <= width < 1320px (Plugins, Preset, Domaines en icônes seules)
    // - Mode ultra-compact : width < 1050px (tous les boutons d'action en icônes seules)
    let is_compact = width < 1320.0;
    let is_ultra = width < 1050.0;

    let mut cur_x = 100.0;

    // 1. Outils de base (Select, Pan)
    buttons.push(TopbarButtonDef {
        action: UiAction::SelectTool(ActiveTool::Select),
        x: cur_x,
        y: 7.0,
        w: 30.0,
        h: 30.0,
        icon: IconType::Select,
        label: "",
        active: ui.active_tool == ActiveTool::Select,
        is_tool: true,
        is_collab: false,
    });
    cur_x += 32.0;

    buttons.push(TopbarButtonDef {
        action: UiAction::SelectTool(ActiveTool::Pan),
        x: cur_x,
        y: 7.0,
        w: 30.0,
        h: 30.0,
        icon: IconType::Pan,
        label: "",
        active: ui.active_tool == ActiveTool::Pan,
        is_tool: true,
        is_collab: false,
    });
    cur_x += 32.0;

    // Séparateur 1
    separators.push(cur_x + 3.0);
    cur_x += 9.0;

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
            y: 7.0,
            w: 30.0,
            h: 30.0,
            icon,
            label: "",
            active: ui.active_tool == t,
            is_tool: true,
            is_collab: false,
        });
        cur_x += 32.0;
    }

    // Séparateur 2
    separators.push(cur_x + 3.0);
    cur_x += 9.0;

    // 3. + Images
    let (img_w, img_label) = if is_ultra { (30.0, "") } else { (78.0, "Images") };
    buttons.push(TopbarButtonDef {
        action: UiAction::AddImages,
        x: cur_x,
        y: 8.0,
        w: img_w,
        h: 28.0,
        icon: IconType::Plus,
        label: img_label,
        active: false,
        is_tool: false,
        is_collab: false,
    });
    cur_x += img_w + 6.0;

    // Séparateur 3
    separators.push(cur_x + 1.0);
    cur_x += 7.0;

    // 4. Ordonner, Timer, Storyboard
    let panels = [
        (UiAction::Organize, IconType::Organize, "Ordonner", 84.0f32, false),
        (UiAction::ToggleTimer, IconType::Timer, "Timer", 66.0f32, false),
        (UiAction::ToggleStoryboard, IconType::Storyboard, "Storyboard", 96.0f32, false),
    ];
    for (act, icon, lbl, full_w, active) in panels {
        let (btn_w, btn_lbl) = if is_ultra { (30.0, "") } else { (full_w, lbl) };
        buttons.push(TopbarButtonDef {
            action: act,
            x: cur_x,
            y: 8.0,
            w: btn_w,
            h: 28.0,
            icon,
            label: btn_lbl,
            active,
            is_tool: false,
            is_collab: false,
        });
        cur_x += btn_w + 4.0;
    }

    // Séparateur 4
    separators.push(cur_x + 2.0);
    cur_x += 8.0;

    // 5. Aimant, Trans-domaines
    let toggles = [
        (UiAction::ToggleMagnet, IconType::Magnet, "Aimant", 76.0f32, ui.smart_align),
        (UiAction::ToggleTransDomain, IconType::TransDomain, "Trans-domaines", 118.0f32, ui.trans_domain),
    ];
    for (act, icon, lbl, full_w, active) in toggles {
        let (btn_w, btn_lbl) = if is_ultra { (30.0, "") } else { (full_w, lbl) };
        buttons.push(TopbarButtonDef {
            action: act,
            x: cur_x,
            y: 8.0,
            w: btn_w,
            h: 28.0,
            icon,
            label: btn_lbl,
            active,
            is_tool: false,
            is_collab: false,
        });
        cur_x += btn_w + 4.0;
    }

    let left_end = cur_x;

    // 6. Groupe de Droite
    let (col_w, col_lbl) = if is_ultra { (30.0, "") } else { (96.0, "Collaborer") };
    let (exp_w, exp_lbl) = if is_ultra { (30.0, "") } else { (84.0, "Exporter") };
    let (plu_w, plu_lbl) = if is_compact { (30.0, "") } else { (76.0, "Plugins") };
    let (pre_w, pre_lbl) = if is_compact { (30.0, "") } else { (72.0, "Preset") };
    let (dom_w, dom_lbl) = if is_compact { (30.0, "") } else { (88.0, "Domaines") };

    let badge_w = if board_img_count > 0 { 42.0 } else { 0.0 };
    let right_total_w = col_w + 8.0 + exp_w + 8.0 + plu_w + 4.0 + pre_w + 4.0 + dom_w + badge_w + 16.0;

    let right_start = (width - right_total_w - 12.0).max(left_end + 16.0);
    let mut rx = right_start;

    // Collaborer
    buttons.push(TopbarButtonDef {
        action: UiAction::ToggleCollab,
        x: rx,
        y: 8.0,
        w: col_w,
        h: 28.0,
        icon: IconType::Collab,
        label: col_lbl,
        active: ui.collab_active,
        is_tool: false,
        is_collab: true,
    });
    rx += col_w + 5.0;

    // Séparateur avant Exporter
    separators.push(rx + 1.0);
    rx += 7.0;

    // Exporter
    buttons.push(TopbarButtonDef {
        action: UiAction::ExportMenu,
        x: rx,
        y: 8.0,
        w: exp_w,
        h: 28.0,
        icon: IconType::Export,
        label: exp_lbl,
        active: false,
        is_tool: false,
        is_collab: false,
    });
    rx += exp_w + 5.0;

    // Séparateur avant Plugins
    separators.push(rx + 1.0);
    rx += 7.0;

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
            y: 8.0,
            w: bw,
            h: 28.0,
            icon,
            label: lbl,
            active: false,
            is_tool: false,
            is_collab: false,
        });
        rx += bw + 4.0;
    }

    let img_badge = if board_img_count > 0 {
        Some((rx + 4.0, format!("{}img", board_img_count)))
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
    width: f32,
    mx: f32,
    my: f32,
) {
    // Fond #1A1A1A
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgba8(26, 26, 26, 255));
    if let Some(rect) = Rect::from_xywh(0.0, 0.0, width, TOPBAR_HEIGHT) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure inférieure #2A2A2A
    let mut border_paint = Paint::default();
    border_paint.set_color(Color::from_rgba8(42, 42, 42, 255));
    if let Some(rect) = Rect::from_xywh(0.0, TOPBAR_HEIGHT - 1.0, width, 1.0) {
        pixmap.fill_rect(rect, &border_paint, Transform::identity(), None);
    }

    // Logo GLUCOSE
    typo.draw_text(
        pixmap,
        "GLUCOSE",
        12.0,
        14.0,
        14.0,
        Color::from_rgba8(255, 255, 255, 255),
        true,
    );

    let img_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let layout = layout_topbar(width, ui, typo, img_count);

    // Séparateurs (filet 1px #2a2a2a haut 20px)
    for sep_x in layout.separators {
        draw_separator(pixmap, sep_x, 12.0);
    }

    // Boutons
    for btn in &layout.buttons {
        let is_hover = mx >= btn.x && mx < btn.x + btn.w && my >= btn.y && my < btn.y + btn.h;
        if btn.is_tool {
            draw_tool_button(pixmap, btn.x, btn.y, btn.w, btn.h, btn.icon, btn.active, is_hover);
        } else {
            draw_action_button(pixmap, typo, btn.x, btn.y, btn.w, btn.h, btn.icon, btn.label, btn.active, is_hover);
            if btn.is_collab && ui.collab_active {
                // Pastille verte #10b981
                let mut dot_paint = Paint::default();
                dot_paint.set_color(Color::from_rgba8(16, 185, 129, 255));
                dot_paint.anti_alias = true;
                let mut dot_pb = PathBuilder::new();
                dot_pb.push_circle(btn.x + 18.0, btn.y + 7.0, 3.0);
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
            16.0,
            11.0,
            Color::from_rgba8(75, 75, 80, 255),
            false,
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
) -> Vec<TabButtonLayout> {
    let mut layouts = Vec::new();
    let mut tab_x = 8.0;
    let active_id = &store.project.active_board_id;

    for board in &store.project.boards {
        let is_active = &board.id == active_id;
        let (tw, _) = typo.measure_text(&board.name, 12.0, is_active);
        let tab_w = tw + 28.0;
        layouts.push(TabButtonLayout {
            board_id: board.id.clone(),
            name: board.name.clone(),
            x: tab_x,
            y: y_start,
            width: tab_w,
            height: TABS_HEIGHT,
            is_active,
            is_plus: false,
        });
        tab_x += tab_w + 4.0;
    }

    // Bouton + (créer un board)
    layouts.push(TabButtonLayout {
        board_id: String::new(),
        name: "+".into(),
        x: tab_x,
        y: y_start,
        width: 30.0,
        height: TABS_HEIGHT,
        is_active: false,
        is_plus: true,
    });

    layouts
}

fn render_board_tabs(
    pixmap: &mut PixmapMut,
    store: &Store,
    _ui: &UiState,
    typo: &Typography,
    width: f32,
    mx: f32,
    my: f32,
) {
    let y_start = TOPBAR_HEIGHT;

    // Fond #111111
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgba8(17, 17, 17, 255));
    if let Some(rect) = Rect::from_xywh(0.0, y_start, width, TABS_HEIGHT) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure inférieure #222222
    let mut border_paint = Paint::default();
    border_paint.set_color(Color::from_rgba8(34, 34, 34, 255));
    if let Some(rect) = Rect::from_xywh(0.0, y_start + TABS_HEIGHT - 1.0, width, 1.0) {
        pixmap.fill_rect(rect, &border_paint, Transform::identity(), None);
    }

    let tabs = layout_tabs(store, typo, y_start);
    for tab in tabs {
        let is_hover = mx >= tab.x && mx < tab.x + tab.width && my >= tab.y && my < tab.y + tab.height;

        if tab.is_plus {
            if is_hover {
                let mut p_paint = Paint::default();
                p_paint.set_color(Color::from_rgba8(30, 30, 30, 255));
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 5.0, 24.0, 24.0) {
                    pixmap.fill_rect(rect, &p_paint, Transform::identity(), None);
                }
            }
            draw_icon(
                pixmap,
                IconType::Plus,
                tab.x + 5.0,
                y_start + 10.0,
                Color::from_rgba8(120, 120, 120, 255),
                1.5,
            );
        } else {
            if is_hover && !tab.is_active {
                let mut h_paint = Paint::default();
                h_paint.set_color(Color::from_rgba8(26, 26, 26, 255));
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 4.0, tab.width, TABS_HEIGHT - 6.0) {
                    pixmap.fill_rect(rect, &h_paint, Transform::identity(), None);
                }
            }

            let text_color = if tab.is_active {
                Color::from_rgba8(255, 255, 255, 255)
            } else {
                Color::from_rgba8(140, 140, 140, 255)
            };

            typo.draw_text(
                pixmap,
                &tab.name,
                tab.x + 14.0,
                y_start + 10.0,
                12.0,
                text_color,
                tab.is_active,
            );

            // Ligne blanche inférieure pour l'onglet actif
            if tab.is_active {
                let mut line_paint = Paint::default();
                line_paint.set_color(Color::from_rgba8(255, 255, 255, 255));
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + TABS_HEIGHT - 2.0, tab.width, 2.0) {
                    pixmap.fill_rect(rect, &line_paint, Transform::identity(), None);
                }
            }
        }
    }
}

fn draw_separator(pixmap: &mut PixmapMut, x: f32, y: f32) -> f32 {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(42, 42, 42, 255));
    if let Some(rect) = Rect::from_xywh(x + 4.0, y, 1.0, 20.0) {
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

fn draw_tool_button(
    pixmap: &mut PixmapMut,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    icon: IconType,
    active: bool,
    hover: bool,
) {
    let bg_color = if active {
        Color::from_rgba8(45, 45, 45, 255)
    } else if hover {
        Color::from_rgba8(34, 34, 37, 255)
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(68, 68, 68, 255));
        sp.anti_alias = true;
        let stroke = Stroke { width: 1.0, ..Default::default() };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 4.0);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let icon_color = if active {
        Color::from_rgba8(255, 255, 255, 255)
    } else if hover {
        Color::from_rgba8(204, 204, 204, 255)
    } else {
        Color::from_rgba8(115, 115, 120, 255)
    };

    let icon_x = x + (w - 14.0) / 2.0;
    let icon_y = y + (h - 14.0) / 2.0;
    draw_icon(pixmap, icon, icon_x, icon_y, icon_color, 1.4);
}

fn draw_action_button(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    icon: IconType,
    label: &str,
    active: bool,
    hover: bool,
) {
    let bg_color = if active {
        Color::from_rgba8(45, 45, 45, 255)
    } else if hover {
        Color::from_rgba8(34, 34, 37, 255)
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(68, 68, 68, 255));
        sp.anti_alias = true;
        let stroke = Stroke { width: 1.0, ..Default::default() };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 4.0);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let color = if active {
        Color::from_rgba8(255, 255, 255, 255)
    } else if hover {
        Color::from_rgba8(204, 204, 204, 255)
    } else {
        Color::from_rgba8(115, 115, 120, 255)
    };

    if label.is_empty() {
        let icon_x = x + (w - 14.0) / 2.0;
        let icon_y = y + (h - 14.0) / 2.0;
        draw_icon(pixmap, icon, icon_x, icon_y, color, 1.3);
    } else {
        let icon_y = y + (h - 14.0) / 2.0;
        draw_icon(pixmap, icon, x + 8.0, icon_y, color, 1.3);
        typo.draw_text(pixmap, label, x + 26.0, y + 7.5, 12.0, color, active);
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
) -> Option<MinimapBounds> {
    let board = store.active_board()?;
    let mm_w = 180.0f32;
    let mm_h = 120.0f32;
    let mm_x = screen_w - mm_w - 16.0;
    let mm_y = screen_h - mm_h - 16.0;

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
    let vp_h = (screen_h as f64 - TOTAL_HEADER_HEIGHT as f64) / vp.scale;
    let cam_left = -vp.x / vp.scale;
    let cam_top = -vp.y / vp.scale;

    min_x = min_x.min(cam_left) - 200.0;
    min_y = min_y.min(cam_top) - 200.0;
    max_x = max_x.max(cam_left + vp_w) + 200.0;
    max_y = max_y.max(cam_top + vp_h) + 200.0;

    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);

    let scale_x = (mm_w - 12.0) / span_x as f32;
    let scale_y = (mm_h - 12.0) / span_y as f32;
    let scale = scale_x.min(scale_y);

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
        scale,
        cam_left,
        cam_top,
        vp_w,
        vp_h,
    })
}

fn render_minimap(pixmap: &mut PixmapMut, store: &Store, w: f32, h: f32) {
    let mb = match layout_minimap(store, w, h) {
        Some(m) => m,
        None => return,
    };

    let board = match store.active_board() {
        Some(b) => b,
        None => return,
    };

    // Fond #16181D (85%)
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgba8(22, 24, 29, 220));
    if let Some(rect) = Rect::from_xywh(mb.mm_x, mb.mm_y, mb.mm_w, mb.mm_h) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure #262B35
    let mut border_paint = Paint::default();
    border_paint.set_color(Color::from_rgba8(38, 43, 53, 255));
    let stroke = Stroke { width: 1.0, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.move_to(mb.mm_x, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y + mb.mm_h);
    pb.line_to(mb.mm_x, mb.mm_y + mb.mm_h);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // Dessine miniatures images
    let mut item_paint = Paint::default();
    item_paint.set_color(Color::from_rgba8(90, 100, 120, 180));
    for img in &board.images {
        let ix = mb.mm_x + 6.0 + ((img.x - img.width / 2.0 - mb.min_x) as f32 * mb.scale);
        let iy = mb.mm_y + 6.0 + ((img.y - img.height / 2.0 - mb.min_y) as f32 * mb.scale);
        let iw = (img.width as f32 * mb.scale).max(2.0);
        let ih = (img.height as f32 * mb.scale).max(2.0);
        if let Some(r) = Rect::from_xywh(ix, iy, iw, ih) {
            pixmap.fill_rect(r, &item_paint, Transform::identity(), None);
        }
    }

    // Dessine miniatures annotations & stickies & membranes
    let mut ann_paint = Paint::default();
    ann_paint.set_color(Color::from_rgba8(160, 150, 90, 160));
    let mut membrane_paint = Paint::default();
    membrane_paint.set_color(Color::from_rgba8(80, 140, 220, 100));

    for ann in &board.annotations {
        match ann {
            Annotation::Text { x, y, width, height, .. } => {
                let aw = width.unwrap_or(200.0) as f32 * mb.scale;
                let ah = height.unwrap_or(48.0) as f32 * mb.scale;
                let ax = mb.mm_x + 6.0 + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + 6.0 + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(2.0), ah.max(2.0)) {
                    pixmap.fill_rect(r, &ann_paint, Transform::identity(), None);
                }
            }
            Annotation::Sticky { x, y, width, height, .. } => {
                let aw = width.unwrap_or(160.0) as f32 * mb.scale;
                let ah = height.unwrap_or(120.0) as f32 * mb.scale;
                let ax = mb.mm_x + 6.0 + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + 6.0 + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(2.0), ah.max(2.0)) {
                    pixmap.fill_rect(r, &ann_paint, Transform::identity(), None);
                }
            }
            Annotation::Membrane { x, y, width, height, .. } => {
                let aw = *width as f32 * mb.scale;
                let ah = *height as f32 * mb.scale;
                let ax = mb.mm_x + 6.0 + ((*x - mb.min_x) as f32 * mb.scale);
                let ay = mb.mm_y + 6.0 + ((*y - mb.min_y) as f32 * mb.scale);
                if let Some(r) = Rect::from_xywh(ax, ay, aw.max(4.0), ah.max(4.0)) {
                    pixmap.fill_rect(r, &membrane_paint, Transform::identity(), None);
                }
            }
            _ => {}
        }
    }

    // Rectangle de la caméra
    let cx = mb.mm_x + 6.0 + ((mb.cam_left - mb.min_x) as f32 * mb.scale);
    let cy = mb.mm_y + 6.0 + ((mb.cam_top - mb.min_y) as f32 * mb.scale);
    let cw = (mb.vp_w as f32 * mb.scale).max(4.0);
    let ch = (mb.vp_h as f32 * mb.scale).max(4.0);

    let mut cam_paint = Paint::default();
    cam_paint.set_color(Color::from_rgba8(255, 255, 255, 220));
    let cam_stroke = Stroke { width: 1.5, ..Default::default() };
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

fn render_toast(pixmap: &mut PixmapMut, toast: &Toast, typo: &Typography, w: f32, h: f32) {
    let alpha = toast.alpha();
    if alpha <= 0.01 {
        return;
    }

    let (tw, _) = typo.measure_text(&toast.message, 13.0, false);
    let toast_w = tw + 40.0;
    let toast_h = 36.0;
    let toast_x = (w - toast_w) / 2.0;
    let toast_y = h - 64.0;

    // Fond pilule sombre avec opacité animée
    let mut bg_paint = Paint::default();
    let a_byte = (alpha * 230.0) as u8;
    bg_paint.set_color(Color::from_rgba8(20, 20, 24, a_byte));

    let mut pb = PathBuilder::new();
    let r = 18.0;
    pb.move_to(toast_x + r, toast_y);
    pb.line_to(toast_x + toast_w - r, toast_y);
    pb.quad_to(toast_x + toast_w, toast_y, toast_x + toast_w, toast_y + r);
    pb.line_to(toast_x + toast_w, toast_y + toast_h - r);
    pb.quad_to(toast_x + toast_w, toast_y + toast_h, toast_x + toast_w - r, toast_y + toast_h);
    pb.line_to(toast_x + r, toast_y + toast_h);
    pb.quad_to(toast_x, toast_y + toast_h, toast_x, toast_y + toast_h - r);
    pb.line_to(toast_x, toast_y + r);
    pb.quad_to(toast_x, toast_y, toast_x + r, toast_y);
    // Bordure subtile #333333
    let mut border_paint = Paint::default();
    let b_byte = (alpha * 120.0) as u8;
    border_paint.set_color(Color::from_rgba8(60, 60, 65, b_byte));
    let stroke = Stroke { width: 1.0, ..Default::default() };

    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // Texte centré
    let text_color = Color::from_rgba8(240, 240, 240, (alpha * 255.0) as u8);
    typo.draw_text(
        pixmap,
        &toast.message,
        toast_x + 20.0,
        toast_y + 11.0,
        13.0,
        text_color,
        false,
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
    if y < TOPBAR_HEIGHT {
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
    } else if y >= TOPBAR_HEIGHT && y < TOTAL_HEADER_HEIGHT {
        // Clic sur la BoardTabs bar via layout_tabs unifié
        let tabs = layout_tabs(store, typo, TOPBAR_HEIGHT);
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
        if let Some(mb) = layout_minimap(store, screen_w, screen_h) {
            if x >= mb.mm_x && x <= mb.mm_x + mb.mm_w && y >= mb.mm_y && y <= mb.mm_y + mb.mm_h {
                let rel_x = ((x - mb.mm_x - 6.0) / (mb.mm_w - 12.0).max(1.0) as f32).clamp(0.0, 1.0) as f64;
                let rel_y = ((y - mb.mm_y - 6.0) / (mb.mm_h - 12.0).max(1.0) as f32).clamp(0.0, 1.0) as f64;
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
}
