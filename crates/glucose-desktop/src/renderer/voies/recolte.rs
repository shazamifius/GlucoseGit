//! **Ce que la passe graphique récolte** : les photos et les cartes de texte que l'écran montre,
//! chacune comme une texture à son rang — ou, plus grande que l'écran, comme des formes (fiche
//! 73) —, et tout ce qui sait se rendre.
//!
//! Extrait de [`super`] quand les formes y sont entrées : six cent quatre lignes là où la
//! fiche 05 en admet six cents. La coupure tombe là où la question change — ici *ce qui part
//! vers la carte*, là-bas *ce que le processeur peint*.

use super::cran;
use super::tenue::pose_tenue;
use super::APoser;
use crate::params::ViewPass;
use crate::present::formes_gpu::{Formes, Suite, Tranche};
use crate::present::scene_gpu::Pose;
use crate::renderer::composants::{Composant, Pieces, Regime};
use crate::renderer::{PaintKit, SymbioticHueCache};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::Viewport;

/// **Ce qu'une tranche récolte** : ce qui se pose, ce qui se dessine en formes, ce qui se
/// prépare, et tout ce qui sait se rendre.
#[derive(Default)]
pub(in crate::renderer) struct Recolte {
    /// Où chaque texture se pose, dans l'ordre du modèle.
    pub posees: Vec<(String, Pose)>,
    /// Ce qui se rend **à la demande** : les cartes de texte, les photos en chemin, et leur
    /// texture à préparer.
    pub composants: Vec<Composant>,
    /// Les formes de ce qui est plus grand que l'écran, chacune à son rang (fiche 73).
    pub formes: Formes,
    /// **Ce qui se prépare dans le temps qui reste**, sans jamais se poser : la texture d'un
    /// composant dessiné en formes, prête pour le dézoom.
    pub a_preparer: Vec<APoser>,
}

impl Recolte {
    /// **Range les pièces d'un composant** dans cette tranche : sa texture à son rang, ou ses
    /// formes juste avant la texture qui le suivra ; et ce qui sait se rendre.
    fn ranger(&mut self, pieces: Pieces, tranche: Tranche) {
        if let Some(c) = pieces.posee {
            self.posees.push((c.cle.clone(), c.pose));
            self.composants.push(c);
        }
        if !pieces.formes.is_empty() {
            let debut = self.formes.primitives.len();
            self.formes.primitives.extend(pieces.formes);
            self.formes.suites.push(Suite {
                tranche,
                avant: self.posees.len(),
                primitives: debut..self.formes.primitives.len(),
            });
        }
        if let Some(c) = pieces.a_preparer {
            self.a_preparer.push(APoser {
                cle: c.cle.clone(),
                identite: c.identite.clone(),
                pose: c.pose,
                repli: None,
            });
            self.composants.push(c);
        }
    }
}

impl Recolte {
    /// **La récolte des photos, puis celle des cartes, confiées à la carte** : les poses de
    /// chacune dans sa tranche, et tout le reste mis bout à bout, dans l'ordre du modèle.
    pub fn puis_les_cartes(self, cartes: Recolte, confie: super::Confie) -> super::Confie {
        let mut composants = self.composants;
        composants.extend(cartes.composants);
        let mut formes = self.formes;
        formes.etendre(cartes.formes);
        let mut a_preparer = self.a_preparer;
        a_preparer.extend(cartes.a_preparer);
        super::Confie {
            photos: self.posees,
            cartes: cartes.posees,
            composants,
            formes,
            a_preparer,
            ..confie
        }
    }
}

/// **Ou chaque photo visible se pose a l'ecran**, en pixels et en radians -- et, pour
/// celles dont les octets ne sont pas encore la, le composant qui les dessinera.
///
/// # Ce que cette fonction est, et ce qu'elle n'est pas
///
/// Elle ne dessine rien. Elle traduit ce que le **modele** dit en ce que la carte attend :
/// un rectangle d'ecran et un angle par photo. C'est exactement la frontiere que la fiche 21
/// trace entre le socle -- le *quoi* -- et les executants -- le *comment*.
///
/// Le culling a deja fait son travail, donc `rangs` ne designe que ce qui touche l'ecran.
///
/// **Reclamer, sinon rien n'est jamais decode.** Le magasin ne decode que ce qu'on lui
/// demande, et il oublie ce qu'on ne lui redemande pas. C'est ici que chaque photo visible se
/// reclame, et `reclamer` dit du meme coup si elle est la : sinon, elle se pose comme un
/// cadre en chemin, **a son rang** -- ni dessous ni dessus les autres, exactement ou la voie
/// processeur la dessine.
///
/// L'angle est celui du modele, en **radians** : le multiplier par pi sur cent quatre-vingts
/// posait droite une photo penchee de pi sur huit, et aucune epreuve ne le voyait parce que
/// la photo penchee du temoin est en chemin.
pub(in crate::renderer) fn poses_des_photos(
    (regime, kit): (&Regime, PaintKit<'_>),
    (magasin, carte): (
        &mut crate::renderer::magasin::Magasin,
        &mut cran::EtatDeLaCarte,
    ),
    (vp, rangs, store): (&Viewport, &[u32], &Store),
) -> PhotosAPoser {
    // Le cran de cette image d'abord, sur la vue qu'elle montre (ETAGES-3).
    carte.juger(magasin, store, (vp, rangs));
    let mut photos = PhotosAPoser::default();
    let Some(board) = store.active_board() else {
        return photos;
    };
    let mut en_chemin = 0.0f64;
    for img in Visibles::nouvelles(rangs, board).images() {
        match pose_tenue(magasin, img, (vp, carte.cran, carte.plafond_des_textures)) {
            Some(tenue) => {
                let cle = format!("{}@{}", tenue.src, tenue.facteur);
                if let Some((facteur, pose)) = tenue.repli {
                    let cle_du_repli = format!("{}@{facteur}", tenue.src);
                    photos
                        .niveaux
                        .insert(cle_du_repli.clone(), (tenue.src.clone(), facteur));
                    photos.replis.insert(cle.clone(), (cle_du_repli, pose));
                }
                photos
                    .niveaux
                    .insert(cle.clone(), (tenue.src, tenue.facteur));
                photos.recolte.posees.push((cle, tenue.pose));
            }
            None => {
                if let Some(pieces) = regime.photo_en_chemin(kit, img) {
                    photos.recolte.ranger(pieces, Tranche::Photos);
                    en_chemin += 1.0;
                }
            }
        }
    }
    crate::perf::compteur("photos_en_chemin", en_chemin);
    photos
}

/// **Ce que la passe des photos confie à la carte** : où chacune se pose, ce qui sait se
/// rendre pour celles en chemin, et le niveau de chacune de celles qui sont là.
#[derive(Default)]
pub(in crate::renderer) struct PhotosAPoser {
    pub recolte: Recolte,
    pub niveaux: std::collections::HashMap<String, (String, u32)>,
    pub replis: std::collections::HashMap<String, (String, Pose)>,
}

/// **Les cartes de texte que l'ecran montre**, comme composants (COMPOSANT-1).
///
/// La teinte symbiotique se calcule ici, comme dans `draw_annotations` : c'est le meme
/// parcours, au meme endroit, et c'est elle qui entre dans l'empreinte.
///
/// # La carte qu'on edite en est une aussi (COMPOSANT-2)
///
/// Elle en etait exclue, et elle se redessinait donc en entier a chaque image : le terrain du
/// 22/09 la chiffre a 9,74 ms en median sur le geste « editer du texte », dont l'image
/// mediane coute 19,48 ms. **Cinquante et une images par seconde pendant qu'on ecrit**, la ou
/// la charte en demande cent -- et ce n'est pas la frappe qui coute, c'est de refaire a
/// l'identique entre deux touches.
///
/// Ce qu'elle montre ne change qu'a la frappe et deux fois par seconde pour le curseur : son
/// empreinte le dit, et la texture ne se refait que la. Elle se pose **en dernier**, donc
/// au-dessus des autres cartes, ce qui est exactement le rang que le processeur lui donnait
/// en la dessinant dans sa seconde passe.
pub(in crate::renderer) fn composants_de_texte(
    (regime, contenants, eclairages): (
        &Regime,
        &crate::renderer::card::Contenants<'_>,
        &[crate::params::Eclairage],
    ),
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    (store, pass, edition): (
        &Store,
        ViewPass<'_>,
        Option<&crate::renderer::TextEditSession>,
    ),
) -> Recolte {
    let mut recolte = Recolte::default();
    let Some(board) = store.active_board() else {
        return recolte;
    };
    // Ce qu'on edite passe au-dessus des autres cartes : on le met de cote et on l'ajoute
    // apres, plutot que de trier une liste dont l'ordre est deja celui du modele.
    let mut en_saisie = None;
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let glucose_core::types::Annotation::Text { x, y, text, .. } = ann else {
            continue;
        };
        let saisie = edition.filter(|e| e.ann_id == ann.id());
        let Some((w, h)) = ann.size() else {
            continue;
        };
        let teinte = crate::renderer::card::teinte_de_carte(hue_cache, ann, (pass.index, board));
        // Le tampon de saisie remplace le texte enregistre : c'est ce qu'on voit a l'ecran
        // pendant qu'on tape, et c'est ce que `carte_de` fait deja sur la voie processeur.
        let corps = saisie.map_or(text.as_str(), |e| e.buffer.as_str());
        let Some(pieces) = regime.carte(
            kit,
            ann.id(),
            (*x, *y, w as f32, h as f32),
            (corps, teinte),
            (
                saisie,
                contenants,
                crate::renderer::passages::eclaires_de(eclairages, ann.id(), teinte),
            ),
        ) else {
            continue;
        };
        if saisie.is_some() {
            en_saisie = Some(pieces);
        } else {
            recolte.ranger(pieces, Tranche::Cartes);
        }
    }
    if let Some(pieces) = en_saisie {
        recolte.ranger(pieces, Tranche::Cartes);
    }
    recolte
}
