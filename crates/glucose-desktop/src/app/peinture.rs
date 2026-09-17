//! Peindre une image : ce qu'on redessine, et ce qu'on se garde de redessiner (A.1).
//!
//! # Trois issues, et une seule decision
//!
//! Une image demandee ne veut pas dire une image a peindre. Selon ce que la salissure dit,
//! trois chemins s'ouvrent :
//!
//! * **rien** -- l'image precedente est encore exacte, on se contente de la representer ;
//! * **une region** -- seule une zone a change, et elle ne touche pas la chrome : on redessine
//!   cette zone seule, dans une image a sa taille, puis on la reporte ;
//! * **tout** -- le cas par defaut, celui d'hier, et celui de tout ce qu'on ne sait pas encore
//!   localiser.
//!
//! La regle de surete ne bouge jamais : **on ne reduit le travail que quand on peut le
//! prouver**. Redessiner tout coute ce que ca coutait ; oublier une zone laisse un morceau
//! d'image perime a l'ecran, et c'est le pire defaut possible -- il se voit, et on ne sait pas
//! d'ou il vient.
//!
//! # Pourquoi la chrome force le rendu complet
//!
//! La scene suit la vue : la decaler de `-x0` deplace l'origine de l'ecran d'autant, et aucune
//! passe n'a besoin de le savoir. La chrome, elle, se place sur la taille de la fenetre -- un
//! bandeau en haut, une minimap en bas. Elle ne sait pas se rendre decalee, donc une zone sale
//! qui la touche retombe sur le chemin complet.
//!
//! C'est aussi pourquoi les deux ont ete separees : elles ne se salissent pas dans le meme
//! repere -- la scene en coordonnees monde, la chrome en coordonnees ecran.

use super::GlucoseApp;
use crate::dock::{render_docks, DockCache, DockManager, DockPass};
use crate::params::{Pointer, SceneOverlay, ScreenFrame};
use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::store::Store;
use tiny_skia::Pixmap;

impl GlucoseApp {
    /// Redessine ce qui doit l'etre, et rien de plus. Ne presente pas.
    ///
    /// Le tampon est **sorti** de l'application le temps de la peinture : sans cela, peindre
    /// emprunterait `self` en ecriture par le tampon et en lecture par le document, et il
    /// faudrait passer chaque champ un par un pour convaincre le compilateur.
    pub(super) fn peindre_ce_qui_a_change(&mut self, fenetre: (u32, u32), tampon_neuf: bool) {
        // La salissure est **consommee** : ce qui est redessine maintenant cesse d'etre sale,
        // et une nouvelle demande arrivee pendant le rendu appartient a l'image suivante.
        let sale = self.salissure.replace(crate::salissure::Salissure::Rien);
        // Un tampon neuf ne contient rien : il n'y a aucune image precedente a menager.
        let sale = if tampon_neuf {
            crate::salissure::Salissure::Tout
        } else {
            sale
        };
        crate::perf::compteur("img_evitee", f64::from(u8::from(sale.est_propre())));
        if sale.est_propre() {
            return;
        }

        let Some(mut pixmap) = self.pixmap.take() else {
            return;
        };
        let header_h = self.ui.header_height();
        let vp = self
            .store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or_default();
        let overlay = SceneOverlay {
            guides: &self.active_guides,
            selection_box: self.selection_box,
            editing: self.editing_session.as_ref(),
        };
        let pointer = Pointer {
            x: self.mouse_pos.0 as f32,
            y: self.mouse_pos.1 as f32,
        };
        let region = sale
            .region(&vp, fenetre, debord_des_passes(vp.scale))
            .filter(|r| !r.touche_le_haut(header_h));

        match region {
            Some(r) => {
                if !r.est_vide() {
                    repeindre_la_region(
                        &mut pixmap,
                        r,
                        &mut self.renderer,
                        &self.store,
                        &self.ui,
                        overlay,
                    );
                }
                crate::perf::compteur("img_region", r.aire() as f64);
            }
            None => {
                crate::perf::compteur("img_region", f64::from(fenetre.0) * f64::from(fenetre.1));
                let echelle = self.ui.scale_factor;
                peindre_tout(
                    &mut pixmap,
                    &mut self.renderer,
                    &self.store,
                    &mut self.ui,
                    &self.dock_manager,
                    &self.dock_cache,
                    overlay,
                    pointer,
                    echelle,
                );
            }
        }
        self.pixmap = Some(pixmap);
    }
}

    /// Ce qu'une passe peut dessiner **au-delà** du rectangle d'un nœud, en pixels.
///
/// C'est la portée du flou des halos, seule passe qui déborde vraiment : `bench_zone` a
/// mesuré que l'écart entre un rendu par région et un rendu complet s'éteint entre seize
/// et quarante-huit pixels à l'échelle 1, ce qui est exactement cette valeur.
///
/// Elle ne se choisit donc pas : elle se lit sur le modèle de halo et suit le zoom, comme
/// le halo lui-même. Le double couvre le flou de part et d'autre de la bordure.
fn debord_des_passes(echelle: f64) -> f32 {
    (crate::renderer::halo::HALO_SPREAD as f64 * echelle * 2.0) as f32
}

/// Redessine **une région** de la scène, et la reporte dans le tampon (A.1).
///
/// La scène se rend dans une image à la taille de la région, la vue décalée d'autant, puis
/// le résultat se recopie en place. `bench_zone` mesure que ces pixels sont identiques au
/// bit près à ceux d'un rendu complet.
///
/// La chrome n'y figure pas, et c'est l'appelant qui s'en assure : elle se place en
/// coordonnées écran et ne sait pas se rendre décalée. Une région qui la toucherait
/// retombe sur le rendu complet.
fn repeindre_la_region(
    pixmap: &mut Pixmap,
    region: crate::salissure::Region,
    renderer: &mut Renderer,
    store: &Store,
    ui: &UiState,
    overlay: SceneOverlay<'_>,
) {
    let Some(mut morceau) = Pixmap::new(region.largeur, region.hauteur) else {
        return;
    };
    let origine = (region.x as f32, region.y as f32);
    renderer.rendre_la_region(
        &mut morceau.as_mut(),
        store,
        ui,
        overlay,
        ui.header_height() - origine.1,
        origine,
    );
    // `Source` et non `SourceOver` : on REMPLACE les pixels périmés, on ne compose pas
    // par-dessus. Composer redoublerait tout ce qui n'est pas opaque.
    pixmap.draw_pixmap(
        region.x as i32,
        region.y as i32,
        morceau.as_ref(),
        &tiny_skia::PixmapPaint {
            blend_mode: tiny_skia::BlendMode::Source,
            ..Default::default()
        },
        tiny_skia::Transform::identity(),
        None,
    );
    crate::perf::stage("region");
}

/// Peint la scène et la chrome dans le tampon.
///
/// Extraite de `redraw` parce que celle-ci a désormais une décision à prendre avant de
/// peindre -- y a-t-il seulement quelque chose à redessiner -- et qu'un ordonnanceur qui
/// peint aussi finit par ne plus laisser voir la décision.
#[allow(clippy::too_many_arguments)]
fn peindre_tout(
    pixmap: &mut Pixmap,
    renderer: &mut Renderer,
    store: &Store,
    ui: &mut UiState,
    dock_manager: &DockManager,
    dock_cache: &DockCache,
    overlay: SceneOverlay<'_>,
    pointer: Pointer,
    scale: f32,
) {
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut vue = pixmap.as_mut();
    renderer.render(&mut vue, store, ui, overlay, pointer);

    // Rendu des panneaux déroulants & flottants (Top & Bottom Docks).
    // `scale` et les coordonnées de la souris sont désormais portés par
    // deux types distincts : les intervertir ne compile plus (R-44).
    render_docks(
        &mut vue,
        dock_manager,
        store,
        &DockPass {
            typo: &renderer.typography,
            theme: &renderer.theme,
            screen: ScreenFrame {
                width: width as f32,
                height: height as f32,
                header_h: ui.header_height(),
                scale,
            },
            pointer,
            cache: Some(dock_cache),
        },
    );
    crate::perf::stage("docks");
}

