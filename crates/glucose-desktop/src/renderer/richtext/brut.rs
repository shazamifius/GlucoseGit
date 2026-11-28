//! **Le texte brut** — celui d'un post-it (fiche 188 ; sa note du 27/11, `text-n3rk-2587`) : aucun
//! Markdown, exprès ; chaque ligne de la source est un paragraphe tel quel, reflué à la largeur.
//!
//! Il rend la même [`TextLayout`] qu'un texte riche : le clic qui devient une position
//! ([`super::hit`]), `↑` `↓` `Début` `Fin`, le surlignage d'une sélection
//! ([`super::draw::draw_line_selection`]) le lisent sans savoir d'où il vient. Le post-it gardait
//! sa propre liste de lignes, que rien de cela ne savait lire : on ne pouvait y choisir aucun mot.

use super::{Fragment, TextBox, TextLayout, VisualLine, NO_TAB};
use crate::renderer::wrap::wrap_paragraph;
use crate::typography::{Face, Typography};
use glucose_core::text::{BlockKind, Emphasis, SpanRole};

/// La mise en page d'un texte brut dans la boîte `bx` : une ligne visuelle et un fragment par
/// tranche que le reflux coupe, au corps courant, d'une seule emphase — le romain d'un post-it, le
/// gras du titre d'une membrane (fiche 188 § 4). Le fragment la porte : le clic et le surlignage
/// mesurent ses lettres dans la face qu'elle dit, celle du dessin.
pub fn layout_brut(
    typography: &Typography,
    source: &str,
    bx: TextBox,
    emphasis: Emphasis,
) -> TextLayout {
    let face = Face::from(emphasis);
    let advance = |_: usize, ch: char| typography.advance(ch, bx.body, face);
    let mut out = TextLayout::default();
    let mut debut = 0usize;
    for paragraphe in source.split('\n') {
        for (i, (s, e)) in wrap_paragraph(paragraphe, bx.usable, advance)
            .into_iter()
            .enumerate()
        {
            let (start, end) = (debut + s, debut + e);
            let fragment = out.fragments.len();
            out.fragments.push(Fragment {
                start,
                end,
                emphasis,
                role: SpanRole::Text,
                tab: NO_TAB,
                eclaire: false,
            });
            let lettres = paragraphe[s..e].chars().map(|ch| (ch, Some(face)));
            let (largeur, encre) = typography.gris(lettres, bx.body);
            out.lines.push(VisualLine {
                start,
                end,
                kind: BlockKind::Body,
                first: i == 0,
                paragraph_start: debut,
                fragments: fragment..fragment + 1,
                largeur,
                encre,
            });
        }
        debut += paragraphe.len() + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Un paragraphe trop long pour sa boîte se coupe en lignes visuelles** qui pavent sa source
    /// sans trou ni recouvrement — c'est ce que `↑` `↓` et le clic suivent dans un post-it.
    #[test]
    fn post_it_sel_le_texte_brut_reflue_a_sa_largeur() {
        let typo = Typography::new();
        let source = "un deux trois quatre cinq six sept huit neuf dix\nonze";
        let bx = TextBox {
            usable: 80.0,
            body: 13.0,
            bullet_indent: 0.0,
            line_height: 17.0,
        };
        let mise = layout_brut(&typo, source, bx, Emphasis::NONE);
        assert!(mise.lines.len() >= 4, "{} lignes", mise.lines.len());
        let premier = &source[..source.find('\n').expect("deux paragraphes")];
        let couvert: String = (mise.lines.iter())
            .filter(|l| l.paragraph_start == 0)
            .map(|l| &source[l.start..l.end])
            .collect();
        assert_eq!(couvert.replace(' ', ""), premier.replace(' ', ""));
        let dernier = mise.lines.last().expect("une ligne");
        assert_eq!(&source[dernier.start..dernier.end], "onze");
        assert!(
            dernier.first,
            "la seconde ligne source commence un paragraphe"
        );
    }
}
