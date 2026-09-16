//! La passe sur les annotations : ce qu'une frame garde constant d'un nœud à l'autre, et le
//! tri qui envoie chaque nœud à son dessin — la carte, le pense-bête, la flèche.

use super::arrow::draw_arrow;
use super::card::{draw_text_card, TextCard};
use super::domain::{draw_domain_gauge, DomainTints};
use super::hue::SymbioticHueCache;
use super::math::MathRenderer;
use super::note::draw_sticky;
use super::scale::WorldScale;
use super::{parse_hex_color, PaintKit, TextEditSession};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, DomainAssignment, Viewport};
use tiny_skia::PixmapMut;

/// Épaisseur de l'anneau de sélection, **en pixels écran**.
///
/// Exception SCALE-1 : c'est une affordance, pas du contenu. Mis à l'échelle, il
/// disparaîtrait en dézoomant au moment précis où l'on cherche ce qu'on a sélectionné.
pub(super) const SELECTION_RING: f32 = 2.0;

// ── Ce qu'une passe d'annotations garde constant ────────────────────────────

/// Le bord de l'écran utile. Tout ce qui en sort est écarté avant d'être dessiné (loi L1).
#[derive(Clone, Copy)]
pub(crate) struct Clip {
    pub width: f32,
    pub height: f32,
    pub top: f32,
}

impl Clip {
    /// La boîte écran `(x, y, w, h)` est-elle entièrement hors champ ou trop petite ?
    pub(super) fn rejects(self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w < 0.0 || x > self.width || y + h < self.top || y > self.height || (w < 3.0 && h < 3.0)
    }
}

/// Ce qui ne change pas d'une annotation à l'autre pendant une frame.
pub(crate) struct Pass<'a> {
    pub typography: &'a Typography,
    /// Le moteur de formules — il ne mute rien de visible, son cache est interne.
    pub math: &'a MathRenderer,
    /// `domain_id → teinte`, déjà résolue pour cette version du document (DOMAIN-TINT-1).
    pub tints: &'a DomainTints,
    pub theme: &'a Theme,
    pub vp: Viewport,
    pub scale: WorldScale,
    pub clip: Clip,
}

/// Dessine les annotations visibles du tableau actif.
pub(super) fn draw_annotations(
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    editing_session: Option<&TextEditSession>,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp: pass.vp,
        scale: WorldScale::new(pass.vp.scale),
        clip: Clip {
            width: pixmap.width() as f32,
            height: pixmap.height() as f32,
            top: pass.header_h,
        },
    };

    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let selected = store.selected_annotation_ids.iter().any(|s| s == ann.id());
        let editing = editing_session.filter(|s| s.ann_id.as_str() == ann.id());
        match ann {
            Annotation::Text {
                x, y, text, color, ..
            } => {
                let (_, tint) = hue_cache.get_or_compute(ann, pass.index, board);
                let tint = color
                    .as_deref()
                    .map(|c| parse_hex_color(c, tint.0, tint.1, tint.2))
                    .unwrap_or(tint);
                let body = editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str());
                let (w, h) = ann
                    .size()
                    .expect("Annotation::size ne rend None que pour une flèche");
                let size = (w as f32, h as f32);
                draw_text_card(
                    &ctx,
                    pixmap,
                    TextCard {
                        origin: (*x, *y),
                        size,
                        body,
                        tint,
                        selected,
                        editing,
                    },
                );
                draw_node_gauge(&ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Sticky { x, y, .. } => {
                draw_sticky(&ctx, pixmap, ann, selected, editing);
                draw_node_gauge(&ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Arrow { .. } => {
                draw_arrow_node(&ctx, pixmap, ann, board, selected, editing);
            }
            _ => {}
        }
    }
}

/// Pose la réglette de domaines d'une annotation au-dessus de son bord haut.
fn draw_node_gauge(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    origin: (f64, f64),
    domains: &[DomainAssignment],
) {
    let (wx, wy) = world_to_screen(origin.0, origin.1, &ctx.vp);
    draw_domain_gauge(
        ctx.typography,
        ctx.tints,
        pixmap,
        ctx.scale,
        (wx as f32, wy as f32),
        domains,
    );
}

/// Une flèche entière : son tracé, puis ce qu'elle dit.
///
/// L'étiquette se pose **après** le tracé : elle doit se lire par-dessus, pas être barrée
/// par lui.
fn draw_arrow_node(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    ann: &Annotation,
    board: &glucose_core::types::Board,
    selected: bool,
    editing: Option<&TextEditSession>,
) {
    draw_arrow_path(ctx, pixmap, ann, board, selected);
    // L'étiquette et le prédicat visent **le même** point du tracé. Quand les deux sont là,
    // l'une monte et l'autre descend : sinon ils se recouvriraient exactement.
    let porte_les_deux = matches!(
        ann,
        Annotation::Arrow {
            text: Some(_),
            predicate: Some(_),
            ..
        }
    );
    super::arrow_label::draw_arrow_label(ctx, pixmap, board, ann, editing, porte_les_deux);
    super::predicate::draw_arrow_predicate(ctx, pixmap, board, ann, porte_les_deux);
}

/// Trace une flèche, tronçon par tronçon.
///
/// Le tracé vient de [`glucose_core::arrow::path_in`], celui-là même que l'arbitre de clic
/// interroge : une flèche ancrée s'arrête sur le bord du nœud qu'elle vise, et elle se
/// **vise** là où elle se dessine. Deux endroits qui recalculeraient cette forme finiraient
/// par en dessiner deux différentes — c'est ce que la rotation a coûté (ARROW-1).
fn draw_arrow_path(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    ann: &Annotation,
    board: &glucose_core::types::Board,
    selected: bool,
) {
    let Some(points) = glucose_core::arrow::path_in(ann, board) else {
        return;
    };
    let dernier = points.len().saturating_sub(2);
    for (i, segment) in points.windows(2).enumerate() {
        // Une polyligne ne porte qu'une pointe, à son dernier tronçon : les coudes sont des
        // passages, pas des arrivées.
        draw_arrow(ctx, pixmap, segment[0], segment[1], selected, i == dernier);
    }
    // Les poignées de coude n'apparaissent que sur une flèche sélectionnée (ARROW-3) : une
    // affordance appartient à ce qu'on manipule, et les montrer toutes couvrirait le
    // canevas de disques.
    if selected {
        super::handles::draw_arrow_handles(pixmap, ctx.theme, ann, board, &ctx.vp);
    }
}
