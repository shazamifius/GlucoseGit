//! COLONNES-1 dans le document : l'arbre gardé, chacun dans sa membrane, un seul geste.

use super::super::agents::{ECART, TITRE_DE_MEMBRANE};
use crate::geometry::Rect;
use crate::store::Store;
use crate::types::Annotation;

/// Pose, comme un agent, un nœud de cette hauteur sous le contenu de `dans` (MEMB-7 fait grandir
/// les parentes).
fn poser(store: &mut Store, board: &str, id: &str, dans: Option<&str>, h: f64, membrane: bool) {
    let (x, y) = store.place_pour_un_agent(board, dans).expect("une place");
    store
        .geste_d_agent(|s| {
            if let Some(m) = dans {
                s.agrandir_la_membrane_pour(board, m, Rect::new(x, y, 240.0, h))?;
            }
            let n = if membrane {
                Annotation::membrane(id, x, y, 240.0, h)
            } else {
                let mut t = Annotation::text(id, x, y, id);
                if let Annotation::Text { width, height, .. } = &mut t {
                    (*width, *height) = (Some(240.0), Some(h));
                }
                t
            };
            s.poser_pour_un_agent(board, n, dans)
        })
        .expect("posé");
}

/// Un article empilé : six paragraphes, une section de quatre, une section vide.
fn un_article() -> (Store, String) {
    let mut store = Store::new("colonnes");
    let board = store.project.active_board_id.clone();
    poser(&mut store, &board, "article", None, 80.0, true);
    for i in 0..6 {
        let id = format!("p{i}");
        poser(
            &mut store,
            &board,
            &id,
            Some("article"),
            80.0 + 37.0 * f64::from(i),
            false,
        );
    }
    poser(&mut store, &board, "section", Some("article"), 80.0, true);
    for i in 0..4 {
        let id = format!("s{i}");
        poser(&mut store, &board, &id, Some("section"), 120.0, false);
    }
    poser(&mut store, &board, "vide", Some("article"), 90.0, true);
    store.journal.clear();
    (store, board)
}

fn boite(store: &Store, board: &str, id: &str) -> Rect {
    store.project.annotation(board, id).expect(id).bounds()
}

fn membrane_de(store: &Store, board: &str, id: &str) -> Option<String> {
    store
        .project
        .annotation(board, id)
        .and_then(Annotation::membrane_id)
        .map(str::to_string)
}

/// **Disposer garde l'arbre, met chacun dans sa membrane, approche la proportion, et se défait
/// d'un seul geste.** La preuve à l'envers : avant, l'article était une colonne bien plus haute que
/// large.
#[test]
fn colonnes_1_disposer_un_article_dans_le_document() {
    let (mut store, board) = un_article();
    let ids = [
        "p0", "p1", "p2", "p3", "p4", "p5", "section", "s0", "s1", "s2", "s3", "vide",
    ];
    let arbre: Vec<_> = ids.iter().map(|i| membrane_de(&store, &board, i)).collect();
    let avant = store.project.clone();
    let coin = (
        boite(&store, &board, "article").left,
        boite(&store, &board, "article").top,
    );
    let a = boite(&store, &board, "article");
    assert!(a.height > 2.0 * a.width, "empilé : {a:?}");

    store.begin_live_edit();
    let n = store.disposer_en_colonnes(&board, "article", 1.6);
    store.end_live_edit();
    assert_eq!(n, ids.len(), "chaque nœud rangé");
    let apres: Vec<_> = ids.iter().map(|i| membrane_de(&store, &board, i)).collect();
    assert_eq!(apres, arbre, "l'arbre ne change pas");
    let a = boite(&store, &board, "article");
    assert_eq!((a.left, a.top), coin, "son coin ne bouge pas");
    assert!(
        (a.width / a.height).ln().abs() < (1.6f64).ln() + 0.7,
        "la proportion approche celle de l'écran : {a:?}"
    );
    for id in ids {
        let parent = membrane_de(&store, &board, id).expect("une membrane");
        let (p, r) = (boite(&store, &board, &parent), boite(&store, &board, id));
        assert!(
            p.left + ECART <= r.left + 1e-9
                && p.top + TITRE_DE_MEMBRANE <= r.top + 1e-9
                && r.right() <= p.right() - ECART + 1e-9
                && r.bottom() <= p.bottom() - ECART + 1e-9,
            "{id} dans {parent}, sous son titre : {r:?} dans {p:?}"
        );
    }
    assert_eq!(
        boite(&store, &board, "vide").width,
        240.0,
        "vide : sa taille"
    );
    // L'ordre de lecture : chaque paragraphe vient après le précédent — dessous dans sa colonne, ou
    // dans une colonne à droite.
    for i in 1..6 {
        let (a, b) = (
            boite(&store, &board, &format!("p{}", i - 1)),
            boite(&store, &board, &format!("p{i}")),
        );
        assert!(
            (b.left == a.left && b.top > a.top) || b.left > a.left,
            "p{i} vient après p{} : {b:?} après {a:?}",
            i - 1
        );
    }

    assert!(store.undo(), "un seul geste");
    assert_eq!(store.project, avant, "et il se défait exactement");
}

/// Une membrane qui ne porte rien, ou qui n'existe pas, ne range rien — et n'écrit rien.
#[test]
fn colonnes_1_rien_a_ranger() {
    let (mut store, board) = un_article();
    let avant = store.project.clone();
    assert_eq!(store.disposer_en_colonnes(&board, "vide", 1.6), 0);
    assert_eq!(store.disposer_en_colonnes(&board, "personne", 1.6), 0);
    assert_eq!(
        store.disposer_en_colonnes(&board, "p0", 1.6),
        0,
        "un texte ne porte rien"
    );
    assert_eq!(store.project, avant);
}

/// **TITRE-1 dans le document** : les membranes d'un article portent leur titre, et « Disposer en
/// colonnes » réserve au-dessus de chacune ce que son nom prend en plus d'un titre ordinaire —
/// aucun ne couvre le bloc du dessus, ni le bandeau de l'article.
#[test]
fn titre_1_disposer_ne_laisse_aucun_nom_couvrir_son_voisin() {
    use crate::membrane_titre::{elevation, surcroit, ELEVATION};
    let (mut store, board) = un_article();
    for i in 0..14 {
        let id = format!("t{i}");
        poser(&mut store, &board, &id, Some("section"), 140.0, false);
    }
    for id in ["article", "section", "vide"] {
        store.update_annotation(&board, id, |a| {
            if let Annotation::Membrane { text, .. } = a {
                *text = Some(format!("Le titre de {id}"));
            }
        });
    }
    store.disposer_en_colonnes(&board, "article", 1.6);
    let parent = boite(&store, &board, "article");
    let section = boite(&store, &board, "section");
    assert!(
        surcroit(section.width, section.height) > 10.0,
        "l'épreuve exerce un grand titre : {section:?}"
    );
    let mut enfants: Vec<Rect> = ["p0", "p1", "p2", "p3", "p4", "p5", "section", "vide"]
        .iter()
        .map(|id| boite(&store, &board, id))
        .collect();
    enfants.sort_by(|a, b| a.left.total_cmp(&b.left).then(a.top.total_cmp(&b.top)));
    for (k, r) in enfants.iter().enumerate() {
        let dessus = match k.checked_sub(1).map(|j| enfants[j]) {
            Some(a) if a.left == r.left => a.bottom() + ECART,
            _ => parent.top + TITRE_DE_MEMBRANE,
        };
        if *r == section {
            let haut_du_titre = r.top - elevation(r.width, r.height) + ELEVATION;
            assert!(haut_du_titre >= dessus - 1e-9, "le titre couvre {dessus}");
        }
    }
}
