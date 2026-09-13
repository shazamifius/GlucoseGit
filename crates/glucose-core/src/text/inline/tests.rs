//! Ce que l'analyse inline garantit — et ce qu'elle refuse de faire.

use super::*;
use crate::text::plain_text;

/// Un corpus qui couvre le nominal, les pièges connus et les entrées dégénérées. Il sert de
/// base à l'invariant SPAN-1, qui doit tenir sur **toutes** ces entrées sans exception.
const CORPUS: &[&str] = &[
    "",
    "du texte",
    "**gras**",
    "*italique*",
    "_italique_",
    "~~barré~~",
    "`code`",
    "***gras italique***",
    "2 * 3 * 4",
    "snake_case_name",
    "un *mot* et **deux** et ~~trois~~ et `quatre`",
    "*a **b** c*",
    "**a *b**",
    "*non fermé",
    "fermé sans ouverture*",
    "`a **b** c`",
    "\\*pas une étoile\\*",
    "****",
    "*",
    "**",
    "``",
    "a`b",
    "~simple~",
    "~~~trois~~~",
    "l'été à Genève — **déjà** ?",
    "$E = mc^2$ et **après**",
    "**", // volontairement deux fois : l'analyse ne garde pas d'état entre appels
];

/// La concaténation des tranches, et l'enchaînement de leurs bornes.
fn partition_holds(source: &str) -> Result<(), String> {
    let spans = inline_spans(source);
    let mut at = 0usize;
    let mut rebuilt = String::new();
    for s in &spans {
        if s.start != at {
            return Err(format!("trou ou recouvrement à {at} (tranche {s:?})"));
        }
        if s.end <= s.start {
            return Err(format!("tranche vide émise : {s:?}"));
        }
        rebuilt.push_str(s.slice(source));
        at = s.end;
    }
    if at != source.len() {
        return Err(format!("la partition s'arrête à {at} / {}", source.len()));
    }
    if rebuilt != source {
        return Err(format!("reconstruit {rebuilt:?} au lieu de {source:?}"));
    }
    Ok(())
}

/// **SPAN-1.** Les tranches couvrent la source exactement : bout à bout, sans trou, sans
/// chevauchement, et leur concaténation rend le texte d'origine. C'est ce qui permet à un
/// curseur, à une sélection et à une ancre de flèche de se placer par un simple `start <= i`.
#[test]
fn test_spans_partition_the_source_exactly() {
    for source in CORPUS {
        if let Err(e) = partition_holds(source) {
            panic!("SPAN-1 rompu sur {source:?} : {e}");
        }
    }
}

/// Le texte lu, tranche par tranche, avec son emphase — les signes exclus.
fn lu(source: &str) -> Vec<(&str, Emphasis)> {
    inline_spans(source)
        .into_iter()
        .filter(|s| s.role == SpanRole::Text)
        .map(|s| (s.slice(source), s.emphasis))
        .collect()
}

#[test]
fn test_the_four_inline_emphases_are_recognised() {
    assert_eq!(lu("**gras**"), [("gras", Emphasis::BOLD)]);
    assert_eq!(lu("*italique*"), [("italique", Emphasis::ITALIC)]);
    assert_eq!(lu("_italique_"), [("italique", Emphasis::ITALIC)]);
    assert_eq!(lu("~~barré~~"), [("barré", Emphasis::STRIKE)]);
    assert_eq!(lu("`code`"), [("code", Emphasis::CODE)]);
    assert_eq!(
        lu("***tout***"),
        [("tout", Emphasis::BOLD | Emphasis::ITALIC)]
    );
}

/// Les emphases se cumulent en s'imbriquant, et un signe intérieur reste du signe.
#[test]
fn test_emphases_nest() {
    assert_eq!(
        lu("*a **b** c*"),
        [
            ("a ", Emphasis::ITALIC),
            ("b", Emphasis::ITALIC | Emphasis::BOLD),
            (" c", Emphasis::ITALIC),
        ]
    );
}

/// **Une multiplication n'est pas une italique.** `strip_inline_markdown` effaçait toutes
/// les étoiles de la chaîne ; ici, une étoile entourée d'espaces n'ouvre ni ne ferme rien.
#[test]
fn test_arithmetic_is_not_emphasis() {
    assert_eq!(lu("2 * 3 * 4"), [("2 * 3 * 4", Emphasis::NONE)]);
    assert_eq!(plain_text("2 * 3 * 4"), "2 * 3 * 4");
}

/// **`snake_case` n'est pas de l'italique.** Un tiret bas collé à des lettres des deux côtés
/// n'est pas un délimiteur — c'est la seule différence de traitement entre `_` et `*`.
#[test]
fn test_underscores_inside_a_word_are_not_emphasis() {
    assert_eq!(lu("snake_case_name"), [("snake_case_name", Emphasis::NONE)]);
    assert_eq!(lu("_mot_"), [("mot", Emphasis::ITALIC)]);
    // Une étoile, elle, coupe à l'intérieur d'un mot : c'est ce que fait CommonMark.
    assert_eq!(
        lu("a*b*c"),
        [
            ("a", Emphasis::NONE),
            ("b", Emphasis::ITALIC),
            ("c", Emphasis::NONE)
        ]
    );
}

/// Un délimiteur qui ne trouve pas son pendant est du texte, pas un style silencieux.
#[test]
fn test_an_unclosed_delimiter_stays_a_character() {
    assert_eq!(lu("*non fermé"), [("*non fermé", Emphasis::NONE)]);
    assert_eq!(
        lu("fermé sans ouverture*"),
        [("fermé sans ouverture*", Emphasis::NONE)]
    );
    // Dans `**a *b**`, l'étoile solitaire est abandonnée quand `**` se referme.
    assert_eq!(lu("**a *b**"), [("a *b", Emphasis::BOLD)]);
}

/// **Le code est opaque.** Entre deux accents graves, aucun autre signe n'est lu : c'est le
/// sens même de la syntaxe, et c'est ce qui permet d'écrire `**` dans du code.
#[test]
fn test_code_hides_every_other_sign() {
    assert_eq!(lu("`a **b** c`"), [("a **b** c", Emphasis::CODE)]);
    assert_eq!(plain_text("`a **b** c`"), "a **b** c");
}

/// Une suite de `n` accents graves ne se ferme que par `n` : c'est ce qui permet de montrer
/// un accent grave dans du code.
#[test]
fn test_backtick_runs_match_by_length() {
    assert_eq!(lu("``a`b``"), [("a`b", Emphasis::CODE)]);
    assert_eq!(lu("a`b"), [("a`b", Emphasis::NONE)]);
}

/// `\*` montre une étoile. Le `\` est un signe : il disparaît au repos, l'étoile reste.
#[test]
fn test_a_backslash_disarms_the_next_sign() {
    assert_eq!(
        lu("\\*pas une étoile\\*"),
        [("*pas une étoile", Emphasis::NONE), ("*", Emphasis::NONE)]
    );
    assert_eq!(plain_text("\\*pas une étoile\\*"), "*pas une étoile*");
    // Hors de la liste, un `\` reste un `\`.
    assert_eq!(lu("C:\\dossier"), [("C:\\dossier", Emphasis::NONE)]);
}

/// **La limite assumée** : ouverture et fermeture doivent avoir la même longueur. CommonMark
/// accepte `***a** b*` ; ici le résultat serait imprévisible à la lecture, donc rien n'est
/// stylé. Ce test existe pour que le choix soit visible, pas pour le figer à jamais.
#[test]
fn test_mismatched_run_lengths_style_nothing() {
    assert_eq!(lu("***a** b*"), [("***a** b*", Emphasis::NONE)]);
}

/// Une suite trop longue pour être un délimiteur est du texte : `****` sépare, il ne style pas.
#[test]
fn test_runs_longer_than_three_are_text() {
    assert_eq!(lu("****"), [("****", Emphasis::NONE)]);
    assert_eq!(lu("~~~trois~~~"), [("~~~trois~~~", Emphasis::NONE)]);
}

/// Un marqueur porte l'emphase de ce qui l'entoure, jamais la sienne : pendant l'édition,
/// le `**` grisé d'un mot en italique est en italique, mais il n'est pas en gras.
#[test]
fn test_a_marker_wears_the_style_around_it_not_its_own() {
    let source = "*a **b** c*";
    let markers: Vec<_> = inline_spans(source)
        .into_iter()
        .filter(|s| s.role == SpanRole::Marker)
        .map(|s| (s.slice(source), s.emphasis))
        .collect();
    assert_eq!(
        markers,
        [
            ("*", Emphasis::NONE),
            ("**", Emphasis::ITALIC),
            ("**", Emphasis::ITALIC),
            ("*", Emphasis::NONE),
        ]
    );
}

/// Une source sans le moindre signe n'alloue qu'une tranche : le cas courant est le cas
/// rapide, puisque l'analyse tourne à chaque frame sur chaque carte visible.
#[test]
fn test_plain_text_yields_a_single_span() {
    assert_eq!(inline_spans("une phrase ordinaire").len(), 1);
    assert!(inline_spans("").is_empty());
}
