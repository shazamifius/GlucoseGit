//! Les lois du dock : ses ancrages, son glissement, et — pour chaque panneau — le fait que
//! **le clic tombe là où le dessin le montre** (loi L4).

use super::*;
use crate::typography::Typography;
use glucose_core::store::Store;
use std::time::Instant;

const SCREEN: ScreenFrame = ScreenFrame {
    width: 1440.0,
    height: 900.0,
    header_h: 78.0,
    scale: 1.0,
};

fn screen_at(scale: f32) -> ScreenFrame {
    ScreenFrame {
        header_h: SCREEN.header_h * scale,
        scale,
        ..SCREEN
    }
}

/// Le cadre d'un panneau ouvert seul dans son dock, à une échelle donnée.
fn frame_of(dock: &DockManager, tab: TabId, screen: ScreenFrame) -> ScaledRect {
    let s = crate::theme::clamp_ui_scale(screen.scale);
    let panel = compute_panel_layouts(dock, screen.width, screen.height, screen.header_h, s)
        .into_iter()
        .find(|b| b.tab == tab)
        .expect("le panneau est ouvert");
    ScaledRect {
        x: panel.x,
        y: panel.y,
        w: panel.width,
        h: panel.height,
        scale: s,
    }
}

fn center(rect: WidgetRect) -> Pointer {
    Pointer {
        x: rect.x + rect.w / 2.0,
        y: rect.y + rect.h / 2.0,
    }
}

fn dock_with(tab: TabId) -> DockManager {
    let mut dock = DockManager::new();
    dock.top_tabs.clear();
    dock.bottom_tabs.clear();
    dock.tabs_mut(tab.anchor()).push(tab);
    dock
}

/// Fiche 10 § 4 — deux docks : Domaines, Presets, Plugins descendent du haut ; Ordonner,
/// Storyboard, Pomodoro montent du bas. Largeurs 320 / 280 / 340 et 250 / 260 / ≥ 160.
/// Fermeture au-delà de 80 px de glissement vers la sortie.
#[test]
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

/// La poignée est du côté de la sortie : en bas pour un panneau du haut, en haut pour un
/// panneau du bas. C'est ce qui rend le geste de fermeture naturel — on tire vers le bord.
#[test]
fn test_the_grip_is_on_the_side_a_panel_leaves_by() {
    let mut dock = DockManager::new();
    dock.top_tabs = vec![TabId::Domains];
    dock.bottom_tabs = vec![TabId::Organize];
    for panel in compute_panel_layouts(&dock, 1440.0, 900.0, 78.0, 1.0) {
        let expected = match panel.tab.anchor() {
            DockAnchor::TopLeft => panel.y + panel.height - panel.grip_height,
            DockAnchor::BottomLeft => panel.y,
        };
        assert_eq!(panel.grip_y, expected, "{:?}", panel.tab);
    }
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

    assert!(dock.update_drag(50.0, 500.0), "le panneau change de place");
    assert_eq!(dock.bottom_tabs, vec![TabId::Pomodoro, TabId::Organize]);

    // 79 px vers le bas : le panneau reste ; 81 px : il se ferme (fiche 10 § 4, 80 px).
    dock.update_drag(50.0, 579.0);
    assert_eq!(dock.finish_drag(), None, "sous le seuil, le panneau reste");
    assert!(dock.is_open(TabId::Pomodoro));

    dock.drag = Some(DragSession {
        tab: TabId::Pomodoro,
        start_x: 300.0,
        start_y: 500.0,
        current_x: 300.0,
        current_y: 500.0,
    });
    dock.update_drag(300.0, 581.0);
    assert_eq!(dock.finish_drag(), Some(TabId::Pomodoro));
    assert!(!dock.is_open(TabId::Pomodoro));
}

// ── ORDONNER ────────────────────────────────────────────────────────────────

/// **R-37, l'origine de ce fichier.** Le clic d'un bouton de tri tombe où il est dessiné,
/// y compris sur « Sombre → Clair » dont le libellé porte un « → » de trois octets : la
/// géométrie du clic comptait les octets, celle du dessin mesurait la police, et les deux
/// divergeaient de 26 px par bouton. À 100 % comme à 150 %.
#[test]
fn test_l4_a_sort_button_is_clicked_where_it_is_drawn() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    for (scale, sort) in [(1.0, SortType::SizeAsc), (1.5, SortType::RatioLand)] {
        let mut dock = dock_with(TabId::Organize);
        let screen = screen_at(scale);
        let frame = frame_of(&dock, TabId::Organize, screen);
        let layout = organize::layout_organize_panel(frame, &typo);
        let button = layout
            .sort_buttons
            .iter()
            .find(|b| b.sort_type == sort)
            .expect("le tri est dans le panneau");

        let result = handle_dock_click(&mut dock, &store, &typo, screen, center(button.rect));
        assert_eq!(result, Some(PanelClickResult::Handled));
        assert_eq!(dock.organize.sort_by, sort, "à l'échelle {scale}");
    }
}

/// Un tri que le noyau ne sait pas faire ne se choisit pas : le bouton est inerte, et l'état
/// ne bouge pas. Il s'allumait, puis la disposition retombait en silence sur l'ordre courant.
#[test]
fn test_an_unimplemented_sort_cannot_be_chosen() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Organize);
    let frame = frame_of(&dock, TabId::Organize, SCREEN);
    let layout = organize::layout_organize_panel(frame, &typo);

    for sort in [SortType::Color, SortType::LumAsc, SortType::LumDesc] {
        assert!(!sort.implemented(), "{sort:?} n'est pas encore faisable");
        let button = layout
            .sort_buttons
            .iter()
            .find(|b| b.sort_type == sort)
            .expect("le tri est dans le panneau");
        handle_dock_click(&mut dock, &store, &typo, SCREEN, center(button.rect));
        assert_eq!(dock.organize.sort_by, SortType::None, "{sort:?}");
    }
    assert!(!LayoutMode::BySlot.implemented());
    let by_slot = layout
        .layout_modes
        .iter()
        .find(|m| m.mode == LayoutMode::BySlot)
        .expect("la disposition est dans le panneau");
    handle_dock_click(&mut dock, &store, &typo, SCREEN, center(by_slot.rect));
    assert_eq!(dock.organize.layout, LayoutMode::Compact);
}

/// `Appliquer` demande la disposition ; les autres clics du panneau ne la demandent pas.
#[test]
fn test_apply_asks_for_a_layout_and_nothing_else_does() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Organize);
    let frame = frame_of(&dock, TabId::Organize, SCREEN);
    let layout = organize::layout_organize_panel(frame, &typo);

    let grid = layout
        .layout_modes
        .iter()
        .find(|m| m.mode == LayoutMode::Grid)
        .expect("la grille est dans le panneau");
    let result = handle_dock_click(&mut dock, &store, &typo, SCREEN, center(grid.rect));
    assert_eq!(result, Some(PanelClickResult::Handled));
    assert_eq!(dock.organize.layout, LayoutMode::Grid);

    let result = handle_dock_click(&mut dock, &store, &typo, SCREEN, center(layout.apply_rect));
    let Some(PanelClickResult::ApplyLayout(state)) = result else {
        panic!("Appliquer demande la disposition : {result:?}");
    };
    assert_eq!(state.layout, LayoutMode::Grid);
}

// ── POMODORO ────────────────────────────────────────────────────────────────

/// Le minuteur démarre, se met en pause, se remet à zéro et change de durée — par le clic,
/// là où les boutons sont dessinés.
#[test]
fn test_l4_the_pomodoro_buttons_do_what_they_show() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Pomodoro);
    let frame = frame_of(&dock, TabId::Pomodoro, SCREEN);
    let layout = pomodoro::layout_pomodoro_panel(frame, &typo);

    handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.start_button),
    );
    assert!(dock.pomodoro.running, "le minuteur démarre");

    // Le libellé passe de « Démarrer » à « Pause », mais **la géométrie ne bouge pas** : le
    // bouton réserve la largeur du plus long des deux, donc un second clic au même endroit
    // met bien en pause, et le voisin `↺` reste où il est. Sans cela, le bouton se dérobait
    // sous le curseur à chaque clic.
    handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.start_button),
    );
    assert!(!dock.pomodoro.running, "le minuteur se met en pause");
    let after = pomodoro::layout_pomodoro_panel(frame, &typo);
    assert_eq!(
        after.reset_button, layout.reset_button,
        "le bouton ↺ ne se déplace pas quand le minuteur démarre"
    );

    let quart = &layout.presets[1];
    handle_dock_click(&mut dock, &store, &typo, SCREEN, center(quart.rect));
    assert_eq!(dock.pomodoro.total_seconds, 15 * 60);
    assert_eq!(dock.pomodoro.left_seconds, 15 * 60);

    dock.pomodoro.left_seconds = 42;
    handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.reset_button),
    );
    assert_eq!(dock.pomodoro.left_seconds, dock.pomodoro.total_seconds);
}

/// Le décompte suit l'horloge : une seconde écoulée, une seconde de moins. Et il s'arrête à
/// zéro plutôt que de déborder.
#[test]
fn test_the_pomodoro_counts_down_and_stops_at_zero() {
    let mut state = PomodoroState {
        total_seconds: 60,
        left_seconds: 3,
        running: true,
        last_tick: Instant::now() - std::time::Duration::from_secs(2),
    };
    assert!(state.tick());
    assert_eq!(state.left_seconds, 1);

    state.last_tick = Instant::now() - std::time::Duration::from_secs(5);
    assert!(state.tick());
    assert_eq!(state.left_seconds, 0, "le décompte s'arrête à zéro");
    assert!(!state.running);
    assert!(!state.tick(), "à l'arrêt, rien ne change plus");
}

// ── STORYBOARD & PLUGINS : les façades honnêtes ─────────────────────────────

/// Fiche 09 § 8 — le storyboard ne touche pas au canevas, et ne prétend pas le contraire :
/// son bouton ne reste pas allumé, et le clic dit pourquoi.
#[test]
fn test_the_storyboard_says_it_is_not_ready() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Storyboard);
    let frame = frame_of(&dock, TabId::Storyboard, SCREEN);
    let layout = storyboard::layout_storyboard_panel(frame);

    let result = handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.activate_button),
    );
    assert_eq!(result, Some(PanelClickResult::StoryboardNotReady));
    assert!(!dock.storyboard.active, "le bouton ne reste pas allumé");

    let result = handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.format_button),
    );
    assert_eq!(result, Some(PanelClickResult::StoryboardNotReady));
}

/// Fiche 09 § 10.3 — pas de moteur, pas de téléchargement. Les réglages, eux, sont un état
/// d'interface : ils se cochent.
#[test]
fn test_the_plugins_panel_selects_settings_but_downloads_nothing() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Plugins);
    let frame = frame_of(&dock, TabId::Plugins, SCREEN);
    let layout = plugins::layout_plugins_panel(frame);

    let result = handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.download_button),
    );
    assert_eq!(result, Some(PanelClickResult::DownloadModel));

    handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.density_options[2]),
    );
    assert_eq!(dock.plugins.density_idx, 2);
    handle_dock_click(
        &mut dock,
        &store,
        &typo,
        SCREEN,
        center(layout.disposition_options[1]),
    );
    assert_eq!(dock.plugins.disposition_idx, 1);
}

/// Un clic dans un panneau ne traverse jamais jusqu'au canevas, même là où il ne fait rien.
#[test]
fn test_a_click_inside_a_panel_never_falls_through() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    for tab in [TabId::Preset, TabId::Storyboard, TabId::Domains] {
        let mut dock = dock_with(tab);
        let frame = frame_of(&dock, tab, SCREEN);
        let middle = Pointer {
            x: frame.x + frame.w / 2.0,
            y: frame.y + frame.h / 2.0,
        };
        assert!(
            handle_dock_click(&mut dock, &store, &typo, SCREEN, middle).is_some(),
            "{tab:?}"
        );
    }
}

/// Hors de tout panneau, le dock ne prend pas le clic.
#[test]
fn test_a_click_outside_every_panel_is_not_taken() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = DockManager::new();
    let middle = Pointer { x: 900.0, y: 400.0 };
    assert!(handle_dock_click(&mut dock, &store, &typo, SCREEN, middle).is_none());
}

// ── Le rendu ────────────────────────────────────────────────────────────────

/// Budget de temps maximal accepté pour un rendu complet de docks (debug).
const DOCK_RENDER_BUDGET_MS: u128 = 2_000;

/// Non-régression du gel de démarrage : `redraw()` passait la position de la souris à
/// l'emplacement de l'argument `scale`, ce qui portait l'échelle des panneaux à 170 et le
/// coût d'une frame à plus de 10 s — la pompe de messages Windows était affamée et la
/// fenêtre restait blanche et « Ne répond pas ».
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
        ScreenFrame {
            scale: 170.0,
            ..SCREEN
        },
        Pointer { x: 0.0, y: 0.0 },
    );
    let elapsed = started.elapsed().as_millis();
    assert!(
        elapsed < DOCK_RENDER_BUDGET_MS,
        "rendu des docks à échelle aberrante : {elapsed} ms (budget {DOCK_RENDER_BUDGET_MS} ms)"
    );

    for b in compute_panel_layouts(&dock, 1440.0, 900.0, 78.0, 170.0) {
        assert!(
            b.width <= 1440.0,
            "panneau {:?} plus large que l'écran",
            b.tab
        );
        assert!(
            b.height <= 900.0,
            "panneau {:?} plus haut que l'écran",
            b.tab
        );
    }
}

/// Chaque panneau met de l'encre sur le pixmap. Le piège serait une géométrie juste et un
/// dessin vide : tous les tests de clic passeraient, et l'écran resterait noir.
#[test]
fn test_every_panel_puts_ink_on_the_pixmap() {
    let theme = Theme::dark();
    let typo = Typography::new();
    let store = Store::new("Encre");
    for tab in [
        TabId::Organize,
        TabId::Pomodoro,
        TabId::Storyboard,
        TabId::Plugins,
        TabId::Preset,
        TabId::Domains,
    ] {
        let mut pixmap = tiny_skia::Pixmap::new(1440, 900).expect("pixmap");
        pixmap.fill(theme.bg_canvas);
        let before = pixmap.data().to_vec();
        render_docks(
            &mut pixmap.as_mut(),
            &dock_with(tab),
            &store,
            &typo,
            &theme,
            SCREEN,
            Pointer { x: 0.0, y: 0.0 },
        );
        assert_ne!(pixmap.data(), &before[..], "{tab:?} n'a rien dessiné");
    }
}
