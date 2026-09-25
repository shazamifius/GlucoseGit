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
                let brush = Brush::nouveau((pass.typo, pass.theme), s, pass.pointer, (0.0, 0.0));
                draw_panel(pixmap, &brush, dock, store, &panel, s);
            }
        }
    }
}

/// La profondeur du voile du liseré, en points (fiche 10 § 5.7 : `inset 0 0 60px`).
const VOILE: f32 = 60.0;
/// L'épaisseur de son trait, en points (`3px solid`).
const TRAIT: f32 = 3.0;

/// **Le liseré du passé tel que la carte le peint** (voir [`crate::present::lisere_gpu`]) : les
/// mêmes nombres que [`lisere_du_passe`], lus au même endroit.
pub fn lisere(theme: &Theme, s: f32) -> crate::present::lisere_gpu::Lisere {
    let rgba = |c: tiny_skia::Color| [c.red(), c.green(), c.blue(), c.alpha()];
    crate::present::lisere_gpu::Lisere {
        voile: VOILE * s,
        trait_: TRAIT * s,
        teinte_du_voile: rgba(theme.temps.voile),
        ambre: rgba(theme.temps.ambre),
    }
}

/// **Le liseré du passé** : quand la Time Machine montre un état passé, un trait ambre borde
/// la fenêtre et un voile ambré glisse vers l'intérieur (fiche 10 § 5.7 : `3px solid`,
/// `inset 0 0 60px`). On ne peut pas oublier qu'on ne regarde pas le présent.
///
/// Le voile est un dégradé linéaire par bord, de la teinte au transparent : aucun nombre de
/// pas à choisir.
///
/// Sur la voie processeur seulement : sur la voie graphique, la carte le peint
/// ([`crate::present::lisere_gpu`]) — au processeur, il coûtait 11,7 ms par image et faisait
/// repartir toute la couche du dessus.
pub fn lisere_du_passe(pixmap: &mut PixmapMut, pass: &DockPass<'_>, s: f32) {
    use tiny_skia::{
        Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Point, Rect, SpreadMode,
        Transform,
    };
    let (w, h) = (pass.screen.width, pass.screen.height);
    let theme = pass.theme;
    let voile = VOILE * s;
    let transparent = Color::from_rgba(
        theme.temps.voile.red(),
        theme.temps.voile.green(),
        theme.temps.voile.blue(),
        0.0,
    )
    .unwrap_or(Color::TRANSPARENT);
    // (rectangle, départ du dégradé, arrivée) pour chaque bord.
    let bords = [
        ((0.0, 0.0, w, voile), (0.0, 0.0), (0.0, voile)),
        ((0.0, h - voile, w, voile), (0.0, h), (0.0, h - voile)),
        ((0.0, 0.0, voile, h), (0.0, 0.0), (voile, 0.0)),
        ((w - voile, 0.0, voile, h), (w, 0.0), (w - voile, 0.0)),
    ];
    for ((x, y, bw, bh), de, a) in bords {
        let (Some(rect), Some(degrade)) = (
            Rect::from_xywh(x, y, bw, bh),
            LinearGradient::new(
                Point::from_xy(de.0, de.1),
                Point::from_xy(a.0, a.1),
                vec![
                    GradientStop::new(0.0, theme.temps.voile),
                    GradientStop::new(1.0, transparent),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ),
        ) else {
            continue;
        };
        let paint = Paint {
            shader: degrade,
            anti_alias: false,
            ..Paint::default()
        };
        let chemin = PathBuilder::from_rect(rect);
        pixmap.fill_path(
            &chemin,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    let brush = Brush::nouveau((pass.typo, theme), s, pass.pointer, (0.0, 0.0));
    let trait_ = TRAIT * s;
    let cadre = WidgetRect::new(trait_ / 2.0, trait_ / 2.0, w - trait_, h - trait_);
    brush.stroke(pixmap, cadre, 0.0, theme.temps.ambre, trait_);
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
    let active = panel.is_dragged || brush.hovered(panel.grip_rect());
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
        TabId::Temps => super::temps::paint::render_temps_panel(pixmap, brush, frame, &dock.temps),
    }
}

// ── Le clic ─────────────────────────────────────────────────────────────────
