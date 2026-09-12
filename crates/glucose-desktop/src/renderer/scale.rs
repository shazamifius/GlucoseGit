//! L'unique mise à l'échelle du monde vers l'écran (standard § 4.4).
//!
//! # SCALE-1 — une seule transformation, jamais une borne par valeur
//!
//! Tout ce qui appartient au monde — cadre, police, marges, rayons, puces, curseur — passe
//! par [`WorldScale::world`], et par elle seule. Une borne posée sur la police sans borne
//! équivalente sur la boîte fait déborder le texte ; six bornes à six seuils cassent la mise
//! en page six fois, à six niveaux de zoom différents. C'est le constat **R-45**, et le code
//! TypeScript d'origine documentait déjà le piège (`HtmlAnnotationLayer.tsx:203-210`) :
//!
//! > *« Une transformation d'échelle plutôt qu'une largeur divisée : réduire la boîte sans
//! > réduire la police ferait déborder le texte, alors que `scale` emporte tout d'un coup —
//! > cadre, police, marges, badges. »*
//!
//! La seule exception admise est [`WorldScale::screen`], la contrepartie du `1 / scale` que
//! le TSX applique à ses guides : ce qui doit garder une **taille écran constante** ne passe
//! pas par la mise à l'échelle, il est écrit en pixels et ne bouge pas.
//!
//! # SCALE-2 — un seul niveau de détail, décidé pour l'élément entier
//!
//! Si une borne de lisibilité est souhaitée, elle porte sur la transformation entière et
//! nulle part ailleurs : sous [`WorldScale::SIMPLIFIED_BELOW`], un élément se dessine
//! **simplifié** — son cadre, sans son texte. C'est un niveau de détail assumé, nommé et
//! testé, pas six bornes qui se déclenchent chacune dans son coin.

/// Le facteur de zoom d'une passe de rendu, et la seule façon d'en dériver une longueur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldScale(f32);

impl WorldScale {
    /// Échelle sous laquelle un élément se dessine simplifié, sans son texte (SCALE-2).
    ///
    /// Le corps d'une carte mesure 14 unités monde : à ce seuil il vaut 2,8 px à l'écran,
    /// soit moins que l'épaisseur d'un jambage. En deçà, le glyphe ne transporte plus
    /// d'information — le rastériser remplirait le cache de tailles que personne ne lit,
    /// et le composer coûterait une passe de mélange pour un gris uniforme.
    pub const SIMPLIFIED_BELOW: f32 = 0.2;

    /// Adopte le zoom d'un viewport.
    ///
    /// Un zoom non fini ou négatif est ramené à zéro : rien ne se dessine, ce qui vaut
    /// infiniment mieux que de laisser un `NaN` atteindre la rastérisation, où il devient
    /// une taille de glyphe aberrante et une boucle de plusieurs centaines de millions
    /// d'itérations.
    pub(super) fn new(scale: f64) -> Self {
        if scale.is_finite() && scale > 0.0 {
            Self(scale as f32)
        } else {
            Self(0.0)
        }
    }

    /// Met une longueur exprimée en **unités monde** à l'échelle de l'écran.
    pub fn world(self, length: f32) -> f32 {
        length * self.0
    }

    /// Une longueur qui garde une **taille écran constante** quel que soit le zoom.
    ///
    /// C'est l'exception de SCALE-1, réservée aux affordances : poignées, anneau de
    /// sélection, traits d'un pixel. Le passage par cette fonction est délibéré — il rend
    /// l'exception visible au point d'appel au lieu d'un littéral muet.
    pub fn screen(self, pixels: f32) -> f32 {
        let _ = self;
        pixels
    }

    /// L'élément se dessine-t-il avec son texte, ou simplifié (SCALE-2) ?
    pub(super) fn draws_detail(self) -> bool {
        self.0 >= Self::SIMPLIFIED_BELOW
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_1_all_world_lengths_share_one_factor() {
        // La preuve de SCALE-1 : quel que soit le zoom, le rapport entre deux longueurs
        // du monde est celui qu'elles ont dans le monde. Aucune borne ne peut s'y glisser.
        for zoom in [0.05_f64, 0.25, 0.5, 1.0, 2.0, 4.0, 20.0] {
            let s = WorldScale::new(zoom);
            let (box_w, font, pad, radius) = (260.0_f32, 14.0, 18.0, 24.0);
            for length in [font, pad, radius] {
                let expected = length / box_w;
                let observed = s.world(length) / s.world(box_w);
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : {length} / {box_w} vaut {observed} au lieu de {expected}"
                );
            }
        }
    }

    #[test]
    fn test_scale_1_screen_lengths_never_follow_the_zoom() {
        for zoom in [0.25_f64, 1.0, 8.0] {
            assert_eq!(WorldScale::new(zoom).screen(2.0), 2.0);
        }
    }

    #[test]
    fn test_scale_1_a_degenerate_zoom_draws_nothing() {
        for bad in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
            let s = WorldScale::new(bad);
            assert_eq!(s.world(1.0), 0.0, "zoom {bad}");
            assert_eq!(s.world(14.0), 0.0);
            assert!(!s.draws_detail());
        }
    }

    #[test]
    fn test_scale_2_the_level_of_detail_is_one_named_threshold() {
        assert!(!WorldScale::new(0.19).draws_detail());
        assert!(WorldScale::new(0.2).draws_detail());
        assert!(WorldScale::new(0.25).draws_detail(), "le zoom 0,25 garde son texte");
    }
}
