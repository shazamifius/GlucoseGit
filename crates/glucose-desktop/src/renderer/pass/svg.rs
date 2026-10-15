//! **Les annotations en SVG** (P5, fiche 69) : la sœur de [`super::draw_annotations`], dans le
//! même ordre — le contenu des cartes d'abord, à leur rang ; puis, dans l'ordre du tableau, ce
//! qui passe au-dessus : les réglettes des cartes, les pense-bêtes et la leur, les flèches,
//! leurs libellés et leurs badges. Rien de ce qui
//! se choisit ou s'édite : un export n'a ni sélection ni saisie.

use super::{carte_de, Clip, Pass};
use crate::params::ViewPass;
use crate::renderer::arrow::{fleche_en_svg, Fleche, NoeudsDuRendu};
use crate::renderer::card::vecteurs::carte_en_vecteurs;
use crate::renderer::card::Contenants;
use crate::renderer::domain::reglette;
use crate::renderer::hue::SymbioticHueCache;
use crate::renderer::note::svg::sticky_en_svg;
use crate::renderer::surface::Vecteurs;
use crate::renderer::svg::Svg;
use crate::renderer::PaintKit;
use glucose_core::arrow;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::Annotation;

/// **Écrit les annotations visibles** de la passe.
pub(in crate::renderer) fn annotations_en_svg(
    hue_cache: &mut SymbioticHueCache,
    (kit, store, pass): (PaintKit<'_>, &Store, ViewPass<'_>),
    taille: (u32, u32),
    svg: &mut Svg,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        mises_en_page: kit.mises_en_page,
        tints: kit.tints,
        theme: kit.theme,
        vp: pass.vp,
        scale: pass.echelle(),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: pass.header_h,
        },
        poignees: false,
    };
    let formes = crate::renderer::scene::formes_des_membranes(store, pass, taille);
    let contenants = Contenants::nouveaux(kit.theme, &formes);
    let visibles = || Visibles::nouvelles(pass.visibles, board).annotations();
    for ann in visibles() {
        if let Some(carte) = carte_de(hue_cache, (ann, store, pass), (&contenants, &[]), None) {
            carte_en_vecteurs(&ctx, carte, svg);
        }
    }
    let noeuds = NoeudsDuRendu {
        board,
        index: Some(pass.index),
        typographie: kit.typography,
        math: kit.math,
        contournement: pass.contournement,
    };
    // La réglette d'un nœud, juste après lui, comme au canevas.
    let reglette = |svg: &mut Svg, (x, y): (f64, f64), ann: &Annotation| {
        let (sx, sy) = crate::canvas::world_to_screen(x, y, &ctx.vp);
        let mut vecteurs = Vecteurs {
            svg,
            typographie: kit.typography,
        };
        let coin = (sx as f32, sy as f32);
        reglette(&mut vecteurs, kit.tints, ctx.scale, coin, ann.domains());
    };
    for ann in visibles() {
        match ann {
            Annotation::Text { x, y, .. } => reglette(svg, (*x, *y), ann),
            Annotation::Sticky { x, y, .. } => {
                sticky_en_svg(&ctx, ann, svg);
                reglette(svg, (*x, *y), ann);
            }
            Annotation::Arrow { predicate, .. } => {
                let Some(fleche) = Fleche::de(hue_cache, ann, (pass, noeuds), false) else {
                    continue;
                };
                fleche_en_svg(&fleche, (&ctx.vp, ctx.scale), svg);
                let Some(milieu) = arrow::milieu_du_trace(&fleche.morceaux) else {
                    continue;
                };
                let etiquette = crate::renderer::arrow_label::montre_une_etiquette(ann, None);
                let (r, g, b) = fleche.teintes.milieu;
                let encre = tiny_skia::Color::from_rgba8(r, g, b, 255);
                let les_deux = etiquette && predicate.is_some();
                crate::renderer::arrow_label::etiquette_en_svg(
                    &ctx,
                    svg,
                    (milieu, encre),
                    ann,
                    les_deux,
                );
                if let Some(centre) =
                    crate::renderer::predicate::centre_du_badge(ann, milieu, etiquette)
                {
                    crate::renderer::predicate::predicat_en_svg(&ctx, svg, centre, ann);
                }
            }
            _ => {}
        }
    }
}
