//! Le dessin du dock : la passe, le cadre d'un panneau, et l'aiguillage vers son contenu.
//!
//! Ce module ne connaît ni les ancrages ni le glissement — il reçoit une géométrie déjà
//! calculée par [`super::compute_panel_layouts`] et la met en pixels. C'est la frontière que
//! la loi L4 demande : **une seule géométrie**, lue ici et par le clic, jamais recalculée.

use super::paint::Brush;
use super::{
    cache, compute_panel_layouts, domains, organize, plugins, pomodoro, preset, storyboard,
    DockCache, DockManager, PanelLayoutBox, TabId, WidgetRect,
};
use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

/// Ce que le rendu du dock a besoin de savoir, en dehors du document lui-même.
///
/// Ces cinq valeurs voyageaient une à une, et la liste avait fini par dépasser ce qu'une
/// signature supporte. Elles vont ensemble : c'est l'état de la fenêtre au moment de
/// l'image — sa typographie, son thème, son cadre, sa souris, et les tampons qu'on garde
/// d'une image à l'autre.
pub struct DockPass<'a> {
    pub typo: &'a Typography,
    pub theme: &'a Theme,
    pub screen: ScreenFrame,
    pub pointer: Pointer,
    /// Les tampons des panneaux, ou `None` pour dessiner directement (DOCK-CACHE-1).
    ///
    /// Le rendu direct n'est pas un vestige : c'est lui qui sert de référence aux tests, qui
    /// vérifient que le cache rend la même image.
    pub cache: Option<&'a DockCache>,
}

/// Le rendu du dock : ses panneaux ouverts, dans l'ordre de leurs ancrages.
pub fn render_docks(
    pixmap: &mut PixmapMut,
    dock: &DockManager,
    store: &Store,
    pass: &DockPass<'_>,
) {
    let s = crate::theme::clamp_ui_scale(pass.screen.scale);
    let layouts = compute_panel_layouts(
        dock,
        pass.screen.width,
        pass.screen.height,
        pass.screen.header_h,
        s,
    );
    for panel in layouts {
        match pass.cache {
            Some(cache) => cache::draw_panel_cached(pixmap, cache, dock, store, pass, &panel, s),
            None => {
                let brush = Brush {
                    typo: pass.typo,
                    theme: pass.theme,
                    s,
                    pointer: pass.pointer,
                    origin: (0.0, 0.0),
                };
                draw_panel(pixmap, &brush, dock, store, &panel, s);
            }
        }
    }
}

/// Dessine un panneau — son cadre puis son contenu — là où le pinceau vise.
pub(super) fn draw_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush<'_>,
    dock: &DockManager,
    store: &Store,
    panel: &PanelLayoutBox,
    s: f32,
) {
    let seen = panel.seen();
    draw_frame(pixmap, brush, panel, seen);
    let frame = ScaledRect {
        x: seen.x,
        y: seen.y,
        w: seen.w,
        h: seen.h,
        scale: s,
    };
    draw_content(pixmap, brush, dock, store, panel.tab, frame);
}

/// L'ombre, le fond, la bordure et la poignée — ce que tout panneau a en commun.
fn draw_frame(pixmap: &mut PixmapMut, brush: &Brush, panel: &PanelLayoutBox, seen: WidgetRect) {
    let theme = brush.theme;
    let shadow = panel.shadow(brush.s);
    let shadow_color = if panel.is_dragged {
        theme.panel_shadow_dragged
    } else {
        theme.panel_shadow
    };
    brush.fill(pixmap, shadow, brush.px(8.0), shadow_color);
    brush.fill(pixmap, seen, brush.px(6.0), theme.bg_panel);
    let border = if panel.is_dragged {
        theme.border_accent
    } else {
        theme.border_subtle
    };
    brush.stroke(pixmap, seen, brush.px(6.0), border, brush.px(1.0));

    let grip_center = (
        seen.x + seen.w / 2.0,
        panel.grip_y + panel.visual_offset_y + panel.grip_height / 2.0,
    );
    let active = panel.is_dragged || panel.grip_contains_point(brush.pointer.x, brush.pointer.y);
    brush.grip(pixmap, grip_center, active);
}

fn draw_content(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    dock: &DockManager,
    store: &Store,
    tab: TabId,
    frame: ScaledRect,
) {
    match tab {
        TabId::Organize => {
            organize::paint::render_organize_panel(pixmap, brush, frame, &dock.organize, store);
        }
        TabId::Pomodoro => {
            pomodoro::paint::render_pomodoro_panel(pixmap, brush, frame, &dock.pomodoro);
        }
        TabId::Storyboard => {
            storyboard::paint::render_storyboard_panel(pixmap, brush, frame, &dock.storyboard);
        }
        TabId::Plugins => {
            plugins::paint::render_plugins_panel(pixmap, brush, frame, &dock.plugins);
        }
        TabId::Preset => preset::paint::render_presets_panel(pixmap, brush, frame),
        TabId::Domains => {
            domains::paint::render_domains_panel(pixmap, store, &dock.domains, brush, frame)
        }
    }
}

// ── Le clic ─────────────────────────────────────────────────────────────────
