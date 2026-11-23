//! ANCRE-1 — une flèche se trouve là où elle se dessine, jamais à la copie de ses bouts (fiche 135 ;
//! sa note du 23/11, `text-kpll-375`).

use super::*;
use crate::arrow::{emprise, node_rect};
use crate::hit_priority::{collect_candidates_indexed, emprise_du_groupe, PickInput, PickKind};
use crate::quadtree::SpatialHash;
use crate::store::Store;

/// Une flèche de `source` à `cible`, sa copie posée au centre de chacun — comme la main la pose.
fn fleche(store: &Store, id: &str, source: &str, cible: &str) -> Annotation {
    let b = store.active_board().expect("un tableau");
    let centre = |n: &str| node_rect(b, n).expect("le nœud").center();
    let (s, c) = (centre(source), centre(cible));
    let mut a = Annotation::arrow(id, s.x, s.y, c.x, c.y);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut a
    {
        *source_id = Some(source.into());
        *target_id = Some(cible.into());
    }
    a
}

/// Sa scène : A d'où part la flèche `f`, B où elle arrive, C posé dessous.
fn scene() -> (Store, String) {
    let mut store = Store::new("ancre");
    let board = store.project.active_board_id.clone();
    for t in [
        Annotation::text("a", 0.0, 0.0, "Le texte A"),
        Annotation::text("b", 2000.0, 0.0, "Le texte B"),
        Annotation::text("c", 0.0, 1500.0, "Le texte C"),
    ] {
        store.add_annotation(&board, t);
    }
    let f = fleche(&store, "f", "a", "b");
    store.add_annotation(&board, f);
    (store, board)
}

fn la_fleche<'s>(store: &'s Store, id: &str) -> &'s Annotation {
    let b = store.active_board().expect("un tableau");
    b.annotations
        .iter()
        .find(|a| a.id() == id)
        .expect("la flèche")
}

/// Le milieu du trait qu'on voit : là où la main la reclique.
fn milieu_du_trait(store: &Store, id: &str) -> (f64, f64) {
    let b = store.active_board().expect("un tableau");
    let points = path_with(la_fleche(store, id), |n: &str| node_rect(b, n)).expect("un tracé");
    let (p, q) = (points[0], points[points.len() - 1]);
    ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0)
}

fn entree<'s>(store: &'s Store, (wx, wy): (f64, f64)) -> PickInput<'s> {
    let b = store.active_board().expect("un tableau");
    PickInput {
        wx,
        wy,
        scale: 1.0,
        images: &b.images,
        annotations: &b.annotations,
        folders: &b.folders,
        selected_image_ids: &store.selected_image_ids,
        selected_annotation_ids: &store.selected_annotation_ids,
        selected_folder_ids: &store.selected_folder_ids,
        noeuds: None,
    }
}

/// **Ce que la main prend en cliquant là** — par l'index, et avec ce qui mesure les nœuds de
/// tout le tableau, comme le vrai clic (`pick_candidates_at`).
fn ce_que_prend_le_clic(store: &Store, point: (f64, f64)) -> Option<(PickKind, String)> {
    let b = store.active_board().expect("un tableau");
    let mut index = SpatialHash::new(1000.0);
    index.index_board(b);
    let mesure = |id: &str| node_rect(b, id);
    let entree = PickInput {
        noeuds: Some(&mesure),
        ..entree(store, point)
    };
    let candidats = collect_candidates_indexed(&entree, &index);
    candidats.first().map(|c| (c.kind, c.id.clone()))
}

/// **L'ancienne loi** : chaque annotation rangée dans l'index par sa copie (son encre gardée) —
/// la flèche se trouve-t-elle près de ce point, avec la marge du clic ?
fn l_index_par_la_copie_la_trouve(store: &Store, id: &str, (x, y): (f64, f64)) -> bool {
    let mut ancien = SpatialHash::new(1000.0);
    for a in &store.active_board().expect("un tableau").annotations {
        ancien.insert_annotation(a);
    }
    let marge = crate::hit_priority::pick_consts::HANDLE_SLOP_PX * 2.0;
    ancien.query_rect_refs(x, y, x, y, marge).contains(id)
}

/// **Son geste** : la flèche choisie avec C, la sélection glissée loin ; puis, rien de choisi, un
/// clic sur la flèche là où on la voit la prend encore, et aucun cadre ne s'étend où elle n'est
/// pas. Preuve à l'envers : sa copie est partie avec le glisser — l'ancienne loi (l'index par la
/// copie) ne la trouvait plus.
#[test]
fn test_ancre_1_la_fleche_glissee_avec_un_texte_se_reprend_ou_on_la_voit() {
    let (mut store, board) = scene();
    store.select_annotation("f".into(), true);
    store.select_annotation("c".into(), true);
    store.move_selected(&board, 9000.0, 7000.0);
    let vue = milieu_du_trait(&store, "f");
    let copie = la_fleche(&store, "f").bounds();
    assert!(
        !copie.contains_point(crate::geometry::Point::new(vue.0, vue.1)),
        "la scène exerce le cas : la copie est partie loin du trait ({copie:?})"
    );
    // Le cadre du groupe : C et le trait, jamais la copie partie au loin.
    let cadre = emprise_du_groupe(&entree(&store, (0.0, 0.0))).expect("un groupe");
    let c = node_rect(store.active_board().expect("un tableau"), "c").expect("C");
    let trait_vu = enveloppe_with(la_fleche(&store, "f"), |n: &str| {
        node_rect(store.active_board().expect("un tableau"), n)
    });
    assert_eq!(
        Some(cadre),
        trait_vu.map(|t| t.union(c)),
        "le cadre du groupe"
    );
    let b = store.active_board().expect("un tableau");
    let redessinee = store.ce_qu_emporte_la_selection(&board).encre(b);
    assert_eq!(
        redessinee,
        Some(cadre),
        "la zone redessinée pendant le glisser"
    );
    store.clear_selection();
    assert_eq!(
        ce_que_prend_le_clic(&store, vue),
        Some((PickKind::Arrow, "f".into())),
        "la flèche se reprend là où on la voit"
    );
    assert!(
        !l_index_par_la_copie_la_trouve(&store, "f", vue),
        "à l'envers : l'index par la copie ne la trouvait plus"
    );
}

/// **Le rangement** (sa capture : la flèche de « archive ») : B part dans un dossier, la flèche le
/// vise ; elle se reprend sur son trait, et un rectangle tracé là où était B ne la prend pas.
#[test]
fn test_ancre_1_une_fleche_qui_vise_un_dossier_se_trouve_sur_son_trait() {
    let (mut store, board) = scene();
    let mut d = crate::types::CanvasFolder::new("archive", "archive", "");
    (d.x, d.y) = (-3000.0, 3000.0);
    store.create_folder(&board, d);
    store
        .ranger_dans_un_dossier(&board, "archive", &["b"])
        .expect("rangé");
    let vue = milieu_du_trait(&store, "f");
    assert_eq!(
        ce_que_prend_le_clic(&store, vue),
        Some((PickKind::Arrow, "f".into())),
        "la flèche se reprend sur son trait, de A au dossier"
    );
    assert!(
        !l_index_par_la_copie_la_trouve(&store, "f", vue),
        "à l'envers : l'index par la copie ne la trouvait plus"
    );
    let b = store.active_board().expect("un tableau");
    let la_ou_etait_b = crate::geometry::Rect::new(1900.0, -100.0, 600.0, 400.0);
    assert!(
        la_fleche(&store, "f").bounds().overlaps(la_ou_etait_b),
        "la scène exerce le cas : la copie est restée là où était B"
    );
    let pris = b.pris_par(la_ou_etait_b, |_| true);
    assert!(
        !pris.annotations.iter().any(|a| a == "f"),
        "le rectangle tracé là où était B ne prend pas la flèche"
    );
    let emprise = emprise(b, "f").expect("son emprise");
    assert!(
        !emprise.overlaps(la_ou_etait_b),
        "F cadre son trait ({emprise:?})"
    );
    let decrits = crate::store::agents::decrire_le_tableau(b, &[]);
    let dite = decrits.iter().find(|d| d.id == "f").expect("dite à l'IA");
    assert_eq!(dite.boite, emprise, "l'IA lit la boîte que l'humain voit");
    let tout = store.content_bounds(&board).expect("des bornes");
    assert!(
        !tout.overlaps(la_ou_etait_b),
        "Alt+F ne cadre pas un vide ({tout:?})"
    );
    store.select_annotation("f".into(), false);
    assert_eq!(
        store.selection_bounds(&board),
        Some(emprise),
        "F sur la flèche choisie"
    );
}

/// **SPAT-1 et SPAT-3 tenus** : une passe sur un tableau qui n'a pas changé n'écrit aucune cellule
/// et ne hache aucun nom — une flèche accrochée dont la copie est loin comprise ; un nœud déplacé
/// loin recale sa flèche.
#[test]
fn test_ancre_1_l_index_ne_paie_rien_quand_rien_ne_change() {
    let (mut store, board) = scene();
    // La copie d'abord envoyée loin du trait (son geste) : une passe qui la relirait écrirait.
    store.select_annotation("f".into(), true);
    store.select_annotation("c".into(), true);
    store.move_selected(&board, 9000.0, 7000.0);
    store.clear_selection();
    let mut index = SpatialHash::new(1000.0);
    index.index_board(store.active_board().expect("un tableau"));
    let (ecritures, hachages) = (index.cell_write_count(), index.hash_count());
    index.index_board(store.active_board().expect("un tableau"));
    assert_eq!(
        (index.cell_write_count(), index.hash_count()),
        (ecritures, hachages),
        "rien n'a changé : rien n'est écrit ni haché"
    );
    store.select_annotation("b".into(), false);
    store.move_selected(&board, 0.0, 9000.0);
    store.clear_selection();
    index.index_board(store.active_board().expect("un tableau"));
    assert_eq!(index.hash_count(), hachages, "un déplacement ne hache rien");
    let vue = milieu_du_trait(&store, "f");
    let rangs = index.query_rect_ranks(vue.0, vue.1, vue.0, vue.1, 0.0);
    let f = index.rang_de("f").expect("indexée");
    assert!(rangs.contains(&f), "la flèche suit B dans l'index");
}

/// **Un coude compte** : la flèche pliée loin sous A et B se reprend sur le tronçon qui y descend.
#[test]
fn test_ancre_1_un_coude_compte() {
    let (mut store, board) = scene();
    store.update_annotation(&board, "f", |f| {
        if let Annotation::Arrow { waypoints, .. } = f {
            waypoints.push(crate::types::Point2D {
                x: 1000.0,
                y: 6000.0,
            });
        }
    });
    let b = store.active_board().expect("un tableau");
    let points = path_with(la_fleche(&store, "f"), |n: &str| node_rect(b, n)).expect("un tracé");
    let (p, q) = (points[0], points[1]);
    let sur_le_troncon = ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
    assert_eq!(
        ce_que_prend_le_clic(&store, sur_le_troncon),
        Some((PickKind::Arrow, "f".into()))
    );
}
