//! Le système de domaines — invariants DOM-1, DOM-2, DOM-3 et R-47.
//!
//! Le constat R-47 tenait en une phrase : le noyau savait presque tout faire, et ce qu'il
//! faisait, il le faisait à moitié. `remove_domain` retirait le domaine du catalogue et
//! laissait chaque nœud pointer vers lui ; `update_domain` ne changeait que le nom ; aucune
//! désassignation n'existait ; `weight` n'était jamais validé ; les images étaient
//! inatteignables ; un identifiant en double passait ; et un `remove_domain` sur un
//! identifiant inconnu empilait quand même une entrée d'annulation.
//!
//! Cette suite tient un test par point réparé, plus la preuve centrale : **après une
//! suppression, aucune référence orpheline ne subsiste nulle part**.

use glucose_core::error::CoreError;
use glucose_core::store::{DomainPatch, Store};
use glucose_core::types::{Annotation, Board, BoardImage, Domain, DomainAssignment};

// ── Échafaudage ─────────────────────────────────────────────────────────────

fn domain(id: &str, name: &str) -> Domain {
    Domain {
        id: id.into(),
        name: name.into(),
        color: "#60a5fa".into(),
        icon: "SCI".into(),
        created_at: 0,
    }
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

fn sticky(id: &str) -> Annotation {
    Annotation::Sticky {
        id: id.into(),
        x: 0.0,
        y: 0.0,
        width: Some(160.0),
        height: Some(120.0),
        text: "s".into(),
        font_size: None,
        color: None,
        bg_color: None,
        cursor_pos: None,
        operator: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn membrane(id: &str) -> Annotation {
    Annotation::Membrane {
        id: id.into(),
        x: 0.0,
        y: 0.0,
        width: 600.0,
        height: 400.0,
        color: None,
        text: None,
        mode: Default::default(),
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

/// Un document à deux tableaux dont **chaque genre de porteur** est représenté : annotation
/// texte, pense-bête, membrane et image, sur le tableau actif comme sur l'autre.
fn populated_store() -> Store {
    let mut store = Store::new("Domaines");
    store.project.boards.push(Board::new("annexe", "Annexe"));
    for (board, ids) in [("main", ["t-1", "s-1", "m-1"]), ("annexe", ["t-2", "s-2", "m-2"])] {
        store.add_annotation(board, text(ids[0]));
        store.add_annotation(board, sticky(ids[1]));
        store.add_annotation(board, membrane(ids[2]));
    }
    for (board, id) in [("main", "img-1"), ("annexe", "img-2")] {
        let target = store.project.boards.iter_mut().find(|b| b.id == board);
        if let Some(b) = target {
            b.images.push(BoardImage::new(id, 0.0, 0.0, 200.0, 150.0));
        }
    }
    store
}

/// Tous les nœuds du document, tableau par tableau — le miroir de lecture de DOM-1.
const ALL_NODES: [(&str, &str); 8] = [
    ("main", "t-1"),
    ("main", "s-1"),
    ("main", "m-1"),
    ("main", "img-1"),
    ("annexe", "t-2"),
    ("annexe", "s-2"),
    ("annexe", "m-2"),
    ("annexe", "img-2"),
];

// ── 1. La cascade de suppression ────────────────────────────────────────────

/// **La preuve demandée.** Créer un domaine, l'assigner à un nœud avec un poids, supprimer le
/// domaine, et vérifier qu'aucune assignation orpheline ne subsiste nulle part.
#[test]
fn test_dom1_removing_a_domain_leaves_no_orphan_reference_anywhere() {
    let mut store = populated_store();
    store.try_add_domain(domain("d-sci", "Science")).expect("catalogue vide");
    store.try_add_domain(domain("d-art", "Art")).expect("identifiant neuf");

    for (board, node) in ALL_NODES {
        store
            .try_assign_domain_to_node(board, node, "d-sci", 0.75)
            .unwrap_or_else(|e| panic!("{board}/{node} : {e}"));
        store
            .try_assign_domain_to_node(board, node, "d-art", 0.25)
            .unwrap_or_else(|e| panic!("{board}/{node} : {e}"));
    }
    assert_eq!(store.domain_assignment_count("d-sci"), ALL_NODES.len());

    let detached = store.try_remove_domain("d-sci").expect("d-sci est au catalogue");

    assert_eq!(detached, ALL_NODES.len(), "la cascade doit toucher tous les porteurs");
    assert_eq!(store.domain_assignment_count("d-sci"), 0);
    assert!(
        store.orphan_domain_references().is_empty(),
        "références orphelines : {:?}",
        store.orphan_domain_references()
    );
    // Le domaine survivant n'a pas été emporté par la cascade.
    for (board, node) in ALL_NODES {
        let carried = store.node_domains(board, node).expect("nœud présent");
        assert_eq!(carried.len(), 1, "{board}/{node}");
        assert_eq!(carried[0].domain_id, "d-art");
        assert_eq!(carried[0].weight, 0.25);
    }
}

/// La même preuve, énoncée sur le document entier plutôt que domaine par domaine : quel que
/// soit l'ordre des suppressions, `orphan_domain_references` reste vide.
#[test]
fn test_dom1_no_order_of_removal_can_leave_a_dangling_reference() {
    let mut store = populated_store();
    for (i, id) in ["d-1", "d-2", "d-3"].iter().enumerate() {
        store.try_add_domain(domain(id, id)).expect("identifiants distincts");
        for (board, node) in ALL_NODES {
            store
                .try_assign_domain_to_node(board, node, id, (i as f64 + 1.0) / 4.0)
                .expect("nœud et domaine présents");
        }
    }

    for id in ["d-2", "d-1", "d-3"] {
        store.try_remove_domain(id).expect("au catalogue");
        assert!(
            store.orphan_domain_references().is_empty(),
            "après suppression de {id} : {:?}",
            store.orphan_domain_references()
        );
    }
    for (board, node) in ALL_NODES {
        assert!(store.node_domains(board, node).expect("nœud présent").is_empty());
    }
}

#[test]
fn test_removing_an_unknown_domain_is_a_named_error() {
    let mut store = populated_store();
    assert_eq!(
        store.try_remove_domain("fantome"),
        Err(CoreError::DomainNotFound("fantome".into()))
    );
}

// ── 2. Une mise à jour touche les trois champs ──────────────────────────────

#[test]
fn test_update_domain_edits_name_color_and_icon() {
    let mut store = Store::new("P");
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");

    store
        .try_update_domain(
            "d",
            DomainPatch::new().with_name("Conlang").with_color("#f472b6").with_icon("LNG"),
        )
        .expect("d est au catalogue");

    let updated = store.domain("d").expect("d est au catalogue");
    assert_eq!(updated.name, "Conlang");
    assert_eq!(updated.color, "#f472b6");
    assert_eq!(updated.icon, "LNG");
}

#[test]
fn test_update_domain_leaves_untouched_fields_alone() {
    let mut store = Store::new("P");
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    store
        .try_update_domain("d", DomainPatch::new().with_color("#34d399"))
        .expect("d est au catalogue");

    let updated = store.domain("d").expect("d est au catalogue");
    assert_eq!(updated.name, "Science", "le nom ne devait pas bouger");
    assert_eq!(updated.icon, "SCI", "l'icône ne devait pas bouger");
    assert_eq!(updated.color, "#34d399");
}

#[test]
fn test_updating_an_unknown_domain_is_a_named_error() {
    let mut store = Store::new("P");
    assert_eq!(
        store.try_update_domain("fantome", DomainPatch::new().with_name("X")),
        Err(CoreError::DomainNotFound("fantome".into()))
    );
}

// ── 3. La désassignation ────────────────────────────────────────────────────

#[test]
fn test_a_domain_can_be_unassigned_from_a_node() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    store.try_assign_domain_to_node("main", "t-1", "d", 0.5).expect("t-1 existe");
    assert_eq!(store.node_domains("main", "t-1").expect("t-1").len(), 1);

    store
        .try_unassign_domain_from_node("main", "t-1", "d")
        .expect("t-1 porte d");

    assert!(store.node_domains("main", "t-1").expect("t-1").is_empty());
    // Le domaine reste au catalogue : retirer une assignation n'est pas supprimer un domaine.
    assert!(store.domain("d").is_some());
}

#[test]
fn test_unassigning_a_domain_the_node_does_not_carry_is_a_named_error() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    assert_eq!(
        store.try_unassign_domain_from_node("main", "t-1", "d"),
        Err(CoreError::DomainNotAssigned {
            node_id: "t-1".into(),
            domain_id: "d".into(),
        })
    );
    assert_eq!(
        store.try_unassign_domain_from_node("main", "fantome", "d"),
        Err(CoreError::NodeNotFound("fantome".into()))
    );
}

// ── 4. La pondération est validée ───────────────────────────────────────────

#[test]
fn test_an_invalid_weight_never_enters_the_model() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");

    for bad in [-0.001_f64, 1.001, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let refused = store.try_assign_domain_to_node("main", "t-1", "d", bad);
        let Err(CoreError::InvalidWeight(_)) = refused else {
            panic!("le poids {bad} aurait dû être refusé, obtenu : {refused:?}");
        };
    }
    assert!(
        store.node_domains("main", "t-1").expect("t-1").is_empty(),
        "aucun poids aberrant ne doit avoir atteint le nœud"
    );
    // Les deux bornes, elles, sont admises.
    for good in [0.0, 1.0] {
        assert!(store.try_assign_domain_to_node("main", "t-1", "d", good).is_ok());
    }
}

/// Un poids invalide ne doit pas non plus laisser d'entrée d'annulation (DOM-3) : la
/// validation précède `push_undo`.
#[test]
fn test_a_refused_weight_leaves_the_undo_stack_untouched() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    let depth = store.undo_depth();
    assert!(store.try_assign_domain_to_node("main", "t-1", "d", f64::NAN).is_err());
    assert_eq!(store.undo_depth(), depth);
}

// ── 5. Les images sont des nœuds comme les autres ───────────────────────────

#[test]
fn test_an_image_carries_domains_just_like_an_annotation() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");

    store
        .try_assign_domain_to_node("main", "img-1", "d", 0.6)
        .expect("img-1 est une image du tableau main");

    let carried = store.node_domains("main", "img-1").expect("img-1");
    assert_eq!(carried, &[DomainAssignment { domain_id: "d".into(), weight: 0.6 }]);

    store
        .try_unassign_domain_from_node("main", "img-1", "d")
        .expect("img-1 porte d");
    assert!(store.node_domains("main", "img-1").expect("img-1").is_empty());
}

#[test]
fn test_reassigning_a_domain_overwrites_its_weight_without_duplicating_it() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    store.try_assign_domain_to_node("main", "t-1", "d", 0.2).expect("t-1");
    store.try_assign_domain_to_node("main", "t-1", "d", 0.9).expect("t-1");

    let carried = store.node_domains("main", "t-1").expect("t-1");
    assert_eq!(carried.len(), 1, "une réassignation n'ajoute pas une ligne");
    assert_eq!(carried[0].weight, 0.9);
}

// ── 6. Pas d'identifiant en double ──────────────────────────────────────────

#[test]
fn test_a_duplicate_domain_id_is_refused() {
    let mut store = Store::new("P");
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    assert_eq!(
        store.try_add_domain(domain("d", "Autre chose")),
        Err(CoreError::DuplicateDomainId("d".into()))
    );
    assert_eq!(store.project.domains.len(), 1);
    assert_eq!(store.domain("d").expect("d").name, "Science");
}

// ── 7. L'annulation est juste (DOM-3) ───────────────────────────────────────

#[test]
fn test_dom3_a_no_op_leaves_no_undo_entry() {
    let mut store = Store::new("P");
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    let depth = store.undo_depth();

    // Suppression d'un domaine inconnu : rien n'a changé, rien ne s'empile.
    assert!(store.try_remove_domain("fantome").is_err());
    assert_eq!(store.undo_depth(), depth, "remove_domain sur un id inconnu");

    // Mise à jour d'un domaine inconnu, puis patch vide, puis patch identique.
    assert!(store.try_update_domain("fantome", DomainPatch::new().with_name("X")).is_err());
    assert!(store.try_update_domain("d", DomainPatch::new()).is_ok());
    assert!(store.try_update_domain("d", DomainPatch::new().with_name("Science")).is_ok());
    assert_eq!(store.undo_depth(), depth, "un patch sans effet ne s'annule pas");

    // Identifiant en double : refusé avant tout instantané.
    assert!(store.try_add_domain(domain("d", "Science")).is_err());
    assert_eq!(store.undo_depth(), depth, "un doublon refusé ne s'annule pas");

    // Et un vrai changement, lui, s'empile une fois.
    store
        .try_update_domain("d", DomainPatch::new().with_name("Art"))
        .expect("d est au catalogue");
    assert_eq!(store.undo_depth(), depth + 1);
}

#[test]
fn test_dom3_undo_after_a_removal_restores_the_domain_and_all_its_assignments() {
    let mut store = populated_store();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    for (board, node) in ALL_NODES {
        store
            .try_assign_domain_to_node(board, node, "d", 0.4)
            .expect("nœud et domaine présents");
    }
    let before = store.project.clone();

    store.try_remove_domain("d").expect("d est au catalogue");
    assert!(store.domain("d").is_none());

    assert!(store.undo(), "la suppression doit être annulable");

    assert!(store.domain("d").is_some(), "le domaine doit revenir");
    assert_eq!(store.project, before, "l'annulation doit restaurer le document entier");
    for (board, node) in ALL_NODES {
        let carried = store.node_domains(board, node).expect("nœud présent");
        assert_eq!(carried.len(), 1, "{board}/{node} a perdu son assignation");
        assert_eq!(carried[0].weight, 0.4);
    }
}

#[test]
fn test_one_gesture_is_one_undo_entry() {
    let mut store = populated_store();
    let depth = store.undo_depth();
    store.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    assert_eq!(store.undo_depth(), depth + 1);
    store.try_assign_domain_to_node("main", "t-1", "d", 0.5).expect("t-1");
    assert_eq!(store.undo_depth(), depth + 2);
    store.try_unassign_domain_from_node("main", "t-1", "d").expect("t-1 porte d");
    assert_eq!(store.undo_depth(), depth + 3);
}

// ── 8. L'ordre des assignations suit le catalogue (DOM-2) ───────────────────

#[test]
fn test_dom2_assignments_follow_the_catalogue_order_whatever_the_click_order() {
    let mut store = populated_store();
    for id in ["d-1", "d-2", "d-3"] {
        store.try_add_domain(domain(id, id)).expect("identifiants distincts");
    }

    // Deux nœuds, deux ordres de clic opposés.
    for (node, order) in [("t-1", ["d-3", "d-1", "d-2"]), ("s-1", ["d-2", "d-3", "d-1"])] {
        for id in order {
            store
                .try_assign_domain_to_node("main", node, id, 0.5)
                .expect("nœud et domaine présents");
        }
    }

    for node in ["t-1", "s-1"] {
        let ids: Vec<&str> = store
            .node_domains("main", node)
            .expect("nœud présent")
            .iter()
            .map(|a| a.domain_id.as_str())
            .collect();
        assert_eq!(ids, ["d-1", "d-2", "d-3"], "{node} : la réglette doit être stable");
    }
}

// ── 9. L'élagage d'un document venu du disque ───────────────────────────────

/// Un fichier écrit avant la cascade peut porter des références vers un domaine disparu.
/// `load_project` les retire et **dit combien** : la réparation n'est pas silencieuse.
#[test]
fn test_loading_a_document_written_before_the_cascade_repairs_and_reports_it() {
    let mut donor = populated_store();
    donor.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    for (board, node) in ALL_NODES {
        donor
            .try_assign_domain_to_node(board, node, "d", 0.5)
            .expect("nœud et domaine présents");
    }
    // Le bug d'origine, reproduit à la main : le catalogue perd le domaine, les nœuds non.
    let mut corrupted = donor.project.clone();
    corrupted.domains.clear();

    let mut store = Store::new("Hôte");
    let repaired = store.load_project(corrupted);

    assert_eq!(repaired, ALL_NODES.len(), "chaque nœud atteint doit être compté");
    assert!(
        store.orphan_domain_references().is_empty(),
        "restes : {:?}",
        store.orphan_domain_references()
    );
    for (board, node) in ALL_NODES {
        assert!(store.node_domains(board, node).expect("nœud présent").is_empty());
    }
}

/// Un poids aberrant venu d'un fichier n'est pas une longueur : le laisser entrer donnerait au
/// rendu une hauteur `NaN`. Le chargement le retire et le compte, comme une orpheline.
#[test]
fn test_loading_a_document_with_an_absurd_weight_drops_it_and_reports_it() {
    let mut donor = populated_store();
    donor.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    donor.try_assign_domain_to_node("main", "t-1", "d", 0.5).expect("t-1");
    donor.try_assign_domain_to_node("main", "s-1", "d", 0.5).expect("s-1");

    // Écriture directe dans le modèle : c'est exactement ce qu'un fichier antérieur contient.
    let mut corrupted = donor.project.clone();
    for (node, weight) in [("t-1", f64::NAN), ("s-1", 42.0)] {
        let board = corrupted.boards.iter_mut().find(|b| b.id == "main").expect("main");
        let ann = board.annotations.iter_mut().find(|a| a.id() == node).expect("nœud");
        ann.domains_mut()[0].weight = weight;
    }

    let mut store = Store::new("Hôte");
    assert_eq!(store.load_project(corrupted), 2, "les deux nœuds doivent être comptés");
    assert!(store.node_domains("main", "t-1").expect("t-1").is_empty());
    assert!(store.node_domains("main", "s-1").expect("s-1").is_empty());
}

#[test]
fn test_loading_a_healthy_document_repairs_nothing() {
    let mut donor = populated_store();
    donor.try_add_domain(domain("d", "Science")).expect("catalogue vide");
    donor.try_assign_domain_to_node("main", "t-1", "d", 0.5).expect("t-1");

    let mut store = Store::new("Hôte");
    let repaired = store.load_project(donor.project.clone());

    assert_eq!(repaired, 0, "un document sain ne doit rien perdre");
    assert_eq!(store.project, donor.project);
}

// ── 10. Lecture ─────────────────────────────────────────────────────────────

#[test]
fn test_node_domains_names_what_it_could_not_find() {
    let store = populated_store();
    assert_eq!(
        store.node_domains("fantome", "t-1"),
        Err(CoreError::BoardNotFound("fantome".into()))
    );
    assert_eq!(
        store.node_domains("main", "fantome"),
        Err(CoreError::NodeNotFound("fantome".into()))
    );
    assert_eq!(store.node_domains("main", "t-1"), Ok(&[][..]));
}

#[test]
fn test_domain_rank_is_the_column_a_domain_occupies() {
    let mut store = Store::new("P");
    for id in ["d-1", "d-2", "d-3"] {
        store.try_add_domain(domain(id, id)).expect("identifiants distincts");
    }
    assert_eq!(store.domain_rank("d-1"), Some(0));
    assert_eq!(store.domain_rank("d-3"), Some(2));
    assert_eq!(store.domain_rank("fantome"), None);
}

// ── 11. La persistance ──────────────────────────────────────────────────────

/// PERSIST-1 appliqué aux domaines : le catalogue **et** les pondérations de chaque nœud
/// reviennent à l'identique. Vérifié plutôt que supposé (standard § 7.5).
#[test]
fn test_domains_and_their_weights_survive_the_round_trip() {
    use glucose_core::persist;
    use glucose_core::types::AssetStore;

    let mut store = populated_store();
    store.try_add_domain(domain("d-sci", "Science")).expect("catalogue vide");
    store
        .try_update_domain(
            "d-sci",
            DomainPatch::new().with_color("#38bdf8").with_icon("SCI"),
        )
        .expect("d-sci est au catalogue");
    store.try_add_domain(domain("d-art", "Art")).expect("identifiant neuf");
    store
        .try_update_domain("d-art", DomainPatch::new().with_color("#f472b6").with_icon("ART"))
        .expect("d-art est au catalogue");

    for (i, (board, node)) in ALL_NODES.iter().enumerate() {
        let weight = i as f64 / (ALL_NODES.len() - 1) as f64;
        store
            .try_assign_domain_to_node(board, node, "d-sci", weight)
            .expect("nœud et domaine présents");
        if i % 2 == 0 {
            store
                .try_assign_domain_to_node(board, node, "d-art", 1.0 - weight)
                .expect("nœud et domaine présents");
        }
    }

    let bytes = persist::encode(&store.project, &AssetStore::new(), 0);
    let reloaded = persist::decode(&bytes).expect("un fichier que l'on vient d'écrire se relit");

    assert_eq!(reloaded.project, store.project, "le document relu diffère");
    assert_eq!(reloaded.project.domains.len(), 2);
    assert_eq!(reloaded.project.domains[0].color, "#38bdf8");
    assert_eq!(reloaded.project.domains[1].icon, "ART");

    let mut host = Store::new("Hôte");
    assert_eq!(host.load_project(reloaded.project), 0, "rien à réparer");
    for (i, (board, node)) in ALL_NODES.iter().enumerate() {
        let carried = host.node_domains(board, node).expect("nœud présent");
        let weight = i as f64 / (ALL_NODES.len() - 1) as f64;
        assert_eq!(carried[0].domain_id, "d-sci", "{board}/{node}");
        assert_eq!(carried[0].weight, weight, "{board}/{node}");
        assert_eq!(carried.len(), if i % 2 == 0 { 2 } else { 1 }, "{board}/{node}");
    }
}

/// Le bord de la plage : `0.0` et `1.0` doivent traverser le disque bit à bit, sans qu'une
/// écriture décimale ne les décale.
#[test]
fn test_the_extreme_weights_survive_bit_for_bit() {
    use glucose_core::persist;

    let mut store = populated_store();
    store.try_add_domain(domain("d-min", "Min")).expect("catalogue vide");
    store.try_add_domain(domain("d-max", "Max")).expect("identifiant neuf");
    store.try_assign_domain_to_node("main", "t-1", "d-min", 0.0).expect("t-1");
    store.try_assign_domain_to_node("main", "t-1", "d-max", 1.0).expect("t-1");
    store
        .try_assign_domain_to_node("main", "img-1", "d-min", 1.0 / 3.0)
        .expect("img-1");

    let reloaded = persist::decode_document(&persist::encode_document(&store.project))
        .expect("relecture du document nu");
    let mut host = Store::new("Hôte");
    host.load_project(reloaded);

    let text_node = host.node_domains("main", "t-1").expect("t-1");
    assert_eq!(text_node[0].weight.to_bits(), 0.0_f64.to_bits());
    assert_eq!(text_node[1].weight, 1.0);
    assert_eq!(host.node_domains("main", "img-1").expect("img-1")[0].weight, 1.0 / 3.0);
}
