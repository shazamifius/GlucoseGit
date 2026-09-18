//! Ce qu'une scène d'images va coûter, et par quel chemin chacune passera (COUT-1).
//!
//! # Pourquoi prévoir plutôt que constater
//!
//! Décider de la finesse d'après l'image précédente, c'est dégrader **après** avoir raté, puis
//! redevenir fin après avoir dégradé : la scène oscille, et l'œil voit l'oscillation. Prévoir
//! décide avant, donc une seule fois, et l'aspect ne change que lorsque le contenu change.
//!
//! Rien n'est dessiné ici et rien n'est modifié : la consultation des vignettes est sans effet
//! de bord, sans quoi prévoir changerait ce qu'on prévoit.

use super::super::super::magasin::Magasin;
use super::super::super::photo;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use glucose_core::cout::{Cout, Finesse, Travail};
use glucose_core::occlusion::{self, Calque};

/// Par quel chemin une photo a été posée. C'est ce qui explique son coût, et rien d'autre ne
/// le dit : les trois diffèrent d'un facteur dix, et la durée seule les confond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Chemin {
    /// Une vignette prête : un pixel pour un pixel, le chemin le moins cher.
    Vignette,
    /// Le rééchantillonnage depuis un niveau de pyramide.
    Echantillon,
    /// Le rastériseur général, pour une image tournée.
    Tournee,
}

impl Chemin {
    pub(super) const TOUS: [Chemin; 3] = [Chemin::Vignette, Chemin::Echantillon, Chemin::Tournee];

    pub(super) fn indice(self) -> usize {
        match self {
            Chemin::Vignette => 0,
            Chemin::Echantillon => 1,
            Chemin::Tournee => 2,
        }
    }

    /// La nature de travail que ce chemin représente pour le modèle de coût.
    ///
    /// Le chemin tourné n'en a aucune : il passe par le rastériseur général, dont le coût ne
    /// se ramène pas au pixel écrit — sa boîte n'est même pas ce qu'il couvre.
    pub(super) fn travail(self, finesse: Finesse) -> Option<Travail> {
        match (self, finesse) {
            (Chemin::Vignette, _) => Some(Travail::PixelRepris),
            (Chemin::Echantillon, Finesse::Lisse) => Some(Travail::PixelLisse),
            (Chemin::Echantillon, Finesse::Pixelisee) => Some(Travail::PixelPixelise),
            (Chemin::Tournee, _) => None,
        }
    }
}

/// Ce que la scène coûtera si on la dessine **au mieux**, ou `None` si la machine ne l'a pas
/// encore démontré.
///
/// # Pourquoi on prévoit le cas lisse et pas le cas dégradé
///
/// La décision porte sur la question « peut-on se permettre le mieux ? ». Prévoir le coût
/// dégradé pour décider s'il faut dégrader tournerait en rond.
///
/// Rien n'est dessiné ici, et rien n'est modifié : la consultation des vignettes est sans
/// effet de bord, sans quoi prévoir changerait ce qu'on prévoit.
pub(super) fn prevoir_la_scene(
    visibles: &[&glucose_core::types::BoardImage],
    caches: &occlusion::Visibles,
    pass: &ViewPass<'_>,
    magasin: &Magasin,
    cout: &Cout,
) -> Option<std::time::Duration> {
    let (mut repris, mut lisses) = (0u64, 0u64);
    for (rang, img) in visibles.iter().enumerate() {
        let parts = caches.parts(rang);
        if parts.is_empty() || img.rotation != 0.0 {
            continue;
        }
        let surface: f64 = parts
            .iter()
            .map(|b| f64::from(b.largeur) * f64::from(b.hauteur))
            .sum();
        let (sx, sy, sw, sh) = boite_ecran(img, pass);
        let forme = photo::Forme::posee(sx, sy, sw, sh);
        if magasin.vignettes.a_prete(&img.id, forme) {
            repris += surface as u64;
        } else {
            lisses += surface as u64;
        }
    }
    cout.prevoir_tout(&[
        (Travail::PixelRepris, repris),
        (Travail::PixelLisse, lisses),
    ])
}

/// Ce que le noyau a besoin de savoir d'une image pour decider si elle se voit.
///
/// Trois conditions font qu'une image en cache une autre, et toutes se **constatent** :
///
/// * elle est **opaque** -- la pyramide l'a verifie pixel par pixel au decodage ;
/// * elle n'est pas **tournee** -- sinon sa boite n'est plus ce qu'elle couvre, et les coins
///   laisseraient voir dessous ;
/// * elle est **decodee** -- une image en chemin se dessine comme un cadre, a travers lequel
///   le fond se voit.
pub(super) fn calque_de(
    img: &glucose_core::types::BoardImage,
    pass: &ViewPass<'_>,
    magasin: &Magasin,
) -> Calque {
    let (sx, sy, sw, sh) = boite_ecran(img, pass);
    let opaque = img.rotation == 0.0
        && img
            .src
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|src| magasin.cache.get(src))
            .is_some_and(|e| e.pyramide.opaque());
    Calque {
        boite: occlusion::Boite::nouvelle(sx, sy, sw, sh),
        opaque,
    }
}

/// La boite ecran d'une image : son coin haut-gauche et sa taille.
pub(super) fn boite_ecran(
    img: &glucose_core::types::BoardImage,
    pass: &ViewPass<'_>,
) -> (f32, f32, f32, f32) {
    let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
    (
        wx as f32,
        wy as f32,
        (img.width * pass.vp.scale) as f32,
        (img.height * pass.vp.scale) as f32,
    )
}
