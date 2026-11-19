//! TEXT-FIT-2 — un texte qui grandit pousse sa membrane, à l'ouverture et à la saisie (fiche 124).

use super::super::tests::{app, text_card};
use crate::app::GlucoseApp;
use crate::renderer::card::text_card_fit_height;
use glucose_core::geometry::Rect;
use glucose_core::store::zones::ECART;
use glucose_core::types::Annotation;

fn boite(app: &GlucoseApp, id: &str) -> Rect {
    app.store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == id))
        .and_then(Annotation::rect)
        .expect("une boîte")
}

fn de_hauteur(mut a: Annotation, h: f64) -> Annotation {
    if let Annotation::Text { height, .. } = &mut a {
        *height = Some(h);
    }
    a
}

/// Un tableau dont les cellules se coupent dans un nœud de 900 — comme ceux que Claude.ai a posés
/// le 19/11 dans « Textes optimisés ».
fn un_tableau_qui_se_coupe() -> String {
    format!(
        "| Comportement | Pourquoi |\n|---|---|\n| aujourd'hui | {} |",
        "une phrase qui ne tient pas sur une ligne ".repeat(12)
    )
}

/// **À l'ouverture** : un document écrit par une version où le tableau ne se coupait pas porte la
/// hauteur d'alors, et une membrane qui la contenait juste. La mise en page neuve le fait grandir ;
/// la membrane le suit, le bas seulement. Preuve à l'envers : sans la loi, le texte en sortait.
#[test]
fn test_text_fit_2_a_l_ouverture_un_texte_grandi_pousse_sa_membrane() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    let texte = un_tableau_qui_se_coupe();
    let (typo, math) = (&app.renderer.typography, &app.renderer.math);
    // La hauteur d'une version qui ne coupait pas : le tableau sur une ligne par rangée.
    let d_avant = text_card_fit_height(typo, math, &texte, 1.0e6);
    let neuve = text_card_fit_height(typo, math, &texte, 900.0);
    // Le cas est exercé quand le texte neuf sort de la membrane d'avant, qui le contenait à l'écart
    // du cadre près — ni plus, ni moins (fiche 125 : un « + 100 » choisi l'avait rendue fausse partout).
    assert!(
        neuve > d_avant + ECART,
        "le cas n'est pas exercé : le texte neuf ({neuve}) tient encore dans la membrane d'avant ({d_avant} + {ECART})"
    );
    let haut = 42.0;
    let membrane = Annotation::membrane("m", 0.0, 0.0, 960.0, haut + d_avant + ECART);
    app.store.add_annotation(&board, membrane);
    let carte = de_hauteur(text_card("t", 30.0, haut, 900.0, &texte), d_avant);
    app.store.add_annotation(&board, carte);
    let cadre_d_avant = boite(&app, "m");
    let dir = crate::epreuve::Dossier::nouveau("text-fit-2");
    let path = dir.join("avant.glucose");
    app.save_to(path.clone());

    // Une seule fenêtre écrit un document : celle-ci le ferme avant qu'une autre le rouvre.
    assert!(app.fermer_le_document());
    let mut rouvert = GlucoseApp::new();
    rouvert.open_from(path);
    let (t, m) = (boite(&rouvert, "t"), boite(&rouvert, "m"));
    assert!(
        (t.height - neuve).abs() < 1e-6,
        "{} contre {neuve}",
        t.height
    );
    assert!(
        cadre_d_avant.bottom() < t.bottom(),
        "l'ancienne loi laissait le texte sortir de sa membrane"
    );
    assert!(
        (m.bottom() - (t.bottom() + ECART)).abs() < 1e-6,
        "{m:?} pour {t:?}"
    );
    assert_eq!(
        (m.left, m.top, m.width),
        (cadre_d_avant.left, cadre_d_avant.top, cadre_d_avant.width),
        "seul le bas bouge"
    );
    assert!(
        !rouvert.is_dirty(),
        "recaler à l'ouverture n'est pas une modification"
    );
    drop((app, rouvert));
}

/// **À la saisie** : un texte qu'on allonge pousse le bas de sa membrane dans le même geste, et un
/// seul `Ctrl+Z` rend le texte, sa hauteur et la membrane d'avant.
#[test]
fn test_text_fit_2_une_saisie_pousse_sa_membrane_et_un_ctrl_z_la_rend() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::membrane("m", 0.0, 0.0, 300.0, 140.0));
    app.store
        .add_annotation(&board, text_card("t", 30.0, 42.0, 240.0, "une ligne"));
    app.fit_text_card_height("t");
    let (cadre, ligne) = (boite(&app, "m"), boite(&app, "t"));
    assert!(
        ligne.bottom() + ECART <= cadre.bottom(),
        "le texte y tient d'abord"
    );
    let entrees = app.store.undo_depth();

    let long = (1..=12)
        .map(|i| format!("ligne {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    app.start_text_edit("t".into(), long);
    app.commit_editing();
    let (m, t) = (boite(&app, "m"), boite(&app, "t"));
    assert!(t.bottom() > cadre.bottom(), "le cas n'est pas exercé");
    assert!(
        (m.bottom() - (t.bottom() + ECART)).abs() < 1e-6,
        "{m:?} pour {t:?}"
    );
    assert_eq!(app.store.undo_depth(), entrees + 1, "une saisie = un geste");
    assert!(app.store.undo());
    assert!((boite(&app, "m").bottom() - cadre.bottom()).abs() < 1e-6);
    assert!((boite(&app, "t").bottom() - ligne.bottom()).abs() < 1e-6);
}
