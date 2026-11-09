//! AGENT-8 : une mise à jour d'agent ne change que ce que l'appel nomme (fiche 98).

use super::{appeler, board};
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, MembraneMode, StickyOperator};

fn membrane(app: &GlucoseApp, id: &str) -> Annotation {
    app.store
        .project
        .annotation(&board(app), id)
        .cloned()
        .expect("la membrane existe")
}

/// **Recolorer une membrane garde son titre ; la renommer garde sa couleur ; ni l'un ni l'autre ne
/// touche ce qu'un agent ne sait pas dire** (ici, son mode). Avant : chaque mise à jour bâtissait
/// une membrane neuve, et ce que l'appel ne redisait pas retombait à vide.
#[test]
fn agent_8_une_membrane_mise_a_jour_garde_ce_que_l_appel_ne_dit_pas() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Eau","x":0,"y":0}"#,
    );
    let b = board(&app);
    app.store.update_annotation(&b, "m", |a| {
        if let Annotation::Membrane { mode, .. } = a {
            *mode = MembraneMode::Stretched;
        }
    });
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r##"{"id":"m","couleur":"#aa3344"}"##,
    );
    let Annotation::Membrane {
        text, color, mode, ..
    } = membrane(&app, "m")
    else {
        panic!("une membrane")
    };
    assert_eq!(text.as_deref(), Some("Eau"), "recolorer garde le titre");
    assert_eq!(color.as_deref(), Some("#aa3344"));
    assert_eq!(
        mode,
        MembraneMode::Stretched,
        "et ce qu'un agent ne dit pas"
    );

    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Mer"}"#,
    );
    let Annotation::Membrane { text, color, .. } = membrane(&app, "m") else {
        panic!("une membrane")
    };
    assert_eq!(text.as_deref(), Some("Mer"));
    assert_eq!(
        color.as_deref(),
        Some("#aa3344"),
        "renommer garde la couleur"
    );
}

/// **Réécrire un post-it garde ce que l'humain y a mis** : son papier, son opérateur, sa taille
/// de lettre. Le texte change, et la boîte suit le texte.
#[test]
fn agent_8_un_texte_reecrit_garde_ce_que_l_humain_y_a_mis() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r##"{"id":"p","texte":"Avant.","post_it":true,"couleur":"#336699"}"##,
    );
    let b = board(&app);
    app.store.update_annotation(&b, "p", |a| {
        if let Annotation::Sticky {
            operator,
            font_size,
            ..
        } = a
        {
            (*operator, *font_size) = (Some(StickyOperator::Because), Some(22.0));
        }
    });
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p","texte":"Après, bien plus long.","post_it":true}"#,
    );
    let Annotation::Sticky {
        text,
        bg_color,
        operator,
        font_size,
        ..
    } = membrane(&app, "p")
    else {
        panic!("un post-it")
    };
    assert_eq!(text, "Après, bien plus long.");
    assert_eq!(bg_color.as_deref(), Some("#336699"), "son papier reste");
    assert_eq!(operator, Some(StickyOperator::Because), "l'opérateur reste");
    assert_eq!(font_size, Some(22.0), "la taille de lettre reste");
}
