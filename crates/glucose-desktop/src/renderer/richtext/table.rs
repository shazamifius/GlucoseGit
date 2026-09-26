//! La mise en page d'un tableau : des colonnes qui s'alignent.
//!
//! # TABLE-1 — une colonne est aussi large que sa plus large cellule
//!
//! C'est la seule partie du texte qui ne se met pas en page ligne par ligne : la largeur
//! d'une colonne dépend de **toutes** les lignes du tableau. Elles sont donc rassemblées
//! avant d'être posées, et c'est pour cela que ce travail vit à part.
//!
//! Chaque cellule reste du texte ordinaire — elle peut porter du gras, du code, un lien —
//! et c'est [`super::Fragment::tab`] qui la place sur le taquet de sa colonne.

use super::{Fragment, TextBox, TextLayout, TextMode, VisualLine, NO_TAB};
use crate::typography::{Face, Typography};
use glucose_core::text::{inline_spans, Block, SpanRole};

/// TABLE-1 — les colonnes d'un tableau s'alignent sur la plus large de leurs cellules.
///
/// Chaque cellule est analysée comme du texte ordinaire — elle peut porter du gras, du code,
/// un lien — puis mesurée. Le taquet d'une colonne est la somme des largeurs des colonnes qui
/// la précèdent, plus une gouttière par colonne franchie. Le premier fragment de chaque
/// cellule porte ce taquet ; les suivants se posent à sa suite, comme partout ailleurs.
///
/// Un tableau ne se reflue pas : une cellule trop large élargit sa colonne, et le tableau
/// déborde de la carte plutôt que de couper une cellule en deux. C'est ce que fait un
/// tableau — le couper le rendrait illisible, et la carte se redimensionne.
pub(super) fn layout_table(
    out: &mut TextLayout,
    typography: &Typography,
    source: &str,
    rows: &[Block],
    bx: TextBox,
    mode: TextMode,
) {
    // La gouttière entre deux colonnes : deux espaces de la police courante. Une longueur
    // dite dans l'unité du texte, donc qui suit sa taille sans avoir à la connaître.
    let gouttiere = typography.advance(' ', bx.body, Face::Regular) * 2.0;

    let cellules: Vec<Vec<std::ops::Range<usize>>> = rows
        .iter()
        .map(|b| {
            let ligne = b.slice(source);
            glucose_core::text::block::cells(ligne)
                .into_iter()
                .map(|r| b.start + r.start..b.start + r.end)
                .collect()
        })
        .collect();

    let colonnes = cellules.iter().map(Vec::len).max().unwrap_or(0);
    let mut largeurs = vec![0.0f32; colonnes];
    for (row, plages) in rows.iter().zip(&cellules) {
        if row.kind.silent() {
            continue;
        }
        for (c, plage) in plages.iter().enumerate() {
            largeurs[c] = largeurs[c].max(mesure_cellule(typography, source, plage.clone(), bx));
        }
    }
    // En corps de la ligne (TABLE-2) : le taquet suit le zoom sans le connaître.
    let taquet =
        |c: usize| -> f32 { (largeurs[..c].iter().sum::<f32>() + gouttiere * c as f32) / bx.body };

    // L'en-tête est la ligne qui **précède** la séparation — c'est elle qui fait d'un
    // tableau un tableau plutôt qu'une grille. Le noyau ne peut pas le dire : un bloc ne
    // regarde pas ses voisins, et c'est ici qu'on les a tous.
    let entete = rows
        .iter()
        .position(|b| b.kind.silent())
        .filter(|&i| i > 0)
        .map(|i| i - 1);

    for (rang, (row, plages)) in rows.iter().zip(&cellules).enumerate() {
        let from = out.fragments.len();
        if !row.kind.silent() {
            let gras = entete == Some(rang);
            for (c, plage) in plages.iter().enumerate() {
                pose_cellule(out, source, plage.clone(), taquet(c), mode, gras);
            }
        }
        out.lines.push(VisualLine {
            start: row.start,
            end: row.end,
            kind: row.kind,
            first: true,
            paragraph_start: row.start,
            fragments: from..out.fragments.len(),
        });
    }
}

/// La largeur qu'une cellule occupe, mesurée comme elle sera dessinée.
fn mesure_cellule(
    typography: &Typography,
    source: &str,
    plage: std::ops::Range<usize>,
    bx: TextBox,
) -> f32 {
    let texte = &source[plage];
    inline_spans(texte)
        .iter()
        .filter(|s| s.role == SpanRole::Text)
        .map(|s| {
            typography
                .measure_text(s.slice(texte), bx.body, Face::from(s.emphasis))
                .0
        })
        .sum()
}

/// Les fragments d'une cellule, le premier posé sur le taquet de sa colonne.
///
/// Les barres d'un tableau ne sont pas dans les cellules : elles ne sont donc ni dessinées ni
/// mesurées, et l'alignement des colonnes les remplace. En édition, la source de la ligne
/// reste lisible telle quelle — ce sont les taquets qui disparaissent, pas le texte.
fn pose_cellule(
    out: &mut TextLayout,
    source: &str,
    plage: std::ops::Range<usize>,
    taquet: f32,
    mode: TextMode,
    gras: bool,
) {
    let texte = &source[plage.clone()];
    let mut premier = true;
    for span in inline_spans(texte) {
        if mode == TextMode::Rendered && span.role == SpanRole::Marker {
            continue;
        }
        out.fragments.push(Fragment {
            tab: if premier { taquet } else { NO_TAB },
            start: plage.start + span.start,
            end: plage.start + span.end,
            emphasis: if gras && span.role == SpanRole::Text {
                span.emphasis.union(glucose_core::text::Emphasis::BOLD)
            } else {
                span.emphasis
            },
            role: span.role,
        });
        premier = false;
    }
    // Une cellule vide n'a aucun fragment : le taquet de la suivante suffit à la laisser
    // vide, puisqu'il est absolu.
}
