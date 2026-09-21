//! **Composer l'ecran a partir des tuiles deja peintes, en bandes paralleles.**
//!
//! # Pourquoi ce module existe a part
//!
//! La grille fait deux travaux qui ne demandent pas la meme chose du cache : peindre
//! l'ECRIT, composer le LIT. Tant qu'ils etaient entrelaces dans une seule boucle, celle-ci
//! tenait le cache en ecriture du debut a la fin, et un seul fil pouvait y toucher.
//!
//! Separe, tout ce qui est ici ne lit que des pixels deja la -- et seize fils peuvent le
//! faire ensemble. C'est ce que `bench_bandes` a mesure : **21,78 ms sur un fil, 2,38 ms sur
//! seize**, pour la meme image interpolee de 2560 x 1600. Le repere qui compte : le meme
//! ecran PIXELISE coute 3,19 ms sur un seul fil. Interpoler sur toute la machine est donc
//! moins cher que pixeliser sur un coeur, et c'est ce qui retire a la pixelisation sa
//! derniere justification.

use super::super::fils::bandes_utiles;
use super::{Couverture, Place, Portee, Tuiles, COTE_TUILE};
use crate::canvas::world_to_screen;
use glucose_core::occlusion::Boite;
use glucose_core::report::{reporter, Melange, Pose, Vue, VueMut};
use glucose_core::tuile::{Adresse, Empreinte};
use tiny_skia::{Pixmap, PixmapMut};

/// **Compose toutes les tuiles de l'écran, en bandes horizontales parallèles.**
///
/// # Ce que la mesure a dit, et pourquoi ce découpage existe
///
/// Sur 2560 × 1600, entre deux niveaux dyadiques, `bench_tuiles` mesurait :
///
/// ```text
///     au texel le plus proche    3,19 ms     — pixelisé, et c'est ce qu'on faisait
///     interpolé                 20,68 ms     — net, et six fois trop cher
/// ```
///
/// D'où la pixelisation en mouvement, que l'utilisateur voit et qui « casse complètement
/// cette idée de smooth ». Mais ces vingt millisecondes n'étaient la limite de rien : elles
/// étaient celles d'**un cœur sur seize, en scalaire**. `bench_bandes` a mesuré le même
/// travail découpé — 21,78 ms sur un fil, **2,38 ms sur seize**. Interpoler sur toute la
/// machine coûte donc moins que pixeliser sur un seul cœur.
///
/// # Pourquoi des bandes, et pourquoi c'est sûr
///
/// Une bande possède ses lignes et personne d'autre n'y écrit : le découpage porte sur la
/// **destination**, jamais sur les tuiles. Une tuile à cheval est posée par les deux bandes,
/// chacune sur sa part, chacune avec son clip — et le compilateur vérifie lui-même qu'aucun
/// fil ne voit les pixels d'un autre, puisque les tranches sont disjointes.
///
/// Le cache, lui, n'est que **lu** ici ([`Tuiles::lire`]) : tout ce qui devait s'y écrire l'a
/// été par `peindre_ce_qui_manque`, avant.
pub(super) fn composer_en_bandes(
    pixmap: &mut PixmapMut,
    tuiles: &Tuiles,
    releve: &Couverture,
    provisoires: &[(Adresse, Pixmap, Portee)],
    modele: Place,
) -> u64 {
    let combien = releve.tuiles.len() + provisoires.len();
    let fils = bandes_utiles(combien, pixmap.height());
    composer_en(fils, pixmap, tuiles, releve, provisoires, modele)
}

/// La même composition, en un nombre de bandes **imposé**.
///
/// # Pourquoi ce paramètre existe, alors que personne ne le choisit en production
///
/// La charte demande que deux voies d'une même opération produisent les mêmes pixels, **au
/// bit près** : c'est ce qui interdit la dégradation silencieuse et garde le rendu testable.
/// Ici les voies sont « un fil » et « seize », et rien ne le prouverait si le nombre de
/// bandes venait toujours de la machine — un test tournerait sur celle qui l'exécute, et
/// dirait seize sur l'une, deux sur l'autre, un sur l'intégration continue.
pub(super) fn composer_en(
    fils: usize,
    pixmap: &mut PixmapMut,
    tuiles: &Tuiles,
    releve: &Couverture,
    provisoires: &[(Adresse, Pixmap, Portee)],
    modele: Place,
) -> u64 {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let (octets, _) = pixmap.data_mut().as_chunks_mut::<4>();
    if fils <= 1 {
        let Some(mut dest) = VueMut::nouvelle(octets, largeur, hauteur) else {
            return 0;
        };
        return composer_dans(&mut dest, 0.0, tuiles, releve, provisoires, modele);
    }
    let par_bande = (hauteur as usize).div_ceil(fils);
    std::thread::scope(|portee| {
        let mut mains = Vec::with_capacity(fils);
        let mut haut = 0u32;
        for bande in octets.chunks_mut(par_bande * largeur as usize) {
            let h = (bande.len() / largeur as usize) as u32;
            mains.push(portee.spawn(move || {
                let Some(mut dest) = VueMut::nouvelle(bande, largeur, h) else {
                    return 0;
                };
                composer_dans(&mut dest, haut as f32, tuiles, releve, provisoires, modele)
            }));
            haut += h;
        }
        mains.into_iter().map(|m| m.join().unwrap_or(0)).sum()
    })
}

/// Toutes les tuiles de l'écran, posées dans une vue qui commence à `decalage`.
fn composer_dans(
    dest: &mut VueMut<'_>,
    decalage: f32,
    tuiles: &Tuiles,
    releve: &Couverture,
    provisoires: &[(Adresse, Pixmap, Portee)],
    modele: Place,
) -> u64 {
    let mut pixels = 0;
    for (adresse, empreinte) in &releve.tuiles {
        if *empreinte == Empreinte::vide() {
            continue;
        }
        if let Some((deja, portee)) = tuiles.lire(*empreinte) {
            let place = Place {
                adresse: *adresse,
                ..modele
            };
            pixels += composer(dest, deja, portee, place, decalage);
        }
    }
    // Les provisoires passent après, et l'ordre ne change rien : deux tuiles d'un même
    // niveau ne se recouvrent jamais — c'est la définition d'une grille.
    for (adresse, peinte, portee) in provisoires {
        let place = Place {
            adresse: *adresse,
            ..modele
        };
        pixels += composer(dest, peinte, portee, place, decalage);
    }
    pixels
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
fn composer(
    dest: &mut VueMut<'_>,
    tuile: &Pixmap,
    portee: &Portee,
    place: Place,
    decalage: f32,
) -> u64 {
    let Some((bx0, by0, bx1, by1)) = portee.boite else {
        return 0;
    };
    let couverte = place.adresse.couvre();
    let (x, y) = world_to_screen(couverte.left, couverte.top, &place.vp);
    let (octets_src, _) = tuile.data().as_chunks::<4>();
    // La tuile connaît ses plages : à l'échelle exacte, la composition ne lit plus un alpha.
    let Some(src) = Vue::nouvelle(octets_src, tuile.width(), tuile.height())
        .map(|v| v.avec_plages(&portee.plages))
    else {
        return 0;
    };
    // **Les bords se partagent, ils ne s'arrondissent pas chacun de son côté.**
    //
    // La première version posait chaque tuile à `round(x)` avec une largeur
    // `round(côté)`. Entre deux niveaux, le côté n'est pas entier : `x + 332,4` arrondit à
    // 342 pour une tuile et la suivante commence à 343 — un pixel de fond entre les deux, et
    // l'utilisateur voit des « croix noires se former » dès qu'il bouge lentement, quand les
    // arrondis basculent image après image.
    //
    // Le bord droit de cette tuile est le bord gauche de la suivante : c'est le **même
    // nombre**, et il s'arrondit une fois. La largeur en découle. Une tuile fait alors 332 ou
    // 333 pixels selon sa place, ce qui ne se voit pas ; un trou d'un pixel se voit partout.
    let (x0, y0) = ((x as f32).round(), (y as f32).round());
    let (x1, y1) = (
        ((x + place.cote_ecran) as f32).round(),
        ((y + place.cote_ecran) as f32).round(),
    );
    // **Les bords s'arrondissent en ordonnées d'ÉCRAN, puis se décalent.** Jamais l'inverse.
    //
    // `f32::round` s'éloigne de zéro : `100,5` monte à `101`, et `-249,5` descend à `-250`.
    // Retrancher le haut de la bande AVANT d'arrondir fait donc basculer tout demi-pixel qui
    // passe du côté négatif — et les deux bandes voisines posent alors la même tuile une
    // ligne plus haut de part et d'autre de leur frontière. Le test au bit près l'a mesuré :
    // 3 620 octets d'écart à l'échelle 1,5, aucun à l'échelle exacte, où rien n'est à demi.
    //
    // C'est la règle des « croix noires », transposée d'un axe à l'autre : un bord partagé
    // s'arrondit **une fois**, dans le repère où il est commun.
    let (y0, y1) = (y0 - decalage, y1 - decalage);
    let pose = Pose {
        x: x0,
        y: y0,
        largeur: x1 - x0,
        hauteur: y1 - y0,
    };
    let utile = boite_utile((bx0, by0, bx1, by1), (x0, y0, x1, y1), place.cote_ecran);
    let sien = Boite::nouvelle(
        place.clip.x,
        place.clip.y - decalage,
        place.clip.largeur,
        place.clip.hauteur,
    );
    let clip = intersection(sien, utile);
    let melange = if portee.opaque {
        Melange::Remplacer
    } else {
        Melange::Composer
    };
    reporter(dest, &src, pose, clip, melange, place.filtre)
}

/// La boîte utile d'une tuile, portée à l'écran — c'est elle qui borne le parcours.
///
/// `boite` est la portée mesurée au rangement, en texels de la tuile ; `pose` sont les quatre
/// bords de la tuile à l'écran, déjà arrondis.
///
/// # Pourquoi les bords extérieurs sont ceux de la pose, et pas un calcul
///
/// Un texel `0` ou `COTE_TUILE` désigne le bord de la tuile, et ce bord **est** celui de la
/// pose — le même nombre que la tuile voisine a arrondi. Le recalculer depuis le facteur
/// donnerait parfois un pixel de moins, et ce pixel serait un trou entre deux tuiles : les
/// « croix noires » que l'arrondi séparé avait déjà produites une fois.
fn boite_utile(boite: (u32, u32, u32, u32), pose: (f32, f32, f32, f32), cote_ecran: f64) -> Boite {
    let (bx0, by0, bx1, by1) = boite;
    let (x0, y0, x1, y1) = pose;
    let facteur = cote_ecran / f64::from(COTE_TUILE);
    let bord = |t: u32, origine: f32, limite: f32| {
        if t == COTE_TUILE {
            limite
        } else {
            origine + (f64::from(t) * facteur).round() as f32
        }
    };
    let ux0 = if bx0 == 0 { x0 } else { bord(bx0, x0, x1) };
    let uy0 = if by0 == 0 { y0 } else { bord(by0, y0, y1) };
    let ux1 = bord(bx1, x0, x1);
    let uy1 = bord(by1, y0, y1);
    Boite::nouvelle(ux0, uy0, (ux1 - ux0).max(0.0), (uy1 - uy0).max(0.0))
}

/// L'intersection de deux boîtes, vide si elles ne se touchent pas.
fn intersection(a: Boite, b: Boite) -> Boite {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.largeur).min(b.x + b.largeur);
    let y1 = (a.y + a.hauteur).min(b.y + b.hauteur);
    Boite::nouvelle(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::geometry::Rect;
    use glucose_core::report::Filtre;
    use glucose_core::tuile::Occupant;
    use glucose_core::types::Viewport;

    /// Une tuile dont chaque pixel depend de sa place : une tuile unie ne verrait pas un
    /// decalage d'une ligne entre deux bandes, qui est exactement le defaut a attraper.
    fn tuile(graine: u32) -> Pixmap {
        let mut p = Pixmap::new(COTE_TUILE, COTE_TUILE).expect("une tuile");
        let cote = COTE_TUILE;
        for (i, bloc) in p.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % cote, i as u32 / cote);
            let v = ((x * 7 + y * 13 + graine * 29) % 251) as u8;
            *bloc = [v, v.wrapping_add(80), v.wrapping_add(160), 255];
        }
        p
    }

    fn empreinte(graine: u64) -> Empreinte {
        Empreinte::de([Occupant {
            boite: Rect::new(0.0, 0.0, 16.0, 16.0),
            aspect: graine,
        }])
    }

    /// Un cache peuple, un releve qui pave l'ecran, et le modele de pose.
    fn scene(echelle: f64) -> (Tuiles, Couverture, Place) {
        let mut cache = Tuiles::nouveau();
        let mut posees = Vec::new();
        for i in 0..12u32 {
            let e = empreinte(u64::from(i) + 1);
            cache.ranger(e, tuile(i));
            let cote = Adresse::cote_monde(0);
            let (x, y) = (f64::from(i % 4) * cote, f64::from(i / 4) * cote);
            posees.push((Adresse::contenant(0, x, y), e));
        }
        let releve = Couverture::pour_test(posees, 0);
        let place = Place {
            adresse: Adresse::contenant(0, 0.0, 0.0),
            vp: Viewport {
                x: -30.0,
                y: -20.0,
                scale: echelle,
            },
            cote_ecran: f64::from(COTE_TUILE) * echelle,
            clip: Boite::nouvelle(0.0, 0.0, 900.0, 700.0),
            filtre: Filtre::Lisse,
        };
        (cache, releve, place)
    }

    fn composee(fils: usize, echelle: f64) -> (Vec<u8>, u64) {
        let (cache, releve, place) = scene(echelle);
        let mut ecran = Pixmap::new(900, 700).expect("l'ecran");
        let pixels = composer_en(fils, &mut ecran.as_mut(), &cache, &releve, &[], place);
        (ecran.data().to_vec(), pixels)
    }

    /// **Deux voies d'une meme operation produisent les memes pixels, AU BIT PRES.**
    ///
    /// C'est la garantie que la charte exige de toute adaptation : elle change *comment* on
    /// arrive au resultat, jamais le resultat. Ici les voies sont « un fil » et « n fils »,
    /// et sans ce test rien ne dirait qu'une bande ne decale pas son contenu d'une ligne --
    /// le defaut typique d'un decoupage, et celui qui se verrait le moins.
    ///
    /// L'echelle 1,5 est le cas qui compte : entre deux niveaux dyadiques, chaque tuile
    /// s'interpole, et les bords tombent sur des demi-pixels. A l'echelle exacte, la
    /// composition est un deplacement de memoire et ne prouverait presque rien.
    #[test]
    fn test_le_nombre_de_bandes_ne_change_aucun_pixel() {
        for echelle in [1.0, 1.5, 1.93] {
            let (temoin, attendus) = composee(1, echelle);
            // Des nombres premiers entre eux avec la hauteur : les bandes ne tombent alors
            // jamais sur les memes lignes d'un essai a l'autre.
            for fils in [2, 3, 5, 7, 16, 64] {
                let (vu, pixels) = composee(fils, echelle);
                assert_eq!(
                    pixels, attendus,
                    "a l'echelle {echelle}, {fils} bandes n'ecrivent pas le meme nombre de pixels"
                );
                let ecarts = temoin.iter().zip(&vu).filter(|(a, b)| a != b).count();
                assert_eq!(
                    ecarts, 0,
                    "a l'echelle {echelle}, {fils} bandes changent {ecarts} octets"
                );
            }
        }
    }
}
