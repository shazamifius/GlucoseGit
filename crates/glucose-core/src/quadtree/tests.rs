//! Tests de l'index spatial — la grille, ses invariants de coût, ses requêtes.

use super::*;

#[test]
fn test_spatial_hash_query() {
    let mut sh = SpatialHash::new(1000.0);
    let items = [
        ("img1", 500.0, 500.0, 100.0, 100.0),
        ("img2", 2500.0, 2500.0, 100.0, 100.0),
    ];
    sh.build(items);

    let visible = sh.query_ids(0.0, 0.0, 1000.0, 1000.0, 0.0);
    assert!(visible.contains("img1"));
    assert!(!visible.contains("img2"));
}

#[test]
fn test_insert_remove_update_incremental() {
    let mut sh = SpatialHash::new(1000.0);
    sh.insert("a", 0.0, 0.0, 10.0, 10.0);
    assert!(sh.contains("a"));
    assert_eq!(sh.len(), 1);

    assert!(sh.update("a", 5000.0, 5000.0, 5010.0, 5010.0));
    assert!(sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0).is_empty());
    assert!(sh
        .query_rect_refs(4900.0, 4900.0, 5100.0, 5100.0, 0.0)
        .contains("a"));

    assert!(sh.remove("a"));
    assert!(!sh.remove("a"));
    assert!(sh.is_empty());
    assert!(sh
        .query_rect_refs(4900.0, 4900.0, 5100.0, 5100.0, 0.0)
        .is_empty());
}

#[test]
fn test_reindex_same_id_does_not_duplicate() {
    let mut sh = SpatialHash::new(1000.0);
    for _ in 0..10 {
        sh.insert("a", 0.0, 0.0, 10.0, 10.0);
    }
    assert_eq!(sh.len(), 1);
    assert_eq!(sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0).len(), 1);
    // Une seule écriture de cellule pour 10 insertions identiques (SPAT-1).
    assert_eq!(sh.cell_write_count(), 1);
}

#[test]
fn test_free_slot_is_reused_after_remove() {
    let mut sh = SpatialHash::new(1000.0);
    sh.insert("a", 0.0, 0.0, 10.0, 10.0);
    sh.remove("a");
    sh.insert("b", 0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        sh.slots.len(),
        1,
        "l'emplacement libéré doit être réutilisé"
    );
    let hit = sh.query_rect_refs(0.0, 0.0, 100.0, 100.0, 0.0);
    assert!(hit.contains("b"));
    assert!(!hit.contains("a"));
}
