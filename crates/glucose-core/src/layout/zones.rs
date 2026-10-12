//! **Ranger dans les zones** — la disposition « Par slot preset » d'Ordonner (PRESET-1,
//! fiche 62).
//!
//! Glucose Tauri groupait les images par une catégorie (`slotId`) qu'aucune ligne n'écrivait :
//! tout tombait dans une seule colonne. Ici, la catégorie d'une image est la zone où son centre
//! est posé, et chaque zone range **ses** images **en elle-même**, sous son bandeau. Une image
//! hors de toute zone ne bouge pas.
//!
//! # Des rangées justifiées
//!
//! Les rangées compactes d'Ordonner s'arrêtent à une largeur fixée d'avance — « cinq rangées
//! par écran ». Dans une zone, c'est la zone qui donne la largeur : chaque rangée se pose à la
//! hauteur visée, puis **se met à l'échelle pour remplir exactement** la largeur intérieure —
//! la mise en page des galeries de photos. La dernière, si elle est courte, garde la hauteur
//! visée. Une image trop large pour la zone y est réduite : rien ne déborde sur les côtés.
//!
//! Le tri choisi vaut dans chaque zone : c'est le même `trier` que les autres dispositions.
//!
//! # Tout tient dans la zone
//!
//! Une zone trop petite pour ses images les ferait déborder par le bas, sur la zone d'en
//! dessous — et, l'appartenance se lisant au centre, un second rangement ferait changer de zone
//! les images débordantes : le geste ne serait pas stable. Quand les rangées ne tiennent pas,
//! la hauteur visée descend donc, **images et écarts d'un même facteur** — la même mise en page
//! vue de plus loin —, jusqu'à la plus grande qui tient. La hauteur des rangées n'est pas une
//! fonction régulière de la hauteur visée (une image change de rangée par sauts) : on la cherche
//! par dichotomie sur la vraie mise en page, à un demi-point près. Toute hauteur assez petite
//! tient, puisque tout rétrécit avec elle : la recherche aboutit toujours.

use super::{ratio, trier, LayoutRect};
use crate::layout::OrganizeSort;
use crate::store::zones::{BANDEAU, MARGE};
use crate::types::{BoardImage, BoardZone};

/// **Les images rangées dans leurs zones** : pour chacune, sa place ; celles hors de toute zone
/// n'ont pas de place nouvelle. `hauteur` est la hauteur visée d'une rangée.
pub fn ranger_dans_les_zones(
    images: &[BoardImage],
    zones: &[BoardZone],
    sort: OrganizeSort,
    hauteur: f64,
    gap: f64,
) -> Vec<LayoutRect> {
    let images = trier(images, sort);
    let mut resultats = Vec::new();
    for (k, zone) in zones.iter().enumerate() {
        // Une image appartient à la plus haute des zones qui la contiennent : la dernière.
        let siennes: Vec<&BoardImage> = images
            .iter()
            .filter(|i| zone_de(zones, i) == Some(k))
            .collect();
        let gauche = zone.x + MARGE;
        let largeur = (zone.width - 2.0 * MARGE).max(1.0);
        let place = (zone.height - BANDEAU - 2.0 * MARGE).max(1.0);
        let (h, gap) = ce_qui_tient(&siennes, (largeur, place), hauteur, gap);
        let mut y = zone.y + BANDEAU + MARGE;
        for rangee in rangees(&siennes, largeur, h, gap) {
            let mut x = gauche;
            for (img, w, h) in &rangee.poses {
                resultats.push(LayoutRect {
                    id: img.id.clone(),
                    x: x + w / 2.0,
                    y: y + h / 2.0,
                    width: *w,
                    height: *h,
                });
                x += w + gap;
            }
            y += rangee.hauteur + gap;
        }
    }
    resultats
}

/// L'indice de la zone d'une image : la dernière qui contient son centre.
fn zone_de(zones: &[BoardZone], img: &BoardImage) -> Option<usize> {
    zones.iter().rposition(|z| {
        (z.x..=z.x + z.width).contains(&img.x) && (z.y..=z.y + z.height).contains(&img.y)
    })
}

/// **La plus grande hauteur visée qui tient** dans la place `(largeur, hauteur)`, et son écart :
/// la visée elle-même si elle tient, sinon une plus petite, l'écart réduit d'autant.
fn ce_qui_tient(images: &[&BoardImage], place: (f64, f64), h: f64, gap: f64) -> (f64, f64) {
    let ecart = |v: f64| gap * v / h.max(f64::EPSILON);
    let tient = |v: f64| {
        let r = rangees(images, place.0, v, ecart(v));
        let ecarts = ecart(v) * r.len().saturating_sub(1) as f64;
        r.iter().map(|r| r.hauteur).sum::<f64>() + ecarts <= place.1
    };
    if tient(h) {
        return (h, gap);
    }
    // D'abord une hauteur qui tient — jamais nulle, même sous le demi-point —, puis
    // l'intervalle se resserre : `bas` tient toujours, `haut` jamais.
    let mut bas = h / 2.0;
    while !tient(bas) {
        bas /= 2.0;
    }
    let mut haut = 2.0 * bas;
    while haut - bas > 0.5 {
        let milieu = (bas + haut) / 2.0;
        if tient(milieu) {
            bas = milieu;
        } else {
            haut = milieu;
        }
    }
    (bas, ecart(bas))
}

/// Une rangée posée : chaque image avec sa largeur et sa hauteur, et la hauteur de la rangée.
struct Rangee<'a> {
    poses: Vec<(&'a BoardImage, f64, f64)>,
    hauteur: f64,
}

/// Les rangées justifiées de ces images dans cette largeur.
fn rangees<'a>(images: &[&'a BoardImage], largeur: f64, h: f64, gap: f64) -> Vec<Rangee<'a>> {
    let mut sortie = Vec::new();
    let mut courante: Vec<&'a BoardImage> = Vec::new();
    let mut somme = 0.0;
    for &img in images {
        courante.push(img);
        somme += h * ratio(img);
        let ecarts = gap * (courante.len() - 1) as f64;
        if somme + ecarts >= largeur {
            sortie.push(justifier(&courante, largeur, gap, h));
            courante.clear();
            somme = 0.0;
        }
    }
    if !courante.is_empty() {
        // La dernière rangée, courte, garde la hauteur visée.
        let poses = courante.iter().map(|&i| (i, h * ratio(i), h)).collect();
        sortie.push(Rangee { poses, hauteur: h });
    }
    sortie
}

/// Une rangée mise à l'échelle pour remplir exactement la largeur : ses images gardent leurs
/// proportions et une même hauteur — la somme de leurs largeurs et des écarts est la largeur.
fn justifier<'a>(images: &[&'a BoardImage], largeur: f64, gap: f64, h: f64) -> Rangee<'a> {
    let ecarts = gap * (images.len() - 1) as f64;
    let somme: f64 = images.iter().map(|&i| h * ratio(i)).sum();
    let hauteur = h * (largeur - ecarts).max(1.0) / somme.max(f64::EPSILON);
    let poses = images
        .iter()
        .map(|&i| (i, hauteur * ratio(i), hauteur))
        .collect();
    Rangee { poses, hauteur }
}

#[cfg(test)]
mod tests;
