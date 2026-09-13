//! Ce que la sélection garantit : SEL-1 (ancre et tête), SEL-2 (frontières), et les
//! mouvements que la main attend.

use super::*;

/// Un texte qui mêle accents, ponctuation, tiret bas et sauts de ligne — tout ce sur quoi un
/// parcours d'octets naïf se casse.
const TEXTE: &str = "L'été à Genève.\nfoo_bar = 42 ;\n";

/// Les offsets de chaque frontière de caractère, pour éprouver une fonction partout.
fn frontieres(text: &str) -> Vec<usize> {
    (0..=text.len())
        .filter(|&i| text.is_char_boundary(i))
        .collect()
}

// ── SEL-1 : une ancre et une tête ───────────────────────────────────────────

/// **Étendre puis revenir ne retourne pas la sélection.** Avec un couple ordonné, `Maj+←`
/// depuis une sélection vers la droite inverserait les bornes et repartirait dans l'autre
/// sens ; avec une ancre, la tête revient simplement sur ses pas.
#[test]
fn test_sel_1_extending_back_and_forth_returns_to_the_same_place() {
    let texte = "abcdef";
    let mut sel = Selection::at(2);
    for _ in 0..3 {
        sel.head = next_char(texte, sel.head);
    }
    assert_eq!((sel.anchor, sel.head), (2, 5));
    assert_eq!(sel.slice(texte), "cde");

    for _ in 0..3 {
        sel.head = prev_char(texte, sel.head);
    }
    assert_eq!((sel.anchor, sel.head), (2, 2));
    assert!(sel.is_empty(), "on est revenu au curseur de départ");

    // Et au-delà : l'ancre tient, la sélection part de l'autre côté.
    sel.head = prev_char(texte, sel.head);
    assert_eq!(sel.range(), (1, 2));
    assert_eq!(sel.slice(texte), "b");
}

/// Un curseur **est** une sélection vide : un seul état à porter, pas deux.
#[test]
fn test_sel_1_a_cursor_is_an_empty_selection() {
    let sel = Selection::at(3);
    assert!(sel.is_empty());
    assert_eq!(sel.range(), (3, 3));
    assert_eq!(sel.slice("abcdef"), "");
}

/// Une flèche sans `Maj` ne bouge pas d'un caractère quand du texte est sélectionné : elle
/// retombe du bon côté. C'est ce que fait tout éditeur, et son absence se remarque tout de
/// suite — le curseur « saute » d'un caractère de trop.
#[test]
fn test_sel_1_collapsing_lands_on_the_right_side() {
    let sel = Selection::spanning(2, 5);
    assert_eq!(sel.collapsed(Direction::Forward), Selection::at(5));
    assert_eq!(sel.collapsed(Direction::Backward), Selection::at(2));
    // Même réponse quand la tête est avant l'ancre : c'est l'étendue qui décide, pas l'ordre.
    let inverse = Selection { anchor: 5, head: 2 };
    assert_eq!(inverse.collapsed(Direction::Forward), Selection::at(5));
    assert_eq!(inverse.collapsed(Direction::Backward), Selection::at(2));
}

// ── SEL-2 : jamais au milieu d'un caractère ─────────────────────────────────

/// **Aucun mouvement ne peut poser un offset au milieu d'un caractère**, depuis n'importe
/// quelle position de départ, y compris invalide. C'est l'invariant qui empêche les paniques
/// de `String::replace_range` et de `&text[a..b]`.
#[test]
fn test_sel_2_no_motion_ever_lands_inside_a_character() {
    let motions = [
        Motion::Char,
        Motion::Word,
        Motion::LineEdge,
        Motion::DocEdge,
    ];
    // Tous les offsets, frontières ou non : un offset invalide doit être réparé, pas propagé.
    for from in 0..=TEXTE.len() + 3 {
        for motion in motions {
            for dir in [Direction::Backward, Direction::Forward] {
                let to = move_offset(TEXTE, from, motion, dir);
                assert!(
                    TEXTE.is_char_boundary(to),
                    "{motion:?} {dir:?} depuis {from} tombe à {to}, au milieu d'un caractère"
                );
                assert!(to <= TEXTE.len(), "{motion:?} {dir:?} sort du texte");
            }
        }
    }
}

/// Un pas de caractère enjambe le caractère entier — « é » en fait deux octets, « € » trois.
#[test]
fn test_sel_2_a_step_crosses_a_whole_character() {
    let s = "aé€";
    assert_eq!(next_char(s, 0), 1);
    assert_eq!(next_char(s, 1), 3);
    assert_eq!(next_char(s, 3), 6);
    assert_eq!(next_char(s, 6), 6, "la fin est un point fixe");
    assert_eq!(prev_char(s, 6), 3);
    assert_eq!(prev_char(s, 3), 1);
    assert_eq!(prev_char(s, 1), 0);
    assert_eq!(prev_char(s, 0), 0, "le début est un point fixe");
    // Depuis un offset invalide, on redescend sur la frontière puis on avance.
    assert_eq!(next_char(s, 2), 3, "l'octet 2 est au milieu du « é »");
    assert_eq!(prev_char(s, 4), 1, "l'octet 4 est au milieu du « € »");
}

/// Une sélection qui survit à un texte raccourci est ramenée dedans, pas laissée au-delà.
#[test]
fn test_sel_2_a_stale_selection_is_brought_back_inside() {
    let court = "abc";
    let sel = Selection::spanning(10, 40).clamped(court);
    assert_eq!(sel.range(), (3, 3));
    assert_eq!(sel.slice(court), "");
    // Et un offset au milieu d'un caractère redescend sur sa frontière : « é » occupe les
    // octets 0 et 1, donc 1 n'est pas une position où un curseur puisse se tenir.
    assert_eq!(Selection::at(1).clamped("é").range(), (0, 0));
    assert_eq!(
        Selection::at(2).clamped("é").range(),
        (2, 2),
        "la fin est valide"
    );
}

// ── Les mouvements par mot ──────────────────────────────────────────────────

/// Le nom du mouvement, appliqué au texte, avec `|` là où le curseur se pose.
fn apres(text: &str, from: usize, motion: Motion, dir: Direction) -> String {
    let at = move_offset(text, from, motion, dir);
    format!("{}|{}", &text[..at], &text[at..])
}

/// `Ctrl+→` pose le curseur **devant le mot suivant**, prêt à écrire — la convention des
/// navigateurs et des traitements de texte.
#[test]
fn test_word_forward_lands_in_front_of_the_next_word() {
    let t = "le chat  dort.";
    assert_eq!(
        apres(t, 0, Motion::Word, Direction::Forward),
        "le |chat  dort."
    );
    assert_eq!(
        apres(t, 3, Motion::Word, Direction::Forward),
        "le chat  |dort."
    );
    // La ponctuation est un « mot » à elle seule : on s'arrête dessus plutôt que de la sauter.
    assert_eq!(
        apres(t, 9, Motion::Word, Direction::Forward),
        "le chat  dort|."
    );
    assert_eq!(
        apres(t, 13, Motion::Word, Direction::Forward),
        "le chat  dort.|"
    );
}

/// `Ctrl+←` remonte au début du mot courant, ou du précédent si l'on y est déjà.
#[test]
fn test_word_backward_reaches_the_start_of_the_word() {
    let t = "le chat  dort.";
    assert_eq!(
        apres(t, 14, Motion::Word, Direction::Backward),
        "le chat  dort|."
    );
    assert_eq!(
        apres(t, 13, Motion::Word, Direction::Backward),
        "le chat  |dort."
    );
    assert_eq!(
        apres(t, 9, Motion::Word, Direction::Backward),
        "le |chat  dort."
    );
    assert_eq!(
        apres(t, 0, Motion::Word, Direction::Backward),
        "|le chat  dort."
    );
}

/// **Un mouvement par mot ne traverse pas un paragraphe.** Sans cet arrêt, `Ctrl+→` sauterait
/// par-dessus une ligne vide et l'on ne pourrait jamais y poser le curseur pour y écrire.
#[test]
fn test_word_motion_stops_at_a_line_break() {
    let t = "fin\n\ndebut";
    let apres_fin = move_offset(t, 3, Motion::Word, Direction::Forward);
    assert_eq!(
        apres_fin, 3,
        "on est déjà au saut de ligne : on n'avance pas"
    );
    assert_eq!(move_offset(t, 4, Motion::Word, Direction::Forward), 4);
    assert_eq!(move_offset(t, 5, Motion::Word, Direction::Forward), 10);
}

/// Le tiret bas fait partie du mot, l'apostrophe non : les deux conventions que la main a
/// apprises ailleurs.
#[test]
fn test_word_classes_follow_what_the_hand_expects() {
    assert_eq!(class_of('a'), CharClass::Word);
    assert_eq!(class_of('é'), CharClass::Word);
    assert_eq!(class_of('4'), CharClass::Word);
    assert_eq!(class_of('_'), CharClass::Word);
    assert_eq!(class_of(' '), CharClass::Space);
    assert_eq!(class_of('\n'), CharClass::Space);
    assert_eq!(class_of('\''), CharClass::Punctuation);
    assert_eq!(class_of('.'), CharClass::Punctuation);
}

// ── Ce qu'un double-clic et un triple-clic prennent ─────────────────────────

fn mot(text: &str, at: usize) -> &str {
    let (s, e) = word_at(text, at);
    &text[s..e]
}

/// Un double-clic prend le mot sous le doigt — `snake_case` d'un bloc, mais `l'été` en deux.
#[test]
fn test_a_double_click_takes_the_word_under_the_finger() {
    let t = "foo_bar l'été.";
    assert_eq!(mot(t, 0), "foo_bar");
    assert_eq!(mot(t, 4), "foo_bar");
    assert_eq!(mot(t, 7), " ");
    assert_eq!(mot(t, 8), "l");
    assert_eq!(mot(t, 9), "'");
    assert_eq!(mot(t, 10), "été");
    assert_eq!(mot(t, 15), ".");
}

/// Au bord droit d'un mot, c'est le mot qu'on vient de quitter qui est pris : un double-clic
/// juste après la dernière lettre ne doit pas rendre une sélection vide.
#[test]
fn test_a_double_click_at_the_end_of_a_word_still_takes_it() {
    let t = "mot";
    assert_eq!(mot(t, 3), "mot");
    assert_eq!(word_at("", 0), (0, 0), "un texte vide n'a pas de mot");
}

/// Un triple-clic prend la ligne logique, saut de ligne exclu.
#[test]
fn test_a_triple_click_takes_the_paragraph() {
    let t = "premiere\nseconde\ntroisieme";
    assert_eq!(paragraph_at(t, 0), (0, 8));
    assert_eq!(paragraph_at(t, 12), (9, 16));
    assert_eq!(paragraph_at(t, 26), (17, 26));
}

/// Début et fin de ligne s'arrêtent au saut de ligne, sans le franchir.
#[test]
fn test_line_edges_stop_at_the_break() {
    let t = "abc\ndef";
    assert_eq!(move_offset(t, 2, Motion::LineEdge, Direction::Backward), 0);
    assert_eq!(move_offset(t, 2, Motion::LineEdge, Direction::Forward), 3);
    assert_eq!(move_offset(t, 5, Motion::LineEdge, Direction::Backward), 4);
    assert_eq!(move_offset(t, 5, Motion::LineEdge, Direction::Forward), 7);
    // Sur le saut de ligne lui-même : il appartient à la ligne d'avant.
    assert_eq!(move_offset(t, 3, Motion::LineEdge, Direction::Forward), 3);
}

// ── Écrire ──────────────────────────────────────────────────────────────────

/// Taper avec du texte sélectionné le remplace, et le curseur se pose après ce qu'on a écrit.
#[test]
fn test_typing_over_a_selection_replaces_it() {
    let mut t = String::from("le chat dort");
    let sel = replace(&mut t, Selection::spanning(3, 7), "chien");
    assert_eq!(t, "le chien dort");
    assert_eq!(sel, Selection::at(8));
    assert!(
        sel.is_empty(),
        "après une saisie, il ne reste qu'un curseur"
    );
}

/// Insérer sans sélection n'efface rien.
#[test]
fn test_inserting_without_a_selection_adds() {
    let mut t = String::from("abcd");
    let sel = replace(&mut t, Selection::at(2), "XY");
    assert_eq!(t, "abXYcd");
    assert_eq!(sel, Selection::at(4));
}

/// `Retour arrière` mange la sélection s'il y en a une, un caractère sinon — et un caractère
/// **entier**, accent compris.
#[test]
fn test_backspace_eats_the_selection_or_one_character() {
    // « l'été » fait sept octets : les deux « é » en prennent deux chacun.
    let mut t = String::from("l'été");
    let fin = t.len();
    assert_eq!(fin, 7);
    let sel = delete(
        &mut t,
        Selection::at(fin),
        Motion::Char,
        Direction::Backward,
    );
    assert_eq!(t, "l'ét");
    assert_eq!(sel, Selection::at(5), "un « é » entier, donc deux octets");

    let mut t = String::from("l'été");
    let sel = delete(
        &mut t,
        Selection::spanning(0, 2),
        Motion::Char,
        Direction::Backward,
    );
    assert_eq!(t, "été", "la sélection passe avant le mouvement");
    assert_eq!(sel, Selection::at(0));
}

/// `Ctrl+Retour arrière` efface le mot qui précède.
#[test]
fn test_ctrl_backspace_eats_the_previous_word() {
    let mut t = String::from("le chat dort");
    let sel = delete(&mut t, Selection::at(12), Motion::Word, Direction::Backward);
    assert_eq!(t, "le chat ");
    assert_eq!(sel, Selection::at(8));
}

/// `Suppr` efface vers l'avant, et ne fait rien au bout du texte.
#[test]
fn test_delete_forward_and_at_the_end() {
    let mut t = String::from("abc");
    let sel = delete(&mut t, Selection::at(1), Motion::Char, Direction::Forward);
    assert_eq!(t, "ac");
    assert_eq!(sel, Selection::at(1));

    let mut t = String::from("abc");
    let sel = delete(&mut t, Selection::at(3), Motion::Char, Direction::Forward);
    assert_eq!(t, "abc", "au bout, il n'y a rien à manger");
    assert_eq!(sel, Selection::at(3));
}

/// **Écrire ne peut pas paniquer**, quel que soit l'offset de départ — même invalide, même
/// au-delà du texte. C'est la garantie qui permet au reste du programme de ne pas vérifier.
#[test]
fn test_writing_never_panics_wherever_the_cursor_claims_to_be() {
    for from in 0..=TEXTE.len() + 5 {
        for to in 0..=TEXTE.len() + 5 {
            let mut t = String::from(TEXTE);
            let sel = replace(
                &mut t,
                Selection {
                    anchor: from,
                    head: to,
                },
                "é",
            );
            assert!(t.is_char_boundary(sel.head));
        }
    }
}

/// Tout sélectionner prend le texte entier, y compris vide.
#[test]
fn test_select_all_takes_everything() {
    assert_eq!(Selection::all(TEXTE).range(), (0, TEXTE.len()));
    assert_eq!(Selection::all(TEXTE).slice(TEXTE), TEXTE);
    assert!(Selection::all("").is_empty());
}

/// Les frontières du texte d'essai sont bien celles qu'on croit — un test qui garde le corpus
/// honnête plutôt que de le supposer.
#[test]
fn test_the_corpus_really_has_multibyte_characters() {
    assert!(
        frontieres(TEXTE).len() < TEXTE.len() + 1,
        "il faut des accents"
    );
    assert!(TEXTE.contains('è') && TEXTE.contains('\n'));
}

// ── La maille d'un geste de souris ──────────────────────────────────────────

/// Un clic simple pose un curseur ; un glisser prend exactement ce qu'il traverse.
#[test]
fn test_a_plain_drag_takes_what_it_crosses() {
    let t = "le chat dort";
    assert_eq!(expand(t, 3, 3, Granularity::Char), Selection::at(3));
    assert_eq!(
        expand(t, 3, 7, Granularity::Char).slice(t),
        "chat",
        "de l'ancre à la tête, ni plus ni moins"
    );
    assert_eq!(expand(t, 7, 3, Granularity::Char).slice(t), "chat");
}

/// **Un glisser entamé par un double-clic prend des mots entiers**, dans les deux sens. Sans
/// cette règle, revenir en arrière laisserait un mot coupé en deux derrière le curseur.
#[test]
fn test_a_word_drag_never_leaves_half_a_word() {
    let t = "le chat dort";
    // Double-clic sur « chat », puis glisser jusqu'au milieu de « dort ».
    let sel = expand(t, 4, 9, Granularity::Word);
    assert_eq!(sel.slice(t), "chat dort");
    // Et le retour, jusqu'au milieu de « le ».
    let sel = expand(t, 4, 1, Granularity::Word);
    assert_eq!(sel.slice(t), "le chat");
    assert!(
        sel.anchor > sel.head,
        "l'ancre reste du côté d'où l'on est parti (SEL-1)"
    );
}

/// Un triple-clic prend le paragraphe, et le glisser qui suit en prend d'autres entiers.
#[test]
fn test_a_paragraph_drag_takes_whole_lines() {
    let t = "premiere\nseconde\ntroisieme";
    assert_eq!(expand(t, 2, 2, Granularity::Paragraph).slice(t), "premiere");
    assert_eq!(
        expand(t, 2, 12, Granularity::Paragraph).slice(t),
        "premiere\nseconde"
    );
    assert_eq!(
        expand(t, 12, 2, Granularity::Paragraph).slice(t),
        "premiere\nseconde",
        "en remontant, le même texte est pris"
    );
}

/// Une maille ne peut pas sortir du texte ni tomber au milieu d'un caractère, quel que soit
/// le point de départ — c'est SEL-2, appliqué aux gestes de souris.
#[test]
fn test_expanding_stays_inside_the_text() {
    for granularity in [Granularity::Char, Granularity::Word, Granularity::Paragraph] {
        for anchor in 0..=TEXTE.len() + 3 {
            for head in 0..=TEXTE.len() + 3 {
                let sel = expand(TEXTE, anchor, head, granularity);
                assert!(TEXTE.is_char_boundary(sel.anchor), "{sel:?}");
                assert!(TEXTE.is_char_boundary(sel.head), "{sel:?}");
            }
        }
    }
}
