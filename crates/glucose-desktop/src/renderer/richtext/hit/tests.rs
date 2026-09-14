//! HIT-1 — le clic et le curseur lisent la même mise en page, et le prouvent.

use super::super::{layout_rich_text, TextBox, TextMode};
use super::*;
use crate::renderer::math::MathRenderer;

const BODY: f32 = 14.0;
const BX: TextBox = TextBox {
    usable: 240.0,
    body: BODY,
    bullet_indent: 14.0,
    line_height: BODY * super::super::LINE_FACTOR,
};

fn pose(source: &str, mode: TextMode) -> (Typography, TextLayout) {
    let typo = Typography::new();
    let layout = layout_rich_text(&typo, &MathRenderer::new(), source, BX, mode);
    (typo, layout)
}

/// Les textes d'épreuve : un nu, un mêlant les quatre emphases, un titre, une puce longue.
const CORPUS: &[&str] = &[
    "une ligne ordinaire",
    "du **gras** et de l'*italique* et du `code`",
    "# Un titre **net**",
    "- une puce **appuyée** assez longue pour devoir se couper en deux lignes visuelles",
];

/// **HIT-1, pendant l'édition.** En mode source, tous les octets sont dessinés : aller de
/// l'offset à l'abscisse puis revenir rend **exactement** le même offset.
///
/// C'est le mode qui compte pour la sélection — on ne sélectionne que ce qu'on édite — et
/// c'est l'invariant sans lequel une sélection à la souris est inutilisable : le curseur se
/// poserait ailleurs que sur le caractère cliqué, et l'écart grandirait avec le nombre de
/// styles de la ligne.
#[test]
fn test_hit_1_while_editing_the_round_trip_is_exact() {
    for source in CORPUS {
        let (typo, layout) = pose(source, TextMode::Source);
        for line in &layout.lines {
            let font = super::super::font_of(line.kind, BODY);
            for fragment in layout.fragments_of(line) {
                for (i, _) in source[fragment.start..fragment.end].char_indices() {
                    let offset = fragment.start + i;
                    let x = offset_to_x(&typo, &layout, line, source, offset, font);
                    let retour = x_to_offset(&typo, &layout, line, source, x, font);
                    assert_eq!(
                        retour, offset,
                        "{source:?} : l'offset {offset} revient en {retour} (abscisse {x})"
                    );
                }
            }
        }
    }
}

/// **HIT-1, au repos.** Les signes effacés n'occupent aucune place, donc plusieurs offsets
/// partagent la même abscisse — `# Un titre` commence à zéro, et l'offset `0` comme l'offset
/// `2` y tombent. L'aller-retour ne peut donc pas rendre le même offset, et ce serait une
/// erreur de l'exiger.
///
/// Ce qui doit tenir, et qui suffit, c'est que l'offset rendu **se dessine au même endroit** :
/// un clic pose le curseur là où le doigt a visé, même si l'octet choisi n'est pas celui que
/// l'on croyait.
#[test]
fn test_hit_1_at_rest_the_round_trip_lands_at_the_same_place() {
    for source in CORPUS {
        let (typo, layout) = pose(source, TextMode::Rendered);
        for line in &layout.lines {
            let font = super::super::font_of(line.kind, BODY);
            for fragment in layout.fragments_of(line) {
                for (i, _) in source[fragment.start..fragment.end].char_indices() {
                    let offset = fragment.start + i;
                    let x = offset_to_x(&typo, &layout, line, source, offset, font);
                    let retour = x_to_offset(&typo, &layout, line, source, x, font);
                    let x_retour = offset_to_x(&typo, &layout, line, source, retour, font);
                    assert!(
                        (x - x_retour).abs() < 1e-3,
                        "{source:?} : l'offset {offset} est en {x}, mais {retour} en {x_retour}"
                    );
                }
            }
        }
    }
}

/// **Le clic est stable.** Cliquer, dessiner le curseur, recliquer au même endroit rend le
/// même offset : la conversion ne dérive pas à force d'aller-retours, ce qui serait le cas
/// si une moitié de caractère était comptée deux fois.
#[test]
fn test_a_click_is_idempotent() {
    for source in CORPUS {
        for mode in [TextMode::Rendered, TextMode::Source] {
            let (typo, layout) = pose(source, mode);
            for line in &layout.lines {
                let font = super::super::font_of(line.kind, BODY);
                for pas in 0..60 {
                    let x = pas as f32 * 4.0;
                    let une = x_to_offset(&typo, &layout, line, source, x, font);
                    let x_une = offset_to_x(&typo, &layout, line, source, une, font);
                    let deux = x_to_offset(&typo, &layout, line, source, x_une, font);
                    assert_eq!(
                        une, deux,
                        "{source:?} en {mode:?} : le clic en {x} dérive de {une} à {deux}"
                    );
                }
            }
        }
    }
}

/// Au repos, cliquer tout à gauche d'un titre pose le curseur **au début du mot**, après le
/// `# ` effacé : c'est là que l'auteur qui vise le début de son titre veut écrire.
///
/// Pendant l'édition, le `# ` est visible et occupe sa place : le même clic pose alors le
/// curseur devant lui, comme devant n'importe quel caractère.
#[test]
fn test_clicking_at_the_left_edge_lands_on_what_is_drawn_there() {
    let source = "# Titre";
    let (typo, layout) = pose(source, TextMode::Rendered);
    let line = &layout.lines[0];
    let font = super::super::font_of(line.kind, BODY);
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, 0.0, font),
        2,
        "au repos, le bord gauche est le « T »"
    );

    let (typo, layout) = pose(source, TextMode::Source);
    let line = &layout.lines[0];
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, 0.0, font),
        0,
        "en édition, le bord gauche est le « # »"
    );
}

/// Cliquer sur la moitié gauche d'une lettre pose le curseur devant, sur la moitié droite
/// derrière. La règle de tous les éditeurs, et la seule prévisible sur un « M » large.
#[test]
fn test_a_click_lands_on_the_nearer_side_of_the_letter() {
    let source = "MMM";
    let (typo, layout) = pose(source, TextMode::Rendered);
    let line = &layout.lines[0];
    let m = typo.advance('M', BODY, crate::typography::Face::Regular);

    assert_eq!(x_to_offset(&typo, &layout, line, source, 0.0, BODY), 0);
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, m * 0.49, BODY),
        0,
        "la moitié gauche du premier M"
    );
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, m * 0.51, BODY),
        1,
        "la moitié droite du premier M"
    );
    assert_eq!(x_to_offset(&typo, &layout, line, source, m * 1.51, BODY), 2);
}

/// À droite de tout, on tombe à la fin de la ligne **source**, signes compris : cliquer après
/// `**gras**` pour entrer en édition pose le curseur après le signe fermant, prêt à écrire
/// hors du gras — et non coincé entre le mot et son signe.
///
/// À gauche, c'est le premier caractère dessiné qui gagne (voir
/// [`test_clicking_at_the_left_edge_lands_on_what_is_drawn_there`]).
#[test]
fn test_clicking_past_the_right_edge_lands_after_the_closing_sign() {
    let source = "**gras**";
    let (typo, layout) = pose(source, TextMode::Rendered);
    let line = &layout.lines[0];
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, -50.0, BODY),
        2,
        "à gauche : le « g », premier caractère dessiné"
    );
    assert_eq!(
        x_to_offset(&typo, &layout, line, source, 9_000.0, BODY),
        source.len(),
        "la fin de la ligne source, pas la fin du dernier fragment dessiné"
    );
}

/// Un clic au-dessus ou en dessous du texte se rabat sur la première ou la dernière ligne :
/// cliquer dans la marge d'une carte pose le curseur au plus près, jamais nulle part.
#[test]
fn test_a_click_outside_the_text_falls_back_on_the_nearest_line() {
    let source = "premiere\nseconde\ntroisieme";
    let (_, layout) = pose(source, TextMode::Rendered);
    assert_eq!(layout.lines.len(), 3);
    let h = BX.line_height;
    assert_eq!(line_at(&layout, -400.0, h), 0);
    assert_eq!(line_at(&layout, 0.0, h), 0);
    assert_eq!(line_at(&layout, h * 1.5, h), 1);
    assert_eq!(line_at(&layout, h * 900.0, h), 2);
}

/// Chaque offset appartient à une ligne, et c'est celle où le curseur se dessinera.
#[test]
fn test_every_offset_belongs_to_a_line() {
    let source = "premiere\nseconde\ntroisieme";
    let (_, layout) = pose(source, TextMode::Rendered);
    for (attendue, offset) in [(0, 0), (0, 8), (1, 9), (1, 16), (2, 17), (2, source.len())] {
        assert_eq!(
            line_of_offset(&layout, offset),
            attendue,
            "l'offset {offset} devrait être sur la ligne {attendue}"
        );
    }
}

/// Le texte d'une puce est décalé : un clic doit tenir compte de son indentation, sinon
/// chaque caractère d'une liste se sélectionne un cran trop à droite.
#[test]
fn test_a_bullet_indent_shifts_the_hit_test() {
    let source = "- une puce";
    let (typo, layout) = pose(source, TextMode::Rendered);
    let vise = source.find("puce").expect("le mot est là");
    let line = &layout.lines[0];
    let x = offset_to_x(&typo, &layout, line, source, vise, BODY);
    // Sans l'indentation, ce point tomberait plus loin dans le mot.
    let avec = offset_at(&typo, &layout, source, (x + BX.bullet_indent, 0.0), &BX);
    assert_eq!(avec, vise);
}
