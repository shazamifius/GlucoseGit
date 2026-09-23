//! ARROW-1 — la géométrie d'une flèche, tenue sans écran.

use super::*;
use crate::types::Point2D;

/// Une flèche qui ne s'accroche à rien : le résolveur ne connaît aucun nœud.
fn libre(_: &str) -> Option<crate::geometry::Rect> {
    None
}

fn fleche(id: &str, a: (f64, f64), b: (f64, f64)) -> Annotation {
    Annotation::arrow(id, a.0, a.1, b.0, b.1)
}

fn coudee(id: &str, a: (f64, f64), par: &[(f64, f64)], b: (f64, f64)) -> Annotation {
    let mut f = fleche(id, a, b);
    if let Annotation::Arrow { waypoints, .. } = &mut f {
        *waypoints = par.iter().map(|(x, y)| Point2D { x: *x, y: *y }).collect();
    }
    f
}

/// Le chemin part de l'origine, passe par les étapes dans l'ordre, et finit à la pointe.
#[test]
fn test_arrow_1_the_path_goes_from_tail_to_head_through_its_waypoints() {
    let f = coudee("a", (0.0, 0.0), &[(10.0, 5.0), (20.0, 5.0)], (30.0, 0.0));
    assert_eq!(
        path(&f).expect("une flèche"),
        vec![(0.0, 0.0), (10.0, 5.0), (20.0, 5.0), (30.0, 0.0)]
    );
}

/// Ce qui n'est pas une flèche n'a pas de chemin — et ce n'est pas un chemin vide.
#[test]
fn test_arrow_1_what_is_not_an_arrow_has_no_path() {
    assert_eq!(path(&Annotation::text("t", 0.0, 0.0, "x")), None);
    assert_eq!(
        distance_to(&Annotation::text("t", 0.0, 0.0, "x"), (0.0, 0.0)),
        None
    );
}

/// La distance à un segment : au milieu, en face d'une extrémité, et au-delà.
#[test]
fn test_arrow_1_the_distance_falls_back_on_the_ends() {
    let f = fleche("a", (0.0, 0.0), (100.0, 0.0));
    // En face du milieu : la perpendiculaire.
    assert!((distance_to(&f, (50.0, 10.0)).expect("d") - 10.0).abs() < 1e-9);
    // Sur le trait : rien.
    assert!(distance_to(&f, (50.0, 0.0)).expect("d") < 1e-9);
    // Au-delà de la pointe : c'est la pointe qui est le point le plus proche, pas la droite.
    assert!((distance_to(&f, (130.0, 0.0)).expect("d") - 30.0).abs() < 1e-9);
    assert!((distance_to(&f, (-30.0, 0.0)).expect("d") - 30.0).abs() < 1e-9);
}

/// Une flèche réduite à un point ne divise pas par zéro.
#[test]
fn test_arrow_1_a_degenerate_arrow_is_still_measurable() {
    let f = fleche("a", (7.0, 7.0), (7.0, 7.0));
    assert!((distance_to(&f, (7.0, 10.0)).expect("d") - 3.0).abs() < 1e-9);
}

/// Un coude se mesure sur le tronçon le plus proche, pas sur la corde.
#[test]
fn test_arrow_1_a_bend_is_measured_on_its_nearest_leg() {
    let f = coudee("a", (0.0, 0.0), &[(50.0, 50.0)], (100.0, 0.0));
    // Le point est à cinq unités sous le sommet du coude.
    assert!(distance_to(&f, (50.0, 45.0)).expect("d") <= 5.0 + 1e-9);
    // Le milieu de la corde, lui, est loin des deux tronçons.
    assert!(distance_to(&f, (50.0, 0.0)).expect("d") > 30.0);
}

/// La bande garde une épaisseur **écran** : viser une flèche ne devient pas plus dur au
/// dézoom, ce qui est exactement ce que `non-scaling-stroke` fait côté Tauri.
#[test]
fn test_arrow_1_the_band_keeps_its_screen_width() {
    let fleches = [fleche("a", (0.0, 0.0), (100.0, 0.0))];
    // À l'échelle 1, la bande vaut 24 px : on attrape jusqu'à 12 unités monde.
    assert!(at(&fleches, libre, (50.0, 11.0), 1.0).is_some());
    assert!(at(&fleches, libre, (50.0, 13.0), 1.0).is_none());
    // Au demi-zoom, la même bande écran couvre deux fois plus de monde.
    assert!(at(&fleches, libre, (50.0, 23.0), 0.5).is_some());
    assert!(at(&fleches, libre, (50.0, 25.0), 0.5).is_none());
}

/// Entre deux flèches qui se croisent, la plus proche gagne ; à égalité, celle du dessus.
#[test]
fn test_arrow_1_the_nearest_wins_and_ties_go_to_the_top_one() {
    let fleches = [
        fleche("dessous", (0.0, 0.0), (100.0, 0.0)),
        fleche("dessus", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, libre, (50.0, 1.0), 1.0).expect("une flèche");
    assert_eq!(
        gagnante.id(),
        "dessus",
        "à égalité, celle dessinée au-dessus"
    );

    let fleches = [
        fleche("loin", (0.0, 20.0), (100.0, 20.0)),
        fleche("pres", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, libre, (50.0, 2.0), 1.0).expect("une flèche");
    assert_eq!(gagnante.id(), "pres");
}

/// Rien à portée : rien. Une flèche lointaine ne se sélectionne pas par accident.
#[test]
fn test_arrow_1_nothing_within_reach_selects_nothing() {
    let fleches = [fleche("a", (0.0, 0.0), (100.0, 0.0))];
    assert!(at(&fleches, libre, (50.0, 400.0), 1.0).is_none());
    assert!(at(&[], libre, (0.0, 0.0), 1.0).is_none());
}

// ── L'ancrage : une flèche s'arrête sur le bord de ce qu'elle vise ─────────

use crate::geometry::Rect;

fn ancree(id: &str, de: Option<&str>, vers: Option<&str>) -> Annotation {
    let mut f = fleche(id, (0.0, 0.0), (400.0, 0.0));
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = de.map(str::to_string);
        *target_id = vers.map(str::to_string);
    }
    f
}

/// Une flèche qui vise un nœud s'arrête sur son **bord**, pas sur son centre.
///
/// C'est ce que `arrow_anchor` savait faire depuis toujours — cent soixante-dix-sept lignes
/// écrites, testées, et que personne n'appelait.
#[test]
fn test_arrow_1_an_anchored_arrow_stops_at_the_border() {
    // Une boîte de 200 de large centrée en (400, 0) : son bord gauche est à 300.
    let resolve = |id: &str| (id == "cible").then(|| Rect::new(300.0, -50.0, 200.0, 100.0));
    let f = ancree("a", None, Some("cible"));

    let points = path_with(&f, resolve).expect("un chemin");
    let (fin_x, fin_y) = *points.last().expect("une pointe");
    assert!(
        fin_x < 300.0,
        "la pointe s'arrête avant le bord gauche ({fin_x}), pas au centre"
    );
    assert!(fin_y.abs() < 1.0, "elle reste sur l'axe");

    // Sans ancrage, elle irait jusqu'à son point brut.
    let brut = path(&f).expect("un chemin");
    assert_eq!(brut.last(), Some(&(400.0, 0.0)));
}

/// Une flèche sans ancre garde exactement son tracé : l'ancrage ne coûte rien à qui n'en veut
/// pas, et `path_with` rend alors la même chose que `path`.
#[test]
fn test_arrow_1_an_unanchored_arrow_keeps_its_raw_path() {
    let f = fleche("a", (0.0, 0.0), (400.0, 0.0));
    assert_eq!(path_with(&f, libre), path(&f));
}

/// Un nœud introuvable ne fait pas disparaître la flèche : elle retombe sur son point brut.
#[test]
fn test_arrow_1_a_missing_node_falls_back_on_the_raw_point() {
    let f = ancree("a", None, Some("fantome"));
    assert_eq!(path_with(&f, libre), path(&f));
}

/// Et le clic suit l'ancrage : on vise la flèche là où elle se dessine.
#[test]
fn test_arrow_1_the_click_follows_the_anchor() {
    let resolve = |id: &str| (id == "cible").then(|| Rect::new(300.0, -50.0, 200.0, 100.0));
    let fleches = [ancree("a", None, Some("cible"))];
    // Un point bien à l'intérieur de la boîte visée : le trait n'y va plus.
    assert!(at(&fleches, resolve, (450.0, 0.0), 1.0).is_none());
    // Alors qu'avant le bord, il est toujours là.
    assert!(at(&fleches, resolve, (200.0, 0.0), 1.0).is_some());
}

// ── ARROW-2 : une flèche s'accroche au dessin ─────────────────────────────

use crate::types::{Board, BoardImage};

fn plateau(cartes: &[(&str, f64, f64, f64, f64)]) -> Board {
    let mut board = Board::new("b", "plateau");
    for (id, x, y, w, h) in cartes {
        let mut carte = Annotation::text(*id, *x, *y, "x");
        if let Annotation::Text { width, height, .. } = &mut carte {
            *width = Some(*w);
            *height = Some(*h);
        }
        board.annotations.push(carte);
    }
    board
}

/// Rien à portée : le bout se pose là où on l'a mis, sans s'accrocher.
#[test]
fn test_arrow_2_nothing_within_reach_stays_free() {
    let board = plateau(&[("carte", 0.0, 0.0, 100.0, 60.0)]);
    let snap = snap_to_nearest(&board, (900.0, 900.0), &[]);
    assert_eq!(snap, Snap::free((900.0, 900.0)));
}

/// Assez près d'une carte : le bout se pose sur son **bord**, et retient son identifiant.
#[test]
fn test_arrow_2_a_nearby_node_captures_the_end_on_its_border() {
    let board = plateau(&[("carte", 0.0, 0.0, 100.0, 60.0)]);
    // Cinquante unités à droite du bord droit : dans la portée de 120.
    let snap = snap_to_nearest(&board, (150.0, 30.0), &[]);
    assert_eq!(snap.node.as_deref(), Some("carte"));
    assert_eq!(snap.point, (100.0, 30.0), "le point du bord le plus proche");
}

/// La portée est exactement celle de Glucose Tauri, et elle se mesure au **bord**.
#[test]
fn test_arrow_2_the_reach_is_measured_from_the_border() {
    let board = plateau(&[("carte", 0.0, 0.0, 100.0, 60.0)]);
    let juste_dedans = snap_to_nearest(&board, (100.0 + SNAP_DIST - 1.0, 30.0), &[]);
    assert_eq!(juste_dedans.node.as_deref(), Some("carte"));
    let juste_dehors = snap_to_nearest(&board, (100.0 + SNAP_DIST + 1.0, 30.0), &[]);
    assert_eq!(juste_dehors.node, None);
}

/// À l'intérieur d'une carte, le point retenu est le curseur : on pointe où l'on veut.
#[test]
fn test_arrow_2_inside_a_node_the_cursor_itself_is_kept() {
    let board = plateau(&[("carte", 0.0, 0.0, 200.0, 100.0)]);
    let snap = snap_to_nearest(&board, (37.0, 61.0), &[]);
    assert_eq!(snap.node.as_deref(), Some("carte"));
    assert_eq!(snap.point, (37.0, 61.0));
}

/// Une **grande** carte visée au bord gagne contre une petite plus loin.
///
/// C'est ce que la distance au bord donne et qu'une distance au centre raterait : le centre
/// d'une carte de mille unités de large est à cinq cents unités de son propre bord.
#[test]
fn test_arrow_2_a_large_node_grabbed_by_its_edge_beats_a_distant_small_one() {
    let mut board = plateau(&[("grande", 0.0, 0.0, 1000.0, 400.0)]);
    board
        .annotations
        .push(Annotation::text("petite", 1020.0, 0.0, "x"));
    if let Some(Annotation::Text { width, height, .. }) = board.annotations.last_mut() {
        *width = Some(20.0);
        *height = Some(20.0);
    }
    // Dix unités à droite du bord de la grande, dix à gauche de la petite.
    let snap = snap_to_nearest(&board, (1010.0, 10.0), &[]);
    assert_eq!(
        snap.node.as_deref(),
        Some("grande"),
        "le bord le plus proche gagne, quelle que soit la taille"
    );
}

/// Les images et les dossiers s'aimantent au même titre que les cartes.
#[test]
fn test_arrow_2_images_are_snap_targets_too() {
    let mut board = Board::new("b", "plateau");
    board
        .images
        .push(BoardImage::new("img", 0.0, 0.0, 80.0, 80.0));
    let snap = snap_to_nearest(&board, (120.0, 40.0), &[]);
    assert_eq!(snap.node.as_deref(), Some("img"));
}

/// Ce qui est exclu n'est jamais visé, même collé au curseur.
#[test]
fn test_arrow_2_an_excluded_node_is_never_targeted() {
    let board = plateau(&[("carte", 0.0, 0.0, 100.0, 60.0)]);
    let snap = snap_to_nearest(&board, (110.0, 30.0), &["carte"]);
    assert_eq!(snap.node, None);
    assert_eq!(snap.point, (110.0, 30.0));
}

/// Pendant un tracé, la pointe n'attrape ni la flèche elle-même ni son origine.
///
/// Sans cette exclusion, une flèche partie d'une carte se refermerait sur cette carte dès
/// le premier pixel de glisser : son origine est le nœud le plus proche qui soit.
#[test]
fn test_arrow_2_a_tip_never_grabs_its_own_source() {
    let mut board = plateau(&[("origine", 0.0, 0.0, 100.0, 60.0)]);
    let mut fleche = Annotation::arrow("a", 100.0, 30.0, 110.0, 30.0);
    if let Annotation::Arrow { source_id, .. } = &mut fleche {
        *source_id = Some("origine".to_string());
    }
    board.annotations.push(fleche);

    // Juste à côté de l'origine : sans exclusion, elle gagnerait.
    let snap = snap_for_tip(&board, "a", (110.0, 30.0));
    assert_eq!(snap.node, None, "ni l'origine, ni la flèche elle-même");

    // Et une autre carte, elle, reste visable.
    board.annotations.push({
        let mut c = Annotation::text("autre", 200.0, 0.0, "x");
        if let Annotation::Text { width, height, .. } = &mut c {
            *width = Some(50.0);
            *height = Some(50.0);
        }
        c
    });
    let snap = snap_for_tip(&board, "a", (190.0, 25.0));
    assert_eq!(snap.node.as_deref(), Some("autre"));
}

// ── Le point d'ancrage de ce qu'une flèche dit ────────────────────────────

/// Sur une flèche droite, l'étiquette se pose au milieu.
#[test]
fn test_the_label_sits_at_the_middle_of_a_straight_arrow() {
    let f = fleche("a", (0.0, 0.0), (100.0, 40.0));
    assert_eq!(label_anchor(&f, libre), Some((50.0, 20.0)));
}

/// Sur une flèche coudée, elle se pose sur le **tronçon médian**, pas sur la corde.
///
/// C'est tout l'intérêt : le milieu de la corde d'un coude en « V » tombe dans le vide,
/// parfois très loin du trait, et l'étiquette s'y détacherait de ce qu'elle nomme.
#[test]
fn test_the_label_follows_the_bend_rather_than_the_chord() {
    // Un « V » profond : la corde passe à cent unités au-dessus du sommet.
    let f = coudee("a", (0.0, 0.0), &[(50.0, 100.0)], (100.0, 0.0));
    let ancre = label_anchor(&f, libre).expect("une ancre");

    let corde = (50.0, 0.0);
    assert!(
        (ancre.1 - corde.1).abs() > 40.0,
        "l'ancre {ancre:?} est retombée sur la corde"
    );
    // Elle est bien sur l'un des deux tronçons : ici le second, de (50,100) à (100,0).
    assert!(distance_to(&f, ancre).expect("d") < 1e-9, "{ancre:?}");
}

/// Sur une flèche à plusieurs coudes, l'ancre reste sur un tronçon du tracé.
#[test]
fn test_the_label_stays_on_the_path_whatever_the_number_of_bends() {
    for etapes in [
        vec![(30.0, 60.0)],
        vec![(30.0, 60.0), (70.0, -40.0)],
        vec![(20.0, 50.0), (50.0, -50.0), (80.0, 30.0)],
    ] {
        let f = coudee("a", (0.0, 0.0), &etapes, (100.0, 0.0));
        let ancre = label_anchor(&f, libre).expect("une ancre");
        assert!(
            distance_to(&f, ancre).expect("d") < 1e-9,
            "{} coudes : l'ancre {ancre:?} n'est pas sur le tracé",
            etapes.len()
        );
    }
}

/// L'ancre suit l'**ancrage** : une flèche qui s'arrête au bord d'un nœud porte son
/// étiquette sur le tracé raccourci, pas sur le tracé brut.
#[test]
fn test_the_label_follows_the_anchored_path() {
    let resolve = |id: &str| (id == "cible").then(|| Rect::new(300.0, -50.0, 200.0, 100.0));
    let f = ancree("a", None, Some("cible"));
    let ancree_pt = label_anchor(&f, resolve).expect("une ancre");
    let brute = label_anchor(&f, libre).expect("une ancre");
    assert!(
        ancree_pt.0 < brute.0,
        "l'ancre ancrée ({ancree_pt:?}) devrait précéder la brute ({brute:?})"
    );
}

/// Ce qui n'est pas une flèche n'a pas d'ancre d'étiquette.
#[test]
fn test_what_is_not_an_arrow_has_no_label_anchor() {
    assert_eq!(
        label_anchor(&Annotation::text("t", 0.0, 0.0, "x"), libre),
        None
    );
}

// ── Les liens trans-domaines (fiche 03 § 11.6) ──────────────────────────────────────

/// Un nœud qui porte ces domaines, à poids égal.
fn porteur(id: &str, domaines: &[&str]) -> Annotation {
    let mut a = Annotation::text(id, 0.0, 0.0, id);
    for d in domaines {
        a.domains_mut().push(crate::types::DomainAssignment {
            domain_id: (*d).to_string(),
            weight: 0.5,
        });
    }
    a
}

/// Une flèche de `source` vers `cible`.
fn liant(source: &str, cible: &str) -> Annotation {
    let mut f = fleche("f", (0.0, 0.0), (10.0, 0.0));
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some(source.to_string());
        *target_id = Some(cible.to_string());
    }
    f
}

/// **Un lien est trans-domaine quand ses deux bouts portent des domaines et n'en partagent
/// aucun** — et seulement alors : un seul domaine commun suffit à le garder dans son
/// territoire, et un bout sans domaine ne dit rien de ce que la flèche traverse.
#[test]
fn test_un_lien_trans_domaine_ne_partage_aucun_domaine() {
    let noeuds = [
        porteur("physique", &["sciences"]),
        porteur("chimie", &["sciences", "industrie"]),
        porteur("peinture", &["arts"]),
        porteur("brouillon", &[]),
    ];
    let domaines = |id: &str| domaines_du_noeud(&[], &noeuds, id);
    assert!(est_trans_domaine(&liant("physique", "peinture"), domaines));
    assert!(
        !est_trans_domaine(&liant("physique", "chimie"), domaines),
        "un domaine commun"
    );
    assert!(
        !est_trans_domaine(&liant("physique", "brouillon"), domaines),
        "un bout sans domaine"
    );
    assert!(
        !est_trans_domaine(&liant("physique", "inconnu"), domaines),
        "un bout introuvable"
    );
    assert!(
        !est_trans_domaine(&fleche("libre", (0.0, 0.0), (5.0, 5.0)), domaines),
        "une flèche libre"
    );
}

/// **Une image est une extrémité comme une autre** : ses domaines se lisent aussi.
#[test]
fn test_une_image_porte_ses_domaines_au_bout_d_une_fleche() {
    let mut photo = crate::types::BoardImage::new("photo", 0.0, 0.0, 10.0, 10.0);
    photo.domains.push(crate::types::DomainAssignment {
        domain_id: "arts".into(),
        weight: 1.0,
    });
    let images = [photo];
    let noeuds = [porteur("physique", &["sciences"])];
    let domaines = |id: &str| domaines_du_noeud(&images, &noeuds, id);
    assert!(est_trans_domaine(&liant("physique", "photo"), domaines));
}
