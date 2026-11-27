use super::super::Lien;
use super::{segment_traverse, Appartenance};
use crate::geometry::Rect;
use crate::store::Store;
use crate::types::{Annotation, Point2D};

fn tableau() -> (Store, String) {
    let store = Store::new("verifier");
    let board = store.project.active_board_id.clone();
    (store, board)
}

fn texte(store: &mut Store, board: &str, id: &str, x: f64, y: f64) {
    store.add_annotation(board, Annotation::text(id, x, y, id));
}

fn relier(store: &mut Store, board: &str, id: &str, de: &str, a: &str) {
    let lien = Lien {
        id,
        source: de,
        cible: a,
        passage_source: None,
        passage_cible: None,
        relation: None,
        texte: None,
    };
    store.relier_pour_un_agent(board, lien).expect("la flèche");
}

/// Des nœuds bien rangés, une membrane qui les contient et le sait, une flèche entre eux : rien.
#[test]
fn verifier_un_tableau_propre_ne_dit_rien() {
    let (mut store, board) = tableau();
    store.add_annotation(
        &board,
        Annotation::membrane("m", -100.0, -100.0, 1200.0, 600.0),
    );
    texte(&mut store, &board, "a", 0.0, 0.0);
    texte(&mut store, &board, "b", 600.0, 0.0);
    for id in ["a", "b"] {
        store.update_annotation(&board, id, |n| n.set_membrane_id(Some("m".into())));
    }
    relier(&mut store, &board, "f", "a", "b");
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    assert!(constat.est_propre(), "{constat:?}");
}

/// Deux textes qui se recouvrent ; un texte à cheval sur une membrane.
#[test]
fn verifier_les_chevauchements_et_ce_qui_est_a_cheval() {
    let (mut store, board) = tableau();
    texte(&mut store, &board, "a", 0.0, 0.0);
    texte(&mut store, &board, "b", 100.0, 10.0);
    store.add_annotation(&board, Annotation::membrane("m", 1000.0, 0.0, 400.0, 300.0));
    texte(&mut store, &board, "c", 900.0, 50.0);
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    let paires: Vec<(&str, &str)> = (constat.chevauchements.iter())
        .map(|(p, q)| (p.as_str(), q.as_str()))
        .collect();
    assert_eq!(paires, [("a", "b"), ("m", "c")]);
}

/// **Dedans à l'œil, dehors pour l'IA** (le 26/11, deux membranes à 98 % dans « 🐞 ») — et
/// l'inverse : rangé dans une membrane qu'il a quittée.
#[test]
fn verifier_les_appartenances_contredites() {
    let (mut store, board) = tableau();
    store.add_annotation(&board, Annotation::membrane("m", 0.0, 0.0, 800.0, 600.0));
    store.add_annotation(&board, Annotation::membrane("n", 50.0, 50.0, 300.0, 300.0));
    texte(&mut store, &board, "dedans", 80.0, 100.0);
    texte(&mut store, &board, "parti", 2000.0, 0.0);
    store.update_annotation(&board, "n", |a| a.set_membrane_id(Some("m".into())));
    store.update_annotation(&board, "parti", |a| a.set_membrane_id(Some("m".into())));
    // « dedans » n'a pas de membrane pour l'IA : à l'œil, la plus petite qui le contient, « n ».
    store.update_annotation(&board, "dedans", |a| a.set_membrane_id(None));
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    assert_eq!(
        constat.appartenances,
        [
            Appartenance {
                id: "dedans".into(),
                a_l_oeil: Some("n".into()),
                pour_l_ia: None
            },
            Appartenance {
                id: "parti".into(),
                a_l_oeil: None,
                pour_l_ia: Some("m".into())
            },
        ]
    );
}

/// **Le tracé dessiné, pas la droite des bouts** : un texte entre deux autres, la flèche qui les
/// relie le contourne — rien ; un coude posé au milieu de lui, elle le traverse.
#[test]
fn verifier_une_fleche_mesuree_sur_son_trace() {
    let (mut store, board) = tableau();
    texte(&mut store, &board, "a", 0.0, 0.0);
    texte(&mut store, &board, "milieu", 600.0, 0.0);
    texte(&mut store, &board, "b", 1200.0, 0.0);
    relier(&mut store, &board, "f", "a", "b");
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    assert!(constat.traversees.is_empty(), "contournée : {constat:?}");
    let centre = store.boite_d_un_noeud(&board, "milieu").expect("le milieu");
    let coude = Point2D {
        x: centre.left + centre.width / 2.0,
        y: centre.top + centre.height / 2.0,
    };
    store.update_annotation(&board, "f", |a| {
        if let Annotation::Arrow { waypoints, .. } = a {
            waypoints.push(coude);
        }
    });
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    assert_eq!(
        constat.traversees,
        [("f".to_string(), vec!["milieu".to_string()])]
    );
}

/// **Sortir de la membrane de sa source n'est pas la traverser** ; en traverser une autre, si.
#[test]
fn verifier_une_fleche_qui_sort_de_sa_membrane() {
    let (mut store, board) = tableau();
    store.add_annotation(
        &board,
        Annotation::membrane("chez-a", -50.0, -50.0, 400.0, 300.0),
    );
    texte(&mut store, &board, "a", 0.0, 0.0);
    store.update_annotation(&board, "a", |n| n.set_membrane_id(Some("chez-a".into())));
    store.add_annotation(
        &board,
        Annotation::membrane("passage", 500.0, -400.0, 200.0, 1000.0),
    );
    texte(&mut store, &board, "b", 1000.0, 0.0);
    relier(&mut store, &board, "f", "a", "b");
    let constat = store.verifier_le_tableau(&board).expect("le tableau");
    let traverses: Vec<&str> = (constat.traversees.iter())
        .flat_map(|(_, ids)| ids.iter().map(String::as_str))
        .collect();
    assert!(!traverses.contains(&"chez-a"), "{constat:?}");
    // « passage » ne contient aucun bout, et la flèche le coupe : une membrane se traverse pour
    // relier deux domaines, ce n'est pas un défaut.
    assert!(constat.traversees.is_empty(), "{constat:?}");
}

/// Liang et Barsky : traverser, passer à côté, longer un bord, finir avant.
#[test]
fn verifier_un_segment_et_une_boite() {
    let r = Rect::new(0.0, 0.0, 100.0, 100.0);
    assert!(
        segment_traverse((-50.0, 50.0), (150.0, 50.0), r),
        "de part en part"
    );
    assert!(
        segment_traverse((50.0, 50.0), (50.0, 300.0), r),
        "depuis l'intérieur"
    );
    assert!(
        !segment_traverse((-50.0, 150.0), (150.0, 150.0), r),
        "à côté"
    );
    assert!(
        !segment_traverse((-50.0, 0.0), (150.0, 0.0), r),
        "le long du bord"
    );
    assert!(
        !segment_traverse((-50.0, 50.0), (-1.0, 50.0), r),
        "finit avant"
    );
    assert!(
        !segment_traverse((-50.0, 50.0), (0.0, 50.0), r),
        "finit sur le bord"
    );
    assert!(
        segment_traverse((-50.0, -50.0), (150.0, 150.0), r),
        "en diagonale"
    );
}
