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
    layout_context_menu(store, &Typography::new(), (at, None), SCREEN, 1.0, false).expect("un menu")
}

fn actions(m: &ContextMenu) -> Vec<MenuAction> {
    m.rows
        .iter()
        .filter_map(|r| match r {
            MenuRow::Item { action, .. } => Some(*action),
            MenuRow::Separator { .. } => None,
        })
        .collect()
}

#[test]
fn test_sur_le_vide_le_menu_ne_propose_que_ce_qui_a_du_sens() {
    let store = store_with(2, 0, false);
    assert_eq!(
        actions(&menu(&store, (100.0, 100.0))),
        vec![
            MenuAction::Annuler,
            MenuAction::AjouterDesImages,
            MenuAction::Paste,
            MenuAction::SelectAll,
            MenuAction::ToutCadrer,
            MenuAction::Signets,
            MenuAction::NouveauDocument,
            MenuAction::OuvrirUnDocument,
            MenuAction::Enregistrer,
            MenuAction::EnregistrerSous,
            MenuAction::Exporter,
            MenuAction::ModeReference
        ],
        "sans sélection, ni dupliquer ni supprimer n'ont d'objet"
    );
}

#[test]
fn test_sur_une_selection_il_propose_les_gestes_de_selection() {
    let store = store_with(2, 1, true);
    assert_eq!(
        actions(&menu(&store, (100.0, 100.0))),
        vec![
            MenuAction::Annuler,
            MenuAction::Copier,
            MenuAction::Couper,
            MenuAction::Duplicate,
            MenuAction::CadrerLaSelection,
            MenuAction::ToggleLock,
            MenuAction::TrimBorders,
            MenuAction::StoryboardDeLaSelection,
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
            MenuRow::Separator { y, .. } => Some(*y),
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

/// **Une image seule offre de l'enregistrer** (fiche 51 § 3) ; deux images, ou une image et une
/// carte, non : un fichier n'en enregistre qu'une. La copier comme une image n'est plus un geste à
/// part : « Copier » emporte l'image elle-même (COPIER-1, fiche 59).
#[test]
fn test_une_image_seule_offre_de_l_enregistrer() {
    let seule = actions(&menu(&store_with(1, 0, true), (100.0, 100.0)));
    assert!(seule.contains(&MenuAction::EnregistrerLImage));
    assert!(seule.contains(&MenuAction::Copier));
    for (images, cartes) in [(2, 0), (1, 1), (0, 1)] {
        let a = actions(&menu(&store_with(images, cartes, true), (100.0, 100.0)));
        assert!(
            !a.contains(&MenuAction::EnregistrerLImage),
            "{images} image(s), {cartes} carte(s)"
        );
        assert!(a.contains(&MenuAction::Copier) && a.contains(&MenuAction::Couper));
    }
}

/// **Ouvert au doigt, le menu prend la taille d'un doigt** (fiche 56) : chaque entrée fait
/// 48 points de haut — à toute densité d'écran — et tait son raccourci, faute de clavier.
#[test]
fn test_au_doigt_chaque_entree_fait_quarante_huit_points_sans_raccourci() {
    let store = store_with(0, 0, false);
    for densite in [1.0f32, 2.0, 3.0] {
        let m = layout_context_menu(
            &store,
            &Typography::new(),
            ((100.0, 100.0), None),
            SCREEN,
            densite,
            true,
        )
        .expect("un menu");
        for row in &m.rows {
            if let MenuRow::Item { rect, shortcut, .. } = row {
                assert_eq!(rect.3, 48.0 * densite, "une entrée à la taille du doigt");
                assert!(
                    shortcut.is_empty(),
                    "aucun raccourci au doigt : « {shortcut} »"
                );
            }
        }
        assert_eq!(m.font, 16.0 * densite);
    }
    // À la souris, rien ne change : 24 points, et les raccourcis.
    let m = menu(&store, (100.0, 100.0));
    assert!(m.rows.iter().any(|r| matches!(
        r,
        MenuRow::Item { rect, shortcut, .. } if rect.3 == 24.0 && *shortcut == "Ctrl+V"
    )));
}

/// **Un dossier choisi a son menu, réduit à ce qui le prend** (DOSSIER-1, fiche 60) : il
/// recevait celui du vide, sans « Supprimer ». Copier, dupliquer et l'ordre ne portent pas les
/// dossiers : les offrir mentirait.
#[test]
fn test_dossier_1_un_dossier_seul_offre_supprimer_et_rien_d_autre() {
    let mut store = store_with(1, 0, false);
    let board = store.project.active_board_id.clone();
    store.create_folder(
        &board,
        glucose_core::types::CanvasFolder::new("d", "Dossier", ""),
    );
    store.select_folder("d".into());
    assert_eq!(
        actions(&menu(&store, (100.0, 100.0))),
        vec![MenuAction::Annuler, MenuAction::Delete]
    );
}

/// **ACCES-1 — Annuler et Rétablir n'existent que s'ils ont un objet**, en tête de tout menu du
/// canevas : sans clavier — le doigt, la souris seule —, rien ne défaisait un geste au bureau.
#[test]
fn test_acces_1_annuler_et_retablir_n_existent_que_s_ils_ont_un_objet() {
    let neuf = Store::new("neuf");
    let a = actions(&menu(&neuf, (100.0, 100.0)));
    assert!(!a.contains(&MenuAction::Annuler) && !a.contains(&MenuAction::Retablir));

    let mut store = store_with(1, 0, true);
    let a = actions(&menu(&store, (100.0, 100.0)));
    assert_eq!(
        a.first(),
        Some(&MenuAction::Annuler),
        "en tete, sur une selection"
    );
    assert!(!a.contains(&MenuAction::Retablir), "rien a refaire");
    assert!(store.undo());
    let a = actions(&menu(&store, (100.0, 100.0)));
    assert!(
        a.contains(&MenuAction::Retablir),
        "le geste defait se refait"
    );
}

/// **ACCES-1 — chaque geste du clavier a son entrée.** Tout cadrer, les signets, enregistrer,
/// enregistrer sous, exporter ne s'atteignaient qu'au clavier — donc jamais au téléphone ; la
/// sélection se cadre aussi d'une entrée.
#[test]
fn test_acces_1_chaque_geste_du_clavier_a_son_entree() {
    let vide = actions(&menu(&store_with(1, 0, false), (100.0, 100.0)));
    for attendue in [
        MenuAction::ToutCadrer,
        MenuAction::Signets,
        MenuAction::Enregistrer,
        MenuAction::EnregistrerSous,
        MenuAction::Exporter,
        MenuAction::AjouterDesImages,
    ] {
        assert!(
            vide.contains(&attendue),
            "{attendue:?} manque au menu du vide"
        );
    }
    let selection = actions(&menu(&store_with(1, 0, true), (100.0, 100.0)));
    assert!(selection.contains(&MenuAction::CadrerLaSelection));
}

/// **ACCES-1 — un menu plus haut que l'écran se range en colonnes, sans rien perdre.** Au doigt,
/// un écran de cinq pouces ne tient qu'une douzaine d'entrées : le bas sortait de l'écran. Les
/// colonnes tiennent dans l'écran, aucune entrée ne se chevauche, toutes sont là ; et sur un
/// écran qui le tient, le menu reste d'une seule colonne.
#[test]
fn test_acces_1_un_menu_trop_haut_se_range_en_colonnes() {
    let store = store_with(1, 0, false);
    let typo = Typography::new();
    let haut = layout_context_menu(&store, &typo, ((10.0, 10.0), None), SCREEN, 1.0, true)
        .expect("un menu");
    // Trois cents pixels : sous les deux premiers groupes, le troisième ne tient plus entier —
    // sans la règle des groupes, la colonne se couperait au milieu de lui.
    let bas = (SCREEN.0, 300.0);
    let range =
        layout_context_menu(&store, &typo, ((10.0, 10.0), None), bas, 1.0, true).expect("un menu");
    assert_eq!(actions(&range), actions(&haut), "aucune entree perdue");
    let colonnes = |m: &ContextMenu| {
        let mut x: Vec<i32> = m
            .rows
            .iter()
            .filter_map(|r| match r {
                MenuRow::Item { rect, .. } => Some(rect.0 as i32),
                MenuRow::Separator { .. } => None,
            })
            .collect();
        x.dedup();
        x.sort_unstable();
        x.dedup();
        x.len()
    };
    assert_eq!(colonnes(&haut), 1, "un ecran qui le tient : une colonne");
    assert!(colonnes(&range) > 1, "trop haut : des colonnes");
    let (_, my, _, mh) = range.rect;
    assert!(my >= 0.0 && my + mh <= bas.1, "le menu tient dans l'ecran");
    // Chaque groupe tient dans une colonne ici : aucune colonne ne commence au milieu d'un
    // groupe. Le début d'un groupe, c'est la première entrée, ou celle qui suit un filet.
    let mut debuts = Vec::new();
    let mut apres_un_filet = true;
    for r in &haut.rows {
        match r {
            MenuRow::Item { action, .. } if apres_un_filet => {
                debuts.push(*action);
                apres_un_filet = false;
            }
            MenuRow::Item { .. } => {}
            MenuRow::Separator { .. } => apres_un_filet = true,
        }
    }
    let mut vues = std::collections::HashSet::new();
    for r in &range.rows {
        if let MenuRow::Item { action, rect, .. } = r {
            if vues.insert(rect.0 as i32) {
                assert!(
                    debuts.contains(action),
                    "{action:?} commence une colonne au milieu de son groupe"
                );
            }
        }
    }
    // Un filet sépare deux groupes d'une même colonne : jamais il ne la commence ni ne la finit.
    let x_de = |r: &MenuRow| match r {
        MenuRow::Item { rect, .. } => rect.0,
        MenuRow::Separator { x, .. } => *x,
    };
    for (i, r) in range.rows.iter().enumerate() {
        if let MenuRow::Separator { x, .. } = r {
            let avant = i.checked_sub(1).map(|j| &range.rows[j]);
            let apres = range.rows.get(i + 1);
            assert!(
                avant.is_some_and(|a| x_de(a) == *x) && apres.is_some_and(|b| x_de(b) == *x),
                "un filet au bord de sa colonne : {r:?}"
            );
        }
    }
    let rects: Vec<_> = range
        .rows
        .iter()
        .filter_map(|r| match r {
            MenuRow::Item { rect, .. } => Some(*rect),
            MenuRow::Separator { .. } => None,
        })
        .collect();
    for (i, a) in rects.iter().enumerate() {
        assert!(a.1 + a.3 <= bas.1 + 0.5, "{a:?} sort par le bas");
        for b in &rects[i + 1..] {
            let disjoints = a.0 + a.2 <= b.0 + 0.5
                || b.0 + b.2 <= a.0 + 0.5
                || a.1 + a.3 <= b.1 + 0.5
                || b.1 + b.3 <= a.1 + 0.5;
            assert!(disjoints, "{a:?} chevauche {b:?}");
        }
    }
}

/// **« Lire le storyboard » n'existe que s'il y a des cases à lire** (fiche 67), juste après
/// les signets — au clic droit comme au doigt.
#[test]
fn test_lire_le_storyboard_n_existe_que_s_il_y_a_des_cases() {
    let mut store = store_with(0, 0, false);
    assert!(!actions(&menu(&store, (100.0, 100.0))).contains(&MenuAction::LireLeStoryboard));
    let board = store.project.active_board_id.clone();
    let grille = glucose_core::store::storyboard::Grille::default();
    store.poser_le_storyboard(&board, grille, (0.0, 0.0));
    store.clear_selection();
    let offertes = actions(&menu(&store, (100.0, 100.0)));
    let signets = offertes.iter().position(|a| *a == MenuAction::Signets);
    let lire = offertes
        .iter()
        .position(|a| *a == MenuAction::LireLeStoryboard);
    assert_eq!(
        lire,
        signets.map(|s| s + 1),
        "après les signets : {offertes:?}"
    );
    // Ses planches, juste après (P2.7, fiche 68) — et pas sans cases.
    let planches = offertes
        .iter()
        .position(|a| *a == MenuAction::ExporterLesPlanches);
    assert_eq!(
        planches,
        lire.map(|l| l + 1),
        "après la lecture : {offertes:?}"
    );
}
