//! **Rendre pour un export** (EXPORT-PNG-1, fiche 62) : la scène à la vue qu'on lui donne, sans
//! les repères de la navigation, et **entière** — chaque image posée à sa finesse, attendue s'il
//! le faut.
//!
//! À l'écran, le rendu demande les images qui lui manquent sans jamais les attendre
//! (DECODE-1) : l'image suivante les montrera. Un export n'a pas d'image suivante. Il attend
//! donc le chantier du magasin et repasse, tant que la passe a demandé du travail — ce que fait
//! un banc pour sa capture, jusqu'au bout.

use super::{halo, Cadrage, Renderer};
use crate::params::SceneOverlay;
use crate::ui::UiState;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use tiny_skia::PixmapMut;

impl Renderer {
    /// **Rend la scène à cette vue, toutes ses images posées**, sans grille ni rien de ce qui
    /// n'est pas du contenu.
    pub fn rendre_pour_l_export(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &mut UiState,
        vue: Viewport,
    ) {
        let guides = SnapGuides::default();
        let cadrage = Cadrage {
            vue: Some(vue),
            ..Cadrage::plein()
        }
        .sans_reperes();
        loop {
            self.magasin.ouvrir();
            self.synchroniser_les_caches(store, ui, (pixmap.width(), pixmap.height()));
            let overlay = SceneOverlay::sans_rien(&guides);
            self.rendre_la_region(pixmap, store, ui, overlay, 0.0, cadrage);
            self.magasin.fermer();
            if self.magasin.en_travail() == 0 {
                return;
            }
            self.magasin.attendre_le_chantier();
        }
    }
}

impl Renderer {
    /// **Écrit la scène à cette vue en SVG** (P5, fiche 69), de `taille` pixels : chaque sœur
    /// vectorielle des dessins, dans l'ordre même du canevas — le fond, les zones et les cases
    /// du storyboard, les membranes, les images, les cartes, puis ce qui passe au-dessus.
    /// `octets` rend ceux d'une image, s'ils se lisent.
    pub fn rendre_en_svg(
        &mut self,
        (store, ui): (&Store, &mut UiState),
        vue: Viewport,
        taille: (u32, u32),
        octets: &mut dyn FnMut(&glucose_core::types::BoardImage) -> Option<Vec<u8>>,
    ) -> String {
        self.synchroniser_les_caches(store, ui, taille);
        self.densite = 1.0;
        let cadrage = Cadrage {
            vue: Some(vue),
            ..Cadrage::plein()
        }
        .sans_reperes();
        let (vp, rangs, densite) = self.cadrer(store, taille, 0.0, cadrage);
        let pass = crate::params::ViewPass {
            vp,
            visibles: &rangs,
            index: &self.spatial_hash,
            contournement: Some(super::arrow::Contournement::de(
                &self.itineraires,
                &self.suivi_du_geste,
            )),
            header_h: 0.0,
            densite,
        };
        let mut svg = super::svg::Svg::new();
        let (fond, _) = super::svg::couleur(self.theme.bg_canvas);
        let (l, h) = (taille.0 as f32, taille.1 as f32);
        svg.rectangle((0.0, 0.0, l, h), 0.0, (&fond, 1.0));
        for r in super::voies::premier_rang(store, pass, (l, h)) {
            let (c, a) = super::svg::couleur(r.couleur);
            let boite = (r.rect.x(), r.rect.y(), r.rect.width(), r.rect.height());
            svg.rectangle(boite, 0.0, (&c, a));
        }
        super::scene::membranes_en_svg((store, pass, &self.theme), taille, &mut svg);
        super::scene::image::svg::images_en_svg(store, pass, &mut svg, octets);
        let kit = super::PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };
        super::pass::svg::annotations_en_svg(
            &mut self.hue_cache,
            (kit, store, pass),
            taille,
            &mut svg,
        );
        svg.finir(taille)
    }
}

/// **Jusqu'où le contenu déborde de ses boîtes**, en pixels, à cette échelle : la lueur au repos
/// d'une carte — sa dilatation, puis la portée de son flou (HALO-2). Au-delà, aucun pixel ne
/// change : c'est la marge exacte d'un export, sans rien de plus ni de moins.
pub fn debord(echelle: f64) -> f64 {
    f64::from(halo::debord_au_repos(echelle as f32))
}
