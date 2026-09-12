//! Le panneau DOMAINES branché sur le document, joué sur le **vrai** `GlucoseApp`.
//!
//! C'est la règle § 7.6 appliquée à R-47 : le noyau a beau être complet, il ne vaut rien tant
//! qu'aucun test ne prouve qu'un clic l'atteint. Chaque test part donc d'une intention telle
//! que le panneau la produit, et vérifie le document, la pile d'undo et la barre de toasts.

use super::*;
use crate::dock::domains::{
    hit_domains_panel, layout_domains_panel, DomainIntent, DomainsUi, WEIGHT_STEPS,
};
use crate::dock::{compute_panel_layouts, TabId};
use crate::params::{Pointer, ScaledRect, ScreenFrame};
use glucose_core::types::{Annotation, BoardImage};
use winit::keyboard::{ModifiersState, SmolStr};

const SCREEN: ScreenFrame =
    ScreenFrame { width: 1440.0, height: 900.0, header_h: 78.0, scale: 1.0 };

/// Une application dont l'onglet DOMAINES est ouvert, avec un nœud texte et une image.
fn app_with_panel() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.dock_manager.top_tabs = vec![TabId::Domains];
    let board = app.store.project.active_board_id.clone();
    // La hauteur d'une carte est celle de son texte (TEXT-FIT-1) : une carte créée
    // autrement serait recalée à l'ouverture, et le document relu différerait.
    let height = crate::renderer::card::text_card_fit_height(&app.renderer.typography, "Newton", 240.0);
    app.store.add_annotation(
        &board,
        Annotation::Text {
            id: "t-1".into(),
            x: 0.0,
            y: 0.0,
            width: Some(240.0),
            height: Some(height),
            text: "Newton".into(),
            font_size: None,
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    if let Some(b) = app.store.active_board_mut() {
        b.images.push(BoardImage::new("img-1", 0.0, 0.0, 200.0, 150.0));
    }
    app
}

/// Crée un domaine par le chemin de l'application et rend son identifiant.
fn create_domain(app: &mut GlucoseApp) -> String {
    app.apply_domain_intent(DomainIntent::Create);
    // La création ouvre la saisie du nom : la valider referme le geste.
    app.commit_domain_rename();
    app.store.project.domains.last().map(|d| d.id.clone()).expect("un domaine vient d'être créé")
}

fn key(named: NamedKey) -> Key {
    Key::Named(named)
}

fn character(text: &str) -> Key {
    Key::Character(SmolStr::new(text))
}

// ── Création ────────────────────────────────────────────────────────────────

/// R-47, refermé : le bouton « + Nouveau domaine » écrit **dans le document**.
#[test]
fn test_creating_a_domain_writes_it_into_the_document() {
    let mut app = app_with_panel();
    assert!(app.store.project.domains.is_empty());

    app.apply_domain_intent(DomainIntent::Create);

    assert_eq!(app.store.project.domains.len(), 1, "le domaine doit être dans le document");
    let created = &app.store.project.domains[0];
    assert!(!created.color.is_empty(), "un domaine neuf a une couleur");
    assert!(!created.icon.is_empty(), "un domaine neuf a un sigle");
    assert!(app.is_dirty(), "créer un domaine modifie le document");
    // La saisie du nom s'ouvre dans la foulée : le libellé par défaut est fait pour partir.
    assert_eq!(
        app.dock_manager.domains.rename.as_ref().map(|r| r.domain_id.as_str()),
        Some(created.id.as_str())
    );
}

/// § 2.5 — l'identifiant vient du générateur monotone, pas de la longueur d'un `Vec`. Le
/// `format!("domain-{}", len + 1)` d'avant produisait deux fois le même après une suppression.
#[test]
fn test_domain_ids_come_from_the_monotonic_generator_not_from_a_length() {
    let mut app = app_with_panel();
    let first = create_domain(&mut app);
    let second = create_domain(&mut app);
    assert_ne!(first, second);

    app.apply_domain_intent(DomainIntent::ConfirmDelete(second.clone()));
    let third = create_domain(&mut app);
    assert_ne!(third, first, "un identifiant ne se réutilise jamais");
    assert_ne!(third, second, "surtout pas celui qu'on vient de supprimer");
}

// ── Renommage ───────────────────────────────────────────────────────────────

#[test]
fn test_typing_a_name_and_pressing_enter_renames_the_domain() {
    let mut app = app_with_panel();
    app.apply_domain_intent(DomainIntent::Create);
    let id = app.store.project.domains[0].id.clone();

    // Le libellé par défaut est effacé puis remplacé, touche par touche.
    for _ in 0..40 {
        app.handle_domain_rename_input(&key(NamedKey::Backspace));
    }
    for letter in ["C", "o", "n", "l", "a", "n", "g"] {
        assert!(app.handle_domain_rename_input(&character(letter)), "« {letter} » non consommé");
    }
    assert!(app.handle_domain_rename_input(&key(NamedKey::Enter)));

    assert_eq!(app.store.domain(&id).expect("le domaine existe").name, "Conlang");
    assert!(app.dock_manager.domains.rename.is_none(), "la saisie doit se refermer");
}

#[test]
fn test_escape_abandons_a_rename_without_touching_the_document() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    let before = app.store.domain(&id).expect("le domaine existe").name.clone();

    app.apply_domain_intent(DomainIntent::StartRename(id.clone()));
    app.handle_domain_rename_input(&character("X"));
    app.handle_domain_rename_input(&key(NamedKey::Escape));

    assert!(app.dock_manager.domains.rename.is_none());
    assert_eq!(app.store.domain(&id).expect("le domaine existe").name, before);
}

#[test]
fn test_an_empty_name_is_refused_and_says_so() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    let before = app.store.domain(&id).expect("le domaine existe").name.clone();

    app.apply_domain_intent(DomainIntent::StartRename(id.clone()));
    for _ in 0..60 {
        app.handle_domain_rename_input(&key(NamedKey::Backspace));
    }
    app.handle_domain_rename_input(&key(NamedKey::Enter));

    assert_eq!(app.store.domain(&id).expect("le domaine existe").name, before);
    let toast = app.ui.current_toast.as_ref().expect("un toast doit expliquer le refus");
    assert!(toast.message.contains("besoin d'un nom"), "toast : {}", toast.message);
}

/// Un accord `Ctrl` valide la saisie et **ne la consomme pas** : sans cela, `Ctrl+S` serait
/// impossible tant qu'un nom est ouvert.
#[test]
fn test_a_control_chord_commits_the_name_and_falls_through() {
    let mut app = app_with_panel();
    app.apply_domain_intent(DomainIntent::Create);
    let id = app.store.project.domains[0].id.clone();
    for _ in 0..40 {
        app.handle_domain_rename_input(&key(NamedKey::Backspace));
    }
    for letter in ["A", "r", "t"] {
        app.handle_domain_rename_input(&character(letter));
    }

    app.modifiers = ModifiersState::CONTROL;
    assert!(
        !app.handle_domain_rename_input(&character("s")),
        "Ctrl+S doit descendre jusqu'aux raccourcis de fichier"
    );
    assert_eq!(app.store.domain(&id).expect("le domaine existe").name, "Art");
}

/// Aucune touche n'est consommée quand aucune saisie n'est ouverte : le panneau ne vole pas le
/// clavier au canevas.
#[test]
fn test_no_key_is_swallowed_while_no_name_is_being_typed() {
    let mut app = app_with_panel();
    for event in [character("a"), key(NamedKey::Enter), key(NamedKey::Escape)] {
        assert!(!app.handle_domain_rename_input(&event));
    }
}

// ── Couleur et sigle ────────────────────────────────────────────────────────

#[test]
fn test_clicking_the_swatch_and_the_sigil_changes_them_in_the_document() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    let before = app.store.domain(&id).expect("le domaine existe").clone();

    app.apply_domain_intent(DomainIntent::CycleColor(id.clone()));
    assert_ne!(app.store.domain(&id).expect("le domaine existe").color, before.color);

    app.apply_domain_intent(DomainIntent::CycleSigil(id.clone()));
    assert_ne!(app.store.domain(&id).expect("le domaine existe").icon, before.icon);
    assert_eq!(
        app.store.domain(&id).expect("le domaine existe").name,
        before.name,
        "changer la couleur ne doit pas toucher au nom"
    );
}

// ── Assignation ─────────────────────────────────────────────────────────────

#[test]
fn test_assigning_to_the_selection_reaches_annotations_and_images_alike() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    app.store.select_annotation("t-1".into(), false);
    app.store.select_image("img-1".into(), true);

    app.apply_domain_intent(DomainIntent::Assign { domain_id: id.clone(), weight: 0.6 });

    for node in ["t-1", "img-1"] {
        let carried = app.store.node_domains("main", node).expect("le nœud existe");
        assert_eq!(carried.len(), 1, "{node}");
        assert_eq!(carried[0].domain_id, id, "{node}");
        assert_eq!(carried[0].weight, 0.6, "{node}");
    }
}

/// DOM-APP-1 — assigner à toute une sélection est **un** geste, donc **une** entrée d'undo.
#[test]
fn test_one_assignment_gesture_is_one_undo_entry() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    app.store.select_annotation("t-1".into(), false);
    app.store.select_image("img-1".into(), true);
    let depth = app.store.undo_depth();

    app.apply_domain_intent(DomainIntent::Assign { domain_id: id, weight: 0.8 });
    assert_eq!(app.store.undo_depth(), depth + 1, "deux nœuds, une seule entrée");

    assert!(app.store.undo());
    for node in ["t-1", "img-1"] {
        assert!(
            app.store.node_domains("main", node).expect("le nœud existe").is_empty(),
            "{node} : un seul Ctrl+Z doit tout défaire"
        );
    }
}

#[test]
fn test_assigning_without_a_selection_changes_nothing_and_stacks_nothing() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    // `add_annotation` sélectionne le nœud qu'elle ajoute : il faut vider la sélection pour
    // poser la question que ce test pose.
    app.store.clear_selection();
    let depth = app.store.undo_depth();

    app.apply_domain_intent(DomainIntent::Assign { domain_id: id, weight: 1.0 });

    assert_eq!(app.store.undo_depth(), depth, "un geste sans effet ne s'annule pas");
    assert!(app.store.node_domains("main", "t-1").expect("le nœud existe").is_empty());
}

#[test]
fn test_removing_an_assignment_from_the_selection() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    app.store.select_annotation("t-1".into(), false);
    app.apply_domain_intent(DomainIntent::Assign { domain_id: id.clone(), weight: 0.4 });

    app.apply_domain_intent(DomainIntent::Unassign(id));

    assert!(app.store.node_domains("main", "t-1").expect("le nœud existe").is_empty());
}

#[test]
fn test_removing_an_assignment_nobody_carries_says_so_and_stacks_nothing() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    app.store.select_annotation("t-1".into(), false);
    let depth = app.store.undo_depth();

    app.apply_domain_intent(DomainIntent::Unassign(id));

    assert_eq!(app.store.undo_depth(), depth);
    let toast = app.ui.current_toast.as_ref().expect("un toast doit le dire");
    assert!(toast.message.contains("ne porte ce domaine"), "toast : {}", toast.message);
}

// ── Suppression ─────────────────────────────────────────────────────────────

#[test]
fn test_a_deletion_needs_its_confirmation_and_cascades_when_confirmed() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    app.store.select_annotation("t-1".into(), false);
    app.store.select_image("img-1".into(), true);
    app.apply_domain_intent(DomainIntent::Assign { domain_id: id.clone(), weight: 0.5 });

    app.apply_domain_intent(DomainIntent::AskDelete(id.clone()));
    assert_eq!(app.dock_manager.domains.pending_delete.as_deref(), Some(id.as_str()));
    assert!(app.store.domain(&id).is_some(), "demander n'est pas supprimer");

    app.apply_domain_intent(DomainIntent::CancelDelete);
    assert!(app.dock_manager.domains.pending_delete.is_none());
    assert!(app.store.domain(&id).is_some());

    app.apply_domain_intent(DomainIntent::AskDelete(id.clone()));
    app.apply_domain_intent(DomainIntent::ConfirmDelete(id.clone()));

    assert!(app.store.domain(&id).is_none());
    assert!(
        app.store.orphan_domain_references().is_empty(),
        "restes : {:?}",
        app.store.orphan_domain_references()
    );
    assert!(app.dock_manager.domains.pending_delete.is_none());
    let toast = app.ui.current_toast.as_ref().expect("un toast doit résumer la cascade");
    assert!(toast.message.contains("2 nœud(s)"), "toast : {}", toast.message);
}

// ── Le chemin complet : clic → document → disque → document ─────────────────

/// **L'aller-retour disque complet**, joué exactement comme l'utilisateur le fait : créer un
/// domaine depuis le panneau, le renommer, le colorer, l'assigner à la sélection, `Ctrl+S`,
/// puis `Ctrl+O` dans une autre instance.
#[test]
fn test_a_domain_created_by_clicking_survives_save_and_reopen() {
    let path = std::env::temp_dir().join("glucose-tests-domains");
    std::fs::create_dir_all(&path).expect("dossier temporaire");
    let path = path.join("domaines-aller-retour.glucose");

    let mut app = app_with_panel();
    let frame = {
        let boxes = compute_panel_layouts(
            &app.dock_manager,
            SCREEN.width,
            SCREEN.height,
            SCREEN.header_h,
            SCREEN.scale,
        );
        let b = boxes.iter().find(|b| b.tab == TabId::Domains).expect("onglet ouvert");
        ScaledRect { x: b.x, y: b.y, w: b.width, h: b.height, scale: SCREEN.scale }
    };

    // 1. Le clic sur « + Nouveau domaine », traduit par le panneau lui-même.
    let layout = layout_domains_panel(frame, &app.store, &app.dock_manager.domains);
    let add = layout.add_button;
    let intent = hit_domains_panel(
        &layout,
        &app.store,
        Pointer { x: add.x + add.w / 2.0, y: add.y + add.h / 2.0 },
        false,
    )
    .expect("le bouton doit répondre");
    app.apply_domain_intent(intent);

    // 2. Le nom, tapé.
    for _ in 0..40 {
        app.handle_domain_rename_input(&key(NamedKey::Backspace));
    }
    for letter in ["S", "c", "i", "e", "n", "c", "e"] {
        app.handle_domain_rename_input(&character(letter));
    }
    app.handle_domain_rename_input(&key(NamedKey::Enter));
    let id = app.store.project.domains[0].id.clone();
    app.apply_domain_intent(DomainIntent::CycleColor(id.clone()));
    let expected = app.store.domain(&id).expect("le domaine existe").clone();

    // 3. L'assignation, au dernier palier, sur les deux genres de nœud.
    app.store.select_annotation("t-1".into(), false);
    app.store.select_image("img-1".into(), true);
    let weight = WEIGHT_STEPS[2];
    app.apply_domain_intent(DomainIntent::Assign { domain_id: id.clone(), weight });

    // 4. Ctrl+S, puis Ctrl+O dans une autre instance.
    app.save_to(path.clone());
    assert!(!app.is_dirty(), "l'enregistrement doit effacer le marqueur");

    let mut reopened = GlucoseApp::new();
    reopened.open_from(path.clone());

    let restored = reopened.store.domain(&id).expect("le domaine doit être revenu");
    assert_eq!(restored, &expected, "le domaine relu diffère de celui qui a été écrit");
    for node in ["t-1", "img-1"] {
        let carried = reopened.store.node_domains("main", node).expect("le nœud existe");
        assert_eq!(carried.len(), 1, "{node}");
        assert_eq!(carried[0].domain_id, id, "{node}");
        assert_eq!(carried[0].weight, weight, "{node}");
    }
    assert!(reopened.store.orphan_domain_references().is_empty());
    assert_eq!(reopened.store.project, app.store.project, "le document entier doit revenir");

    std::fs::remove_file(&path).expect("nettoyage");
}

/// Ouvrir un autre document abandonne les gestes en cours : leurs identifiants ne désignent
/// plus rien (DOM-UI-1).
#[test]
fn test_opening_a_document_drops_any_gesture_in_flight() {
    let path = std::env::temp_dir().join("glucose-tests-domains");
    std::fs::create_dir_all(&path).expect("dossier temporaire");
    let path = path.join("domaines-changement.glucose");

    let mut app = app_with_panel();
    create_domain(&mut app);
    app.save_to(path.clone());

    app.dock_manager.domains.pending_delete = Some("domain-de-l-autre-document".into());
    app.open_from(path.clone());

    assert_eq!(app.dock_manager.domains, DomainsUi::default());
    std::fs::remove_file(&path).expect("nettoyage");
}
