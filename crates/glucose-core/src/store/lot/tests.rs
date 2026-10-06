//! Les épreuves du lot (fiche 51 § 2).
//!
//! La preuve forte est un **aller-retour** : ce qu'on colle, re-extrait, redonne le lot
//! d'origine — à ses identifiants près, et rien d'autre. La comparaison passe par la forme
//! `Debug` du lot, où chaque identifiant est une chaîne entre guillemets : remplacer chaque
//! identifiant neuf par l'ancien, guillemets compris, ne peut toucher que des identifiants.

use super::*;
use crate::types::{BoardImage, Domain, DomainAssignment};

const B: &str = "main";

/// Une membrane qui possède une image, un texte qui porte un domaine, une flèche qui relie le
/// texte à l'image, une autre vers un texte resté dehors.
fn scene() -> Store {
    let mut s = Store::new("source");
    s.try_add_domain(Domain {
        id: "d-art".into(),
        name: "Art".into(),
        color: "#ff0000".into(),
        icon: "A".into(),
        created_at: 0,
    })
    .expect("un domaine");
    s.add_annotation(B, Annotation::membrane("m", 0.0, 0.0, 400.0, 300.0));
    s.add_image(B, BoardImage::new("i", 50.0, 50.0, 80.0, 60.0));
    s.add_annotation(B, Annotation::text("t", 500.0, 0.0, "Bonjour"));
    s.add_annotation(B, Annotation::text("dehors", 900.0, 0.0, "Ailleurs"));
    s.try_assign_domain_to_node(B, "t", "d-art", 0.5)
        .expect("assigné");
    let mut a = Annotation::arrow("a", 500.0, 20.0, 90.0, 80.0);
    let mut b = Annotation::arrow("b", 500.0, 20.0, 900.0, 20.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut a
    {
        *source_id = Some("t".into());
        *target_id = Some("i".into());
    }
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut b
    {
        *source_id = Some("t".into());
        *target_id = Some("dehors".into());
    }
    s.add_annotation(B, a);
    s.add_annotation(B, b);
    s
}

fn selectionner(s: &mut Store, annotations: &[&str]) {
    s.clear_selection();
    s.set_selected_annotation_ids(annotations.iter().map(|a| a.to_string()).collect());
}

/// Le lot, et le centre de sa boîte.
fn lot_et_centre(s: &mut Store, annotations: &[&str]) -> (Project, (f64, f64)) {
    selectionner(s, annotations);
    let lot = s.extraire_la_selection(B).expect("un lot");
    let c = boite_du_contenu(&lot.boards[0])
        .expect("une boîte")
        .center();
    (lot, (c.x, c.y))
}

/// Ce que le lot porte, identifiants neufs remplacés par les anciens.
fn sans_les_noms(lot: &Project, anciens: &Project, poses: &[String]) -> String {
    let b = &anciens.boards[0];
    let vieux = b
        .images
        .iter()
        .map(|i| i.id.clone())
        .chain(b.annotations.iter().map(|a| a.id().to_string()));
    let mut texte = format!("{lot:?}");
    for (neuf, vieux) in poses.iter().zip(vieux) {
        texte = texte.replace(&format!("\"{neuf}\""), &format!("\"{vieux}\""));
    }
    texte
}

/// **Coller puis re-extraire redonne le lot, au bit près, sous des noms neufs.** Le lot passe
/// par l'encodage du document, comme dans le presse-papiers.
///
/// Il se colle dans un document vierge qui porte le même domaine, à sa propre place : collé sur
/// l'original, la membrane copiée tomberait **dans** l'originale et lui appartiendrait — c'est
/// MEMB-1, et ce n'est pas ce que cette épreuve regarde.
#[test]
fn test_lot_1_coller_rend_le_lot_au_bit_pres() {
    let mut s = scene();
    let (lot, centre) = lot_et_centre(&mut s, &["m", "t"]);
    let octets = crate::persist::encode_document(&lot);
    let relu = crate::persist::decode_document(&octets).expect("relu");
    assert_eq!(relu, lot, "le lot traverse l'encodage du document");

    let mut cible = Store::new("source");
    cible
        .try_add_domain(lot.domains[0].clone())
        .expect("le domaine");
    let ancien = scene();
    let avant: HashSet<String> = ancien.project.boards[0]
        .images
        .iter()
        .map(|i| i.id.clone())
        .chain(
            ancien.project.boards[0]
                .annotations
                .iter()
                .map(|a| a.id().to_string()),
        )
        .collect();
    let poses = cible.coller_un_lot(B, relu, centre, Provenance::Ailleurs);
    assert_eq!(
        poses.len(),
        4,
        "la membrane, son image, le texte, la flèche qui les relie"
    );
    assert!(poses.iter().all(|p| !avant.contains(p)), "des noms neufs");

    let recopie = cible
        .extraire_la_selection(B)
        .expect("le collage est sélectionné");
    assert_eq!(sans_les_noms(&recopie, &lot, &poses), format!("{lot:?}"));
}

/// **Un collage est un geste** : un `Ctrl+Z` le retire en entier, et le document redevient
/// celui d'avant.
#[test]
fn test_lot_2_un_collage_se_defait_d_un_coup() {
    let mut s = scene();
    let (lot, _) = lot_et_centre(&mut s, &["m", "t", "b"]);
    let avant = s.project.clone();
    let profondeur = s.undo_depth();
    s.coller_un_lot(B, lot, (3000.0, 3000.0), Provenance::CeDocument);
    assert_eq!(s.undo_depth(), profondeur + 1, "un seul geste");
    assert!(s.undo());
    assert_eq!(s.project, avant, "tout est retiré");
}

/// **Le lot se pose centré sur le curseur**, ses places relatives intactes.
#[test]
fn test_lot_3_le_lot_se_pose_centre_sur_le_curseur() {
    let mut s = scene();
    let (lot, _) = lot_et_centre(&mut s, &["m", "t"]);
    s.coller_un_lot(B, lot, (3000.0, -2000.0), Provenance::CeDocument);
    let recopie = s.extraire_la_selection(B).expect("sélectionné");
    let c = boite_du_contenu(&recopie.boards[0])
        .expect("boîte")
        .center();
    assert!((c.x - 3000.0).abs() < 1e-9 && (c.y + 2000.0).abs() < 1e-9);
    let b = &recopie.boards[0];
    let image = &b.images[0];
    let texte = b
        .annotations
        .iter()
        .find(|a| a.own_text().as_deref() == Some("Bonjour"));
    let texte = texte.expect("le texte");
    assert!(
        ((texte.x() - image.x) - 450.0).abs() < 1e-9,
        "écart d'origine : 500 − 50"
    );
}

/// **Une flèche qui relie deux nœuds emportés part avec eux** ; une flèche sélectionnée dont un
/// bout reste dehors part aussi, ce bout libre à sa place.
#[test]
fn test_lot_4_les_fleches_suivent_ce_qu_elles_relient() {
    let mut s = scene();
    let (lot, _) = lot_et_centre(&mut s, &["m", "t", "b"]);
    let poses = s.coller_un_lot(B, lot, (0.0, 2000.0), Provenance::CeDocument);
    let board = &s.project.boards[0];
    let neuf = |ancien_texte: &str| {
        board
            .annotations
            .iter()
            .find(|a| {
                poses.contains(&a.id().to_string()) && a.own_text().as_deref() == Some(ancien_texte)
            })
            .map(|a| a.id().to_string())
    };
    let t = neuf("Bonjour").expect("le texte collé");
    let i = board
        .images
        .iter()
        .find(|i| poses.contains(&i.id))
        .expect("l'image collée");
    let fleches: Vec<_> = board
        .annotations
        .iter()
        .filter(|a| poses.contains(&a.id().to_string()))
        .filter_map(|a| match a {
            Annotation::Arrow {
                source_id,
                target_id,
                ..
            } => Some((source_id.clone(), target_id.clone())),
            _ => None,
        })
        .collect();
    assert!(
        fleches.contains(&(Some(t.clone()), Some(i.id.clone()))),
        "la flèche qui relie"
    );
    assert!(
        fleches.contains(&(Some(t), None)),
        "le bout dehors devient libre"
    );
    let m = board
        .annotations
        .iter()
        .find(|a| poses.contains(&a.id().to_string()) && matches!(a, Annotation::Membrane { .. }))
        .expect("la membrane collée");
    assert_eq!(
        i.membrane_id.as_deref(),
        Some(m.id()),
        "l'image reste à sa membrane"
    );
    let texte = board
        .annotations
        .iter()
        .find(|a| poses.contains(&a.id().to_string()) && a.own_text().as_deref() == Some("Bonjour"))
        .expect("le texte collé");
    assert_eq!(
        texte.domains()[0].domain_id,
        "d-art",
        "le domaine d'ici, pas un double"
    );
    assert_eq!(s.project.domains.len(), 1);
}

/// **Un lot venu d'ailleurs** retrouve un domaine identique, apporte le sien sinon, et ne vise
/// aucun tableau d'ici.
#[test]
fn test_lot_5_un_lot_venu_d_ailleurs() {
    let mut source = scene();
    let (lot, _) = lot_et_centre(&mut source, &["t"]);
    let domaine_du_lot = lot.domains[0].clone();

    let mut vierge = Store::new("autre");
    vierge.coller_un_lot(B, lot.clone(), (0.0, 0.0), Provenance::Ailleurs);
    assert_eq!(vierge.project.domains.len(), 1, "le domaine arrive");
    let assigne = |s: &Store| -> Vec<DomainAssignment> {
        s.project.boards[0].annotations[0].domains().to_vec()
    };
    assert_eq!(assigne(&vierge)[0].domain_id, vierge.project.domains[0].id);

    let mut jumeau = Store::new("autre");
    jumeau
        .try_add_domain(domaine_du_lot)
        .expect("le même domaine");
    jumeau.coller_un_lot(B, lot, (0.0, 0.0), Provenance::Ailleurs);
    assert_eq!(
        jumeau.project.domains.len(),
        1,
        "le domaine identique est retrouvé"
    );
    assert_eq!(assigne(&jumeau)[0].domain_id, "d-art");
}

/// **Supprimer une sélection mêlée est un seul geste.** Les images et les annotations partaient
/// chacune dans le leur : un `Ctrl+Z` n'en rendait que la moitié.
#[test]
fn test_lot_6_supprimer_une_selection_melee_se_defait_d_un_coup() {
    let mut s = scene();
    let avant = s.project.clone();
    s.set_selected_image_ids(vec!["i".into()]);
    s.set_selected_annotation_ids(vec!["t".into(), "dehors".into()]);
    s.delete_selected(B);
    assert!(s.undo());
    assert_eq!(s.project, avant, "un seul Ctrl+Z rend tout");
}
