//! Composants graphiques d'interface de Glucose (TopBar, BoardTabs, Minimap, Toasts).

use crate::icons::{draw_icon, IconType};
use crate::typography::Typography;
use glucose_core::store::Store;
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

#[derive(Debug, Clone)]
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

fn render_topbar(
    pixmap: &mut PixmapMut,
    _store: &Store,
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
        14.0,
        13.0,
        15.0,
        Color::from_rgba8(255, 255, 255, 255),
        true,
    );

    let mut cur_x = 108.0;

    // Groupe Outils de base
    let tools = [
        (ActiveTool::Select, IconType::Select, "V"),
        (ActiveTool::Pan, IconType::Pan, "Espace"),
    ];
    for (t, icon, _title) in tools {
        let is_active = ui.active_tool == t;
        let is_hovered = mx >= cur_x && mx < cur_x + 30.0 && my >= 7.0 && my < 37.0;
        draw_tool_button(pixmap, cur_x, 7.0, 30.0, 30.0, icon, is_active, is_hovered);
        cur_x += 32.0;
    }

    cur_x += draw_separator(pixmap, cur_x, 12.0);

    // Groupe Annotations
    let ann_tools = [
        (ActiveTool::Text, IconType::Text, "T"),
        (ActiveTool::Sticky, IconType::Sticky, "N"),
        (ActiveTool::Arrow, IconType::Arrow, "A"),
        (ActiveTool::Folder, IconType::Folder, "F"),
        (ActiveTool::Membrane, IconType::Membrane, "M"),
    ];
    for (t, icon, _title) in ann_tools {
        let is_active = ui.active_tool == t;
        let is_hovered = mx >= cur_x && mx < cur_x + 30.0 && my >= 7.0 && my < 37.0;
        draw_tool_button(pixmap, cur_x, 7.0, 30.0, 30.0, icon, is_active, is_hovered);
        cur_x += 32.0;
    }

    cur_x += draw_separator(pixmap, cur_x, 12.0);

    // Bouton + Images
    let img_hover = mx >= cur_x && mx < cur_x + 85.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        cur_x,
        7.0,
        85.0,
        30.0,
        IconType::Plus,
        "Images",
        false,
        img_hover,
    );
    cur_x += 92.0;

    cur_x += draw_separator(pixmap, cur_x, 12.0);

    // Boutons de navigation & organisation
    let actions = [
        ("Ordonner", IconType::Organize, false),
        ("Timer", IconType::Timer, false),
        ("Storyboard", IconType::Storyboard, false),
    ];
    for (label, icon, active) in actions {
        let (tw, _) = typo.measure_text(label, 12.0, false);
        let btn_w = tw + 32.0;
        let is_hover = mx >= cur_x && mx < cur_x + btn_w && my >= 7.0 && my < 37.0;
        draw_action_button(pixmap, typo, cur_x, 7.0, btn_w, 30.0, icon, label, active, is_hover);
        cur_x += btn_w + 6.0;
    }

    cur_x += draw_separator(pixmap, cur_x, 12.0);

    // Toggle Aimant SNAP-1
    let aimant_hover = mx >= cur_x && mx < cur_x + 80.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        cur_x,
        7.0,
        80.0,
        30.0,
        IconType::Magnet,
        "Aimant",
        ui.smart_align,
        aimant_hover,
    );
    cur_x += 86.0;

    // Toggle Trans-domaines
    let td_hover = mx >= cur_x && mx < cur_x + 120.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        cur_x,
        7.0,
        120.0,
        30.0,
        IconType::TransDomain,
        "Trans-domaines",
        ui.trans_domain,
        td_hover,
    );

    // Côté droit
    let mut right_x = width - 14.0;

    // Domaines
    right_x -= 88.0;
    let dom_hover = mx >= right_x && mx < right_x + 88.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        right_x,
        7.0,
        88.0,
        30.0,
        IconType::Domains,
        "Domaines",
        false,
        dom_hover,
    );

    // Preset
    right_x -= 78.0;
    let pre_hover = mx >= right_x && mx < right_x + 78.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        right_x,
        7.0,
        78.0,
        30.0,
        IconType::Preset,
        "Preset",
        false,
        pre_hover,
    );

    // Plugins
    right_x -= 82.0;
    let plu_hover = mx >= right_x && mx < right_x + 82.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        right_x,
        7.0,
        82.0,
        30.0,
        IconType::Plugins,
        "Plugins",
        false,
        plu_hover,
    );

    right_x -= 12.0;
    draw_separator(pixmap, right_x, 12.0);

    // Exporter
    right_x -= 90.0;
    let exp_hover = mx >= right_x && mx < right_x + 90.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        right_x,
        7.0,
        90.0,
        30.0,
        IconType::Export,
        "Exporter",
        false,
        exp_hover,
    );

    right_x -= 12.0;
    draw_separator(pixmap, right_x, 12.0);

    // Collaborer
    right_x -= 100.0;
    let col_hover = mx >= right_x && mx < right_x + 100.0 && my >= 7.0 && my < 37.0;
    draw_action_button(
        pixmap,
        typo,
        right_x,
        7.0,
        100.0,
        30.0,
        IconType::Collab,
        "Collaborer",
        ui.collab_active,
        col_hover,
    );
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

    let mut tab_x = 8.0;
    let active_id = store.project.active_board_id.clone();

    for board in &store.project.boards {
        let is_active = board.id == active_id;
        let (tw, _) = typo.measure_text(&board.name, 12.0, is_active);
        let tab_w = tw + 28.0;

        let is_hover = mx >= tab_x && mx < tab_x + tab_w && my >= y_start && my < y_start + TABS_HEIGHT;

        if is_hover && !is_active {
            let mut h_paint = Paint::default();
            h_paint.set_color(Color::from_rgba8(26, 26, 26, 255));
            if let Some(rect) = Rect::from_xywh(tab_x, y_start + 4.0, tab_w, TABS_HEIGHT - 6.0) {
                pixmap.fill_rect(rect, &h_paint, Transform::identity(), None);
            }
        }

        let text_color = if is_active {
            Color::from_rgba8(255, 255, 255, 255)
        } else {
            Color::from_rgba8(140, 140, 140, 255)
        };

        typo.draw_text(
            pixmap,
            &board.name,
            tab_x + 14.0,
            y_start + 10.0,
            12.0,
            text_color,
            is_active,
        );

        // Ligne blanche inférieure pour l'onglet actif
        if is_active {
            let mut line_paint = Paint::default();
            line_paint.set_color(Color::from_rgba8(255, 255, 255, 255));
            if let Some(rect) = Rect::from_xywh(tab_x, y_start + TABS_HEIGHT - 2.0, tab_w, 2.0) {
                pixmap.fill_rect(rect, &line_paint, Transform::identity(), None);
            }
        }

        tab_x += tab_w + 4.0;
    }

    // Bouton + (créer un board)
    let is_plus_hover = mx >= tab_x && mx < tab_x + 28.0 && my >= y_start + 4.0 && my < y_start + 30.0;
    if is_plus_hover {
        let mut p_paint = Paint::default();
        p_paint.set_color(Color::from_rgba8(30, 30, 30, 255));
        if let Some(rect) = Rect::from_xywh(tab_x, y_start + 5.0, 24.0, 24.0) {
            pixmap.fill_rect(rect, &p_paint, Transform::identity(), None);
        }
    }
    draw_icon(
        pixmap,
        IconType::Plus,
        tab_x + 5.0,
        y_start + 10.0,
        Color::from_rgba8(120, 120, 120, 255),
        1.5,
    );
}

fn draw_separator(pixmap: &mut PixmapMut, x: f32, y: f32) -> f32 {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(42, 42, 42, 255));
    if let Some(rect) = Rect::from_xywh(x + 4.0, y, 1.0, 20.0) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
    9.0
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
        Color::from_rgba8(32, 32, 32, 255)
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, &p, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(68, 68, 68, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let icon_color = if active {
        Color::from_rgba8(255, 255, 255, 255)
    } else if hover {
        Color::from_rgba8(200, 200, 200, 255)
    } else {
        Color::from_rgba8(115, 115, 115, 255)
    };

    draw_icon(pixmap, icon, x + 8.0, y + 8.0, icon_color, 1.4);
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
        Color::from_rgba8(30, 30, 30, 255)
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, &p, Transform::identity(), None);
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(Color::from_rgba8(68, 68, 68, 255));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let color = if active {
        Color::from_rgba8(240, 240, 240, 255)
    } else if hover {
        Color::from_rgba8(210, 210, 210, 255)
    } else {
        Color::from_rgba8(130, 130, 130, 255)
    };

    draw_icon(pixmap, icon, x + 8.0, y + 8.0, color, 1.3);
    typo.draw_text(pixmap, label, x + 26.0, y + 9.0, 12.0, color, active);
}

fn render_minimap(pixmap: &mut PixmapMut, store: &Store, w: f32, h: f32) {
    let mm_w = 180.0;
    let mm_h = 120.0;
    let mm_x = w - mm_w - 16.0;
    let mm_y = h - mm_h - 16.0;

    // Fond #16181D (85%)
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgba8(22, 24, 29, 220));
    if let Some(rect) = Rect::from_xywh(mm_x, mm_y, mm_w, mm_h) {
        pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
    }

    // Bordure #262B35
    let mut border_paint = Paint::default();
    border_paint.set_color(Color::from_rgba8(38, 43, 53, 255));
    let stroke = Stroke { width: 1.0, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.move_to(mm_x, mm_y);
    pb.line_to(mm_x + mm_w, mm_y);
    pb.line_to(mm_x + mm_w, mm_y + mm_h);
    pb.line_to(mm_x, mm_y + mm_h);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    let board = match store.active_board() {
        Some(b) => b,
        None => return,
    };

    // Calcul de l'étendue des éléments du board
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

    // Inclure la vue caméra
    let vp = &board.viewport;
    let vp_w = w as f64 / vp.scale;
    let vp_h = (h as f64 - TOTAL_HEADER_HEIGHT as f64) / vp.scale;
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

    // Dessine miniatures images
    let mut item_paint = Paint::default();
    item_paint.set_color(Color::from_rgba8(90, 100, 120, 180));
    for img in &board.images {
        let ix = mm_x + 6.0 + ((img.x - img.width / 2.0 - min_x) as f32 * scale);
        let iy = mm_y + 6.0 + ((img.y - img.height / 2.0 - min_y) as f32 * scale);
        let iw = (img.width as f32 * scale).max(2.0);
        let ih = (img.height as f32 * scale).max(2.0);
        if let Some(r) = Rect::from_xywh(ix, iy, iw, ih) {
            pixmap.fill_rect(r, &item_paint, Transform::identity(), None);
        }
    }

    // Rectangle de la caméra
    let cx = mm_x + 6.0 + ((cam_left - min_x) as f32 * scale);
    let cy = mm_y + 6.0 + ((cam_top - min_y) as f32 * scale);
    let cw = (vp_w as f32 * scale).max(4.0);
    let ch = (vp_h as f32 * scale).max(4.0);

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
    pb.close();

    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);

        let mut border_paint = Paint::default();
        border_paint.set_color(Color::from_rgba8(60, 60, 75, (alpha * 180.0) as u8));
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    typo.draw_text(
        pixmap,
        &toast.message,
        toast_x + 20.0,
        toast_y + 11.0,
        13.0,
        Color::from_rgba8(240, 240, 245, (alpha * 255.0) as u8),
        false,
    );
}

/// Détecte si un clic souris se situe sur l'interface et retourne l'action associée
pub fn handle_ui_click(
    x: f32,
    y: f32,
    screen_w: f32,
    _screen_h: f32,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
) -> Option<UiAction> {
    if y < TOPBAR_HEIGHT {
        // Clic sur la TopBar
        let mut cur_x = 108.0;

        // Outils
        if x >= cur_x && x < cur_x + 30.0 {
            ui.active_tool = ActiveTool::Select;
            return Some(UiAction::SelectTool(ActiveTool::Select));
        }
        cur_x += 32.0;
        if x >= cur_x && x < cur_x + 30.0 {
            ui.active_tool = ActiveTool::Pan;
            return Some(UiAction::SelectTool(ActiveTool::Pan));
        }
        cur_x += 32.0 + 9.0;

        // Annotations
        let ann_tools = [
            ActiveTool::Text,
            ActiveTool::Sticky,
            ActiveTool::Arrow,
            ActiveTool::Folder,
            ActiveTool::Membrane,
        ];
        for t in ann_tools {
            if x >= cur_x && x < cur_x + 30.0 {
                ui.active_tool = t;
                return Some(UiAction::SelectTool(t));
            }
            cur_x += 32.0;
        }

        cur_x += 9.0;

        // + Images
        if x >= cur_x && x < cur_x + 85.0 {
            return Some(UiAction::AddImages);
        }
        cur_x += 92.0 + 9.0;

        // Ordonner, Timer, Storyboard
        let actions = [
            ("Ordonner", UiAction::Organize),
            ("Timer", UiAction::ToggleTimer),
            ("Storyboard", UiAction::ToggleStoryboard),
        ];
        for (label, act) in actions {
            let (tw, _) = typo.measure_text(label, 12.0, false);
            let btn_w = tw + 32.0;
            if x >= cur_x && x < cur_x + btn_w {
                return Some(act);
            }
            cur_x += btn_w + 6.0;
        }

        cur_x += 9.0;

        // Aimant SNAP-1
        if x >= cur_x && x < cur_x + 80.0 {
            ui.smart_align = !ui.smart_align;
            ui.show_toast(if ui.smart_align { "✨ Aimant activé" } else { "Aimant désactivé" });
            return Some(UiAction::ToggleMagnet);
        }
        cur_x += 86.0;

        // Trans-domaines
        if x >= cur_x && x < cur_x + 120.0 {
            ui.trans_domain = !ui.trans_domain;
            return Some(UiAction::ToggleTransDomain);
        }

        // Côté droit
        let mut right_x = screen_w - 14.0;
        right_x -= 88.0;
        if x >= right_x && x < right_x + 88.0 {
            return Some(UiAction::ToggleDomains);
        }
        right_x -= 78.0;
        if x >= right_x && x < right_x + 78.0 {
            return Some(UiAction::TogglePreset);
        }
        right_x -= 82.0;
        if x >= right_x && x < right_x + 82.0 {
            return Some(UiAction::TogglePlugins);
        }
        right_x -= 12.0;
        right_x -= 90.0;
        if x >= right_x && x < right_x + 90.0 {
            return Some(UiAction::ExportMenu);
        }
        right_x -= 12.0;
        right_x -= 100.0;
        if x >= right_x && x < right_x + 100.0 {
            ui.collab_active = !ui.collab_active;
            ui.show_toast(if ui.collab_active { "🌐 Collaboration connectée" } else { "Collaboration déconnectée" });
            return Some(UiAction::ToggleCollab);
        }
    } else if y >= TOPBAR_HEIGHT && y < TOTAL_HEADER_HEIGHT {
        // Clic sur la BoardTabs bar
        let mut tab_x = 8.0;
        for board in &store.project.boards {
            let (tw, _) = typo.measure_text(&board.name, 12.0, false);
            let tab_w = tw + 28.0;
            if x >= tab_x && x < tab_x + tab_w {
                return Some(UiAction::SelectBoard(board.id.clone()));
            }
            tab_x += tab_w + 4.0;
        }

        // Clic sur le bouton +
        if x >= tab_x && x < tab_x + 30.0 {
            return Some(UiAction::AddBoard);
        }
    } else {
        // Clic sur la Minimap (en bas à droite)
        let mm_w = 180.0;
        let mm_h = 120.0;
        let mm_x = screen_w - mm_w - 16.0;
        let mm_y = _screen_h - mm_h - 16.0;

        if x >= mm_x && x <= mm_x + mm_w && y >= mm_y && y <= mm_y + mm_h {
            let rel_x = (x - mm_x) / mm_w;
            let rel_y = (y - mm_y) / mm_h;
            let target_wx = (rel_x as f64 - 0.5) * 2000.0;
            let target_wy = (rel_y as f64 - 0.5) * 2000.0;
            return Some(UiAction::MinimapPan(target_wx, target_wy));
        }
    }

    None
}
