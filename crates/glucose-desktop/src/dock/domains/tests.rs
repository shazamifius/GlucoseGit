//! Le panneau DOMAINES : DOM-UI-1 (aucune copie du document), DOM-UI-2 (une seule liste de
//! widgets) et DOM-UI-3 (le coût ne suit pas la taille du document).

use super::*;
use crate::dock::{compute_panel_layouts, handle_dock_click, DockManager, PanelClickResult, TabId};
use crate::params::ScreenFrame;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::types::{Annotation, BoardImage, Domain};
use tiny_skia::Pixmap;

const SCREEN: ScreenFrame =
    ScreenFrame { width: 1440.0, height: 900.0, header_h: 78.0, scale: 1.0 };

/// Un domaine tel que l'application en crée : chaque rang a sa couleur et son sigle.
fn domain_at(rank: usize, id: &str, name: &str) -> Domain {
    let (color, icon) = fresh_look(rank);
    Domain { id: id.into(), name: name.into(), color: color.into(), icon: icon.into(), created_at: 0 }
}

fn domain(id: &str, name: &str) -> Domain {
    domain_at(0, id, name)
}

fn text(id: &str) -> Annotation {
    Annotation::Text {
        id: id.into(),
        x: 0.0,
        y: 0.0,
        width: Some(240.0),
        height: Some(48.0),
        text: "n".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

/// Un document à trois domaines, un nœud texte et une image.
fn populated() -> Store {
    let mut store = Store::new("Panneau");
    for (rank, (id, name)) in [("d-1", "Science"), ("d-2", "Art"), ("d-3", "Jeu video")]
        .into_iter()
        .enumerate()
    {
        store.try_add_domain(domain_at(rank, id, name)).expect("identifiants distincts");
    }
    store.add_annotation("main", text("t-1"));
    if let Some(board) = store.active_board_mut() {
        board.images.push(BoardImage::new("img-1", 0.0, 0.0, 200.0, 150.0));
    }
    store
}

/// Le cadre du panneau DOMAINES tel que le dock le place réellement.
fn domains_frame(dock: &DockManager) -> ScaledRect {
    let boxes = compute_panel_layouts(dock, SCREEN.width, SCREEN.height, SCREEN.header_h, SCREEN.scale);
    let b = boxes
        .iter()
        .find(|b| b.tab == TabId::Domains)
        .expect("l'onglet DOMAINES doit être ouvert");
    ScaledRect { x: b.x, y: b.y, w: b.width, h: b.height, scale: SCREEN.scale }
}

fn dock_with_domains() -> DockManager {
    let mut dock = DockManager::new();
    dock.top_tabs = vec![TabId::Domains];
    dock
}

fn center(rect: WidgetRect) -> Pointer {
    Pointer { x: rect.x + rect.w / 2.0, y: rect.y + rect.h / 2.0 }
}

// ── DOM-UI-1 — le panneau n'a aucune liste à lui ────────────────────────────

/// R-47 en une assertion : le panneau affiche exactement ce que le document contient. Ajouter
/// un domaine au store le fait apparaître ; en retirer un le fait disparaître. Aucune écriture
/// dans le panneau n'est nécessaire, parce qu'il n'a rien où écrire.
#[test]
fn test_dom_ui_1_the_panel_shows_the_document_and_only_the_document() {
    let mut store = Store::new("Vue");
    let dock = dock_with_domains();
    let frame = domains_frame(&dock);

    assert!(layout_domains_panel(frame, &store, &dock.domains).rows.is_empty());

    store.try_add_domain(domain("d-1", "Science")).expect("catalogue vide");
    store.try_add_domain(domain("d-2", "Art")).expect("identifiant neuf");
    assert_eq!(layout_domains_panel(frame, &store, &dock.domains).rows.len(), 2);

    store.try_remove_domain("d-1").expect("d-1 est au catalogue");
    let rows = layout_domains_panel(frame, &store, &dock.domains).rows;
    assert_eq!(rows.len(), 1);
    assert_eq!(store.project.domains[rows[0].index].id, "d-2");
}

/// L'état d'interface ne retient que des gestes, jamais des données : après un `undo`, le
/// panneau n'a aucune vieille copie à exhiber.
#[test]
fn test_dom_ui_1_the_interface_state_holds_gestures_not_data() {
    let ui = DomainsUi::default();
    assert!(ui.pending_delete.is_none());
    assert!(ui.rename.is_none());

    let mut ui = DomainsUi {
        pending_delete: Some("d-1".into()),
        rename: Some(DomainRename {
            domain_id: "d-2".into(),
            entry: TextEntry::new("En cours"),
        }),
    };
    ui.reset();
    assert_eq!(ui, DomainsUi::default(), "un changement de document efface tout geste");
}

// ── DOM-UI-2 — dessin et clic lisent la même liste ──────────────────────────

/// Chaque widget rend l'intention qu'il annonce, et il la rend **là où il est dessiné**.
#[test]
fn test_dom_ui_2_every_widget_answers_at_the_place_it_is_drawn() {
    let store = populated();
    let dock = dock_with_domains();
    let frame = domains_frame(&dock);
    let layout = layout_domains_panel(frame, &store, &dock.domains);
    let row = &layout.rows[1];
    let id = store.project.domains[row.index].id.clone();

    let cases: Vec<(WidgetRect, DomainIntent)> = vec![
        (row.swatch, DomainIntent::CycleColor(id.clone())),
        (row.sigil, DomainIntent::CycleSigil(id.clone())),
        (row.name, DomainIntent::StartRename(id.clone())),
        (row.delete, DomainIntent::AskDelete(id.clone())),
        (row.unassign, DomainIntent::Unassign(id.clone())),
        (layout.add_button, DomainIntent::Create),
    ];
    for (rect, expected) in cases {
        assert_eq!(
            hit_domains_panel(&layout, &store, center(rect), true),
            Some(expected.clone()),
            "le widget {rect:?} devrait rendre {expected:?}"
        );
    }
}

#[test]
fn test_each_weight_step_assigns_its_own_weight() {
    let store = populated();
    let dock = dock_with_domains();
    let layout = layout_domains_panel(domains_frame(&dock), &store, &dock.domains);
    let row = &layout.rows[0];
    let id = store.project.domains[row.index].id.clone();

    for (step, rect) in row.steps.iter().enumerate() {
        assert_eq!(
            hit_domains_panel(&layout, &store, center(*rect), true),
            Some(DomainIntent::Assign { domain_id: id.clone(), weight: WEIGHT_STEPS[step] }),
            "palier {step}"
        );
    }
}

/// Aucun widget ne se recouvre : un clic ne peut jamais déclencher deux gestes.
#[test]
fn test_no_two_widgets_of_a_row_overlap() {
    let store = populated();
    let dock = dock_with_domains();
    let layout = layout_domains_panel(domains_frame(&dock), &store, &dock.domains);
    for row in &layout.rows {
        let mut rects = vec![row.swatch, row.sigil, row.name, row.delete, row.unassign];
        rects.extend(row.steps.iter().copied());
        for (i, a) in rects.iter().enumerate() {
            for b in rects.iter().skip(i + 1) {
                let separated = a.x + a.w <= b.x
                    || b.x + b.w <= a.x
                    || a.y + a.h <= b.y
                    || b.y + b.h <= a.y;
                assert!(separated, "{a:?} recouvre {b:?}");
            }
        }
    }
}

/// § 5.4 — sans sélection, les paliers et le retrait ne font rien plutôt que de simuler.
#[test]
fn test_without_a_selection_the_weight_buttons_do_nothing() {
    let store = populated();
    let dock = dock_with_domains();
    let layout = layout_domains_panel(domains_frame(&dock), &store, &dock.domains);
    let row = &layout.rows[0];

    assert_eq!(hit_domains_panel(&layout, &store, center(row.steps[2]), false), None);
    assert_eq!(hit_domains_panel(&layout, &store, center(row.unassign), false), None);
    // Le reste de la ligne, lui, reste utilisable : renommer ne demande pas de sélection.
    assert!(hit_domains_panel(&layout, &store, center(row.name), false).is_some());
}

/// La suppression demande confirmation : les deux réponses prennent la place des paliers, et
/// aucun palier ne répond plus tant que la question est posée.
#[test]
fn test_a_deletion_asks_before_it_acts() {
    let store = populated();
    let mut dock = dock_with_domains();
    let frame = domains_frame(&dock);

    let plain = layout_domains_panel(frame, &store, &dock.domains);
    assert!(plain.rows.iter().all(|r| r.confirm.is_none()));

    dock.domains.pending_delete = Some("d-2".into());
    let asking = layout_domains_panel(frame, &store, &dock.domains);
    assert!(asking.rows[0].confirm.is_none(), "seule la ligne visée pose la question");
    let (yes, no) = asking.rows[1].confirm.expect("d-2 doit demander confirmation");

    assert_eq!(
        hit_domains_panel(&asking, &store, center(yes), true),
        Some(DomainIntent::ConfirmDelete("d-2".into()))
    );
    assert_eq!(
        hit_domains_panel(&asking, &store, center(no), true),
        Some(DomainIntent::CancelDelete)
    );
    assert_eq!(
        hit_domains_panel(&asking, &store, center(asking.rows[1].steps[0]), true),
        None,
        "tant que la question est posée, les paliers ne répondent plus"
    );
}

// ── DOM-UI-3 — la hauteur du panneau borne le travail ───────────────────────

#[test]
fn test_a_panel_too_short_shows_what_fits_and_says_how_many_are_missing() {
    let mut store = Store::new("Beaucoup");
    for i in 0..40 {
        store
            .try_add_domain(domain(&format!("d-{i}"), &format!("Domaine {i}")))
            .expect("identifiants distincts");
    }
    let dock = dock_with_domains();
    let layout = layout_domains_panel(domains_frame(&dock), &store, &dock.domains);

    assert!(!layout.rows.is_empty(), "le panneau doit montrer ce qu'il peut");
    assert!(layout.rows.len() < 40, "il ne peut pas tout montrer");
    assert_eq!(layout.rows.len() + layout.hidden, 40, "aucun domaine n'est perdu du compte");
    let last = layout.rows.last().expect("au moins une ligne");
    assert!(
        last.unassign.y + last.unassign.h <= layout.add_button.y,
        "la dernière ligne déborde sur le bouton de création"
    );
}

// ── Les palettes d'édition ──────────────────────────────────────────────────

#[test]
fn test_cycling_a_colour_walks_the_palette_and_comes_back() {
    let mut colour = DOMAIN_PALETTE[0].to_string();
    let mut seen = vec![colour.clone()];
    for _ in 1..DOMAIN_PALETTE.len() {
        colour = next_color(&colour).to_string();
        assert!(!seen.contains(&colour), "{colour} revient trop tôt");
        seen.push(colour.clone());
    }
    assert_eq!(next_color(&colour), DOMAIN_PALETTE[0], "le tour doit se refermer");
    // Une couleur venue d'ailleurs repart du début plutôt que de bloquer le bouton.
    assert_eq!(next_color("#123456"), DOMAIN_PALETTE[0]);
    assert_eq!(next_color(""), DOMAIN_PALETTE[0]);
}

#[test]
fn test_cycling_a_sigil_walks_the_palette_and_comes_back() {
    let mut sigil = DOMAIN_SIGILS[0].to_string();
    for _ in 1..DOMAIN_SIGILS.len() {
        sigil = next_sigil(&sigil).to_string();
    }
    assert_eq!(next_sigil(&sigil), DOMAIN_SIGILS[0]);
    assert_eq!(next_sigil("inconnu"), DOMAIN_SIGILS[0]);
}

/// Deux domaines créés à la suite ne se ressemblent pas : sans cela, deux colonnes voisines de
/// la réglette porteraient la même couleur et le même sigle.
#[test]
fn test_two_domains_created_in_a_row_do_not_look_alike() {
    for rank in 0..DOMAIN_PALETTE.len() - 1 {
        assert_ne!(fresh_look(rank), fresh_look(rank + 1), "rangs {rank} et {}", rank + 1);
    }
}

// ── Le chemin complet depuis la souris (§ 7.6) ──────────────────────────────

/// Le panneau est **atteignable depuis l'application** : un clic aux coordonnées de l'écran
/// traverse `handle_dock_click` et ressort en intention.
#[test]
fn test_a_click_on_the_panel_reaches_the_intent_through_the_dock() {
    let store = populated();
    let mut dock = dock_with_domains();
    let typo = Typography::new();
    let layout = layout_domains_panel(domains_frame(&dock), &store, &dock.domains);

    let clicked = handle_dock_click(&mut dock, &store, &typo, SCREEN, center(layout.add_button));
    assert_eq!(clicked, Some(PanelClickResult::Domain(DomainIntent::Create)));

    let row = &layout.rows[0];
    let id = store.project.domains[row.index].id.clone();
    let mut store_with_selection = populated();
    store_with_selection.select_annotation("t-1".into(), false);
    let clicked = handle_dock_click(
        &mut dock,
        &store_with_selection,
        &typo,
        SCREEN,
        center(row.steps[4]),
    );
    assert_eq!(
        clicked,
        Some(PanelClickResult::Domain(DomainIntent::Assign { domain_id: id, weight: 1.0 }))
    );
}

/// Un clic dans le panneau mais à côté de tout widget est **consommé** et ne retombe pas sur
/// le canevas : cliquer entre deux lignes ne doit pas désélectionner.
#[test]
fn test_a_click_inside_the_panel_never_falls_through_to_the_canvas() {
    let store = populated();
    let mut dock = dock_with_domains();
    let typo = Typography::new();
    let frame = domains_frame(&dock);
    let empty_spot = Pointer { x: frame.x + frame.w / 2.0, y: frame.y + 2.0 };

    assert_eq!(
        handle_dock_click(&mut dock, &store, &typo, SCREEN, empty_spot),
        Some(PanelClickResult::Handled)
    );
}

// ── Rendu ───────────────────────────────────────────────────────────────────

/// Le panneau se dessine dans tous ses états sans sortir de son cadre ni du pixmap.
#[test]
fn test_the_panel_draws_in_every_state_within_its_own_frame() {
    let mut store = populated();
    store.select_annotation("t-1".into(), false);
    store.select_image("img-1".into(), true);
    let mut dock = dock_with_domains();
    let typo = Typography::new();
    let theme = Theme::dark();

    let states = [
        DomainsUi::default(),
        DomainsUi { pending_delete: Some("d-2".into()), rename: None },
        DomainsUi {
            pending_delete: None,
            rename: Some(DomainRename {
                domain_id: "d-1".into(),
                entry: TextEntry::new("Épistémologie"),
            }),
        },
    ];
    for (index, ui) in states.into_iter().enumerate() {
        dock.domains = ui;
        let frame = domains_frame(&dock);
        let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
        pixmap.fill(theme.bg_canvas);
        let before = pixmap.data().to_vec();
        render_domains_panel(
            &mut pixmap.as_mut(),
            &store,
            &dock.domains,
            &typo,
            &theme,
            frame,
            Pointer { x: frame.x + 20.0, y: frame.y + 60.0 },
        );
        assert_ne!(pixmap.data(), before.as_slice(), "état {index} : rien n'a été dessiné");
    }
}

/// La capture de référence du panneau, dans ses trois états.
#[test]
fn test_domains_panel_png_capture() {
    let dir = std::path::Path::new("target/domains-panel");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let mut store = populated();
    store.select_annotation("t-1".into(), false);
    store
        .try_assign_domain_to_node("main", "t-1", "d-1", 0.6)
        .expect("le nœud et le domaine existent");
    let mut dock = dock_with_domains();
    let typo = Typography::new();
    let theme = Theme::dark();

    for (name, ui) in [
        ("liste", DomainsUi::default()),
        ("confirmation", DomainsUi { pending_delete: Some("d-2".into()), rename: None }),
        (
            "renommage",
            DomainsUi {
                pending_delete: None,
                rename: Some(DomainRename {
                    domain_id: "d-3".into(),
                    entry: TextEntry::new("Conlang"),
                }),
            },
        ),
    ] {
        dock.domains = ui;
        let frame = domains_frame(&dock);
        let mut pixmap = Pixmap::new(520, 620).expect("pixmap du panneau");
        pixmap.fill(theme.bg_panel);
        let shifted = ScaledRect { x: 24.0, y: 24.0, ..frame };
        render_domains_panel(
            &mut pixmap.as_mut(),
            &store,
            &dock.domains,
            &typo,
            &theme,
            shifted,
            Pointer { x: -1.0, y: -1.0 },
        );
        pixmap.save_png(dir.join(format!("panneau-{name}.png"))).expect("écriture du png");
    }
}
