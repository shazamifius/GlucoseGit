//! Ce que la mise en page riche garantit : RICH-1, MODE-1, et l'accord avec le reflux.

use super::*;
use crate::renderer::math::MathRenderer;

/// Une boîte assez large pour qu'un paragraphe court y tienne d'un seul tenant.
const LARGE: TextBox = TextBox {
    usable: 600.0,
    body: 14.0,
    bullet_indent: 14.0,
    line_height: 14.0 * LINE_FACTOR,
};

fn pose(source: &str, bx: TextBox, mode: TextMode) -> TextLayout {
    let typo = Typography::new();
    layout_rich_text(&typo, &MathRenderer::new(), source, bx, mode)
}

/// Ce qui serait dessiné : par ligne, les fragments et leur visage.
fn dessine<'a>(layout: &TextLayout, source: &'a str) -> Vec<Vec<(&'a str, Face)>> {
    layout
        .lines
        .iter()
        .map(|l| {
            layout
                .fragments_of(l)
                .iter()
                .map(|f| (&source[f.start..f.end], f.face()))
                .collect()
        })
        .collect()
}

/// **RICH-1.** Une ligne n'a plus un style : elle a des fragments, chacun avec son visage.
#[test]
fn test_a_line_carries_one_fragment_per_style() {
    let source = "du **gras** et de l'*italique* et du `code`";
    let layout = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(
        dessine(&layout, source),
        [[
            ("du ", Face::Regular),
            ("gras", Face::Bold),
            (" et de l'", Face::Regular),
            ("italique", Face::Italic),
            (" et du ", Face::Regular),
            ("code", Face::Mono),
        ]]
    );
}

/// **MODE-1.** Au repos les signes n'existent pas ; en édition ils sont là, en gris, et le
/// texte garde son style. Les octets, eux, ne bougent pas d'un mode à l'autre — c'est de cela
/// que le curseur dépend.
#[test]
fn test_the_markers_vanish_at_rest_and_return_while_editing() {
    let source = "**gras**";
    let rendu = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(dessine(&rendu, source), [[("gras", Face::Bold)]]);

    let edition = pose(source, LARGE, TextMode::Source);
    assert_eq!(
        dessine(&edition, source),
        [[
            ("**", Face::Regular),
            ("gras", Face::Bold),
            ("**", Face::Regular),
        ]]
    );
    // Le fragment du texte désigne les mêmes octets dans les deux vues.
    let au_repos = rendu.fragments[0];
    let en_edition = edition.fragments[1];
    assert_eq!(
        (au_repos.start, au_repos.end),
        (en_edition.start, en_edition.end)
    );
}

/// Le préfixe d'un bloc est un signe comme un autre : invisible au repos, visible à
/// l'édition. Il ne l'était **dans aucun des deux** avant ce module — on éditait `# Titre`
/// en voyant `Titre`, et écrire au tout début insérait avant un `#` qu'on ne voyait pas.
#[test]
fn test_a_block_prefix_is_a_marker_too() {
    let source = "# Titre";
    let rendu = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(dessine(&rendu, source), [[("Titre", Face::Bold)]]);

    let edition = pose(source, LARGE, TextMode::Source);
    assert_eq!(
        dessine(&edition, source),
        [[("# ", Face::Regular), ("Titre", Face::Bold)]]
    );
    assert_eq!(
        edition.fragments[0].role,
        SpanRole::Marker,
        "le `# ` est un signe, donc il se dessine en gris"
    );
}

/// L'emphase du genre s'unit à celle des fragments : un mot en italique dans un titre est en
/// gras italique, sans que personne n'ait eu à écrire ce cas.
#[test]
fn test_a_headings_weight_combines_with_inline_emphasis() {
    let source = "# Le *mot* juste";
    let layout = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(
        dessine(&layout, source),
        [[
            ("Le ", Face::Bold),
            ("mot", Face::BoldItalic),
            (" juste", Face::Bold),
        ]]
    );
}

/// Le code l'emporte sur la graisse : `**`x`**` est de la chasse fixe, pas du gras.
#[test]
fn test_code_wins_over_weight() {
    let source = "**du `code` gras**";
    let layout = pose(source, LARGE, TextMode::Rendered);
    let visages: Vec<_> = layout.fragments.iter().map(|f| f.face()).collect();
    assert_eq!(
        visages,
        [Face::Bold, Face::Mono, Face::Bold],
        "le code garde sa chasse fixe au milieu d'un passage en gras"
    );
}

/// **La largeur d'une ligne tient compte des visages.** Le gras est plus large que le corps :
/// à largeur de boîte égale, un texte entièrement en gras se coupe plus tôt. Avant RICH-1,
/// la mesure ne connaissait qu'une police et la coupe était fausse.
#[test]
fn test_wrapping_measures_each_fragment_with_its_own_face() {
    let bx = TextBox {
        usable: 120.0,
        ..LARGE
    };
    let maigre = pose("alpha beta gamma delta epsilon", bx, TextMode::Rendered);
    let gras = pose("**alpha beta gamma delta epsilon**", bx, TextMode::Rendered);
    assert!(
        gras.line_count() >= maigre.line_count(),
        "le gras est plus large : il ne peut pas tenir sur moins de lignes ({} contre {})",
        gras.line_count(),
        maigre.line_count()
    );
}

/// **Les signes n'occupent aucune place au repos.** Sans cela, une ligne se couperait plus
/// tôt qu'elle ne s'affiche, et la carte réserverait de la hauteur pour du vide.
#[test]
fn test_markers_take_no_room_when_they_are_not_drawn() {
    let bx = TextBox {
        usable: 120.0,
        ..LARGE
    };
    let nu = pose("alpha beta gamma delta", bx, TextMode::Rendered);
    let marque = pose("*alpha* *beta* *gamma* *delta*", bx, TextMode::Rendered);
    assert_eq!(
        nu.line_count(),
        marque.line_count(),
        "huit étoiles effacées ne doivent pas coûter une ligne de plus"
    );
}

/// La partition de la source survit au reflux : les fragments d'une ligne se suivent, et
/// ceux de toutes les lignes couvrent le texte sans en perdre un octet.
#[test]
fn test_fragments_stay_in_order_across_wrapped_lines() {
    let source = "un **texte** assez long pour *devoir* se couper en plusieurs lignes visuelles";
    let bx = TextBox {
        usable: 140.0,
        ..LARGE
    };
    let layout = pose(source, bx, TextMode::Rendered);
    assert!(layout.line_count() > 1, "le texte doit se couper");
    let mut at = 0usize;
    for fragment in &layout.fragments {
        assert!(
            fragment.start >= at,
            "les fragments reculent : {fragment:?} après {at}"
        );
        assert!(
            fragment.end > fragment.start,
            "fragment vide : {fragment:?}"
        );
        at = fragment.end;
    }
}

/// Un paragraphe sans le moindre signe donne un fragment par ligne : le cas courant reste le
/// cas économe, alors même que la mise en page tourne à chaque frame.
#[test]
fn test_plain_paragraphs_cost_one_fragment_per_line() {
    let source = "une phrase entièrement ordinaire";
    let layout = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.fragments.len(), 1);
}

/// Une puce garde son indentation, et son texte peut être stylé.
#[test]
fn test_a_bullet_keeps_its_indent_and_can_be_styled() {
    let source = "- une **puce**";
    let layout = pose(source, LARGE, TextMode::Rendered);
    assert_eq!(layout.lines[0].kind, BlockKind::Bullet);
    assert_eq!(
        dessine(&layout, source),
        [[("une ", Face::Regular), ("puce", Face::Bold)]]
    );
}

/// Une formule est une **formule au repos, et du texte pendant qu'on l'écrit**.
///
/// Au repos elle réserve la hauteur de son résultat et ne se coupe pas. En édition elle
/// redevient un paragraphe ordinaire : elle reflue si elle est trop large, et ses `$` sont des
/// signes — sans quoi rien ne pourrait les colorer selon qu'elle compile.
#[test]
fn test_une_formule_est_du_texte_pendant_quon_lecrit() {
    let typo = Typography::new();
    let math = MathRenderer::new();
    let source = "$$a^2 + b^2 + c^2 + d^2 + e^2 + f^2 = g^2 + h^2$$";
    let bx = TextBox {
        usable: 160.0,
        body: 14.0,
        bullet_indent: 14.0,
        line_height: 19.6,
    };

    let repos = layout_rich_text(&typo, &math, source, bx, TextMode::Rendered);
    let signes_au_repos = repos
        .fragments
        .iter()
        .filter(|f| f.role == SpanRole::Marker)
        .count();
    assert_eq!(signes_au_repos, 0, "au repos, les signes n'existent plus");

    let edition = layout_rich_text(&typo, &math, source, bx, TextMode::Source);
    let signes = edition
        .fragments
        .iter()
        .filter(|f| f.role == SpanRole::Marker)
        .count();
    assert_eq!(
        signes, 2,
        "en édition : les `$$` d'ouverture et de fermeture"
    );
    assert!(
        edition.lines.len() > 1,
        "et une source plus large que la carte se coupe au lieu de déborder"
    );
    assert!(
        edition.lines.iter().all(|l| l.kind == repos.lines[0].kind),
        "elle reste une formule pour qui l'interroge"
    );
}

// ── Les tableaux (fiche 08 § 2.1) ────────────────────────────────────────────

fn table_layout(source: &str) -> TextLayout {
    let typo = Typography::new();
    let math = MathRenderer::new();
    layout_rich_text(
        &typo,
        &math,
        source,
        TextBox {
            usable: 400.0,
            body: 14.0,
            bullet_indent: 14.0,
            line_height: 19.6,
        },
        TextMode::Rendered,
    )
}

/// TABLE-1 — deux lignes de longueurs différentes mettent quand même leurs colonnes en face.
#[test]
fn test_table_1_les_colonnes_salignent_dune_ligne_a_lautre() {
    let source = "| a | court |\n|---|---|\n| beaucoup plus long | x |";
    let layout = table_layout(source);

    // Le taquet de la deuxième colonne, sur chaque ligne qui en a une.
    let taquets: Vec<f32> = layout
        .lines
        .iter()
        .filter(|l| !l.kind.silent())
        .filter_map(|l| {
            let frags = layout.fragments_of(l);
            frags.iter().filter(|f| f.tab >= 0.0).nth(1).map(|f| f.tab)
        })
        .collect();

    assert_eq!(taquets.len(), 2, "deux lignes de contenu");
    assert!(
        (taquets[0] - taquets[1]).abs() < 1e-6,
        "les deuxièmes colonnes ne sont pas en face : {taquets:?}"
    );
    assert!(taquets[0] > 0.0, "et la colonne est bien décalée");
}

/// La ligne de séparation ne porte aucun texte : c'est un filet, pas des tirets à lire.
#[test]
fn test_la_ligne_de_separation_ne_porte_aucun_texte() {
    let layout = table_layout("| a | b |\n|---|---|\n| c | d |");
    let regle = layout
        .lines
        .iter()
        .find(|l| l.kind == BlockKind::TableRule)
        .expect("une ligne de séparation");
    assert_eq!(layout.fragments_of(regle).len(), 0);
}

/// L'en-tête est la ligne qui précède la séparation, et elle seule.
#[test]
fn test_len_tete_est_la_ligne_avant_la_separation() {
    let layout = table_layout("| a | b |\n|---|---|\n| c | d |");
    let gras: Vec<bool> = layout
        .lines
        .iter()
        .filter(|l| !l.kind.silent())
        .map(|l| layout.fragments_of(l).iter().all(|f| f.emphasis.bold()))
        .collect();
    assert_eq!(gras, vec![true, false], "la première seulement");

    // Sans séparation, aucune ligne n'est un en-tête : c'est une grille, pas un tableau.
    let sans = table_layout("| a | b |\n| c | d |");
    assert!(sans.fragments.iter().all(|f| !f.emphasis.bold()));
}

/// Un taquet ne sert qu'aux tableaux : partout ailleurs, un fragment suit le précédent.
#[test]
fn test_hors_tableau_aucun_fragment_ne_porte_de_taquet() {
    for source in ["du corps", "# Titre", "- une puce", "**gras** et `code`"] {
        let layout = table_layout(source);
        assert!(
            layout.fragments.iter().all(|f| f.tab == NO_TAB),
            "{source:?} pose un taquet"
        );
    }
}

/// Les cellules gardent leur style : une cellule en gras le reste.
#[test]
fn test_une_cellule_garde_son_markdown() {
    let source = "| a | b |\n|---|---|\n| **fort** | `code` |";
    let layout = table_layout(source);
    let derniere = layout.lines.last().expect("une dernière ligne");
    let frags = layout.fragments_of(derniere);
    assert!(
        frags.iter().any(|f| f.emphasis.bold()),
        "le gras d'une cellule"
    );
    assert!(
        frags.iter().any(|f| f.emphasis.code()),
        "et le code d'une autre"
    );
}
