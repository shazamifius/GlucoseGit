//! WRAP-1 — le découpage d'un paragraphe en lignes qui tiennent dans une largeur.
//!
//! Une carte ou un pense-bête redimensionné en largeur **reflue** son texte : la largeur
//! change, le découpage en lignes change. Avant ce module, un texte n'était coupé qu'à ses
//! retours à la ligne et débordait de sa boîte dès qu'une ligne était trop longue.
//!
//! Le découpage se fait en **unités monde, avant la mise à l'échelle** (CARD-1) : la largeur
//! de chaque caractère est celle de la police à l'échelle 1, et le résultat est le même à
//! tous les zooms. `Typography::measure_text` additionne les avances des glyphes sans
//! crénage, donc mesurer caractère par caractère est exact, et le coût est linéaire.

/// Découpe `body` en lignes d'au plus `max_width`, coupées de préférence aux espaces.
///
/// Rend des tranches `[start, end)` d'octets de `body`. L'espace où l'on coupe n'appartient
/// à aucune ligne. Un mot plus large que la ligne est coupé entre deux caractères plutôt que
/// de déborder. Un corps vide donne une ligne vide : une carte sans texte a quand même une
/// hauteur.
pub(super) fn wrap_paragraph(body: &str, max_width: f32, advance: impl Fn(char) -> f32) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut line_start = 0usize;
    let mut width = 0.0f32;
    // Dernier espace de la ligne courante : l'endroit où l'on préfère couper.
    let mut last_space: Option<usize> = None;

    for (i, ch) in body.char_indices() {
        let w = advance(ch);
        let overflows = width + w > max_width && i > line_start;
        if !overflows {
            if ch == ' ' {
                last_space = Some(i);
            }
            width += w;
            continue;
        }

        if ch == ' ' {
            // L'espace débordant devient la coupe : il n'est ni dessiné ni compté.
            lines.push((line_start, i));
            line_start = i + 1;
            width = 0.0;
            last_space = None;
            continue;
        }

        match last_space {
            Some(sp) => {
                lines.push((line_start, sp));
                line_start = sp + 1;
                last_space = None;
                width = body[line_start..i].chars().map(&advance).sum::<f32>() + w;
            }
            None => {
                // Un mot plus large que la ligne : on coupe entre deux caractères.
                lines.push((line_start, i));
                line_start = i;
                width = w;
            }
        }
    }
    lines.push((line_start, body.len()));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une police de test : chaque caractère avance de 10, quel qu'il soit.
    fn mono(_: char) -> f32 {
        10.0
    }

    fn pieces(body: &str, max_width: f32) -> Vec<&str> {
        wrap_paragraph(body, max_width, mono).into_iter().map(|(s, e)| &body[s..e]).collect()
    }

    #[test]
    fn test_wrap_1_a_short_line_is_left_alone() {
        assert_eq!(pieces("hello world", 200.0), vec!["hello world"]);
        assert_eq!(pieces("", 200.0), vec![""], "un corps vide a une ligne");
    }

    #[test]
    fn test_wrap_1_lines_break_at_spaces_and_the_space_vanishes() {
        // 12 caractères par ligne : « hello world » (11) tient, « hello world again » non.
        assert_eq!(pieces("hello world again", 120.0), vec!["hello world", "again"]);
        assert_eq!(pieces("aaa bbb ccc ddd", 70.0), vec!["aaa bbb", "ccc ddd"]);
    }

    #[test]
    fn test_wrap_1_narrowing_the_width_adds_lines() {
        let body = "une carte reflue son texte quand sa largeur change";
        let wide = pieces(body, 600.0).len();
        let narrow = pieces(body, 150.0).len();
        assert_eq!(wide, 1);
        assert!(narrow > wide, "{narrow} lignes à 150 contre {wide} à 600");
        // Rien ne dépasse : chaque ligne tient dans 15 caractères.
        for piece in pieces(body, 150.0) {
            assert!(piece.chars().count() <= 15, "« {piece} » déborde");
        }
    }

    #[test]
    fn test_wrap_1_a_word_wider_than_the_line_is_cut_instead_of_overflowing() {
        assert_eq!(pieces("abcdefghij", 40.0), vec!["abcd", "efgh", "ij"]);
        assert_eq!(pieces("ab cdefghij", 40.0), vec!["ab", "cdef", "ghij"]);
    }

    #[test]
    fn test_wrap_1_ranges_index_the_source_bytes_even_with_accents() {
        let body = "été chaud";
        let ranges = wrap_paragraph(body, 50.0, mono);
        assert_eq!(ranges, vec![(0, 5), (6, 11)]);
        assert_eq!(&body[ranges[0].0..ranges[0].1], "été");
    }

    #[test]
    fn test_wrap_1_the_overflowing_space_becomes_the_cut() {
        // « abcd » remplit la ligne ; l'espace qui suit déborde et sert de coupe.
        assert_eq!(pieces("abcd efgh", 40.0), vec!["abcd", "efgh"]);
    }
}
