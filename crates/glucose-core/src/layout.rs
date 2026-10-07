//! Moteur d'organisation et de mise en page automatique du canvas (100% Rust std).
//!
//! Une seule convention de coordonnées (R-11) : une `BoardImage`, comme un `LayoutRect`, est
//! ancrée en son CENTRE (x - w/2, y - h/2, x + w/2, y + h/2).

use crate::types::BoardImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizeMode {
    #[default]
    Grid,
    Masonry,
    SameHeight,
    SameWidth,
    CompactRows,
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizeSort {
    #[default]
    None,
    SizeDesc,
    SizeAsc,
    RatioLandscape,
    RatioPortrait,
    NameAsc,
    DateRecent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRect {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// **Le coin haut-gauche d'un ensemble de rectangles**, ou `None` s'il est vide.
///
/// Les deux repères du module cohabitent ici, et c'est voulu : une image est ancrée en son
/// CENTRE, un rectangle de disposition aussi. La demi-largeur se retranche donc des deux
/// côtés, et ce serait faux pour une annotation.
fn coin_des_rects<'a>(
    rects: impl Iterator<Item = (f64, f64, f64, f64)> + 'a,
) -> Option<(f64, f64)> {
    rects.fold(None, |acc: Option<(f64, f64)>, (x, y, w, h)| {
        let (gauche, haut) = (x - w / 2.0, y - h / 2.0);
        Some(match acc {
            Some((gx, gy)) => (gx.min(gauche), gy.min(haut)),
            None => (gauche, haut),
        })
    })
}

/// **Repose une disposition là où elle était**, sans rien changer à sa forme.
///
/// # Le défaut que cette fonction supprime, et il rendait « Ordonner » inutilisable
///
/// La disposition partait d'une origine inventée, et `calculate_image_layout` le cachait
/// bien :
///
/// ```text
///     start_x = avg_x - (images.len() as f64 * target_size * 0.3)
/// ```
///
/// Le point de départ **reculait avec le nombre d'images**. Quatre cents images de deux cents
/// unités de large partaient donc vingt-quatre mille unités à gauche de leur centre — et le
/// `0.3` ne répondait à aucune géométrie : une grille a `cols` colonnes, jamais `len`.
///
/// La réponse ne demande aucune constante. La disposition se calcule depuis l'origine, puis
/// on la **translate** pour que son coin haut-gauche retrouve celui qu'occupait l'ensemble
/// avant : le bloc rangé commence là où le bloc épars commençait. C'est ce que fait tout outil
/// de mise en page, et cela vaut pour les cinq modes d'un seul coup, sans qu'aucun ait à
/// connaître sa propre ancre.
fn reposer_sur_place(avant: Option<(f64, f64)>, resultats: &mut [LayoutRect]) {
    let Some((ax, ay)) = avant else {
        return;
    };
    let Some((rx, ry)) = coin_des_rects(resultats.iter().map(|r| (r.x, r.y, r.width, r.height)))
    else {
        return;
    };
    let (dx, dy) = (ax - rx, ay - ry);
    for r in resultats.iter_mut() {
        r.x += dx;
        r.y += dy;
    }
}

/// **Le rapport largeur / hauteur d'une image**, tel que toute disposition le lit.
///
/// Il vient des dimensions d'ORIGINE, jamais de la boite courante : ranger doit redonner a
/// chaque image sa forme propre, pas perpetuer celle qu'un redimensionnement lui a donnee. Le
/// plancher evite qu'une image sans dimensions connues produise une hauteur infinie.
///
/// Cinq dispositions le calculaient chacune de leur cote, a l'identique -- la geometrie
/// calculee cinq fois que la fiche 05 interdit.
fn ratio(img: &BoardImage) -> f64 {
    (img.original_width / img.original_height.max(1.0)).max(0.1)
}

/// Les images dans l'ordre demande. Un tri que le noyau ne sait pas faire laisse l'ordre tel
/// quel, ce qui est exactement ce que `OrganizeSort::None` veut dire.
fn trier(images: &[BoardImage], sort: OrganizeSort) -> Vec<BoardImage> {
    let mut triees = images.to_vec();
    let cle: fn(&BoardImage) -> f64 = match sort {
        OrganizeSort::SizeDesc | OrganizeSort::SizeAsc => |i| i.width * i.height,
        OrganizeSort::RatioPortrait | OrganizeSort::RatioLandscape => ratio,
        _ => return triees,
    };
    let decroissant = matches!(sort, OrganizeSort::SizeDesc | OrganizeSort::RatioLandscape);
    triees.sort_by(|a, b| {
        let (x, y) = (cle(a), cle(b));
        if decroissant {
            y.total_cmp(&x)
        } else {
            x.total_cmp(&y)
        }
    });
    triees
}

/// Une grille reguliere de `cols` colonnes, chaque rangee aussi haute que sa plus haute image.
fn en_grille(triees: &[BoardImage], w: f64, gap: f64, cols: usize) -> Vec<LayoutRect> {
    let colonnes = if cols > 0 {
        cols
    } else {
        (triees.len() as f64).sqrt().round().max(1.0) as usize
    };
    let mut resultats = Vec::with_capacity(triees.len());
    let mut y = 0.0;
    for rangee in triees.chunks(colonnes) {
        let hauteur = rangee.iter().map(|i| w / ratio(i)).fold(0.0f64, f64::max);
        for (i, img) in rangee.iter().enumerate() {
            let h = w / ratio(img);
            resultats.push(LayoutRect {
                id: img.id.clone(),
                x: i as f64 * (w + gap) + w / 2.0,
                y: y + h / 2.0,
                width: w,
                height: h,
            });
        }
        y += hauteur + gap;
    }
    resultats
}

/// Une mosaique : chaque image va dans la colonne la moins remplie, comme un mur de briques.
fn en_mosaique(triees: &[BoardImage], w: f64, gap: f64, cols: usize) -> Vec<LayoutRect> {
    const COLONNES_PAR_DEFAUT: usize = 3;
    let colonnes = if cols > 0 { cols } else { COLONNES_PAR_DEFAUT };
    let mut bas = vec![0.0f64; colonnes];
    let mut resultats = Vec::with_capacity(triees.len());
    for img in triees {
        let (rang, _) =
            bas.iter().enumerate().fold(
                (0usize, f64::MAX),
                |(ri, ry), (i, &y)| {
                    if y < ry {
                        (i, y)
                    } else {
                        (ri, ry)
                    }
                },
            );
        let h = w / ratio(img);
        resultats.push(LayoutRect {
            id: img.id.clone(),
            x: rang as f64 * (w + gap) + w / 2.0,
            y: bas[rang] + h / 2.0,
            width: w,
            height: h,
        });
        bas[rang] += h + gap;
    }
    resultats
}

/// Une seule ligne, toutes les images a la meme hauteur.
fn a_meme_hauteur(triees: &[BoardImage], h: f64, gap: f64) -> Vec<LayoutRect> {
    let mut x = 0.0;
    triees
        .iter()
        .map(|img| {
            let w = h * ratio(img);
            let rect = LayoutRect {
                id: img.id.clone(),
                x: x + w / 2.0,
                y: h / 2.0,
                width: w,
                height: h,
            };
            x += w + gap;
            rect
        })
        .collect()
}

/// Une seule colonne, toutes les images a la meme largeur.
fn a_meme_largeur(triees: &[BoardImage], w: f64, gap: f64) -> Vec<LayoutRect> {
    let mut y = 0.0;
    triees
        .iter()
        .map(|img| {
            let h = w / ratio(img);
            let rect = LayoutRect {
                id: img.id.clone(),
                x: w / 2.0,
                y: y + h / 2.0,
                width: w,
                height: h,
            };
            y += h + gap;
            rect
        })
        .collect()
}

/// Des rangees de meme hauteur, coupees quand la ligne devient trop large.
///
/// La largeur de coupe vaut cinq fois la hauteur cible : c'est ce qui donne des rangees dont
/// le rapport approche celui d'un ecran, et c'est la seule raison pour laquelle ce nombre
/// existe. Il porte donc son nom.
fn en_rangees_compactes(triees: &[BoardImage], h: f64, gap: f64) -> Vec<LayoutRect> {
    const RANGEES_PAR_ECRAN: f64 = 5.0;
    let largeur_max = h * RANGEES_PAR_ECRAN;
    let mut resultats = Vec::with_capacity(triees.len());
    let mut rangee: Vec<(String, f64)> = Vec::new();
    let mut largeur = 0.0;
    let mut y = 0.0;
    let poser = |rangee: &mut Vec<(String, f64)>, y: f64, out: &mut Vec<LayoutRect>| {
        let mut x = 0.0;
        for (id, w) in rangee.drain(..) {
            out.push(LayoutRect {
                id,
                x: x + w / 2.0,
                y: y + h / 2.0,
                width: w,
                height: h,
            });
            x += w + gap;
        }
    };
    for img in triees {
        let w = h * ratio(img);
        if largeur + w > largeur_max && !rangee.is_empty() {
            poser(&mut rangee, y, &mut resultats);
            y += h + gap;
            largeur = 0.0;
        }
        rangee.push((img.id.clone(), w));
        largeur += w + gap;
    }
    poser(&mut rangee, y, &mut resultats);
    resultats
}

/// Calcule la réorganisation géométrique des images selon le mode sélectionné.
///
/// **Chaque disposition se construit depuis l'origine**, et une seule translation la repose ou
/// l'ensemble se tenait ([`reposer_sur_place`]). Aucune n'a donc a connaitre sa propre ancre,
/// et le defaut qui envoyait les images d'autant plus loin qu'il y en avait ne peut pas
/// revenir par l'une d'elles.
pub fn calculate_image_layout(
    images: &[BoardImage],
    mode: OrganizeMode,
    sort: OrganizeSort,
    target_size: f64,
    gap: f64,
    cols: usize,
) -> Vec<LayoutRect> {
    if images.is_empty() {
        return Vec::new();
    }
    let avant = coin_des_rects(images.iter().map(|i| (i.x, i.y, i.width, i.height)));
    let triees = trier(images, sort);
    let mut resultats = match mode {
        OrganizeMode::Grid => en_grille(&triees, target_size, gap, cols),
        OrganizeMode::Masonry => en_mosaique(&triees, target_size, gap, cols),
        OrganizeMode::SameHeight => a_meme_hauteur(&triees, target_size, gap),
        OrganizeMode::SameWidth => a_meme_largeur(&triees, target_size, gap),
        // Rangees compactes par defaut : c'est aussi ce que le panneau retient quand il
        // recoit un mode que le noyau ne sait pas poser.
        _ => en_rangees_compactes(&triees, target_size, gap),
    };
    reposer_sur_place(avant, &mut resultats);
    resultats
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Des images posees loin de l'origine, autour de `(centre, centre)`.
    fn images_autour(combien: usize, centre: f64) -> Vec<BoardImage> {
        (0..combien)
            .map(|i| {
                let mut img = BoardImage::new(
                    format!("img-{i}"),
                    centre + (i % 7) as f64 * 30.0,
                    centre + (i / 7) as f64 * 30.0,
                    200.0,
                    150.0,
                );
                img.width = 200.0;
                img.height = 150.0;
                img.original_width = 200.0;
                img.original_height = 150.0;
                img
            })
            .collect()
    }

    /// Le coin haut-gauche d'un ensemble d'images, toutes ancrees en leur centre.
    fn coin(images: &[BoardImage]) -> (f64, f64) {
        coin_des_rects(images.iter().map(|i| (i.x, i.y, i.width, i.height))).expect("des images")
    }

    /// **Ranger ne deplace pas le bloc**, et ce qu'il devient ne depend pas du NOMBRE d'images.
    ///
    /// Le point de depart valait `avg_x - len * target_size * 0.3` : il reculait avec le nombre
    /// d'images, donc quatre cents images de deux cents unites de large partaient vingt-quatre
    /// mille unites a gauche de leur centre. L'utilisateur l'a dit dans ses mots, le 22/09 :
    /// « les image parte super loin alors que on les a organiser a une telle coordoner ».
    ///
    /// Ce test porte sa preuve : il rejoue l'ancienne formule sur les memes images et verifie
    /// qu'elle les envoyait ailleurs -- et de plus en plus loin a mesure qu'il y en a.
    #[test]
    fn test_ranger_repose_le_bloc_ou_il_etait_quel_que_soit_le_nombre() {
        const CENTRE: f64 = 12_000.0;
        for combien in [4usize, 40, 400] {
            let images = images_autour(combien, CENTRE);
            let (ax, ay) = coin(&images);
            let range = calculate_image_layout(
                &images,
                OrganizeMode::Grid,
                OrganizeSort::None,
                200.0,
                20.0,
                0,
            );
            let (rx, ry) = coin_des_rects(range.iter().map(|r| (r.x, r.y, r.width, r.height)))
                .expect("un rangement");
            assert!(
                (rx - ax).abs() < 0.001 && (ry - ay).abs() < 0.001,
                "{combien} images : le bloc range part de ({rx}, {ry}) au lieu de ({ax}, {ay})"
            );

            // **L'ancienne formule, rejouee** : elle seule dit que ce test prouve quelque
            // chose. Sa derive vaut `len * taille * 0.3`, donc elle CROIT avec le nombre
            // d'images -- alors que la largeur du bloc range, elle, croit comme sa racine.
            // Passe quelques dizaines d'images, le bloc ne recouvre meme plus sa place.
            let avg_x = images.iter().map(|i| i.x).sum::<f64>() / images.len() as f64;
            let derive = images.len() as f64 * 200.0 * 0.3;
            let largeur = range
                .iter()
                .map(|r| r.x + r.width / 2.0)
                .fold(f64::MIN, f64::max)
                - range
                    .iter()
                    .map(|r| r.x - r.width / 2.0)
                    .fold(f64::MAX, f64::min);
            assert!(
                (avg_x - derive - ax).abs() > 0.0,
                "l'ancienne formule ne derivait pas, le test ne prouve rien"
            );
            if combien >= 40 {
                assert!(
                    derive > largeur,
                    "{combien} images : derive {derive:.0} pour un bloc large de                      {largeur:.0} -- le test ne prouve pas le defaut"
                );
            }
        }
    }

    /// **Ranger une poignee d'images ne touche qu'elles.**
    ///
    /// Le panneau recevait `&board.images` en entier. Ce test fixe la frontiere du cote du
    /// noyau : ce qu'on ne lui donne pas, il ne le rend pas -- donc l'appelant seul decide de
    /// ce qui est range, et `apply_dock_layout` lui donne desormais la selection.
    #[test]
    fn test_le_rangement_ne_rend_que_ce_qu_on_lui_donne() {
        let toutes = images_autour(10, 3_000.0);
        let trois: Vec<_> = toutes.iter().take(3).cloned().collect();
        let range = calculate_image_layout(
            &trois,
            OrganizeMode::Grid,
            OrganizeSort::None,
            200.0,
            20.0,
            0,
        );
        assert_eq!(range.len(), 3);
        for r in &range {
            assert!(
                trois.iter().any(|i| i.id == r.id),
                "{} n'etait pas dans ce qu'on a donne",
                r.id
            );
        }
    }
}
