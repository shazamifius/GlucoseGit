//! Cache des teintes symbiotiques.
//!
//! La teinte d'une carte dépend de son **voisinage**, donc la recalculer coûte une passe sur
//! les cartes proches : c'est le seul état du rendu qui mérite d'être conservé d'une frame à
//! l'autre (cf. `halo.rs`, HALO-2). L'invalidation se fait par rayon : une carte qui bouge
//! n'invalide que ce qu'elle peut réellement teinter.

use glucose_core::types::Annotation;
use std::collections::{HashMap, HashSet};

/// Rayon d'influence d'un déplacement, en unités monde. Au-delà, une carte qui bouge ne
/// change la teinte de personne : son entrée de cache reste valide.
pub const INVALIDATION_RADIUS: f64 = 1200.0;

#[derive(Debug, Clone)]
pub struct CachedHue {
    pub x: f64,
    pub y: f64,
    pub hue: f64,
    pub rgb: (u8, u8, u8),
}

pub struct SymbioticHueCache {
    entries: HashMap<String, CachedHue>,
    last_positions: HashMap<String, (f64, f64)>,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for SymbioticHueCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbioticHueCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_positions: HashMap::new(),
        }
    }

    /// Invalidation par voisinage : si une carte a bougé, seules les cartes à moins de
    /// [`INVALIDATION_RADIUS`] sont invalidées.
    pub fn update_positions_and_invalidate(&mut self, annotations: &[Annotation]) {
        let mut moved_points: Vec<(f64, f64)> = Vec::new();
        let mut current_ids: HashSet<&str> = HashSet::with_capacity(annotations.len());

        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                current_ids.insert(id.as_str());
                match self.last_positions.get(id.as_str()) {
                    Some(&(lx, ly)) => {
                        if (lx - x).abs() > 0.01 || (ly - y).abs() > 0.01 {
                            moved_points.push((*x, *y));
                            moved_points.push((lx, ly));
                        }
                    }
                    None => {
                        moved_points.push((*x, *y));
                    }
                }
            }
        }

        // Détecter les cartes supprimées et les retirer de last_positions sans réallocation globale
        self.last_positions.retain(|old_id, pos| {
            let (lx, ly) = *pos;
            if !current_ids.contains(old_id.as_str()) {
                moved_points.push((lx, ly));
                false
            } else {
                true
            }
        });

        if !moved_points.is_empty() {
            const RADIUS_SQ: f64 = INVALIDATION_RADIUS * INVALIDATION_RADIUS;
            self.entries.retain(|id, entry| {
                if !current_ids.contains(id.as_str()) {
                    return false;
                }
                for &(mx, my) in &moved_points {
                    let dx = entry.x - mx;
                    let dy = entry.y - my;
                    if dx * dx + dy * dy <= RADIUS_SQ {
                        return false;
                    }
                }
                true
            });
        }

        // Mettre à jour les coordonnées en place sans réallouer de chaînes pour les cartes existantes (R-39)
        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                if let Some(pos) = self.last_positions.get_mut(id.as_str()) {
                    *pos = (*x, *y);
                } else {
                    self.last_positions.insert(id.clone(), (*x, *y));
                }
            }
        }
    }

    pub fn get_or_compute(&mut self, ann: &Annotation, all_annotations: &[Annotation]) -> (f64, (u8, u8, u8)) {
        let id = ann.id();
        let ax = ann.x();
        let ay = ann.y();

        if let Some(entry) = self.entries.get(id) {
            return (entry.hue, entry.rgb);
        }

        let hue = glucose_core::symbiotic_hue::get_symbiotic_hue(ann, all_annotations);
        let rgb = glucose_core::symbiotic_hue::hsl_to_rgb(hue, 0.75, 0.65);
        self.entries.insert(id.to_string(), CachedHue { x: ax, y: ay, hue, rgb });
        (hue, rgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: &str, x: f64, y: f64) -> Annotation {
        crate::renderer::card::tests::probe_card(id, x, y)
    }

    #[test]
    fn test_symbiotic_hue_cache_invalidation_radius() {
        let mut cache = SymbioticHueCache::new();

        // 3 cartes : T1 à (0,0), T2 à (500,0) (voisine), T3 à (3000,0) (lointaine)
        let t1 = card("T1", 0.0, 0.0);
        let t2 = card("T2", 500.0, 0.0);
        let t3 = card("T3", 3000.0, 0.0);
        let list = vec![t1.clone(), t2.clone(), t3.clone()];

        cache.update_positions_and_invalidate(&list);
        let _ = cache.get_or_compute(&t1, &list);
        let _ = cache.get_or_compute(&t2, &list);
        let _ = cache.get_or_compute(&t3, &list);

        assert_eq!(cache.entries.len(), 3);

        // Déplacer T1 de 50 px : T1 et T2 (< 1 200 px) doivent être invalidées, T3 doit rester
        let t1_moved = card("T1", 50.0, 0.0);
        let list_moved = vec![t1_moved.clone(), t2.clone(), t3.clone()];

        cache.update_positions_and_invalidate(&list_moved);

        assert!(cache.entries.contains_key("T3"));
        assert!(!cache.entries.contains_key("T1"));
        assert!(!cache.entries.contains_key("T2"));
    }

    #[test]
    fn test_symbiotic_hue_cache_stationary_preserves_all_entries() {
        let mut cache = SymbioticHueCache::new();
        let t1 = card("T1", 100.0, 100.0);
        let t2 = card("T2", 200.0, 200.0);
        let list = vec![t1.clone(), t2.clone()];

        cache.update_positions_and_invalidate(&list);
        let (h1, _) = cache.get_or_compute(&t1, &list);
        let (h2, _) = cache.get_or_compute(&t2, &list);

        // Deuxième frame stationnaire : aucune modification de position
        cache.update_positions_and_invalidate(&list);
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.entries.get("T1").map(|e| e.hue), Some(h1));
        assert_eq!(cache.entries.get("T2").map(|e| e.hue), Some(h2));
    }
}
