//! Ce que le menu contextuel propose selon l'endroit, et ce qu'il refuse de proposer.

use super::*;
use crate::typography::Typography;
use glucose_core::types::{Annotation, BoardImage};

const SCREEN: (f32, f32) = (1440.0, 900.0);

fn store_with(images: usize, cartes: usize, selection: bool) -> Store {
    let mut store = Store::new("menu");
    let board = store.project.active_board_id.clone();
    for k in 0..images {
        store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    for k in 0..cartes {
        store.add_annotation(&board, Annotation::text(format!("a{k}"), 0.0, 0.0, "x"));
    }
    // `add_image` sélectionne ce qu'il pose : sans ce nettoyage, « le vide » n'en serait pas.
    store.clear_selection();
    if selection {
        store.set_selected_image_ids((0..images).map(|k| format!("i{k}")).collect());
        store.set_selected_annotation_ids((0..cartes).map(|k| format!("a{k}")).collect());
    }
    store
}

fn menu(store: &Store, at: (f32, f32)) -> ContextMenu {
    layout_context_menu(store, &Typography::new(), (at, None), SCREEN, 1.0).expect("un menu")
}

fn actions(m: &ContextMenu) -> Vec<MenuAction> {
    m.rows
        .iter()
        .filter_map(|r| match r {
            MenuRow::Item { action, .. } => Some(*action),
            MenuRow::Separator(_) => None,
        })
        .collect()
}

#[test]
fn test_sur_le_vide_le_menu_ne_propose_que_ce_qui_a_du_sens() {
    let store = store_with(2, 0, false);
    assert_eq!(
        actions(&menu(&store, (100.0, 100.0))),
        vec![MenuAction::Paste, MenuAction::SelectAll],
        "sans sélection, ni dupliquer ni supprimer n'ont d'objet"
    );
}

#[test]
fn test_sur_une_selection_il_propose_les_gestes_de_selection() {
    let store = store_with(2, 1, true);
    assert_eq!(
        actions(&menu(&store, (100.0, 100.0))),
        vec![
            MenuAction::Duplicate,
            MenuAction::ToggleLock,
            MenuAction::TrimBorders,
            MenuAction::ToFront,
            MenuAction::ToBack,
            MenuAction::Delete,
        ]
    );
}

/// Une carte ou un dossier ne porte pas de verrou : l'entrée disparaît plutôt que d'être
/// grisée — un menu qui offre ce qui n'existe pas est la faute R-33.
#[test]
fn test_sans_image_lentree_du_verrou_disparait() {
    let store = store_with(0, 2, true);
    let a = actions(&menu(&store, (100.0, 100.0)));
    assert!(!a.contains(&MenuAction::ToggleLock));
    assert!(a.contains(&MenuAction::Delete));
}

#[test]
fn test_lentree_du_verrou_dit_ce_quelle_fera() {
    let mut store = store_with(2, 0, true);
    let libelle = |m: &ContextMenu| {
        m.rows
            .iter()
            .find_map(|r| match r {
                MenuRow::Item {
                    action: MenuAction::ToggleLock,
                    label,
                    ..
                } => Some(*label),
                _ => None,
            })
            .expect("l'entrée du verrou")
    };
    assert_eq!(libelle(&menu(&store, (0.0, 0.0))), "Verrouiller");

    let board = store.project.active_board_id.clone();
    store.toggle_lock_selection(&board);
    assert_eq!(libelle(&menu(&store, (0.0, 0.0))), "Déverrouiller");
}

#[test]
fn test_chaque_entree_repond_en_son_centre() {
    let store = store_with(1, 1, true);
    let m = menu(&store, (200.0, 200.0));
    for row in &m.rows {
        if let MenuRow::Item { action, rect, .. } = row {
            let (x, y, w, h) = *rect;
            assert_eq!(
                hit_context_menu(&m, x + w / 2.0, y + h / 2.0),
                Some(*action),
                "{action:?} ne répond pas en son centre"
            );
        }
    }
}

/// Un filet n'est pas cliquable : viser entre deux groupes ne déclenche rien.
#[test]
fn test_un_separateur_ne_declenche_rien() {
    let store = store_with(1, 0, true);
    let m = menu(&store, (200.0, 200.0));
    let sy = m
        .rows
        .iter()
        .find_map(|r| match r {
            MenuRow::Separator(y) => Some(*y),
            _ => None,
        })
        .expect("un séparateur");
    assert_eq!(hit_context_menu(&m, m.rect.0 + m.rect.2 / 2.0, sy), None);
    assert!(
        covers(&m, m.rect.0 + m.rect.2 / 2.0, sy),
        "mais il le couvre"
    );
}

/// Ouvert près d'un bord, le menu bascule de l'autre côté du curseur plutôt que de sortir de
/// la fenêtre.
#[test]
fn test_le_menu_reste_dans_la_fenetre() {
    let store = store_with(2, 0, true);
    for at in [
        (SCREEN.0 - 4.0, SCREEN.1 - 4.0),
        (SCREEN.0 - 4.0, 10.0),
        (10.0, SCREEN.1 - 4.0),
    ] {
        let m = menu(&store, at);
        let (x, y, w, h) = m.rect;
        assert!(x >= 0.0 && x + w <= SCREEN.0, "déborde en x depuis {at:?}");
        assert!(y >= 0.0 && y + h <= SCREEN.1, "déborde en y depuis {at:?}");
    }
}

/// Les entrées se suivent du haut vers le bas, sans se chevaucher ni sortir du cadre.
#[test]
fn test_les_entrees_sempilent_sans_se_chevaucher() {
    let store = store_with(2, 1, true);
    let m = menu(&store, (300.0, 300.0));
    let (mx, my, mw, mh) = m.rect;
    let mut bas = my;
    for row in &m.rows {
        if let MenuRow::Item { rect, .. } = row {
            let (x, y, w, h) = *rect;
            assert!(y >= bas - 0.5, "{row:?} remonte");
            assert!(x >= mx && x + w <= mx + mw + 0.5, "{row:?} déborde en x");
            assert!(y + h <= my + mh + 0.5, "{row:?} déborde en bas");
            bas = y + h;
        }
    }
}
