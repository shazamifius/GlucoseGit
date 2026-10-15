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
    /// **Écrit la scène à cette vue en SVG** (P5, fiches 69 et 70), de `taille` pixels : chaque
    /// dessin, dans l'ordre même des deux moitiés du canevas ([`super::voies`]) — sous les
    /// photos, les photos, puis ce qui passe dessus. `octets` rend ceux d'une image, s'ils se
    /// lisent.
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
        let kit = super::PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };
        let mut svg = super::svg::Svg::new();
        sous_les_photos_en_svg(kit, &mut svg, (store, pass), taille);
        super::scene::image::svg::images_en_svg(store, pass, &mut svg, octets);
        // Ce qui passe sur les photos (`dessiner_sur_les_photos`) : les lueurs, toutes avant les
        // cartes (LUEUR-3) ; les annotations ; puis les ornements des photos, au-dessus de tout.
        super::halo::svg::lueurs_en_svg(&mut self.hue_cache, (store, pass), taille, &mut svg);
        super::pass::svg::annotations_en_svg(
            &mut self.hue_cache,
            (kit, store, pass),
            taille,
            &mut svg,
        );
        let quoi = (kit.typography, kit.tints);
        super::scene::image::svg::reglettes_des_images_en_svg(quoi, (store, pass), &mut svg);
        svg.finir(taille)
    }
}

/// **Ce qui passe sous les photos**, en SVG — dans l'ordre de `dessiner_sous_les_photos` : le
/// fond, le premier rang (les zones, les cases), les membranes et leurs ornements, les textes
/// des zones et des cases, les dossiers.
fn sous_les_photos_en_svg(
    kit: super::PaintKit<'_>,
    svg: &mut super::svg::Svg,
    (store, pass): (&Store, crate::params::ViewPass<'_>),
    taille: (u32, u32),
) {
    let (fond, _) = super::svg::couleur(kit.theme.bg_canvas);
    let ecran = (taille.0 as f32, taille.1 as f32);
    svg.rectangle((0.0, 0.0, ecran.0, ecran.1), 0.0, (&fond, 1.0));
    let mut vecteurs = super::surface::Vecteurs {
        svg,
        typographie: kit.typography,
    };
    for r in super::voies::premier_rang(store, pass, ecran) {
        super::surface::Surface::remplir_net(&mut vecteurs, r.rect, r.couleur);
    }
    super::scene::membranes_en_svg((kit, store, pass), taille, vecteurs.svg);
    super::zone::textes(&mut vecteurs, (store, pass), ecran);
    super::storyboard::textes(&mut vecteurs, (store, pass), ecran);
    super::folder::dossiers_en_svg(kit, vecteurs.svg, (store, pass), taille);
}

/// **Jusqu'où le contenu déborde de ses boîtes**, en pixels, à cette échelle : la lueur au repos
/// d'une carte — sa dilatation, puis la portée de son flou (HALO-2). Au-delà, aucun pixel ne
/// change : c'est la marge exacte d'un export, sans rien de plus ni de moins.
pub fn debord(echelle: f64) -> f64 {
    f64::from(halo::debord_au_repos(echelle as f32))
}
