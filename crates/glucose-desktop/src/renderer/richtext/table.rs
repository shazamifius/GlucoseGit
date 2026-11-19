//! La mise en page d'un tableau : des colonnes qui s'alignent, et qui tiennent dans le nœud.
//!
//! # TABLE-1 — les colonnes s'alignent d'une rangée à l'autre
//!
//! C'est la seule partie du texte qui ne se met pas en page paragraphe par paragraphe : la
//! largeur d'une colonne dépend de **toutes** les rangées du tableau. Elles sont donc
//! rassemblées avant d'être posées, et c'est pour cela que ce travail vit à part.
//!
//! Chaque cellule reste du texte ordinaire — elle peut porter du gras, du code, un lien —
//! et c'est [`super::Fragment::tab`] qui la place sur le taquet de sa colonne.
//!
//! # TABLE-3 — un tableau tient dans la largeur de son nœud (fiche 123)
//!
//! TABLE-1 disait qu'« un tableau ne se reflue pas » : une colonne prenait sa plus large
//! cellule, et le tableau débordait du nœud. C'était faux de ce que fait un tableau — un
//! navigateur, donc Glucose Tauri, coupe ses cellules en lignes —, et sa capture du 19/11
//! l'a montré : les tableaux qu'une IA a posés par la porte sortaient de leur bloc et
//! couvraient la membrane voisine.
//!
//! La largeur se répartit comme dans la spécification CSS des tableaux (*CSS Tables 3*,
//! § 3.9.3) : chaque colonne a une largeur **minimale** — son mot le plus large, celui qu'on
//! ne coupe pas — et une largeur **naturelle** — sa plus longue cellule sur une seule ligne.
//! Si la place suffit, les colonnes gardent leur largeur naturelle ; sinon, chacune prend
//! *« la combinaison linéaire, de poids qui somment à 1 »*, de ses deux largeurs qui remplit
//! exactement la place. Aucun nombre à choisir. Et là où la CSS laisse déborder — quand même
//! les mots les plus larges ne tiennent pas —, Glucose réduit les colonnes en proportion et
//! coupe les mots entre deux lettres, comme partout ailleurs ([`wrap_paragraph`]) : un texte
//! ne sort jamais de son nœud.
//!
//! Chaque cellule se coupe alors dans sa colonne ; une rangée a la hauteur de sa plus haute
//! cellule, et ses lignes visuelles portent, chacune, la ligne de même rang de chaque cellule.
//!
//! Au repos seulement : en édition, une rangée est sa source, barres comprises, coupée comme
//! un paragraphe (MODE-1 — les signes reviennent quand on corrige).

use super::{span_at, Fragment, TextBox, TextLayout, TextMode, VisualLine, NO_TAB};
use crate::renderer::wrap::wrap_paragraph;
use crate::typography::{Face, Typography};
use glucose_core::text::{inline_spans, Block, Emphasis, Span, SpanRole};
use std::ops::Range;

/// Une cellule : sa tranche de la source, ses tranches de Markdown, et si elle est d'en-tête.
struct Cellule<'a> {
    plage: Range<usize>,
    texte: &'a str,
    spans: Vec<Span>,
    gras: bool,
}

impl Cellule<'_> {
    /// Le visage de l'octet `at` de la cellule — ou rien pour un signe effacé au repos
    /// (MODE-1), qui n'occupe aucune place.
    fn face(&self, at: usize, mode: TextMode) -> Option<Face> {
        let span = span_at(&self.spans, at)?;
        if mode == TextMode::Rendered && span.role == SpanRole::Marker {
            return None;
        }
        Some(Face::from(self.emphase(span)))
    }

    /// L'emphase qu'un fragment de cette cellule porte : l'en-tête est en gras, ses signes non.
    fn emphase(&self, span: &Span) -> Emphasis {
        if self.gras && span.role == SpanRole::Text {
            span.emphasis.union(Emphasis::BOLD)
        } else {
            span.emphasis
        }
    }

    /// L'avance de l'octet `at`, qui porte `ch`, au corps `corps`.
    fn avance(
        &self,
        (typo, corps, mode): (&Typography, f32, TextMode),
        at: usize,
        ch: char,
    ) -> f32 {
        self.face(at, mode)
            .map_or(0.0, |f| typo.advance(ch, corps, f))
    }

    /// **Ses deux largeurs** (TABLE-3) : son mot le plus large — là où [`wrap_paragraph`] ne
    /// coupe pas, entre deux espaces — et sa largeur sur une seule ligne. Les avances
    /// s'additionnent dans l'ordre où la coupe les additionne : une colonne large de son
    /// minimum ne coupe aucun mot.
    fn largeurs(&self, outils: (&Typography, f32, TextMode)) -> (f32, f32) {
        let (mut mot, mut minimale, mut naturelle) = (0.0f32, 0.0f32, 0.0f32);
        for (at, ch) in self.texte.char_indices() {
            let a = self.avance(outils, at, ch);
            naturelle += a;
            mot = if ch == ' ' { 0.0 } else { mot + a };
            minimale = minimale.max(mot);
        }
        (minimale, naturelle)
    }
}

/// TABLE-1 et TABLE-3 — les colonnes s'alignent, et le tableau tient dans `bx.usable`.
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
    let rangees = cellules(source, rows);
    let outils = (typography, bx.body, mode);

    let colonnes = rangees.iter().map(Vec::len).max().unwrap_or(0);
    let (mut minimales, mut naturelles) = (vec![0.0f32; colonnes], vec![0.0f32; colonnes]);
    for (row, cellules) in rows.iter().zip(&rangees) {
        if row.kind.silent() {
            continue;
        }
        for (c, cellule) in cellules.iter().enumerate() {
            let (minimale, naturelle) = cellule.largeurs(outils);
            minimales[c] = minimales[c].max(minimale);
            naturelles[c] = naturelles[c].max(naturelle);
        }
    }
    let gouttieres = gouttiere * colonnes.saturating_sub(1) as f32;
    let largeurs = repartir(&minimales, &naturelles, (bx.usable - gouttieres).max(0.0));
    // En corps de la ligne (TABLE-2) : le taquet suit le zoom sans le connaître.
    let taquets: Vec<f32> = (0..colonnes)
        .map(|c| (largeurs[..c].iter().sum::<f32>() + gouttiere * c as f32) / bx.body)
        .collect();

    for (row, cellules) in rows.iter().zip(&rangees) {
        if row.kind.silent() {
            out.lines.push(ligne_de_la_rangee(row, 0, 0..0, (0.0, 0.0)));
            continue;
        }
        let coupes: Vec<Vec<(usize, usize)>> = cellules
            .iter()
            .zip(&largeurs)
            .map(|(cellule, &largeur)| {
                wrap_paragraph(cellule.texte, largeur, |at, ch| {
                    cellule.avance(outils, at, ch)
                })
            })
            .collect();
        let hauteur = coupes.iter().map(Vec::len).max().unwrap_or(0).max(1);
        for rang in 0..hauteur {
            poser_une_ligne(out, (row, cellules, &coupes), (&taquets, rang), outils);
        }
    }
}

/// **TABLE-3 — la largeur de chaque colonne**, pour que le tableau remplisse au plus `place`.
///
/// La spécification CSS des tableaux (§ 3.9.3) : entre la répartition où chaque colonne a sa
/// largeur minimale et celle où elle a sa largeur naturelle, la combinaison linéaire qui
/// remplit la place ; au-delà de la naturelle, la naturelle. En deçà de la minimale (la CSS
/// déborde), les minimales réduites en proportion : la fonction reste continue, et un nœud
/// qu'on rétrécit resserre ses colonnes sans saut.
fn repartir(minimales: &[f32], naturelles: &[f32], place: f32) -> Vec<f32> {
    let somme_min: f32 = minimales.iter().sum();
    let somme_nat: f32 = naturelles.iter().sum();
    if place >= somme_nat {
        return naturelles.to_vec();
    }
    if place >= somme_min {
        // Ici `somme_nat > place >= somme_min` : le dénominateur n'est jamais nul.
        let t = (place - somme_min) / (somme_nat - somme_min);
        return minimales
            .iter()
            .zip(naturelles)
            .map(|(m, n)| m + (n - m) * t)
            .collect();
    }
    // `place < somme_min` et `place >= 0` : `somme_min` est strictement positive.
    minimales.iter().map(|m| m * place / somme_min).collect()
}

/// Les cellules de chaque rangée, analysées une fois — l'en-tête en gras.
///
/// L'en-tête est la rangée qui **précède** la séparation — c'est elle qui fait d'un tableau
/// un tableau plutôt qu'une grille. Le noyau ne peut pas le dire : un bloc ne regarde pas ses
/// voisins, et c'est ici qu'on les a tous.
fn cellules<'a>(source: &'a str, rows: &[Block]) -> Vec<Vec<Cellule<'a>>> {
    let entete = rows
        .iter()
        .position(|b| b.kind.silent())
        .filter(|&i| i > 0)
        .map(|i| i - 1);
    rows.iter()
        .enumerate()
        .map(|(rang, b)| {
            glucose_core::text::block::cells(b.slice(source))
                .into_iter()
                .map(|r| {
                    let plage = b.start + r.start..b.start + r.end;
                    let texte = &source[plage.clone()];
                    Cellule {
                        spans: inline_spans(texte),
                        plage,
                        texte,
                        gras: entete == Some(rang),
                    }
                })
                .collect()
        })
        .collect()
}

/// La ligne de rang `rang` d'une rangée : celle de même rang de chaque cellule, sur son taquet.
fn poser_une_ligne(
    out: &mut TextLayout,
    (row, cellules, coupes): (&Block, &[Cellule<'_>], &[Vec<(usize, usize)>]),
    (taquets, rang): (&[f32], usize),
    outils: (&Typography, f32, TextMode),
) {
    let (typography, corps, mode) = outils;
    let from = out.fragments.len();
    let (mut largeur, mut encre) = (0.0f32, 0.0f32);
    for ((cellule, coupe), &taquet) in cellules.iter().zip(coupes).zip(taquets) {
        let Some(&(s, e)) = coupe.get(rang) else {
            continue;
        };
        poser_un_morceau(out, cellule, s..e, taquet, mode);
        // GRIS-1 : la ligne s'étend jusqu'au bout de son morceau le plus à droite.
        let lettres = cellule.texte[s..e]
            .char_indices()
            .map(|(i, ch)| (ch, cellule.face(s + i, mode)));
        let (l, en) = typography.gris(lettres, corps);
        if l > 0.0 {
            largeur = largeur.max(taquet * corps + l);
        }
        encre += en;
    }
    let fragments = from..out.fragments.len();
    out.lines
        .push(ligne_de_la_rangee(row, rang, fragments, (largeur, encre)));
}

/// Une ligne visuelle d'une rangée. Toutes portent la tranche de la rangée entière : leurs
/// morceaux s'entrelacent (la première ligne de chaque cellule, puis la deuxième…), et c'est
/// la rangée qu'un clic à droite de tout désigne.
fn ligne_de_la_rangee(
    row: &Block,
    rang: usize,
    fragments: Range<usize>,
    (largeur, encre): (f32, f32),
) -> VisualLine {
    VisualLine {
        start: row.start,
        end: row.end,
        kind: row.kind,
        first: rang == 0,
        paragraph_start: row.start,
        fragments,
        largeur,
        encre,
    }
}

/// Les fragments du morceau `[s, e)` d'une cellule, le premier posé sur le taquet de sa
/// colonne.
///
/// Les barres d'un tableau ne sont pas dans les cellules : elles ne sont donc ni dessinées ni
/// mesurées, et l'alignement des colonnes les remplace.
fn poser_un_morceau(
    out: &mut TextLayout,
    cellule: &Cellule<'_>,
    morceau: Range<usize>,
    taquet: f32,
    mode: TextMode,
) {
    let mut premier = true;
    for span in cellule
        .spans
        .iter()
        .filter(|sp| sp.start < morceau.end && sp.end > morceau.start)
    {
        if mode == TextMode::Rendered && span.role == SpanRole::Marker {
            continue;
        }
        out.fragments.push(Fragment {
            tab: if premier { taquet } else { NO_TAB },
            eclaire: false,
            start: cellule.plage.start + span.start.max(morceau.start),
            end: cellule.plage.start + span.end.min(morceau.end),
            emphasis: cellule.emphase(span),
            role: span.role,
        });
        premier = false;
    }
    // Une cellule vide n'a aucun fragment : le taquet de la suivante suffit à la laisser
    // vide, puisqu'il est absolu.
}

/// **Les tranches d'une rangée de tableau en édition** (TABLE-3) : ses cellules analysées
/// comme du texte, et ce qui les sépare — les barres et leurs espaces — en signes, gris comme
/// un `**` (MODE-1). Elles partitionnent la rangée (SPAN-1).
pub(super) fn spans_d_une_rangee(rangee: &str) -> Vec<Span> {
    let signe = |start, end| Span {
        start,
        end,
        emphasis: Emphasis::NONE,
        role: SpanRole::Marker,
    };
    let mut spans = Vec::new();
    let mut fin = 0;
    for cellule in glucose_core::text::block::cells(rangee) {
        if cellule.start > fin {
            spans.push(signe(fin, cellule.start));
        }
        spans.extend(
            inline_spans(&rangee[cellule.clone()])
                .into_iter()
                .map(|s| Span {
                    start: s.start + cellule.start,
                    end: s.end + cellule.start,
                    ..s
                }),
        );
        fin = fin.max(cellule.end);
    }
    if fin < rangee.len() {
        spans.push(signe(fin, rangee.len()));
    }
    spans
}

#[cfg(test)]
mod tests;
