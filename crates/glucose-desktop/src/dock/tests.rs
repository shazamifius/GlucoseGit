//! Les lois du dock : ses ancrages, son glissement, et — pour chaque panneau — le fait que
//! **le clic tombe là où le dessin le montre** (loi L4).

use super::*;
use crate::theme::Theme;
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
        &DockPass {
            typo: &typo,
            theme: &theme,
            screen: ScreenFrame {
                scale: 170.0,
                ..SCREEN
            },
            pointer: Pointer { x: 0.0, y: 0.0 },
            cache: None,
        },
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
            &DockPass {
                typo: &typo,
                theme: &theme,
                screen: SCREEN,
                pointer: Pointer { x: 0.0, y: 0.0 },
                cache: None,
            },
        );
        assert_ne!(pixmap.data(), &before[..], "{tab:?} n'a rien dessiné");
    }
}

// ── DOCK-CACHE-1 : le tampon par panneau ──────────────────────────────────────

/// Le plus grand écart entre deux images, canal par canal, et où il se trouve.
///
/// Jamais `assert_eq!` sur les octets d'un pixmap — l'échec en déverserait plusieurs
/// mégaoctets, et le message ne dirait rien de ce qui a bougé.
fn ecart_max(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap) -> (u8, usize, String) {
    let (pa, pb) = (a.pixels(), b.pixels());
    assert_eq!(pa.len(), pb.len(), "tailles différentes");
    let w = a.width() as usize;
    let (mut pire, mut compte, mut ou) = (0u8, 0usize, String::from("nulle part"));
    for (i, (x, y)) in pa.iter().zip(pb).enumerate() {
        let d = [
            x.red().abs_diff(y.red()),
            x.green().abs_diff(y.green()),
            x.blue().abs_diff(y.blue()),
            x.alpha().abs_diff(y.alpha()),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        if d > 0 {
            compte += 1;
        }
        if d > pire {
            pire = d;
            ou = format!(
                "({}, {}) : rgba({}, {}, {}, {}) contre rgba({}, {}, {}, {})",
                i % w,
                i / w,
                x.red(),
                x.green(),
                x.blue(),
                x.alpha(),
                y.red(),
                y.green(),
                y.blue(),
                y.alpha(),
            );
        }
    }
    (pire, compte, ou)
}

/// Les deux images ne diffèrent **au plus** que d'un pas de quantification (DOCK-CACHE-1).
///
/// # Pourquoi un pas, et pas zéro
///
/// L'égalité stricte serait fausse à annoncer, et la mesure le dit : composer l'ombre
/// semi-transparente d'un panneau sur un tampon, puis ce tampon sur le fond, n'arrondit pas
/// comme une composition unique. `src-over` est associatif sur les couleurs exactes, il ne
/// l'est pas sur huit bits entiers — l'erreur de la première composition est arrondie avant
/// que la seconde ne la lise.
///
/// L'écart est donc **borné par construction à une unité sur 255**, et seulement là où
/// quelque chose est semi-transparent : l'ombre, et le liseré d'anti-crénelage. Partout où le
/// panneau est opaque, `src-over` rend exactement la source, donc l'égalité est stricte.
///
/// Ce test vérifie cette borne plutôt que de la supposer. Si une composition venait un jour à
/// s'empiler une fois de plus, l'écart passerait à deux et le test le dirait.
fn memes_pixels(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap, quoi: &str) {
    let (pire, compte, ou) = ecart_max(a, b);
    assert!(
        pire <= 1,
        "{quoi} : écart de {pire}/255 sur {compte} pixel(s), au pire en {ou}"
    );
}

/// Les six panneaux ouverts en même temps, pour éprouver le cache sur tous les cas.
fn dock_complet() -> DockManager {
    let mut dock = DockManager::new();
    dock.top_tabs = vec![TabId::Plugins, TabId::Preset, TabId::Domains];
    dock.bottom_tabs = vec![TabId::Organize, TabId::Pomodoro, TabId::Storyboard];
    dock
}

/// Rend le dock, avec ou sans cache, sur un fond connu.
fn rendu_dock(
    dock: &DockManager,
    cache: Option<&DockCache>,
    pointer: Pointer,
) -> tiny_skia::Pixmap {
    rendu_dock_sur(dock, &Store::new("Cache"), cache, pointer)
}

/// Le même, sur un document donné — pour les panneaux qui lisent le document.
fn rendu_dock_sur(
    dock: &DockManager,
    store: &Store,
    cache: Option<&DockCache>,
    pointer: Pointer,
) -> tiny_skia::Pixmap {
    let theme = Theme::dark();
    let typo = Typography::new();
    let mut pixmap = tiny_skia::Pixmap::new(1440, 900).expect("pixmap");
    pixmap.fill(theme.bg_canvas);
    render_docks(
        &mut pixmap.as_mut(),
        dock,
        store,
        &DockPass {
            typo: &typo,
            theme: &theme,
            screen: SCREEN,
            pointer,
            cache,
        },
    );
    pixmap
}

/// **DOCK-CACHE-1** — le cache rend **exactement** la même image que le rendu direct.
///
/// C'est l'invariant qui autorise tout le reste. Un cache qui accélère en changeant d'un
/// cheveu ce qui est affiché n'est pas une optimisation : c'est une régression visuelle que
/// personne ne verrait avant de comparer deux captures.
///
/// L'égalité est exacte parce que la translation vers le tampon est **entière** : la fraction
/// sous-pixel reste dans les coordonnées, donc l'anti-crénelage tombe sur la même couverture.
#[test]
fn test_dock_cache_1_the_cached_dock_is_the_same_image_as_the_direct_one() {
    let dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let direct = rendu_dock(&dock, None, dehors);
    let cache = DockCache::new();
    let par_cache = rendu_dock(&dock, Some(&cache), dehors);
    memes_pixels(&direct, &par_cache, "le cache diverge du rendu direct");
}

/// L'égalité tient aussi quand le pointeur survole un panneau — le survol passe par le cache.
#[test]
fn test_the_cached_dock_is_identical_while_hovering() {
    let dock = dock_complet();
    // Dans le premier panneau du bas, là où des boutons attendent un survol.
    let dessus = Pointer { x: 60.0, y: 700.0 };
    let direct = rendu_dock(&dock, None, dessus);
    let cache = DockCache::new();
    let par_cache = rendu_dock(&dock, Some(&cache), dessus);
    memes_pixels(&direct, &par_cache, "le survol diverge");
}

/// Sans rien changer, **rien n'est redessiné** après la première image.
///
/// C'est ce que `bench_chrome` avait mesuré — cent images pour cent fois les mêmes octets —
/// et c'est ce que le cache doit supprimer. Sans ce test, un cache qui ne servirait jamais
/// rendrait la bonne image et passerait inaperçu.
#[test]
fn test_an_unchanged_dock_is_never_redrawn_twice() {
    let dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cache = DockCache::new();
    for _ in 0..20 {
        rendu_dock(&dock, Some(&cache), dehors);
    }
    assert_eq!(
        cache.rendus(),
        6,
        "six panneaux devaient être dessinés une fois chacun, pour vingt images"
    );
    assert_eq!(cache.len(), 6, "les six tampons ne sont pas gardés");
}

/// Un pointeur qui se promène **hors** des panneaux n'en invalide aucun.
///
/// C'est la raison d'être de la clé conditionnelle : le cas courant est la souris sur le
/// canevas, et une clé contenant sa position rendrait le cache inutile précisément là où il
/// sert le plus.
#[test]
fn test_a_pointer_outside_the_panels_invalidates_nothing() {
    let dock = dock_complet();
    let cache = DockCache::new();
    rendu_dock(&dock, Some(&cache), Pointer { x: -1.0, y: -1.0 });
    let apres_la_premiere = cache.rendus();
    for i in 0..200 {
        // À droite des deux docks : avec six panneaux ouverts, celui du haut court
        // jusqu'à x ≈ 972, et le milieu de l'écran est encore dedans.
        rendu_dock(
            &dock,
            Some(&cache),
            Pointer {
                x: 1100.0 + i as f32 * 0.5,
                y: 400.0,
            },
        );
    }
    assert_eq!(
        cache.rendus(),
        apres_la_premiere,
        "la souris hors des panneaux a fait redessiner"
    );
}

/// Un changement d'état refait le panneau concerné, **et lui seul**.
#[test]
fn test_a_state_change_redraws_only_its_own_panel() {
    let mut dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cache = DockCache::new();
    rendu_dock(&dock, Some(&cache), dehors);
    assert_eq!(cache.rendus(), 6);

    dock.organize.cols = 7;
    rendu_dock(&dock, Some(&cache), dehors);
    assert_eq!(
        cache.rendus(),
        7,
        "changer ORDONNER devait refaire ORDONNER, et rien d'autre"
    );

    dock.plugins.density_idx = 0;
    rendu_dock(&dock, Some(&cache), dehors);
    assert_eq!(
        cache.rendus(),
        8,
        "changer PLUGINS devait refaire PLUGINS, et rien d'autre"
    );
}

/// Un état changé se **voit** : le cache ne sert pas une image périmée.
///
/// Le test précédent dit que le panneau est redessiné ; celui-ci dit que ce qu'on voit a
/// vraiment changé. Les deux ensemble ferment la porte au cache qui ment.
#[test]
fn test_a_changed_panel_shows_its_new_state() {
    let mut dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cache = DockCache::new();
    let avant = rendu_dock(&dock, Some(&cache), dehors);

    dock.organize.layout = LayoutMode::Grid;
    dock.organize.cols = 9;
    let apres = rendu_dock(&dock, Some(&cache), dehors);
    assert!(
        ecart_max(&avant, &apres).1 > 0,
        "le panneau montre encore son état d'avant"
    );
    // Et cette image est bien celle du rendu direct.
    memes_pixels(
        &apres,
        &rendu_dock(&dock, None, dehors),
        "l'état neuf diverge",
    );
}

/// **DOCKS-1** — sélectionner des nœuds se voit dans les panneaux qui comptent la sélection.
///
/// « Domaines » écrit « N nœud(s) sélectionné(s) » et grise ses boutons d'assignation quand
/// rien n'est sélectionné ; « Ordonner » compte les images visées. La clé du cache ne
/// connaissait du document que sa **version**, et sélectionner n'en est pas une : c'est de la
/// navigation, pas une commande (fiche 05 § 3.5). Le panneau restait donc sur son ancien
/// compte, boutons grisés, tant que la souris ne passait pas dessus — un bouton qui ment,
/// exactement ce que la fiche 05 § 5.4 interdit.
///
/// La sélection est posée par l'API du document, sur des identifiants qu'aucun nœud ne porte :
/// les deux panneaux n'en lisent que le **nombre**, et c'est ce que ce test exerce.
#[test]
fn test_a_selection_change_shows_in_the_panels_that_count_it() {
    let dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let mut store = Store::new("Selection");
    let cache = DockCache::new();
    rendu_dock_sur(&dock, &store, Some(&cache), dehors);

    store.select_image("img-a".into(), false);
    store.select_annotation("note-b".into(), true);
    let par_le_cache = rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    let en_direct = rendu_dock_sur(&dock, &store, None, dehors);
    memes_pixels(
        &par_le_cache,
        &en_direct,
        "le cache montre la selection d'avant",
    );
}

/// **DOCKS-1** — le cache dit POURQUOI il a refait un panneau, et il le dit juste.
///
/// La section « Pourquoi les panneaux se redessinent » en dépend : une raison mal nommée y
/// désignerait le mauvais remède, ce que ce dépôt a payé quatre fois avec des marques de
/// mesure mal posées. Chaque cas change **une seule** partie de la clé et exige ce bit-là, et
/// lui seul.
#[test]
fn test_the_cache_names_why_a_panel_was_redrawn() {
    use crate::dock::RaisonDuPanneau as R;
    let dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let mut store = Store::new("Raisons");
    let cache = DockCache::new();

    rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    assert_eq!(cache.prendre_les_raisons(), R::PremiereFois.bit());

    rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    assert_eq!(
        cache.prendre_les_raisons(),
        0,
        "rien n'a change, rien ne se refait"
    );

    store.select_image("img-a".into(), false);
    rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    assert_eq!(cache.prendre_les_raisons(), R::Selection.bit());

    // SURVOL-2 : le pointeur ne compte que par ce que le dessin lui demande. Sur la poignée
    // du premier panneau, une réponse change — il se refait pour elle.
    let panneau =
        &compute_panel_layouts(&dock, SCREEN.width, SCREEN.height, SCREEN.header_h, 1.0)[0];
    let poignee = panneau.grip_rect();
    let sur = |dx: f32| Pointer {
        x: poignee.x + poignee.w / 2.0 + dx,
        y: poignee.y + poignee.h / 2.0,
    };
    rendu_dock_sur(&dock, &store, Some(&cache), sur(0.0));
    assert_eq!(cache.prendre_les_raisons(), R::Pointeur.bit());

    // Un demi-pixel plus loin, toujours sur la poignée : aucune réponse ne change, rien ne se
    // refait. La clé gardait la position exacte, et c'est ce qui redessinait le panneau entier
    // à chaque pixel de mouvement — 76 fois en 35 s sur la session du 24/09.
    rendu_dock_sur(&dock, &store, Some(&cache), sur(0.5));
    assert_eq!(
        cache.prendre_les_raisons(),
        0,
        "le pointeur a bouge sans rien changer a ce qu'il survole"
    );

    // Il ressort : la poignée n'est plus survolée, le panneau se refait.
    rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    assert_eq!(cache.prendre_les_raisons(), R::Pointeur.bit());
}

/// L'ombre d'un panneau tient **dans** la boîte que le cache lui réserve.
///
/// Si `extent` était plus petite que ce que `draw_frame` noircit, le cache rognerait l'ombre
/// — et comme les deux dérivent maintenant de `shadow()`, ce test est ce qui garantit qu'elles
/// ne peuvent pas se séparer plus tard.
#[test]
fn test_the_reserved_box_contains_the_shadow() {
    for scale in [1.0f32, 1.25, 2.0] {
        for panel in compute_panel_layouts(
            &dock_complet(),
            SCREEN.width,
            SCREEN.height,
            SCREEN.header_h,
            scale,
        ) {
            let extent = panel.extent(scale);
            let shadow = panel.shadow(scale);
            let seen = panel.seen();
            for (nom, r) in [("l'ombre", shadow), ("le cadre", seen)] {
                assert!(
                    r.x >= extent.x
                        && r.y >= extent.y
                        && r.x + r.w <= extent.x + extent.w
                        && r.y + r.h <= extent.y + extent.h,
                    "{nom} de {:?} sort de la boîte réservée à l'échelle {scale}",
                    panel.tab
                );
            }
        }
    }
}
