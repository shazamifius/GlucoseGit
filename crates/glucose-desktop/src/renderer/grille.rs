//! La passe des images **par la grille de tuiles** — TUILE-1, branché au rendu.
//!
//! # Ce que cette passe remplace, et pourquoi
//!
//! La passe des images posait chaque photo à chaque image, depuis sa pyramide ou depuis une
//! vignette à la forme exacte. Sur le terrain : quatre cent vingt-neuf photos, `redessine
//! 100 %`, `report` à trente millisecondes — pour une scène qui, entre deux images, a bougé
//! de quatre pixels. Et le cache de vignettes, indexé par une phase sous-pixel, servait à
//! zéro pour cent.
//!
//! Une tuile est ancrée au **monde**, pas à l'écran : déplacer la vue ne change rien à ses
//! pixels, seulement à sa place. Ce qui coûtait trente millisecondes coûte alors une
//! composition — `bench_grille` mesure 0,18 ms à chaud, cinquante-deux fois moins.
//!
//! **C'est la variance du coût qui tombe, et c'est elle qui faisait le judder.** Une image à
//! six millisecondes suivie d'une à soixante-sept posait le contenu soixante pixels à côté de
//! sa trajectoire ; des images qui coûtent toutes la même chose le posent où il doit être.
//!
//! # Les trois régimes, et ce qui les décide
//!
//! L'échelle de la vue tombe entre deux niveaux dyadiques, et c'est elle qui décide :
//!
//! * **exacte** — l'échelle est une puissance de deux. Les tuiles se composent pixel pour
//!   pixel, au bit près identiques à un rendu direct (`bench_grille`) ;
//! * **au plus proche** — l'échelle est entre deux niveaux, et l'œil tolère qu'on abîme
//!   l'image ([`crate::perception`]). La tuile du niveau inférieur s'agrandit d'un facteur
//!   entre un et deux, au texel le plus proche : 1,7 ms pour un écran, contre dix fois plus
//!   en interpolant. Jamais le filtre lisse ici — il ne tient pas dans le budget ;
//! * **au plus proche, encore** — l'échelle est entre deux niveaux, l'œil ne tolère plus,
//!   mais **la vue bouge encore** : la fin d'un freinage. Repasser ici au rendu direct ferait
//!   sauter le contenu de soixante pixels, ce qui se voit infiniment plus qu'un agrandissement
//!   d'un facteur un virgule trois. C'est un choix de ressenti, écrit dans [`Regard`], et il
//!   se juge à l'écran ;
//! * **direct** — l'échelle est entre deux niveaux et plus rien ne bouge : l'arrêt. L'ancienne
//!   passe reprend la main, avec ses vignettes, et la finesse est intégrale. C'est le seul
//!   régime où le coût reste celui d'hier, et il ne se produit qu'une fois par arrêt si la
//!   salissure tient sa promesse.
//!
//! # Ce que la grille ne contient pas
//!
//! Rien de ce qui n'appartient pas au document : ni le cadre de sélection, ni les poignées,
//! ni la jauge de domaines, ni le carré d'une image encore en chemin. Les trois premiers se
//! dessinent par-dessus, en direct ; le dernier fait que la tuile n'est **pas gardée**, pour
//! qu'elle se repeigne dès que les octets arrivent.

use super::scale::WorldScale;
use super::scene::image::{draw_image_ornaments, draw_images, PasseImages};
use super::tuiles::{a_l_ecran, ce_que_porte, Portee, Tuiles, COTE_TUILE};
#[allow(unused_imports)]
use super::Regard;
use super::{Cadrage, PaintKit};
use crate::canvas::{screen_to_world, world_to_screen};
use crate::params::ViewPass;
use glucose_core::cout::Cout;
use glucose_core::occlusion::Boite;
use glucose_core::quadtree::{SpatialHash, Visibles};
use glucose_core::report::{reporter, Filtre, Melange, Pose, Vue, VueMut};
use glucose_core::store::Store;
use glucose_core::tuile::{Adresse, Empreinte};
use glucose_core::types::Viewport;
use tiny_skia::{Pixmap, PixmapMut};

/// Par quel chemin les images se posent sous cette vue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Regime {
    /// L'échelle est dyadique : les tuiles se composent pixel pour pixel.
    Exact,
    /// Entre deux niveaux, et l'œil tolère : la tuile s'agrandit au texel le plus proche.
    PlusProche,
    /// Entre deux niveaux, et l'œil ne tolère rien : la passe directe.
    Direct,
}

impl Regime {
    /// Le régime que cette vue commande, sous ce cadrage.
    ///
    /// Une tuile ne se rend jamais par la grille — ce serait se rendre soi-même — ni une
    /// scène réduite, qui est déjà une pixelisation pilotée par la perception.
    pub(super) fn pour(cadrage: Cadrage, vp: Viewport) -> Self {
        if cadrage.vue.is_some() || cadrage.reduction > 1.0 {
            return Self::Direct;
        }
        let niveau = Adresse::niveau_pour(vp.scale);
        let facteur = vp.scale / Adresse::echelle(niveau);
        if facteur == 1.0 {
            return Self::Exact;
        }
        // **Le mouvement, et non la tolerance de l'oeil.** La perception refuse de degrader
        // des que la vitesse passe sous celle de la poursuite -- donc sur toute la fin d'un
        // freinage, qui est precisement l'endroit ou repasser a trente millisecondes par
        // image ferait sauter le contenu de soixante pixels. Un agrandissement d'un facteur
        // un virgule trois se voit moins qu'un tel saut ; la finesse revient a l'arret.
        if cadrage.en_mouvement || cadrage.degradation_permise {
            return Self::PlusProche;
        }
        Self::Direct
    }
}

/// Ce que la passe par la grille a besoin d'emprunter au moteur, champ par champ.
///
/// Séparés parce que le moteur ne peut pas se prêter entier : rendre une tuile emprunte le
/// magasin et le modèle de coût en écriture pendant que le kit emprunte la typographie en
/// lecture. Le compilateur l'autorise sur des champs distincts, jamais à travers `&mut self`.
pub(super) struct Atelier<'a> {
    pub magasin: &'a mut super::magasin::Magasin,
    pub cout: &'a mut Cout,
    pub tuiles: &'a mut Tuiles,
    pub index: &'a SpatialHash,
    pub kit: PaintKit<'a>,
}

/// Pose les images : par la grille quand la vue le permet, en direct sinon.
pub(super) fn poser_les_images(
    atelier: &mut Atelier<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
    cadrage: Cadrage,
) {
    let regime = Regime::pour(cadrage, pass.vp);
    if regime == Regime::Direct {
        draw_images(
            atelier.magasin,
            atelier.cout,
            atelier.kit,
            pixmap,
            store,
            pass,
            PasseImages {
                degradation_permise: cadrage.degradation_permise,
                en_tuile: false,
            },
        );
        crate::perf::compteur("tuiles_peintes", 0.0);
        crate::perf::compteur("tuiles_reprises", 0.0);
        return;
    }
    let (peintes, reprises) = poser_par_la_grille(atelier, pixmap, store, pass, regime);
    crate::perf::compteur("tuiles_peintes", peintes as f64);
    crate::perf::compteur("tuiles_reprises", reprises as f64);
}

/// Pose les images de l'écran depuis la grille, et rend combien de tuiles ont été peintes et
/// reprises pendant cette image.
///
/// Le pixmap porte déjà le fond, la grille et les halos : les tuiles se **composent** dessus,
/// jamais ne le remplacent — une tuile est transparente partout où aucune photo ne passe.
fn poser_par_la_grille(
    atelier: &mut Atelier<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
    regime: Regime,
) -> (u64, u64) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let ecran = (largeur as f32, hauteur as f32);
    let clip = Boite::nouvelle(0.0, pass.header_h, ecran.0, ecran.1 - pass.header_h);
    let (niveau, adresses) = a_l_ecran(pass.vp, ecran);
    let facteur = pass.vp.scale / Adresse::echelle(niveau);
    let cote_ecran = f64::from(COTE_TUILE) * facteur;
    let filtre = match regime {
        Regime::Exact => Filtre::Lisse,
        _ => Filtre::PlusProche,
    };

    // Ce qui est à l'écran reste hors d'atteinte de l'éviction, même quand aucune photo n'est
    // posée par la passe directe : sans cela, le magasin rendrait à la machine des images
    // qu'il faudrait redécoder à la première tuile invalidée.
    if let Some(board) = store.active_board() {
        for img in Visibles::nouvelles(pass.visibles, board).images() {
            if let Some(src) = img.src.as_deref() {
                atelier.magasin.reclamer(src);
            }
        }
    }

    let (peintes_avant, reprises_avant) = (atelier.tuiles.peintes(), atelier.tuiles.reprises());
    atelier.tuiles.ouvrir();
    let mut pixels: u64 = 0;
    for adresse in adresses {
        let empreinte = ce_que_porte(store, adresse, pass.visibles);
        // Une tuile vide n'a rien à composer : c'est le cas le plus fréquent d'un canevas
        // infini, et c'est lui qui rend le déplacement sur du vide gratuit.
        if empreinte == Empreinte::vide() {
            continue;
        }
        let place = Place {
            adresse,
            vp: pass.vp,
            cote_ecran,
            clip,
            filtre,
        };
        if let Some((deja, portee)) = atelier.tuiles.deja_peinte(empreinte) {
            pixels += composer(pixmap, deja, portee, place);
            continue;
        }
        let Some((peinte, complete)) = rendre_une_tuile(atelier, store, adresse) else {
            continue;
        };
        // Une tuile dont une photo manquait encore ne se garde pas : elle se repeindra à
        // l'image où les octets seront là, et le cadre « en chemin » n'aura pas survécu.
        if complete {
            let portee = atelier.tuiles.ranger(empreinte, peinte);
            if let Some((deja, _)) = atelier.tuiles.deja_peinte(empreinte) {
                pixels += composer(pixmap, deja, portee, place);
            }
        } else {
            pixels += composer(pixmap, &peinte, Portee::de(&peinte), place);
        }
    }
    atelier.tuiles.fermer();
    crate::perf::stage("grille");

    dessiner_les_ornements(atelier.kit, pixmap, store, pass);
    noter_ce_que_l_ecran_a_recu(pixmap, store, pass, pixels);
    (
        atelier.tuiles.peintes() - peintes_avant,
        atelier.tuiles.reprises() - reprises_avant,
    )
}

/// Rend une tuile dans son propre repère : les photos qui la traversent, et rien d'autre.
///
/// Rend aussi si **toutes** ces photos étaient décodées. Sinon, la tuile montre un cadre
/// en chemin, et l'appelant ne doit pas la garder.
fn rendre_une_tuile(
    atelier: &mut Atelier<'_>,
    store: &Store,
    adresse: Adresse,
) -> Option<(Pixmap, bool)> {
    let cote = COTE_TUILE;
    let mut pixmap = Pixmap::new(cote, cote)?;
    let couverte = adresse.couvre();
    let cadrage = Cadrage::tuile(adresse.niveau, (couverte.left, couverte.top));
    let vp = cadrage.vue?;
    let (min_wx, min_wy) = screen_to_world(0.0, 0.0, &vp);
    let (max_wx, max_wy) = screen_to_world(f64::from(cote), f64::from(cote), &vp);
    let rangs = atelier
        .index
        .query_rect_ranks(min_wx, min_wy, max_wx, max_wy, 200.0);
    let pass = ViewPass {
        vp,
        visibles: &rangs,
        index: atelier.index,
        header_h: 0.0,
    };
    let complete = draw_images(
        atelier.magasin,
        atelier.cout,
        atelier.kit,
        &mut pixmap.as_mut(),
        store,
        pass,
        PasseImages {
            degradation_permise: false,
            en_tuile: true,
        },
    );
    Some((pixmap, complete))
}

/// Où et comment une tuile se pose à l'écran.
#[derive(Clone, Copy)]
struct Place {
    adresse: Adresse,
    vp: Viewport,
    cote_ecran: f64,
    clip: Boite,
    filtre: Filtre,
}

/// Compose une tuile à sa place à l'écran, et rend combien de pixels ont été écrits.
///
/// # Seulement ce que la tuile porte, et en le remplaçant quand c'est opaque
///
/// Composer chaque tuile en entier, en source-over, coûtait cinq millisecondes pour un écran
/// de 2560 × 1600 — le report lisait et mélangeait des millions de pixels transparents. La
/// portée mesurée au rangement borne le parcours à ce qui existe, et une boîte opaque se
/// **remplace** : un déplacement de mémoire par ligne, la primitive la moins chère du
/// programme.
fn composer(pixmap: &mut PixmapMut, tuile: &Pixmap, portee: Portee, place: Place) -> u64 {
    let Some((bx0, by0, bx1, by1)) = portee.boite else {
        return 0;
    };
    let couverte = place.adresse.couvre();
    let (x, y) = world_to_screen(couverte.left, couverte.top, &place.vp);
    let (dw, dh) = (pixmap.width(), pixmap.height());
    let (octets_src, _) = tuile.data().as_chunks::<4>();
    let Some(src) = Vue::nouvelle(octets_src, tuile.width(), tuile.height()) else {
        return 0;
    };
    let (octets_dest, _) = pixmap.data_mut().as_chunks_mut::<4>();
    let Some(mut dest) = VueMut::nouvelle(octets_dest, dw, dh) else {
        return 0;
    };
    // La position est arrondie au pixel : c'est ce qui ouvre le chemin exact du report, un
    // pixel pour un pixel. Le contenu avance donc par pixels entiers pendant un glissement —
    // ce que fait tout canevas à tuiles, et ce qu'un écran ne peut de toute façon pas
    // montrer autrement.
    let pose = Pose {
        x: (x as f32).round(),
        y: (y as f32).round(),
        largeur: place.cote_ecran.round() as f32,
        hauteur: place.cote_ecran.round() as f32,
    };
    // La boîte utile de la tuile, portée à l'écran : c'est elle qui borne le parcours.
    let facteur = place.cote_ecran / f64::from(COTE_TUILE);
    let utile = Boite::nouvelle(
        pose.x + (f64::from(bx0) * facteur).floor() as f32,
        pose.y + (f64::from(by0) * facteur).floor() as f32,
        (f64::from(bx1 - bx0) * facteur).ceil() as f32,
        (f64::from(by1 - by0) * facteur).ceil() as f32,
    );
    let clip = intersection(place.clip, utile);
    let melange = if portee.opaque {
        Melange::Remplacer
    } else {
        Melange::Composer
    };
    reporter(&mut dest, &src, pose, clip, melange, place.filtre)
}

/// L'intersection de deux boîtes, vide si elles ne se touchent pas.
fn intersection(a: Boite, b: Boite) -> Boite {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.largeur).min(b.x + b.largeur);
    let y1 = (a.y + a.hauteur).min(b.y + b.hauteur);
    Boite::nouvelle(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

/// Ce qui n'appartient pas au document, par-dessus les tuiles : cadre de sélection,
/// poignées, jauge de domaines.
fn dessiner_les_ornements(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let scale = WorldScale::new(pass.vp.scale);
    for img in Visibles::nouvelles(pass.visibles, board).images() {
        let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        draw_image_ornaments(
            kit,
            pixmap,
            store,
            scale,
            img,
            (wx as f32, wy as f32, sw, sh),
        );
    }
}

/// Les compteurs de la chronique, tels que l'**écran** les vit — et non la dernière tuile.
///
/// Chaque tuile rendue passe par la passe directe, qui déclare ses propres compteurs : sans
/// cette reprise, la trace dirait que l'image a posé deux photos alors que l'écran en montre
/// quatre cents.
fn noter_ce_que_l_ecran_a_recu(pixmap: &PixmapMut, store: &Store, pass: ViewPass<'_>, pixels: u64) {
    let fenetre = f64::from(pixmap.width()) * f64::from(pixmap.height());
    let posees = store.active_board().map_or(0, |b| {
        Visibles::nouvelles(pass.visibles, b).images().count()
    });
    crate::perf::compteur("img_n", posees as f64);
    crate::perf::compteur("img_ecrans", pixels as f64 / fenetre.max(1.0));
    crate::perf::compteur("img_par_vignette", 0.0);
    crate::perf::compteur("img_pixelise", 0.0);
}

#[cfg(test)]
mod tests;
