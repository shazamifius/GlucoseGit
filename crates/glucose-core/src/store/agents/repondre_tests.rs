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

/// **Le 27/11** : à droite de sa note, une membrane de 6 000 de large ; dessous et à gauche, des
/// voisins ; au-dessus, le vide. À droite et dessous seulement, la réponse tombait à des milliers
/// d'unités ; elle se pose juste au-dessus, et rien ne chevauche.
#[test]
fn mcp_repondre_le_vide_au_dessus_plutot_que_loin() {
    let (mut store, board, t) = une_note();
    let m = Annotation::membrane("loin", t.right() + ECART, t.top - 50.0, 6000.0, 4000.0);
    store.add_annotation(&board, m);
    store.add_annotation(
        &board,
        Annotation::text("dessous", t.left, t.bottom() + ECART, "un voisin"),
    );
    store.add_annotation(
        &board,
        Annotation::text("gauche", t.left - 300.0, t.top, "un voisin"),
    );
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    assert_eq!(
        (p.x, p.y),
        (t.left, t.top - ECART - 100.0),
        "juste au-dessus"
    );
    let r = crate::geometry::Rect::new(p.x, p.y, 240.0, 100.0);
    for d in store.decrire_pour_un_agent(&board).expect("le tableau") {
        assert!(!r.overlaps(d.boite), "{r:?} chevauche {}", d.id);
    }
}

/// **À gauche, quand c'est le plus près** : un voisin à droite, un autre dessous, la gauche libre.
#[test]
fn mcp_repondre_a_gauche_quand_droite_et_dessous_sont_pris() {
    let (mut store, board, t) = une_note();
    store.add_annotation(
        &board,
        Annotation::text("droite", t.right() + ECART, t.top, "un voisin"),
    );
    store.add_annotation(
        &board,
        Annotation::text("dessous", t.left, t.bottom() + ECART, "un voisin"),
    );
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    assert_eq!((p.x, p.y), (t.left - ECART - 240.0, t.top));
}

/// L'écart de deux boîtes : le vide qui les sépare, de côté, d'en haut, ou nul.
#[test]
fn mcp_repondre_l_ecart_de_deux_boites() {
    use crate::geometry::Rect;
    let a = Rect::new(0.0, 0.0, 100.0, 100.0);
    assert_eq!(super::ecart(a, Rect::new(130.0, 50.0, 10.0, 10.0)), 30.0);
    assert_eq!(super::ecart(a, Rect::new(-50.0, 0.0, 10.0, 10.0)), 40.0);
    assert_eq!(super::ecart(a, Rect::new(0.0, -70.0, 10.0, 20.0)), 50.0);
    assert_eq!(super::ecart(a, Rect::new(50.0, 50.0, 10.0, 10.0)), 0.0);
}

/// **À gauche, elle glisse derrière son voisin** : à droite, dessous et au-dessus, de grandes
/// membranes ; à gauche, un voisin collé — la réponse passe derrière lui, sans le chevaucher.
#[test]
fn mcp_repondre_a_gauche_glisse_derriere_son_voisin() {
    let (mut store, board, t) = une_note();
    let mur = |id: &str, x: f64, y: f64| Annotation::membrane(id, x, y, 8000.0, 8000.0);
    store.add_annotation(&board, mur("droite", t.right() + ECART, t.top - 4000.0));
    // Dessous et au-dessus, à partir du bord gauche de la note : la voie de gauche reste ouverte.
    store.add_annotation(&board, mur("dessous", t.left, t.bottom() + ECART));
    store.add_annotation(&board, mur("dessus", t.left, t.top - ECART - 8000.0));
    store.add_annotation(
        &board,
        Annotation::text("voisin", t.left - ECART - 100.0, t.top, "v"),
    );
    let v = store
        .project
        .annotation(&board, "voisin")
        .expect("le voisin")
        .bounds();
    let p = store
        .place_pour_une_reponse(&board, "n", (240.0, 100.0))
        .expect("une place");
    assert_eq!(
        (p.x, p.y),
        (v.left - ECART - 240.0, t.top),
        "derrière le voisin"
    );
}
