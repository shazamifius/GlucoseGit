//! Les épreuves de l'import (BOARDS-2).
//!
//! La preuve n'emprunte rien au code qu'elle éprouve. L'import garde l'**ordre** de l'original
//! — ses tableaux à la suite, chaque liste dans son ordre —, donc la correspondance entre un
//! élément et sa copie se lit par position. Chaque référence de la copie doit alors être
//! l'image, par cette correspondance, de la référence de l'original.

use super::*;
use crate::curtain_model::{create_curtain, CurtainOwner};
use crate::synth;

/// Le témoin, enrichi de ce qui porte des références : un domaine assigné, une image née
/// dans la membrane, un miroir de dossier, un rideau qui vise le tableau du dossier.
fn document_riche() -> Project {
    let mut store = synth::witness();
    let b = store.project.active_board_id.clone();
    store
        .try_add_domain(Domain {
            id: "d-art".into(),
            name: "Art".into(),
            color: "#ff0000".into(),
            icon: "A".into(),
            created_at: 0,
        })
        .expect("un domaine");
    store
        .try_assign_domain_to_node(&b, "m-titree", "d-art", 0.5)
        .expect("assigné à la membrane");
    store.add_image(&b, BoardImage::new("i-dans-m", 200.0, -150.0, 80.0, 60.0));
    store
        .try_mirror_folder(&b, "f-temoin", 900.0, 0.0)
        .expect("un miroir");
    let tableau_du_dossier = store.project.boards[0].folders[0].child_board_id.clone();
    store.update_annotation(&b, "m-titree", |a| {
        if let Annotation::Membrane { curtains, .. } = a {
            let mut rideau = create_curtain(
                "c-1",
                CurtainOwner {
                    id: "moi".into(),
                    name: "Moi".into(),
                    color: "#00ff00".into(),
                },
                0,
            );
            rideau.board_id = Some(tableau_du_dossier.clone());
            curtains.push(rideau);
        }
    });
    store.project
}

/// Tous les identifiants que porte une suite de tableaux, dans un ordre fixe.
fn ids_des_tableaux(boards: &[Board]) -> Vec<String> {
    let mut ids = Vec::new();
    for b in boards {
        ids.push(b.id.clone());
        ids.extend(b.images.iter().map(|i| i.id.clone()));
        for a in &b.annotations {
            ids.push(a.id().to_string());
            if let Annotation::Membrane { curtains, .. } = a {
                ids.extend(curtains.iter().map(|c| c.id.clone()));
            }
        }
        ids.extend(b.folders.iter().map(|f| f.id.clone()));
        ids.extend(b.panels.iter().map(|p| p.id.clone()));
    }
    ids
}

/// Toutes les références que porte une suite de tableaux, dans le même ordre fixe.
fn references(boards: &[Board]) -> Vec<Option<String>> {
    let mut refs = Vec::new();
    for b in boards {
        for i in &b.images {
            refs.extend([i.membrane_id.clone(), i.mirror_of.clone()]);
            refs.extend(i.domains.iter().map(|d| Some(d.domain_id.clone())));
        }
        for a in &b.annotations {
            refs.push(a.membrane_id().map(str::to_string));
            if let Annotation::Arrow {
                source_id,
                target_id,
                target_board_id,
                ..
            } = a
            {
                refs.extend([
                    source_id.clone(),
                    target_id.clone(),
                    target_board_id.clone(),
                ]);
            }
            if let Annotation::Membrane {
                curtains, domains, ..
            } = a
            {
                refs.extend(curtains.iter().map(|c| c.board_id.clone()));
                refs.extend(domains.iter().map(|d| Some(d.domain_id.clone())));
            }
        }
        for f in &b.folders {
            refs.extend([Some(f.child_board_id.clone()), f.mirror_of.clone()]);
        }
    }
    refs
}

/// **Importé dans lui-même, un document ne heurte rien, et chaque référence de la copie vise
/// la copie** — jamais l'original, qui porte pourtant les mêmes identifiants d'origine.
#[test]
fn test_chaque_reference_suit_sa_copie_et_aucun_identifiant_ne_se_heurte() {
    let doc = document_riche();
    assert!(
        references(&doc.boards)
            .iter()
            .filter(|r| r.is_some())
            .count()
            >= 6,
        "le document d'épreuve porte vraiment des références"
    );
    let mut store = Store::new("courant");
    store.load_project(doc.clone());
    let n = store.project.boards.len();
    let d = store.project.domains.len();
    store.importer_un_document(doc.clone(), "Import");

    let copie = &store.project.boards[n..];
    let (avant, apres) = (ids_des_tableaux(&doc.boards), ids_des_tableaux(copie));
    assert_eq!(avant.len(), apres.len(), "tout est copié");
    let mut table: HashMap<String, String> = avant.into_iter().zip(apres).collect();
    for (o, c) in doc.domains.iter().zip(&store.project.domains[d..]) {
        table.insert(o.id.clone(), c.id.clone());
    }
    let mut tous = ids_des_tableaux(&store.project.boards);
    tous.extend(store.project.domains.iter().map(|x| x.id.clone()));
    let uniques: std::collections::HashSet<&String> = tous.iter().collect();
    assert_eq!(uniques.len(), tous.len(), "aucun identifiant ne se heurte");

    let attendues: Vec<Option<String>> = references(&doc.boards)
        .into_iter()
        .map(|r| r.map(|id| table.get(&id).cloned().unwrap_or(format!("absent:{id}"))))
        .collect();
    assert_eq!(
        references(copie),
        attendues,
        "chaque référence suit sa copie"
    );
}

/// **Un seul geste** : un `Ctrl+Z` rend le document tel qu'il était, domaines compris ; le
/// premier onglet importé devient l'onglet actif.
#[test]
fn test_l_import_est_un_seul_geste() {
    let doc = document_riche();
    let mut store = Store::new("courant");
    let avant = store.project.clone();
    let onglets = store.importer_un_document(doc, "Ancien projet");
    assert_eq!(
        onglets.len(),
        1,
        "le témoin a un onglet : son dossier n'en est pas un"
    );
    assert_eq!(store.project.active_board_id, onglets[0]);
    let noms: Vec<&str> = store.onglets().map(|(_, n, _)| n).collect();
    assert_eq!(noms, ["Canvas Principal", "Ancien projet"]);
    assert!(store.undo(), "un geste");
    assert!(store.project == avant, "Ctrl+Z rend le document d'avant");
    assert!(!store.undo(), "et rien d'autre");
}

/// **Plusieurs onglets gardent leur nom, préfixé de celui du document.**
#[test]
fn test_plusieurs_onglets_gardent_leur_nom() {
    let mut autre = Store::new("autre");
    let premier = autre.project.active_board_id.clone();
    autre.rename_board(&premier, "Idées");
    autre.add_board("Sources");
    let mut store = Store::new("courant");
    store.importer_un_document(autre.project, "Thèse");
    let noms: Vec<&str> = store.onglets().map(|(_, n, _)| n).collect();
    assert_eq!(
        noms,
        ["Canvas Principal", "Thèse — Idées", "Thèse — Sources"]
    );
}

/// **Une référence vers ce que le document importé ne contient pas disparaît** — même si le
/// document courant contient un élément de ce nom.
#[test]
fn test_une_reference_vers_un_absent_disparait() {
    let mut store = Store::new("courant");
    let b = store.project.active_board_id.clone();
    store.add_annotation(&b, Annotation::text("t-orpheline", 0.0, 0.0, "ici"));
    let mut autre = Store::new("autre");
    let ab = autre.project.active_board_id.clone();
    let mut fleche = Annotation::arrow("a-1", 0.0, 0.0, 10.0, 10.0);
    if let Annotation::Arrow { target_id, .. } = &mut fleche {
        *target_id = Some("t-orpheline".into());
    }
    autre.add_annotation(&ab, fleche);
    store.importer_un_document(autre.project, "Autre");
    let importee = store
        .project
        .boards
        .last()
        .and_then(|b| b.annotations.first())
        .expect("la flèche importée");
    let Annotation::Arrow { target_id, .. } = importee else {
        panic!("une flèche");
    };
    assert_eq!(
        *target_id, None,
        "elle ne vise pas la carte du document courant"
    );
}
