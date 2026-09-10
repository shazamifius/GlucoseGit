//! Index spatial accéléré pour le culling du viewport (O(cells_in_viewport)).
//! 100% Rust Standard Library (0 dépendance).

use std::collections::{HashMap, HashSet};

pub struct SpatialHash {
    grid: HashMap<(i32, i32), Vec<String>>,
    cell_size: f64,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size: if cell_size > 0.0 { cell_size } else { 2000.0 },
        }
    }

    pub fn build<'a, I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (&'a str, f64, f64, f64, f64)>, // id, x, y, width, height (anchor center)
    {
        self.grid.clear();
        for (id, x, y, width, height) in items {
            let min_ci = ((x - width / 2.0) / self.cell_size).floor() as i32;
            let max_ci = ((x + width / 2.0) / self.cell_size).floor() as i32;
            let min_cj = ((y - height / 2.0) / self.cell_size).floor() as i32;
            let max_cj = ((y + height / 2.0) / self.cell_size).floor() as i32;

            for ci in min_ci..=max_ci {
                for cj in min_cj..=max_cj {
                    self.grid.entry((ci, cj)).or_default().push(id.to_string());
                }
            }
        }
    }

    pub fn insert_bbox(&mut self, id: &str, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        let min_ci = (min_x / self.cell_size).floor() as i32;
        let max_ci = (max_x / self.cell_size).floor() as i32;
        let min_cj = (min_y / self.cell_size).floor() as i32;
        let max_cj = (max_y / self.cell_size).floor() as i32;

        for ci in min_ci..=max_ci {
            for cj in min_cj..=max_cj {
                self.grid.entry((ci, cj)).or_default().push(id.to_string());
            }
        }
    }

    pub fn insert_image(&mut self, img: &crate::types::BoardImage) {
        let hw = img.width / 2.0;
        let hh = img.height / 2.0;
        self.insert_bbox(&img.id, img.x - hw, img.y - hh, img.x + hw, img.y + hh);
    }

    pub fn insert_annotation(&mut self, ann: &crate::types::Annotation) {
        match ann {
            crate::types::Annotation::Text { id, x, y, width, height, .. } => {
                let w = width.unwrap_or(240.0);
                let h = height.unwrap_or(48.0);
                self.insert_bbox(id, *x, *y, *x + w, *y + h);
            }
            crate::types::Annotation::Sticky { id, x, y, width, height, .. } => {
                let w = width.unwrap_or(160.0);
                let h = height.unwrap_or(120.0);
                self.insert_bbox(id, *x, *y, *x + w, *y + h);
            }
            crate::types::Annotation::Membrane { id, x, y, width, height, .. } => {
                self.insert_bbox(id, *x, *y, *x + *width, *y + *height);
            }
            crate::types::Annotation::Arrow { id, x, y, x2, y2, .. } => {
                let min_x = x.min(*x2);
                let min_y = y.min(*y2);
                let max_x = x.max(*x2);
                let max_y = y.max(*y2);
                self.insert_bbox(id, min_x, min_y, max_x, max_y);
            }
        }
    }

    pub fn index_board(&mut self, board: &crate::types::Board) {
        self.clear();
        for img in &board.images {
            self.insert_image(img);
        }
        for ann in &board.annotations {
            self.insert_annotation(ann);
        }
    }

    pub fn query_rect_refs(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, margin: f64) -> HashSet<&str> {
        let mut out = HashSet::new();
        let min_ci = ((min_x - margin) / self.cell_size).floor() as i32;
        let max_ci = ((max_x + margin) / self.cell_size).floor() as i32;
        let min_cj = ((min_y - margin) / self.cell_size).floor() as i32;
        let max_cj = ((max_y + margin) / self.cell_size).floor() as i32;

        for ci in min_ci..=max_ci {
            for cj in min_cj..=max_cj {
                if let Some(ids) = self.grid.get(&(ci, cj)) {
                    for id in ids {
                        out.insert(id.as_str());
                    }
                }
            }
        }
        out
    }

    pub fn query_rect(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, margin: f64) -> HashSet<String> {
        self.query_rect_refs(min_x, min_y, max_x, max_y, margin)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn query_ids(&self, x: f64, y: f64, w: f64, h: f64, margin: f64) -> HashSet<String> {
        self.query_rect(x, y, x + w, y + h, margin)
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_query() {
        let mut sh = SpatialHash::new(1000.0);
        let items = [
            ("img1", 500.0, 500.0, 100.0, 100.0),
            ("img2", 2500.0, 2500.0, 100.0, 100.0),
        ];
        sh.build(items);

        let visible = sh.query_ids(0.0, 0.0, 1000.0, 1000.0, 0.0);
        assert!(visible.contains("img1"));
        assert!(!visible.contains("img2"));
    }

    #[test]
    fn test_spatial_hash_index_board() {
        use crate::types::{Board, BoardImage, Annotation};

        let mut board = Board::new("board-1", "Test Board");
        let mut img = BoardImage::new("img-1", 100.0, 100.0, 200.0, 200.0);
        img.x = 200.0;
        img.y = 200.0;
        board.images.push(img);

        let ann = Annotation::Text {
            id: "text-1".into(),
            x: 3000.0,
            y: 3000.0,
            width: Some(200.0),
            height: Some(50.0),
            text: "Hello".into(),
            font_size: None,
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        };
        board.annotations.push(ann);

        let mut sh = SpatialHash::new(1000.0);
        sh.index_board(&board);

        // Near (0,0) to (1000, 1000)
        let visible_near = sh.query_rect(0.0, 0.0, 500.0, 500.0, 50.0);
        assert!(visible_near.contains("img-1"));
        assert!(!visible_near.contains("text-1"));

        // Far (2500, 2500) to (3500, 3500)
        let visible_far = sh.query_rect(2800.0, 2800.0, 3200.0, 3200.0, 50.0);
        assert!(!visible_far.contains("img-1"));
        assert!(visible_far.contains("text-1"));
    }
}
