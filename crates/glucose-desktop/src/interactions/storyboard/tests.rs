//! STORYBOARD-1 — les cases à la vraie souris : par leur bande, avec ce qu'elles portent ; le
//! cadre, qui reste au canevas ; la description, la croix, `Suppr`, `Échap`.

use crate::app::GlucoseApp;
use crate::ui::question::Suite;
use glucose_core::store::storyboard::{Format, Grille, DESCRIPTION};
use glucose_core::types::{BoardImage, StoryboardPanel, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn grille() -> Grille {
    Grille {
        format: Format::Cinema,
        largeur: 320.0,
        colonnes: 4,
        ecart: 24.0,
    }
}

/// Une application vue à l'échelle 1 depuis l'origine — un point de l'écran est le même point
/// du monde —, et un storyboard de huit cases centré en (720, 450).
fn app() -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    // La grille centrée sur l'écran, comme avant FANTOME-1 : son coin s'en déduit.
    let (l, h) = grille().etendue(grille().nombre_initial());
    app.store
        .poser_le_storyboard(&board, grille(), (720.0 - l / 2.0, 450.0 - h / 2.0));
    app.store.clear_selection();
    app.store.journal.clear();
    (app, board)
}

fn premiere(app: &GlucoseApp, board: &str) -> StoryboardPanel {
    app.store.cases(board)[0].clone()
}

/// Un point de la bande de cette case, sous son cadre.
fn dans_la_bande(c: &StoryboardPanel) -> (f64, f64) {
    (c.x + c.width - 60.0, c.y + c.height + DESCRIPTION / 2.0)
}

/// Appuie en `de`, glisse jusqu'à `a`, relâche.
fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.mouse_pos = de;
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

/// **La bande prend la case, et la case lâchée prend la place sous elle** : les autres glissent
/// d'un cran, l'image suit sa case, et un seul `Ctrl+Z` rend tout (STORYBOARD-2).
#[test]
fn test_storyboard_2_la_case_lachee_prend_la_place_sous_elle() {
    let (mut app, board) = app();
    let avant: Vec<StoryboardPanel> = app.store.cases(&board).into_iter().cloned().collect();
    let case = avant[0].clone();
    let centre = (case.x + case.width / 2.0, case.y + case.height / 2.0);
    app.store.add_image(
        &board,
        BoardImage::new("i", centre.0, centre.1, 200.0, 200.0),
    );
    // Ajustée dans sa case, comme toute image qu'on y lâche, et non choisie : les poignées d'un
    // nœud choisi passent avant tout, et celles de ses coins mordent sur la bande.
    app.store.ajuster_dans_les_cases(&board, &["i".to_string()]);
    app.store.clear_selection();
    app.store.journal.clear();
    let de = dans_la_bande(&case);
    // Deux places à droite : le centre de son cadre passe dans la troisième case.
    let deux_places = avant[2].x - case.x;
    glisser(&mut app, de, (de.0 + deux_places, de.1 + 10.0));
    let apres = app.store.cases(&board);
    assert_eq!(apres[2].id, case.id, "elle est troisième");
    assert_eq!(
        (apres[2].x, apres[2].y),
        (avant[2].x, avant[2].y),
        "à sa place exacte"
    );
    assert_eq!(apres[0].id, avant[1].id, "la deuxième ouvre la suite");
    assert_eq!((apres[0].x, apres[0].y), (avant[0].x, avant[0].y));
    let image = app.store.image(&board, "i").expect("l'image");
    assert!(apres[2].contient((image.x, image.y)), "elle suit sa case");
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    assert!(app.store.undo());
    assert_eq!(premiere(&app, &board), case);
    assert_eq!(app.store.image(&board, "i").map(|i| i.x), Some(centre.0));
    assert!(!app.store.undo(), "un seul geste");
}

/// **Une case glissée de peu revient à sa place** — et ne laisse rien à annuler.
#[test]
fn test_storyboard_2_une_case_glissee_de_peu_revient() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let de = dans_la_bande(&case);
    glisser(&mut app, de, (de.0 + 40.0, de.1 + 30.0));
    assert_eq!(premiere(&app, &board), case);
    assert!(!app.store.can_undo(), "rien n'a changé");
}

/// **Pendant qu'on tient une case, le fantôme montre la place où elle tombera.**
#[test]
fn test_storyboard_2_le_fantome_montre_la_place_visee() {
    let (mut app, board) = app();
    let cases: Vec<StoryboardPanel> = app.store.cases(&board).into_iter().cloned().collect();
    let de = dans_la_bande(&cases[0]);
    app.mouse_pos = de;
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    let vers = (
        de.0 + cases[5].x - cases[0].x,
        de.1 + cases[5].y - cases[0].y,
    );
    app.handle_cursor_moved(PhysicalPosition::new(vers.0, vers.1));
    let fantome = app.fantome_du_storyboard();
    assert_eq!(fantome.len(), 1, "une place");
    assert_eq!((fantome[0].left, fantome[0].top), (cases[5].x, cases[5].y));
    assert_eq!(
        fantome[0].height,
        cases[5].height + DESCRIPTION,
        "cadre et bande"
    );
    app.handle_mouse_up(MouseButton::Left);
    assert!(app.fantome_du_storyboard().is_empty());
}

/// **La grille armée suit le curseur** : un mouvement redemande une image — sans quoi son
/// fantôme restait où il était (son retour du 13/10 : « ne fonctionne pas au survol »).
#[test]
fn test_storyboard_2_la_grille_armee_se_redessine_au_survol() {
    let (mut app, board) = app();
    app.store.effacer_le_storyboard(&board);
    let bouger = |app: &mut GlucoseApp, x: f64| {
        let _ = app.prendre_la_salissure();
        app.handle_cursor_moved(PhysicalPosition::new(x, 500.0));
        !app.prendre_la_salissure().est_propre()
    };
    assert!(
        !bouger(&mut app, 700.0),
        "sans grille armée, rien à redessiner"
    );
    app.ui.placement.storyboard = Some(grille());
    assert!(bouger(&mut app, 720.0), "le fantôme suit");
}

/// **Le cadre reste au canevas** : glisser dans une case vide ne la prend pas et ne la choisit
/// pas — c'est le rectangle de sélection, comme partout.
#[test]
fn test_storyboard_1_le_cadre_reste_au_canevas() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let dedans = (case.x + 40.0, case.y + 40.0);
    glisser(&mut app, dedans, (dedans.0 + 50.0, dedans.1 + 30.0));
    assert_eq!(premiere(&app, &board), case);
    assert_eq!(app.ui.storyboard.choisie, None);
}

/// **Une image lâchée dans une case s'y ajuste** — contenue, centrée — dans le geste même du
/// glisser : un `Ctrl+Z` la rend là d'où elle venait, à sa taille.
#[test]
fn test_storyboard_1_une_image_lachee_dans_une_case_s_y_ajuste() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    app.store
        .add_image(&board, BoardImage::new("i", 1200.0, 820.0, 300.0, 300.0));
    app.store.clear_selection();
    app.store.journal.clear();
    let cible = (case.x + 30.0, case.y + 30.0);
    glisser(&mut app, (1200.0, 820.0), cible);
    let image = app.store.image(&board, "i").expect("l'image").clone();
    assert_eq!(
        (image.x, image.y),
        (case.x + case.width / 2.0, case.y + case.height / 2.0)
    );
    assert_eq!(image.height, case.height, "contenue par la hauteur");
    assert!(app.store.undo());
    let rendue = app.store.image(&board, "i").expect("l'image");
    assert_eq!((rendue.x, rendue.y, rendue.width), (1200.0, 820.0, 300.0));
}

/// Une frappe au clavier, comme la boucle la remet : la touche, et le texte que la
/// composition a produit.
fn frapper(app: &mut GlucoseApp, touche: Key, texte: Option<&str>) {
    assert!(
        app.prendre_la_touche(&touche, true, texte),
        "la question prend la touche"
    );
    app.prendre_la_touche(&touche, false, None);
}

fn lettre(c: &str) -> Key {
    Key::Character(c.into())
}

/// **Deux clics sur la bande décrivent la case, au clavier** : le champ naît tout choisi, la
/// première lettre le remplace, l'accent d'une touche morte s'écrit, Retour arrière efface,
/// les flèches déplacent le curseur, Entrée valide — en un geste (CHAMP-QUESTION-1 : au PC, le
/// champ n'écrivait rien, et l'épreuve d'avant écrivait dans le champ à la main).
#[test]
fn test_storyboard_1_deux_clics_decrivent_la_case_au_clavier() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    app.store.update_panel(&board, &case.id, "ancien");
    app.store.journal.clear();
    let point = dans_la_bande(&case);
    for _ in 0..2 {
        glisser(&mut app, point, point);
    }
    let Some((_, Suite::DecrireLaCase { case: id, .. })) = &app.ui.question else {
        panic!("la description se demande");
    };
    assert_eq!(id, &case.id);
    for c in ["P", "l", "a", "n", " ", "v", "u", "e"] {
        frapper(&mut app, lettre(c), Some(c));
    }
    frapper(&mut app, Key::Named(NamedKey::Backspace), None);
    // « ê » : la touche morte ne compose rien, la lettre qui suit porte l'accent.
    frapper(&mut app, Key::Dead(Some('^')), None);
    frapper(&mut app, lettre("e"), Some("ê"));
    for _ in 0.."vuê".chars().count() {
        frapper(&mut app, Key::Named(NamedKey::ArrowLeft), None);
    }
    frapper(&mut app, lettre("d"), Some("d"));
    frapper(&mut app, lettre("e"), Some("e"));
    frapper(&mut app, Key::Named(NamedKey::Space), Some(" "));
    frapper(&mut app, Key::Named(NamedKey::Enter), None);
    assert!(app.ui.question.is_none(), "Entrée valide");
    assert_eq!(premiere(&app, &board).description, "Plan de vuê");
    assert!(app.store.undo());
    assert_eq!(premiere(&app, &board).description, "ancien");
}

/// **Échap renonce à la description** : rien ne s'écrit.
#[test]
fn test_storyboard_1_echap_renonce_a_la_description() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let point = dans_la_bande(&case);
    for _ in 0..2 {
        glisser(&mut app, point, point);
    }
    frapper(&mut app, lettre("x"), Some("x"));
    frapper(&mut app, Key::Named(NamedKey::Escape), None);
    assert!(app.ui.question.is_none());
    assert_eq!(premiere(&app, &board).description, "");
}

/// **La croix et `Suppr` retirent la case choisie**, et les suivantes se renumérotent.
#[test]
fn test_storyboard_1_la_croix_et_suppr_retirent_la_case() {
    let (mut app, board) = app();
    let case = premiere(&app, &board);
    let point = dans_la_bande(&case);
    glisser(&mut app, point, point);
    assert_eq!(app.ui.storyboard.choisie.as_deref(), Some(case.id.as_str()));
    let vp = app.store.viewport();
    let boite = crate::renderer::storyboard::boite_a_l_ecran(&case, &vp);
    let bande = DESCRIPTION as f32 * vp.scale as f32;
    let (cx, cy, cw, ch) = crate::renderer::storyboard::croix(boite, bande, app.ui.scale());
    let croix = (f64::from(cx + cw / 2.0), f64::from(cy + ch / 2.0));
    glisser(&mut app, croix, croix);
    let cases = app.store.cases(&board);
    assert_eq!(cases.len(), 7, "la croix l'a retirée");
    assert_eq!(cases[0].order, 0, "la suivante prend son numéro");

    let seconde = cases[0].clone();
    let point = dans_la_bande(&seconde);
    glisser(&mut app, point, point);
    app.handle_shortcut_input(&Key::Named(NamedKey::Delete), ElementState::Pressed);
    assert_eq!(app.store.cases(&board).len(), 6, "Suppr aussi");
}

/// **`Échap` renonce à la grille armée** ; armée, elle montre une boîte par case.
#[test]
fn test_storyboard_1_echap_renonce_a_la_grille_armee() {
    let (mut app, board) = app();
    app.store.effacer_le_storyboard(&board);
    app.ui.placement.storyboard = Some(grille());
    assert_eq!(app.fantome_du_storyboard().len(), 8);
    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    assert!(app.ui.placement.storyboard.is_none());
    assert!(app.fantome_du_storyboard().is_empty());
}

/// Le coin bas droit de la boîte du storyboard, à l'écran (l'échelle est 1 depuis l'origine).
fn coin_du_storyboard(app: &GlucoseApp, board: &str) -> (f64, f64) {
    let d = app
        .store
        .disposition_actuelle(board, grille())
        .expect("un storyboard");
    let (x, y, w, h) = d.boite();
    (x + w, y + h)
}

/// Choisit la première case, par un clic sur sa bande.
fn choisir_la_premiere(app: &mut GlucoseApp, board: &str) {
    let point = dans_la_bande(&premiere(app, board));
    glisser(app, point, point);
    app.store.journal.clear();
}

/// **Saisir la poignée sans bouger ne change rien**, et ne laisse rien à annuler.
#[test]
fn test_storyboard_2_saisir_la_poignee_ne_change_rien() {
    let (mut app, board) = app();
    choisir_la_premiere(&mut app, &board);
    let avant: Vec<StoryboardPanel> = app.store.cases(&board).into_iter().cloned().collect();
    let coin = coin_du_storyboard(&app, &board);
    glisser(&mut app, coin, coin);
    let apres: Vec<StoryboardPanel> = app.store.cases(&board).into_iter().cloned().collect();
    assert_eq!(apres, avant);
    assert!(!app.store.can_undo());
}

/// **Tirer la poignée retrace le storyboard** : une colonne de moins, et les cases vides de
/// trop partent ; la grille garde trois colonnes ; un `Ctrl+Z` rend tout.
#[test]
fn test_storyboard_2_tirer_la_poignee_retrace_le_storyboard() {
    let (mut app, board) = app();
    choisir_la_premiere(&mut app, &board);
    let coin = coin_du_storyboard(&app, &board);
    let pas = grille().pas().0;
    app.mouse_pos = coin;
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(coin.0 - pas, coin.1));
    assert_eq!(
        app.fantome_du_storyboard().len(),
        6,
        "le fantôme montre six cases"
    );
    app.handle_mouse_up(MouseButton::Left);
    assert_eq!(app.store.cases(&board).len(), 6);
    assert_eq!(
        app.store.grille_du_storyboard(&board).map(|g| g.colonnes),
        Some(3)
    );
    assert!(app.store.undo());
    assert_eq!(app.store.cases(&board).len(), 8);
}

/// **Tirée vers l'intérieur, la poignée ne retire jamais une case qui porte quelque chose** :
/// la grille s'allonge plutôt.
#[test]
fn test_storyboard_2_la_poignee_garde_les_cases_decrites() {
    let (mut app, board) = app();
    let huitieme = app.store.cases(&board)[7].id.clone();
    app.store.update_panel(&board, &huitieme, "la fin");
    choisir_la_premiere(&mut app, &board);
    let coin = coin_du_storyboard(&app, &board);
    let pas = grille().pas().0;
    glisser(&mut app, coin, (coin.0 - pas, coin.1));
    let cases = app.store.cases(&board);
    assert_eq!(cases.len(), 8, "la huitième est décrite");
    assert_eq!(cases[7].description, "la fin");
    assert_eq!(
        cases[6].x, cases[0].x,
        "trois colonnes : la septième ouvre la troisième rangée"
    );
}

/// **Sans case choisie, pas de poignée** : le coin est au canevas.
#[test]
fn test_storyboard_2_sans_case_choisie_pas_de_poignee() {
    let (mut app, board) = app();
    let coin = coin_du_storyboard(&app, &board);
    app.mouse_pos = coin;
    assert!(!app.prendre_la_poignee_du_storyboard());
    assert!(app.ui.placement.trace.is_none());
}

/// **Saisie à côté de son coin, la poignée garde l'écart de la main** : le rectangle tracé part
/// du coin, pas de la main — sans quoi saisir la poignée déplacerait déjà le bord.
#[test]
fn test_storyboard_2_la_poignee_garde_l_ecart_de_la_main() {
    let (mut app, board) = app();
    choisir_la_premiere(&mut app, &board);
    let coin = coin_du_storyboard(&app, &board);
    let origine = (premiere(&app, &board).x, premiere(&app, &board).y);
    app.mouse_pos = (coin.0 - 4.0, coin.1 - 3.0);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(coin.0 + 6.0, coin.1 + 7.0));
    assert_eq!(
        app.rectangle_trace_a_l_ecran(),
        Some((origine.0, origine.1, coin.0 + 10.0, coin.1 + 10.0)),
        "la main a bougé de (10, 10) : le coin aussi, pas plus"
    );
    app.handle_mouse_up(MouseButton::Left);
}

/// **Tab enregistre la description et passe à la case suivante** ; Maj+Tab revient à la
/// précédente, son texte tout choisi ; après la dernière, la question se referme (fiche 65 § 9).
#[test]
fn test_storyboard_2_tab_passe_a_la_case_suivante() {
    let (mut app, board) = app();
    let point = dans_la_bande(&premiere(&app, &board));
    for _ in 0..2 {
        glisser(&mut app, point, point);
    }
    frapper(&mut app, lettre("a"), Some("a"));
    frapper(&mut app, Key::Named(NamedKey::Tab), None);
    let cases: Vec<StoryboardPanel> = app.store.cases(&board).into_iter().cloned().collect();
    assert_eq!(cases[0].description, "a", "la première est écrite");
    let Some((_, Suite::DecrireLaCase { case, .. })) = &app.ui.question else {
        panic!("la suivante se demande");
    };
    assert_eq!(case, &cases[1].id);
    frapper(&mut app, lettre("b"), Some("b"));
    app.modifiers = winit::keyboard::ModifiersState::SHIFT;
    frapper(&mut app, Key::Named(NamedKey::Tab), None);
    app.modifiers = winit::keyboard::ModifiersState::empty();
    let Some((question, Suite::DecrireLaCase { case, .. })) = &app.ui.question else {
        panic!("la précédente se demande");
    };
    assert_eq!(case, &cases[0].id);
    assert_eq!(question.champ.as_ref().map(|c| c.texte.as_str()), Some("a"));
    assert_eq!(app.store.cases(&board)[1].description, "b");
    let derniere = app
        .store
        .cases(&board)
        .last()
        .map(|c| c.id.clone())
        .expect("la dernière");
    frapper(&mut app, Key::Named(NamedKey::Escape), None);
    app.decrire_la_case(&derniere);
    frapper(&mut app, Key::Named(NamedKey::Tab), None);
    assert!(
        app.ui.question.is_none(),
        "après la dernière, la question se referme"
    );
}
