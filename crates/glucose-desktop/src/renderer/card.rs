//! Rendu des annotations : la carte de texte, et l'aiguillage vers les autres formes.
//!
//! La **mise en page** du texte n'est plus ici : elle vit dans [`super::richtext`], qui la
//! calcule en fragments stylés. Ce module dessine la boîte — fond, cadre, puce, curseur,
//! poignées — et pose les fragments que la mise en page lui donne.
//!
//! # CARD-1 — la carte est décrite en unités monde, puis mise à l'échelle une seule fois
//!
//! [`CardLayout`] ne connaît pas le zoom : elle décrit une carte comme si elle était
//! dessinée à l'échelle 1 — sa boîte, sa police, ses marges, son rayon de coin, l'indentation
//! de ses puces et l'épaisseur de son cadre, tous dans la même unité. [`CardLayout::scaled`]
//! applique ensuite **le même facteur à tous ces champs d'un coup** (SCALE-1).
//!
//! C'est la seule façon d'obtenir une carte semblable à elle-même à tous les zooms, et c'est
//! ce que **R-45** reprochait au code précédent : la boîte s'y mettait à l'échelle librement
//! pendant que des `clamp` figeaient son contenu à six seuils différents. Mesuré sur la même
//! carte (`card/proof.rs`), cela donnait cinq cartes pour un seul document :
//!
//! | Zoom | Ce que la carte d'avant montrait |
//! |---:|---|
//! | 0,25 | **aucun texte** (police bornée à 8 px, sous le seuil de rendu) et une boîte 70 % trop haute |
//! | 0,5 | **aucun texte** non plus |
//! | 1 | fidèle — le seul zoom où aucune borne ne se déclenchait |
//! | 2 | texte figé à 24 px : 0,49 de la largeur au lieu de 0,60 |
//! | 4 | texte toujours figé : 0,25 de la largeur |
//!
//! Corollaire important : **la mise en page se calcule avant la mise à l'échelle**. Le
//! découpage en lignes ([`super::richtext::layout_rich_text`], WRAP-1) et la hauteur nécessaire
//! au contenu (`CardLayout::text_card`) sont dérivés d'une police en unités monde ; calculés
//! après, ils dépendraient d'une police écran et la carte se réorganiserait à chaque palier
//! de zoom.
//!
//! Les ornements du Markdown — le fond d'un `` `code` ``, la barre d'un `~~barré~~` — ne
//! sont pas des champs de [`CardLayout`] : ce sont des **fractions du corps**, et le corps
//! est déjà à l'échelle. Une longueur qui peut se dire en multiples d'une autre n'a pas à
//! être mise à l'échelle séparément, donc elle n'a pas à exister séparément.
//!
//! # TEXT-FIT-1 — le texte est le **plancher** de la carte, pas sa mesure
//!
//! Une carte ne tronque jamais son contenu : sa hauteur ne descend pas sous celle de son
//! texte reflué à sa largeur. Au-dessus, elle est libre — on la tire où l'on veut, et le
//! texte qui grandit la pousse sans jamais la fixer. **La liberté n'empêche pas la
//! contrainte.**
//!
//! La règle antérieure faisait de la hauteur une conséquence exacte du texte, et retirait
//! donc les poignées haute et basse. Viser le milieu du bord haut ne trouvait rien — ce qui
//! se lit comme une zone de clic trop petite, pas comme une poignée absente.
//!
//! Le geste de redimensionnement et la validation d'une saisie écrivent la hauteur effective
//! dans le document ([`text_card_fit_height`]), pour que le test de clic, l'index spatial et
//! les poignées voient la même boîte que l'écran : c'est **une** hauteur, jamais deux.
//!
//! La hauteur se mesure **sur le texte rendu**, jamais sur sa source : une carte ne doit pas
//! changer de taille au moment où on la sélectionne pour l'éditer. Pendant l'édition, les
//! signes réapparaissent et le texte peut demander une ligne de plus — la carte l'affiche,
//! puisque `text_card` prend le maximum entre la hauteur écrite et celle qu'il faut.

use super::handles::draw_resize_handles;
use super::pass::{Pass, SELECTION_RING};
use super::richtext::draw::{draw_line, draw_line_selection};
use super::richtext::hit::offset_to_x;
use super::richtext::{
    font_of, indent_of, ink_of, layout_rich_text, Ink, TextBox, TextLayout, TextMode, VisualLine,
    LINE_FACTOR,
};
use super::scale::WorldScale;
use super::{push_rounded_rect, TextEditSession};
use crate::canvas::world_to_screen;
use crate::renderer::math::MathRenderer;
use crate::typography::Typography;

mod ornament;
use crate::theme::Theme;
use glucose_core::resize::Handle;
use glucose_core::text::{BlockKind, Selection};
use ornament::{draw_code_plate, draw_ornament};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

// ── Mesures d'une carte, en unités monde ────────────────────────────────────

/// Corps de texte d'une carte.
pub(super) const BODY_FONT: f32 = 14.0;
/// Marge horizontale entre le bord de la carte et son texte (fiche 06 § 5.1 : `16px 24px`).
const PAD_X: f32 = 24.0;
/// Marge verticale entre le bord de la carte et son texte.
const PAD_Y: f32 = 16.0;
/// Rayon des coins de la carte (fiche 06 § 5.1 : 32 px, « nuage / brume »).
const CORNER_RADIUS: f32 = 32.0;
/// Décalage du texte d'une puce `- ` par rapport au reste.
const BULLET_INDENT: f32 = 14.0;
/// Rayon du disque d'une puce.
const BULLET_RADIUS: f32 = 2.2;
/// Décalage du disque d'une puce depuis la marge gauche.
const BULLET_OFFSET: f32 = 3.0;
/// Position de la puce sur la hauteur de la ligne, en multiples du corps.
const BULLET_BASELINE: f32 = 0.45;
/// Épaisseur du cadre d'une carte au repos.
const BORDER: f32 = 1.0;
/// Largeur du curseur d'édition.
const CURSOR_WIDTH: f32 = 2.0;
/// Hauteur du curseur d'édition, en multiples du corps.
const CURSOR_HEIGHT: f32 = 1.2;

/// La mise en page du texte d'une carte de `width` unités monde, dans le mode demandé.
pub fn card_text_layout(
    typography: &Typography,
    math: &MathRenderer,
    body: &str,
    width: f32,
    mode: TextMode,
) -> TextLayout {
    layout_rich_text(typography, math, body, text_box(width), mode)
}

/// La boîte offerte au texte dans une carte de `width` unités monde.
pub fn text_box(width: f32) -> TextBox {
    TextBox {
        usable: (width - PAD_X * 2.0).max(BODY_FONT),
        body: BODY_FONT,
        bullet_indent: BULLET_INDENT,
        line_height: BODY_FONT * LINE_FACTOR,
    }
}

/// TEXT-FIT-1 — la hauteur, en unités monde, qu'une carte de `width` doit avoir pour
/// contenir `text` sans le tronquer. C'est ce que le geste de redimensionnement et la
/// validation d'une saisie écrivent dans le document.
pub fn text_card_fit_height(
    typography: &Typography,
    math: &MathRenderer,
    text: &str,
    width: f64,
) -> f64 {
    let lines =
        card_text_layout(typography, math, text, width as f32, TextMode::Rendered).line_count();
    CardLayout::text_card(width as f32, 0.0, lines).height as f64
}

/// Le décalage du texte depuis le coin haut-gauche de la carte, en unités monde.
///
/// La souris et le clavier en ont besoin pour rapporter un point à la mise en page ; il est
/// donné ici plutôt que recopié là-bas, sans quoi un changement de marge décalerait le clic
/// sans décaler le texte.
pub const TEXT_ORIGIN: (f32, f32) = (PAD_X, PAD_Y);

// ── Mise en page d'une carte ────────────────────────────────────────────────

/// Mise en page d'une carte de texte. **En unités monde tant que `scaled` n'a pas été
/// appelée** ; en pixels écran après, et rien d'autre ne dérive du zoom entre les deux.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    pub width: f32,
    pub height: f32,
    pub font: f32,
    pub line_height: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub indent: f32,
    pub bullet: f32,
    pub bullet_offset: f32,
    pub border: f32,
}

impl CardLayout {
    /// Mise en page d'une carte de `width` × `height` unités monde contenant `lines` lignes.
    ///
    /// La hauteur s'étire pour contenir le texte : une carte ne tronque jamais son contenu.
    pub(super) fn text_card(width: f32, height: f32, lines: usize) -> Self {
        let line_height = BODY_FONT * LINE_FACTOR;
        let needed = PAD_Y * 2.0 + lines.max(1) as f32 * line_height;
        Self {
            width,
            height: height.max(needed),
            font: BODY_FONT,
            line_height,
            pad_x: PAD_X,
            pad_y: PAD_Y,
            radius: CORNER_RADIUS,
            indent: BULLET_INDENT,
            bullet: BULLET_RADIUS,
            bullet_offset: BULLET_OFFSET,
            border: BORDER,
        }
    }

    /// L'UNIQUE transformation d'échelle de la carte (SCALE-1) : tous les champs, le même
    /// facteur, au même instant. Ajouter un champ ici sans le mettre à l'échelle, ou le
    /// borner au passage, c'est ramener R-45.
    pub fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            font: s.world(self.font),
            line_height: s.world(self.line_height),
            pad_x: s.world(self.pad_x),
            pad_y: s.world(self.pad_y),
            radius: s.world(self.radius),
            indent: s.world(self.indent),
            bullet: s.world(self.bullet),
            bullet_offset: s.world(self.bullet_offset),
            border: s.world(self.border),
        }
    }
}

pub(super) struct TextCard<'a> {
    pub origin: (f64, f64),
    pub size: (f32, f32),
    pub body: &'a str,
    pub tint: (u8, u8, u8),
    pub selected: bool,
    pub editing: Option<&'a TextEditSession>,
}

impl TextCard<'_> {
    /// MODE-1 — une carte qu'on corrige montre ses signes ; une carte qu'on lit ne les
    /// montre pas.
    fn mode(&self) -> TextMode {
        if self.editing.is_some() {
            TextMode::Source
        } else {
            TextMode::Rendered
        }
    }
}

pub(super) fn draw_text_card(ctx: &Pass, pixmap: &mut PixmapMut, card: TextCard) {
    // Le découpage en lignes et la hauteur nécessaire se calculent en unités monde, AVANT
    // l'unique mise à l'échelle (CARD-1, WRAP-1).
    let text = card_text_layout(
        ctx.typography,
        ctx.math,
        card.body,
        card.size.0,
        card.mode(),
    );
    let layout =
        CardLayout::text_card(card.size.0, card.size.1, text.line_count()).scaled(ctx.scale);

    let (wx, wy) = world_to_screen(card.origin.0, card.origin.1, &ctx.vp);
    let (sx, sy) = (wx as f32, wy as f32);
    if ctx.clip.rejects(sx, sy, layout.width, layout.height) {
        return;
    }

    draw_card_frame(ctx, pixmap, (sx, sy), &layout, &card);

    // SCALE-2 — l'unique niveau de détail : sous le seuil, la carte s'arrête à son cadre.
    if ctx.scale.draws_detail() {
        draw_card_body(ctx, pixmap, (sx, sy), &layout, &text, &card);
        ornament::draw_formula_preview(ctx, pixmap, (sx, sy), &layout, &text, &card);
    }
    if card.selected {
        let screen_box = (sx, sy, layout.width, layout.height);
        draw_resize_handles(pixmap, ctx.theme, ctx.scale, screen_box, &Handle::ALL);
    }
}

/// Le fond teinté de la carte et son cadre.
fn draw_card_frame(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    card: &TextCard,
) {
    let mut pb = PathBuilder::new();
    push_rounded_rect(
        &mut pb,
        at.0,
        at.1,
        layout.width,
        layout.height,
        layout.radius,
    );
    let Some(path) = pb.finish() else {
        return;
    };
    let (r, g, b) = card.tint;

    // Aura douce d'ambiance : #18181B teinté de 12 % de la teinte symbiotique.
    let mut fill = Paint {
        anti_alias: true,
        ..Default::default()
    };
    fill.set_color(Color::from_rgba8(
        ((r as u16 * 12 + 24 * 88) / 100) as u8,
        ((g as u16 * 12 + 24 * 88) / 100) as u8,
        ((b as u16 * 12 + 27 * 88) / 100) as u8,
        248,
    ));
    pixmap.fill_path(
        &path,
        &fill,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );

    let highlighted = card.selected || card.editing.is_some();
    let mut stroke_paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    stroke_paint.set_color(if highlighted {
        ctx.theme.selection_frame
    } else {
        Color::from_rgba8(r, g, b, 60)
    });
    let stroke = Stroke {
        // Le cadre au repos appartient à la carte et suit son échelle ; l'anneau de
        // sélection est une affordance et garde sa taille écran (exception SCALE-1).
        width: if highlighted {
            ctx.scale.screen(SELECTION_RING)
        } else {
            layout.border
        },
        ..Default::default()
    };
    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
}

/// Le texte de la carte, ligne visuelle par ligne visuelle, curseur d'édition compris.
fn draw_card_body(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    text: &TextLayout,
    card: &TextCard,
) {
    let mut caret = Caret::of(card);
    let mut cur_y = at.1 + layout.pad_y;
    for (num, line) in text.lines.iter().enumerate() {
        let last = num + 1 == text.lines.len();
        let rang = (text, line, last);
        draw_text_line(ctx, pixmap, (at.0, cur_y), layout, rang, card, &mut caret);
        cur_y += layout.line_height;
    }
}

/// Ce que la saisie en cours dit au tracé : ce qui est sélectionné, où est le curseur, s'il
/// doit se voir à cet instant — et s'il a déjà été posé, puisqu'une seule ligne le porte.
struct Caret {
    selection: Selection,
    visible: bool,
    drawn: bool,
}

impl Caret {
    fn of(card: &TextCard) -> Self {
        Self {
            selection: card.editing.map(|s| s.selection).unwrap_or_default(),
            // Le curseur clignote, la sélection non : un fond qui s'allume et s'éteint
            // rendrait la lecture du texte sélectionné impossible.
            //
            // La phase se **lit** (BLINK-1) : la calculer ici rendrait le dessin dépendant de
            // l'instant où il a lieu, donc non reproductible.
            visible: card.editing.is_some_and(|s| s.curseur_visible),
            drawn: false,
        }
    }
}

/// Une ligne de carte : son ornement, son surlignage, son texte, et le curseur s'il y tombe.
///
/// `left` est le bord gauche de la carte ; tout le reste s'en déduit — la marge, puis le
/// retrait que le genre du bloc demande.
fn draw_text_line(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    (left, y): (f32, f32),
    layout: &CardLayout,
    (text, line, last): (&TextLayout, &VisualLine, bool),
    card: &TextCard,
    caret: &mut Caret,
) {
    let text_left = left + layout.pad_x;
    let start_x = text_left + indent_of(line.kind, layout.indent);
    let font = font_of(line.kind, layout.font);

    // Le fond d'un bloc de code n'est pas un signe mais une matière : il reste pendant qu'on
    // écrit dedans, là où puce, numéro et barre s'effacent au profit de leur signe.
    if line.kind == BlockKind::Code {
        draw_code_plate(ctx, pixmap, (text_left, y), layout);
    }

    // Les ornements **remplacent** un signe que le repos efface ; pendant l'édition, c'est le
    // signe qu'on voit et qu'on corrige (MODE-1). Une formule obéit à la même règle : on
    // n'édite pas une fraction, on édite le texte qui la décrit.
    if card.editing.is_none()
        && draw_ornament(ctx, pixmap, (text_left, start_x, y), layout, line, card)
    {
        return;
    }

    let ink = Ink {
        text: ink_of(line.kind, ctx.theme),
        link: ctx.theme.link,
        marker: marker_ink(ctx.math, ctx.theme, line, card.body, card.editing.is_some()),
    };
    // Le surlignage passe sous le texte : dessiné après, il le recouvrirait.
    draw_line_selection(
        ctx,
        pixmap,
        (start_x, y),
        (text, line),
        (font, layout.line_height),
        card.body,
        caret.selection,
    );
    draw_line(
        ctx,
        pixmap,
        (start_x, y),
        (text, line),
        (font, ink),
        card.body,
    );

    if caret.visible && !caret.drawn {
        let cursor = (card.body, caret.selection.head);
        caret.drawn = draw_line_cursor(
            ctx,
            pixmap,
            (start_x, y),
            (text, line, last),
            (font, ink.text),
            cursor,
            layout,
        );
    }
}

/// Le paragraphe entier auquel appartient une ligne visuelle.
///
/// Une ligne refluée n'est qu'un morceau : les `$` d'ouverture sont sur la première, ceux de
/// fermeture sur la dernière. Tout ce qui interroge la **nature** d'un paragraphe — sa
/// formule, sa validité — doit donc le lire en entier, pas la tranche qu'il a sous la main.
pub(super) fn paragraph_of<'a>(source: &'a str, line: &VisualLine) -> &'a str {
    // Le separateur de paragraphes du modele.
    const LF: char = '\u{000A}';
    let debut = line.paragraph_start.min(source.len());
    let fin = source[debut..].find(LF).map_or(source.len(), |i| debut + i);
    &source[debut..fin]
}

/// L'encre des **signes** d'une ligne.
///
/// Grise pour tout le monde — sauf les `$` d'une formule pendant l'édition, qui disent si
/// elle compile : verte quand oui, rouge quand non. C'est le seul retour immédiat qu'on
/// puisse donner à quelqu'un qui écrit du LaTeX, et il ne coûte rien de plus qu'une mise en
/// page déjà faite : le résultat de la composition est mis en cache, et le repos n'y passe
/// même pas, puisqu'au repos les signes n'existent plus.
///
/// Le vert et le rouge sont ceux du **contenu** (`success`, `danger`), pas ceux de
/// l'interface : une formule qui compile ou non est quelque chose que l'auteur a écrit, pas
/// l'état d'un bouton.
fn marker_ink(
    math: &super::math::MathRenderer,
    theme: &Theme,
    line: &VisualLine,
    source: &str,
    editing: bool,
) -> Color {
    let BlockKind::Math { display } = line.kind else {
        return theme.card_marker;
    };
    if !editing {
        return theme.card_marker;
    }
    let texte = paragraph_of(source, line);
    let Some((corps, _)) = glucose_core::text::block::formula(texte) else {
        return theme.card_marker;
    };
    if math
        .layout(&texte[corps], super::richtext::mode_of(display))
        .is_ok()
    {
        theme.success
    } else {
        theme.danger
    }
}

/// Le curseur, s'il tombe sur cette ligne — et `true` quand il y a été posé.
///
/// Un curseur posé dans le préfixe d'un bloc (`# `, `> `) se rattache au début de sa première
/// ligne ; la dernière ligne du texte recueille tout ce qui dépasse sa fin.
#[allow(clippy::too_many_arguments)]
fn draw_line_cursor(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    (text, line, last): (&TextLayout, &VisualLine, bool),
    (font, ink): (f32, Color),
    (source, cursor): (&str, usize),
    layout: &CardLayout,
) -> bool {
    let from = if line.first {
        line.paragraph_start
    } else {
        line.start
    };
    if cursor < from || (cursor > line.end && !last) {
        return false;
    }
    let dx = offset_to_x(ctx.typography, text, line, source, cursor, font);
    draw_cursor(pixmap, (at.0 + dx, at.1), layout, ctx.scale, ink);
    true
}

/// Le curseur d'édition, à l'encre de la ligne qu'il édite.
fn draw_cursor(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    scale: WorldScale,
    ink: Color,
) {
    // Le curseur mesure le texte qu'il édite — sa hauteur suit la police — mais son trait
    // est une affordance : il garde sa largeur écran (exception SCALE-1), comme le curseur
    // de n'importe quel éditeur. Mis à l'échelle, il s'effacerait au dézoom.
    let width = scale.screen(CURSOR_WIDTH);
    if let Some(rect) = Rect::from_xywh(at.0, at.1, width, layout.font * CURSOR_HEIGHT) {
        // Sur la grille (SCALE-3) : un curseur d'un pixel posé à une demi-position devient
        // deux demi-traits gris, et il clignote — donc il attire l'œil sur son propre flou.
        super::scale::fill_crisp(pixmap, rect, ink);
    }
}

#[cfg(test)]
mod proof;

#[cfg(test)]
pub mod tests;
