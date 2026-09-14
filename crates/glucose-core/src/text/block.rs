//! Ce qu'un paragraphe **est**, avant de savoir avec quoi le dessiner.
//!
//! [`super::inline`] dit ce que les signes disent **à l'intérieur** d'une ligne : gras,
//! italique, code. Il manquait le symétrique — ce que les premiers octets d'une ligne disent
//! de la ligne entière : un titre, une puce, une citation, une formule. Cette lecture-là
//! vivait dans le rendu, où elle ne pouvait être ni testée sans écran, ni relue par l'export,
//! ni consultée par les ancres de texte.
//!
//! Elle est ici, en fonctions pures, et le rendu ne garde que ce qui dépend vraiment d'une
//! fonte : la taille d'un titre, l'encre d'une citation, le retrait d'une puce.
//!
//! # BLOCK-1 — les blocs partitionnent la source, et chaque octet sait à quoi il sert
//!
//! [`blocks`] rend un bloc par paragraphe, dans l'ordre, chacun couvrant sa ligne entière :
//! le premier commence à `0`, chacun reprend un octet après la fin du précédent — le `\n`
//! qui les sépare —, et le dernier finit à `source.len()`. Aucun paragraphe n'est sauté,
//! aucun n'est rendu deux fois.
//!
//! À l'intérieur, un bloc se lit en trois morceaux : un **préfixe** (`# `, `> `, `1. `), un
//! **corps**, un **suffixe** (le `$$` de fermeture d'une formule). Préfixe et suffixe sont
//! des signes au même titre que `**` : ils s'effacent au repos et reviennent à l'édition
//! (MODE-1), sans que personne ait à les nommer un par un. Le corps, lui, est ce que le
//! lecteur lit — et c'est lui, et lui seul, que l'analyse en ligne parcourt.
//!
//! # Un bloc de code se souvient de la ligne d'avant
//!
//! C'est le seul genre qui ne se décide pas ligne par ligne : entre deux clôtures ```` ``` ````
//! tout est du code, y compris ce qui ressemble à un titre. L'analyse porte donc un état —
//! d'où un itérateur plutôt qu'une fonction sur une ligne isolée, et zéro allocation pour un
//! travail refait à chaque image.

use std::ops::Range;

/// Les trois accents graves qui ouvrent et ferment un bloc de code.
const FENCE: &str = "```";
/// Le nombre de tirets à partir duquel une ligne devient un trait.
const RULE_MIN: usize = 3;
/// Le dernier niveau de titre du Markdown : `######`.
const HEADING_MAX: usize = 6;

/// Le genre d'un paragraphe : ce que ses premiers octets commandent à toute la ligne.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    /// `# ` à `###### ` — un titre, et son niveau de `1` à [`HEADING_MAX`].
    ///
    /// Un niveau plutôt que six variantes : le Markdown en compte six, le rendu n'en
    /// distingue que quelques-uns, et un entier laisse cette décision au rendu sans que le
    /// noyau ait à la connaître.
    Heading(u8),
    /// `- ` ou `* ` — un élément de liste à puce.
    Bullet,
    /// `1. ` — un élément numéroté. Le numéro est **celui que l'auteur a écrit** : Glucose
    /// montre le texte tel qu'il est, il ne renumérote pas derrière lui.
    Ordered(u32),
    /// `> ` — une citation.
    Quote,
    /// `-# ` — du petit texte. Syntaxe maison : le Markdown n'a rien pour ça, et une note de
    /// bas de carte est un besoin courant sur un canva.
    Small,
    /// `---` seul sur sa ligne — un trait de séparation, pas du texte.
    Rule,
    /// Une ligne ```` ``` ```` : elle ouvre ou ferme un bloc de code, et ne se lit pas.
    Fence,
    /// Une ligne **dans** un bloc de code : ses octets se lisent tels quels.
    Code,
    /// Un paragraphe entièrement occupé par une formule, `$…$` ou `$$…$$`.
    ///
    /// Le noyau ne connaît pas `glucose_math` — il dit seulement si la formule demande le mode
    /// bloc, et la crate des formules traduit ce booléen en son propre mode.
    Math { display: bool },
    /// Tout le reste.
    Body,
}

impl BlockKind {
    /// Un bloc littéral ne contient pas de Markdown : `**` y est deux astérisques.
    pub const fn literal(self) -> bool {
        matches!(self, Self::Code | Self::Math { .. })
    }

    /// Un bloc muet n'a aucun texte à donner au moteur de texte : sa ligne se dessine
    /// autrement, ou pas du tout.
    pub const fn silent(self) -> bool {
        matches!(self, Self::Rule | Self::Fence)
    }
}

/// Un paragraphe de la source, et ce qu'il est.
///
/// `Copy` et sans allocation : il en naît un par paragraphe et par carte visible, à chaque
/// image.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Block {
    /// Premier octet du paragraphe dans la source.
    pub start: usize,
    /// Octet qui suit le paragraphe — le `\n`, ou la fin de la source.
    pub end: usize,
    /// Longueur en octets des signes de tête (`# `, `> `, `1. `, le `$$` d'ouverture).
    pub prefix: usize,
    /// Longueur en octets des signes de queue (le `$$` de fermeture).
    pub suffix: usize,
    pub kind: BlockKind,
}

impl Block {
    /// Le paragraphe entier, signes compris.
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Ce que le lecteur lit : le paragraphe sans ses signes de tête ni de queue.
    pub fn body(&self) -> Range<usize> {
        self.start + self.prefix..self.end - self.suffix
    }

    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.range()]
    }

    pub fn body_slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.body()]
    }
}

/// Les paragraphes de `source`, dans l'ordre (BLOCK-1).
pub fn blocks(source: &str) -> Blocks<'_> {
    Blocks {
        source,
        at: 0,
        done: false,
        fenced: false,
    }
}

/// L'itérateur de [`blocks`]. Il porte l'état de clôture, que rien d'autre n'a à connaître.
pub struct Blocks<'a> {
    source: &'a str,
    at: usize,
    done: bool,
    fenced: bool,
}

impl Iterator for Blocks<'_> {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        if self.done {
            return None;
        }
        let rest = &self.source[self.at..];
        let len = match rest.find('\n') {
            Some(i) => i,
            None => {
                // La dernière ligne n'a pas de `\n` derrière elle ; une source vide en est une.
                self.done = true;
                rest.len()
            }
        };
        let (kind, prefix, suffix) = classify(&rest[..len], self.fenced);
        if kind == BlockKind::Fence {
            self.fenced = !self.fenced;
        }
        let block = Block {
            start: self.at,
            end: self.at + len,
            prefix,
            suffix,
            kind,
        };
        self.at += len + 1;
        Some(block)
    }
}

/// Le genre d'une ligne, la longueur de ses signes de tête et celle de ses signes de queue.
///
/// L'ordre des essais est celui qui lève les ambiguïtés : `---` avant `-# ` avant `- `.
/// Chacune de ces paires partage un premier octet, aucune ne partage le second.
fn classify(line: &str, fenced: bool) -> (BlockKind, usize, usize) {
    // Une clôture se reconnaît des deux côtés — sans quoi un bloc de code ne se fermerait
    // jamais — et son éventuel nom de langage est de la métadonnée, pas du texte.
    if line.starts_with(FENCE) {
        return (BlockKind::Fence, line.len(), 0);
    }
    if fenced {
        return (BlockKind::Code, 0, 0);
    }
    if let Some((body, display)) = formula(line) {
        return (
            BlockKind::Math { display },
            body.start,
            line.len() - body.end,
        );
    }
    let bare = line.trim_end();
    if bare.len() >= RULE_MIN && bare.bytes().all(|b| b == b'-') {
        return (BlockKind::Rule, bare.len(), line.len() - bare.len());
    }
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    if (1..=HEADING_MAX).contains(&hashes) && line[hashes..].starts_with(' ') {
        return (BlockKind::Heading(hashes as u8), hashes + 1, 0);
    }
    for (mark, kind) in [
        ("-# ", BlockKind::Small),
        ("- ", BlockKind::Bullet),
        ("* ", BlockKind::Bullet),
        ("> ", BlockKind::Quote),
    ] {
        if line.starts_with(mark) {
            return (kind, mark.len(), 0);
        }
    }
    // Une ligne de citation vide s'écrit `>` tout court ; sans ce cas, un paragraphe cité
    // coupé en deux perdrait sa barre au milieu.
    if line == ">" {
        return (BlockKind::Quote, 1, 0);
    }
    if let Some((n, prefix)) = ordered(line) {
        return (BlockKind::Ordered(n), prefix, 0);
    }
    (BlockKind::Body, 0, 0)
}

/// Le numéro d'un `12. `, et la longueur du signe qui le porte.
///
/// Un numéro qui ne tient pas dans un `u32` n'en est pas un : la ligne redevient du corps,
/// sans borne arbitraire à choisir.
fn ordered(line: &str) -> Option<(u32, usize)> {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || !line[digits..].starts_with(". ") {
        return None;
    }
    let n = line[..digits].parse().ok()?;
    Some((n, digits + ". ".len()))
}

/// Le corps d'un paragraphe qui est **entièrement** une formule, et son mode.
///
/// Le LaTeX **au milieu** d'une phrase n'est pas traité ici : il demande de mesurer des
/// segments de nature différente sur une même ligne. Le cas fréquent sur un canva est la
/// formule posée seule.
pub fn formula(paragraph: &str) -> Option<(Range<usize>, bool)> {
    let start = paragraph.len() - paragraph.trim_start().len();
    let t = paragraph.trim();
    let inner = |marks: usize| start + marks..start + t.len() - marks;

    if let Some(corps) = t.strip_prefix("$$").and_then(|r| r.strip_suffix("$$")) {
        if !corps.trim().is_empty() {
            return Some((inner("$$".len()), true));
        }
    }
    if let Some(corps) = t.strip_prefix('$').and_then(|r| r.strip_suffix('$')) {
        // Un `$` de plus au milieu, et ce sont deux formules — ou un prix en dollars.
        if !corps.trim().is_empty() && !corps.contains('$') {
            return Some((inner("$".len()), false));
        }
    }
    None
}

#[cfg(test)]
mod tests;
