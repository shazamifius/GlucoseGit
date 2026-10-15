//! **Les lueurs en SVG** (P5, fiche 70) : la sœur de [`super::draw_halos`] — chaque lueur
//! visible, toutes **avant** les cartes, comme au canevas (LUEUR-3) : la lueur d'une carte
//! passe sous sa voisine, jamais dessus.
//!
//! La géométrie est celle du rendu ([`super::halo_geometry`]), la teinte celle du cache des
//! teintes symbiotiques, l'opacité celle de l'éclat. Le SVG dit l'ombre de boîte de Glucose
//! Tauri exactement — la boîte dilatée, ses coins arrondis du rayon de la carte plus la
//! dilatation, floutée d'une gaussienne — là où le rendu approxime les coins droits pour aller
//! vite (HALO-1). Sa découpe dans la carte (LUEUR-1) est implicite : la brume de la carte, qui
//! passe par-dessus, est opaque.

use super::{halo_geometry, Eclat};
use crate::params::ViewPass;
use crate::renderer::hue::SymbioticHueCache;
use crate::renderer::svg::{hex, Svg};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;

/// **Écrit la lueur de chaque carte visible.**
pub(in crate::renderer) fn lueurs_en_svg(
    hue_cache: &mut SymbioticHueCache,
    (store, pass): (&Store, ViewPass<'_>),
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
        let Some(halo) = halo_geometry(ann, (&pass.vp, pass.echelle()), ecran, eclat) else {
            continue;
        };
        let (_, rgb) = hue_cache.get_or_compute(ann, pass.index, board);
        let rayon = halo.carte.map_or(0.0, |c| c.rayon + (c.gauche - halo.left));
        let boite = (
            halo.left,
            halo.top,
            halo.right - halo.left,
            halo.bottom - halo.top,
        );
        let opacite = f32::from(eclat.alpha()) / 255.0;
        svg.lueur(boite, rayon, (&hex(rgb), opacite), halo.sigma);
    }
}
