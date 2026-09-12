//! MEMB-4 — Le mode étiré, à l'échelle d'un board (géométrie PURE, 0 dépendance).

use crate::geometry::Rect;
use crate::membrane_space::{content_extent, parent_map, SpaceItem, STRETCH_PADDING};
use crate::types::MembraneMode;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct StretchPlan {
    pub desired: Rect,
    pub allowed: Rect,
    pub blockers: Vec<SpaceItem>,
    pub blocked: bool,
}

pub fn stretch_plan(
    membrane: &SpaceItem,
    children: &[&SpaceItem],
    foreigners: &[&SpaceItem],
) -> StretchPlan {
    let m_rect = membrane.rect();
    let (ext_w, ext_h) = content_extent(m_rect, children);
    let pad = STRETCH_PADDING;
    let desired = Rect::new(
        membrane.x,
        membrane.y,
        membrane.width.max(ext_w + pad),
        membrane.height.max(ext_h + pad),
    );

    let mut max_w = desired.width;
    let mut max_h = desired.height;
    let mut blockers = Vec::new();

    for &f in foreigners {
        if f.id == membrane.id {
            continue;
        }
        let f_rect = f.rect();
        if !desired.overlaps(f_rect) {
            continue;
        }
        if m_rect.overlaps(f_rect) {
            continue;
        }

        blockers.push(f.clone());
        let cut_w = f.x - membrane.x;
        let cut_h = f.y - membrane.y;
        let lose_w = desired.width - cut_w;
        let lose_h = desired.height - cut_h;
        if lose_w <= lose_h {
            max_w = max_w.min(membrane.width.max(cut_w));
        } else {
            max_h = max_h.min(membrane.height.max(cut_h));
        }
    }

    let allowed = Rect::new(membrane.x, membrane.y, max_w, max_h);
    let blocked = allowed.width < desired.width || allowed.height < desired.height;

    StretchPlan {
        desired,
        allowed,
        blockers,
        blocked,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StretchOutcome {
    pub membrane_id: String,
    pub width: f64,
    pub height: f64,
    pub grew: bool,
    pub blocked: bool,
    pub blocker_ids: Vec<String>,
}

fn depth_of(id: &str, parents: &HashMap<String, String>) -> usize {
    let mut seen = HashSet::new();
    seen.insert(id);
    let mut d = 0;
    let mut cur = parents.get(id);
    while let Some(c) = cur {
        if seen.contains(c.as_str()) {
            break;
        }
        seen.insert(c.as_str());
        d += 1;
        cur = parents.get(c);
    }
    d
}

/// Croissance de TOUTES les membranes étirées d'un board (par profondeur décroissante).
pub fn plan_board_stretch(items: &[SpaceItem]) -> Vec<StretchOutcome> {
    let parents = parent_map(items);
    let mut work: Vec<SpaceItem> = items.to_vec();

    let mut target_indices: Vec<usize> = work
        .iter()
        .enumerate()
        .filter(|(_, i)| {
            i.kind == crate::membrane_space::SpaceItemKind::Membrane
                && i.mode == Some(MembraneMode::Stretched)
        })
        .map(|(idx, _)| idx)
        .collect();

    target_indices.sort_by(|&a, &b| {
        let da = depth_of(&work[a].id, &parents);
        let db = depth_of(&work[b].id, &parents);
        db.cmp(&da)
    });

    let mut out = Vec::new();
    for idx in target_indices {
        let m = work[idx].clone();
        let frame = parents.get(&m.id).cloned();

        let mut children = Vec::new();
        let mut foreigners = Vec::new();

        for it in &work {
            if it.id == m.id {
                continue;
            }
            let p = parents.get(&it.id).cloned();
            if p.as_deref() == Some(&m.id) {
                children.push(it);
            } else if p == frame {
                foreigners.push(it);
            }
        }

        let plan = stretch_plan(&m, &children, &foreigners);
        let width = plan.allowed.width;
        let height = plan.allowed.height;
        let grew = width != m.width || height != m.height;
        if !grew && !plan.blocked {
            continue;
        }

        work[idx].width = width;
        work[idx].height = height;

        out.push(StretchOutcome {
            membrane_id: m.id,
            width,
            height,
            grew,
            blocked: plan.blocked,
            blocker_ids: plan.blockers.into_iter().map(|b| b.id).collect(),
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::membrane_space::SpaceItemKind;

    #[test]
    fn test_stretch_plan_unblocked() {
        let m = SpaceItem {
            id: "M1".into(),
            kind: SpaceItemKind::Membrane,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            mode: Some(MembraneMode::Stretched),
            membrane_id: None,
        };
        let kid = SpaceItem {
            id: "K1".into(),
            kind: SpaceItemKind::Image,
            x: 20.0,
            y: 20.0,
            width: 150.0,
            height: 150.0,
            mode: None,
            membrane_id: Some("M1".into()),
        };

        let plan = stretch_plan(&m, &[&kid], &[]);
        assert!(!plan.blocked);
        assert!(plan.allowed.width >= 170.0 + STRETCH_PADDING);
        assert!(plan.allowed.height >= 170.0 + STRETCH_PADDING);
    }

    #[test]
    fn test_stretch_plan_blocked_by_foreigner() {
        let m = SpaceItem {
            id: "M".into(),
            kind: SpaceItemKind::Membrane,
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
            mode: Some(MembraneMode::Stretched),
            membrane_id: None,
        };
        let kid = SpaceItem {
            id: "I".into(),
            kind: SpaceItemKind::Image,
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 100.0,
            mode: None,
            membrane_id: Some("M".into()),
        };
        let obstacle = SpaceItem {
            id: "ETR".into(),
            kind: SpaceItemKind::Image,
            x: 250.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            mode: None,
            membrane_id: None,
        };

        let plan = stretch_plan(&m, &[&kid], &[&obstacle]);
        assert!(plan.blocked);
        assert_eq!(plan.blockers.len(), 1);
        assert_eq!(plan.blockers[0].id, "ETR");
        assert_eq!(plan.allowed.width, 250.0);
    }
}
