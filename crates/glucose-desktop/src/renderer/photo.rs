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
//!
//! # Invariant ETAGES-1 — on ne lit que ce qui est tenu (fiche 32)
//!
//! Les 243 photos du document de l'utilisateur pesaient 1 186 Mo en mémoire vive, toutes
//! tenues tout le temps — alors que l'écran n'en montre qu'une partie, et que la carte
//! graphique détient déjà ce qu'il montre. Chaque niveau a donc désormais un **état** : tenu
//! tant que l'écran s'en sert, offert au système sinon ([`niveau`]). Les dimensions restent
//! toujours connues ; les pixels, seulement quand ils sont tenus.
//!
//! Lire un niveau laisse une trace : il est **voulu** pour cette image du rendu. C'est ce que
//! le magasin relit à la fermeture pour décider quoi tenir, quoi offrir, quoi reprendre — la
//! pyramide ne décide rien, elle dit ce qu'on lui a demandé.

mod niveau;
pub use niveau::{Etat, Retour, Transit};

use glucose_core::occlusion::Boite;
use glucose_core::report;
use niveau::{Niveau, Octets};
use std::sync::atomic::{AtomicU64, Ordering};
use tiny_skia::{Pixmap, PixmapRef};

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
/// Le niveau de rang `0` est la résolution native ; le rang `k` a ses deux côtés divisés par
/// `2^k`, arrondis vers le haut pour ne jamais tomber à zéro. Le **facteur** d'un niveau est
/// `2^k` : c'est lui que les appelants manipulent, parce que c'est lui qui dit quels pixels
/// natifs un texel moyenne (BORDURES-4).
pub struct Pyramide {
    niveaux: Vec<Niveau>,
    opaque: bool,
    /// Les rangs lus ou voulus depuis que le magasin a regardé, un bit par rang (ETAGES-1).
    ///
    /// Atomique et non `Cell` : la composition se découpe en bandes sur plusieurs fils, qui
    /// empruntent la même pyramide en même temps.
    voulus: AtomicU64,
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
    /// Cette forme copie l'image donnée ; l'atelier, lui, décode directement dans les pages
    /// de la pyramide ([`Pyramide::depuis_rgba`]).
    pub fn nouvelle(native: Pixmap) -> Self {
        let mut niveau = Niveau::vide(native.width(), native.height())
            .expect("la place d'une image qu'on tient déjà se retrouve");
        if let Some(octets) = niveau.octets_mut() {
            octets.copy_from_slice(native.data());
        }
        Self::batir(niveau).expect("la place d'une image qu'on tient déjà se retrouve")
    }

    /// **Une image décodée par l'atelier**, prémultipliée directement dans les pages de son
    /// niveau natif — sans passer par un tampon qu'il faudrait recopier.
    ///
    /// `None` quand le système refuse la place.
    pub fn depuis_rgba(largeur: u32, hauteur: u32, rgba: &[u8]) -> Option<Self> {
        let mut niveau = Niveau::vide(largeur, hauteur)?;
        premultiplier(rgba, niveau.octets_mut()?);
        Self::batir(niveau)
    }

    /// Construit toutes les réductions sous le niveau natif, et constate l'opacité au passage.
    fn batir(native: Niveau) -> Option<Self> {
        let opaque = native.vue()?.data().as_chunks::<4>().0.iter().all(|p| p[3] == 255);
        let mut niveaux = vec![native];
        // La descente s'arrête d'elle-même à une image de 1 × 1 : la largeur et la hauteur
        // sont divisées par deux en arrondissant vers le haut, donc elles atteignent 1 et n'en
        // bougent plus.
        loop {
            let suivant = {
                let dernier = niveaux.last()?;
                if dernier.largeur <= 1 && dernier.hauteur <= 1 {
                    break;
                }
                let (dw, dh) = (
                    dernier.largeur.div_ceil(2).max(1),
                    dernier.hauteur.div_ceil(2).max(1),
                );
                let mut suivant = Niveau::vide(dw, dh)?;
                reduire_par_deux(dernier.vue()?, suivant.octets_mut()?, (dw, dh));
                suivant
            };
            niveaux.push(suivant);
        }
        Some(Self {
            niveaux,
            opaque,
            voulus: AtomicU64::new(0),
        })
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

    /// Les dimensions de la résolution native — toujours connues, même quand ses pixels ne
    /// sont pas tenus.
    pub fn dimensions_natives(&self) -> (u32, u32) {
        (self.niveaux[0].largeur, self.niveaux[0].hauteur)
    }

    /// Les dimensions du niveau réduit `facteur` fois.
    pub fn dimensions(&self, facteur: u32) -> (u32, u32) {
        let n = &self.niveaux[self.rang_de(facteur)];
        (n.largeur, n.hauteur)
    }

    /// **Combien de fois** le niveau qui couvre `largeur_ecran` est réduit : un pour la
    /// résolution native, deux pour le premier niveau, et ainsi de suite.
    ///
    /// « Couvre encore » veut dire : au moins aussi large que ce qu'on va dessiner. Choisir
    /// plus petit reviendrait à agrandir, donc à perdre du détail que la source avait.
    ///
    /// C'est aussi ce qu'un recadrage doit savoir pour ne lire aucun texel qui mêle ce qu'il
    /// garde à ce qu'il retire (BORDURES-4) : le texel `k` d'un niveau réduit `f` fois moyenne
    /// les pixels natifs `[k · f, (k + 1) · f[`.
    pub fn facteur_pour(&self, largeur_ecran: f32) -> u32 {
        1 << self.rang_pour(largeur_ecran)
    }

    /// **Le niveau réduit `facteur` fois**, s'il est tenu — et voulu, qu'il le soit ou non.
    ///
    /// Un facteur au-delà du dernier niveau désigne le dernier, le plus petit qui existe.
    pub fn niveau(&self, facteur: u32) -> Option<PixmapRef<'_>> {
        let rang = self.rang_de(facteur);
        self.vouloir(rang);
        self.niveaux[rang].vue()
    }

    /// La résolution native, si elle est tenue.
    pub fn native(&self) -> Option<PixmapRef<'_>> {
        self.niveau(1)
    }

    /// Le niveau qui couvre `largeur_ecran`, **exactement**, s'il est tenu.
    ///
    /// Pour ce qui se garde — une vignette servira des dizaines d'images, et la bâtir sur un
    /// autre niveau y figerait un flou ou un crénelage pour toutes.
    pub fn niveau_pour(&self, largeur_ecran: f32) -> Option<PixmapRef<'_>> {
        self.niveau(self.facteur_pour(largeur_ecran))
    }

    /// **Le meilleur niveau tenu pour `largeur_ecran`**, et son facteur — pour ce qui se
    /// dessine une fois et se refera à l'image suivante.
    ///
    /// Le voulu d'abord ; sinon un plus petit, qui floute un instant sans rien lire de trop ;
    /// sinon un plus grand, qui coûte davantage à réduire et crénèle, mais vaut mieux qu'un
    /// trou. Le voulu est marqué dans tous les cas, et celui qui sert aussi : le magasin
    /// reprendra le premier et ne rendra pas le second tant qu'il sert.
    pub fn meilleur_pour(&self, largeur_ecran: f32) -> Option<(u32, PixmapRef<'_>)> {
        let voulu = self.rang_pour(largeur_ecran);
        self.vouloir(voulu);
        let rang = (voulu..self.niveaux.len())
            .chain((0..voulu).rev())
            .find(|&r| self.niveaux[r].etat() == Etat::Tenu)?;
        self.vouloir(rang);
        Some((1 << rang, self.niveaux[rang].vue()?))
    }

    /// Le rang du niveau réduit `facteur` fois.
    fn rang_de(&self, facteur: u32) -> usize {
        (facteur.max(1).trailing_zeros() as usize).min(self.niveaux.len() - 1)
    }

    /// Le rang du niveau choisi : le plus réduit dont la largeur couvre encore l'écran.
    pub(crate) fn rang_pour(&self, largeur_ecran: f32) -> usize {
        let voulue = largeur_ecran.max(1.0);
        self.niveaux
            .iter()
            .take_while(|n| n.largeur as f32 >= voulue || n.largeur <= 1)
            .count()
            .saturating_sub(1)
    }

    fn vouloir(&self, rang: usize) {
        self.voulus.fetch_or(1 << rang, Ordering::Relaxed);
    }

    /// Rééchantillonne l'image à la forme voulue, phase comprise — ou rien si le niveau qui
    /// la couvre n'est pas tenu.
    ///
    /// La phase entre ici, et non au moment de poser : c'est elle qui fait la netteté, et la
    /// reporter au moment de poser obligerait à une position fractionnaire, donc au pipeline
    /// générique qu'on cherche justement à éviter.
    pub fn rendre(&self, forme: Forme) -> Option<Pixmap> {
        let mut vignette = Self::vignette_vide(forme)?;
        let hauteur = vignette.height();
        self.rendre_bande(forme, &mut vignette, 0, hauteur)
            .then_some(vignette)
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

    /// Remplit les lignes `y0..y1` d'une vignette déjà allouée (CASCADE-1), et dit si le
    /// niveau qu'il fallait était là.
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
    pub fn rendre_bande(&self, forme: Forme, dest: &mut Pixmap, y0: u32, y1: u32) -> bool {
        let Some(source) = self.niveau_pour(forme.largeur as f32) else {
            return false;
        };
        let (texels, _) = source.data().as_chunks::<4>();
        let Some(vue) = report::Vue::nouvelle(texels, source.width(), source.height()) else {
            return true;
        };
        let (largeur, hauteur) = (dest.width(), dest.height());
        let (pixels, _) = dest.data_mut().as_chunks_mut::<4>();
        let Some(mut cible) = report::VueMut::nouvelle(pixels, largeur, hauteur) else {
            return true;
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
        true
    }

    /// Le nombre de niveaux construits.
    pub fn niveaux_construits(&self) -> usize {
        self.niveaux.len()
    }

    /// Les octets que cette pyramide représente, où qu'ils soient.
    pub fn octets(&self) -> usize {
        self.niveaux.iter().map(Niveau::taille).sum()
    }

    /// Les octets de ses niveaux dans cet état.
    pub fn octets_en(&self, etat: Etat) -> usize {
        self.niveaux
            .iter()
            .filter(|n| n.etat() == etat)
            .map(Niveau::taille)
            .sum()
    }

    // ── Ce que le magasin demande pour décider (ETAGES-1) ─────────────────────────────

    /// Les rangs voulus depuis la dernière fois, et l'oubli de cette liste.
    pub(crate) fn prendre_les_voulus(&self) -> u64 {
        self.voulus.swap(0, Ordering::Relaxed)
    }

    /// Où sont les octets du niveau de ce rang.
    pub(crate) fn etat(&self, rang: usize) -> Etat {
        self.niveaux[rang].etat()
    }

    /// Ce niveau peut-il être offert ? Non s'il tient dans moins d'une page : le système
    /// n'offre que des pages entières ([`crate::plateforme::offre`]).
    pub(crate) fn offrable(&self, rang: usize) -> bool {
        match &self.niveaux[rang].octets {
            Octets::Tenus(t) => t.offrable(),
            _ => true,
        }
    }

    /// **Les octets de ce niveau partent chez un ouvrier**, qui les offrira ou les reprendra.
    /// Le niveau est en chemin jusqu'à ce qu'ils rentrent ; rien s'il n'y a rien à faire.
    pub(crate) fn sortir(&mut self, rang: usize) -> Option<Transit> {
        let niveau = &mut self.niveaux[rang];
        match std::mem::replace(&mut niveau.octets, Octets::EnChemin) {
            Octets::Tenus(t) => Some(Transit::AOffrir(t)),
            Octets::Offerts(o) => Some(Transit::AReprendre(o)),
            autre => {
                niveau.octets = autre;
                None
            }
        }
    }

    /// **Les octets rentrent**, offerts, repris, ou perdus. Un niveau qui n'attendait rien
    /// laisse tomber ce qui arrive — la pyramide a pu être refaite entre-temps.
    pub(crate) fn rentrer(&mut self, rang: usize, retour: Retour) {
        if let Some(niveau) = self.niveaux.get_mut(rang) {
            if niveau.etat() == Etat::EnChemin {
                niveau.octets = retour.en_octets();
            }
        }
    }
}

/// Prémultiplie des pixels RGBA droits dans `dest`, en entiers et arrondis au plus proche.
///
/// Sur le fil de fond, parce que la prémultiplication coûte autant que le décodage sur une
/// grande photo — la laisser au fil de rendu aurait déplacé le problème d'un mètre.
fn premultiplier(rgba: &[u8], dest: &mut [u8]) {
    let (source, _) = rgba.as_chunks::<4>();
    let (destination, _) = dest.as_chunks_mut::<4>();
    for (px, out) in source.iter().zip(destination.iter_mut()) {
        let a = u32::from(px[3]);
        // `t = c·a + 128`, puis `(t + (t >> 8)) >> 8` : le décalage remplace la division par
        // 255 exactement sur toute la plage — c'est vérifié canal par canal, et `a = 255`
        // redonne `c` sans écart.
        //
        // La version flottante qui était ici tronquait : un canal à 255 sur un pixel opaque
        // ressortait à 254. Invisible sur une photo, visible sur un aplat près d'une bordure.
        // Ma première réécriture se trompait dans l'autre sens — 128 devenait 129 — et c'est
        // le test qui l'a dit, pas la relecture.
        let premultiplie = |c: u8| {
            let t = u32::from(c) * a + 128;
            ((t + (t >> 8)) >> 8) as u8
        };
        *out = [
            premultiplie(px[0]),
            premultiplie(px[1]),
            premultiplie(px[2]),
            px[3],
        ];
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
fn reduire_par_deux(source: PixmapRef<'_>, dst: &mut [u8], (dw, dh): (u32, u32)) {
    let (sw, sh) = (source.width(), source.height());
    let src = source.data();
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
}

#[cfg(test)]
mod tests;
