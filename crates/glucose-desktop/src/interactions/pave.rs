//! **Le pavé tactile tel que le doigt le fait** (fiche 53) : le pincement et le déplacement à deux
//! doigts, reçus par *Direct Manipulation* et non plus traduits en molette.
//!
//! # Ce qui n'allait pas, et que trois réglages n'ont pas réparé
//!
//! Une application qui ne prend pas le pavé de précision le reçoit sous la forme que Windows
//! donne à tout le monde : des défilements de molette, `Ctrl` en plus pour un pincement
//! ([`super::pincement`]). Ces paquets arrivent au rythme que Windows choisit, quantifiés, et ont
//! déjà perdu l'échelle du geste. Le 07/10, avec la conduite et la glissade il était « trop
//! smooth » ; tout direct, « strate par strate » ; conduit sans glissade, « légèrement trop
//! lent, et pas fluide, comme s'il sautait ». Aucun gain ni aucun lissage ne rend une continuité
//! perdue en route — alors que dans PureRef ou un navigateur, le pincement est « instantané, de
//! manière fluide ».
//!
//! # Ce que font Chromium et Blender
//!
//! Ils prennent le pavé par *Direct Manipulation* (`direct_manipulation_helper_win.cc`,
//! `GHOST_TrackpadWin32.cc`) : un *viewport* fictif, que le système fait glisser et grandir
//! comme le doigt le fait, avec son point de départ, sa cadence et son inertie. L'application
//! lit sa transformation à chaque image et en tire ce qui a changé depuis la précédente.
//!
//! Ce module est la **lecture** de cette transformation — pure, éprouvable partout — et ce que
//! Glucose en fait. La voie de Windows, le COM, vit dans `plateforme::pave_windows`.
//!
//! # Deux règles, empruntées et vérifiées
//!
//! * **Un pincement le reste jusqu'à la fin du geste.** Pendant qu'il agrandit, le système
//!   déplace aussi le contenu pour garder le point entre les doigts : ce déplacement-là n'est
//!   pas un geste, et Blender le dit « absurde ». Le zoom se fait autour du curseur, qui ne
//!   bouge pas pendant qu'on pince.
//! * **Un déplacement peut devenir un pincement** : le système ne reconnaît pas toujours l'écart
//!   des doigts dès le premier instant (Blender, encore).
//!
//! Ce que Glucose en fait passe par la porte de la souris ([`super::elan::Elan::placer_pan`]) :
//! montré en entier à l'image suivante. Le lissage et l'inertie sont ceux du système — ceux
//! d'Edge et de Chrome —, et le déplacement à deux doigts garde son élan parce que le système le
//! lui donne. Le pincement n'en a pas : il s'arrête avec les doigts.

use crate::app::GlucoseApp;

/// Ce que le pavé a fait depuis l'image précédente.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mouvement {
    /// Le contenu suit les doigts de tant de pixels de la fenêtre.
    Deplacer(f64, f64),
    /// L'échelle change de tant d'octaves, autour du curseur.
    Zoomer(f64),
}

/// **Une source de gestes du pavé** : la voie de Windows, ou celle qu'une épreuve fournit.
pub trait Pave {
    /// Un geste est-il en cours — les doigts posés, ou l'inertie qu'ils ont laissée ? C'est la
    /// seule raison de faire avancer le système : hors geste, Glucose dort.
    fn en_geste(&self) -> bool;
    /// Fait avancer le système d'une image, et rend ce qui a bougé.
    fn avancer(&mut self) -> Vec<Mouvement>;
}

/// **La lecture de la transformation** que le système applique à son contenu : échelle, puis
/// décalage, depuis le début du geste.
#[derive(Debug, Clone, PartialEq)]
pub struct Lecteur {
    dernier: (f32, f32, f32),
    pincement: bool,
}

impl Default for Lecteur {
    fn default() -> Self {
        Self {
            dernier: (1.0, 0.0, 0.0),
            pincement: false,
        }
    }
}

impl Lecteur {
    /// Le geste est fini et le *viewport* revenu à l'identité : la lecture repart de zéro.
    pub fn remettre(&mut self) {
        *self = Self::default();
    }

    /// **Ce qui a changé** depuis la dernière transformation lue : `(échelle, x, y)`.
    pub fn lire(&mut self, (echelle, x, y): (f32, f32, f32)) -> Option<Mouvement> {
        let (avant, ax, ay) = std::mem::replace(&mut self.dernier, (echelle, x, y));
        if !egaux(echelle, avant) && avant > 0.0 && echelle > 0.0 {
            self.pincement = true;
            // Une différence de logarithmes, en `f64` : la somme des octaves d'un geste vaut
            // exactement celles de son échelle finale — un rapport en `f32` en perdait.
            let octaves = f64::from(echelle).log2() - f64::from(avant).log2();
            return Some(Mouvement::Zoomer(octaves));
        }
        if self.pincement {
            return None;
        }
        let (dx, dy) = (f64::from(x - ax), f64::from(y - ay));
        (dx != 0.0 || dy != 0.0).then_some(Mouvement::Deplacer(dx, dy))
    }
}

/// Deux échelles égales **à la précision où le système les donne** : des `f32`. Pas de seuil
/// choisi — l'écart d'un seul arrondi de `f32` n'est pas un pincement.
fn egaux(a: f32, b: f32) -> bool {
    (a - b).abs() <= f32::EPSILON * a.abs().max(b.abs())
}

impl GlucoseApp {
    /// **À chaque passage de la boucle** : si un geste du pavé est en cours, le système avance
    /// d'une image, et ce qu'il rend se montre à l'image qui vient.
    pub(crate) fn suivre_le_pave(&mut self) {
        let Some(pave) = self.pave.as_mut() else {
            return;
        };
        if !pave.en_geste() {
            return;
        }
        let mouvements = pave.avancer();
        for mouvement in &mouvements {
            self.appliquer_le_pave(*mouvement);
        }
        if !mouvements.is_empty() {
            self.mark_dirty();
        }
    }

    /// Un mouvement du pavé, montré en entier à l'image suivante : le système l'a déjà lissé.
    fn appliquer_le_pave(&mut self, mouvement: Mouvement) {
        use crate::chronique::navigation::Decision;
        self.vol.poser();
        self.defilement_au_doigt = true;
        match mouvement {
            Mouvement::Zoomer(octaves) => {
                self.chronique.navigation.evenement(Decision::Pincement);
                // Au mode référence, `Alt` + pincer agrandit la fenêtre (REFERENCE-2).
                if !self.redimensionner_au_pincement(octaves) {
                    let ancre = self.ancre_du_zoom();
                    self.elan.placer_zoom(octaves, ancre);
                }
            }
            Mouvement::Deplacer(dx, dy) => {
                self.chronique.navigation.evenement(Decision::Pan);
                self.elan.placer_pan(dx, dy);
            }
        }
    }
}

#[cfg(test)]
mod tests;
