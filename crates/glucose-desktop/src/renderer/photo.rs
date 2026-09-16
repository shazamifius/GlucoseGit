//! Les images décodées, et leurs réductions successives par deux.
//!
//! # Invariant MIP-1 — on ne réduit jamais une image de 1:10 à la volée
//!
//! Une photo de téléphone fait 4032 × 3024 ; posée sur le canevas, elle occupe quelques
//! centaines de pixels. Le rendu la réduisait donc d'un facteur dix **à chaque image**, en
//! bilinéaire : pour chaque pixel produit, quatre texels lus à dix pixels d'intervalle dans
//! quarante-huit mégaoctets. Mesuré sur trente-six photos en 2560 × 1600 :
//!
//! ```text
//!   réduite à la volée, bilinéaire     47,4 ms   9,95 ns/px
//!   réduite à la volée, au plus proche 17,9 ms   3,75 ns/px
//!   source déjà à la bonne taille      17,2 ms   3,61 ns/px
//! ```
//!
//! Le filtre ne coûte cher que parce qu'il lit loin. Une source déjà proche de la taille
//! d'affichage le rend presque gratuit — et donne au passage une **meilleure** image : réduire
//! de 1:10 en une fois n'échantillonne que quatre texels sur cent, ce qui est de l'aliasing,
//! là où des réductions par deux successives font la moyenne de tous.
//!
//! # Pourquoi par deux, et pas à la taille exacte
//!
//! Réduire à la taille exacte obligerait à tout refaire au moindre cran de zoom — c'est-à-dire
//! pendant tout un geste de molette, au pire moment. Une pyramide dyadique ne dépend pas du
//! zoom : le niveau choisi change par sauts, et le facteur résiduel reste entre 1/2 et 1, là
//! où le filtre lit des texels voisins.
//!
//! Les niveaux se construisent **à la demande**, un par un, depuis celui du dessus. Une image
//! qu'on ne regarde que de loin ne paie que le chemin qui y mène, et une image qu'on ne
//! regarde jamais ne paie rien.
//!
//! # Invariant MIP-2 — une taille qui se répète se paie une fois
//!
//! La pyramide seule ne suffit pas, et la mesure l'a dit sans ambiguïté. Le rasteriseur n'a un
//! chemin rapide que pour une échelle **exactement** égale à 1 posée sur un **entier** ; tout
//! le reste passe par son pipeline générique :
//!
//! ```text
//!   échelle 1, position entière        19,2 ms
//!   échelle 0,998                      50,8 ms
//!   échelle 0,83 (ce que la pyramide laisse)  41,1 ms
//!   échelle 1, décalée d'un demi-pixel 35,4 ms
//! ```
//!
//! Une image garde donc aussi sa **vignette exacte** : la taille où elle est posée, phase
//! sous-pixel comprise, prête à être reportée sans transformation. C'est ce que la typographie
//! fait depuis toujours pour ses glyphes (GLYPH-1) — la phase se lit sur la position écran, et
//! seule la cellule entière se translate.
//!
//! Ce module sait **fabriquer** une telle vignette ([`Pyramide::rendre`]) ; c'est
//! [`super::vignette`] qui décide quand elle en vaut la peine et qui la garde, parce que sa
//! taille et sa phase appartiennent au nœud du canevas, non au fichier.

use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

/// La forme exacte sous laquelle une image est posée à l'écran : sa taille en pixels, et la
/// partie fractionnaire de sa position.
///
/// Deux images posées à la même taille mais à des phases différentes ne sont pas les mêmes
/// pixels — d'où la phase dans la clé. Elle est comparée **par ses bits** : deux `f32` égaux
/// ont les mêmes bits, et c'est la seule comparaison qui ait un sens ici, puisqu'un seuil
/// serait arbitraire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Forme {
    largeur: u32,
    hauteur: u32,
    phase: (u32, u32),
}

impl Forme {
    /// La forme d'une image posée en `(x, y)` et mesurant `(w, h)` pixels écran.
    pub fn posee(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            largeur: (w.round() as i64).clamp(1, u32::MAX as i64) as u32,
            hauteur: (h.round() as i64).clamp(1, u32::MAX as i64) as u32,
            phase: ((x - x.floor()).to_bits(), (y - y.floor()).to_bits()),
        }
    }

    fn phase_x(self) -> f32 {
        f32::from_bits(self.phase.0)
    }

    fn phase_y(self) -> f32 {
        f32::from_bits(self.phase.1)
    }
}

/// Une image décodée et ses réductions successives par deux.
///
/// Le niveau `0` est la résolution native ; le niveau `k` a ses deux côtés divisés par `2^k`,
/// arrondis vers le haut pour ne jamais tomber à zéro.
pub struct Pyramide {
    niveaux: Vec<Pixmap>,
    opaque: bool,
}

impl Pyramide {
    /// Fonde la pyramide sur une image décodée. Aucun niveau réduit n'est construit ici.
    ///
    /// L'opacité se constate une fois, au décodage : c'est un parcours des octets alpha, du
    /// même ordre que la conversion en prémultiplié qui vient de l'être faite. La constater à
    /// chaque image, en revanche, coûterait autant que le dessin lui-même.
    pub fn nouvelle(native: Pixmap) -> Self {
        let opaque = native.data().as_chunks::<4>().0.iter().all(|p| p[3] == 255);
        Self {
            niveaux: vec![native],
            opaque,
        }
    }

    /// Vrai si aucun pixel de la source n'est translucide.
    ///
    /// Une image opaque n'a rien à mélanger avec le fond : « source par-dessus » calcule alors
    /// un mélange dont le résultat est la source. Le rasteriseur ne le devine pas, et le
    /// remplacement pur lui économise une lecture et une multiplication par pixel — mesuré à
    /// 16,5 ms contre 13,8 sur trente-six images.
    ///
    /// Une réduction par deux préserve l'opacité : la moyenne de valeurs toutes égales à 255
    /// vaut 255. Le drapeau constaté sur le niveau natif vaut donc pour toute la pyramide.
    pub fn opaque(&self) -> bool {
        self.opaque
    }

    /// La résolution native.
    pub fn native(&self) -> &Pixmap {
        &self.niveaux[0]
    }

    /// Le niveau le plus économique dont la largeur couvre encore `largeur_ecran`.
    ///
    /// « Couvre encore » veut dire : au moins aussi large que ce qu'on va dessiner. Choisir
    /// plus petit reviendrait à agrandir, donc à perdre du détail que la source avait.
    ///
    /// La descente s'arrête d'elle-même à une image de 1 × 1 : elle ne dépend d'aucune borne
    /// posée à la main.
    pub fn niveau_pour(&mut self, largeur_ecran: f32) -> &Pixmap {
        let voulue = largeur_ecran.max(1.0);
        while self.dernier().width() as f32 >= voulue * 2.0 && self.dernier().width() > 1 {
            let reduit = reduire_par_deux(self.dernier());
            self.niveaux.push(reduit);
        }
        // Le dernier niveau construit est, par la boucle ci-dessus, le plus petit qui reste au
        // moins aussi large que voulu — ou le plus petit possible.
        self.dernier()
    }

    fn dernier(&self) -> &Pixmap {
        self.niveaux
            .last()
            .expect("une pyramide a son niveau natif")
    }

    /// Rééchantillonne l'image à la forme voulue, phase comprise.
    ///
    /// La phase entre ici, et non au moment de poser : c'est elle qui fait la netteté, et la
    /// reporter au moment de poser obligerait à une position fractionnaire, donc au pipeline
    /// générique qu'on cherche justement à éviter.
    pub fn rendre(&mut self, forme: Forme) -> Pixmap {
        // Une phase non nulle déborde d'un pixel sur la droite et le bas : la vignette est donc
        // dessinée un pixel plus grande, et ce pixel porte la part de l'image qui dépasse.
        let (w, h) = (forme.largeur + 1, forme.hauteur + 1);
        let source = self.niveau_pour(forme.largeur as f32);
        let echelle = (
            forme.largeur as f32 / source.width() as f32,
            forme.hauteur as f32 / source.height() as f32,
        );
        let transforme = Transform::from_scale(echelle.0, echelle.1)
            .post_translate(forme.phase_x(), forme.phase_y());

        let mut vignette = Pixmap::new(w, h).expect("une vignette a une taille non nulle");
        vignette.draw_pixmap(
            0,
            0,
            source.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..Default::default()
            },
            transforme,
            None,
        );
        vignette
    }

    /// Le nombre de niveaux construits. Rend la construction paresseuse observable.
    pub fn niveaux_construits(&self) -> usize {
        self.niveaux.len()
    }

    /// Les octets que cette pyramide occupe, niveau natif compris.
    pub fn octets(&self) -> usize {
        self.niveaux
            .iter()
            .map(|n| n.width() as usize * n.height() as usize * 4)
            .sum()
    }
}

/// Réduit une image de moitié sur chaque côté, en moyennant les quatre pixels d'un bloc.
///
/// Les côtés impairs sont arrondis vers le haut, et le bloc est alors tronqué : le dernier
/// pixel d'une ligne impaire n'a pas de voisin de droite, et la moyenne porte sur ce qui
/// existe. Aucune borne, aucun cas particulier écrit à la main.
///
/// La moyenne se fait sur les octets tels quels, donc sur des valeurs prémultipliées en sRGB.
/// C'est ce que fait déjà le filtre bilinéaire du rasteriseur : le niveau réduit et la
/// réduction à la volée sont ainsi cohérents entre eux, ce qui est ce qui compte ici.
fn reduire_par_deux(source: &Pixmap) -> Pixmap {
    let (sw, sh) = (source.width(), source.height());
    let (dw, dh) = (sw.div_ceil(2).max(1), sh.div_ceil(2).max(1));
    let mut sortie = Pixmap::new(dw, dh).expect("une réduction a une taille non nulle");

    let src = source.data();
    let dst = sortie.data_mut();
    for y in 0..dh {
        for x in 0..dw {
            let mut somme = [0u32; 4];
            let mut compte = 0u32;
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (sx, sy) = (x * 2 + ox, y * 2 + oy);
                if sx >= sw || sy >= sh {
                    continue;
                }
                let i = ((sy * sw + sx) * 4) as usize;
                for c in 0..4 {
                    somme[c] += src[i + c] as u32;
                }
                compte += 1;
            }
            let j = ((y * dw + x) * 4) as usize;
            for c in 0..4 {
                dst[j + c] = (somme[c] / compte) as u8;
            }
        }
    }
    sortie
}

#[cfg(test)]
mod tests;
