//! TITRE-1 — la loi du titre d'une membrane, et son encre.

use super::*;

/// Une membrane neuve garde exactement le titre d'aujourd'hui.
#[test]
fn une_membrane_neuve_garde_le_titre_d_aujourd_hui() {
    let (l, h) = NAISSANCE;
    assert_eq!(echelle(l, h), 1.0);
    assert_eq!(elevation(l, h), ELEVATION);
    assert_eq!(surcroit(l, h), 0.0);
}

/// **La part de surface est la même pour toutes** : le carré du corps sur la surface ne dépend
/// pas de la taille de la membrane, dès qu'elle dépasse une neuve. Et l'ancienne loi — un corps
/// fixe — donnait à une grande région une part cent fois plus petite : la preuve à l'envers.
#[test]
fn chaque_membrane_consacre_a_son_nom_la_meme_part_de_sa_surface() {
    let part = |l: f64, h: f64| (CORPS * echelle(l, h)).powi(2) / (l * h);
    let neuve = part(NAISSANCE.0, NAISSANCE.1);
    for (l, h) in [
        (640.0, 480.0),
        (8300.0, 5412.8),
        (1000.0, 600.0),
        (90_000.0, 70.0),
    ] {
        let p = part(l, h);
        assert!(
            ((p - neuve) / neuve).abs() < 1e-12,
            "{l} × {h} : part {p}, une neuve {neuve}"
        );
        let ancienne = CORPS * CORPS / (l * h);
        if l * h > 50.0 * NAISSANCE.0 * NAISSANCE.1 {
            assert!(ancienne < neuve / 50.0, "l'ancienne loi gardait la part");
        }
    }
}

/// La loi ne regarde que la surface : une membrane haute et étroite a le titre d'une membrane
/// carrée de même surface.
#[test]
fn la_loi_ne_depend_que_de_la_surface() {
    assert!((echelle(4000.0, 1000.0) - echelle(2000.0, 2000.0)).abs() < 1e-12);
    assert!(
        (echelle(8300.0, 5412.8) - 24.0).abs() < 0.3,
        "l'article « Eau »"
    );
}

/// Rien de ce qui est plus petit qu'une membrane neuve ne rapetisse, et une taille absurde rend
/// le titre d'aujourd'hui.
#[test]
fn rien_ne_rapetisse_et_rien_d_absurde_ne_passe() {
    assert_eq!(echelle(10.0, 10.0), 1.0);
    assert_eq!(echelle(0.0, 0.0), 1.0);
    assert_eq!(echelle(f64::NAN, 300.0), 1.0);
    assert_eq!(echelle(f64::INFINITY, 300.0), 1.0);
    assert_eq!(echelle(-640.0, 480.0), 2.0);
}

/// L'encre d'une membrane titrée monte au-dessus de son bord de toute l'élévation de son titre ;
/// sans titre, elle est sa boîte ; un autre nœud aussi.
#[test]
fn l_encre_d_une_membrane_porte_son_titre() {
    let mut m = Annotation::membrane("m", 100.0, 1000.0, 3200.0, 2400.0);
    assert_eq!(m.encre(), m.bounds(), "sans titre, rien au-dessus");
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some("Eau".to_string());
    }
    let e = m.encre();
    assert_eq!(e.left, 100.0);
    assert_eq!(e.width, 3200.0);
    assert!((e.top - (1000.0 - 18.0 * 10.0)).abs() < 1e-9, "{e:?}");
    assert!((e.bottom() - 3400.0).abs() < 1e-9);
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some(String::new());
    }
    assert_eq!(m.encre(), m.bounds(), "un titre vide ne se dessine pas");
    let t = Annotation::text("t", 0.0, 0.0, "x");
    assert_eq!(t.encre(), t.bounds());
}

/// Une membrane titrée de 3 200 × 2 400 (`s` = 10) sur un tableau neuf.
fn une_grande_membrane() -> (crate::store::Store, String, f64) {
    let mut store = crate::store::Store::new("titre");
    let board = store.project.active_board_id.clone();
    let mut m = Annotation::membrane("m", 0.0, 0.0, 3200.0, 2400.0);
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some("Eau".to_string());
    }
    store.add_annotation(&board, m);
    (store, board, -elevation(3200.0, 2400.0))
}

/// **Cadrer une membrane montre son nom** : tout cadrer (et l'export qui en part), `F` sur elle, et
/// le vol vers elle prennent son encre — le titre au-dessus de son bord compris.
#[test]
fn cadrer_une_membrane_montre_son_nom() {
    let (mut store, board, haut) = une_grande_membrane();
    let tout = store.content_bounds(&board).expect("un contenu");
    assert!((tout.top - haut).abs() < 1e-9, "tout cadrer : {tout:?}");
    store.select_annotation("m".into(), false);
    let choisi = store.selection_bounds(&board).expect("une sélection");
    assert!((choisi.top - haut).abs() < 1e-9, "F : {choisi:?}");
    let b = store.active_board().expect("un tableau");
    let vol = crate::arrow::emprise(b, "m").expect("une emprise");
    assert!((vol.top - haut).abs() < 1e-9, "le vol : {vol:?}");
}
