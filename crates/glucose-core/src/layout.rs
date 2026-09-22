//! Moteur d'organisation et de mise en page automatique du canvas (100% Rust std).
//!
//! Résout définitivement R-11 en unifiant les conventions de coordonnées :
//! - `BoardImage` est ancré en son CENTRE (x - w/2, y - h/2, x + w/2, y + h/2).
//! - `Annotation` est ancrée en HAUT-GAUCHE (x, y, x + w, y + h).

use crate::types::{Annotation, Board, BoardImage};

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
/// Les deux dispositions du module partaient d'une origine inventée. `organize_board_grid`
/// commençait littéralement à `(0, 0)` : ranger un tableau posé à dix mille unités de là
/// renvoyait tout à l'origine du canevas. Et `calculate_image_layout` faisait pire, parce que
/// c'était moins visible :
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

/// Ordonne l'intégralité du board (images et annotations) selon une grille propre sans aucun chevauchement.
/// Respecte strictement la dualité des repères :
/// - Les images sont positionnées à `cur_x + w / 2.0` et `cur_y + h / 2.0`.
/// - Les annotations sont positionnées à `cur_x` et `cur_y`.
pub fn organize_board_grid(board: &mut Board, padding: f64) {
    let total_count = board.images.len() + board.annotations.len();
    if total_count == 0 {
        return;
    }

    let cols = (total_count as f64).sqrt().ceil() as usize;
    let cols = cols.max(1);

    // **Ou le tableau se tenait avant d'etre range.** La grille se construit depuis l'origine
    // et se translate ensuite : sans cela, ranger un tableau pose a dix mille unites de la
    // renvoie tout au point zero du canevas, ce qui est la forme la plus brutale de perdre son
    // travail -- on ne retrouve meme plus ce qu'on rangeait.
    let avant = coin_du_board(board);

    let mut cur_x = 0.0;
    let mut cur_y = 0.0;
    let mut row_max_h = 0.0f64;
    let mut idx = 0;

    // 1. Placement des images (ancrage CENTRE)
    for img in &mut board.images {
        img.x = cur_x + img.width / 2.0;
        img.y = cur_y + img.height / 2.0;
        row_max_h = row_max_h.max(img.height);
        cur_x += img.width + padding;
        idx += 1;

        if idx % cols == 0 {
            cur_x = 0.0;
            cur_y += row_max_h + padding;
            row_max_h = 0.0;
        }
    }

    // 2. Placement des annotations (ancrage HAUT-GAUCHE). La boîte vient du modèle ; une
    //    flèche, qui n'en a pas, occupe l'enveloppe de son vecteur.
    for ann in &mut board.annotations {
        let (w, h) = ann.size().unwrap_or_else(|| arrow_extent(ann));
        ann.move_to(cur_x, cur_y);
        row_max_h = row_max_h.max(h);
        cur_x += w + padding;
        idx += 1;

        if idx % cols == 0 {
            cur_x = 0.0;
            cur_y += row_max_h + padding;
            row_max_h = 0.0;
        }
    }

    reposer_le_board(board, avant);
}

/// Le coin haut-gauche de tout ce que ce tableau porte, images et annotations confondues.
///
/// Les deux repères cohabitent : une image est ancrée en son centre, une annotation en son
/// coin haut-gauche. Les mélanger sans le dire est exactement le défaut R-11 que ce module
/// existe pour avoir résolu.
fn coin_du_board(board: &Board) -> Option<(f64, f64)> {
    let images = board.images.iter().map(|i| (i.x, i.y, i.width, i.height));
    let annotations = board.annotations.iter().map(|a| {
        let (w, h) = a.size().unwrap_or_else(|| arrow_extent(a));
        // Ramené au centre, pour que les deux familles se comparent dans le même repère.
        (a.x() + w / 2.0, a.y() + h / 2.0, w, h)
    });
    coin_des_rects(images.chain(annotations))
}

/// Translate tout le tableau pour que son coin haut-gauche retrouve `avant`.
fn reposer_le_board(board: &mut Board, avant: Option<(f64, f64)>) {
    let Some((ax, ay)) = avant else {
        return;
    };
    let Some((rx, ry)) = coin_du_board(board) else {
        return;
    };
    let (dx, dy) = (ax - rx, ay - ry);
    for img in &mut board.images {
        img.x += dx;
        img.y += dy;
    }
    for ann in &mut board.annotations {
        let (x, y) = (ann.x(), ann.y());
        ann.move_to(x + dx, y + dy);
    }
}

/// La place qu'une flèche occupe dans une grille : l'enveloppe de son vecteur, avec un
/// plancher pour qu'une flèche très courte reste saisissable.
fn arrow_extent(ann: &Annotation) -> (f64, f64) {
    const MIN_ARROW_CELL: (f64, f64) = (80.0, 30.0);
    let Annotation::Arrow { x, y, x2, y2, .. } = ann else {
        return MIN_ARROW_CELL;
    };
    (
        (x2 - x).abs().max(MIN_ARROW_CELL.0),
        (y2 - y).abs().max(MIN_ARROW_CELL.1),
    )
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
    use crate::types::Annotation;

    #[test]
    fn test_organize_board_grid_no_overlap() {
        let mut board = Board::new("b1", "Test");
        for i in 0..4 {
            let mut img = BoardImage::new(format!("img-{}", i), 100.0, 100.0, 100.0, 100.0);
            img.width = 100.0;
            img.height = 100.0;
            board.images.push(img);
        }

        for i in 0..4 {
            board.annotations.push(Annotation::Text {
                id: format!("text-{}", i),
                x: 0.0,
                y: 0.0,
                width: Some(120.0),
                height: Some(40.0),
                text: "sample".into(),
                font_size: None,
                color: None,
                cursor_pos: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            });
        }

        organize_board_grid(&mut board, 20.0);

        // Extract bounding boxes
        let mut bboxes = Vec::new();
        for img in &board.images {
            let hw = img.width / 2.0;
            let hh = img.height / 2.0;
            bboxes.push((img.x - hw, img.y - hh, img.x + hw, img.y + hh));
        }
        for ann in &board.annotations {
            if let Annotation::Text {
                x,
                y,
                width,
                height,
                ..
            } = ann
            {
                let w = width.unwrap();
                let h = height.unwrap();
                bboxes.push((*x, *y, *x + w, *y + h));
            }
        }

        // Verify none overlap
        for i in 0..bboxes.len() {
            for j in (i + 1)..bboxes.len() {
                let b1 = &bboxes[i];
                let b2 = &bboxes[j];
                let overlap_x = b1.0 < b2.2 && b1.2 > b2.0;
                let overlap_y = b1.1 < b2.3 && b1.3 > b2.1;
                assert!(
                    !(overlap_x && overlap_y),
                    "Overlap detected between item {} and item {}",
                    i,
                    j
                );
            }
        }
    }

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

    /// **Ranger tout le tableau ne le renvoie pas au point zero.**
    ///
    /// `organize_board_grid` commencait litteralement a `(0, 0)` : ranger un tableau pose a
    /// douze mille unites de la renvoyait tout a l'origine du canevas, images ET annotations.
    /// On ne retrouvait meme plus ce qu'on venait de ranger.
    #[test]
    fn test_ordonner_le_canevas_ne_le_renvoie_pas_au_point_zero() {
        const CENTRE: f64 = 12_000.0;
        let mut board = Board::new("b1", "Test");
        board.images = images_autour(9, CENTRE);
        let avant = coin(&board.images);

        organize_board_grid(&mut board, 20.0);

        let apres = coin(&board.images);
        assert!(
            (apres.0 - avant.0).abs() < 0.001 && (apres.1 - avant.1).abs() < 0.001,
            "le tableau range part de {apres:?} au lieu de {avant:?}"
        );
        assert!(
            apres.0.abs() > 1_000.0,
            "il est retombe pres du point zero : {apres:?}"
        );
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
