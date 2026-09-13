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
use glucose_core::text::{inline_spans, Emphasis, Span, SpanRole};

/// Interligne, en multiples du corps (fiche 06 § 5.1 : `lineHeight: 1.4`).
pub const LINE_FACTOR: f32 = 1.4;
/// Grossissement d'un titre `# `.
const H1_FACTOR: f32 = 1.25;
/// Grossissement d'un sous-titre `## `.
const H2_FACTOR: f32 = 1.10;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Heading1,
    Heading2,
    Bullet,
    Body,
    /// Un paragraphe qui est **entièrement** une formule : `$...$` ou `$$...$$`, seuls sur leur
    /// ligne. Il ne se reflue pas — une formule ne se coupe pas en deux — et se dessine par le
    /// moteur mathématique au lieu du moteur de texte.
    Math,
}

/// Les délimiteurs d'une formule qui occupe tout un paragraphe, et le mode qu'ils demandent.
///
/// Le LaTeX **au milieu** d'une phrase n'est pas traité ici : il demande de découper une ligne
/// en segments de nature différente, et de mesurer chacun. C'est un chantier à part, et le cas
/// fréquent dans un canva est la formule posée seule.
pub fn formule_entiere(paragraph: &str) -> Option<(&str, glucose_math::Mode)> {
    let t = paragraph.trim();
    if let Some(corps) = t.strip_prefix("$$").and_then(|r| r.strip_suffix("$$")) {
        if !corps.trim().is_empty() {
            return Some((corps, glucose_math::Mode::Display));
        }
    }
    if let Some(corps) = t.strip_prefix('$').and_then(|r| r.strip_suffix('$')) {
        if !corps.trim().is_empty() && !corps.contains('$') {
            return Some((corps, glucose_math::Mode::Inline));
        }
    }
    None
}

impl LineKind {
    /// Le genre du paragraphe et la longueur en octets de son préfixe.
    pub fn of(paragraph: &str) -> (Self, usize) {
        if formule_entiere(paragraph).is_some() {
            (Self::Math, 0)
        } else if paragraph.starts_with("# ") {
            (Self::Heading1, 2)
        } else if paragraph.starts_with("## ") {
            (Self::Heading2, 3)
        } else if paragraph.starts_with("- ") || paragraph.starts_with("* ") {
            (Self::Bullet, 2)
        } else {
            (Self::Body, 0)
        }
    }

    /// Les grossissements de titre sont des multiples du corps : ils héritent de l'unique
    /// transformation au lieu de redériver du zoom.
    pub fn font(self, body: f32) -> f32 {
        match self {
            Self::Heading1 => body * H1_FACTOR,
            Self::Heading2 => body * H2_FACTOR,
            Self::Bullet | Self::Body | Self::Math => body,
        }
    }

    /// L'emphase que le genre impose à toute la ligne. Elle s'**unit** à celle des fragments,
    /// ce qui fait qu'un `*mot*` dans un titre est en gras italique sans cas particulier.
    pub fn emphasis(self) -> Emphasis {
        match self {
            Self::Heading1 | Self::Heading2 => Emphasis::BOLD,
            Self::Bullet | Self::Body | Self::Math => Emphasis::NONE,
        }
    }

    fn indent(self, bullet_indent: f32) -> f32 {
        if self == Self::Bullet {
            bullet_indent
        } else {
            0.0
        }
    }

    pub fn color(self, theme: &Theme) -> tiny_skia::Color {
        match self {
            Self::Heading1 => theme.card_heading,
            Self::Heading2 => theme.card_subheading,
            Self::Bullet | Self::Body | Self::Math => theme.card_body,
        }
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
    pub kind: LineKind,
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

/// Les tranches d'un paragraphe, préfixe de bloc compris.
///
/// Le préfixe (`# `, `- `) est un signe au même titre que `**` : c'est ce qui le fait
/// disparaître au repos et revenir à l'édition, sans code particulier nulle part (MODE-1).
fn paragraph_spans(paragraph: &str, prefix: usize) -> Vec<Span> {
    // Le cas courant — un paragraphe sans préfixe de bloc — n'a rien à décaler ni à recopier.
    if prefix == 0 {
        return inline_spans(paragraph);
    }
    let inner = inline_spans(&paragraph[prefix..]);
    let mut spans = Vec::with_capacity(inner.len() + 1);
    spans.push(Span {
        start: 0,
        end: prefix,
        emphasis: Emphasis::NONE,
        role: SpanRole::Marker,
    });
    spans.extend(inner.into_iter().map(|s| Span {
        start: s.start + prefix,
        end: s.end + prefix,
        ..s
    }));
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
    let mut offset = 0usize;

    for paragraph in source.split('\n') {
        let (kind, prefix) = LineKind::of(paragraph);

        if kind == LineKind::Math {
            layout_formula(&mut out, math, paragraph, offset, bx);
            offset += paragraph.len() + 1;
            continue;
        }

        let spans = paragraph_spans(paragraph, prefix);
        let base = kind.emphasis();
        let font = kind.font(bx.body);
        let usable = (bx.usable - kind.indent(bx.bullet_indent)).max(bx.body);

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
                kind,
                first: i == 0,
                paragraph_start: offset,
                fragments: from..out.fragments.len(),
            });
        }
        offset += paragraph.len() + 1;
    }
    out
}

/// Une formule occupe **plusieurs hauteurs de ligne**, mais une seule d'entre elles porte sa
/// source : les autres ne sont là que pour réserver la place. C'est ce qui permet à la hauteur
/// d'une carte de rester « le nombre de lignes × la hauteur d'une ligne », sans cas
/// particulier ailleurs.
fn layout_formula(
    out: &mut TextLayout,
    math: &MathRenderer,
    paragraph: &str,
    offset: usize,
    bx: TextBox,
) {
    let (corps, mode) = formule_entiere(paragraph).expect("le genre vient d'être reconnu");
    let hauteur = math
        .measure(corps, mode, bx.body)
        .map(|(_, h, d)| h + d)
        .unwrap_or(bx.line_height);
    let rangs = (hauteur / bx.line_height).ceil().max(1.0) as usize;
    for i in 0..rangs {
        let from = out.fragments.len();
        if i == 0 {
            out.fragments.push(Fragment {
                start: offset,
                end: offset + paragraph.len(),
                emphasis: Emphasis::NONE,
                role: SpanRole::Text,
            });
        }
        out.lines.push(VisualLine {
            start: if i == 0 {
                offset
            } else {
                offset + paragraph.len()
            },
            end: offset + paragraph.len(),
            kind: LineKind::Math,
            first: i == 0,
            paragraph_start: offset,
            fragments: from..out.fragments.len(),
        });
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
