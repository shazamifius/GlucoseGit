//! La mise en page d'un texte riche : du Markdown à des fragments posés sur des lignes.
//!
//! # Ce que ce module ajoute à [`glucose_core::text`]
//!
//! Le noyau dit **ce que le Markdown veut dire** — des tranches, chacune avec son emphase —
//! sans rien savoir des polices. Ici s'ajoute tout ce qui dépend d'une fonte : le visage que
//! chaque emphase demande, la largeur que chaque caractère occupe, et l'endroit où une ligne
//! doit se couper pour tenir dans la boîte.
//!
//! Le résultat, [`TextLayout`], est ce que le dessin consomme et ce que le clic interroge.
//! Il est calculé **en unités monde, avant toute mise à l'échelle** (CARD-1) : la même carte
//! se découpe en les mêmes lignes à tous les zooms.
//!
//! # RICH-1 — une ligne n'a plus un style, elle a des fragments
//!
//! Avant, une ligne visuelle était une tranche d'octets et un style uniforme : un titre, une
//! puce ou du corps, dessiné d'un seul `draw_text`. Le gras au milieu d'une phrase rend cela
//! impossible — la largeur d'une ligne cesse d'être la somme des avances d'une seule police.
//!
//! Une ligne porte donc une plage de [`Fragment`], chacun homogène : un visage, une emphase,
//! une tranche. La mesure, le reflux, le tracé et la position du curseur parcourent tous la
//! même plage, dans le même ordre. C'est ce qui garantit que le curseur tombe **là où le
//! texte est dessiné**, y compris au milieu d'un mot en italique suivi de code.
//!
//! # MODE-1 — les signes du Markdown s'effacent au repos et reviennent à l'édition
//!
//! `**gras**` se lit « gras » quand on regarde la carte, et `**gras**` quand on la corrige.
//! Ces deux vues sont **la même mise en page**, à une fonction de largeur près : en mode
//! [`TextMode::Rendered`], un signe a une avance nulle et n'est pas tracé ; en mode
//! [`TextMode::Source`], il a sa vraie largeur et se dessine en gris.
//!
//! Un seul chemin de code, donc un seul endroit où un décalage peut naître — et les octets
//! de la source restent à leur place dans les deux vues, ce dont le curseur dépend
//! entièrement.
//!
//! Corollaire longtemps manqué : le préfixe d'un bloc (`# `, `- `) est un signe comme un
//! autre. Il disparaissait **aussi pendant l'édition**, si bien qu'on éditait `# Titre` en
//! voyant `Titre` — et qu'écrire au tout début du texte insérait avant un `#` invisible.

pub mod draw;
pub mod hit;

use super::math::MathRenderer;
use super::wrap::wrap_paragraph;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::text::{blocks, inline_spans, Block, BlockKind, Emphasis, Span, SpanRole};

/// Interligne, en multiples du corps (fiche 06 § 5.1 : `lineHeight: 1.4`).
pub const LINE_FACTOR: f32 = 1.4;
/// Grossissement d'un titre `# `.
const H1_FACTOR: f32 = 1.25;
/// Grossissement d'un sous-titre `## `.
const H2_FACTOR: f32 = 1.10;
/// Rapetissement d'un `-# `.
///
/// Il n'a pas de valeur à lui : le petit texte est au titre ce que le corps est au titre,
/// donc l'inverse du grossissement d'un `# `. Une constante de moins à choisir, et la
/// symétrie tient toute seule si le facteur des titres change un jour.
const SMALL_FACTOR: f32 = 1.0 / H1_FACTOR;
// Les titres sont plus grands que le corps, et un titre plus qu'un sous-titre : vérifié à la
// compilation, pas dans un test qu'on pourrait oublier de lancer.
const _: () = assert!(H1_FACTOR > H2_FACTOR && H2_FACTOR > 1.0);

/// Ce que l'affichage fait des signes du Markdown (MODE-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMode {
    /// Au repos : les signes s'effacent, le texte porte le style qu'ils commandent.
    Rendered,
    /// Pendant l'édition : les signes restent visibles, en gris, pour être corrigés.
    Source,
}

/// Le visage qu'une emphase demande.
///
/// Le code passe avant tout le reste : `**`gras`**` se dessine en chasse fixe, parce que
/// c'est du code — la graisse n'a pas de second fichier à lui offrir, et une chasse fixe
/// grasse dirait « code gras » là où l'auteur a écrit « du code, dans une phrase en gras ».
impl From<Emphasis> for Face {
    fn from(e: Emphasis) -> Self {
        match (e.code(), e.bold(), e.italic()) {
            (true, _, _) => Self::Mono,
            (false, false, false) => Self::Regular,
            (false, true, false) => Self::Bold,
            (false, false, true) => Self::Italic,
            (false, true, true) => Self::BoldItalic,
        }
    }
}

// ── Du genre d'un bloc aux pixels ─────────────────────────────────────────────
// Le genre est décidé par `glucose_core::text::block`, qui ne connaît ni police ni couleur.
// Les quatre fonctions qui suivent sont la seule frontière entre ce qu'un bloc *est* et ce
// qu'il *devient* à l'écran — taille, graisse, encre, retrait. Rien d'autre n'a le droit de
// regarder un `BlockKind` pour en tirer une valeur.

/// Le corps d'un bloc. Les grossissements sont des **multiples du corps courant** : ils
/// héritent de l'unique transformation au lieu de redériver du zoom (CARD-1).
///
/// Au-delà du sous-titre, les niveaux gardent le corps et se distinguent par la graisse. La
/// référence ne donne de valeur que pour `#` et `##` ; en inventer quatre autres serait
/// affirmer des chiffres que personne n'a mesurés.
pub fn font_of(kind: BlockKind, body: f32) -> f32 {
    match kind {
        BlockKind::Heading(1) => body * H1_FACTOR,
        BlockKind::Heading(2) => body * H2_FACTOR,
        BlockKind::Small => body * SMALL_FACTOR,
        _ => body,
    }
}

/// L'emphase que le genre impose à toute la ligne.
///
/// Elle s'**unit** à celle des fragments, ce qui fait qu'un `*mot*` dans un titre est en gras
/// italique sans cas particulier — et qu'une ligne de bloc de code passe en chasse fixe par
/// le même chemin qu'un `` `code` `` en ligne, avec son fond et tout le reste.
pub fn emphasis_of(kind: BlockKind) -> Emphasis {
    match kind {
        BlockKind::Heading(_) => Emphasis::BOLD,
        BlockKind::Code => Emphasis::CODE,
        _ => Emphasis::NONE,
    }
}

/// L'encre d'une ligne.
///
/// Une citation garde celle du corps : ce qui la distingue est sa barre et son retrait, pas
/// une couleur de plus. La barre, elle, est grise — elle **est** le `>` que le repos efface,
/// donc elle prend l'encre des signes.
pub fn ink_of(kind: BlockKind, theme: &Theme) -> tiny_skia::Color {
    match kind {
        BlockKind::Heading(1) => theme.card_heading,
        BlockKind::Heading(_) => theme.card_subheading,
        _ => theme.card_body,
    }
}

/// Le retrait d'une ligne, en multiples de `unit`.
///
/// Une seule règle, un seul endroit : la mise en page l'applique pour savoir où couper, le
/// tracé pour savoir où poser la plume, le clic pour savoir où viser. Elle vivait en deux
/// exemplaires, et les deux devaient rester d'accord à la main.
pub fn indent_of(kind: BlockKind, unit: f32) -> f32 {
    match kind {
        BlockKind::Bullet | BlockKind::Ordered(_) | BlockKind::Quote => unit,
        _ => 0.0,
    }
}

/// Un morceau de ligne dessiné d'un seul trait : même visage, même emphase, même rôle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fragment {
    /// Tranche `[start, end)` de la **source**, jamais d'un texte reconstruit.
    pub start: usize,
    pub end: usize,
    pub emphasis: Emphasis,
    pub role: SpanRole,
}

impl Fragment {
    pub fn face(&self) -> Face {
        Face::from(self.emphasis)
    }
}

/// Une ligne visuelle : sa tranche de source, son genre, et ses fragments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualLine {
    pub start: usize,
    pub end: usize,
    pub kind: BlockKind,
    /// Première ligne de son paragraphe : la seule qui porte la puce.
    pub first: bool,
    /// Début du paragraphe brut, préfixe compris — là où un curseur posé dans le préfixe
    /// se rattache.
    pub paragraph_start: usize,
    /// Les fragments de cette ligne, comme plage dans [`TextLayout::fragments`].
    pub fragments: std::ops::Range<usize>,
}

/// Le texte d'une boîte, mis en page : des lignes, et les fragments qu'elles portent.
///
/// Les fragments de toutes les lignes vivent dans un seul tableau, chaque ligne n'en
/// désignant qu'une plage : une mise en page, une allocation, quel que soit le nombre de
/// styles — ce qui compte puisqu'elle est recalculée à chaque frame pour chaque carte visible.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextLayout {
    pub lines: Vec<VisualLine>,
    pub fragments: Vec<Fragment>,
}

impl TextLayout {
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn fragments_of(&self, line: &VisualLine) -> &[Fragment] {
        &self.fragments[line.fragments.clone()]
    }
}

/// Ce qu'il faut savoir d'une boîte pour y couler du texte, **en unités monde**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextBox {
    /// Largeur offerte au texte, marges déjà retirées.
    pub usable: f32,
    /// Corps du texte courant.
    pub body: f32,
    /// Décalage supplémentaire du texte d'une puce.
    pub bullet_indent: f32,
    /// Hauteur d'une ligne — elle sert à savoir combien de rangs une formule occupe.
    pub line_height: f32,
}

/// La tranche qui porte l'octet `at`.
///
/// Les tranches partitionnent la source (SPAN-1), donc elles sont triées et disjointes : une
/// recherche dichotomique suffit, et la mesure d'un paragraphe reste en `O(n log k)`.
fn span_at(spans: &[Span], at: usize) -> Option<&Span> {
    let found = spans.binary_search_by(|s| {
        if at < s.start {
            std::cmp::Ordering::Greater
        } else if at >= s.end {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });
    found.ok().map(|i| &spans[i])
}

/// Les tranches d'un paragraphe, signes de bloc compris, **relatives au paragraphe**.
///
/// Le préfixe (`# `, `> `, `1. `) et le suffixe (le `$$` de fermeture) sont des signes au
/// même titre que `**` : c'est ce qui les fait disparaître au repos et revenir à l'édition,
/// sans code particulier nulle part (MODE-1).
///
/// Un bloc littéral — du code, une formule — n'est pas analysé : ses octets se lisent tels
/// quels, et son corps ne fait qu'une tranche.
fn paragraph_spans(paragraph: &str, block: Block) -> Vec<Span> {
    let (prefix, suffix) = (block.prefix, block.suffix);
    let body = prefix..paragraph.len() - suffix;

    // Le cas courant — un paragraphe sans signe de bloc à analyser — n'a rien à décaler ni à
    // recopier.
    if prefix == 0 && suffix == 0 && !block.kind.literal() {
        return inline_spans(paragraph);
    }

    let inner = if block.kind.literal() {
        (!body.is_empty())
            .then(|| Span {
                start: 0,
                end: body.len(),
                emphasis: Emphasis::NONE,
                role: SpanRole::Text,
            })
            .into_iter()
            .collect()
    } else {
        inline_spans(&paragraph[body.clone()])
    };

    let mut spans = Vec::with_capacity(inner.len() + 2);
    let mark = |start, end| Span {
        start,
        end,
        emphasis: Emphasis::NONE,
        role: SpanRole::Marker,
    };
    if prefix > 0 {
        spans.push(mark(0, prefix));
    }
    spans.extend(inner.into_iter().map(|s| Span {
        start: s.start + prefix,
        end: s.end + prefix,
        ..s
    }));
    if suffix > 0 {
        spans.push(mark(paragraph.len() - suffix, paragraph.len()));
    }
    spans
}

/// WRAP-1 + RICH-1 — la mise en page de `source` dans une boîte de `bx` unités monde.
pub fn layout_rich_text(
    typography: &Typography,
    math: &MathRenderer,
    source: &str,
    bx: TextBox,
    mode: TextMode,
) -> TextLayout {
    let mut out = TextLayout::default();
    for block in blocks(source) {
        match block.kind {
            BlockKind::Math { display } => {
                layout_formula(&mut out, math, source, block, display, bx)
            }
            // Un trait, une clôture : rien à mesurer. Au repos la clôture ne prend même pas
            // de place — on n'a pas à voir un blanc là où un signe s'est effacé. En édition
            // elle redevient une ligne comme une autre, parce qu'on doit pouvoir la corriger.
            BlockKind::Fence if mode == TextMode::Rendered => {}
            BlockKind::Rule => out.lines.push(bare_line(block, block.kind)),
            _ => layout_paragraph(&mut out, typography, source, block, bx, mode),
        }
    }
    out
}

/// Une ligne sans texte : elle occupe son rang et ne porte aucun fragment.
fn bare_line(block: Block, kind: BlockKind) -> VisualLine {
    VisualLine {
        start: block.start,
        end: block.end,
        kind,
        first: true,
        paragraph_start: block.start,
        fragments: 0..0,
    }
}

/// Un paragraphe de texte : sa largeur utile, ses tranches, ses lignes refluées.
fn layout_paragraph(
    out: &mut TextLayout,
    typography: &Typography,
    source: &str,
    block: Block,
    bx: TextBox,
    mode: TextMode,
) {
    let paragraph = block.slice(source);
    let offset = block.start;
    let spans = paragraph_spans(paragraph, block);
    let base = emphasis_of(block.kind);
    let font = font_of(block.kind, bx.body);
    let usable = (bx.usable - indent_of(block.kind, bx.bullet_indent)).max(bx.body);

    let advance = |at: usize, ch: char| {
        let span = span_at(&spans, at);
        // MODE-1 : au repos, un signe n'occupe aucune place — c'est tout ce qui distingue
        // les deux vues, et la coupe des lignes en découle sans autre traitement.
        if mode == TextMode::Rendered && span.is_some_and(|s| s.role == SpanRole::Marker) {
            return 0.0;
        }
        let emphasis = span.map_or(Emphasis::NONE, |s| s.emphasis);
        let face = Face::from(base.union(emphasis));
        typography.advance(ch, font, face)
    };

    for (i, (s, e)) in wrap_paragraph(paragraph, usable, advance)
        .into_iter()
        .enumerate()
    {
        let from = out.fragments.len();
        for span in spans.iter().filter(|sp| sp.start < e && sp.end > s) {
            if mode == TextMode::Rendered && span.role == SpanRole::Marker {
                continue;
            }
            out.fragments.push(Fragment {
                start: offset + span.start.max(s),
                end: offset + span.end.min(e),
                // Un signe ne porte jamais l'emphase qu'il commande — et le `# ` d'un
                // titre commande le gras de sa ligne au même titre que `**` commande
                // celui d'un mot. Il garde donc le **corps** du titre, qui est sa taille,
                // sans en prendre la graisse : c'est ce qui le distingue du texte à côté.
                emphasis: match span.role {
                    SpanRole::Marker => span.emphasis,
                    SpanRole::Text => base.union(span.emphasis),
                },
                role: span.role,
            });
        }
        out.lines.push(VisualLine {
            start: offset + s,
            end: offset + e,
            kind: block.kind,
            first: i == 0,
            paragraph_start: offset,
            fragments: from..out.fragments.len(),
        });
    }
}

/// Une formule occupe **plusieurs hauteurs de ligne**, mais une seule d'entre elles porte sa
/// source : les autres ne sont là que pour réserver la place. C'est ce qui permet à la hauteur
/// d'une carte de rester « le nombre de lignes × la hauteur d'une ligne », sans cas
/// particulier ailleurs.
fn layout_formula(
    out: &mut TextLayout,
    math: &MathRenderer,
    source: &str,
    block: Block,
    display: bool,
    bx: TextBox,
) {
    let hauteur = math
        .measure(block.body_slice(source), mode_of(display), bx.body)
        .map(|(_, h, d)| h + d)
        .unwrap_or(bx.line_height);
    let rangs = (hauteur / bx.line_height).ceil().max(1.0) as usize;
    for i in 0..rangs {
        let from = out.fragments.len();
        if i == 0 {
            out.fragments.push(Fragment {
                start: block.start,
                end: block.end,
                emphasis: Emphasis::NONE,
                role: SpanRole::Text,
            });
        }
        out.lines.push(VisualLine {
            start: if i == 0 { block.start } else { block.end },
            end: block.end,
            kind: block.kind,
            first: i == 0,
            paragraph_start: block.start,
            fragments: from..out.fragments.len(),
        });
    }
}

/// Le booléen du noyau, traduit dans le vocabulaire de la crate des formules.
///
/// C'est la seule ligne où les deux se rencontrent : `glucose-core` n'a aucune dépendance, et
/// ne peut donc pas nommer `glucose_math::Mode` lui-même.
pub fn mode_of(display: bool) -> glucose_math::Mode {
    if display {
        glucose_math::Mode::Display
    } else {
        glucose_math::Mode::Inline
    }
}

/// Le style d'un fragment : son visage, son corps, son encre.
///
/// L'encre est celle que l'appelant donne à la ligne, sauf pour un signe de Markdown, qui
/// s'écrit en gris afin de se distinguer du texte qu'il commande. Le code, lui, garde
/// l'encre de sa ligne : c'est son fond et sa chasse fixe qui le détachent (comme dans la
/// référence), pas une couleur de plus.
pub fn fragment_style(
    fragment: &Fragment,
    font: f32,
    theme: &Theme,
    ink: tiny_skia::Color,
) -> TextStyle {
    let color = match fragment.role {
        SpanRole::Marker => theme.card_marker,
        SpanRole::Text => ink,
    };
    TextStyle {
        size: font,
        color,
        face: fragment.face(),
    }
}

#[cfg(test)]
mod tests;
