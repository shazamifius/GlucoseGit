//! La sélection de texte : une ancre, une tête, et les mouvements qui la déplacent.
//!
//! # SEL-1 — une ancre et une tête, jamais un couple ordonné
//!
//! Une sélection n'est pas « du début à la fin » : c'est **le point qu'on a posé** et **le
//! point qui bouge**. La distinction n'est pas cosmétique, c'est elle qui fait qu'un
//! `Maj+←` puis un `Maj+→` revient exactement sur ses pas au lieu d'inverser les bornes et
//! de repartir dans l'autre sens.
//!
//! L'ancre se pose au clic ou au premier `Maj` ; la tête suit la souris, les flèches, le
//! `Maj+clic`. Quand la sélection est vide, les deux coïncident : **le curseur est une
//! sélection de longueur nulle**, et il n'y a donc qu'un seul état à porter, pas deux.
//!
//! # SEL-2 — un offset est un octet, toujours sur une frontière de caractère
//!
//! `String` s'indexe en octets et `é` en occupe deux : un offset au milieu d'un caractère
//! fait **paniquer** `insert`, `drain` et le découpage. Chaque fonction de ce module rend un
//! offset valide pour le texte qu'on lui donne, et [`Selection::clamped`] répare une
//! sélection devenue invalide — ce qui arrive dès qu'un texte est modifié ailleurs (annulation,
//! miroir, collaboration).
//!
//! # Ce que « mot » veut dire ici
//!
//! Le découpage en mots de l'Unicode (UAX#29) tient compte des syllabes thaïes, des idéogrammes
//! et des émoji composés. Ce module en retient **trois classes** — mot, espace, ponctuation —
//! qui suffisent à ce qu'un `Ctrl+←` et un double-clic fassent ce qu'on attend d'eux dans une
//! langue à alphabet, et qui se lisent en dix lignes au lieu d'une table de six cents entrées.
//! Le jour où le thaï comptera, ce sera un remplacement local, derrière [`class_of`].

use std::cmp::Ordering;

/// À quelle famille appartient un caractère, pour le découpage en mots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharClass {
    /// Lettre, chiffre, ou tiret bas — ce qu'un double-clic prend d'un seul bloc.
    Word,
    /// Espace, tabulation, saut de ligne.
    Space,
    /// Tout le reste : ponctuation, symboles, opérateurs.
    Punctuation,
}

/// La famille d'un caractère.
///
/// Le tiret bas est du mot : `snake_case` se sélectionne d'un seul double-clic, comme dans un
/// navigateur. L'apostrophe, elle, n'en est pas — `l'entropie` donne donc deux mots, ce qui
/// est la convention de tous les éditeurs et ce à quoi la main s'attend.
pub fn class_of(ch: char) -> CharClass {
    if ch.is_alphanumeric() || ch == '_' {
        CharClass::Word
    } else if ch.is_whitespace() {
        CharClass::Space
    } else {
        CharClass::Punctuation
    }
}

/// Le sens d'un mouvement.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Backward,
    Forward,
}

/// De combien un mouvement déplace la tête.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Motion {
    /// Un caractère — jamais un octet (SEL-2).
    Char,
    /// Un mot : les espaces se franchissent, puis la classe courante.
    Word,
    /// Le bord de la **ligne logique** : jusqu'au `\n` le plus proche, sans le franchir.
    /// Le bord d'une ligne *visuelle*, lui, dépend du reflux, donc de la mise en page.
    LineEdge,
    /// Le bord du texte.
    DocEdge,
}

/// Une sélection de texte, en octets (SEL-1).
///
/// Vide — `anchor == head` — elle est un simple curseur.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Selection {
    /// Le point posé, qui ne bouge pas tant qu'on étend.
    pub anchor: usize,
    /// Le point qui bouge, et où le curseur se dessine.
    pub head: usize,
}

impl Selection {
    /// Un curseur seul, sans étendue.
    pub fn at(offset: usize) -> Self {
        Self {
            anchor: offset,
            head: offset,
        }
    }

    /// Une sélection couvrant `[start, end)`, tête à la fin.
    pub fn spanning(start: usize, end: usize) -> Self {
        Self {
            anchor: start,
            head: end,
        }
    }

    /// Le texte entier.
    pub fn all(text: &str) -> Self {
        Self::spanning(0, text.len())
    }

    /// Les bornes remises dans l'ordre.
    pub fn range(&self) -> (usize, usize) {
        match self.anchor.cmp(&self.head) {
            Ordering::Greater => (self.head, self.anchor),
            _ => (self.anchor, self.head),
        }
    }

    pub fn start(&self) -> usize {
        self.range().0
    }

    pub fn end(&self) -> usize {
        self.range().1
    }

    /// Vraie quand la sélection n'a pas d'étendue : c'est alors un curseur.
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Le texte sélectionné.
    pub fn slice<'a>(&self, text: &'a str) -> &'a str {
        let (start, end) = self.range();
        &text[start..end]
    }

    /// La sélection réduite à un curseur, à l'endroit demandé.
    ///
    /// `Forward` pose le curseur à la fin, `Backward` au début : c'est ce que fait une flèche
    /// sans `Maj` quand du texte est sélectionné — elle ne déplace pas le curseur d'un
    /// caractère, elle **retombe du bon côté** de la sélection.
    pub fn collapsed(self, dir: Direction) -> Self {
        let (start, end) = self.range();
        Self::at(match dir {
            Direction::Backward => start,
            Direction::Forward => end,
        })
    }

    /// La même sélection, ramenée dans `text` et sur des frontières de caractères (SEL-2).
    ///
    /// Indispensable dès qu'un texte change sous une sélection : une annulation, un miroir ou
    /// une édition venue d'ailleurs peut le raccourcir, et un offset resté au-delà ferait
    /// paniquer le prochain découpage.
    pub fn clamped(self, text: &str) -> Self {
        Self {
            anchor: nearest_boundary(text, self.anchor),
            head: nearest_boundary(text, self.head),
        }
    }
}

/// La frontière de caractère la plus proche de `offset`, en descendant.
fn nearest_boundary(text: &str, offset: usize) -> usize {
    let mut at = offset.min(text.len());
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// La frontière de caractère qui précède `offset`, ou `0`.
pub fn prev_char(text: &str, offset: usize) -> usize {
    let at = nearest_boundary(text, offset);
    if at == 0 {
        return 0;
    }
    let mut prev = at - 1;
    while prev > 0 && !text.is_char_boundary(prev) {
        prev -= 1;
    }
    prev
}

/// La frontière de caractère qui suit `offset`, ou la fin du texte.
pub fn next_char(text: &str, offset: usize) -> usize {
    let at = nearest_boundary(text, offset);
    if at >= text.len() {
        return text.len();
    }
    let mut next = at + 1;
    while next < text.len() && !text.is_char_boundary(next) {
        next += 1;
    }
    next
}

/// Le caractère qui précède `offset`, s'il existe.
fn char_before(text: &str, offset: usize) -> Option<char> {
    text[..nearest_boundary(text, offset)].chars().next_back()
}

/// Le caractère qui commence à `offset`, s'il existe.
fn char_at(text: &str, offset: usize) -> Option<char> {
    text[nearest_boundary(text, offset)..].chars().next()
}

/// L'offset atteint depuis `from` par `motion` dans `dir`.
pub fn move_offset(text: &str, from: usize, motion: Motion, dir: Direction) -> usize {
    match (motion, dir) {
        (Motion::Char, Direction::Backward) => prev_char(text, from),
        (Motion::Char, Direction::Forward) => next_char(text, from),
        (Motion::Word, Direction::Backward) => word_backward(text, from),
        (Motion::Word, Direction::Forward) => word_forward(text, from),
        (Motion::LineEdge, Direction::Backward) => line_start(text, from),
        (Motion::LineEdge, Direction::Forward) => line_end(text, from),
        (Motion::DocEdge, Direction::Backward) => 0,
        (Motion::DocEdge, Direction::Forward) => text.len(),
    }
}

/// Le début du mot courant, ou celui du mot précédent si l'on y est déjà.
///
/// Les espaces qui précèdent se franchissent d'abord, puis toute la classe rencontrée : c'est
/// ce qui fait qu'un `Ctrl+←` depuis « fin de phrase. » s'arrête sur « phrase » et non sur le
/// point.
fn word_backward(text: &str, from: usize) -> usize {
    let mut at = nearest_boundary(text, from);
    while at > 0 && char_before(text, at).map(class_of) == Some(CharClass::Space) {
        at = prev_char(text, at);
    }
    let Some(class) = char_before(text, at).map(class_of) else {
        return at;
    };
    while at > 0 && char_before(text, at).map(class_of) == Some(class) {
        at = prev_char(text, at);
    }
    at
}

/// Le début du mot suivant : la classe courante se franchit, puis les espaces.
///
/// C'est la convention des navigateurs et des traitements de texte — `Ctrl+→` pose le curseur
/// **devant** le mot suivant, prêt à écrire, pas à la fin du mot courant.
fn word_forward(text: &str, from: usize) -> usize {
    let mut at = nearest_boundary(text, from);
    if let Some(class) = char_at(text, at).map(class_of) {
        if class != CharClass::Space {
            while at < text.len() && char_at(text, at).map(class_of) == Some(class) {
                at = next_char(text, at);
            }
        }
    }
    // Les espaces horizontales se franchissent, mais **pas** le saut de ligne : sans cet
    // arrêt, `Ctrl+→` traverserait les paragraphes sans jamais se poser sur la ligne vide
    // qui les sépare, et il deviendrait impossible d'y écrire.
    while matches!(char_at(text, at), Some(' ') | Some('\t')) {
        at = next_char(text, at);
    }
    at
}

/// Le début de la ligne logique qui porte `offset`.
pub fn line_start(text: &str, offset: usize) -> usize {
    let at = nearest_boundary(text, offset);
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// La fin de la ligne logique qui porte `offset`, saut de ligne exclu.
pub fn line_end(text: &str, offset: usize) -> usize {
    let at = nearest_boundary(text, offset);
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// Les bornes du mot qui porte `offset` — ce qu'un double-clic sélectionne.
///
/// Sur une espace, c'est la suite d'espaces qui est prise ; sur une ponctuation, la suite de
/// ponctuations. Un double-clic rend donc toujours quelque chose, jamais une sélection vide,
/// et ce quelque chose est ce que le doigt a désigné.
pub fn word_at(text: &str, offset: usize) -> (usize, usize) {
    let at = nearest_boundary(text, offset);
    // Au bord droit d'un mot, c'est le mot qu'on vient de quitter que l'on veut — sinon un
    // double-clic juste après le dernier caractère ne prendrait rien.
    let class = char_at(text, at)
        .map(class_of)
        .or_else(|| char_before(text, at).map(class_of));
    let Some(class) = class else {
        return (at, at);
    };
    let mut start = at;
    while start > 0 && char_before(text, start).map(class_of) == Some(class) {
        start = prev_char(text, start);
    }
    let mut end = at;
    while end < text.len() && char_at(text, end).map(class_of) == Some(class) {
        end = next_char(text, end);
    }
    (start, end)
}

/// Les bornes du paragraphe qui porte `offset` — ce qu'un triple-clic sélectionne.
pub fn paragraph_at(text: &str, offset: usize) -> (usize, usize) {
    (line_start(text, offset), line_end(text, offset))
}

/// Remplace la plage sélectionnée par `insert`, et rend le curseur qui en résulte.
///
/// C'est **l'unique** porte d'écriture d'une saisie : taper, coller, effacer et remplacer
/// passent tous par là. Écrire sur le texte à côté, c'est se charger soi-même de replacer la
/// sélection — et c'est ainsi qu'un curseur finit au milieu d'un « é ».
pub fn replace(text: &mut String, selection: Selection, insert: &str) -> Selection {
    let (start, end) = selection.clamped(text).range();
    text.replace_range(start..end, insert);
    Selection::at(start + insert.len())
}

/// Efface la sélection ; si elle est vide, efface d'abord d'un mouvement dans `dir`.
///
/// Rend le curseur résultant. C'est le comportement qu'on attend d'une touche `Retour arrière`
/// ou `Suppr` : elle mange la sélection quand il y en a une, et un caractère sinon.
pub fn delete(
    text: &mut String,
    selection: Selection,
    motion: Motion,
    dir: Direction,
) -> Selection {
    let selection = selection.clamped(text);
    if selection.is_empty() {
        let to = move_offset(text, selection.head, motion, dir);
        return replace(text, Selection::spanning(selection.head, to), "");
    }
    replace(text, selection, "")
}

/// La maille d'un geste de souris : ce qu'un clic, un double-clic et un triple-clic prennent.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Granularity {
    /// Un clic : le curseur se pose entre deux caractères.
    #[default]
    Char,
    /// Un double-clic : le mot entier.
    Word,
    /// Un triple-clic : le paragraphe entier.
    Paragraph,
}

/// La sélection que produit un geste de `granularity` allant de `anchor` à `head`.
///
/// C'est **la même fonction** pour le clic posé et pour le glisser qui suit : c'est elle qui
/// fait qu'un glisser entamé par un double-clic continue de prendre des mots entiers, et qu'il
/// n'en laisse jamais un à moitié derrière lui quand on revient en arrière.
pub fn expand(text: &str, anchor: usize, head: usize, granularity: Granularity) -> Selection {
    let bornes = |at: usize| match granularity {
        Granularity::Char => (at, at),
        Granularity::Word => word_at(text, at),
        Granularity::Paragraph => paragraph_at(text, at),
    };
    let (debut_ancre, fin_ancre) = bornes(anchor);
    let (debut_tete, fin_tete) = bornes(head);
    if head >= anchor {
        Selection {
            anchor: debut_ancre,
            head: fin_tete,
        }
    } else {
        Selection {
            anchor: fin_ancre,
            head: debut_tete,
        }
    }
    .clamped(text)
}

#[cfg(test)]
mod tests;
