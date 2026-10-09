//! **Ce qui attend une décision, confié à la carte après les panneaux** (DECISION-1, fiche 59).
//!
//! Sur la voie graphique, les panneaux sont des textures que la carte pose **après** la couche
//! du dessus (PANNEAUX-1). Ce qui attend une décision — la barre d'action, le menu, la question —
//! se peignait dans cette couche : il passait donc **sous** les panneaux, et la question du
//! journal technique se cachait derrière Ordonner et Pomodoro (son téléphone, le 07/10).
//!
//! Sans panneau ouvert — l'état du lancement —, rien ne change : ce groupe va dans la couche du
//! dessus, et ne coûte rien de plus. Avec un panneau, il se peint ici, dans un tampon à lui, et
//! la carte le pose après les panneaux. Seules les lignes qu'il porte partent, relevées comme
//! celles de la couche du dessus (BANDE-1) ; et sa clé ne change que si ses pixels changent :
//! une question immobile ne repart pas sur le bus à chaque image.

use crate::present::bandes::Bandes;
use crate::present::scene_gpu::Pose;
use crate::renderer::voies::APoser;
use tiny_skia::{Pixmap, PixmapMut};

/// Ce que la carte reconnaît : toujours la même chose, à des moments différents.
const IDENTITE: &str = "decision";

/// La clé d'une génération : une nouvelle dès que les pixels changent.
fn cle(generation: usize) -> String {
    format!("{IDENTITE}@{generation}")
}

/// Le tampon de ce qui attend une décision, et ce que la carte en détient.
#[derive(Default)]
pub struct Decision {
    tampon: Option<Pixmap>,
    /// Les lignes que l'image précédente y a écrites : seules celles-là s'effacent.
    ecrites: Bandes,
    /// La tranche que la carte pose, et le haut où elle se pose.
    posee: Option<(u32, Pixmap)>,
    generation: usize,
}

impl Decision {
    /// **Peint ce qui attend une décision** dans le tampon, et dit ce que la carte doit poser
    /// — rien si le dessin n'a rien écrit.
    pub fn peindre(
        &mut self,
        (largeur, hauteur): (u32, u32),
        dessiner: impl FnOnce(&mut PixmapMut),
    ) -> Option<APoser> {
        let tampon = self
            .tampon
            .take()
            .filter(|t| t.width() == largeur && t.height() == hauteur);
        let mut tampon = match tampon {
            Some(t) => t,
            // Un tampon neuf est transparent partout : il n'a rien à effacer.
            None => {
                self.ecrites = Bandes::default();
                Pixmap::new(largeur, hauteur)?
            }
        };
        self.ecrites.effacer(&mut tampon);
        dessiner(&mut tampon.as_mut());
        self.ecrites = Bandes::relever(&tampon);
        let pose = self.trancher(&tampon);
        self.tampon = Some(tampon);
        pose
    }

    /// Plus rien n'est posé d'ici : sans panneau, la couche du dessus porte ce groupe.
    pub fn retirer(&mut self) {
        self.posee = None;
    }

    /// Les pixels de la tranche que cette clé désigne, si c'est la tranche posée.
    pub fn pixels(&self, demandee: &str) -> Option<Pixmap> {
        let (_, pixels) = self.posee.as_ref()?;
        (demandee == cle(self.generation)).then(|| pixels.clone())
    }

    /// **La tranche qui porte de l'encre**, de sa première ligne à sa dernière, et une
    /// génération nouvelle seulement si elle a changé.
    fn trancher(&mut self, tampon: &Pixmap) -> Option<APoser> {
        let intervalles = self.ecrites.intervalles();
        let (Some(premiere), Some(derniere)) = (intervalles.first(), intervalles.last()) else {
            self.posee = None;
            return None;
        };
        let (haut, bas) = (premiere.start, derniere.end);
        let rang = tampon.width() as usize * 4;
        let octets = &tampon.data()[haut as usize * rang..bas as usize * rang];
        let inchangee = self
            .posee
            .as_ref()
            .is_some_and(|(h, p)| *h == haut && p.data() == octets);
        if !inchangee {
            let mut pixels = Pixmap::new(tampon.width(), bas - haut)?;
            pixels.data_mut().copy_from_slice(octets);
            self.generation += 1;
            self.posee = Some((haut, pixels));
        }
        Some(APoser {
            identite: IDENTITE.to_string(),
            cle: cle(self.generation),
            pose: Pose {
                x: 0.0,
                y: haut as f32,
                largeur: tampon.width() as f32,
                hauteur: (bas - haut) as f32,
                opacite: 1.0,
                angle: 0.0,
                fenetre: Pose::TOUT,
                bornes: Pose::PARTOUT,
            },
            repli: None,
        })
    }
}

#[cfg(test)]
mod tests;
