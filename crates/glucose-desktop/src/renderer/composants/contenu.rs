//! **Ce qu'un composant montre** : une carte de texte, ou le cadre d'une photo en chemin — et
//! l'empreinte qui dit quand ses pixels changent.
//!
//! Extrait de [`super`] quand le découpage des composants plus grands que l'écran (DE-PRES-1) y
//! est entré : six cent sept lignes là où la fiche 05 en admet six cents. La coupure tombe là
//! où la question change — ici *ce qu'on montre*, là-bas *comment on le pose*.

use std::hash::{Hash, Hasher};

/// Ce qu'un composant montre, et ce qu'il faut pour le dessiner.
#[derive(Debug, Clone)]
pub(super) enum Contenu {
    Carte {
        origine: (f64, f64),
        taille: (f32, f32),
        corps: String,
        teinte: (u8, u8, u8),
        /// Le fond que la carte peint sous son texte (LUEUR-3) : le mode Focus le teinte, et la
        /// texture change avec lui.
        fond: (u8, u8, u8),
        /// La saisie en cours sur cette carte, **figee** (COMPOSANT-2).
        ///
        /// Une carte qu'on edite changeait a chaque image parce que personne ne s'etait
        /// demande a quelle frequence elle change VRAIMENT : son texte bouge a la frappe,
        /// son curseur deux fois par seconde, et rien d'autre. Le terrain du 22/09 la
        /// chiffre a 9,74 ms en median sur le geste « editer du texte », dont l'image
        /// mediane coute 19,48 ms -- cinquante et une images par seconde pendant qu'on
        /// ecrit, la ou la charte en demande cent.
        edition: Option<crate::renderer::TextEditSession>,
    },
    PhotoEnChemin {
        id: String,
        /// La taille de la photo **dans le monde** : celle de rendu s'en déduit à chaque
        /// échelle par [`cadre_en_chemin`], y compris celle du repli, qui n'est pas la même.
        taille: (f64, f64),
        /// En radians, autour du centre — l'unité du modèle.
        rotation: f64,
    },
}

/// **La taille d'une photo en chemin à l'échelle `echelle`, et la boîte englobante de son
/// cadre tourné**, en pixels.
///
/// Une seule fonction pour la mesure et pour le rendu : l'englobante était gardée telle quelle
/// parce que la recalculer depuis la texture arrondie décalait le cadre d'un pixel, et un
/// cadre anti-crénelé décalé d'un pixel n'a plus un bord en commun avec lui-même. La calculer
/// deux fois par la même expression donne les mêmes bits.
pub(super) fn cadre_en_chemin(
    taille: (f64, f64),
    rotation: f64,
    echelle: f64,
) -> ((f32, f32), (f32, f32)) {
    let rendu = ((taille.0 * echelle) as f32, (taille.1 * echelle) as f32);
    let (cos, sin) = (rotation.cos().abs() as f32, rotation.sin().abs() as f32);
    let englobante = (rendu.0 * cos + rendu.1 * sin, rendu.0 * sin + rendu.1 * cos);
    (rendu, englobante)
}

impl Contenu {
    pub(super) fn hacher(&self, h: &mut impl Hasher) {
        match self {
            Self::Carte {
                taille,
                corps,
                teinte,
                fond,
                edition,
                ..
            } => {
                0u8.hash(h);
                corps.hash(h);
                taille.0.to_bits().hash(h);
                taille.1.to_bits().hash(h);
                teinte.hash(h);
                fond.hash(h);
                // **Pas la selection** (COMPOSANT-4) : l'anneau qui la montre se pose au-dessus,
                // et selectionner une carte ne refait plus sa texture.
                // **Ce que la saisie change, et rien d'autre.** Le texte est deja dans
                // `corps` -- `carte_de` y met le tampon d'edition. Reste l'etendue
                // selectionnee, que la texture peint sous le texte ; **pas le curseur**, qui
                // se pose au-dessus (COMPOSANT-3). Une selection vide ne peint rien : ni sa
                // place ni la phase du curseur n'entrent dans la cle, et ni un clignotement ni
                // une fleche du clavier ne refont la texture.
                //
                // `goal_x` et `blink_timer` n'y sont pas non plus : ils decident de ce que le
                // curseur fera, jamais de ce que la texture montre.
                match edition {
                    Some(e) => {
                        1u8.hash(h);
                        if !e.selection.is_empty() {
                            e.selection.range().hash(h);
                        }
                    }
                    None => 0u8.hash(h),
                }
            }
            Self::PhotoEnChemin {
                id,
                taille,
                rotation,
            } => {
                1u8.hash(h);
                id.hash(h);
                taille.0.to_bits().hash(h);
                taille.1.to_bits().hash(h);
                rotation.to_bits().hash(h);
            }
        }
    }
}
