//! La lueur d'une carte : l'ombre portée de Glucose Tauri, reproduite.
//!
//! # Ce que Glucose Tauri dessine, et ce que celui-ci dessinait
//!
//! Glucose Tauri pose une ombre CSS sur la boîte de la carte (`HtmlAnnotationLayer.tsx`) :
//!
//! ```css
//! box-shadow: 0 0 60px 30px color-mix(in srgb, AURA 15%, transparent);
//! ```
//!
//! — la boîte **dilatée de 30**, puis **floutée de 60**, à 15 % d'opacité. Son conteneur
//! porte `translate(…) scale(…)`, donc ces longueurs sont des unités **monde** : la lueur
//! grandit avec la carte, comme le veut SCALE-1, et rien ici n'a besoin de l'exception
//! écran.
//!
//! Ici, c'était un **disque** de rayon `max(w, h) × 1,5 + 50`. Sur une carte de 240 × 60,
//! cela fait 410 : la lueur débordait de 380 unités au-dessus d'une carte qui n'en mesure
//! que 60 de haut, et plus une carte est large, plus la bulle déborde **verticalement**.
//! L'opacité, elle, était juste — 35 sur 255 contre les 38 de Tauri. Ce n'était donc pas
//! une question d'intensité : c'était une forme qui ne désignait plus sa carte.
//!
//! # HALO-1 — une ombre de boîte est SÉPARABLE, et c'est ce qui la rend gratuite
//!
//! Flouter un rectangle par une gaussienne, c'est multiplier deux profils d'une seule
//! dimension :
//!
//! ```text
//! α(x, y) = A · P(x ; gauche, droite) · P(y ; haut, bas)
//! P(t ; a, b) = Φ(t − a) − Φ(t − b)
//! ```
//!
//! C'est **exact**, et non une approximation : une gaussienne à deux dimensions est le
//! produit de deux gaussiennes à une dimension, et l'intégrale sur un rectangle se
//! factorise donc terme à terme. Il suffit d'une table de `Φ` — elle ne dépend que de
//! l'écart-type en pixels, donc du zoom, et une frame n'en construit qu'une —, d'un
//! tableau par carte, et d'une multiplication par pixel.
//!
//! La seule chose que le produit ne capture pas est le `border-radius: 32px` de la boîte
//! de Tauri : il sépare un rectangle **droit**, donc la lueur est très légèrement plus
//! carrée dans les coins. L'écart s'y borne à la différence d'aire entre le carré du coin
//! et son quart de disque, étalée par le flou — quelques niveaux sur une lueur qui
//! plafonne à 38.
//!
//! La comparaison de coût, à géométrie égale (une carte de 240 × 60 à ×1) :
//!
//! | | disque | ombre séparable |
//! |---|---|---|
//! | pixels touchés | π · 410² ≈ **528 000** | 432 × 252 ≈ **109 000** |
//! | par pixel | recherche d'anneau, composition | une multiplication, composition |
//!
//! # HALO-2 — la portée se déduit du huit bits, elle ne se règle pas
//!
//! Une gaussienne n'a pas de bord : il faut bien décider où l'on cesse de la dessiner. Ce
//! n'est pas un réglage. La lueur s'écrit sur huit bits, donc une contribution inférieure
//! à un demi-niveau ne change aucun pixel, et la table de `Φ` se tronque exactement là où
//! sa masse restante passe sous ce seuil — pas avant, jamais après. La portée qui en sort
//! dépend de l'opacité et de l'écart-type, et c'est [`EdgeProfile::reach`] qui la dit.
//!
//! # HALO-3 — pourquoi il n'y a *aucun* cache ici
//!
//! Un cache de lueurs pré-rastérisées achèterait de la mémoire — et une borne à tenir, et
//! une invalidation — pour supprimer une multiplication par pixel. Le seul état conservé
//! d'une frame à l'autre reste la teinte symbiotique, dans [`SymbioticHueCache`] : elle,
//! contrairement au profil, dépend du voisinage et coûte cher à recalculer.

use super::scale::WorldScale;
use super::SymbioticHueCache;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;

use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::PixmapMut;

/// Dilatation de la boîte avant le flou, en unités monde — le `30px` de Tauri.
pub const HALO_SPREAD: f32 = 30.0;

/// Rayon de flou, en unités monde — le `60px` de Tauri.
///
/// CSS définit ce rayon comme **le double** de l'écart-type de la gaussienne : σ vaut
/// donc 30 unités monde.
pub const HALO_BLUR: f32 = 60.0;

/// Opacité de la lueur, sur 255 — les 15 % de `color-mix(in srgb, AURA 15%, transparent)`.
pub const HALO_ALPHA: u8 = 38;

/// **L'éclat d'une lueur, entre deux intensités — celles de Tauri** : au repos, et quand la
/// carte est **désignée** — la cible qu'une flèche en train de naître vise, les bouts d'une
/// flèche survolée (`isHighlightBox` : `0 0 80px 40px`, à 40 %).
///
/// C'est l'indice qui dit, pendant qu'on tire une flèche, à quoi elle va se lier. Et c'est une
/// **vivacité** continue, de 0 au repos à 1 désignée, parce que la lueur de Tauri passe de l'une
/// à l'autre en 0,2 s (LUEUR-2) : le module `animation::designation` la fait glisser.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eclat {
    vivacite: f32,
}

/// La dilatation, l'écart-type et l'opacité d'une lueur désignée : `0 0 80px 40px` à 40 %.
const DESIGNEE: (f32, f32, f32) = (40.0, 80.0 / 2.0, 102.0);

impl Eclat {
    /// Au repos.
    pub const REPOS: Self = Self { vivacite: 0.0 };
    /// Désignée, pleinement.
    pub const DESIGNEE: Self = Self { vivacite: 1.0 };

    /// L'éclat d'une carte : sa vivacité parmi `designees`, le repos sinon.
    pub fn de(ann: &Annotation, designees: &[(String, f32)]) -> Self {
        let vivacite = designees
            .iter()
            .find(|(id, _)| id == ann.id())
            .map_or(0.0, |(_, v)| v.clamp(0.0, 1.0));
        Self { vivacite }
    }

    fn entre(self, repos: f32, designee: f32) -> f32 {
        repos + (designee - repos) * self.vivacite
    }

    /// La dilatation de la boîte avant le flou, en unités monde.
    fn etalement(self) -> f32 {
        self.entre(HALO_SPREAD, DESIGNEE.0)
    }

    /// L'écart-type de la gaussienne, en unités monde (CSS : `blur = 2σ`).
    fn sigma(self) -> f32 {
        self.entre(HALO_BLUR / 2.0, DESIGNEE.1)
    }

    /// L'opacité au plateau, sur 255.
    pub fn alpha(self) -> u8 {
        self.entre(f32::from(HALO_ALPHA), DESIGNEE.2).round() as u8
    }
}

/// Divise par 255 avec arrondi au plus proche, sans division entière.
///
/// Exact pour tout produit de deux octets : c'est l'identité `(x + 128 + (x + 128) / 256) / 256`
/// qu'utilisent les compositeurs 8 bits pour rester au niveau près de `x / 255.0`.
///
/// **Ne sert plus qu'a prouver [`div255_swar`]**, qui fait la meme chose sur deux canaux a la
/// fois. C'est la garantie du projet : deux voies d'une meme operation rendent les memes bits,
/// et celle qu'on garde est celle qui se lit.
#[cfg(test)]
fn div255(value: u32) -> u32 {
    let biased = value + 128;
    (biased + (biased >> 8)) >> 8
}

/// Terme source d'un niveau d'opacité : la couleur de la lueur déjà multipliée par lui.
///
/// La lueur ne prend que [`HALO_ALPHA`] niveaux distincts une fois écrite sur huit bits,
/// donc tous se précalculent en une fois, et la boucle chaude se réduit à une indexation.
#[derive(Clone, Copy)]
struct LevelSource {
    /// Canaux 0 et 2, chacun multiplie par `alpha`, loges dans deux champs de seize bits.
    paires: u32,
    /// Canaux 1 et 3, de meme.
    impaires: u32,
    /// `255 - alpha`, le poids qui reste à la destination.
    inv_alpha: u32,
}

impl LevelSource {
    fn new((r, g, b): (u8, u8, u8), alpha: u8) -> Self {
        let poids = u32::from(alpha);
        Self {
            // Les canaux 0 et 2 dans un champ, les canaux 1 et 3 dans l'autre : chacun
            // occupe seize bits, ce qui laisse exactement la place au produit d'un octet par
            // un poids de huit bits.
            paires: (u32::from(r) * poids) | ((u32::from(b) * poids) << 16),
            impaires: (u32::from(g) * poids) | ((255 * poids) << 16),
            inv_alpha: 255 - poids,
        }
    }
}

/// Un canal sur deux, chacun loge dans seize bits.
const UN_CANAL_SUR_DEUX: u32 = 0x00FF_00FF;

/// Divise par 255 avec arrondi au plus proche, **deux canaux a la fois**.
///
/// La meme identite que [`div255`], appliquee aux deux champs d'un mot. Le decalage ferait
/// deborder le canal haut sur le bas, d'ou le masque : sans lui, le vert emprunterait a
/// l'alpha, ce qui ne se verrait qu'aux teintes extremes.
fn div255_swar(x: u32) -> u32 {
    let biaise = x + 0x0080_0080;
    ((biaise + ((biaise >> 8) & UN_CANAL_SUR_DEUX)) >> 8) & UN_CANAL_SUR_DEUX
}

/// Compose une couleur d'opacité constante sur un pixel (boucle chaude de la lueur).
///
/// La composition « source-over » prémultipliée s'écrit `(c * a + d * (255 - a)) / 255`
/// pour chaque canal. L'écrire ainsi — plutôt qu'en prémultipliant d'abord la source —
/// n'arrondit **qu'une fois**, comme le fait le pipeline flottant de tiny-skia.
fn blend_pixel(pixel: &mut [u8; 4], source: LevelSource) {
    let d = u32::from_ne_bytes(*pixel);
    let inv = source.inv_alpha;
    // `c x a + d x (255 - a)` vaut au plus `255 x 255 = 65 025` : chaque canal reste donc
    // dans ses seize bits, et les deux ne peuvent pas deborder l'un sur l'autre. C'est ce qui
    // permet de traiter quatre canaux en deux multiplications au lieu de huit -- sans une
    // seule instruction qui depende de la machine, donc aussi vite sur un telephone de 2013.
    let paires = div255_swar((d & UN_CANAL_SUR_DEUX) * inv + source.paires);
    let impaires = div255_swar(((d >> 8) & UN_CANAL_SUR_DEUX) * inv + source.impaires);
    // `r`, `g` et `b` restent ≤ `a` : la source est valide (canal ≤ 255) et la destination
    // l'est aussi (canal ≤ alpha). Le premultiplie reste donc valide sans avoir a le verifier.
    *pixel = (paires | (impaires << 8)).to_ne_bytes();
}

/// Le profil d'un bord flouté : la gaussienne cumulée, échantillonnée au pixel.
///
/// `Φ(t)` est la fraction de la lueur qui tombe à gauche de la distance `t`. Elle ne
/// dépend que de l'écart-type **en pixels écran**, donc du zoom : une seule table sert à
/// toutes les cartes d'une frame (HALO-1).
struct EdgeProfile {
    /// Masse cumulée aux frontières de pixel, de `0` à `1`. Sa longueur vaut `2r + 2`.
    cdf: Vec<f32>,
    /// Le rayon `r` du noyau, en pixels : `cdf[0]` est la masse à gauche de `−r − ½`.
    radius: f32,
}

impl EdgeProfile {
    /// Le profil d'une gaussienne d'écart-type `sigma`, en pixels écran.
    ///
    /// Un écart-type nul — ou sous le demi-pixel — donne un noyau d'un seul échantillon,
    /// c'est-à-dire exactement la marche d'un bord net. Ce cas ne demande donc aucune
    /// garde : il tombe de la construction.
    fn new(sigma: f32) -> Self {
        // Majorant d'allocation : au-delà de quatre écarts-types, la masse restante vaut
        // 3 · 10⁻⁵ — sous le demi-niveau de huit bits pour toute opacité admissible.
        // Ce n'est pas la portée : celle-ci est mesurée plus bas, sur la table obtenue.
        let radius = if sigma.is_finite() && sigma > 0.0 {
            (4.0 * sigma).ceil().min(4096.0)
        } else {
            0.0
        };
        let span = 2.0f32.mul_add(radius, 1.0) as usize;
        let two_sigma_squared = 2.0 * sigma * sigma;

        let mut cdf = Vec::with_capacity(span + 1);
        cdf.push(0.0);
        let mut total = 0.0f32;
        for k in 0..span {
            let d = k as f32 - radius;
            total += if two_sigma_squared > 0.0 {
                (-d * d / two_sigma_squared).exp()
            } else {
                f32::from(d == 0.0)
            };
            cdf.push(total);
        }
        for value in &mut cdf {
            *value /= total;
        }
        Self { cdf, radius }
    }

    /// La distance, en pixels, au-delà de laquelle la lueur ne peut plus changer un pixel.
    ///
    /// Elle ne se règle pas, elle se mesure (HALO-2). À la distance `d` au-delà d'un bord,
    /// la lueur vaut au plus `alpha · Q(d)`, où `Q` est la masse que la table laisse
    /// derrière elle ; elle s'arrondit donc à zéro dès que `Q(d)` passe sous `½ / alpha`.
    ///
    /// **La portée est le point où `Q` franchit ce seuil, pas la dernière frontière avant
    /// lui.** Entre deux frontières de pixel, [`Self::cumulative`] interpole : la queue y passe
    /// encore au-dessus du seuil sur une fraction de pixel. S'arrêter à la frontière coupait
    /// une rangée de lueur au niveau un, à la même place sur les deux voies — ce qui la rendait
    /// invisible à leur épreuve d'accord ; c'est la loi calculée pixel par pixel qui l'a vue
    /// (LUEUR-1).
    fn reach(&self, alpha: u8) -> f32 {
        let negligible = 0.5 / f32::from(alpha).max(1.0);
        let Some(i) = self.cdf.iter().rposition(|&mass| 1.0 - mass >= negligible) else {
            return 0.0;
        };
        let (q_i, q_suivant) = (
            1.0 - self.cdf[i],
            self.cdf.get(i + 1).map_or(0.0, |mass| 1.0 - mass),
        );
        let fraction = if q_i > q_suivant {
            (q_i - negligible) / (q_i - q_suivant)
        } else {
            0.0
        };
        // `i` est une frontière de pixel ; sa distance au centre est `i − (r + ½)`.
        (i as f32 - self.radius - 0.5 + fraction).max(0.0)
    }

    /// `Φ(t)` : la fraction de la lueur qui tombe à gauche de la distance `t`, en pixels.
    fn cumulative(&self, t: f32) -> f32 {
        let u = t + self.radius + 0.5;
        if u <= 0.0 {
            return 0.0;
        }
        let last = self.cdf.len() - 1;
        if u >= last as f32 {
            return 1.0;
        }
        let index = u as usize;
        let fraction = u - index as f32;
        self.cdf[index] + (self.cdf[index + 1] - self.cdf[index]) * fraction
    }

    /// La part de la lueur qu'une bande `[a, b]` dépose à la position `t`.
    ///
    /// C'est `∫ₐᵇ g(t − s) ds` : le changement de variable `u = t − s` le rend égal à
    /// `Φ(t − a) − Φ(t − b)`, soit une soustraction de deux lectures de table.
    fn band(&self, t: f32, a: f32, b: f32) -> f32 {
        self.cumulative(t - a) - self.cumulative(t - b)
    }
}

/// La boîte **dilatée** d'une carte, en pixels écran : celle que le flou étale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HaloBox {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    /// L'écart-type de la gaussienne, en pixels écran.
    pub sigma: f32,
    /// La carte que la lueur entoure, en pixels écran — ou rien pour une lueur libre.
    ///
    /// LUEUR-1 : l'ombre de Tauri est une `box-shadow`, et CSS la **découpe** à l'intérieur de
    /// la boîte qui la porte. La carte ne pose donc jamais sa lueur sous son propre texte :
    /// c'est ce qui fait « le texte sur fond noir, le contour en lueur ».
    pub carte: Option<glucose_core::membrane_forme::Arrondi>,
}

/// Première rangée (ou colonne) dont le centre de pixel atteint `position`, bornée à l'écran.
fn first_pixel_at_or_after(position: f32, limit: i32) -> i32 {
    if !position.is_finite() {
        return if position.is_sign_negative() {
            0
        } else {
            limit
        };
    }
    let index = (position - 0.5).ceil();
    if index <= 0.0 {
        0
    } else if index >= limit as f32 {
        limit
    } else {
        index as i32
    }
}

/// Pose UNE lueur, sans rien preparer d'avance.
///
/// Les lueurs des cartes se preparent toutes d'un coup ([`LueurPrete`]), pour que le
/// preambule ne se refasse pas par bande. Celle-ci sert une lueur isolee : les epreuves
/// d'aspect, et le passage qu'une fleche survolee fait briller (FLECHE-4).
pub(crate) fn draw_halo(dst: &mut PixmapMut, halo: HaloBox, rgb: (u8, u8, u8), alpha: u8) {
    let (largeur, hauteur) = (dst.width() as i32, dst.height() as i32);
    if let Some(prete) = LueurPrete::nouvelle(halo, rgb, alpha, largeur, hauteur) {
        prete.peindre(dst, 0);
    }
}

/// De quel cote de la lueur on se trouve, donc dans quel sens son niveau varie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sens {
    Montant,
    Descendant,
}

/// L'indice du maximum du profil : la frontiere entre sa montee et sa descente.
///
/// Sur le plateau, toutes les valeurs se valent et n'importe laquelle convient -- les deux
/// moities restent monotones au sens large, ce qui suffit a la recherche par dichotomie.
fn index_du_sommet(colonnes: &[f32]) -> usize {
    colonnes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map_or(0, |(i, _)| i)
}

/// Peint une moitie de ligne **par segments de niveau constant**.
///
/// # Pourquoi des segments, et non un calcul par pixel
///
/// `niveau(x) = arrondi(poids x profil(x))` ne prend que `alpha + 1` valeurs distinctes, et le
/// profil est monotone de chaque cote du sommet. Les pixels de meme niveau sont donc
/// **contigus**, et chercher leurs frontieres par dichotomie coute quelques centaines
/// d'evaluations par ligne la ou le calcul direct en demandait plusieurs milliers.
///
/// La mesure le reclamait sans ambiguite : sur un ecran de 2560 x 1440, le melange coutait
/// 3,5 ms quand le seul calcul du niveau en coutait 12,7. Le remede n'etait pas d'ecrire moins
/// de pixels, mais d'en calculer moins -- et j'avais commence par l'autre.
///
/// Le resultat est le meme, pixel pour pixel : chacun recoit le niveau de sa colonne, obtenu
/// une fois pour tout son segment au lieu d'etre recalcule.
fn peindre_par_segments(
    ligne: &mut [[u8; 4]],
    poids: f32,
    colonnes: &[f32],
    levels: &[LevelSource],
    sens: Sens,
) {
    let dernier = levels.len() - 1;
    let niveau = |c: f32| ((poids * c).round() as usize).min(dernier);
    let mut i = 0;
    while i < ligne.len() {
        let k = niveau(colonnes[i]);
        // La frontiere du segment : le premier pixel dont le niveau differe. `partition_point`
        // exige un predicat vrai puis faux, d'ou le sens.
        let reste = &colonnes[i..ligne.len()];
        let longueur = match sens {
            Sens::Montant => reste.partition_point(|c| niveau(*c) <= k),
            Sens::Descendant => reste.partition_point(|c| niveau(*c) >= k),
        }
        // Le profil n'est monotone qu'a la precision du flottant : sans ce plancher, une
        // oscillation d'un ulp sur le plateau ferait boucler sans fin.
        .max(1);
        if k > 0 {
            for pixel in &mut ligne[i..i + longueur] {
                blend_pixel(pixel, levels[k]);
            }
        }
        i += longueur;
    }
}

/// La boîte dilatée de la lueur d'une carte, ou `None` si elle ne touche pas le cadre.
pub(crate) fn halo_geometry(
    ann: &Annotation,
    (vp, scale): (&Viewport, WorldScale),
    (screen_w, screen_h, header_h): (f32, f32, f32),
    eclat: Eclat,
) -> Option<HaloBox> {
    if !matches!(ann, Annotation::Text { .. }) {
        return None;
    }
    let rect = ann.rect()?;
    let (sx, sy) = world_to_screen(rect.left, rect.top, vp);
    let (sx, sy) = (sx as f32, sy as f32);
    let (w, h) = (
        scale.world(rect.width as f32),
        scale.world(rect.height as f32),
    );
    // Toutes ces longueurs sont des unités monde mises à l'échelle : la lueur grandit
    // avec sa carte (SCALE-1), et rien ici ne demande l'exception écran.
    let spread = scale.world(eclat.etalement());
    let halo = HaloBox {
        left: sx - spread,
        top: sy - spread,
        right: sx + w + spread,
        bottom: sy + h + spread,
        sigma: scale.world(eclat.sigma()),
        carte: Some(super::card::forme_de_la_carte(
            (sx, sy),
            (w, h),
            scale.world(super::card::CORNER_RADIUS),
        )),
    };

    // Frustum culling : la lueur déborde de la portée du flou, et pas d'un pixel de plus.
    //
    // La condition dit ce qu'il faut pour dessiner, et non les quatre façons de sortir.
    // Ce n'est pas qu'une question de lisibilité : un viewport brisé rend des bornes
    // `NaN`, toute comparaison avec `NaN` est fausse, et une condition de **rejet** les
    // laissait donc toutes passer — `halo_geometry` annonçait alors une boîte qui n'existe
    // pas. Écrite en positif, elle les rejette sans avoir à les nommer.
    let reach = EdgeProfile::new(halo.sigma).reach(eclat.alpha());
    let visible = halo.right + reach >= 0.0
        && halo.left - reach <= screen_w
        && halo.bottom + reach >= header_h
        && halo.top - reach <= screen_h;
    visible.then_some(halo)
}

/// **La portée du flou** : la distance au-delà de laquelle une lueur d'opacité `alpha` ne peut
/// plus changer un pixel, en pixels d'écran (HALO-2).
///
/// Elle ne se règle pas, elle se mesure sur la table obtenue — voir [`EdgeProfile::reach`].
/// La voie graphique en a besoin pour dimensionner le quad d'une lueur : au-delà, elle
/// dessinerait des pixels dont elle a déjà prouvé qu'ils ne bougeront pas.
pub(crate) fn portee_du_flou(sigma: f32, alpha: u8) -> f32 {
    EdgeProfile::new(sigma).reach(alpha)
}

/// Passe de rendu des lueurs d'ambiance (L1 : ne parcourt que les cartes visibles).
pub fn draw_halos(
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    store: &Store,
    (pass, designees): (ViewPass<'_>, &[(String, f32)]),
) {
    let ViewPass {
        visibles, header_h, ..
    } = pass;
    let vp = &pass.vp;
    let Some(board) = store.active_board() else {
        return;
    };

    let screen_w = pixmap.width() as f32;
    let screen_h = pixmap.height() as f32;

    // On va droit aux nœuds visibles (CULL-1). La version précédente parcourait le tableau
    // entier en demandant de chacun s'il était visible : sur un million de nœuds dont cinq
    // cents à l'écran, c'était un million de hachages de chaîne pour cette seule passe.
    //
    // **Deux temps, parce que la teinte s'écrit et que la lueur se peint.** Le cache des
    // teintes symbiotiques se modifie — il dépend du voisinage et coûte cher —, donc il tient
    // le premier temps à lui seul. La peinture, elle, ne lit plus que des couleurs déjà
    // décidées, et seize fils peuvent la faire ensemble.
    let mut a_peindre: Vec<(HaloBox, (u8, u8, u8), u8)> = Vec::new();
    for ann in Visibles::nouvelles(visibles, board).annotations() {
        let eclat = Eclat::de(ann, designees);
        let ecran = (screen_w, screen_h, header_h);
        let Some(halo) = halo_geometry(ann, (vp, pass.echelle()), ecran, eclat) else {
            continue;
        };
        let (_hue, rgb) = hue_cache.get_or_compute(ann, pass.index, board);
        a_peindre.push((halo, rgb, eclat.alpha()));
    }
    peindre_en_bandes(pixmap, &a_peindre);
}

mod bandes;

#[cfg(test)]
use bandes::peindre_en;
use bandes::{peindre_en_bandes, LueurPrete};

#[cfg(test)]
mod tests;
