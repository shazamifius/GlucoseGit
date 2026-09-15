//! KEY-1 — ce que chaque frappe demande, et ce que la saisie en fait.
//!
//! La lecture ([`Command::of`]) se teste sans application : c'est une fonction pure. L'action
//! se teste sur une application complète, parce qu'`↑`, `↓`, `Début` et `Fin` dépendent du
//! reflux, donc de la mise en page réelle.

use super::*;
use glucose_core::types::{Annotation, Viewport};
use winit::keyboard::SmolStr;

fn touche(c: &str) -> Key {
    Key::Character(SmolStr::new(c))
}

fn nommee(k: NamedKey) -> Key {
    Key::Named(k)
}

const AUCUN: ModifiersState = ModifiersState::empty();

/// La commande d'une frappe, sans texte composé.
fn lit(key: Key, mods: ModifiersState) -> Option<(Command, bool)> {
    Command::of(&key, &mods, None)
}

// ── La lecture d'une frappe ─────────────────────────────────────────────────

/// `Maj` étend, `Ctrl` change de maille, et les deux se combinent — une seule règle pour tous
/// les mouvements.
#[test]
fn test_key_1_shift_extends_and_ctrl_changes_the_grain() {
    let gauche = nommee(NamedKey::ArrowLeft);
    assert_eq!(
        lit(gauche.clone(), AUCUN),
        Some((Command::Move(Motion::Char, Direction::Backward), false))
    );
    assert_eq!(
        lit(gauche.clone(), ModifiersState::SHIFT),
        Some((Command::Move(Motion::Char, Direction::Backward), true))
    );
    assert_eq!(
        lit(gauche.clone(), ModifiersState::CONTROL),
        Some((Command::Move(Motion::Word, Direction::Backward), false))
    );
    assert_eq!(
        lit(gauche, ModifiersState::CONTROL | ModifiersState::SHIFT),
        Some((Command::Move(Motion::Word, Direction::Backward), true)),
        "Ctrl+Maj+← étend d'un mot : les deux modificateurs se cumulent"
    );
}

/// `Début` et `Fin` visent la ligne **visuelle** ; avec `Ctrl`, le document entier.
#[test]
fn test_key_1_home_and_end_follow_the_visual_line_unless_ctrl() {
    assert_eq!(
        lit(nommee(NamedKey::Home), AUCUN),
        Some((Command::MoveVisualEdge(Direction::Backward), false))
    );
    assert_eq!(
        lit(nommee(NamedKey::End), ModifiersState::CONTROL),
        Some((Command::Move(Motion::DocEdge, Direction::Forward), false))
    );
    assert_eq!(
        lit(nommee(NamedKey::Home), ModifiersState::SHIFT),
        Some((Command::MoveVisualEdge(Direction::Backward), true))
    );
}

/// Les quatre raccourcis d'édition, et rien d'autre sous `Ctrl`.
#[test]
fn test_key_1_the_editing_shortcuts() {
    let ctrl = ModifiersState::CONTROL;
    assert_eq!(lit(touche("a"), ctrl), Some((Command::SelectAll, false)));
    assert_eq!(lit(touche("C"), ctrl), Some((Command::Copy, false)));
    assert_eq!(lit(touche("x"), ctrl), Some((Command::Cut, false)));
    assert_eq!(lit(touche("v"), ctrl), Some((Command::Paste, false)));
    assert_eq!(lit(touche("q"), ctrl), None, "Ctrl+Q ne veut rien dire ici");
}

/// **Ce que la couche de composition produit est inséré tel quel** : c'est ainsi qu'un accent
/// composé, un idéogramme ou une touche morte arrivent dans le texte (IME). Sans cela, taper
/// `é` sur un clavier qui le compose en deux temps était impossible.
#[test]
fn test_key_1_composed_text_is_inserted_as_it_comes() {
    let compose = Command::of(&nommee(NamedKey::Process), &AUCUN, Some("é"));
    assert_eq!(compose, Some((Command::Insert("é".into()), false)));
    // Un caractère de contrôle n'est pas du texte : il ne doit pas entrer dans le document.
    assert_eq!(Command::of(&touche("a"), &AUCUN, Some("\u{7f}")), None);
    // Sous Ctrl, ce n'est pas de la saisie mais un raccourci.
    assert_eq!(
        Command::of(&touche("z"), &ModifiersState::CONTROL, Some("z")),
        None
    );
}

/// `Entrée` va à la ligne — avec ou sans `Maj` ; `Ctrl+Entrée` et `Échap` valident.
///
/// Le `Maj` compte dans ce test, parce que c'est lui qui masquait la faute : une
/// majuscule de début de phrase le tient enfoncé, et le saut de ligne marchait alors
/// une fois sur deux — assez souvent pour qu'on croie à un aléa, jamais assez pour
/// qu'un test le voie.
#[test]
fn test_key_1_enter_breaks_the_line_whatever_shift_does() {
    assert_eq!(
        lit(nommee(NamedKey::Enter), AUCUN),
        Some((Command::Insert("\n".into()), false))
    );
    assert_eq!(
        lit(nommee(NamedKey::Enter), ModifiersState::SHIFT),
        Some((Command::Insert("\n".into()), true))
    );
    assert_eq!(
        lit(nommee(NamedKey::Enter), ModifiersState::CONTROL),
        Some((Command::Commit, false))
    );
    assert_eq!(
        lit(nommee(NamedKey::Escape), AUCUN),
        Some((Command::Commit, false))
    );
}

/// `Ctrl+S` et `Ctrl+O` traversent une saisie ; les lettres seules, non.
#[test]
fn test_file_commands_escape_a_text_session() {
    let ctrl = ModifiersState::CONTROL;
    assert!(is_file_command(&ctrl, &touche("s")));
    assert!(is_file_command(&ctrl, &touche("O")));
    assert!(!is_file_command(&AUCUN, &touche("s")));
    assert!(!is_file_command(&ctrl, &touche("a")));
}

// ── Ce que la saisie en fait ────────────────────────────────────────────────

const TEXTE: &str = "le chat dort\nsur le tapis";

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
        *width = Some(400.0);
        *height = Some(200.0);
    }
    app.store.add_annotation(&board, carte);
    app.start_text_edit("c".into(), TEXTE.into());
    app
}

fn pose(app: &mut GlucoseApp, selection: Selection) {
    app.editing_session.as_mut().expect("session").selection = selection;
}

fn selection(app: &GlucoseApp) -> Selection {
    app.editing_session.as_ref().expect("session").selection
}

fn tampon(app: &GlucoseApp) -> String {
    app.editing_session
        .as_ref()
        .expect("session")
        .buffer
        .clone()
}

fn selectionne(app: &GlucoseApp) -> String {
    let session = app.editing_session.as_ref().expect("session");
    session.selection.slice(&session.buffer).to_string()
}

/// **`Maj`+flèche étend, la flèche seule déplace.** La même touche, la même ligne de code.
#[test]
fn test_shift_arrow_extends_while_a_plain_arrow_moves() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::at(3));

    app.apply_text_command(Command::Move(Motion::Char, Direction::Forward), true);
    app.apply_text_command(Command::Move(Motion::Char, Direction::Forward), true);
    assert_eq!(selectionne(&app), "ch");
    assert_eq!(selection(&app).anchor, 3, "l'ancre est restée posée");

    app.apply_text_command(Command::Move(Motion::Char, Direction::Forward), false);
    assert!(
        selection(&app).is_empty(),
        "sans Maj, la flèche referme la sélection"
    );
}

/// Sans `Maj`, une flèche sur une sélection **retombe du bon côté** au lieu d'avancer d'un
/// caractère de plus.
#[test]
fn test_a_plain_arrow_collapses_to_the_right_side() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::spanning(3, 7));
    app.apply_text_command(Command::Move(Motion::Char, Direction::Backward), false);
    assert_eq!(
        selection(&app),
        Selection::at(3),
        "à gauche de la sélection"
    );

    pose(&mut app, Selection::spanning(3, 7));
    app.apply_text_command(Command::Move(Motion::Char, Direction::Forward), false);
    assert_eq!(
        selection(&app),
        Selection::at(7),
        "à droite de la sélection"
    );
}

/// `Ctrl+Maj+→` étend d'un mot entier.
#[test]
fn test_ctrl_shift_arrow_extends_by_a_whole_word() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::at(3));
    app.apply_text_command(Command::Move(Motion::Word, Direction::Forward), true);
    assert_eq!(selectionne(&app), "chat ");
}

/// `Ctrl+A` prend tout, puis taper remplace tout — le geste le plus courant d'une correction.
#[test]
fn test_select_all_then_typing_replaces_everything() {
    let mut app = app_en_edition();
    app.apply_text_command(Command::SelectAll, false);
    assert_eq!(selectionne(&app), TEXTE);
    app.apply_text_command(Command::Insert("neuf".into()), false);
    assert_eq!(tampon(&app), "neuf");
    assert_eq!(selection(&app), Selection::at(4));
}

/// `Retour arrière` mange la sélection quand il y en a une, un caractère sinon.
#[test]
fn test_backspace_eats_the_selection_first() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::spanning(3, 7));
    app.apply_text_command(Command::Delete(Motion::Char, Direction::Backward), false);
    assert_eq!(tampon(&app), "le  dort\nsur le tapis");

    let mut app = app_en_edition();
    pose(&mut app, Selection::at(2));
    app.apply_text_command(Command::Delete(Motion::Char, Direction::Backward), false);
    assert_eq!(tampon(&app), "l chat dort\nsur le tapis");
}

/// **`Début` et `Fin` visent la ligne visuelle**, pas le paragraphe : sur un texte reflué, ce
/// n'est pas la même chose, et c'est la ligne qu'on voit que la main désigne.
#[test]
fn test_home_and_end_reach_the_visual_line_edges() {
    let mut app = app_en_edition();
    // Deuxième paragraphe : « sur le tapis » commence à l'octet 13.
    pose(&mut app, Selection::at(17));
    app.apply_text_command(Command::MoveVisualEdge(Direction::Backward), false);
    assert_eq!(selection(&app), Selection::at(13));
    app.apply_text_command(Command::MoveVisualEdge(Direction::Forward), false);
    assert_eq!(selection(&app), Selection::at(TEXTE.len()));
}

/// `↑` et `↓` changent de ligne **en gardant la colonne** : c'est la mise en page qui le dit,
/// pas le nombre de caractères.
#[test]
fn test_up_and_down_keep_the_column() {
    let mut app = app_en_edition();
    // « dort » sur la première ligne, « tapis » à peu près sous lui sur la seconde.
    let depart = TEXTE.find("dort").expect("mot");
    pose(&mut app, Selection::at(depart));
    app.apply_text_command(Command::MoveLine(Direction::Forward), false);
    let apres = selection(&app).head;
    assert!(
        apres > 13 && apres <= TEXTE.len(),
        "on est descendu sur la seconde ligne, en {apres}"
    );

    app.apply_text_command(Command::MoveLine(Direction::Backward), false);
    assert_eq!(
        selection(&app).head,
        depart,
        "remonter ramène à la colonne de départ"
    );
}

/// En haut, `↑` va au début du texte ; en bas, `↓` va à sa fin. Jamais nulle part.
#[test]
fn test_up_at_the_top_and_down_at_the_bottom_reach_the_ends() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::at(5));
    app.apply_text_command(Command::MoveLine(Direction::Backward), false);
    assert_eq!(selection(&app), Selection::at(0));

    pose(&mut app, Selection::at(17));
    app.apply_text_command(Command::MoveLine(Direction::Forward), false);
    assert_eq!(selection(&app), Selection::at(TEXTE.len()));
}

/// **Taper un accent composé l'écrit en entier** : c'est le chemin de l'IME, et il ne passe
/// pas par les touches nommées.
#[test]
fn test_typing_an_accent_writes_the_whole_character() {
    let mut app = app_en_edition();
    pose(&mut app, Selection::at(0));
    app.apply_text_command(Command::Insert("é".into()), false);
    assert_eq!(tampon(&app), "éle chat dort\nsur le tapis");
    assert_eq!(
        selection(&app),
        Selection::at(2),
        "le curseur saute les deux octets du « é »"
    );
}

/// Un mouvement ne peut pas laisser la tête au milieu d'un caractère, quelle que soit la
/// commande — l'invariant SEL-2 vu depuis le clavier.
#[test]
fn test_no_command_ever_leaves_the_cursor_inside_a_character() {
    let accents = "l'été à Genève\nfoo_bar";
    let mut app = app_en_edition();
    app.apply_text_command(Command::SelectAll, false);
    app.apply_text_command(Command::Insert(accents.into()), false);

    let commandes = [
        Command::Move(Motion::Char, Direction::Backward),
        Command::Move(Motion::Char, Direction::Forward),
        Command::Move(Motion::Word, Direction::Backward),
        Command::Move(Motion::Word, Direction::Forward),
        Command::MoveLine(Direction::Backward),
        Command::MoveLine(Direction::Forward),
        Command::MoveVisualEdge(Direction::Backward),
        Command::MoveVisualEdge(Direction::Forward),
    ];
    for offset in 0..=accents.len() {
        for commande in &commandes {
            pose(&mut app, Selection::at(offset).clamped(accents));
            app.apply_text_command(commande.clone(), true);
            let sel = selection(&app);
            let buffer = tampon(&app);
            assert!(
                buffer.is_char_boundary(sel.head) && buffer.is_char_boundary(sel.anchor),
                "{commande:?} depuis {offset} laisse {sel:?} au milieu d'un caractère"
            );
        }
    }
}

/// **Sur un pense-bête, les touches font quand même quelque chose.** Son texte ne passe pas
/// encore par la mise en page refluée ; `↑`, `↓`, `Début` et `Fin` se replient alors sur la
/// ligne **logique**, colonne gardée — moins juste qu'un pas de ligne visuelle, mais infiniment
/// préférable à une touche inerte.
#[test]
fn test_a_sticky_note_still_answers_the_vertical_keys() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let texte = "premiere ligne\nseconde ligne";
    app.store
        .add_annotation(&board, Annotation::sticky("s", 0.0, 0.0, texte));
    app.start_text_edit("s".into(), texte.into());
    assert!(
        app.editing_layout().is_none(),
        "un pense-bête n'a pas encore de mise en page refluée : c'est le cas que ce test couvre"
    );

    pose(&mut app, Selection::at(3));
    app.apply_text_command(Command::MoveLine(Direction::Forward), false);
    assert_eq!(
        selection(&app),
        Selection::at(18),
        "descendre garde la colonne : trois caractères après le début de la seconde ligne"
    );
    app.apply_text_command(Command::MoveLine(Direction::Backward), false);
    assert_eq!(selection(&app), Selection::at(3), "et remonter y revient");

    app.apply_text_command(Command::MoveVisualEdge(Direction::Forward), false);
    assert_eq!(
        selection(&app),
        Selection::at(14),
        "la fin de la ligne logique"
    );
    app.apply_text_command(Command::MoveVisualEdge(Direction::Backward), false);
    assert_eq!(selection(&app), Selection::at(0));
}
