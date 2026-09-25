//! **L'éditeur d'ancres, par les vraies entrées** : la barre, la souris, le clavier.

use crate::app::GlucoseApp;
use crate::ui::ancrage::Etape;
use glucose_core::text_anchors::resolve_text_sel;
use glucose_core::types::{Annotation, TextSelection};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState, NamedKey};

const SOURCE: &str = "bonjours\ntest\ntest\nbonjours";
const CIBLE: &str = "aurevoire\ntest\ntest\naurevoire";
const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Deux cartes, une flèche de l'une à l'autre, sélectionnée.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("source", 100.0, 150.0, SOURCE));
    app.store
        .add_annotation(&board, Annotation::text("cible", 700.0, 150.0, CIBLE));
    let mut f = Annotation::arrow("f", 220.0, 200.0, 820.0, 200.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some("source".into());
        *target_id = Some("cible".into());
    }
    app.store.add_annotation(&board, f);
    // Mesurées, comme la saisie et l'import les mesurent (TEXT-FIT-1) : la boîte d'une carte
    // contient ses lignes.
    app.fit_text_card_height("source");
    app.fit_text_card_height("cible");
    app.store.clear_selection();
    app.store.select_annotation("f".into(), false);
    app.store.journal.clear();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app
}

fn aller(app: &mut GlucoseApp, (x, y): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
}

fn clic(app: &mut GlucoseApp, p: (f64, f64)) {
    aller(app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

fn touche(app: &mut GlucoseApp, nom: NamedKey) {
    assert!(app.touche_de_l_ancrage(&Key::Named(nom)));
}

/// Le point d'écran d'un octet d'une carte — sur la ligne qui le porte, là où il se dessine.
fn point(app: &GlucoseApp, carte: &str, texte: &str, octet: usize) -> (f64, f64) {
    let board = app.store.active_board().expect("un tableau");
    let ann = board
        .annotations
        .iter()
        .find(|a| a.id() == carte)
        .expect("la carte");
    let (w, _) = ann.size().expect("une taille");
    let fin = texte[octet..]
        .chars()
        .next()
        .map_or(octet, |c| octet + c.len_utf8());
    let r = crate::renderer::passages::rectangles(
        (&app.renderer.typography, &app.renderer.math),
        texte,
        w as f32,
        &[(octet, fin)],
    )[0];
    let monde = (
        ann.x() + f64::from(r.0) + 1.0,
        ann.y() + f64::from(r.1 + r.3 / 2.0),
    );
    crate::canvas::world_to_screen(monde.0, monde.1, &app.store.viewport())
}

/// « Ancrer… » dans la barre d'options.
fn ouvrir(app: &mut GlucoseApp) {
    let barre = crate::ui::options_de_fleche::layout_options_de_fleche(
        &app.store,
        &app.renderer.typography,
        ECRAN,
        app.ui.scale_factor,
    )
    .expect("la barre");
    let b = barre
        .boutons
        .iter()
        .find(|b| b.contenu == crate::ui::options_de_fleche::Contenu::Texte("Ancrer…"))
        .expect("le bouton Ancrer…");
    clic(
        app,
        (
            f64::from(b.rect.0 + 4.0),
            f64::from(b.rect.1 + b.rect.3 / 2.0),
        ),
    );
    // La caméra part vers la carte ; l'épreuve vise dans la vue d'où elle part.
    app.vol.poser();
}

/// Ce qu'un côté de la flèche désigne, tel quel dans son texte.
fn designe(app: &GlucoseApp, source: bool) -> Vec<String> {
    let board = app.store.active_board().expect("un tableau");
    let Some(Annotation::Arrow {
        source_text_sel,
        target_text_sel,
        ..
    }) = board.annotations.iter().find(|a| a.id() == "f")
    else {
        panic!("la flèche");
    };
    let (texte, sel): (&str, Option<&TextSelection>) = if source {
        (SOURCE, source_text_sel.as_ref())
    } else {
        (CIBLE, target_text_sel.as_ref())
    };
    resolve_text_sel(texte, sel)
        .into_iter()
        .map(|r| format!("{}@{}", &texte[r.start..r.end], r.start))
        .collect()
}

/// **De bout en bout** : un clic prend le second « bonjours », `Entrée` passe à la cible, un
/// glisser y prend « aure », `Entrée` termine — et un seul `Ctrl+Z` défait tout.
#[test]
fn test_fleche_4_ancrer_de_bout_en_bout() {
    let mut app = application();
    ouvrir(&mut app);
    assert_eq!(
        app.ui.ancrage.as_ref().map(|a| a.etape),
        Some(Etape::Source)
    );
    let second = SOURCE.rfind("bonjours").expect("le second");
    let p = point(&app, "source", SOURCE, second + 3);
    clic(&mut app, p);
    touche(&mut app, NamedKey::Enter);
    assert_eq!(app.ui.ancrage.as_ref().map(|a| a.etape), Some(Etape::Cible));
    app.vol.poser();

    // Un glisser sur « aure » seulement — ce qu'un clic, qui prend le mot entier, ne peut pas
    // donner : c'est bien le glisser qui choisit.
    let p = point(&app, "cible", CIBLE, 0);
    aller(&mut app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    let p = point(&app, "cible", CIBLE, 4);
    aller(&mut app, p);
    app.handle_mouse_up(MouseButton::Left);
    touche(&mut app, NamedKey::Enter);

    assert!(app.ui.ancrage.is_none(), "l'éditeur se referme");
    assert_eq!(designe(&app, true), [format!("bonjours@{second}")]);
    assert_eq!(designe(&app, false), ["aure@0"]);
    assert!(app.store.undo(), "un geste");
    assert!(designe(&app, true).is_empty() && designe(&app, false).is_empty());
}

/// **`Échap` laisse tout comme avant**, et un clic hors de la carte ne désélectionne rien.
#[test]
fn test_fleche_4_echap_annule_et_un_faux_clic_ne_defait_rien() {
    let mut app = application();
    ouvrir(&mut app);
    clic(&mut app, (300.0, 650.0));
    assert!(app.ui.ancrage.is_some(), "toujours ouvert");
    assert!(
        app.ui
            .ancrage
            .as_ref()
            .is_some_and(|a| a.ancres().is_empty()),
        "un clic hors de la carte ne choisit rien"
    );
    assert_eq!(
        app.store.selected_arrows().len(),
        1,
        "toujours sélectionnée"
    );
    let p = point(&app, "source", SOURCE, 2);
    clic(&mut app, p);
    touche(&mut app, NamedKey::Escape);
    assert!(app.ui.ancrage.is_none());
    assert!(designe(&app, true).is_empty(), "rien n'est écrit");
}

/// **`Ctrl` ajoute un passage au lieu de remplacer** : les deux « bonjours », chacun le sien.
#[test]
fn test_fleche_4_ctrl_ajoute_un_passage() {
    let mut app = application();
    ouvrir(&mut app);
    let second = SOURCE.rfind("bonjours").expect("le second");
    let p = point(&app, "source", SOURCE, 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::CONTROL;
    let p = point(&app, "source", SOURCE, second + 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::empty();
    touche(&mut app, NamedKey::Enter);
    touche(&mut app, NamedKey::Enter);
    assert_eq!(
        designe(&app, true),
        ["bonjours@0".to_string(), format!("bonjours@{second}")]
    );
}
