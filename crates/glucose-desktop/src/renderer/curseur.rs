//! **Le curseur d'édition — une seule loi** pour tout ce qui s'écrit sur place : un nœud de texte,
//! un post-it, le titre d'une membrane (fiche 99).
//!
//! Le nœud de texte et le post-it en avaient chacun une copie — la même largeur, la même hauteur,
//! mais l'une posée sur la grille des pixels et l'autre non : un post-it qu'on écrit montrait deux
//! demi-traits gris là où un texte montrait un trait net. Le titre d'une membrane n'en avait
//! **aucune** : on y tapait sans rien voir, et les lettres n'apparaissaient qu'après avoir cliqué
//! ailleurs (sa parole du 9/11/2026, « le système de double-clic fonctionne très bizarrement »).

use super::scale::WorldScale;
use tiny_skia::{Color, PixmapMut, Rect};

/// Sa largeur, en pixels d'écran : une affordance (exception SCALE-1), comme le curseur de
/// n'importe quel éditeur — mise à l'échelle, elle s'effacerait au dézoom.
const LARGEUR: f32 = 2.0;
/// Sa hauteur, en multiples du corps du texte qu'il édite : il mesure ce qu'il édite.
pub(crate) const HAUTEUR: f32 = 1.2;

/// **Pose le curseur** : son coin haut-gauche à l'écran, le corps du texte qu'il édite, son encre.
///
/// Sur la grille (SCALE-3) : un curseur d'un pixel posé à une demi-position devient deux
/// demi-traits gris, et il clignote — donc il attire l'œil sur son propre flou.
pub(crate) fn poser(
    pixmap: &mut PixmapMut,
    (x, haut): (f32, f32),
    corps: f32,
    scale: WorldScale,
    encre: Color,
) {
    if let Some(barre) = Rect::from_xywh(x, haut, scale.screen(LARGEUR), corps * HAUTEUR) {
        super::scale::fill_crisp(pixmap, barre, encre);
    }
}
