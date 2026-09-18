//! Ce que l'image doit montrer avant d'etre peinte : le mouvement de la camera, et la
//! finesse a laquelle la scene merite d'etre rendue.
//!
//! Les deux vont ensemble et n'allaient nulle part ailleurs. Le mouvement decide de ce qu'on
//! regarde ; la finesse decide de ce que cela coute. Ils se lisent a la meme seconde de
//! l'image -- juste avant de peindre -- et une boucle de rendu qui les portait en plus du
//! reste ne laissait plus voir sa propre structure.

use super::GlucoseApp;
use tiny_skia::Pixmap;

impl GlucoseApp {
    /// Accorde le tampon de la scene reduite au palier courant. Rend vrai s'il a change.
    ///
    /// Un changement de palier rend l'image precedente inutilisable : elle n'a plus la meme
    /// taille, donc plus rien a dire de ce qui est deja a l'ecran. L'appelant le traite comme
    /// un tampon neuf, ce qui force un rendu complet -- et c'est exact.
    pub(super) fn accorder_le_tampon_reduit(&mut self, largeur: u32, hauteur: u32) -> bool {
        let f = self.resolution.facteur();
        if f == 1 {
            return self.tampon_reduit.take().is_some();
        }
        // Le plafond arrondit vers le haut : une fenetre de 1001 pixels reduite de moitie en
        // demande 501, pas 500 -- sinon la derniere colonne n'aurait pas de source.
        let (w, h) = (largeur.div_ceil(f).max(1), hauteur.div_ceil(f).max(1));
        if self
            .tampon_reduit
            .as_ref()
            .is_some_and(|p| p.width() == w && p.height() == h)
        {
            return false;
        }
        self.tampon_reduit = Pixmap::new(w, h);
        true
    }

    /// Publique pour que les bancs et les tests puissent jouer une image sans fenetre : le
    /// geste ne deplace plus rien tout seul, donc le verifier demande de jouer l'image.
    pub fn appliquer_l_elan(&mut self, largeur: u32, hauteur: u32) {
        let diagonale = f64::from(largeur).hypot(f64::from(hauteur));
        let Some(m) = self.elan.avancer(std::time::Instant::now(), diagonale) else {
            return;
        };
        if m.pan != (0.0, 0.0) {
            self.store.pan(m.pan.0, m.pan.1);
        }
        if m.octaves != 0.0 {
            // Le zoom se dit en octaves et s'applique en facteur : `2^n`, et rien d'autre.
            let facteur = m.octaves.exp2();
            let bornes = crate::interactions::pan_zoom::WHEEL_SCALE_RANGE;
            self.store.zoom(facteur, m.ancre.0, m.ancre.1, bornes);
        }
    }
}
