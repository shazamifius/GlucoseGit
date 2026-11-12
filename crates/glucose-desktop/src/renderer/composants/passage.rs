//! **REPOS-4 — le passage de la vue des textures à l'écran** (fiche 109) : en mouvement près du
//! repos, les textures se composent dans la vue du repos, et la carte graphique les pose par la
//! transformation exacte qui mène de cette vue à l'écran.

use glucose_core::types::Viewport;

/// **D'un point de la vue des textures au même point de l'écran** : `écran = k × point + d`, la
/// transformation exacte entre deux vues, puisque `écran = monde × échelle + vue`
/// ([`crate::canvas::world_to_screen`]). La carte graphique pose la texture par elle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct VersLEcran {
    k: f64,
    d: (f64, f64),
}

impl VersLEcran {
    pub(super) const RIEN: Self = Self {
        k: 1.0,
        d: (0.0, 0.0),
    };

    /// De la vue `depuis` à la vue `vers`.
    pub(super) fn entre(depuis: Viewport, vers: Viewport) -> Self {
        let k = vers.scale / depuis.scale;
        Self {
            k,
            d: (vers.x - depuis.x * k, vers.y - depuis.y * k),
        }
    }

    pub(super) fn poser(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (
            (f64::from(x) * self.k + self.d.0) as f32,
            (f64::from(y) * self.k + self.d.1) as f32,
        )
    }
}

/// **REPOS-4 — la vue du repos sert-elle encore ?** (fiche 109) Tant que l'échelle de la vue reste
/// à une demi-octave de la sienne : c'est la garantie même du palier du mouvement (le plus proche,
/// à un rapport de racine de deux au plus, [`super::palier_dyadique`]), et la texture du repos, elle, est
/// déjà rendue. Au-delà, le palier.
///
/// Sa carte de Wikipédia l'a montré : à chaque cran de molette, l'image du mouvement refaisait au
/// palier presque toutes les textures (186 à 304 sur 385), puis l'image du repos les refaisait
/// exactes — deux fois tout, pour un cran. La translation entière de la fiche 100 en est le cas
/// exact : même échelle, décalage entier, l'image du repos au pixel près.
pub(super) fn le_repos_sert(repos: Viewport, vue: Viewport) -> bool {
    (vue.scale / repos.scale).log2().abs() <= 0.5
}
