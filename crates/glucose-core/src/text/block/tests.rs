//! Ce que l'analyse de bloc garantit — et les pièges qu'elle ne doit pas prendre pour des
//! signes.

use super::*;

/// Le nominal, les paires qui partagent un premier octet, et les entrées dégénérées.
/// L'invariant BLOCK-1 doit tenir sur **toutes**, sans exception.
const CORPUS: &[&str] = &[
    "",
    "\n",
    "\n\n\n",
    "du texte",
    "# Titre",
    "## Sous-titre",
    "### un titre de niveau trois",
    "####### pas un titre",
    "#",
    "#pas un titre",
    "# ",
    "- puce",
    "* puce",
    "-# petit",
    "---",
    "-----",
    "--",
    "- - -",
    "---   ",
    "> citation",
    ">",
    ">pas une citation",
    "1. un",
    "12. douze",
    "0. zéro",
    "99999999999999999999. trop grand",
    "1.pas d'espace",
    ".1 pas un numéro",
    "$E = mc^2$",
    "$$\\sum_{i=0}^{n} i$$",
    "  $x$  ",
    "$5 et $6",
    "$$",
    "$",
    "```",
    "```rust",
    "```\nfn main() {}\n```",
    "```\n# pas un titre dedans\n- ni une puce\n```",
    "```\njamais fermé\n# ni après",
    "# Titre\n\ndu corps\n- une puce\n> une citation\n---\n1. un\n2. deux",
    "texte\n```\ncode\n```\ntexte",
];

/// Les blocs se suivent sans trou, séparés d'un `\n`, et couvrent la source entière.
fn partition_holds(source: &str) -> Result<(), String> {
    let mut at = 0usize;
    let mut count = 0usize;
    for b in blocks(source) {
        if b.start != at {
            return Err(format!("le bloc {b:?} ne reprend pas à {at}"));
        }
        if b.end > source.len() {
            return Err(format!("le bloc {b:?} dépasse la source"));
        }
        if source[b.range()].contains('\n') {
            return Err(format!("le bloc {b:?} enjambe un retour à la ligne"));
        }
        if b.prefix + b.suffix > b.end - b.start {
            return Err(format!("les signes de {b:?} débordent son paragraphe"));
        }
        // Les bornes du corps tombent sur des frontières de caractères : tout le reste en
        // dépend, du curseur à l'ancre de flèche.
        if !source.is_char_boundary(b.body().start) || !source.is_char_boundary(b.body().end) {
            return Err(format!("le corps de {b:?} coupe un caractère"));
        }
        at = b.end + 1;
        count += 1;
    }
    if at != source.len() + 1 {
        return Err(format!(
            "la partition s'arrête à {at} / {}",
            source.len() + 1
        ));
    }
    let paragraphs = source.split('\n').count();
    if count != paragraphs {
        return Err(format!("{count} blocs pour {paragraphs} paragraphes"));
    }
    Ok(())
}

#[test]
fn test_block_1_blocks_partition_the_source_exactly() {
    for source in CORPUS {
        if let Err(why) = partition_holds(source) {
            panic!("BLOCK-1 rompu sur {source:?} : {why}");
        }
    }
}

#[test]
fn test_block_1_a_body_plus_its_marks_rebuilds_the_paragraph() {
    for source in CORPUS {
        for b in blocks(source) {
            let head = &source[b.start..b.start + b.prefix];
            let tail = &source[b.end - b.suffix..b.end];
            let rebuilt = format!("{head}{}{tail}", b.body_slice(source));
            assert_eq!(rebuilt, b.slice(source), "sur {source:?}, bloc {b:?}");
        }
    }
}

/// Le genre de chaque paragraphe, et le texte que le lecteur en lit.
fn read(source: &str) -> Vec<(BlockKind, &str)> {
    blocks(source)
        .map(|b| (b.kind, b.body_slice(source)))
        .collect()
}

#[test]
fn test_headings_keep_their_text_and_lose_their_hashes() {
    assert_eq!(read("# Titre"), vec![(BlockKind::Heading(1), "Titre")]);
    assert_eq!(read("## Sous"), vec![(BlockKind::Heading(2), "Sous")]);
    assert_eq!(read("# "), vec![(BlockKind::Heading(1), "")]);
    // Les six niveaux du Markdown, jusqu'au dernier.
    assert_eq!(read("###### six"), vec![(BlockKind::Heading(6), "six")]);
    // Un de plus n'est plus un titre.
    assert_eq!(read("####### x"), vec![(BlockKind::Body, "####### x")]);
    // Sans espace, ce n'est pas un signe — un mot-dièse reste un mot-dièse.
    assert_eq!(read("#glucose"), vec![(BlockKind::Body, "#glucose")]);
    assert_eq!(read("#"), vec![(BlockKind::Body, "#")]);
}

#[test]
fn test_bullets_accept_both_marks() {
    assert_eq!(read("- un"), vec![(BlockKind::Bullet, "un")]);
    assert_eq!(read("* un"), vec![(BlockKind::Bullet, "un")]);
    // Une emphase en début de ligne n'est pas une puce : il lui manque l'espace.
    assert_eq!(read("*gras*"), vec![(BlockKind::Body, "*gras*")]);
}

#[test]
fn test_ordered_keeps_the_number_the_author_wrote() {
    assert_eq!(read("1. un"), vec![(BlockKind::Ordered(1), "un")]);
    assert_eq!(read("12. douze"), vec![(BlockKind::Ordered(12), "douze")]);
    assert_eq!(read("0. zéro"), vec![(BlockKind::Ordered(0), "zéro")]);
    // Glucose ne renumérote pas : trois « 1. » restent trois « 1. ».
    assert_eq!(
        read("1. a\n1. b"),
        vec![(BlockKind::Ordered(1), "a"), (BlockKind::Ordered(1), "b")]
    );
    assert_eq!(read("1.collé"), vec![(BlockKind::Body, "1.collé")]);
    assert_eq!(read(".1 x"), vec![(BlockKind::Body, ".1 x")]);
    // Un numéro qui déborde d'un u32 n'est plus un numéro, et rien ne panique.
    let huge = "99999999999999999999. x";
    assert_eq!(read(huge), vec![(BlockKind::Body, huge)]);
}

#[test]
fn test_quotes_keep_their_empty_lines() {
    assert_eq!(read("> dit-il"), vec![(BlockKind::Quote, "dit-il")]);
    assert_eq!(read(">"), vec![(BlockKind::Quote, "")]);
    assert_eq!(read(">collé"), vec![(BlockKind::Body, ">collé")]);
}

#[test]
fn test_small_text_is_not_a_bullet() {
    assert_eq!(read("-# note"), vec![(BlockKind::Small, "note")]);
    // L'ordre des essais compte : `-# ` partage son premier octet avec `- `.
    assert_ne!(read("-# note")[0].0, BlockKind::Bullet);
}

#[test]
fn test_a_rule_is_three_dashes_or_more_and_nothing_else() {
    assert_eq!(read("---"), vec![(BlockKind::Rule, "")]);
    assert_eq!(read("-----"), vec![(BlockKind::Rule, "")]);
    assert_eq!(read("--"), vec![(BlockKind::Body, "--")]);
    // Le Markdown accepte `- - -` comme trait ; Glucose non, et c'est délibéré : la règle
    // « que des tirets » s'énonce sans exception, et `- - -` est d'abord une puce dont le
    // texte est « - - ». Personne n'écrit un séparateur avec des espaces dedans.
    assert_eq!(read("- - -"), vec![(BlockKind::Bullet, "- -")]);
    // Des espaces traînants ne changent pas la nature de la ligne.
    assert_eq!(read("---   "), vec![(BlockKind::Rule, "")]);
    // Un tiret cadratin n'est pas un tiret bas d'ASCII.
    assert_eq!(read("———"), vec![(BlockKind::Body, "———")]);
}

#[test]
fn test_formulas_are_recognised_whole_and_lose_their_dollars() {
    assert_eq!(
        read("$E = mc^2$"),
        vec![(BlockKind::Math { display: false }, "E = mc^2")]
    );
    assert_eq!(
        read("$$\\sum i$$"),
        vec![(BlockKind::Math { display: true }, "\\sum i")]
    );
    // Le trim appartient aux signes, pas au corps.
    assert_eq!(
        read("  $x$  "),
        vec![(BlockKind::Math { display: false }, "x")]
    );
    // Deux prix ne sont pas une formule.
    assert_eq!(read("$5 et $6"), vec![(BlockKind::Body, "$5 et $6")]);
    assert_eq!(read("$$"), vec![(BlockKind::Body, "$$")]);
    assert_eq!(read("$"), vec![(BlockKind::Body, "$")]);
    // Une formule au milieu d'une phrase n'est pas un bloc de formule.
    assert_eq!(
        read("avant $x$ après"),
        vec![(BlockKind::Body, "avant $x$ après")]
    );
}

#[test]
fn test_inside_a_fence_nothing_is_markdown() {
    let source = "```rust\n# pas un titre\n- ni une puce\n$x$\n```";
    assert_eq!(
        read(source),
        vec![
            (BlockKind::Fence, ""),
            (BlockKind::Code, "# pas un titre"),
            (BlockKind::Code, "- ni une puce"),
            (BlockKind::Code, "$x$"),
            (BlockKind::Fence, ""),
        ]
    );
}

#[test]
fn test_a_fence_that_never_closes_swallows_the_rest() {
    let source = "texte\n```\ncode\n# toujours du code";
    assert_eq!(
        read(source),
        vec![
            (BlockKind::Body, "texte"),
            (BlockKind::Fence, ""),
            (BlockKind::Code, "code"),
            (BlockKind::Code, "# toujours du code"),
        ]
    );
}

#[test]
fn test_markdown_resumes_after_a_fence_closes() {
    let source = "```\ncode\n```\n# un vrai titre";
    let kinds: Vec<_> = blocks(source).map(|b| b.kind).collect();
    assert_eq!(kinds.last(), Some(&BlockKind::Heading(1)));
}

#[test]
fn test_an_empty_source_is_one_empty_body_block() {
    assert_eq!(read(""), vec![(BlockKind::Body, "")]);
    assert_eq!(
        read("\n"),
        vec![(BlockKind::Body, ""), (BlockKind::Body, "")]
    );
}

#[test]
fn test_literal_and_silent_name_what_the_renderer_must_not_do() {
    assert!(BlockKind::Code.literal(), "le code ne s'analyse pas");
    assert!(BlockKind::Math { display: true }.literal());
    assert!(
        !BlockKind::Quote.literal(),
        "une citation garde son Markdown"
    );
    assert!(BlockKind::Rule.silent(), "un trait n'a pas de texte");
    assert!(BlockKind::Fence.silent());
    assert!(!BlockKind::Body.silent());
}

#[test]
fn test_blocks_index_the_real_bytes_even_with_accents() {
    let source = "# Été\n- déjà";
    let found: Vec<_> = blocks(source).collect();
    assert_eq!(found[0].body(), 2..7, "« Été » fait cinq octets");
    assert_eq!(&source[found[0].body()], "Été");
    assert_eq!(&source[found[1].body()], "déjà");
}

// ── Les tableaux (fiche 08 § 2.1) ────────────────────────────────────────────

/// Les cellules d'une ligne, telles qu'on les lit.
fn cellules(line: &str) -> Vec<&str> {
    cells(line).into_iter().map(|r| &line[r]).collect()
}

#[test]
fn test_une_ligne_de_tableau_se_reconnait_a_ses_deux_barres() {
    assert_eq!(read("| a | b |")[0].0, BlockKind::TableRow);
    assert_eq!(read("| a | b")[0].0, BlockKind::TableRow);
    // Une seule barre n'est pas un tableau : personne n'écrit ça en y pensant.
    assert_eq!(read("| seul")[0].0, BlockKind::Body);
    assert_eq!(read("pas | au début")[0].0, BlockKind::Body);
    assert_eq!(read("|")[0].0, BlockKind::Body);
}

#[test]
fn test_la_ligne_de_separation_se_distingue_du_contenu() {
    assert_eq!(read("|---|---|")[0].0, BlockKind::TableRule);
    assert_eq!(read("|:--|--:|")[0].0, BlockKind::TableRule);
    assert_eq!(read("| --- | :---: |")[0].0, BlockKind::TableRule);
    // Sans tiret, ce n'est pas une séparation.
    assert_eq!(read("| : | : |")[0].0, BlockKind::TableRow);
    assert_eq!(read("| a | b |")[0].0, BlockKind::TableRow);
}

#[test]
fn test_les_cellules_sont_rognees_de_leurs_espaces() {
    assert_eq!(cellules("| a | b |"), vec!["a", "b"]);
    assert_eq!(cellules("|a|b|"), vec!["a", "b"]);
    assert_eq!(cellules("|   a   |   b   |"), vec!["a", "b"]);
    // La barre finale est facultative : les deux formes donnent les mêmes cellules.
    assert_eq!(cellules("| a | b"), cellules("| a | b |"));
}

#[test]
fn test_une_cellule_vide_reste_une_cellule() {
    assert_eq!(cellules("| a |  | c |"), vec!["a", "", "c"]);
    assert_eq!(cellules("|||"), vec!["", ""]);
}

/// Les tranches indexent bien la ligne : c'est ce dont le rendu et le curseur dépendent.
#[test]
fn test_les_cellules_indexent_les_vrais_octets() {
    let line = "| été | déjà |";
    let plages = cells(line);
    assert_eq!(&line[plages[0].clone()], "été");
    assert_eq!(&line[plages[1].clone()], "déjà");
    for p in &plages {
        assert!(line.is_char_boundary(p.start) && line.is_char_boundary(p.end));
    }
}

#[test]
fn test_un_tableau_dans_un_bloc_de_code_reste_du_code() {
    let source = "```\n| a | b |\n```";
    assert!(blocks(source).all(|b| !b.kind.in_table()));
}
