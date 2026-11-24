use super::super::{Refus, ECART};
use crate::store::Store;
use crate::types::Annotation;

/// Une note « n » en (0, 0), seule sur son tableau ; sa boîte.
fn une_note() -> (Store, String, crate::geometry::Rect) {
    let mut store = Store::new("repondre");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("n", 0.0, 0.0, "ma note"));
    let t = store
        .project
        .annotation(&board, "n")
        .expect("la note")
        .bounds();
    (store, board, t)
}

/// **Seule, la réponse se pose à droite de la note**, à son sommet.
#[test]
fn mcp_repondre_seule_a_droite_de_la_note() {
    let (store, board, t) = une_note();
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    assert_eq!((p.x, p.y), (t.right() + ECART, t.top));
}

/// **Un voisin à droite : elle passe dessous**, plus près que derrière lui ; et rien ne chevauche.
#[test]
fn mcp_repondre_un_voisin_a_droite_la_fait_passer_dessous() {
    let (mut store, board, t) = une_note();
    let large = Annotation::text("v", t.right() + ECART, t.top, "un voisin");
    store.add_annotation(&board, large);
    let v = store
        .project
        .annotation(&board, "v")
        .expect("le voisin")
        .bounds();
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    let r = crate::geometry::Rect::new(p.x, p.y, 240.0, 100.0);
    assert!(!r.overlaps(v) && !r.overlaps(t), "{r:?} chevauche");
    // Les deux voies : derrière le voisin, ou sous la note ; la plus proche gagne.
    let derriere = v.right() + ECART - t.right();
    let dessous = t.bottom() + ECART - t.bottom();
    assert_eq!(
        dessous < derriere,
        (p.x, p.y) == (t.left, t.bottom() + ECART),
        "{p:?}"
    );
}

/// **Dans une membrane, la réponse y entre** : la membrane n'est pas un obstacle, et la place le
/// dit.
#[test]
fn mcp_repondre_dans_la_membrane_de_la_note() {
    let mut store = Store::new("repondre");
    let board = store.project.active_board_id.clone();
    let m = Annotation::membrane("m", -100.0, -100.0, 2000.0, 1500.0);
    store.add_annotation(&board, m);
    store.add_annotation(&board, Annotation::text("n", 0.0, 0.0, "ma note"));
    let t = store
        .project
        .annotation(&board, "n")
        .expect("la note")
        .bounds();
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    assert_eq!((p.x, p.y), (t.right() + ECART, t.top));
    assert_eq!(p.membrane.as_deref(), Some("m"));
}

/// Répondre à un nœud qui n'existe pas le dit.
#[test]
fn mcp_repondre_a_un_inconnu_le_dit() {
    let (store, board, _) = une_note();
    assert!(matches!(
        store.place_pour_une_reponse(&board, "personne", (240.0, 100.0)),
        Err(Refus::NoeudInconnu(_))
    ));
}
