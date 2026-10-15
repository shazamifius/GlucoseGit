//! **Un composant plus grand que l'écran, en formes** (DE-PRES-1, fiche 73).
//!
//! Ce qu'il montre se relève à l'échelle **exacte** de la vue — ni palier ni phase : rien ne
//! se repeint, la carte dessine chaque image depuis les formes ([`crate::present::formes_gpu`]).
//! Une carte de texte par son dessin vectoriel, le même que l'export SVG
//! ([`crate::renderer::card::vecteurs`]) ; une photo en chemin par son cadre.

use super::contenu::Contenu;
use super::{Composant, Regime};
use crate::canvas::world_to_screen;
use crate::present::formes_gpu::Primitive;
use crate::renderer::card::vecteurs::carte_en_vecteurs;
use crate::renderer::pass::Pass;
use crate::renderer::scale::WorldScale;
use crate::renderer::scene::image::ornement::cadre_manquant_en_vecteurs;
use crate::renderer::vectoriel::Releve;
use crate::renderer::PaintKit;

impl Composant {
    /// **Les formes de ce composant**, à l'échelle de la vue, sans ce qui ne touche pas l'écran.
    pub(super) fn en_formes(&self, kit: PaintKit<'_>, regime: &Regime) -> Vec<Primitive> {
        let ecran = (regime.ecran.0 as f32, regime.ecran.1 as f32);
        let mut releve = Releve::nouveau(ecran);
        match self.contenu.as_ref() {
            Contenu::Carte { .. } => {
                let ctx = Pass {
                    typography: kit.typography,
                    math: kit.math,
                    tints: kit.tints,
                    theme: kit.theme,
                    vp: regime.vue,
                    scale: WorldScale::new(regime.vue.scale, regime.densite),
                    clip: regime.clip,
                    poignees: true,
                };
                if let Some(carte) = self.contenu.carte() {
                    carte_en_vecteurs(&ctx, carte, &mut releve);
                }
            }
            Contenu::PhotoEnChemin {
                id,
                centre,
                taille,
                rotation,
            } => {
                let (cx, cy) = world_to_screen(centre.0, centre.1, &regime.vue);
                let taille = (
                    (taille.0 * regime.vue.scale) as f32,
                    (taille.1 * regime.vue.scale) as f32,
                );
                let at = (cx as f32 - taille.0 / 2.0, cy as f32 - taille.1 / 2.0);
                cadre_manquant_en_vecteurs(kit, &mut releve, (at, taille), (id, *rotation));
            }
        }
        releve.primitives
    }
}
