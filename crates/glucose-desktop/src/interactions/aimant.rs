//! **L'aimant ne regarde que ses voisines, et ce que l'écran montre** (SNAP-4).
//!
//! Son retour du 29/09 : *« le système actuel de l'aimant est assez chiant, même pour les
//! créateurs : quand on a trop de nœuds, c'est juste invivable pour positionner ce qu'on
//! souhaite »* — et, dézoomé, glisser ne doit pas ralentir parce qu'on voit beaucoup. Il a
//! demandé les deux ensemble : **ce qu'on voit à l'écran**, et **une limitation** aux plus
//! pertinents, les proches.
//!
//! Les deux se tiennent sans aucun nombre choisi :
//!
//! * **l'écran** borne ce qu'on demande à l'index — la colonne et la rangée du rectangle,
//!   coupées au bord de la fenêtre ; ce qui n'est pas à l'écran n'aimante pas, et un guide ne
//!   montre jamais une ligne dont on ne voit pas la raison ;
//! * **les voisines** ([`Lignes::des_voisines`]) — la plus proche dans chacune des quatre
//!   directions — sont les seules qui aimantent : quatre boîtes au plus, douze lignes par axe,
//!   quel que soit le nombre de nœuds à l'écran.
//!
//! Le coût d'un mouvement est celui de deux requêtes de l'index sur une bande de l'écran : il
//! ne dépend ni du document, ni de ce qu'on voit hors de la colonne et de la rangée.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::quadtree::noeud_au_rang;
use glucose_core::smart_align::{AlignRect, Lignes, SnapOptions};
use std::collections::HashSet;

impl GlucoseApp {
    /// Les options de l'aimant à l'échelle présente : son seuil est en pixels logiques (DPI-1).
    pub(crate) fn options_d_aimant(&self) -> SnapOptions {
        SnapOptions {
            scale: self.zoom_logique(),
            ..Default::default()
        }
    }

    /// **Les lignes qui aimantent `rect`** : celles de ses voisines, parmi ce que l'écran montre
    /// dans sa colonne et dans sa rangée, sans ce que le geste emporte (`exclus`).
    pub(crate) fn lignes_d_aimant(&mut self, rect: AlignRect, exclus: &HashSet<String>) -> Lignes {
        let seuil = self.options_d_aimant().seuil();
        let vp = self.store.viewport();
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let (x0, y0) = screen_to_world(0.0, f64::from(self.ui.header_height()), &vp);
        let (x1, y1) = screen_to_world(f64::from(largeur), f64::from(hauteur), &vp);
        // La colonne et la rangée du rectangle, au seuil près, coupées au bord de l'écran.
        let colonne = (
            (rect.left - seuil).max(x0),
            y0,
            (rect.right() + seuil).min(x1),
            y1,
        );
        let rangee = (
            x0,
            (rect.top - seuil).max(y0),
            x1,
            (rect.bottom() + seuil).min(y1),
        );
        let mut rangs = Vec::new();
        for (a, b, c, d) in [colonne, rangee] {
            if a < c && b < d {
                rangs.extend(self.renderer.rangs_du_present(&self.store, (a, b, c, d)));
            }
        }
        rangs.sort_unstable();
        rangs.dedup();
        let Some(board) = self.store.active_board() else {
            return Lignes::AUCUNE;
        };
        // Ce qui a une boîte — photo, carte, note, membrane, dossier ; une flèche n'en a pas —,
        // sans ce que le geste emporte.
        let cibles: Vec<AlignRect> = rangs
            .into_iter()
            .filter_map(|k| noeud_au_rang(board, k))
            .filter(|noeud| !exclus.contains(noeud.id()))
            .filter_map(|noeud| noeud.rect())
            .collect();
        Lignes::des_voisines(rect, &cibles, seuil)
    }
}

#[cfg(test)]
#[path = "aimant_tests.rs"]
mod tests;
