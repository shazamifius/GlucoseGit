//! Le texte riche : ce qu'un fragment de Markdown **veut dire**, avant tout dessin.
//!
//! # Pourquoi ce module est dans le noyau
//!
//! Jusqu'ici, tout ce que Glucose savait du Markdown vivait dans le rendu : `LineKind`
//! reconnaissait `# `, `## ` et `- ` au moment de dessiner, et `export::strip_inline_markdown`
//! effaçait `*`, `_` et `` ` `` avec un `replace` qui mutilait `2 * 3` autant que `*terme*`.
//! Deux lectures du même texte, aucune des deux exacte, et aucune testable sans écran.
//!
//! Ici, le Markdown est **analysé une fois**, en fonctions pures, et ce que les autres
//! couches en reçoivent est un découpage : des tranches d'octets de la source, chacune avec
//! son emphase et son rôle. Le rendu choisit des polices, l'export choisit du texte nu, les
//! ancres de texte ([`crate::text_anchors`]) choisissent des bornes — tous lisent le même
//! découpage, et aucun ne réanalyse.
//!
//! # SPAN-1 — aucun octet de la source n'est perdu, aucun n'est compté deux fois
//!
//! [`inline_spans`] rend une partition **exacte et ordonnée** de la source : les tranches se
//! suivent bout à bout, de `0` à `source.len()`, sans trou ni chevauchement. C'est ce qui
//! permet à un curseur, à une sélection et à une ancre de flèche de se placer par simple
//! recherche de l'octet qui les porte, et à l'édition de retrouver la source en concaténant.
//! Le test `test_spans_partition_the_source_exactly` le tient pour toute entrée.
//!
//! # Le rôle d'une tranche décide de ce qu'on en fait, pas le mode d'affichage
//!
//! Les signes du Markdown ne sont pas du texte : `**` commande, il ne se lit pas. Mais ils ne
//! sont pas rien non plus — pendant l'édition, l'auteur doit les voir pour les corriger. Une
//! tranche porte donc [`SpanRole`], et la décision d'afficher ou non les marqueurs appartient
//! à l'appelant : au repos ils disparaissent, en édition ils se montrent en gris. Un seul
//! découpage, deux vues, et les octets restent à leur place dans les deux.

pub mod block;
pub mod inline;
pub mod selection;

pub use block::{blocks, Block, BlockKind};
pub use inline::inline_spans;
pub use selection::{Direction, Motion, Selection};

/// Les emphases d'un fragment de texte, cumulables.
///
/// Un jeu de drapeaux plutôt qu'une énumération : le Markdown les combine (`***gras
/// italique***`, `**`code` gras**`), et une énumération obligerait à nommer chaque
/// combinaison. Huit bits suffisent et le type reste `Copy`.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Emphasis(u8);

impl Emphasis {
    /// Le corps de texte, sans aucune emphase.
    pub const NONE: Self = Self(0);
    /// `**gras**` ou `__gras__`.
    pub const BOLD: Self = Self(1 << 0);
    /// `*italique*` ou `_italique_`.
    pub const ITALIC: Self = Self(1 << 1);
    /// `~~barré~~`.
    pub const STRIKE: Self = Self(1 << 2);
    /// `` `code` `` — chasse fixe, fond propre, et **aucun autre signe n'y est interprété**.
    pub const CODE: Self = Self(1 << 3);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bold(self) -> bool {
        self.contains(Self::BOLD)
    }

    pub const fn italic(self) -> bool {
        self.contains(Self::ITALIC)
    }

    pub const fn strike(self) -> bool {
        self.contains(Self::STRIKE)
    }

    pub const fn code(self) -> bool {
        self.contains(Self::CODE)
    }
}

impl std::ops::BitOr for Emphasis {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for Emphasis {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

/// `Emphasis(9)` ne dit rien ; `BOLD|CODE` se lit. Un échec de test doit nommer ce qui
/// diffère, pas le laisser décoder.
impl std::fmt::Debug for Emphasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("NONE");
        }
        let mut first = true;
        for (flag, nom) in [
            (Self::BOLD, "BOLD"),
            (Self::ITALIC, "ITALIC"),
            (Self::STRIKE, "STRIKE"),
            (Self::CODE, "CODE"),
        ] {
            if self.contains(flag) {
                if !first {
                    f.write_str("|")?;
                }
                f.write_str(nom)?;
                first = false;
            }
        }
        Ok(())
    }
}

/// Ce qu'une tranche est pour le lecteur : le texte qu'il lit, ou le signe qui le commande.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpanRole {
    /// Ce que l'auteur a écrit pour être lu.
    Text,
    /// Un signe de Markdown — `**`, `` ` ``, le `\` d'un échappement. Il s'efface au repos et
    /// se montre en gris pendant l'édition.
    Marker,
}

/// Une tranche `[start, end)` de la source, et ce qu'elle est.
///
/// Les tranches d'un même texte forment une partition exacte (SPAN-1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub emphasis: Emphasis,
    pub role: SpanRole,
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// La tranche de `source` qu'elle désigne.
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

/// Le texte nu d'une source Markdown : ce que le lecteur lit, sans les signes.
///
/// Remplace `export::strip_inline_markdown`, qui effaçait tous les `*`, `_` et `` ` ``
/// de la chaîne — y compris ceux qui n'ouvraient rien. `2 * 3 * 4` y perdait ses
/// multiplications, `snake_case` son tiret bas, et `a`b` son accent grave.
pub fn plain_text(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for span in inline_spans(source) {
        if span.role == SpanRole::Text {
            out.push_str(span.slice(source));
        }
    }
    out
}
