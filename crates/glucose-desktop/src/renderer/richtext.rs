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

pub mod brut;
pub mod draw;
mod genre;
pub mod hit;
pub(crate) mod place;
pub mod table;

pub use genre::{emphasis_of, font_of, indent_of, ink_of, retrait_d_ornement};

use super::math::MathRenderer;
use super::wrap::wrap_paragraph;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::text::{blocks, inline_spans, Block, BlockKind, Emphasis, Span, SpanRole};

/// Interligne, en multiples du corps (fiche 06 § 5.1 : `lineHeight: 1.4`).
pub const LINE_FACTOR: f32 = 1.4;

/// Ce que l'affichage fait des signes du Markdown (MODE-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

// Du genre d'un bloc aux pixels — taille, graisse, encre, retrait : [`genre`].

/// Valeur de [`Fragment::tab`] qui veut dire « à la suite du fragment précédent ».
///
/// Une sentinelle plutôt qu'un `Option<f32>` : ce champ n'existe que pour les cellules d'un
/// tableau, qui sont rares, et un `Option` ferait grossir d'un tiers une structure allouée
/// par milliers à chaque image. Un taquet est toujours positif ou nul, donc une abscisse
/// négative ne peut désigner aucune colonne.
pub const NO_TAB: f32 = -1.0;

/// Un morceau de ligne dessiné d'un seul trait : même visage, même emphase, même rôle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fragment {
    /// Tranche `[start, end)` de la **source**, jamais d'un texte reconstruit.
    pub start: usize,
    pub end: usize,
    pub emphasis: Emphasis,
    pub role: SpanRole,
    /// L'abscisse où ce fragment commence, depuis le début de sa ligne, **en multiples du
    /// corps de sa ligne** — ou [`NO_TAB`] pour « là où le précédent s'est arrêté », qui est
    /// le cas de tout ce qui n'est ni un tableau ni un passage éclairé.
    ///
    /// C'est ce qui permet à des colonnes de s'aligner : sans taquet, un fragment ne peut
    /// que suivre, et deux lignes de longueurs différentes ne tomberaient jamais en face.
    ///
    /// # TABLE-2 — une longueur dite dans l'unité du texte
    ///
    /// Le taquet était une abscisse du monde, et le tracé l'ajoutait à une position d'écran
    /// sans la mettre à l'échelle : à ×2, la seconde colonne d'un tableau se posait à la
    /// moitié de sa place, sur la première. Dit en corps, il suit le zoom sans le connaître —
    /// le tracé le multiplie par le corps de l'écran, le clic par celui du monde.
    pub tab: f32,
    /// **Dans un passage qui brille** (PASSAGE-2) : le fragment se peint à la teinte de
    /// l'éclat. C'est [`place::ouvrir_la_place`] qui coupe les fragments aux bords des
    /// passages : un fragment est dedans ou dehors, jamais à moitié — la teinte suit les
    /// lettres, et non un rectangle découpé.
    pub eclaire: bool,
}

impl Fragment {
    pub fn face(&self) -> Face {
        Face::from(self.emphasis)
    }
}

/// Une ligne visuelle : sa tranche de source, son genre, et ses fragments.
#[derive(Clone, Debug, PartialEq)]
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
    /// **Ce que la ligne montre de loin** (GRIS-1, fiche 95), à la taille du monde où le texte se
    /// met en page : la largeur de son texte (sans ses blancs de fin), et son **encre** — la
    /// surface que ses lettres couvrent ([`Typography::encre`]). Son gris est l'encre sur la place.
    pub largeur: f32,
    pub encre: f32,
}

/// Le texte d'une boîte, mis en page : des lignes, et les fragments qu'elles portent.
///
/// Les fragments de toutes les lignes vivent dans un seul tableau, chaque ligne n'en
/// désignant qu'une plage : une mise en page, une allocation, quel que soit le nombre de
/// styles — ce qui compte puisqu'elle est recalculée à chaque frame pour chaque carte visible.
#[derive(Clone, Debug, Default, PartialEq)]
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

    /// **TABLE-4 — la ligne `rang` ouvre une rangée qui en suit une autre** (fiche 125) : la
    /// première ligne d'une rangée dont la ligne d'avant est une rangée aussi — ni l'en-tête, ni la
    /// rangée qui suit le filet de l'en-tête. C'est là qu'un filet les sépare.
    pub fn separe_une_rangee(&self, rang: usize) -> bool {
        let rangee = |l: &VisualLine| l.kind == BlockKind::TableRow;
        rang > 0
            && self.lines.get(rang).is_some_and(|l| rangee(l) && l.first)
            && rangee(&self.lines[rang - 1])
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
    if block.kind == BlockKind::TableRow {
        return table::spans_d_une_rangee(paragraph);
    }
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
    // Les blocs sont collectés plutôt que parcourus au fil de l'eau : un tableau ne se met en
    // page qu'en voyant **toutes** ses lignes d'un coup, puisque la largeur d'une colonne est
    // la plus large de ses cellules. Une allocation par carte visible, contre des colonnes
    // qui ne s'alignent pas.
    let liste: Vec<Block> = blocks(source).collect();
    let mut i = 0usize;
    while i < liste.len() {
        let block = liste[i];
        // Au repos seulement (TABLE-3) : en édition, une rangée est sa source, barres
        // comprises, coupée comme un paragraphe — les signes reviennent quand on corrige
        // (MODE-1), et le curseur, qui suit des lignes sans trou, y marche comme partout.
        if block.kind.in_table() && mode == TextMode::Rendered {
            let fin = liste[i..].iter().take_while(|b| b.kind.in_table()).count() + i;
            table::layout_table(&mut out, typography, source, &liste[i..fin], bx, mode);
            i = fin;
            continue;
        }
        i += 1;
        match block.kind {
            // Une formule est une **formule au repos, et du texte pendant qu'on l'écrit**.
            // C'est ce que le tracé faisait déjà — il montre la source en édition, parce
            // qu'on n'édite pas une fraction — mais la mise en page, elle, réservait dans les
            // deux cas la hauteur du résultat et posait la source en un seul fragment. Deux
            // conséquences : une source plus large que la carte débordait sans se couper, et
            // ses `$` n'étaient pas des signes, donc rien ne pouvait les colorer.
            BlockKind::Math { display } if mode == TextMode::Rendered => layout_formula(
                &mut out,
                (typography, math),
                source,
                block,
                (display, bx.body),
            ),
            // Un trait, une clôture : rien à mesurer. Au repos la clôture ne prend même pas
            // de place — on n'a pas à voir un blanc là où un signe s'est effacé. En édition
            // elle redevient une ligne comme une autre, parce qu'on doit pouvoir la corriger.
            BlockKind::Fence if mode == TextMode::Rendered => {}
            // La séparation d'un tableau reste un filet en édition, comme un `---`.
            BlockKind::Rule | BlockKind::TableRule => out.lines.push(bare_line(block, block.kind)),
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
        largeur: 0.0,
        encre: 0.0,
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

    // MODE-1 : au repos, un signe n'occupe aucune place — c'est tout ce qui distingue les deux
    // vues, et la coupe des lignes en découle sans autre traitement.
    let face_de = |at: usize| -> Option<Face> {
        let span = span_at(&spans, at);
        if mode == TextMode::Rendered && span.is_some_and(|s| s.role == SpanRole::Marker) {
            return None;
        }
        let emphasis = span.map_or(Emphasis::NONE, |s| s.emphasis);
        Some(Face::from(base.union(emphasis)))
    };
    let advance =
        |at: usize, ch: char| face_de(at).map_or(0.0, |f| typography.advance(ch, font, f));

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
                tab: NO_TAB,
                eclaire: false,
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
        // GRIS-1 : ce que la ligne montre de loin.
        let lettres = paragraph[s..e]
            .char_indices()
            .map(|(i, ch)| (ch, face_de(s + i)));
        let (largeur, encre) = typography.gris(lettres, font);
        out.lines.push(VisualLine {
            start: offset + s,
            end: offset + e,
            kind: block.kind,
            first: i == 0,
            paragraph_start: offset,
            fragments: from..out.fragments.len(),
            largeur,
            encre,
        });
    }
}

/// **La place d'une formule parmi les lignes d'une carte**, en corps de texte : combien de
/// lignes elle réserve, et à quelle hauteur, sous le haut de la première, tombe sa ligne de
/// base.
///
/// Une ligne de texte laisse un interligne entre ses glyphes et ceux de la suivante —
/// `LINE_FACTOR − 1` corps. La formule le garde aussi, moitié au-dessus, moitié en dessous :
/// deux formules hautes l'une sous l'autre ne se touchent jamais. Elles se touchaient (sa
/// capture du 28/09 : le « D » sous une intégrale double sur le « n » de l'accolade suivante),
/// parce qu'une formule réservait juste assez de lignes pour sa boîte et s'y posait en haut.
///
/// Tout est en corps : la carte se met en page à la taille du monde et se dessine à celle de
/// l'écran, et les deux lisent ces mêmes nombres — une ligne réservée de plus ou de moins d'un
/// zoom à l'autre, par un arrondi, n'existe pas. Une formule fausse réserve une ligne.
pub fn place_d_une_formule(
    math: &MathRenderer,
    corps: &str,
    mode: glucose_math::Mode,
) -> (usize, f32) {
    let Some((_, h, d)) = math.measure(corps, mode, 1.0) else {
        return (1, 1.0);
    };
    let interligne = LINE_FACTOR - 1.0;
    let rangs = ((h + d + interligne) / LINE_FACTOR).ceil().max(1.0);
    let libre = rangs * LINE_FACTOR - (h + d);
    (rangs as usize, libre / 2.0 + h)
}

/// Une formule occupe **plusieurs hauteurs de ligne**, mais une seule d'entre elles porte sa
/// source : les autres ne sont là que pour réserver la place. C'est ce qui permet à la hauteur
/// d'une carte de rester « le nombre de lignes × la hauteur d'une ligne », sans cas
/// particulier ailleurs.
fn layout_formula(
    out: &mut TextLayout,
    (typography, math): (&Typography, &MathRenderer),
    source: &str,
    block: Block,
    (display, corps): (bool, f32),
) {
    let (rangs, _) = place_d_une_formule(math, block.body_slice(source), mode_of(display));
    // GRIS-1 : de loin, une formule a le gris d'un texte de sa largeur, sur chaque ligne qu'elle
    // occupe — son encre n'est pas celle de lettres qu'on lit une à une.
    let largeur = math
        .measure(block.body_slice(source), mode_of(display), 1.0)
        .map_or(0.0, |(l, _, _)| l * corps);
    let encre = largeur * corps * typography.densite(Face::Regular);
    for i in 0..rangs {
        let from = out.fragments.len();
        if i == 0 {
            out.fragments.push(Fragment {
                tab: NO_TAB,
                eclaire: false,
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
            largeur,
            encre,
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

/// Les deux encres d'une ligne : celle de son texte, celle de ses signes.
///
/// Deux plutôt qu'une, parce qu'une ligne de formule colore ses `$` pendant l'édition selon
/// que la formule compile. Le gris des signes ordinaires ne dit rien ; le vert et le rouge
/// disent quelque chose, et ils le disent à l'endroit exact où l'auteur corrige.
#[derive(Clone, Copy, Debug)]
pub struct Ink {
    pub text: tiny_skia::Color,
    pub marker: tiny_skia::Color,
    pub link: tiny_skia::Color,
    /// L'encre d'un passage qui brille (PASSAGE-2) : la teinte de sa carte.
    pub eclat: tiny_skia::Color,
}

impl Ink {
    /// Les encres ordinaires d'un genre de bloc : son encre de texte, et le gris des signes.
    pub fn of(kind: BlockKind, theme: &Theme) -> Self {
        let text = ink_of(kind, theme);
        Self {
            text,
            marker: theme.card_marker,
            link: theme.link,
            eclat: text,
        }
    }
}

/// Le style d'un fragment : son visage, son corps, son encre.
///
/// Le code garde l'encre de sa ligne : c'est son fond et sa chasse fixe qui le détachent
/// (comme dans la référence), pas une couleur de plus.
pub fn fragment_style(fragment: &Fragment, font: f32, ink: Ink) -> TextStyle {
    TextStyle {
        size: font,
        color: match fragment.role {
            // Un passage qui brille prend la teinte de sa carte, comme le `<mark>` de Tauri —
            // lien compris : c'est le passage qu'on désigne, plus le lien.
            SpanRole::Text if fragment.eclaire => ink.eclat,
            SpanRole::Marker => ink.marker,
            // Un lien porte sa propre couleur : c'est ce qui dit qu'il mène ailleurs, et le
            // trait sous lui n'en est que la confirmation.
            SpanRole::Text if fragment.emphasis.link() => ink.link,
            SpanRole::Text => ink.text,
        },
        face: fragment.face(),
    }
}

#[cfg(test)]
mod tests;
