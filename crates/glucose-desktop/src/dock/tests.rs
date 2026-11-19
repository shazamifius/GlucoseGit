//! Les lois du dock : ses côtés, son glissement, et — pour chaque panneau — le fait que
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
    let store = Store::new("essai");
    let panel = compute_panel_layouts(
        dock,
        &store,
        screen.width,
        screen.height,
        screen.header_h,
        s,
    )
    .into_iter()
    .find(|b| b.tab == tab)
    .expect("le panneau est ouvert");
    // Le cadre de son contenu, sans la poignée (TIROIR-1) : celui que l'application lit.
    panel.contenu(s)
}

fn center(rect: WidgetRect) -> Pointer {
    Pointer {
        x: rect.x + rect.w / 2.0,
        y: rect.y + rect.h / 2.0,
    }
}

fn dock_with(tab: TabId) -> DockManager {
    let mut dock = DockManager::new();
    dock.tout_fermer();
    dock.tabs_mut(tab.cote()).push(tab);
    dock
}

/// La géométrie que le dessin lit, sur l'écran de ces épreuves : ce que le glisser reçoit.
fn geometrie(store: &Store) -> impl Fn(&DockManager) -> Vec<PanelLayoutBox> + '_ {
    move |d| compute_panel_layouts(d, store, SCREEN.width, SCREEN.height, SCREEN.header_h, 1.0)
}

/// Les largeurs de la fiche 10 § 4 : Domaines 320, Presets 280, Plugins 340 ; Ordonner 250,
/// Storyboard 260, Pomodoro 160 au minimum. Fermeture au-delà de 80 px hors de l'écran.
#[test]
fn test_dock_defaults() {
    assert_eq!(TabId::Domains.default_width(), 320.0);
    assert_eq!(TabId::Preset.default_width(), 280.0);
    assert_eq!(TabId::Plugins.default_width(), 340.0);
    assert_eq!(TabId::Organize.default_width(), 250.0);
    assert_eq!(TabId::Storyboard.default_width(), 260.0);
    assert!(TabId::Pomodoro.default_width() >= 160.0);
    assert_eq!(DISMISS_DRAG_PX, 80.0);
}

/// **TIROIR-4 — collés au bord, sous la bande, la poignée en bas.** Un tiroir seul de son côté
/// touche le bord de l'écran et la bande ; sa poignée est sous son contenu.
#[test]
fn test_tiroir_4_colles_au_bord_sous_la_bande() {
    for tab in [
        TabId::Temps,
        TabId::Domains,
        TabId::Organize,
        TabId::Plugins,
    ] {
        let b = compute_panel_layouts(
            &dock_with(tab),
            &Store::new("essai"),
            1440.0,
            900.0,
            78.0,
            1.0,
        )
        .into_iter()
        .find(|b| b.tab == tab)
        .expect("ouvert");
        let bord = match tab.cote() {
            Cote::Gauche => b.x,
            Cote::Droite => 1440.0 - (b.x + b.width),
        };
        assert_eq!(
            (bord, b.y),
            (0.0, 78.0),
            "{tab:?} collé au bord, sous la bande"
        );
        assert_eq!(
            b.grip_y,
            b.y + b.height - b.grip_height,
            "{tab:?} : la poignée en bas"
        );
    }
}

/// **TIROIR-4 — une colonne, une largeur ; une colonne voisine quand la hauteur manque.** Sur un
/// écran bas, Ordonner puis Pomodoro puis le Storyboard : les deux premiers s'empilent, le
/// troisième ouvre une colonne vers l'intérieur ; chaque tiroir d'une colonne en a la largeur.
#[test]
fn test_tiroir_4_une_colonne_voisine_quand_la_hauteur_manque() {
    let mut dock = DockManager::new();
    for tab in [TabId::Organize, TabId::Pomodoro, TabId::Storyboard] {
        dock.toggle_tab(tab);
    }
    let boites = compute_panel_layouts(&dock, &Store::new("essai"), 1440.0, 800.0, 78.0, 1.0);
    let b = |tab| boites.iter().find(|b| b.tab == tab).expect("ouvert");
    let (o, p, sb) = (b(TabId::Organize), b(TabId::Pomodoro), b(TabId::Storyboard));
    assert_eq!((o.x, o.y), (0.0, 78.0));
    assert_eq!(
        (p.x, p.y),
        (0.0, o.y + o.height),
        "Pomodoro sous Ordonner, sans écart"
    );
    assert_eq!(
        p.width, o.width,
        "une colonne, une largeur : celle d'Ordonner, la plus large"
    );
    assert!(p.y + p.height <= 800.0);
    assert_eq!(
        (sb.x, sb.y),
        (o.width, 78.0),
        "le Storyboard ouvre la colonne voisine"
    );
    assert_eq!(
        sb.width,
        TabId::Storyboard.default_width(),
        "sa colonne a sa largeur"
    );
}

/// Ordonner et Pomodoro ouverts, dans cet ordre : ce que Glucose Tauri ouvrait au lancement, et
/// que Glucose n'ouvre plus (fiche 59).
fn ordonner_et_pomodoro() -> DockManager {
    let mut dock = DockManager::new();
    dock.toggle_tab(TabId::Organize);
    dock.toggle_tab(TabId::Pomodoro);
    dock
}

#[test]
fn test_dock_manager_toggle_and_dismiss() {
    let mut dock = ordonner_et_pomodoro();
    assert!(dock.is_open(TabId::Organize));
    assert!(dock.is_open(TabId::Pomodoro));
    assert_eq!(dock.gauche.len(), 2);

    dock.toggle_tab(TabId::Organize);
    assert!(!dock.is_open(TabId::Organize));
    assert_eq!(dock.gauche.len(), 1);

    dock.toggle_tab(TabId::Plugins);
    assert!(dock.is_open(TabId::Plugins));
    assert_eq!(dock.droite.len(), 1);

    dock.dismiss_tab(TabId::Plugins);
    assert!(!dock.is_open(TabId::Plugins));
}

/// Un glisser qui commence sur la poignée de ce tiroir.
fn tenir(dock: &mut DockManager, store: &Store, tab: TabId) -> (f32, f32) {
    let b = geometrie(store)(dock)
        .into_iter()
        .find(|b| b.tab == tab)
        .expect("ouvert");
    let (x, y) = (b.x + b.width / 2.0, b.grip_y + b.grip_height / 2.0);
    dock.drag = Some(DragSession {
        tab,
        start_x: x,
        start_y: y,
        current_x: x,
        current_y: y,
    });
    (x, y)
}

/// **Glisser range, tirer hors de l'écran ferme** (TIROIR-4). Pomodoro, sous Ordonner, tiré
/// vers le haut, passe au-dessus — et ne saute pas : il reste là où la main le tient. Tiré vers
/// la gauche, 79 px au-delà du bord le gardent ; 81 le ferment (fiche 10 § 4).
#[test]
fn test_dock_drag_swap_and_dismiss() {
    let store = Store::new("essai");
    let mut dock = ordonner_et_pomodoro();
    assert_eq!(dock.gauche, vec![TabId::Organize, TabId::Pomodoro]);

    let (x, y) = tenir(&mut dock, &store, TabId::Pomodoro);
    let vu = |dock: &DockManager| {
        geometrie(&store)(dock)
            .into_iter()
            .find(|b| b.tab == TabId::Pomodoro)
            .expect("ouvert")
            .seen()
    };
    let depart = vu(&dock);
    assert!(
        !dock.update_drag((x, y - 100.0), geometrie(&store)),
        "trop peu"
    );
    assert!(
        dock.update_drag((x, y - 330.0), geometrie(&store)),
        "il change de place"
    );
    assert_eq!(dock.gauche, vec![TabId::Pomodoro, TabId::Organize]);
    let tenu = vu(&dock);
    assert_eq!(
        (tenu.x, tenu.y),
        (depart.x, depart.y - 330.0),
        "il ne saute pas : il reste sous la main"
    );
    assert_eq!(dock.finish_drag(SCREEN.width, geometrie(&store)), None);

    let (x, y) = tenir(&mut dock, &store, TabId::Pomodoro);
    dock.update_drag((x - 79.0, y), geometrie(&store));
    assert_eq!(
        dock.finish_drag(SCREEN.width, geometrie(&store)),
        None,
        "79 px : il reste"
    );
    assert!(dock.is_open(TabId::Pomodoro));
    let (x, y) = tenir(&mut dock, &store, TabId::Pomodoro);
    dock.update_drag((x - 81.0, y), geometrie(&store));
    assert_eq!(
        dock.finish_drag(SCREEN.width, geometrie(&store)),
        Some(TabId::Pomodoro)
    );
    assert!(!dock.is_open(TabId::Pomodoro));

    // À droite, la sortie est le bord droit.
    let mut dock = dock_with(TabId::Domains);
    let (x, y) = tenir(&mut dock, &store, TabId::Domains);
    dock.update_drag((x - 200.0, y), geometrie(&store));
    assert_eq!(
        dock.finish_drag(SCREEN.width, geometrie(&store)),
        None,
        "vers l'intérieur : il reste"
    );
    let (x, y) = tenir(&mut dock, &store, TabId::Domains);
    dock.update_drag((x + 81.0, y), geometrie(&store));
    assert_eq!(
        dock.finish_drag(SCREEN.width, geometrie(&store)),
        Some(TabId::Domains)
    );
}

/// **Un tiroir de la colonne voisine traverse la première sans se fermer** : c'est un
/// rangement, pas une sortie — la sortie est le bord de l'écran.
#[test]
fn test_tiroir_4_la_colonne_voisine_se_range_sans_se_fermer() {
    let store = Store::new("essai");
    let mut dock = DockManager::new();
    for tab in [TabId::Organize, TabId::Pomodoro, TabId::Storyboard] {
        dock.toggle_tab(tab);
    }
    let places = |d: &DockManager| compute_panel_layouts(d, &store, 1440.0, 800.0, 78.0, 1.0);
    let sb = places(&dock)
        .into_iter()
        .find(|b| b.tab == TabId::Storyboard)
        .expect("ouvert");
    assert!(sb.x > 0.0, "le Storyboard est dans la colonne voisine");
    let (x, y) = (sb.x + sb.width / 2.0, sb.grip_y + sb.grip_height / 2.0);
    dock.drag = Some(DragSession {
        tab: TabId::Storyboard,
        start_x: x,
        start_y: y,
        current_x: x,
        current_y: y,
    });
    dock.update_drag((x - 120.0, y), places);
    assert_eq!(
        dock.finish_drag(1440.0, places),
        None,
        "120 px vers le bord : il reste"
    );
    assert!(dock.is_open(TabId::Storyboard));
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

/// **ORDONNER-2 — les trois tris par la couleur se choisissent**, par le clic, là où ils sont
/// dessinés ; ils étaient grisés faute des pixels. Une disposition que le noyau ne sait pas
/// faire, elle, ne se choisit toujours pas : le bouton est inerte, et l'état ne bouge pas.
#[test]
fn test_the_colour_sorts_are_chosen_and_an_unimplemented_layout_is_not() {
    let typo = Typography::new();
    let store = Store::new("Clic");
    let mut dock = dock_with(TabId::Organize);
    let frame = frame_of(&dock, TabId::Organize, SCREEN);
    let layout = organize::layout_organize_panel(frame, &typo);

    for sort in [SortType::Color, SortType::LumAsc, SortType::LumDesc] {
        assert!(sort.implemented(), "{sort:?} se fait");
        assert!(
            sort.tri_par_couleur().is_some(),
            "{sort:?} lit les couleurs"
        );
        let button = layout
            .sort_buttons
            .iter()
            .find(|b| b.sort_type == sort)
            .expect("le tri est dans le panneau");
        handle_dock_click(&mut dock, &store, &typo, SCREEN, center(button.rect));
        assert_eq!(dock.organize.sort_by, sort);
    }
    // « Par slot preset » aussi, depuis que les gabarits se posent (PRESET-1).
    assert!(LayoutMode::BySlot.implemented());
    let by_slot = layout
        .layout_modes
        .iter()
        .find(|m| m.mode == LayoutMode::BySlot)
        .expect("la disposition est dans le panneau");
    handle_dock_click(&mut dock, &store, &typo, SCREEN, center(by_slot.rect));
    assert_eq!(dock.organize.layout, LayoutMode::BySlot);
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

// ── PLUGINS : une façade honnête ─────────────────────────────────────────────

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

    for b in compute_panel_layouts(&dock, &Store::new("essai"), 1440.0, 900.0, 78.0, 170.0) {
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

/// Les deux images sont **les mêmes, au bit près** (DOCK-CACHE-1).
///
/// # Pourquoi zéro, et plus un pas
///
/// Le rendu sans cache peignait les panneaux directement sur l'image, et le cache les pose
/// depuis leur tampon. `src-over` est associatif sur les couleurs exactes, pas sur huit bits :
/// chaque couche semi-transparente s'arrondit avant que la suivante ne la lise, et le tampon
/// ajoute une composition. L'écart admis était « une unité, par construction » — ce n'était
/// pas une borne : il vaut une unité **par couche transparente superposée**, et deux pixels
/// sont passés à deux quand les coins sont devenus des arcs (ARC-1), là où deux ombres se
/// croisent, et là où l'ombre, le fond et le trait d'un panneau se superposent au coin.
///
/// Le rendu sans cache passe maintenant par un cache neuf, le même chemin : les arrondis sont
/// les mêmes, et l'égalité est stricte. Plus rien n'est toléré, donc un panneau périmé d'un
/// seul niveau se voit.
fn memes_pixels(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap, quoi: &str) {
    let (pire, compte, ou) = ecart_max(a, b);
    assert!(
        pire == 0,
        "{quoi} : écart de {pire}/255 sur {compte} pixel(s), au pire en {ou}"
    );
}

/// Les six panneaux ouverts en même temps, pour éprouver le cache sur tous les cas.
fn dock_complet() -> DockManager {
    let mut dock = DockManager::new();
    dock.gauche = vec![TabId::Organize, TabId::Pomodoro, TabId::Storyboard];
    dock.droite = vec![TabId::Plugins, TabId::Preset, TabId::Domains];
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

/// **DOCK-CACHE-1** — ce que le cache **réutilise** est exactement ce qu'un rendu neuf peint.
///
/// C'est l'invariant qui autorise tout le reste. Un cache qui accélère en changeant d'un
/// cheveu ce qui est affiché n'est pas une optimisation : c'est une régression visuelle que
/// personne ne verrait avant de comparer deux captures. La seconde image est prise dans les
/// tampons gardés — c'est elle qu'on compare, pas la première, qui se dessine.
#[test]
fn test_dock_cache_1_the_cached_dock_is_the_same_image_as_the_direct_one() {
    let dock = dock_complet();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cache = DockCache::new();
    rendu_dock(&dock, Some(&cache), dehors);
    let reutilise = rendu_dock(&dock, Some(&cache), dehors);
    assert_eq!(
        cache.rendus(),
        6,
        "la seconde image est prise dans le cache"
    );
    let neuf = rendu_dock(&dock, None, dehors);
    memes_pixels(&neuf, &reutilise, "le cache diverge du rendu neuf");
}

/// L'égalité tient quand le pointeur **entre** dans un panneau : le cache, rempli pointeur
/// dehors, doit refaire ce que le survol change.
#[test]
fn test_the_cached_dock_is_identical_while_hovering() {
    let dock = dock_complet();
    // Dans le premier panneau du bas, là où des boutons attendent un survol.
    let dessus = Pointer { x: 60.0, y: 700.0 };
    let cache = DockCache::new();
    rendu_dock(&dock, Some(&cache), Pointer { x: -1.0, y: -1.0 });
    let par_cache = rendu_dock(&dock, Some(&cache), dessus);
    let neuf = rendu_dock(&dock, None, dessus);
    memes_pixels(&neuf, &par_cache, "le survol diverge");
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

    // DOMAINE-1 : le domaine regardé, que la version ne voit pas non plus.
    store.set_domaine_regarde(Some("eau".into()));
    rendu_dock_sur(&dock, &store, Some(&cache), dehors);
    assert_eq!(cache.prendre_les_raisons(), R::Lentille.bit());

    // SURVOL-2 : le pointeur ne compte que par ce que le dessin lui demande. Sur la poignée
    // du premier panneau, une réponse change — il se refait pour elle.
    let panneau = &compute_panel_layouts(
        &dock,
        &Store::new("essai"),
        SCREEN.width,
        SCREEN.height,
        SCREEN.header_h,
        1.0,
    )[0];
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

/// Le cadre d'un panneau, son trait compris, tient **dans** la boîte que le cache lui réserve
/// (DOCK-CACHE-1) — et la serre : plat, il n'a plus d'ombre qui déborde (TIROIR-4).
#[test]
fn test_the_reserved_box_contains_the_frame() {
    for scale in [1.0f32, 1.25, 2.0] {
        for panel in compute_panel_layouts(
            &dock_complet(),
            &Store::new("essai"),
            SCREEN.width,
            SCREEN.height,
            SCREEN.header_h,
            scale,
        ) {
            let (extent, seen) = (panel.extent(scale), panel.seen());
            assert!(
                seen.x >= extent.x
                    && seen.y >= extent.y
                    && seen.x + seen.w <= extent.x + extent.w
                    && seen.y + seen.h <= extent.y + extent.h,
                "le cadre de {:?} sort de la boîte réservée à l'échelle {scale}",
                panel.tab
            );
            assert!(
                extent.w - seen.w <= scale + 1e-3,
                "{:?} : la boîte réserve plus que le trait",
                panel.tab
            );
        }
    }
}

/// Les sept panneaux ouverts, la Time Machine comprise.
fn les_sept() -> DockManager {
    let mut dock = dock_complet();
    dock.gauche.push(TabId::Temps);
    dock
}

/// Un document à `n` domaines.
fn avec_des_domaines(n: usize) -> Store {
    let mut store = Store::new("essai");
    for i in 0..n {
        let domaine = glucose_core::types::Domain {
            id: format!("d-{i}"),
            name: format!("domaine {i}"),
            color: "#888888".into(),
            icon: "◆".into(),
            defini_a: 0,
        };
        store.try_add_domain(domaine).expect("des noms distincts");
    }
    store
}

/// **TIROIR-1 — le contenu ne couvre jamais la poignée.** Sa capture du 17/11 : *« le tiroir de
/// domaine lui a littéralement UX SUR le bouton pour refermer le tiroir »*. « + Nouveau
/// domaine », le bas d'un panneau du haut, tombait sur sa poignée.
#[test]
fn test_tiroir_1_le_contenu_ne_couvre_jamais_la_poignee() {
    for n in [0, 2, 40] {
        let store = avec_des_domaines(n);
        for scale in [1.0, 1.5] {
            let screen = screen_at(scale);
            let boites = compute_panel_layouts(
                &les_sept(),
                &store,
                screen.width,
                screen.height,
                screen.header_h,
                scale,
            );
            for b in &boites {
                let contenu = b.contenu(scale);
                let poignee = b.grip_rect();
                assert!(
                    contenu.y + contenu.h <= poignee.y + 0.01,
                    "{:?} à {scale} : le contenu couvre la poignée",
                    b.tab
                );
            }
            let domaines = boites
                .iter()
                .find(|b| b.tab == TabId::Domains)
                .expect("ouvert");
            let ajouter = domains::layout_domains_panel(
                domaines.contenu(scale),
                &store,
                &domains::DomainsUi::default(),
            )
            .add_button;
            assert!(
                ajouter.y + ajouter.h <= domaines.grip_rect().y,
                "{n} domaines, à {scale} : « + Nouveau domaine » est sur la poignée"
            );
        }
    }
}

/// **TIROIR-2 — un panneau a la hauteur de ce qu'il montre.** Le Storyboard et la Time Machine
/// prenaient toute la hauteur de l'écran pour quelques lignes ; Domaines 430 pixels pour deux
/// domaines (sa capture : *« pourquoi le storyboard est aussi long à sa création ? »*).
#[test]
fn test_tiroir_2_un_panneau_a_la_hauteur_de_ce_qu_il_montre() {
    let haut_de = |dock: &DockManager, store: &Store, tab: TabId| {
        compute_panel_layouts(dock, store, 1440.0, 900.0, 78.0, 1.0)
            .into_iter()
            .find(|b| b.tab == tab)
            .expect("ouvert")
    };
    let vide = Store::new("essai");
    // Le Storyboard sans storyboard : ses réglages et « Activer », rien de plus.
    let sb = haut_de(&dock_with(TabId::Storyboard), &vide, TabId::Storyboard);
    assert!(sb.height < 400.0, "le Storyboard fait {} pixels", sb.height);
    let bas = storyboard::layout_storyboard_panel(
        sb.contenu(1.0),
        &StoryboardState::default(),
        &storyboard::Contenu::default(),
    )
    .bas;
    assert!(bas <= sb.y + sb.height + 0.01, "son contenu tient dedans");
    // Le Mood board sans gabarit suivi : ses six gabarits, pas la place d'un septième.
    let mb = haut_de(&dock_with(TabId::Preset), &vide, TabId::Preset);
    let plein = preset::hauteur_naturelle() + GRIP_HEIGHT;
    assert!(
        mb.height < plein - 50.0,
        "le Mood board fait {} pixels",
        mb.height
    );
    // La Time Machine sans jalon.
    let tm = haut_de(&dock_with(TabId::Temps), &vide, TabId::Temps);
    assert!(
        tm.height < 400.0,
        "la Time Machine fait {} pixels",
        tm.height
    );
    // Domaines grandit avec ses domaines, et les montre tous tant que l'écran le permet.
    let dock = dock_with(TabId::Domains);
    let (deux, six) = (avec_des_domaines(2), avec_des_domaines(6));
    let (h2, h6) = (
        haut_de(&dock, &deux, TabId::Domains),
        haut_de(&dock, &six, TabId::Domains),
    );
    assert!(h2.height < h6.height, "{} puis {}", h2.height, h6.height);
    for (store, b) in [(&deux, &h2), (&six, &h6)] {
        let l =
            domains::layout_domains_panel(b.contenu(1.0), store, &domains::DomainsUi::default());
        assert_eq!(l.hidden, 0, "toutes les rangées se montrent");
        assert!(
            l.reste.abs() < 1.0,
            "et rien de plus ({} pixels vides)",
            l.reste
        );
    }
}

/// **Ordonner et Pomodoro ont la hauteur de leur contenu** (TIROIR-2) : la leur est écrite une
/// fois, et cette épreuve tombe le jour où leur mise en page change sans elle.
#[test]
fn test_tiroir_2_ordonner_et_pomodoro_sont_pleins() {
    let typo = Typography::new();
    let vide = Store::new("essai");
    for scale in [1.0, 1.5] {
        let cadre = |tab: TabId| {
            compute_panel_layouts(&dock_with(tab), &vide, 1440.0, 2000.0, 78.0, scale)
                .into_iter()
                .find(|b| b.tab == tab)
                .expect("ouvert")
                .contenu(scale)
        };
        let o = cadre(TabId::Organize);
        let l = organize::layout_organize_panel(o, &typo);
        let champs = l.size_rect.y + l.size_rect.h;
        assert!(
            champs <= l.apply_rect.y,
            "les champs passent sous « Appliquer »"
        );
        assert!(
            l.apply_rect.y - champs <= 14.0 * scale,
            "un trou sous les champs"
        );
        let p = cadre(TabId::Pomodoro);
        let l = pomodoro::layout_pomodoro_panel(p, &typo);
        let bas = l
            .presets
            .iter()
            .map(|b| b.rect.y + b.rect.h)
            .fold(0.0, f32::max);
        assert!(
            (p.y + p.h - bas - 14.0 * scale).abs() < 0.01,
            "le Pomodoro finit à sa marge sous ses durées"
        );
    }
}

/// Deux tiroirs se croisent-ils ?
fn se_croisent(a: &PanelLayoutBox, b: &PanelLayoutBox) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

/// **TIROIR-3 — deux tiroirs ne se posent jamais l'un sur l'autre**, quand l'écran a la largeur
/// de leurs colonnes : tous ouverts aux écrans de bureau, six en 1440 × 900 (deux domaines), et
/// le cas de tous les jours (la Time Machine, Domaines, Ordonner) jusqu'aux plus petits.
/// Domaines s'ouvrait caché sous le Storyboard (sa capture du 17/11). Et nulle part un tiroir ne
/// sort de l'écran. **La limite, mesurée** (fiche 122) : six ouverts en 1440 × 900 avec huit
/// domaines, Domaines (494 pixels) ne tient sous aucun autre et ouvre une troisième colonne —
/// 1 450 pixels de colonnes pour 1 440 ; c'est le cas « trop étroit », plus bas.
#[test]
fn test_tiroir_3_deux_panneaux_ne_se_posent_jamais_l_un_sur_l_autre() {
    let (deux, huit) = (avec_des_domaines(2), avec_des_domaines(8));
    let mut courants = DockManager::new();
    for tab in [TabId::Temps, TabId::Domains, TabId::Organize] {
        courants.toggle_tab(tab);
    }
    let mut echecs = Vec::new();
    for (w, h, scale, dock, store) in [
        (1440.0, 900.0, 1.0, dock_complet(), &deux),
        (1920.0, 1080.0, 1.0, les_sept(), &huit),
        (2560.0, 1440.0, 1.25, les_sept(), &huit),
        (1366.0, 768.0, 1.0, courants.clone(), &huit),
        (1280.0, 720.0, 1.0, courants.clone(), &huit),
    ] {
        let boites = compute_panel_layouts(&dock, store, w, h, 78.0 * scale, scale);
        let mut fautes = Vec::new();
        for (i, a) in boites.iter().enumerate() {
            if a.x < 0.0 || a.x + a.width > w + 0.01 || a.y + a.height > h + 0.01 {
                fautes.push(format!("{:?} sort de l'écran", a.tab));
            }
            for b in &boites[i + 1..] {
                if se_croisent(a, b) {
                    fautes.push(format!("{:?} et {:?} se croisent", a.tab, b.tab));
                }
            }
        }
        if !fautes.is_empty() {
            let places: Vec<String> = boites
                .iter()
                .map(|b| format!("{:?} x {} y {} {}×{}", b.tab, b.x, b.y, b.width, b.height))
                .collect();
            echecs.push(format!(
                "{w} × {h} : {fautes:?}\n    {}",
                places.join("\n    ")
            ));
        }
    }
    assert!(echecs.is_empty(), "\n{}", echecs.join("\n"));
}

/// **Un écran trop étroit pour leurs colonnes** (sept tiroirs en 1280 × 720) : aucun ne sort de
/// l'écran, et là où deux se croisent, le clic va à celui qu'on voit — le dernier dessiné.
#[test]
fn test_tiroir_3_trop_etroit_le_clic_va_a_celui_qu_on_voit() {
    let store = avec_des_domaines(8);
    let ecran = ScreenFrame {
        width: 1280.0,
        height: 720.0,
        header_h: 78.0,
        scale: 1.0,
    };
    let dock = les_sept();
    let boites = places(&dock, &store, ecran);
    let mut croisements = 0;
    for (i, a) in boites.iter().enumerate() {
        assert!(
            a.x >= 0.0 && a.x + a.width <= 1280.01,
            "{:?} sort de l'écran",
            a.tab
        );
        for b in &boites[i + 1..] {
            if !se_croisent(a, b) {
                continue;
            }
            croisements += 1;
            // Le milieu de leur intersection : celui de dessus (`b`, dessiné après) le prend.
            let x = (a.x.max(b.x) + (a.x + a.width).min(b.x + b.width)) / 2.0;
            let y = (a.y.max(b.y) + (a.y + a.height).min(b.y + b.height)) / 2.0;
            let sous = panneau_sous(&dock, &store, ecran, Pointer { x, y }).map(|(t, _)| t);
            let dessus = boites
                .iter()
                .rev()
                .find(|c| c.seen().contains(x, y))
                .map(|c| c.tab);
            assert_eq!(sous, dessus, "{:?} sur {:?}", b.tab, a.tab);
        }
    }
    assert!(
        croisements > 0,
        "le cas n'est pas trop étroit : il n'éprouve rien"
    );
}
