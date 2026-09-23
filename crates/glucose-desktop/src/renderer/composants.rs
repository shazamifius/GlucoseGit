//! **Les composants comme textures** : un composant qui n'a pas changé est une image qu'on a
//! déjà, et la carte graphique le pose comme elle pose une photo.
//!
//! # Ce que la mesure a dit, et ce qu'elle a démenti
//!
//! `bench_texte`, 480 cartes, 2560 × 1600, par la voie graphique :
//!
//! ```text
//!     immobile     annotations  12,0 ms   glyphes rasterises  0
//!     glissement   annotations  13,0 ms   glyphes rasterises  0
//!     zoom         annotations  15,1 ms   glyphes rasterises  93
//!
//!     dont, par carte visible :   cadre 7,7 ms   corps 1,9 ms   mise en page 0,55 ms
//! ```
//!
//! L'hypothèse était le cache de glyphes — évincé en boucle pendant un glissement, contourné
//! par la taille pendant un zoom. Les deux sont vrais, et les deux sont **sans effet** :
//! l'image immobile, qui ne rastérise rien, coûte la même chose. Ce qui coûte est le
//! **remplissage** de soixante-douze rectangles arrondis anti-aliasés à chaque image, pour des
//! pixels qui n'ont pas changé depuis l'image d'avant.
//!
//! L'utilisateur l'avait dit dans ses mots : comprendre « ce bloc texte comme un composant,
//! pas comme une image ». Un composant est mis en page une fois, rendu une fois, et composé à
//! chaque image — c'est ce que fait un navigateur d'un bloc qui n'a pas changé, et c'est de
//! là que Glucose Tauri tenait sa netteté et sa légèreté.
//!
//! # COMPOSANT-1 — la même loi que les photos, et que les tuiles
//!
//! Un composant se rend **une fois** dans un tampon à sa taille, devient une texture, et ne
//! coûte plus que sa pose — quatre nombres — tant que rien de ce qu'il montre ne change. Ce
//! qui invalide la texture est **tout ce qui change ses pixels**, et rien d'autre : une
//! empreinte les résume ; une empreinte nouvelle est une clé nouvelle, et l'ancienne texture
//! s'oublie toute seule à la fin de l'image, comme une photo sortie de l'écran.
//!
//! La texture se rend **à la demande**, quand la carte graphique dit qu'elle ne la connaît
//! pas — et jamais avant : le socle ne tient pas de liste de ce que la carte détient, parce
//! que deux listes qui doivent rester d'accord finissent par ne plus l'être.
//!
//! # En mouvement la carte interpole, à l'arrêt tout est exact
//!
//! C'est la politique des tuiles (fiche 19 § 5.4), et elle vaut ici mot pour mot :
//!
//! * **en mouvement**, la texture est rendue au **palier dyadique** le plus proche de
//!   l'échelle, et posée à sa position fractionnaire. La carte graphique la filtre en
//!   bilinéaire — un adoucissement d'au plus un demi-pixel, jamais une pixelisation — et un
//!   zoom d'une octave ne la refait qu'**une** fois au lieu de cent ;
//! * **à l'arrêt**, elle est rendue à l'échelle exacte, avec sa phase sous-pixel exacte, et
//!   posée sur un pixel entier : les pixels sont ceux que le processeur aurait écrits.
//!
//! # Ce qui est un composant aujourd'hui
//!
//! * **Une carte de texte** qui n'est pas en édition. Celle qu'on édite change à chaque
//!   frappe, son curseur clignote et sa prévisualisation de formule déborde de sa boîte :
//!   elle reste sur le chemin processeur, dans la couche du dessus — donc au premier plan, ce
//!   qui est juste pour ce qu'on est en train d'écrire.
//! * **Une photo en chemin** — dont les octets ne sont pas encore là. Son cadre se posait dans
//!   la couche du dessus, donc au-dessus de tout ; il se pose maintenant **à son rang**, et
//!   l'écart d'ordre de la fiche 22 § 5.1 n'existe plus.
//!
//! Ce qui ne l'est pas : un composant plus grand que l'écran, dont la texture ne tiendrait
//! pas — il se dessine en direct, comme une photo en zoom proche. Et les **ornements** —
//! poignées, réglette de domaines — qui sont des affordances en pixels écran, pas du contenu :
//! ils restent dans la couche du dessus, comme pour les photos.

use super::card::{card_text_layout, draw_card_contenu, CardLayout, TextCard};
use super::pass::{Clip, Pass, SELECTION_RING};
use super::richtext::TextMode;
use super::scale::WorldScale;
use super::scene::image::ornement::draw_missing_image;
use super::{PaintKit, Regard};
use crate::canvas::world_to_screen;
use crate::present::scene_gpu::Pose;
use glucose_core::types::{BoardImage, Viewport};
use std::hash::{Hash, Hasher};
use tiny_skia::Pixmap;

/// Un composant que la carte graphique posera, et tout ce qu'il faut pour le rendre **hors
/// contexte** — sans le document, sans la vue — si sa texture manque encore.
#[derive(Debug, Clone)]
pub struct Composant {
    /// La clé de la texture : la nature, l'identifiant et l'empreinte de ce qu'elle montre.
    pub cle: String,
    /// **Ce que ce composant est**, indépendamment de ce qu'il montre en ce moment : la
    /// nature et l'identifiant, sans l'empreinte.
    ///
    /// Deux textures de la même identité sont le même composant à deux paliers, ou avant et
    /// après une frappe. C'est ce qui permet de **garder l'ancienne posée** tant que la
    /// nouvelle n'est pas rendue (CASCADE-2) : un composant un peu flou vaut mieux qu'un
    /// composant absent, et infiniment mieux qu'une image qui gèle pour le rendre.
    ///
    /// Elle est portée explicitement et non déduite de la clé : découper sur le dernier
    /// deux-points supposerait qu'aucun identifiant n'en contienne, ce que rien ne garantit.
    pub identite: String,
    /// Où la texture se pose, à l'échelle de la **vue**.
    pub pose: Pose,
    contenu: Contenu,
    /// L'échelle à laquelle la texture se rend — exacte à l'arrêt, un palier en mouvement.
    echelle: f64,
    /// La phase sous-pixel du composant dans sa texture : exacte à l'arrêt, nulle en mouvement.
    phase: (f32, f32),
    /// La marge autour de la boîte, pour l'anneau de sélection et l'anti-crénelage.
    marge: f32,
    /// La taille de la texture, en pixels.
    pixels: (u32, u32),
}

/// Ce qu'un composant montre, et ce qu'il faut pour le dessiner.
#[derive(Debug, Clone)]
enum Contenu {
    Carte {
        origine: (f64, f64),
        taille: (f32, f32),
        corps: String,
        teinte: (u8, u8, u8),
        selectionnee: bool,
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
        /// La taille de la photo à l'échelle de rendu, en pixels.
        taille: (f32, f32),
        /// La boîte englobante du cadre tourné, à l'échelle de rendu — gardée telle quelle,
        /// parce que la recalculer depuis la texture arrondie décalait le cadre d'un pixel,
        /// et un cadre anti-crénelé décalé d'un pixel n'a plus un bord en commun avec lui-même.
        englobante: (f32, f32),
        /// En radians, autour du centre — l'unité du modèle.
        rotation: f64,
    },
}

impl Contenu {
    fn hacher(&self, h: &mut impl Hasher) {
        match self {
            Self::Carte {
                taille,
                corps,
                teinte,
                selectionnee,
                edition,
                ..
            } => {
                0u8.hash(h);
                corps.hash(h);
                taille.0.to_bits().hash(h);
                taille.1.to_bits().hash(h);
                teinte.hash(h);
                selectionnee.hash(h);
                // **Ce que la saisie change, et rien d'autre.** Le texte est deja dans
                // `corps` -- `carte_de` y met le tampon d'edition. Restent l'etendue
                // selectionnee et la phase du curseur, que BLINK-1 a sortie de l'horloge
                // pour en faire un booleen : sans elle, deux phases opposees donneraient la
                // meme cle et le curseur cesserait de clignoter.
                //
                // `goal_x` et `blink_timer` n'y sont PAS, et c'est voulu : ils decident de
                // ce que le curseur fera, jamais de ce qu'il montre. Les hacher referait la
                // texture a chaque touche de direction sans qu'un pixel change.
                match edition {
                    Some(e) => {
                        1u8.hash(h);
                        e.selection.anchor.hash(h);
                        e.selection.head.hash(h);
                        e.curseur_visible.hash(h);
                    }
                    None => 0u8.hash(h),
                }
            }
            Self::PhotoEnChemin {
                id,
                taille,
                rotation,
                ..
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

/// Ce que le composant montre, résumé en un nombre : tout ce qui change ses pixels, rien
/// d'autre.
fn empreinte(c: &Composant) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    c.contenu.hacher(&mut h);
    c.echelle.to_bits().hash(&mut h);
    c.phase.0.to_bits().hash(&mut h);
    c.phase.1.to_bits().hash(&mut h);
    h.finish()
}

/// Le palier dyadique le plus proche de cette échelle : la puissance de deux dont le
/// logarithme est le plus près, donc un rapport d'au plus racine de deux dans un sens ou
/// l'autre.
///
/// Le plus proche, et non le supérieur : rendre au palier supérieur coûterait jusqu'à quatre
/// fois les pixels pour une netteté que l'œil ne résout pas en mouvement (fiche 18, loi I).
fn palier_dyadique(echelle: f64) -> f64 {
    if !echelle.is_finite() || echelle <= 0.0 {
        return 1.0;
    }
    2f64.powi(echelle.log2().round() as i32)
}

/// Comment les composants de cette image se rendent : à quelle échelle, avec quelle phase.
///
/// Décidé une fois par image, pour tous — c'est ce qui garantit que deux composants voisins
/// se rendent au même palier et se posent de la même façon.
#[derive(Debug, Clone, Copy)]
pub(super) struct Regime {
    vue: Viewport,
    /// L'échelle de rendu : exacte à l'arrêt, un palier dyadique en mouvement.
    echelle: f64,
    en_mouvement: bool,
    clip: Clip,
    ecran: (u32, u32),
}

impl Regime {
    pub(super) fn de(vue: Viewport, regard: Regard, ecran: (u32, u32), header_h: f32) -> Self {
        Self {
            vue,
            echelle: if regard.en_mouvement {
                palier_dyadique(vue.scale)
            } else {
                vue.scale
            },
            en_mouvement: regard.en_mouvement,
            clip: Clip {
                width: ecran.0 as f32,
                height: ecran.1 as f32,
                top: header_h,
            },
            ecran,
        }
    }

    /// Le rapport entre l'échelle de la vue et celle du rendu : ce par quoi la carte
    /// graphique agrandit la texture.
    fn rapport(&self) -> f32 {
        (self.vue.scale / self.echelle) as f32
    }

    /// Où un composant de coin écran `(sx, sy)` se pose, et avec quelle phase.
    ///
    /// À l'arrêt, le composant garde sa phase sous-pixel et se pose sur un pixel entier : les
    /// pixels sont ceux que le processeur aurait écrits. En mouvement, la phase est nulle et
    /// la pose fractionnaire : la carte graphique interpole.
    fn phase_et_coin(&self, sx: f32, sy: f32) -> ((f32, f32), (f32, f32)) {
        if self.en_mouvement {
            ((0.0, 0.0), (sx, sy))
        } else {
            ((sx - sx.floor(), sy - sy.floor()), (sx.floor(), sy.floor()))
        }
    }

    /// Assemble le composant, ou `None` si sa texture dépasserait l'écran.
    fn composer(
        &self,
        prefixe: &str,
        id: &str,
        (sx, sy): (f32, f32),
        (largeur, hauteur, marge): (f32, f32, f32),
        contenu: Contenu,
    ) -> Option<Composant> {
        // La phase sous-pixel pousse le contenu de moins d'un pixel : la texture le compte,
        // sans quoi le dernier rang du trait anti-crenele est coupe -- vingt-cinq niveaux aux
        // coins bas d'une carte selectionnee, et c'est l'epreuve des deux voies qui l'a vu.
        let pixels = (
            (largeur + 2.0 * marge + 1.0).ceil() as u32,
            (hauteur + 2.0 * marge + 1.0).ceil() as u32,
        );
        if pixels.0 == 0 || pixels.1 == 0 || pixels.0 > self.ecran.0 || pixels.1 > self.ecran.1 {
            return None;
        }
        let (phase, coin) = self.phase_et_coin(sx, sy);
        let rapport = self.rapport();
        let mut composant = Composant {
            cle: String::new(),
            identite: format!("{prefixe}:{id}"),
            pose: Pose {
                x: coin.0 - marge * rapport,
                y: coin.1 - marge * rapport,
                largeur: pixels.0 as f32 * rapport,
                hauteur: pixels.1 as f32 * rapport,
                opacite: 1.0,
                angle: 0.0,
                fenetre: Pose::TOUT,
                bornes: Pose::PARTOUT,
            },
            contenu,
            echelle: self.echelle,
            phase,
            marge,
            pixels,
        };
        composant.cle = format!("{prefixe}:{id}:{:016x}", empreinte(&composant));
        Some(composant)
    }

    /// **Une carte de texte**, ou `None` si elle ne touche pas l'écran ou le dépasse.
    pub(super) fn carte(
        &self,
        kit: PaintKit<'_>,
        id: &str,
        (x, y, w, h): (f64, f64, f32, f32),
        (corps, teinte, selectionnee): (&str, (u8, u8, u8), bool),
        edition: Option<&crate::renderer::TextEditSession>,
    ) -> Option<Composant> {
        // MODE-1 : une carte qu'on corrige montre ses signes, une carte qu'on lit ne les
        // montre pas -- et le decoupage en lignes n'est pas le meme dans les deux modes.
        let mode = if edition.is_some() {
            TextMode::Source
        } else {
            TextMode::Rendered
        };
        // La hauteur suit le texte : une carte ne tronque jamais son contenu (TEXT-FIT-1).
        let mise_en_page = card_text_layout(kit.typography, kit.math, corps, w, mode);
        // **Une previsualisation de formule ne rentre pas dans une texture.** Elle se pose a
        // DROITE de la carte, hors de sa boite, et bascule a gauche quand le bord de l'ecran
        // approche -- son placement lit `clip.width`, qui vaut l'ecran dans une passe et la
        // texture dans un composant. La carte reste donc au processeur tant qu'elle en
        // montre une ; c'est le seul cas, et il dure le temps qu'un curseur traverse une
        // formule.
        if edition.is_some_and(|e| super::card::previsualisation_en_cours(&mise_en_page, e)) {
            return None;
        }
        let lignes = mise_en_page.line_count();
        let vue = CardLayout::text_card(w, h, lignes).scaled(WorldScale::new(self.vue.scale));
        let (sx, sy) = world_to_screen(x, y, &self.vue);
        let (sx, sy) = (sx as f32, sy as f32);
        if self.clip.rejects(sx, sy, vue.width, vue.height) {
            return None;
        }
        let rendu = CardLayout::text_card(w, h, lignes).scaled(WorldScale::new(self.echelle));
        let anneau = WorldScale::new(self.echelle).screen(SELECTION_RING);
        let marge = (rendu.border.max(anneau) / 2.0).ceil() + 1.0;
        self.composer(
            "carte",
            id,
            (sx, sy),
            (rendu.width, rendu.height, marge),
            Contenu::Carte {
                origine: (x, y),
                taille: (w, h),
                corps: corps.to_string(),
                teinte,
                selectionnee,
                edition: edition.cloned(),
            },
        )
    }

    /// **Une photo dont les octets ne sont pas encore là**, comme un cadre à son rang.
    ///
    /// La texture est la boîte englobante du cadre **tourné** : le libellé, lui, reste droit,
    /// comme le processeur le dessine — la texture se pose donc sans angle.
    pub(super) fn photo_en_chemin(&self, img: &BoardImage) -> Option<Composant> {
        let (sx, sy) =
            world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &self.vue);
        let (sx, sy) = (sx as f32, sy as f32);
        let (sw, sh) = (
            (img.width * self.vue.scale) as f32,
            (img.height * self.vue.scale) as f32,
        );
        if self.clip.rejects(sx, sy, sw, sh) {
            return None;
        }
        let taille = (
            (img.width * self.echelle) as f32,
            (img.height * self.echelle) as f32,
        );
        // La boîte englobante d'un rectangle tourné autour de son centre.
        let (cos, sin) = (
            img.rotation.cos().abs() as f32,
            img.rotation.sin().abs() as f32,
        );
        let englobante = (
            taille.0 * cos + taille.1 * sin,
            taille.0 * sin + taille.1 * cos,
        );
        // Le centre de la photo à l'écran, d'où le coin de la boîte englobante se déduit.
        let rapport = self.rapport();
        let centre = (sx + sw / 2.0, sy + sh / 2.0);
        let coin = (
            centre.0 - englobante.0 / 2.0 * rapport,
            centre.1 - englobante.1 / 2.0 * rapport,
        );
        self.composer(
            "chemin",
            &img.id,
            coin,
            (englobante.0, englobante.1, 2.0),
            Contenu::PhotoEnChemin {
                id: img.id.clone(),
                taille,
                englobante,
                rotation: img.rotation,
            },
        )
    }
}

impl Composant {
    /// **Rend le composant dans son propre tampon**, exactement comme le processeur le
    /// dessine en place — même passe, même mise en page, même anti-crénelage — la vue étant
    /// celle qui pose son coin sur la marge, à sa phase.
    pub fn rendre(&self, kit: PaintKit<'_>) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(self.pixels.0, self.pixels.1)?;
        let coin = (self.marge + self.phase.0, self.marge + self.phase.1);
        match &self.contenu {
            Contenu::Carte {
                origine,
                taille,
                corps,
                teinte,
                selectionnee,
                edition,
            } => {
                // `world_to_screen` vaut `monde x echelle + vue` : la vue qui place le coin
                // de la carte en `(marge + phase)` s'en deduit en une ligne.
                let vp = Viewport {
                    scale: self.echelle,
                    x: f64::from(coin.0) - origine.0 * self.echelle,
                    y: f64::from(coin.1) - origine.1 * self.echelle,
                };
                let ctx = Pass {
                    typography: kit.typography,
                    math: kit.math,
                    tints: kit.tints,
                    theme: kit.theme,
                    vp,
                    scale: WorldScale::new(self.echelle),
                    clip: Clip {
                        width: self.pixels.0 as f32,
                        height: self.pixels.1 as f32,
                        top: 0.0,
                    },
                };
                draw_card_contenu(
                    &ctx,
                    &mut pixmap.as_mut(),
                    TextCard {
                        origin: *origine,
                        size: *taille,
                        body: corps,
                        tint: *teinte,
                        selected: *selectionnee,
                        editing: edition.as_ref(),
                    },
                );
            }
            Contenu::PhotoEnChemin {
                id,
                taille,
                englobante,
                rotation,
            } => {
                // Le cadre non tourne se place au centre de la boite englobante ; c'est la
                // rotation autour de ce centre qui le fait tenir dedans.
                let at = (
                    coin.0 + (englobante.0 - taille.0) / 2.0,
                    coin.1 + (englobante.1 - taille.1) / 2.0,
                );
                draw_missing_image(
                    kit.typography,
                    kit.theme,
                    &mut pixmap.as_mut(),
                    at,
                    *taille,
                    id,
                    *rotation,
                );
            }
        }
        Some(pixmap)
    }
}

#[cfg(test)]
mod tests;
