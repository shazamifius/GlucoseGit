//! **Le glyphe d'un caractère, et ce qu'on en lit** (EMOJI-1, EMOJI-2 ; fiches 58 et 125).
//!
//! Sept polices, dans l'ordre de [`toutes_les_polices`] : les cinq visages, puis les deux replis
//! des émojis — **Twemoji en couleur** ([`COULEUR`]), puis **ce qui reste de Noto Emoji**
//! ([`RESTE`]) : le cadre de touche U+20E3, les joints et les sélecteurs invisibles, que Twemoji
//! n'a pas. Le visage dessine ce qu'il a ; sinon la couleur ; sinon le reste ; sinon le `.notdef`
//! du visage — un caractère que personne ne dessine se voit, plutôt que de disparaître. Aucune
//! table écrite à la main : la `cmap` de chaque police décide, caractère par caractère.
//!
//! La mesure, le dessin en pixels, le tracé des grandes lettres, le SVG et la carte graphique
//! passent par ce seul choix ([`Typography::choix`]) : ils ne peuvent pas diverger.

use super::trace::{Couches, Glyphe};
use super::{toutes_les_polices, Face, Police, Typography};
use fontdue::{Font, Metrics};
use tiny_skia::{Color, PixmapMut};

/// Le rang de Twemoji en couleur parmi les polices (EMOJI-2).
pub(crate) const COULEUR: usize = Face::ALL.len();
/// Le rang de ce qui reste de Noto Emoji (EMOJI-1, pour ce que Twemoji n'a pas).
pub(crate) const RESTE: usize = COULEUR + 1;

impl Typography {
    /// **Le glyphe qui dessine `ch` dans ce visage** : le rang de sa police, et son indice.
    pub(super) fn choix(&self, ch: char, face: Face) -> Glyphe {
        let indice = self.font(face).lookup_glyph_index(ch);
        if indice != 0 {
            return (face.index(), indice);
        }
        if let Some(indice) = self.contours.indice(COULEUR, ch) {
            return (COULEUR, indice);
        }
        match self.polices.reste.lookup_glyph_index(ch) {
            0 => (face.index(), 0),
            repli => (RESTE, repli),
        }
    }

    /// **La police nommée qui dessine `ch` dans ce visage**, et l'indice de son glyphe : le SVG
    /// écrit le glyphe même que le canevas pose (P5, fiche 70).
    pub fn source(&self, ch: char, face: Face) -> (Police, u16) {
        let (rang, indice) = self.choix(ch, face);
        (toutes_les_polices()[rang], indice)
    }

    /// **Les couches de `ch`** (EMOJI-2) : une seule, à l'encre du texte, pour une lettre ; celles
    /// d'un émoji, chacune de sa couleur. Le rang de chaque glyphe se nomme par
    /// [`toutes_les_polices`].
    pub fn couches(&self, ch: char, face: Face) -> Couches {
        self.contours.couches(self.choix(ch, face))
    }

    /// De combien la plume avance après `ch` — **sans toucher au cache de glyphes**.
    ///
    /// # MEASURE-1 — mesurer n'est pas dessiner
    ///
    /// Le reflux d'un paragraphe demande une largeur par caractère, et rien d'autre. Passer
    /// par [`Self::get_glyph`] pour l'obtenir fait payer un hachage, un emprunt de `RefCell`,
    /// un `Rc::clone` et une écriture du compteur LRU — pour lire un seul `f32` déjà présent
    /// dans les tables de la police. Mesuré sur 89 caractères : **3 415 ns par le cache contre
    /// 1 844 ns en direct**, soit presque le double pour le service le plus fréquent du moteur,
    /// puisqu'il tourne à chaque frame sur chaque carte visible.
    ///
    /// Le dessin avance de la même mesure ([`Self::avance_de`]) : le curseur ne peut pas tomber à
    /// côté du texte.
    pub fn advance(&self, ch: char, size: f32, face: Face) -> f32 {
        self.avance_de(self.choix(ch, face), super::clamp_font_size(size))
    }

    /// L'avance de ce glyphe, à ce corps.
    pub(super) fn avance_de(&self, glyphe: Glyphe, size: f32) -> f32 {
        match self.par_fontdue(glyphe.0) {
            Some(font) => font.metrics_indexed(glyphe.1, size).advance_width,
            None => self.contours.avance(glyphe, size),
        }
    }

    /// Les métriques de ce glyphe, sans son image.
    pub(super) fn metriques(&self, glyphe: Glyphe, size: f32) -> Metrics {
        match self.par_fontdue(glyphe.0) {
            Some(font) => font.metrics_indexed(glyphe.1, size),
            None => self.contours.metriques(glyphe, size),
        }
    }

    /// **L'image de ce glyphe** à ce corps : par `fontdue` pour les visages et le reste de Noto,
    /// par son contour pour Twemoji — dont `fontdue` ne charge pas les couches (EMOJI-2). Celle
    /// d'un émoji est sa silhouette : le gris de loin et le halo d'un titre la lisent.
    pub(super) fn rasteriser(&self, glyphe: Glyphe, size: f32) -> (Metrics, Vec<u8>) {
        match self.par_fontdue(glyphe.0) {
            Some(font) => font.rasterize_indexed(glyphe.1, size),
            None => self.contours.rasteriser(glyphe, size),
        }
    }

    /// La police que `fontdue` lit à ce rang — aucune pour Twemoji.
    fn par_fontdue(&self, rang: usize) -> Option<&Font> {
        match rang {
            COULEUR => None,
            RESTE => Some(&self.polices.reste),
            visage => self.polices.faces.get(visage),
        }
    }

    /// **Trace ce caractère par ses contours** (GRAND-1), couche par couche, et rend son avance.
    pub(super) fn tracer(
        &self,
        pixmap: &mut PixmapMut,
        (ch, size, face): (char, f32, Face),
        plume: (f32, f32),
        couleur: Color,
    ) -> f32 {
        let glyphe = self.choix(ch, face);
        for couche in self.contours.couches(glyphe).iter() {
            self.contours
                .remplir(pixmap, couche.glyphe, plume, size, couche.sur(couleur));
        }
        self.avance_de(glyphe, size)
    }
}
