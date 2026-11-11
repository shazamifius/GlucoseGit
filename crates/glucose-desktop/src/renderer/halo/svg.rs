//! **Les lueurs en SVG** (P5, fiche 70) : la sœur de [`super::draw_halos`] — chaque lueur
//! visible, toutes **avant** les cartes, comme au canevas (LUEUR-3) : la lueur d'une carte
//! passe sous sa voisine, jamais dessus.
//!
//! La géométrie est celle du rendu ([`super::halo_geometry`]), la couleur et la sûreté celles de
//! l'IA ([`crate::renderer::sens`]), l'opacité celle de l'éclat.
//!
//! **Les coins sont ceux du canevas** : l'ombre séparable, carrée aux coins (HALO-1), et non
//! l'ombre arrondie de Glucose Tauri. Le SVG la disait exactement jusqu'au 11/11/2026 ; mais un
//! export montre ce que le canevas montre, et la lueur, deux fois plus forte depuis PARENTE-2
//! (76 au lieu de 38), rendait l'écart des coins aussi grand qu'une teinte faussée de douze
//! niveaux — l'épreuve de superposition ne distinguait plus l'un de l'autre (fiche 106). Sa
//! découpe dans la carte (LUEUR-1) est implicite : la brume de la carte, qui passe par-dessus,
//! est opaque.

use super::{halo_geometry, portee_du_flou, Eclat};
use crate::params::ViewPass;
use crate::renderer::svg::{hex, Svg};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;

/// **Écrit la lueur de chaque carte visible.**
pub(in crate::renderer) fn lueurs_en_svg(
    (store, pass, kit): (&Store, ViewPass<'_>, crate::renderer::PaintKit<'_>),
    taille: (u32, u32),
    svg: &mut Svg,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let ecran = (taille.0 as f32, taille.1 as f32, pass.header_h);
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        // Un export n'a rien de désigné : la même loi rend chaque lueur au repos.
        let eclat = Eclat::de(ann, &[]);
        // La boîte que la carte montre, comme au canevas (fiche 100) ; un export n'édite rien.
        let mesure = (kit.typography, kit.math, kit.mises_en_page);
        let Some(boite) = crate::renderer::card::boite_montree(mesure, ann, None) else {
            continue;
        };
        let (rgb, surete) = kit.sens.lueur(ann);
        let Some(halo) = halo_geometry(boite, (&pass.vp, pass.echelle()), ecran, eclat) else {
            continue;
        };
        let boite = (
            halo.left,
            halo.top,
            halo.right - halo.left,
            halo.bottom - halo.top,
        );
        let opacite = f32::from(eclat.alpha(surete)) / 255.0;
        let portee = portee_du_flou(halo.sigma, eclat.alpha(surete));
        svg.lueur(boite, (&hex(rgb), opacite), (halo.sigma, portee));
    }
}
