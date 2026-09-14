//! MOUSE-1 — le geste de sélection à la souris, de bout en bout.
//!
//! Ces tests montent une application complète, y posent une carte, et **cliquent** : l'offset
//! traverse la mise en page réelle, pas une mise en page d'essai. C'est la seule façon
//! d'attraper un décalage entre ce qui est dessiné et ce que la souris vise.

use super::*;
use crate::app::GlucoseApp;
use crate::canvas::world_to_screen;
use crate::renderer::card::TEXT_ORIGIN;
use crate::renderer::richtext::hit::offset_to_x;
use crate::renderer::richtext::LINE_FACTOR;
use glucose_core::types::{Annotation, Viewport};

const TEXTE: &str = "le chat dort\nsur le tapis";
const LARGEUR: f64 = 400.0;
const CORPS: f32 = 14.0;

/// Une application avec une carte à l'origine, ouverte en édition, curseur au début.
fn app_en_edition() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    let mut carte = Annotation::text("c", 0.0, 0.0, TEXTE);
    if let Annotation::Text { width, height, .. } = &mut carte {
        *width = Some(LARGEUR);
        *height = Some(200.0);
    }
    app.store.add_annotation(&board, carte);
    app.start_text_edit("c".into(), TEXTE.into());
    app
}

/// Le point écran où le curseur se dessine pour `offset` — l'endroit exact où il faut cliquer
/// pour viser cet octet.
fn point_de(app: &GlucoseApp, offset: usize) -> (f64, f64) {
    let (layout, _) = app.editing_layout().expect("une carte en édition");
    let index = crate::renderer::richtext::hit::line_of_offset(&layout, offset);
    let line = &layout.lines[index];
    let x = offset_to_x(
        &app.renderer.typography,
        &layout,
        line,
        TEXTE,
        offset,
        crate::renderer::richtext::font_of(line.kind, CORPS),
    );
    let vp = app
        .store
        .active_board()
        .map(|b| b.viewport)
        .expect("un tableau");
    // Au milieu de la hauteur de ligne : viser le haut tomberait sur la ligne d'au-dessus.
    let monde = (
        (TEXT_ORIGIN.0 + x) as f64,
        (TEXT_ORIGIN.1 + (index as f32 + 0.5) * CORPS * LINE_FACTOR) as f64,
    );
    world_to_screen(monde.0, monde.1, &vp)
}

fn selection(app: &GlucoseApp) -> Selection {
    app.editing_session.as_ref().expect("session").selection
}

fn selectionne(app: &GlucoseApp) -> &str {
    selection(app).slice(TEXTE)
}

/// Un clic pose le curseur là où l'on a cliqué, sans rien sélectionner.
#[test]
fn test_a_single_click_puts_the_cursor_where_it_clicked() {
    let mut app = app_en_edition();
    let vise = TEXTE.find("chat").expect("le mot est là");
    assert!(app.click_text_at(point_de(&app, vise), 1, false));
    assert_eq!(selection(&app), Selection::at(vise));
    assert!(selection(&app).is_empty());
}

/// **Un double-clic prend le mot**, un triple le paragraphe, et un quatrième repose un
/// curseur — la boucle des navigateurs.
#[test]
fn test_the_click_count_chooses_the_grain() {
    let mut app = app_en_edition();
    let vise = TEXTE.find("chat").expect("le mot est là") + 1;
    let point = point_de(&app, vise);

    assert!(app.click_text_at(point, 1, false));
    assert_eq!(selectionne(&app), "");
    assert!(app.click_text_at(point, 2, false));
    assert_eq!(selectionne(&app), "chat");
    assert!(app.click_text_at(point, 3, false));
    assert_eq!(
        selectionne(&app),
        "le chat dort",
        "le paragraphe, pas les deux"
    );
    assert!(app.click_text_at(point, 4, false));
    assert_eq!(selectionne(&app), "", "le quatrième clic repose un curseur");
}

/// **`Maj`+clic étend depuis l'ancre**, sans avoir à glisser entre les deux points. C'est le
/// geste que l'utilisateur a nommé : sélectionner deux endroits qu'un glisser d'un trait
/// n'aurait pas reliés.
#[test]
fn test_shift_click_extends_from_the_anchor() {
    let mut app = app_en_edition();
    let debut = TEXTE.find("chat").expect("mot");
    let fin = TEXTE.find("tapis").expect("mot") + 5;

    assert!(app.click_text_at(point_de(&app, debut), 1, false));
    app.end_text_drag();
    assert!(app.click_text_at(point_de(&app, fin), 1, true));
    assert_eq!(selectionne(&app), "chat dort\nsur le tapis");
    assert_eq!(selection(&app).anchor, debut, "l'ancre n'a pas bougé");

    // Et un second `Maj`+clic, plus près, rétrécit au lieu d'inverser (SEL-1).
    app.end_text_drag();
    let milieu = TEXTE.find("dort").expect("mot");
    assert!(app.click_text_at(point_de(&app, milieu), 1, true));
    assert_eq!(selectionne(&app), "chat ");
}

/// Glisser étend la sélection au fil du mouvement, dans les deux sens.
#[test]
fn test_dragging_extends_as_the_mouse_moves() {
    let mut app = app_en_edition();
    let depart = TEXTE.find("chat").expect("mot");
    assert!(app.click_text_at(point_de(&app, depart), 1, false));

    let fin = TEXTE.find("dort").expect("mot") + 4;
    assert!(app.drag_text_to(point_de(&app, fin)));
    assert_eq!(selectionne(&app), "chat dort");

    // Revenir en arrière rétrécit, puis repart de l'autre côté.
    assert!(app.drag_text_to(point_de(&app, 0)));
    assert_eq!(selectionne(&app), "le ");
    assert!(selection(&app).anchor > selection(&app).head);
}

/// **Un glisser entamé par un double-clic garde des mots entiers.** C'est la propriété qui
/// distingue un vrai geste de sélection d'un simple suivi de curseur.
#[test]
fn test_a_drag_started_by_a_double_click_keeps_whole_words() {
    let mut app = app_en_edition();
    let dans_chat = TEXTE.find("chat").expect("mot") + 2;
    assert!(app.click_text_at(point_de(&app, dans_chat), 2, false));
    assert_eq!(selectionne(&app), "chat");

    // Au milieu du mot suivant : le mot entier vient quand même.
    let dans_dort = TEXTE.find("dort").expect("mot") + 2;
    assert!(app.drag_text_to(point_de(&app, dans_dort)));
    assert_eq!(selectionne(&app), "chat dort");
}

/// Relâcher termine le geste : bouger ensuite ne change plus la sélection.
#[test]
fn test_releasing_ends_the_drag() {
    let mut app = app_en_edition();
    assert!(app.click_text_at(point_de(&app, 3), 1, false));
    app.end_text_drag();
    assert!(!app.drag_text_to(point_de(&app, 12)));
    assert_eq!(selectionne(&app), "");
}

/// La maille se lit sur le nombre de clics, et rien d'autre.
#[test]
fn test_the_grain_cycles_with_the_click_count() {
    assert_eq!(granularity_of(1), Granularity::Char);
    assert_eq!(granularity_of(2), Granularity::Word);
    assert_eq!(granularity_of(3), Granularity::Paragraph);
    assert_eq!(granularity_of(4), Granularity::Char);
    assert_eq!(granularity_of(5), Granularity::Word);
}

// ── Ouvrir une carte en visant un mot ───────────────────────────────────────

const MARKDOWN: &str = "# Titre **net** ici";

/// Une application avec une carte Markdown **au repos** — personne ne l'édite encore.
fn app_au_repos() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    let mut carte = Annotation::text("c", 0.0, 0.0, MARKDOWN);
    if let Annotation::Text { width, height, .. } = &mut carte {
        *width = Some(LARGEUR);
        *height = Some(200.0);
    }
    app.store.add_annotation(&board, carte);
    app
}

/// **Viser un mot se fait dans la vue qu'on avait sous les yeux.** Au repos, les signes du
/// Markdown n'occupent aucune place ; calculer le clic dans la mise en page de la saisie
/// décalerait le mot désigné de toute la largeur des signes qui le précèdent — ici quatre
/// caractères, soit près d'un mot entier.
///
/// Le défaut n'était visible que sur un texte à signes : c'est pourquoi ce test en emploie un.
#[test]
fn test_opening_a_card_aims_in_the_view_the_user_was_looking_at() {
    let app = app_au_repos();
    // L'abscisse où « net » est dessiné **au repos**, signes effacés.
    let (layout, _) = app
        .card_layout_of("c", MARKDOWN, crate::renderer::richtext::TextMode::Rendered)
        .expect("une carte de texte");
    let line = &layout.lines[0];
    let cible = MARKDOWN.find("net").expect("le mot est là") + 1;
    let x = offset_to_x(
        &app.renderer.typography,
        &layout,
        line,
        MARKDOWN,
        cible,
        crate::renderer::richtext::font_of(line.kind, CORPS),
    );
    let vp = app
        .store
        .active_board()
        .map(|b| b.viewport)
        .expect("un tableau");
    let point = world_to_screen(
        (TEXT_ORIGIN.0 + x) as f64,
        (TEXT_ORIGIN.1 + 0.5 * CORPS * LINE_FACTOR) as f64,
        &vp,
    );

    let selection = app.selection_opening_at("c", MARKDOWN, point);
    assert_eq!(
        selection.slice(MARKDOWN),
        "net",
        "le double-clic doit prendre le mot visé, pas celui qui le suit"
    );
}
