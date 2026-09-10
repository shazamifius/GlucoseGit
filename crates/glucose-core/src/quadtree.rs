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

    pub fn query_ids(&self, x: f64, y: f64, w: f64, h: f64, margin: f64) -> HashSet<String> {
        let mut out = HashSet::new();
        let min_ci = ((x - margin) / self.cell_size).floor() as i32;
        let max_ci = ((x + w + margin) / self.cell_size).floor() as i32;
        let min_cj = ((y - margin) / self.cell_size).floor() as i32;
        let max_cj = ((y + h + margin) / self.cell_size).floor() as i32;

        for ci in min_ci..=max_ci {
            for cj in min_cj..=max_cj {
                if let Some(ids) = self.grid.get(&(ci, cj)) {
                    for id in ids {
                        out.insert(id.clone());
                    }
                }
            }
        }
        out
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
}
