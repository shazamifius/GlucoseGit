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
//! Ce qui ne l'est pas : les **ornements** — poignées, réglette de domaines — qui sont des
//! affordances en pixels écran, pas du contenu : ils restent dans la couche du dessus, comme
//! pour les photos.
//!
//! # DE-PRES-1 — un composant plus grand que l'écran se dessine en formes (fiche 73)
//!
//! Sa texture entière dépasserait l'écran : elle coûterait des pixels que personne ne voit, et
//! une carte graphique ne l'accepte pas au-delà de sa taille maximale. La fiche 24 le refusait
//! — de près, une carte disparaissait —, puis la fiche 29 le découpait en carrés de 256 pixels
//! rendus un à un, la texture entière agrandie posée en attendant : *« absolument illisible »*
//! de près (fiche 72). Désormais **la carte le dessine depuis ses formes**
//! ([`crate::present::formes_gpu`]) : chaque lettre par Slug, chaque fond par la loi des
//! membranes — net à tout zoom, sans rien repeindre. Le détail est dans [`formes`].
//!
//! Sa texture au plus haut palier qui tient dans l'écran se **prépare** encore, dans le temps
//! qui reste, et ne se pose jamais tant qu'il est dessiné en formes : c'est elle qui se pose à
//! l'instant où, dézoomé, il retient dans l'écran — au lieu d'un trou, ou d'une image qui gèle
//! le temps de la peindre.

use super::card::{draw_card_contenu, CardLayout, Contenants};
use super::pass::{Clip, Pass, SELECTION_RING};
use super::richtext::TextMode;
use super::scale::WorldScale;
use super::scene::image::ornement::draw_missing_image;
use super::{PaintKit, Regard};
use crate::canvas::world_to_screen;
use crate::present::scene_gpu::Pose;
use glucose_core::tuile::Adresse;
use glucose_core::types::{BoardImage, Viewport};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tiny_skia::Pixmap;

mod contenu;
mod formes;

use contenu::{cadre_en_chemin, Contenu};

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
    /// Partagé entre les tuiles d'un même composant : une carte découpée en quatre-vingts
    /// carrés ne copie pas quatre-vingts fois son texte.
    contenu: Arc<Contenu>,
    /// L'échelle à laquelle la texture se rend — exacte à l'arrêt, un palier en mouvement.
    echelle: f64,
    /// La densité de l'écran (DPI-1) : l'anneau d'une carte sélectionnée est en pixels
    /// logiques. Elle entre dans l'empreinte : une fenêtre qui change d'écran refait ses
    /// textures.
    densite: f32,
    /// La phase sous-pixel du composant dans sa texture : exacte à l'arrêt, nulle en mouvement.
    phase: (f32, f32),
    /// La marge autour de la boîte, pour l'anneau de sélection et l'anti-crénelage.
    marge: f32,
    /// La taille de la texture, en pixels.
    pixels: (u32, u32),
}

/// **Ce qu'un composant devient à l'écran** : sa texture — ou, s'il est plus grand que
/// l'écran, ses formes, et sa texture à préparer (DE-PRES-1).
#[derive(Debug, Default)]
pub struct Pieces {
    /// La texture qui se pose, s'il tient dans l'écran.
    pub posee: Option<Composant>,
    /// **Ses formes**, s'il ne tient pas dans l'écran : la carte les dessine (fiche 73).
    pub formes: Vec<crate::present::formes_gpu::Primitive>,
    /// Sa texture au plus haut palier qui tient dans l'écran, **à préparer** dans le temps qui
    /// reste : elle ne se pose jamais tant qu'il est dessiné en formes, mais elle est là quand,
    /// dézoomé, il retient dans l'écran.
    pub a_preparer: Option<Composant>,
}

/// Ce que le composant montre, résumé en un nombre : tout ce qui change ses pixels, rien
/// d'autre.
fn empreinte(c: &Composant) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    c.contenu.hacher(&mut h);
    c.echelle.to_bits().hash(&mut h);
    c.densite.to_bits().hash(&mut h);
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
    /// La densité de l'écran (DPI-1).
    densite: f32,
    en_mouvement: bool,
    clip: Clip,
    ecran: (u32, u32),
}

impl Regime {
    pub(super) fn de(
        (vue, densite): (Viewport, f32),
        regard: Regard,
        ecran: (u32, u32),
        header_h: f32,
    ) -> Self {
        Self {
            vue,
            densite,
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

    /// **Ce que ce composant devient à l'écran** : sa texture entière si elle tient dans
    /// l'écran, ses tuiles visibles et leur repli sinon (DE-PRES-1).
    ///
    /// `coin` est le coin du contenu à l'écran, à l'échelle de la vue ; `mesure` donne, pour
    /// une échelle de rendu, la largeur, la hauteur et la marge du contenu en pixels — c'est
    /// elle qui permet de mesurer le repli à un autre palier que celui de l'image.
    fn composer(
        &self,
        kit: PaintKit<'_>,
        (prefixe, id): (&str, &str),
        coin: (f32, f32),
        mesure: &dyn Fn(f64) -> (f32, f32, f32),
        contenu: Contenu,
    ) -> Pieces {
        let identite = format!("{prefixe}:{id}");
        let contenu = Arc::new(contenu);
        let (phase, pose) = self.phase_et_coin(coin.0, coin.1);
        let entier = self.assembler(
            (identite, &contenu),
            self.echelle,
            (phase, pose),
            mesure(self.echelle),
        );
        if entier.tient_dans(self.ecran) {
            return Pieces {
                posee: Some(entier),
                ..Pieces::default()
            };
        }
        Pieces {
            posee: None,
            formes: entier.en_formes(kit, self),
            a_preparer: self.repli(&entier, coin, mesure),
        }
    }

    /// Le composant entier à l'échelle `echelle`, posé en `coin` avec cette phase.
    fn assembler(
        &self,
        (identite, contenu): (String, &Arc<Contenu>),
        echelle: f64,
        (phase, coin): ((f32, f32), (f32, f32)),
        (largeur, hauteur, marge): (f32, f32, f32),
    ) -> Composant {
        // La phase sous-pixel pousse le contenu de moins d'un pixel : la texture le compte,
        // sans quoi le dernier rang du trait anti-crenele est coupe -- vingt-cinq niveaux aux
        // coins bas d'une carte selectionnee, et c'est l'epreuve des deux voies qui l'a vu.
        let pixels = (
            (largeur + 2.0 * marge + 1.0).ceil() as u32,
            (hauteur + 2.0 * marge + 1.0).ceil() as u32,
        );
        let rapport = (self.vue.scale / echelle) as f32;
        let mut composant = Composant {
            cle: String::new(),
            identite,
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
            contenu: Arc::clone(contenu),
            echelle,
            densite: self.densite,
            phase,
            marge,
            pixels,
        };
        composant.cle = format!("{}:{:016x}", composant.identite, empreinte(&composant));
        composant
    }

    /// **La texture à préparer d'un composant dessiné en formes** : lui tout entier, au plus
    /// haut palier dyadique dont la texture tient dans l'écran.
    ///
    /// Aucun nombre n'est choisi : c'est la même limite que celle qui fait dessiner en formes,
    /// et un palier dyadique ne change pas pendant qu'on zoome plus près — elle se rend donc
    /// une fois, et attend tout le temps qu'on reste de près.
    ///
    /// Elle porte **l'identité du composant entier** : quand, dézoomé, il retient dans
    /// l'écran, c'est elle que la carte détient sous cette identité, et qu'elle pose — un
    /// palier plus bas peut-être, le temps que le sien se rende (CASCADE-2).
    fn repli(
        &self,
        entier: &Composant,
        coin: (f32, f32),
        mesure: &dyn Fn(f64) -> (f32, f32, f32),
    ) -> Option<Composant> {
        let mut palier = Adresse::echelle(Adresse::niveau_pour(self.echelle));
        // Chaque tour divise par deux ; le zéro de la virgule flottante borne la boucle, et
        // un écran sans pixel n'a de toute façon rien à montrer.
        while palier > 0.0 {
            let repli = self.assembler(
                (entier.identite.clone(), &entier.contenu),
                palier,
                ((0.0, 0.0), coin),
                mesure(palier),
            );
            if repli.tient_dans(self.ecran) {
                return Some(repli);
            }
            palier /= 2.0;
        }
        None
    }

    /// **Une carte de texte**, ou `None` si elle ne touche pas l'écran.
    pub(super) fn carte(
        &self,
        kit: PaintKit<'_>,
        id: &str,
        (x, y, w, h): (f64, f64, f32, f32),
        (corps, teinte): (&str, (u8, u8, u8)),
        (edition, contenants, eclaires): (
            Option<&crate::renderer::TextEditSession>,
            &Contenants<'_>,
            Option<super::passages::Eclaires<'_>>,
        ),
    ) -> Option<Pieces> {
        // MODE-1 : une carte qu'on corrige montre ses signes, une carte qu'on lit ne les
        // montre pas -- et le decoupage en lignes n'est pas le meme dans les deux modes.
        let mode = if edition.is_some() {
            TextMode::Source
        } else {
            TextMode::Rendered
        };
        // La hauteur suit le texte : une carte ne tronque jamais son contenu (TEXT-FIT-1).
        //
        // Une carte qu'on edite est toujours une texture : ce qui suit son curseur -- le trait,
        // la previsualisation d'une formule -- se pose au-dessus (COMPOSANT-3).
        let lignes = kit
            .mises_en_page
            .de((kit.typography, kit.math), corps, w, mode)
            .line_count();
        let vue = CardLayout::text_card(w, h, lignes)
            .scaled(WorldScale::new(self.vue.scale, self.densite));
        let (sx, sy) = world_to_screen(x, y, &self.vue);
        let (sx, sy) = (sx as f32, sy as f32);
        if self.clip.rejects(sx, sy, vue.width, vue.height) {
            return None;
        }
        let mesure = |echelle: f64| {
            let a_cette_echelle = WorldScale::new(echelle, self.densite);
            let rendu = CardLayout::text_card(w, h, lignes).scaled(a_cette_echelle);
            let anneau = a_cette_echelle.screen(SELECTION_RING);
            let marge = (rendu.border.max(anneau) / 2.0).ceil() + 1.0;
            (rendu.width, rendu.height, marge)
        };
        Some(
            self.composer(
                kit,
                ("carte", id),
                (sx, sy),
                &mesure,
                Contenu::Carte {
                    origine: (x, y),
                    taille: (w, h),
                    corps: corps.to_string(),
                    teinte,
                    fond: contenants.fond_en((sx + vue.width / 2.0, sy + vue.height / 2.0)),
                    // Pendant l'édition, la carte montre sa source : rien n'y brille.
                    eclaires: eclaires
                        .filter(|_| edition.is_none())
                        .map(|e| contenu::Passages {
                            plages: e.plages.to_vec(),
                            teinte: e.teinte,
                        }),
                    edition: edition.cloned(),
                },
            ),
        )
    }

    /// **Une photo dont les octets ne sont pas encore là**, comme un cadre à son rang.
    ///
    /// La texture est la boîte englobante du cadre **tourné** : le libellé, lui, reste droit,
    /// comme le processeur le dessine — la texture se pose donc sans angle.
    pub(super) fn photo_en_chemin(&self, kit: PaintKit<'_>, img: &BoardImage) -> Option<Pieces> {
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
        let taille = (img.width, img.height);
        let mesure = |echelle: f64| {
            let (_, englobante) = cadre_en_chemin(taille, img.rotation, echelle);
            (englobante.0, englobante.1, 2.0)
        };
        // Le centre de la photo à l'écran, d'où le coin de la boîte englobante se déduit. La
        // boîte est prise à l'échelle de RENDU puis ramenée à la vue, exactement comme avant
        // le découpage : les deux voies gardent le même coin, au bit près.
        let rapport = self.rapport();
        let (_, englobante) = cadre_en_chemin(taille, img.rotation, self.echelle);
        let centre = (sx + sw / 2.0, sy + sh / 2.0);
        let coin = (
            centre.0 - englobante.0 / 2.0 * rapport,
            centre.1 - englobante.1 / 2.0 * rapport,
        );
        Some(self.composer(
            kit,
            ("chemin", &img.id),
            coin,
            &mesure,
            Contenu::PhotoEnChemin {
                id: img.id.clone(),
                centre: (img.x, img.y),
                taille,
                rotation: img.rotation,
            },
        ))
    }
}

impl Composant {
    /// **Rend le composant dans son propre tampon**, exactement comme le processeur le
    /// dessine en place — même passe, même mise en page, même anti-crénelage — la vue étant
    /// celle qui pose son coin sur la marge, à sa phase.
    pub fn rendre(&self, kit: PaintKit<'_>) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(self.pixels.0, self.pixels.1)?;
        let coin = (self.marge + self.phase.0, self.marge + self.phase.1);
        match self.contenu.as_ref() {
            Contenu::Carte { origine, .. } => {
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
                    mises_en_page: kit.mises_en_page,
                    tints: kit.tints,
                    theme: kit.theme,
                    vp,
                    scale: WorldScale::new(self.echelle, self.densite),
                    clip: Clip {
                        width: self.pixels.0 as f32,
                        height: self.pixels.1 as f32,
                        top: 0.0,
                    },
                    poignees: true,
                };
                if let Some(carte) = self.contenu.carte() {
                    draw_card_contenu(&ctx, &mut pixmap.as_mut(), carte);
                }
            }
            Contenu::PhotoEnChemin {
                id,
                taille,
                rotation,
                ..
            } => {
                let (taille, englobante) = cadre_en_chemin(*taille, *rotation, self.echelle);
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
                    taille,
                    id,
                    *rotation,
                );
            }
        }
        Some(pixmap)
    }

    /// Sa texture tient-elle dans l'écran ? C'est la limite qui le fait dessiner en formes.
    fn tient_dans(&self, ecran: (u32, u32)) -> bool {
        self.pixels.0 <= ecran.0 && self.pixels.1 <= ecran.1
    }
}

#[cfg(test)]
impl Pieces {
    /// **La texture unique d'un composant qui tient dans l'écran** : ni formes, ni texture à
    /// préparer.
    fn seule(self) -> Composant {
        assert!(
            self.formes.is_empty() && self.a_preparer.is_none(),
            "un composant qui tient dans l'ecran est une texture, et une seule"
        );
        self.posee
            .expect("la texture d'un composant qui tient dans l'ecran")
    }
}

#[cfg(test)]
mod tests;
