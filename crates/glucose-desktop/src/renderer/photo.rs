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

use glucose_core::occlusion::Boite;
use glucose_core::report;
use tiny_skia::Pixmap;

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

    /// Sa largeur en pixels d'écran. L'atelier s'en sert pour estimer ce qu'elle coûtera.
    pub fn largeur(self) -> u32 {
        self.largeur
    }

    /// Sa hauteur en pixels d'écran.
    pub fn hauteur(self) -> u32 {
        self.hauteur
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
    /// Fonde la pyramide sur une image décodée, **entièrement construite**.
    ///
    /// # Pourquoi tous les niveaux, et pas à la demande
    ///
    /// Les niveaux se construisaient au premier besoin, donc **dans l'image qui en avait
    /// besoin**. Poser une photo de téléphone à quatre cents pixels demandait quatre
    /// réductions successives — seize mégapixels à moyenner — au milieu d'une frame. Mesuré :
    /// **58 ms pour une seule photo**, alors même que le décodage était déjà parti sur un fil
    /// de fond. Le travail avait changé de nom, pas de place.
    ///
    /// Tout construire d'avance coûte environ **un tiers de plus** que le niveau natif :
    /// chaque réduction divise la surface par quatre, et la série `1 + 1/4 + 1/16 + ...`
    /// converge vers `4/3`. Ce n'est pas une estimation, c'est une somme géométrique — donc
    /// il n'y a **aucun seuil à choisir**, ni en nombre de niveaux, ni en taille minimale.
    ///
    /// La borne est asymptotique : l'arrondi des côtés vers le haut ajoute un surcoût en
    /// `O(largeur + hauteur)`, invisible sur une photo et sensible seulement sur une image de
    /// quelques pixels, où il se compte en dizaines d'octets.
    ///
    /// Et cela vaut pour le zoom aussi : la molette ne construit plus rien, elle choisit.
    ///
    /// L'opacité se constate au passage, dans le même parcours.
    pub fn nouvelle(native: Pixmap) -> Self {
        let opaque = native.data().as_chunks::<4>().0.iter().all(|p| p[3] == 255);
        let mut niveaux = vec![native];
        // La descente s'arrête d'elle-même à une image de 1 × 1 : la largeur et la hauteur
        // sont divisées par deux en arrondissant vers le haut, donc elles atteignent 1 et n'en
        // bougent plus.
        while let Some(dernier) = niveaux.last() {
            if dernier.width() <= 1 && dernier.height() <= 1 {
                break;
            }
            let reduit = reduire_par_deux(dernier);
            niveaux.push(reduit);
        }
        Self { niveaux, opaque }
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
    /// Les niveaux existent tous dès la construction : cette méthode **choisit**, elle ne
    /// fabrique rien. C'est ce qui la rend utilisable en pleine frame, et en `&self`.
    pub fn niveau_pour(&self, largeur_ecran: f32) -> &Pixmap {
        let voulue = largeur_ecran.max(1.0);
        self.niveaux
            .iter()
            .take_while(|n| n.width() as f32 >= voulue || n.width() <= 1)
            .last()
            .unwrap_or_else(|| self.native())
    }

    /// Rééchantillonne l'image à la forme voulue, phase comprise.
    ///
    /// La phase entre ici, et non au moment de poser : c'est elle qui fait la netteté, et la
    /// reporter au moment de poser obligerait à une position fractionnaire, donc au pipeline
    /// générique qu'on cherche justement à éviter.
    pub fn rendre(&self, forme: Forme) -> Pixmap {
        let Some(mut vignette) = Self::vignette_vide(forme) else {
            return Pixmap::new(1, 1).expect("un pixel tient toujours en memoire");
        };
        let hauteur = vignette.height();
        self.rendre_bande(forme, &mut vignette, 0, hauteur);
        vignette
    }

    /// Le tampon d'une vignette, vide, aux dimensions que `forme` demande.
    ///
    /// Une phase non nulle déborde d'un pixel sur la droite et le bas : la vignette est donc
    /// d'un pixel plus grande, et ce pixel porte la part de l'image qui dépasse.
    ///
    /// Rend `None` quand l'allocation échoue. Ce point portait un `expect` dont le message
    /// parlait d'une taille NULLE, alors que le cas réel est l'inverse : en zoom proche, la
    /// largeur écran d'une photo atteint des dizaines de milliers de pixels, et la vignette
    /// demandait trente-deux gigaoctets. L'application plantait là, sur un message qui ne
    /// désignait ni la cause ni l'endroit.
    pub fn vignette_vide(forme: Forme) -> Option<Pixmap> {
        Pixmap::new(forme.largeur + 1, forme.hauteur + 1)
    }

    /// Remplit les lignes `y0..y1` d'une vignette déjà allouée (CASCADE-1).
    ///
    /// # Pourquoi une vignette se construit par bandes
    ///
    /// La construire d'un bloc était ce qui gelait une image : quatre-vingt-neuf vignettes
    /// demandées au même instant coûtaient 471 ms, mesurées chez l'utilisateur. Étaler ne
    /// suffit pas si la plus petite tranche indivisible reste une vignette entière — il faut
    /// que le travail se **découpe**, sans quoi une seule tranche peut encore faire rater
    /// l'image.
    ///
    /// Le grain devient donc la ligne, qui coûte quelques microsecondes. Et ce découpage ne
    /// coûte rien à écrire : pour le report, une bande n'est qu'un clip, et le clip est son
    /// domaine d'itération.
    pub fn rendre_bande(&self, forme: Forme, dest: &mut Pixmap, y0: u32, y1: u32) {
        let source = self.niveau_pour(forme.largeur as f32);
        let (texels, _) = source.data().as_chunks::<4>();
        let Some(vue) = report::Vue::nouvelle(texels, source.width(), source.height()) else {
            return;
        };
        let (largeur, hauteur) = (dest.width(), dest.height());
        let (pixels, _) = dest.data_mut().as_chunks_mut::<4>();
        let Some(mut cible) = report::VueMut::nouvelle(pixels, largeur, hauteur) else {
            return;
        };
        report::reporter(
            &mut cible,
            &vue,
            report::Pose {
                x: forme.phase_x(),
                y: forme.phase_y(),
                largeur: forme.largeur as f32,
                hauteur: forme.hauteur as f32,
            },
            Boite::nouvelle(0.0, y0 as f32, largeur as f32, y1.saturating_sub(y0) as f32),
            report::Melange::Remplacer,
            // Une vignette se construit toujours au mieux : elle servira des dizaines
            // d'images, et la pixeliser une fois la pixeliserait pour toutes.
            report::Filtre::Lisse,
        );
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
