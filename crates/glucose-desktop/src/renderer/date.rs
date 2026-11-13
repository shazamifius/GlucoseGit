//! **DATE-1 — la date d'un nœud, à l'écran** (fiche 113) : la pastille de Glucose Tauri
//! (`TemporalBadge`), en bas à droite du nœud et débordant de son coin — l'année, la plage ou
//! l'ère, à l'encre ambre du temps.
//!
//! * **Une marque du monde** (SCALE-1) : ses mesures sont celles de Tauri à l'échelle 1, et la
//!   pastille suit le zoom en bloc, comme la réglette des domaines ; de loin, son mot devient le
//!   gris de son encre (GRIS-2) — elle ne se tait jamais, c'est du sens.
//! * **Sur les deux voies, par construction** (ORNEMENTS-1) : elle se pose dans
//!   [`super::voies::dessiner_sur_les_photos`], la moitié que les deux voies partagent, après les
//!   nœuds — au-dessus de tout ce qu'elle date, comme dans Tauri. Sur la carte graphique, c'est la
//!   couche du dessus, qui part à l'écran par les bandes qu'elle a écrites (BANDE-1) : une date
//!   paraît même sans rien de choisi (`tests/voies_suite.rs`).
//! * **Dans l'export SVG, au même rang** : le dessin est écrit une fois sur une [`Surface`].
//! * **Sa place** est la boîte du nœud que le noyau donne aux flèches (`arrow::emprise`) : une
//!   seule loi pour dire où est un nœud.

use super::push_rounded_rect;
use super::scale::WorldScale;
use super::surface::{Surface, Trait};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle};
use glucose_core::arrow::emprise;
use glucose_core::geometry::Rect;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::timeline::format_anchor;
use glucose_core::types::{TemporalAnchor, Viewport};
use tiny_skia::PathBuilder;

/// Les mesures de la pastille, en unités du monde : celles de Tauri (18 de haut, coins en
/// demi-cercle, 6 de marge, un corps de 10, un filet de 1,5), et son débord de 10 au-delà du
/// coin du nœud.
const HAUTEUR: f32 = 18.0;
const MARGE: f32 = 6.0;
const CORPS: f32 = 10.0;
const FILET: f32 = 1.5;
const DEBORD: f32 = 10.0;

/// **Pose la date de chaque nœud visible qui en porte une** — textes, post-it, flèches,
/// membranes, puis images — sur une surface de `ecran` pixels : les pixels du canevas, ou le SVG
/// d'un export, **le même tracé** des deux côtés (P5, fiche 70).
pub(super) fn dessiner_les_dates(
    surface: &mut impl Surface,
    theme: &Theme,
    (store, pass): (&Store, ViewPass<'_>),
    ecran: (f32, f32),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let visibles = Visibles::nouvelles(pass.visibles, board);
    let vue = (&pass.vp, pass.echelle(), ecran);
    for ann in visibles.annotations() {
        if let (Some(date), Some(boite)) = (ann.temporal_anchor(), emprise(board, ann.id())) {
            dessiner_la_date(surface, theme, vue, boite, date);
        }
    }
    for img in visibles.images() {
        if let Some(date) = &img.temporal_anchor {
            dessiner_la_date(surface, theme, vue, img.rect(), date);
        }
    }
}

/// **Pose les dates sur les pixels du canevas**, sur les deux voies : la moitié qu'elles partagent
/// ([`super::voies::dessiner_sur_les_photos`]) l'appelle après les nœuds.
pub(super) fn poser_les_dates(
    pixmap: &mut tiny_skia::PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, super::PaintKit<'_>),
) {
    let ecran = (pixmap.width() as f32, pixmap.height() as f32);
    let mut pixels = super::surface::Pixels {
        pixmap,
        typographie: kit.typography,
    };
    dessiner_les_dates(&mut pixels, kit.theme, (store, pass), ecran);
}

/// **Pose la date de ce nœud**, dont `boite` est la place dans le monde ; rien si la pastille
/// tombe hors de l'écran.
fn dessiner_la_date(
    surface: &mut impl Surface,
    theme: &Theme,
    (vp, echelle, (l, h)): (&Viewport, WorldScale, (f32, f32)),
    boite: Rect,
    date: &TemporalAnchor,
) {
    let mot = format_anchor(date);
    let corps = echelle.world(CORPS);
    let (largeur_du_mot, hauteur_du_mot) =
        surface
            .typographie()
            .measure_text(&mot, corps, Face::Regular);
    let (largeur, hauteur) = (
        largeur_du_mot + echelle.world(2.0 * MARGE),
        echelle.world(HAUTEUR),
    );
    let (droite, bas) = world_to_screen(boite.left + boite.width, boite.top + boite.height, vp);
    let x = droite as f32 + echelle.world(DEBORD) - largeur;
    let y = bas as f32 + echelle.world(DEBORD) - hauteur;
    if x > l || y > h || x + largeur < 0.0 || y + hauteur < 0.0 {
        return;
    }
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, x, y, largeur, hauteur, hauteur / 2.0);
    let Some(pilule) = pb.finish() else {
        return;
    };
    surface.remplir(&pilule, theme.temps.fond);
    surface.tracer(
        &pilule,
        theme.temps.filet,
        Trait::plein(echelle.world(FILET)),
    );
    let style = TextStyle {
        size: corps,
        color: theme.temps.encre,
        face: Face::Regular,
    };
    let at = (
        x + (largeur - largeur_du_mot) / 2.0,
        y + (hauteur - hauteur_du_mot) / 2.0,
    );
    surface.ecrire(at, &mot, style);
}

#[cfg(test)]
mod tests;
