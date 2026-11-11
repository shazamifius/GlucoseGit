//! **PARENTE-2 — le halo d'un texte dit sa parenté** (fiche 106) : ce que l'IA dit de chaque texte
//! ([`crate::parente`]), en couleurs.
//!
//! Sa règle (VISION § 5) : des textes qui ont un rapport ont des couleurs proches ; si l'IA n'est
//! pas sûre, pas de liaison ; *« et du coup le système d'opacité et de niveaux de blanc ou noir
//! prend sens »*. La grammaire est celle des membranes — une seule pour tout Glucose :
//!
//! * **la teinte** dit la famille : celle que la sériation lui donne sur le cercle, la famille du
//!   premier texte du document prenant la teinte du bleu de Tauri ;
//! * **la clarté** est celle du bleu de Tauri, la clarté d'une membrane sans couleur (NEUTRE-1).
//!   Elle dira l'accessibilité (sa parole du 11/11/2026) quand l'IA saura la lire ; d'ici là elle
//!   ne varie pas, et ne dit donc rien ;
//! * **la vivacité** dit la sûreté : du gris d'une membrane sans couleur, pour un texte dont l'IA
//!   n'est pas sûre (*« pas de liaison de couleur »*), au plus grand cercle de teintes que l'écran
//!   montre à cette clarté, pour un texte sûr — toutes les familles également vives, aucune ne
//!   crie parce que l'écran la montre mieux ;
//! * **un texte seul** prend le gris d'une membrane sans couleur ;
//! * **l'opacité** dit aussi la sûreté — de la lueur d'un bloc à la lueur sûre, le halo la peint
//!   ([`super::halo::Eclat::alpha`]) ;
//! * **la couleur qu'on a choisie** pour un texte passe devant, sûre (COULEUR-4 : *« le halo
//!   lumineux autour du bloc — PAS son background ! »*).
//!
//! **Une seule fonction pour tout ce qui lit la couleur d'un texte** — sa lueur, ses ornements, ses
//! passages qui brillent ([`SensDesTextes::lueur`]) : elle s'écrivait en deux copies, et la lueur
//! d'un passage prenait une autre teinte que sa carte.

use crate::parente::Avis;
use glucose_core::couleur::Oklch;
use glucose_core::types::Annotation;
use std::collections::HashMap;

/// Une couleur sRGB, en octets.
type Rgb = (u8, u8, u8);

/// **Le sens des textes du tableau, en couleurs** : la couleur de chacun et la sûreté qu'elle porte
/// — et le fil qui les calcule, comme le magasin tient ceux qui décodent les images.
pub struct SensDesTextes {
    par_texte: HashMap<String, (Rgb, f32)>,
    /// Le gris d'un texte seul, ou que l'IA n'a pas encore lu.
    gris: Rgb,
    /// Ce que l'IA dit des textes, et le fil qui le calcule (PARENTE-2).
    pub(crate) parente: crate::parente::Parente,
    /// **Ce qu'elle a dit, tel quel**, et de quel tableau : un agent le lit (PARENTE-3) — ce que
    /// l'humain voit, une IA doit pouvoir le lire.
    avis: Option<crate::parente::AvisDuTableau>,
}

impl Default for SensDesTextes {
    fn default() -> Self {
        Self {
            par_texte: HashMap::new(),
            gris: super::scene::membranes::sans_couleur(),
            parente: Default::default(),
            avis: None,
        }
    }
}

impl SensDesTextes {
    /// **Ce que l'IA vient de dire**, s'il y a du neuf, se pose ; rend vrai si les couleurs ont
    /// pu changer.
    pub(crate) fn suivre(&mut self, store: &glucose_core::store::Store) -> bool {
        let Some(avis) = self.parente.suivre(store) else {
            return false;
        };
        self.poser(&avis.1);
        self.avis = Some(avis);
        true
    }

    /// **L'avis de l'IA sur les textes de ce tableau**, s'il est celui qu'elle a lu.
    pub(crate) fn avis_du_tableau(&self, tableau: &str) -> Option<&crate::parente::Avis> {
        self.avis
            .as_ref()
            .filter(|(t, _)| t == tableau)
            .map(|(_, a)| a)
    }

    /// **Pose ce que l'IA vient de dire** : chaque teinte sur le cercle de la clarté du bleu de
    /// Tauri, tournée pour que la famille du premier texte prenne sa teinte.
    pub(crate) fn poser(&mut self, avis: &Avis) {
        let bleu = Oklch::des_octets(super::scene::membranes::BLEU_DE_TAURI);
        let vivacite = Oklch::cercle_des_teintes(bleu.l);
        self.par_texte = avis
            .iter()
            .map(|(id, a)| {
                let couleur = a.teinte.map_or(self.gris, |t| {
                    Oklch {
                        l: bleu.l,
                        c: vivacite * f64::from(a.surete.clamp(0.0, 1.0)),
                        h: (t + bleu.h).rem_euclid(std::f64::consts::TAU),
                    }
                    .octets()
                });
                (id.clone(), (couleur, a.surete))
            })
            .collect();
    }

    /// **La lueur d'un texte** : sa couleur, et la sûreté qu'elle porte — pleine pour la couleur
    /// qu'on a choisie, celle de l'IA sinon, nulle pour un texte qu'elle n'a pas lu (gris).
    pub(crate) fn lueur(&self, ann: &Annotation) -> (Rgb, f32) {
        let choisie = match ann {
            Annotation::Text { color: Some(c), .. } => super::parse_hex_rgb(c),
            _ => None,
        };
        choisie.map_or_else(
            || {
                self.par_texte
                    .get(ann.id())
                    .copied()
                    .unwrap_or((self.gris, 0.0))
            },
            |c| (c, 1.0),
        )
    }

    /// La couleur d'un texte, sans sa sûreté : ses ornements, ses passages qui brillent, le bout
    /// d'une flèche qui le touche.
    pub(crate) fn teinte(&self, ann: &Annotation) -> Rgb {
        self.lueur(ann).0
    }

    /// Le gris de ce dont l'IA ne dit rien.
    pub(crate) fn gris(&self) -> Rgb {
        self.gris
    }
}

/// Pour une épreuve qui regarde la lueur et non l'IA : chaque texte du tableau actif sûr, dans la
/// famille de la teinte d'origine.
#[cfg(test)]
impl SensDesTextes {
    pub(crate) fn tous_surs(store: &glucose_core::store::Store) -> Self {
        let avis = store
            .active_board()
            .map(|b| {
                glucose_core::sens::textes_du_tableau(b)
                    .map(|(id, _)| {
                        let sur = crate::parente::AvisDUnTexte {
                            famille: Some(0),
                            teinte: Some(0.0),
                            surete: 1.0,
                        };
                        (id.to_string(), sur)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut sens = Self::default();
        sens.poser(&avis);
        sens
    }
}

#[cfg(test)]
mod tests;
