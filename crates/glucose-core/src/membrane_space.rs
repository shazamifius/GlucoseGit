//! MEMB-1 — Repère local des membranes (géométrie PURE, 0 dépendance).

use crate::geometry::Rect;
use crate::types::{Annotation, Board, BoardImage, MembraneMode};
use std::collections::{HashMap, HashSet};

pub const MIN_CONTENT_SCALE: f64 = 0.08;
pub const STRETCH_PADDING: f64 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceItemKind {
    Image,
    Text,
    Sticky,
    Membrane,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceItem {
    pub id: String,
    pub kind: SpaceItemKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub mode: Option<MembraneMode>,
    pub membrane_id: Option<String>,
}

impl SpaceItem {
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

pub fn contains_center(outer: Rect, inner: Rect) -> bool {
    outer.contains_center(inner)
}

pub fn overlaps(a: Rect, b: Rect) -> bool {
    a.overlaps(b)
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

/// Table `élément -> membrane propriétaire` validée et sans cycle.
pub fn parent_map(items: &[SpaceItem]) -> HashMap<String, String> {
    let membranes: HashSet<&str> = items
        .iter()
        .filter(|i| i.kind == SpaceItemKind::Membrane)
        .map(|i| i.id.as_str())
        .collect();

    let mut raw = HashMap::new();
    for it in items {
        if let Some(ref p) = it.membrane_id {
            if p == &it.id || !membranes.contains(p.as_str()) {
                continue;
            }
            raw.insert(it.id.clone(), p.clone());
        }
    }

    let mut out = raw.clone();
    for id in raw.keys() {
        let mut seen = HashSet::new();
        seen.insert(id.as_str());
        let mut cur = raw.get(id);
        while let Some(c) = cur {
            if seen.contains(c.as_str()) {
                out.remove(id);
                break;
            }
            seen.insert(c.as_str());
            cur = raw.get(c);
        }
    }
    out
}

/// Éléments dont le centre tombe dans la boîte de `membrane` (coordonnées naturelles).
pub fn contained_in<'a>(items: &'a [SpaceItem], membrane: &SpaceItem) -> Vec<&'a SpaceItem> {
    let m_rect = membrane.rect();
    items
        .iter()
        .filter(|it| it.id != membrane.id && contains_center(m_rect, it.rect()))
        .collect()
}

/// Enfants directs de chaque membrane dans l'ordre des `items`.
pub fn children_by_membrane<'a>(
    items: &'a [SpaceItem],
    members: &HashMap<String, String>,
) -> HashMap<String, Vec<&'a SpaceItem>> {
    let mut out: HashMap<String, Vec<&'a SpaceItem>> = HashMap::new();
    for it in items {
        if let Some(parent) = members.get(&it.id) {
            out.entry(parent.clone()).or_default().push(it);
        }
    }
    out
}

/// Étendue nécessaire pour afficher le contenu à taille naturelle depuis l'origine de la membrane.
pub fn content_extent(membrane_rect: Rect, children: &[&SpaceItem]) -> (f64, f64) {
    let mut w: f64 = 0.0;
    let mut h: f64 = 0.0;
    for c in children {
        w = w.max(c.x + c.width - membrane_rect.left);
        h = h.max(c.y + c.height - membrane_rect.top);
    }
    (w.max(0.0), h.max(0.0))
}

/// Échelle du contenu d'une membrane :
/// k = min(1, min(largeur/étendueX, hauteur/étendueY))
pub fn content_scale(mode: MembraneMode, box_rect: Rect, extent_w: f64, extent_h: f64) -> f64 {
    if mode != MembraneMode::Minimized {
        return 1.0;
    }
    if extent_w <= 0.0 || extent_h <= 0.0 {
        return 1.0;
    }
    let fit = (box_rect.width / extent_w).min(box_rect.height / extent_h);
    clamp(fit, MIN_CONTENT_SCALE, 1.0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedItem {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
    pub membrane_id: Option<String>,
    pub content_scale: Option<f64>,
}

impl ResolvedItem {
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolveOptions<'a> {
    pub focused_membrane_id: Option<&'a str>,
}

pub fn resolve_items(items: &[SpaceItem], opts: ResolveOptions) -> HashMap<String, ResolvedItem> {
    let members = parent_map(items);
    let children = children_by_membrane(items, &members);
    let mut out = HashMap::new();
    let focus_id = opts.focused_membrane_id;

    #[allow(clippy::too_many_arguments)]
    fn walk(
        item: &SpaceItem,
        ox: f64,
        oy: f64,
        s: f64,
        in_focus: bool,
        members: &HashMap<String, String>,
        children: &HashMap<String, Vec<&SpaceItem>>,
        focus_id: Option<&str>,
        out: &mut HashMap<String, ResolvedItem>,
    ) {
        let mut resolved = ResolvedItem {
            id: item.id.clone(),
            x: ox,
            y: oy,
            width: item.width * s,
            height: item.height * s,
            scale: s,
            membrane_id: members.get(&item.id).cloned(),
            content_scale: None,
        };

        if item.kind == SpaceItemKind::Membrane {
            let kids = children.get(&item.id).map(|v| v.as_slice()).unwrap_or(&[]);
            let focus_here = in_focus || focus_id == Some(&item.id);
            let (ext_w, ext_h) = content_extent(item.rect(), kids);
            let k = if focus_here {
                1.0
            } else {
                content_scale(
                    item.mode.unwrap_or(MembraneMode::Classic),
                    item.rect(),
                    ext_w,
                    ext_h,
                )
            };
            resolved.content_scale = Some(k);

            out.insert(item.id.clone(), resolved);

            let child_scale = s * k;
            for kid in kids {
                walk(
                    kid,
                    ox + (kid.x - item.x) * child_scale,
                    oy + (kid.y - item.y) * child_scale,
                    child_scale,
                    focus_here,
                    members,
                    children,
                    focus_id,
                    out,
                );
            }
        } else {
            out.insert(item.id.clone(), resolved);
        }
    }

    for it in items {
        if members.contains_key(&it.id) {
            continue;
        }
        walk(
            it, it.x, it.y, 1.0, false, &members, &children, focus_id, &mut out,
        );
    }

    // Orphelins éventuels
    for it in items {
        if !out.contains_key(&it.id) {
            walk(
                it, it.x, it.y, 1.0, false, &members, &children, focus_id, &mut out,
            );
        }
    }

    out
}

/// CHEMIN RAPIDE : une membrane réduit-elle quoi que ce soit sur ce board ?
pub fn has_scaling(items: &[SpaceItem]) -> bool {
    items
        .iter()
        .any(|i| i.kind == SpaceItemKind::Membrane && i.mode == Some(MembraneMode::Minimized))
}

#[derive(Debug, Clone, PartialEq)]
pub struct MembershipChange {
    pub id: String,
    pub membrane_id: Option<String>,
}

/// Changements d'appartenance à écrire après un DÉPÔT.
pub fn reconcile_membership(
    items: &[SpaceItem],
    resolved: &HashMap<String, ResolvedItem>,
    moved_ids: &[String],
) -> Vec<MembershipChange> {
    if moved_ids.is_empty() {
        return Vec::new();
    }
    let parents = parent_map(items);
    let by_id: HashMap<&str, &SpaceItem> = items.iter().map(|i| (i.id.as_str(), i)).collect();
    let membranes: Vec<&SpaceItem> = items
        .iter()
        .filter(|i| i.kind == SpaceItemKind::Membrane)
        .collect();
    let mut out = Vec::new();

    let descends_from = |candidate: &str, root_id: &str| -> bool {
        let mut seen = HashSet::new();
        let mut cur = Some(candidate);
        while let Some(c) = cur {
            if seen.contains(c) {
                break;
            }
            if c == root_id {
                return true;
            }
            seen.insert(c);
            cur = parents.get(c).map(|s| s.as_str());
        }
        false
    };

    let unique_moved: HashSet<&str> = moved_ids.iter().map(|s| s.as_str()).collect();

    for &id in &unique_moved {
        let item = match by_id.get(id) {
            Some(i) => i,
            None => continue,
        };
        let r = match resolved.get(id) {
            Some(r) => r,
            None => continue,
        };

        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;

        let mut best: Option<&SpaceItem> = None;
        let mut best_area = f64::INFINITY;

        for &m in &membranes {
            if m.id == id || descends_from(&m.id, id) {
                continue;
            }
            let rm = match resolved.get(&m.id) {
                Some(rm) => rm,
                None => continue,
            };
            if cx < rm.x || cx > rm.x + rm.width || cy < rm.y || cy > rm.y + rm.height {
                continue;
            }
            let area = (rm.width * rm.height).abs();
            if area < best_area {
                best = Some(m);
                best_area = area;
            }
        }

        let next = best.map(|m| m.id.clone());
        let current = item.membrane_id.clone();
        if next != current {
            out.push(MembershipChange {
                id: id.to_string(),
                membrane_id: next,
            });
        }
    }

    out
}

pub fn can_switch_mode(from: MembraneMode, to: MembraneMode) -> bool {
    if from == to {
        return true;
    }
    if to == MembraneMode::Classic {
        return from == MembraneMode::Classic;
    }
    true
}

pub fn items_of_board(board: &Board) -> Vec<SpaceItem> {
    let mut out = Vec::new();
    for img in &board.images {
        out.push(SpaceItem {
            id: img.id.clone(),
            kind: SpaceItemKind::Image,
            x: img.x - img.width / 2.0,
            y: img.y - img.height / 2.0,
            width: img.width,
            height: img.height,
            mode: None,
            membrane_id: img.membrane_id.clone(),
        });
    }
    for ann in &board.annotations {
        match ann {
            Annotation::Arrow { .. } => {}
            Annotation::Membrane {
                id,
                x,
                y,
                width,
                height,
                mode,
                membrane_id,
                ..
            } => {
                out.push(SpaceItem {
                    id: id.clone(),
                    kind: SpaceItemKind::Membrane,
                    x: *x,
                    y: *y,
                    width: *width,
                    height: *height,
                    mode: Some(*mode),
                    membrane_id: membrane_id.clone(),
                });
            }
            Annotation::Text {
                id,
                x,
                y,
                width,
                height,
                membrane_id,
                ..
            } => {
                let w = width.unwrap_or(0.0);
                let h = height.unwrap_or(0.0);
                if w > 0.0 && h > 0.0 {
                    out.push(SpaceItem {
                        id: id.clone(),
                        kind: SpaceItemKind::Text,
                        x: *x,
                        y: *y,
                        width: w,
                        height: h,
                        mode: None,
                        membrane_id: membrane_id.clone(),
                    });
                }
            }
            Annotation::Sticky {
                id,
                x,
                y,
                width,
                height,
                membrane_id,
                ..
            } => {
                let w = width.unwrap_or(160.0);
                let h = height.unwrap_or(120.0);
                if w > 0.0 && h > 0.0 {
                    out.push(SpaceItem {
                        id: id.clone(),
                        kind: SpaceItemKind::Sticky,
                        x: *x,
                        y: *y,
                        width: w,
                        height: h,
                        mode: None,
                        membrane_id: membrane_id.clone(),
                    });
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginScale {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

pub fn scale_of(resolved: Option<&HashMap<String, ResolvedItem>>, id: &str) -> f64 {
    resolved
        .and_then(|r| r.get(id))
        .map(|r| r.scale)
        .unwrap_or(1.0)
}

pub fn origin_of(
    resolved: Option<&HashMap<String, ResolvedItem>>,
    id: &str,
) -> Option<OriginScale> {
    let r = resolved.and_then(|m| m.get(id))?;
    if (r.scale - 1.0).abs() < 1e-9 {
        return None;
    }
    Some(OriginScale {
        x: r.x,
        y: r.y,
        scale: r.scale,
    })
}

pub fn image_box(img: &BoardImage) -> Rect {
    Rect::new(
        img.x - img.width / 2.0,
        img.y - img.height / 2.0,
        img.width,
        img.height,
    )
}

pub fn image_center_of(r: &ResolvedItem) -> (f64, f64) {
    (r.x + r.width / 2.0, r.y + r.height / 2.0)
}

fn arrow_frame(
    source_id: Option<&str>,
    target_id: Option<&str>,
    resolved: &HashMap<String, ResolvedItem>,
) -> Option<String> {
    let s = source_id?;
    let t = target_id?;
    let a = resolved.get(s).and_then(|r| r.membrane_id.as_deref());
    let b = resolved.get(t).and_then(|r| r.membrane_id.as_deref());
    match (a, b) {
        (Some(ma), Some(mb)) if ma == mb => Some(ma.to_string()),
        _ => None,
    }
}

pub fn project_board(board: &Board, resolved: Option<&HashMap<String, ResolvedItem>>) -> Board {
    let res = match resolved {
        Some(r) => r,
        None => return board.clone(),
    };

    let mut new_images = Vec::with_capacity(board.images.len());
    for img in &board.images {
        if let Some(r) = res.get(&img.id) {
            if (r.scale - 1.0).abs() >= 1e-9 {
                let mut p_img = img.clone();
                let (cx, cy) = image_center_of(r);
                p_img.x = cx;
                p_img.y = cy;
                p_img.width = r.width;
                p_img.height = r.height;
                new_images.push(p_img);
                continue;
            }
        }
        new_images.push(img.clone());
    }

    let mut natural_membranes: HashMap<String, (f64, f64)> = HashMap::new();
    for a in &board.annotations {
        if let Annotation::Membrane { id, x, y, .. } = a {
            natural_membranes.insert(id.clone(), (*x, *y));
        }
    }

    let mut new_annotations = Vec::with_capacity(board.annotations.len());
    for ann in &board.annotations {
        match ann {
            Annotation::Arrow {
                id,
                x,
                y,
                x2,
                y2,
                text,
                font_size,
                color,
                arrow_type,
                arrow_bidirectional,
                predicate,
                stroke_width,
                waypoints,
                source_id,
                target_id,
                source_block_id,
                target_block_id,
                source_text_sel,
                target_text_sel,
                long_text,
                target_board_id,
                membrane_id,
                domains,
                mirror_of,
                temporal_anchor,
            } => {
                if let Some(frame_id) = arrow_frame(source_id.as_deref(), target_id.as_deref(), res)
                {
                    if let (Some(r_m), Some(&(nat_x, nat_y))) =
                        (res.get(&frame_id), natural_membranes.get(&frame_id))
                    {
                        let k = r_m.scale * r_m.content_scale.unwrap_or(1.0);
                        if (k - 1.0).abs() >= 1e-9 {
                            let px = |coord: f64| r_m.x + (coord - nat_x) * k;
                            let py = |coord: f64| r_m.y + (coord - nat_y) * k;
                            let new_waypoints = waypoints
                                .iter()
                                .map(|wp| crate::types::Point2D {
                                    x: px(wp.x),
                                    y: py(wp.y),
                                })
                                .collect();
                            new_annotations.push(Annotation::Arrow {
                                id: id.clone(),
                                x: px(*x),
                                y: py(*y),
                                x2: px(*x2),
                                y2: py(*y2),
                                text: text.clone(),
                                font_size: *font_size,
                                color: color.clone(),
                                arrow_type: arrow_type.clone(),
                                arrow_bidirectional: *arrow_bidirectional,
                                predicate: *predicate,
                                stroke_width: *stroke_width,
                                waypoints: new_waypoints,
                                source_id: source_id.clone(),
                                target_id: target_id.clone(),
                                source_block_id: source_block_id.clone(),
                                target_block_id: target_block_id.clone(),
                                source_text_sel: source_text_sel.clone(),
                                target_text_sel: target_text_sel.clone(),
                                long_text: long_text.clone(),
                                target_board_id: target_board_id.clone(),
                                membrane_id: membrane_id.clone(),
                                domains: domains.clone(),
                                mirror_of: mirror_of.clone(),
                                temporal_anchor: temporal_anchor.clone(),
                            });
                            continue;
                        }
                    }
                }
                new_annotations.push(ann.clone());
            }
            Annotation::Membrane {
                id,
                color,
                text,
                mode,
                curtains,
                membrane_id,
                domains,
                mirror_of,
                temporal_anchor,
                ..
            } => {
                if let Some(r) = res.get(id) {
                    if (r.scale - 1.0).abs() >= 1e-9 {
                        new_annotations.push(Annotation::Membrane {
                            id: id.clone(),
                            x: r.x,
                            y: r.y,
                            width: r.width,
                            height: r.height,
                            color: color.clone(),
                            text: text.clone(),
                            mode: *mode,
                            curtains: curtains.clone(),
                            membrane_id: membrane_id.clone(),
                            domains: domains.clone(),
                            mirror_of: mirror_of.clone(),
                            temporal_anchor: temporal_anchor.clone(),
                        });
                        continue;
                    }
                }
                new_annotations.push(ann.clone());
            }
            Annotation::Text {
                id,
                font_size,
                color,
                cursor_pos,
                source_file,
                membrane_id,
                domains,
                mirror_of,
                temporal_anchor,
                text,
                ..
            } => {
                if let Some(r) = res.get(id) {
                    if (r.scale - 1.0).abs() >= 1e-9 {
                        new_annotations.push(Annotation::Text {
                            id: id.clone(),
                            x: r.x,
                            y: r.y,
                            width: Some(r.width),
                            height: Some(r.height),
                            text: text.clone(),
                            font_size: *font_size,
                            color: color.clone(),
                            cursor_pos: *cursor_pos,
                            source_file: source_file.clone(),
                            membrane_id: membrane_id.clone(),
                            domains: domains.clone(),
                            mirror_of: mirror_of.clone(),
                            temporal_anchor: temporal_anchor.clone(),
                        });
                        continue;
                    }
                }
                new_annotations.push(ann.clone());
            }
            Annotation::Sticky {
                id,
                font_size,
                color,
                bg_color,
                cursor_pos,
                operator,
                source_file,
                membrane_id,
                domains,
                mirror_of,
                temporal_anchor,
                text,
                ..
            } => {
                if let Some(r) = res.get(id) {
                    if (r.scale - 1.0).abs() >= 1e-9 {
                        new_annotations.push(Annotation::Sticky {
                            id: id.clone(),
                            x: r.x,
                            y: r.y,
                            width: Some(r.width),
                            height: Some(r.height),
                            text: text.clone(),
                            font_size: *font_size,
                            color: color.clone(),
                            bg_color: bg_color.clone(),
                            cursor_pos: *cursor_pos,
                            operator: *operator,
                            source_file: source_file.clone(),
                            membrane_id: membrane_id.clone(),
                            domains: domains.clone(),
                            mirror_of: mirror_of.clone(),
                            temporal_anchor: temporal_anchor.clone(),
                        });
                        continue;
                    }
                }
                new_annotations.push(ann.clone());
            }
        }
    }

    Board {
        images: new_images,
        annotations: new_annotations,
        ..board.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_scale_deduction() {
        let m_box = Rect::new(0.0, 0.0, 200.0, 100.0);
        // Contenu s'étend jusqu'à 400x200
        let k = content_scale(MembraneMode::Minimized, m_box, 400.0, 200.0);
        // min(200/400, 100/200) = 0.5
        assert!((k - 0.5).abs() < 1e-6);

        // Si le contenu est plus petit que la membrane, plafonné à 1.0
        let k_cap = content_scale(MembraneMode::Minimized, m_box, 100.0, 50.0);
        assert_eq!(k_cap, 1.0);

        // Classic donne toujours 1.0
        let k_classic = content_scale(MembraneMode::Classic, m_box, 1000.0, 1000.0);
        assert_eq!(k_classic, 1.0);
    }

    #[test]
    fn test_parent_map_cycle_elimination() {
        let items = vec![
            SpaceItem {
                id: "A".into(),
                kind: SpaceItemKind::Membrane,
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                mode: None,
                membrane_id: Some("B".into()),
            },
            SpaceItem {
                id: "B".into(),
                kind: SpaceItemKind::Membrane,
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                mode: None,
                membrane_id: Some("A".into()), // cycle A <-> B
            },
            SpaceItem {
                id: "C".into(),
                kind: SpaceItemKind::Image,
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                mode: None,
                membrane_id: Some("A".into()),
            },
        ];

        let pmap = parent_map(&items);
        // Le cycle A <-> B est brisé
        assert!(!pmap.contains_key("A") || !pmap.contains_key("B"));
    }
}
