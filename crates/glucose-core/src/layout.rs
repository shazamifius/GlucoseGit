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

    // 2. Placement des annotations (ancrage HAUT-GAUCHE)
    for ann in &mut board.annotations {
        match ann {
            Annotation::Text {
                x,
                y,
                width,
                height,
                ..
            } => {
                let w = width.unwrap_or(240.0);
                let h = height.unwrap_or(48.0);
                *x = cur_x;
                *y = cur_y;
                row_max_h = row_max_h.max(h);
                cur_x += w + padding;
                idx += 1;
            }
            Annotation::Sticky {
                x,
                y,
                width,
                height,
                ..
            } => {
                let w = width.unwrap_or(160.0);
                let h = height.unwrap_or(120.0);
                *x = cur_x;
                *y = cur_y;
                row_max_h = row_max_h.max(h);
                cur_x += w + padding;
                idx += 1;
            }
            Annotation::Membrane {
                x,
                y,
                width,
                height,
                ..
            } => {
                *x = cur_x;
                *y = cur_y;
                row_max_h = row_max_h.max(*height);
                cur_x += *width + padding;
                idx += 1;
            }
            Annotation::Arrow { x, y, x2, y2, .. } => {
                let dx = *x2 - *x;
                let dy = *y2 - *y;
                *x = cur_x;
                *y = cur_y;
                *x2 = cur_x + dx;
                *y2 = cur_y + dy;
                let h = dy.abs().max(30.0);
                row_max_h = row_max_h.max(h);
                cur_x += dx.abs().max(80.0) + padding;
                idx += 1;
            }
        }

        if idx % cols == 0 {
            cur_x = 0.0;
            cur_y += row_max_h + padding;
            row_max_h = 0.0;
        }
    }
}

/// Calcule la réorganisation géométrique des images selon le mode sélectionné.
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

    let mut sorted = images.to_vec();

    match sort {
        OrganizeSort::None => {}
        OrganizeSort::SizeDesc => {
            sorted.sort_by(|a, b| {
                (b.width * b.height)
                    .partial_cmp(&(a.width * a.height))
                    .unwrap()
            });
        }
        OrganizeSort::SizeAsc => {
            sorted.sort_by(|a, b| {
                (a.width * a.height)
                    .partial_cmp(&(b.width * b.height))
                    .unwrap()
            });
        }
        OrganizeSort::RatioPortrait => {
            sorted.sort_by(|a, b| {
                (a.width / a.height.max(1.0))
                    .partial_cmp(&(b.width / b.height.max(1.0)))
                    .unwrap()
            });
        }
        OrganizeSort::RatioLandscape => {
            sorted.sort_by(|a, b| {
                (b.width / b.height.max(1.0))
                    .partial_cmp(&(a.width / a.height.max(1.0)))
                    .unwrap()
            });
        }
        _ => {}
    }

    let avg_x = images.iter().map(|img| img.x).sum::<f64>() / images.len() as f64;
    let avg_y = images.iter().map(|img| img.y).sum::<f64>() / images.len() as f64;
    let start_x = avg_x - (images.len() as f64 * target_size * 0.3);
    let start_y = avg_y - (target_size * 0.5);

    match mode {
        OrganizeMode::Grid => {
            let actual_cols = if cols > 0 {
                cols
            } else {
                (images.len() as f64).sqrt().round().max(1.0) as usize
            };
            let w = target_size;
            let mut results = Vec::new();
            let mut cur_y = start_y;

            for chunk in sorted.chunks(actual_cols) {
                let max_h = chunk
                    .iter()
                    .map(|img| {
                        let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                        w / ratio
                    })
                    .fold(0.0f64, f64::max);

                for (i, img) in chunk.iter().enumerate() {
                    let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                    let h = w / ratio;
                    results.push(LayoutRect {
                        id: img.id.clone(),
                        x: start_x + i as f64 * (w + gap) + w / 2.0,
                        y: cur_y + h / 2.0,
                        width: w,
                        height: h,
                    });
                }
                cur_y += max_h + gap;
            }
            results
        }
        OrganizeMode::Masonry => {
            let actual_cols = if cols > 0 { cols } else { 3 };
            let w = target_size;
            let mut col_y = vec![start_y; actual_cols];
            let mut results = Vec::new();

            for img in sorted {
                let mut min_idx = 0;
                let mut min_y = col_y[0];
                for (ci, &cy) in col_y.iter().enumerate() {
                    if cy < min_y {
                        min_y = cy;
                        min_idx = ci;
                    }
                }

                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                let h = w / ratio;
                let x = start_x + min_idx as f64 * (w + gap);
                results.push(LayoutRect {
                    id: img.id,
                    x: x + w / 2.0,
                    y: col_y[min_idx] + h / 2.0,
                    width: w,
                    height: h,
                });
                col_y[min_idx] += h + gap;
            }
            results
        }
        OrganizeMode::SameHeight => {
            let h = target_size;
            let mut cur_x = start_x;
            let mut results = Vec::new();

            for img in sorted {
                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                let w = h * ratio;
                results.push(LayoutRect {
                    id: img.id,
                    x: cur_x + w / 2.0,
                    y: start_y + h / 2.0,
                    width: w,
                    height: h,
                });
                cur_x += w + gap;
            }
            results
        }
        OrganizeMode::SameWidth => {
            let w = target_size;
            let mut cur_y = start_y;
            let mut results = Vec::new();

            for img in sorted {
                let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                let h = w / ratio;
                results.push(LayoutRect {
                    id: img.id,
                    x: start_x + w / 2.0,
                    y: cur_y + h / 2.0,
                    width: w,
                    height: h,
                });
                cur_y += h + gap;
            }
            results
        }
        _ => {
            // CompactRows par défaut
            let target_h = target_size;
            let max_row_w = target_h * 5.0;
            let mut results = Vec::new();

            let scaled: Vec<_> = sorted
                .into_iter()
                .map(|img| {
                    let ratio = (img.original_width / img.original_height.max(1.0)).max(0.1);
                    let w = target_h * ratio;
                    (img.id, w, target_h)
                })
                .collect();

            let mut cur_row: Vec<(String, f64, f64)> = Vec::new();
            let mut cur_w = 0.0;
            let mut row_y = start_y;

            for (id, w, h) in scaled {
                if cur_w + w > max_row_w && !cur_row.is_empty() {
                    let mut rx = start_x;
                    for (item_id, item_w, item_h) in cur_row.drain(..) {
                        results.push(LayoutRect {
                            id: item_id,
                            x: rx + item_w / 2.0,
                            y: row_y + item_h / 2.0,
                            width: item_w,
                            height: item_h,
                        });
                        rx += item_w + gap;
                    }
                    row_y += target_h + gap;
                    cur_w = 0.0;
                }
                cur_row.push((id, w, h));
                cur_w += w + gap;
            }

            if !cur_row.is_empty() {
                let mut rx = start_x;
                for (item_id, item_w, item_h) in cur_row {
                    results.push(LayoutRect {
                        id: item_id,
                        x: rx + item_w / 2.0,
                        y: row_y + item_h / 2.0,
                        width: item_w,
                        height: item_h,
                    });
                    rx += item_w + gap;
                }
            }

            results
        }
    }
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
}
