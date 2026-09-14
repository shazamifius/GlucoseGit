//! L'analyse du Markdown **en ligne** : gras, italique, barré, code, et l'échappement.
//!
//! # Deux passes, parce qu'un délimiteur ne sait pas encore s'il en est un
//!
//! `*` au milieu d'une phrase peut ouvrir une italique, la fermer, ou n'être qu'une
//! multiplication. La réponse n'arrive qu'après — un `*` qui n'est jamais fermé reste une
//! étoile. Émettre le style au fil de la lecture obligerait donc à revenir en arrière et à
//! corriger ce qui a déjà été écrit.
//!
//! D'où deux passes :
//!
//! 1. **l'appariement** ([`pair_up`]) ne produit que des **paires** de délimiteurs, par une
//!    pile ; ce qui n'a pas trouvé son pendant n'existe pas ;
//! 2. **le découpage** ([`inline_spans`]) parcourt les frontières de ces paires et émet la
//!    partition, chaque tranche portant l'union des emphases qui la couvrent.
//!
//! Le résultat est une partition exacte de la source (SPAN-1, voir [`super`]).
//!
//! # Ce qui décide qu'un délimiteur ouvre ou ferme
//!
//! La règle est celle de CommonMark, restreinte à ce qu'un lecteur attend :
//!
//! * un délimiteur **ouvre** si ce qui le suit n'est pas une espace : `*mot` peut ouvrir,
//!   `* ` non ;
//! * il **ferme** si ce qui le précède n'est pas une espace : `mot*` peut fermer ;
//! * `_` ne fait ni l'un ni l'autre **à l'intérieur d'un mot**, sans quoi `snake_case_name`
//!   deviendrait du texte en italique.
//!
//! C'est ce qui laisse `2 * 3 * 4` intact : chacune de ses étoiles est entourée d'espaces,
//! donc aucune n'ouvre ni ne ferme.
//!
//! # Ce que cette analyse ne fait pas, et l'assume
//!
//! Une ouverture et sa fermeture doivent avoir **la même longueur** : `***a***` donne bien
//! gras + italique, mais `***a** b*` — que CommonMark accepte — ne donne rien. Le cas est
//! rare, l'exiger rend le résultat prévisible à la lecture, et la règle tient en une ligne.

use super::{Emphasis, Span, SpanRole};

/// Les caractères qu'un `\` peut désamorcer. Hors de cette liste, le `\` est un `\`.
const ESCAPABLE: &[u8] = b"\\*_~`#->[]$";

/// Une paire de délimiteurs appariés, et l'emphase qu'elle commande entre les deux.
#[derive(Clone, Copy, Debug)]
struct Pair {
    /// La tranche du délimiteur ouvrant.
    open: (usize, usize),
    /// La tranche du délimiteur fermant.
    close: (usize, usize),
    emphasis: Emphasis,
}

/// Un délimiteur ouvert, en attente de son pendant.
#[derive(Clone, Copy)]
struct Open {
    start: usize,
    len: usize,
    ch: u8,
}

/// Le caractère qui précède l'octet `at`, qui doit être une frontière de caractère.
fn char_before(source: &str, at: usize) -> Option<char> {
    source[..at].chars().next_back()
}

/// Le caractère qui commence à l'octet `at`, qui doit être une frontière de caractère.
fn char_at(source: &str, at: usize) -> Option<char> {
    source[at..].chars().next()
}

/// La longueur de la suite de `ch` qui commence à `at`.
fn run_len(bytes: &[u8], at: usize, ch: u8) -> usize {
    bytes[at..].iter().take_while(|&&b| b == ch).count()
}

/// L'emphase qu'une suite de `len` fois `ch` commande, si elle en commande une.
fn emphasis_of(ch: u8, len: usize) -> Option<Emphasis> {
    match (ch, len) {
        (b'~', 2) => Some(Emphasis::STRIKE),
        (b'*' | b'_', 1) => Some(Emphasis::ITALIC),
        (b'*' | b'_', 2) => Some(Emphasis::BOLD),
        (b'*' | b'_', 3) => Some(Emphasis::BOLD.union(Emphasis::ITALIC)),
        _ => None,
    }
}

/// Ce qu'une suite de délimiteurs peut faire, d'après ce qui l'entoure.
fn flanking(source: &str, start: usize, len: usize, ch: u8) -> (bool, bool) {
    let before = char_before(source, start);
    let after = char_at(source, start + len);
    let mut opens = after.is_some_and(|c| !c.is_whitespace());
    let mut closes = before.is_some_and(|c| !c.is_whitespace());
    if ch == b'_' {
        // `snake_case` n'est pas de l'italique : un tiret bas collé à des lettres des deux
        // côtés n'est un délimiteur ni d'un côté ni de l'autre.
        opens &= before.is_none_or(|c| !c.is_alphanumeric());
        closes &= after.is_none_or(|c| !c.is_alphanumeric());
    }
    (opens, closes)
}

/// La fin de la suite de `n` accents graves qui ferme celle ouverte à `from`, s'il y en a une.
///
/// Une suite de trois accents graves ne se ferme que par trois : c'est ce qui permet
/// d'écrire `` ``a`b`` `` pour montrer un accent grave dans du code.
fn closing_backticks(bytes: &[u8], from: usize, n: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run = run_len(bytes, i, b'`');
        if run == n {
            return Some(i);
        }
        i += run;
    }
    None
}

/// Les bornes d'un `[texte](url)` qui commence au `[` en `at`.
///
/// Rend la tranche du `]`…`)` de fermeture. Un lien est une **paire** comme une autre : son
/// ouverture est le `[`, sa fermeture est tout le reste — crochet, parenthèses et adresse.
/// L'adresse est donc un signe, et disparaît au repos exactement comme les `**` d'un gras,
/// sans une seule ligne de traitement particulier.
///
/// Les crochets s'imbriquent (`[a [b] c](url)`), les parenthèses aussi (`[a](b(c)d)` — une
/// adresse en porte parfois) : les deux se comptent plutôt que de s'arrêter au premier
/// fermant venu.
fn link_close(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut profondeur = 1usize;
    let mut i = at + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'[' => profondeur += 1,
            b']' => {
                profondeur -= 1;
                if profondeur == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if profondeur != 0 || bytes.get(i + 1) != Some(&b'(') {
        return None;
    }
    let crochet = i;
    let mut parens = 1usize;
    i += 2;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'(' => parens += 1,
            b')' => {
                parens -= 1;
                if parens == 0 {
                    return Some((crochet, i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Passe 1 — les paires de délimiteurs, et les `\` qui en désamorcent un.
fn pair_up(source: &str) -> (Vec<Pair>, Vec<usize>) {
    let bytes = source.as_bytes();
    let mut pairs = Vec::new();
    let mut escapes = Vec::new();
    let mut open: Vec<Open> = Vec::new();
    // Les tranches à ne pas lire : l'adresse d'un lien, où un `*` n'ouvre rien.
    let mut sautees: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if let Some(&(_, fin)) = sautees.iter().find(|(d, _)| *d == i) {
            i = fin;
            continue;
        }
        match bytes[i] {
            b'\\' if bytes.get(i + 1).is_some_and(|c| ESCAPABLE.contains(c)) => {
                escapes.push(i);
                // Le caractère désamorcé est sauté : il ne peut plus rien ouvrir.
                i += 2;
            }
            b'`' => {
                let n = run_len(bytes, i, b'`');
                match closing_backticks(bytes, i + n, n) {
                    Some(j) => {
                        // Tout ce qui est entre les deux est du code : aucun autre signe n'y
                        // est lu, c'est le sens même de `` ` ``.
                        pairs.push(Pair {
                            open: (i, i + n),
                            close: (j, j + n),
                            emphasis: Emphasis::CODE,
                        });
                        i = j + n;
                    }
                    None => i += n,
                }
            }
            b'[' => {
                match link_close(bytes, i) {
                    Some((crochet, fin)) => {
                        pairs.push(Pair {
                            open: (i, i + 1),
                            close: (crochet, fin),
                            emphasis: Emphasis::LINK,
                        });
                        // L'intérieur reste analysé — un lien peut porter du gras — mais son
                        // adresse, non : `[a](b*c*d)` n'a pas d'italique dans son URL.
                        sautees.push((crochet, fin));
                        i += 1;
                    }
                    None => i += 1,
                }
            }
            ch @ (b'*' | b'_' | b'~') => {
                i += emphasis_run(source, i, ch, &mut open, &mut pairs);
            }
            _ => i += 1,
        }
    }
    (pairs, escapes)
}

/// Une suite de `*`, `_` ou `~` : elle ferme une paire ouverte, en ouvre une, ou n'est que
/// ce qu'elle paraît. Rend le nombre d'octets consommés.
fn emphasis_run(
    source: &str,
    at: usize,
    ch: u8,
    open: &mut Vec<Open>,
    pairs: &mut Vec<Pair>,
) -> usize {
    let n = run_len(source.as_bytes(), at, ch);
    let Some(emphasis) = emphasis_of(ch, n) else {
        return n;
    };
    let (opens, closes) = flanking(source, at, n, ch);
    // Une fermeture d'abord : `*a*` referme avant d'ouvrir quoi que ce soit.
    let matched = closes
        .then(|| open.iter().rposition(|d| d.ch == ch && d.len == n))
        .flatten();
    if let Some(index) = matched {
        let d = open[index];
        // Ce qui s'est ouvert à l'intérieur et n'a pas fermé est abandonné : dans
        // `**a *b**`, l'étoile solitaire redevient une étoile.
        open.truncate(index);
        pairs.push(Pair {
            open: (d.start, d.start + d.len),
            close: (at, at + n),
            emphasis,
        });
    } else if opens {
        open.push(Open {
            start: at,
            len: n,
            ch,
        });
    }
    n
}

/// Le découpage inline de `source` : une partition exacte, chaque tranche avec son emphase
/// et son rôle (SPAN-1).
///
/// Les tranches vides ne sont pas émises ; celles qui restent s'enchaînent bout à bout et
/// couvrent toute la source.
pub fn inline_spans(source: &str) -> Vec<Span> {
    let (mut pairs, escapes) = pair_up(source);
    if pairs.is_empty() && escapes.is_empty() {
        return if source.is_empty() {
            Vec::new()
        } else {
            vec![Span {
                start: 0,
                end: source.len(),
                emphasis: Emphasis::NONE,
                role: SpanRole::Text,
            }]
        };
    }

    // Les coupes : les quatre frontières de chaque paire, les deux de chaque échappement.
    let mut cuts = Vec::with_capacity(pairs.len() * 4 + escapes.len() * 2 + 2);
    cuts.push(0);
    cuts.push(source.len());
    for p in &pairs {
        cuts.extend([p.open.0, p.open.1, p.close.0, p.close.1]);
    }
    for &e in &escapes {
        cuts.extend([e, e + 1]);
    }
    cuts.sort_unstable();
    cuts.dedup();

    // Les tranches qui sont des signes, dans l'ordre : les deux délimiteurs de chaque paire
    // et chaque `\` désamorçant. Elles sont disjointes, donc un index qui avance suffit à
    // les reconnaître pendant le parcours des coupes.
    let mut markers: Vec<usize> = escapes;
    markers.extend(pairs.iter().flat_map(|p| [p.open.0, p.close.0]));
    markers.sort_unstable();
    let mut next_marker = 0;

    // Les paires sont bien imbriquées — la pile de la passe 1 le garantit — donc les
    // parcourir dans l'ordre d'ouverture suffit à tenir l'emphase courante à jour.
    pairs.sort_unstable_by_key(|p| p.open.0);
    let mut active: Vec<Pair> = Vec::new();
    let mut next = 0;

    let mut spans = Vec::with_capacity(cuts.len());
    for window in cuts.windows(2) {
        let (start, end) = (window[0], window[1]);
        while next < pairs.len() && pairs[next].open.1 <= start {
            active.push(pairs[next]);
            next += 1;
        }
        active.retain(|p| p.close.0 > start);

        // Un délimiteur porte l'emphase de ce qui l'entoure, jamais la sienne : il est hors
        // du contenu de sa propre paire, donc `active` ne le couvre pas. C'est ce qui fait
        // qu'un `**` grisé pendant l'édition n'est pas lui-même en gras.
        let emphasis = active
            .iter()
            .fold(Emphasis::NONE, |acc, p| acc | p.emphasis);
        while next_marker < markers.len() && markers[next_marker] < start {
            next_marker += 1;
        }
        let is_marker = markers.get(next_marker) == Some(&start);
        spans.push(Span {
            start,
            end,
            emphasis,
            role: if is_marker {
                SpanRole::Marker
            } else {
                SpanRole::Text
            },
        });
    }
    spans
}

#[cfg(test)]
mod tests;
