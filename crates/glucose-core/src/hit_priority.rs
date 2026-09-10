//! PICK-1 — Arbitre de sélection au clic et priorité des cibles.
//! 100% Rust Standard Library (0 dépendance).

use crate::geometry::{in_rect, in_rotated_box, on_rect_edge};
use crate::types::{Annotation, BoardImage, CanvasFolder};

pub const PICK_RANK_HANDLE: i32 = 0;
pub const PICK_RANK_MEMBRANE_EDGE: i32 = 10;
pub const PICK_RANK_FOLDER_EDGE: i32 = 10;
pub const PICK_RANK_ARROW: i32 = 20;
pub const PICK_RANK_IMAGE: i32 = 30;
pub const PICK_RANK_STICKY: i32 = 40;
pub const PICK_RANK_TEXT: i32 = 50;
pub const PICK_RANK_MEMBRANE_BODY: i32 = 60;
pub const PICK_RANK_FOLDER_BODY: i32 = 60;

pub mod pick_consts {
    pub const HANDLE_SLOP_PX: f64 = 24.0;
    pub const HANDLE_SLOP_MAX_RATIO: f64 = 0.35;
    pub const HANDLE_SLOP_MIN_PX: f64 = 6.0;
    pub const EDGE_BAND_PX: f64 = 14.0;
    pub const FOLDER_HEADER: f64 = 38.0;
    pub const FOLDER_HANDLE_INSET: f64 = 8.0;
    pub const MEMBRANE_LABEL_BAND: f64 = 30.0;
    pub const CYCLE_RADIUS_PX: f64 = 8.0;
    pub const CYCLE_TTL_MS: i64 = 2500;
    pub const DBLCLICK_MS: i64 = 400;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickOwner {
    Image,
    Annotation,
    Membrane,
    Folder,
    Arrow,
}

impl PickOwner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Annotation => "annotation",
            Self::Membrane => "membrane",
            Self::Folder => "folder",
            Self::Arrow => "arrow",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickKind {
    Handle,
    MembraneEdge,
    MembraneBody,
    FolderEdge,
    FolderBody,
    Arrow,
    Image,
    Sticky,
    Text,
}

impl PickKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Handle => "handle",
            Self::MembraneEdge => "membrane-edge",
            Self::MembraneBody => "membrane-body",
            Self::FolderEdge => "folder-edge",
            Self::FolderBody => "folder-body",
            Self::Arrow => "arrow",
            Self::Image => "image",
            Self::Sticky => "sticky",
            Self::Text => "text",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickCandidate {
    pub owner: PickOwner,
    pub id: String,
    pub kind: PickKind,
    pub rank: i32,
    pub z: usize,
    pub corner: Option<String>,
    pub dist: f64,
    pub area: f64,
    pub terminal: bool,
}

pub fn handle_slop_world(scale: f64, box_w: f64, box_h: f64) -> f64 {
    let s = scale.max(1e-6);
    let wanted = pick_consts::HANDLE_SLOP_PX / s;
    let cap = (pick_consts::HANDLE_SLOP_MIN_PX / s)
        .max(box_w.abs().min(box_h.abs()) * pick_consts::HANDLE_SLOP_MAX_RATIO);
    wanted.min(cap)
}

pub fn handle_cursor(corner: &str) -> &'static str {
    if corner == "tl" || corner == "br" {
        "nwse-resize"
    } else {
        "nesw-resize"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomHint<'a> {
    pub owner: PickOwner,
    pub id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct PickInput<'a> {
    pub wx: f64,
    pub wy: f64,
    pub scale: f64,
    pub images: &'a [BoardImage],
    pub annotations: &'a [Annotation],
    pub folders: &'a [CanvasFolder],
    pub selected_image_ids: &'a [String],
    pub selected_annotation_ids: &'a [String],
    pub selected_folder_id: Option<&'a str>,
    pub arrow_id: Option<&'a str>,
    pub dom_hint: Option<DomHint<'a>>,
}

fn corners_of_rect(x: f64, y: f64, w: f64, h: f64) -> [(&'static str, f64, f64); 4] {
    [
        ("tl", x, y),
        ("tr", x + w, y),
        ("bl", x, y + h),
        ("br", x + w, y + h),
    ]
}

#[allow(clippy::too_many_arguments)]
fn push_handles(
    out: &mut Vec<PickCandidate>,
    owner: PickOwner,
    id: &str,
    z: usize,
    corners: &[(&'static str, f64, f64)],
    wx: f64,
    wy: f64,
    slop: f64,
) {
    for &(corner, cx, cy) in corners {
        let d = f64::hypot(wx - cx, wy - cy);
        if d <= slop {
            out.push(PickCandidate {
                owner,
                id: id.to_string(),
                kind: PickKind::Handle,
                rank: PICK_RANK_HANDLE,
                z,
                corner: Some(corner.to_string()),
                dist: d,
                area: 0.0,
                terminal: false,
            });
        }
    }
}

fn collect_handles(input: &PickInput, out: &mut Vec<PickCandidate>) {
    let wx = input.wx;
    let wy = input.wy;
    let scale = input.scale;

    if !input.selected_image_ids.is_empty() {
        for (z, img) in input.images.iter().enumerate() {
            if img.locked || !input.selected_image_ids.contains(&img.id) {
                continue;
            }
            let hw = img.width / 2.0;
            let hh = img.height / 2.0;
            let rot = img.rotation;
            let c = rot.cos();
            let s = rot.sin();
            let offsets = [
                ("tl", -hw, -hh),
                ("tr", hw, -hh),
                ("bl", -hw, hh),
                ("br", hw, hh),
            ];
            let corners: Vec<(&'static str, f64, f64)> = offsets
                .iter()
                .map(|&(k, ox, oy)| (k, img.x + ox * c - oy * s, img.y + ox * s + oy * c))
                .collect();
            push_handles(
                out,
                PickOwner::Image,
                &img.id,
                z,
                &corners,
                wx,
                wy,
                handle_slop_world(scale, img.width, img.height),
            );
        }
    }

    if !input.selected_annotation_ids.is_empty() {
        for (z, ann) in input.annotations.iter().enumerate() {
            if !input.selected_annotation_ids.contains(&ann.id().to_string()) {
                continue;
            }
            match ann {
                Annotation::Arrow { .. } => continue,
                Annotation::Membrane {
                    id,
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let corners = corners_of_rect(*x, *y, *width, *height);
                    push_handles(
                        out,
                        PickOwner::Membrane,
                        id,
                        z,
                        &corners,
                        wx,
                        wy,
                        handle_slop_world(scale, *width, *height),
                    );
                }
                Annotation::Text {
                    id,
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let w = width.unwrap_or(0.0);
                    let h = height.unwrap_or(0.0);
                    if w > 0.0 && h > 0.0 {
                        let corners = corners_of_rect(*x, *y, w, h);
                        push_handles(
                            out,
                            PickOwner::Annotation,
                            id,
                            z,
                            &corners,
                            wx,
                            wy,
                            handle_slop_world(scale, w, h),
                        );
                    }
                }
                Annotation::Sticky {
                    id,
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let w = width.unwrap_or(160.0);
                    let h = height.unwrap_or(120.0);
                    if w > 0.0 && h > 0.0 {
                        let corners = corners_of_rect(*x, *y, w, h);
                        push_handles(
                            out,
                            PickOwner::Annotation,
                            id,
                            z,
                            &corners,
                            wx,
                            wy,
                            handle_slop_world(scale, w, h),
                        );
                    }
                }
            }
        }
    }

    if let Some(sel_folder_id) = input.selected_folder_id {
        for (z, f) in input.folders.iter().enumerate() {
            if f.id != sel_folder_id {
                continue;
            }
            let k = pick_consts::FOLDER_HANDLE_INSET;
            let br = [("br", f.x + f.width - k, f.y + f.height - k)];
            push_handles(
                out,
                PickOwner::Folder,
                &f.id,
                z,
                &br,
                wx,
                wy,
                handle_slop_world(scale, f.width, f.height),
            );
        }
    }
}

pub fn hit_handle(input: &PickInput) -> Option<PickCandidate> {
    let mut out = Vec::new();
    collect_handles(input, &mut out);
    if out.is_empty() {
        return None;
    }
    out.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(std::cmp::Ordering::Equal));
    out.into_iter().next()
}

fn ensure_dom_hint(input: &PickInput, out: &mut Vec<PickCandidate>) {
    let hint = match &input.dom_hint {
        Some(h) => h,
        None => return,
    };

    if out
        .iter()
        .any(|c| c.kind != PickKind::Handle && c.owner == hint.owner && c.id == hint.id)
    {
        return;
    }

    match hint.owner {
        PickOwner::Image => {
            if let Some((z, img)) = input
                .images
                .iter()
                .enumerate()
                .find(|(_, i)| i.id == hint.id)
            {
                if !img.locked {
                    out.push(PickCandidate {
                        owner: PickOwner::Image,
                        id: img.id.clone(),
                        kind: PickKind::Image,
                        rank: PICK_RANK_IMAGE,
                        z,
                        corner: None,
                        dist: 0.0,
                        area: 0.0,
                        terminal: false,
                    });
                }
            }
        }
        PickOwner::Membrane | PickOwner::Annotation => {
            if let Some((z, ann)) = input
                .annotations
                .iter()
                .enumerate()
                .find(|(_, a)| a.id() == hint.id)
            {
                match ann {
                    Annotation::Arrow { .. } => {}
                    Annotation::Membrane {
                        id, width, height, ..
                    } => {
                        out.push(PickCandidate {
                            owner: PickOwner::Membrane,
                            id: id.clone(),
                            kind: PickKind::MembraneBody,
                            rank: PICK_RANK_MEMBRANE_BODY,
                            z,
                            corner: None,
                            dist: 0.0,
                            area: (width * height).abs(),
                            terminal: false,
                        });
                    }
                    Annotation::Sticky { id, .. } => {
                        out.push(PickCandidate {
                            owner: PickOwner::Annotation,
                            id: id.clone(),
                            kind: PickKind::Sticky,
                            rank: PICK_RANK_STICKY,
                            z,
                            corner: None,
                            dist: 0.0,
                            area: 0.0,
                            terminal: true,
                        });
                    }
                    Annotation::Text { id, .. } => {
                        out.push(PickCandidate {
                            owner: PickOwner::Annotation,
                            id: id.clone(),
                            kind: PickKind::Text,
                            rank: PICK_RANK_TEXT,
                            z,
                            corner: None,
                            dist: 0.0,
                            area: 0.0,
                            terminal: true,
                        });
                    }
                }
            }
        }
        PickOwner::Folder => {
            if let Some((z, f)) = input
                .folders
                .iter()
                .enumerate()
                .find(|(_, f)| f.id == hint.id)
            {
                out.push(PickCandidate {
                    owner: PickOwner::Folder,
                    id: f.id.clone(),
                    kind: PickKind::FolderBody,
                    rank: PICK_RANK_FOLDER_BODY,
                    z,
                    corner: None,
                    dist: 0.0,
                    area: (f.width * f.height).abs(),
                    terminal: false,
                });
            }
        }
        PickOwner::Arrow => {}
    }
}

pub fn collect_candidates(input: &PickInput) -> Vec<PickCandidate> {
    let wx = input.wx;
    let wy = input.wy;
    let scale = input.scale;
    let band = pick_consts::EDGE_BAND_PX / scale.max(1e-6);
    let mut out = Vec::new();

    collect_handles(input, &mut out);

    // Images
    for (z, img) in input.images.iter().enumerate() {
        if img.locked {
            continue;
        }
        if in_rotated_box(wx, wy, img.x, img.y, img.width, img.height, img.rotation) {
            out.push(PickCandidate {
                owner: PickOwner::Image,
                id: img.id.clone(),
                kind: PickKind::Image,
                rank: PICK_RANK_IMAGE,
                z,
                corner: None,
                dist: 0.0,
                area: 0.0,
                terminal: false,
            });
        }
    }

    // Annotations
    for (z, ann) in input.annotations.iter().enumerate() {
        match ann {
            Annotation::Arrow { .. } => {}
            Annotation::Membrane {
                id,
                x,
                y,
                width,
                height,
                text,
                ..
            } => {
                let w = *width;
                let h = *height;
                let area = (w * h).abs();
                let on_label = text.is_some()
                    && wx >= *x
                    && wx <= *x + w
                    && wy >= *y - pick_consts::MEMBRANE_LABEL_BAND
                    && wy <= *y;

                if on_label || on_rect_edge(wx, wy, *x, *y, w, h, band) {
                    out.push(PickCandidate {
                        owner: PickOwner::Membrane,
                        id: id.clone(),
                        kind: PickKind::MembraneEdge,
                        rank: PICK_RANK_MEMBRANE_EDGE,
                        z,
                        corner: None,
                        dist: 0.0,
                        area,
                        terminal: false,
                    });
                } else if in_rect(wx, wy, *x, *y, w, h) {
                    out.push(PickCandidate {
                        owner: PickOwner::Membrane,
                        id: id.clone(),
                        kind: PickKind::MembraneBody,
                        rank: PICK_RANK_MEMBRANE_BODY,
                        z,
                        corner: None,
                        dist: 0.0,
                        area,
                        terminal: false,
                    });
                }
            }
            Annotation::Text {
                id,
                x,
                y,
                width,
                height,
                ..
            } => {
                let w = width.unwrap_or(0.0);
                let h = height.unwrap_or(0.0);
                if w > 0.0 && h > 0.0 && in_rect(wx, wy, *x, *y, w, h) {
                    out.push(PickCandidate {
                        owner: PickOwner::Annotation,
                        id: id.clone(),
                        kind: PickKind::Text,
                        rank: PICK_RANK_TEXT,
                        z,
                        corner: None,
                        dist: 0.0,
                        area: 0.0,
                        terminal: true,
                    });
                }
            }
            Annotation::Sticky {
                id,
                x,
                y,
                width,
                height,
                ..
            } => {
                let w = width.unwrap_or(160.0);
                let h = height.unwrap_or(120.0);
                if w > 0.0 && h > 0.0 && in_rect(wx, wy, *x, *y, w, h) {
                    out.push(PickCandidate {
                        owner: PickOwner::Annotation,
                        id: id.clone(),
                        kind: PickKind::Sticky,
                        rank: PICK_RANK_STICKY,
                        z,
                        corner: None,
                        dist: 0.0,
                        area: 0.0,
                        terminal: true,
                    });
                }
            }
        }
    }

    // Folders
    for (z, f) in input.folders.iter().enumerate() {
        let area = (f.width * f.height).abs();
        let on_header = wx >= f.x
            && wx <= f.x + f.width
            && wy >= f.y
            && wy <= f.y + pick_consts::FOLDER_HEADER;
        if on_header || on_rect_edge(wx, wy, f.x, f.y, f.width, f.height, band) {
            out.push(PickCandidate {
                owner: PickOwner::Folder,
                id: f.id.clone(),
                kind: PickKind::FolderEdge,
                rank: PICK_RANK_FOLDER_EDGE,
                z,
                corner: None,
                dist: 0.0,
                area,
                terminal: false,
            });
        } else if in_rect(wx, wy, f.x, f.y, f.width, f.height) {
            out.push(PickCandidate {
                owner: PickOwner::Folder,
                id: f.id.clone(),
                kind: PickKind::FolderBody,
                rank: PICK_RANK_FOLDER_BODY,
                z,
                corner: None,
                dist: 0.0,
                area,
                terminal: false,
            });
        }
    }

    // Arrow
    if let Some(arrow_id) = input.arrow_id {
        out.push(PickCandidate {
            owner: PickOwner::Arrow,
            id: arrow_id.to_string(),
            kind: PickKind::Arrow,
            rank: PICK_RANK_ARROW,
            z: 0,
            corner: None,
            dist: 0.0,
            area: 0.0,
            terminal: false,
        });
    }

    ensure_dom_hint(input, &mut out);

    // Tri total et stable
    out.sort_by(|a, b| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| a.dist.partial_cmp(&b.dist).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.area.partial_cmp(&b.area).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| b.z.cmp(&a.z))
            .then_with(|| a.id.cmp(&b.id))
            .then_with(|| a.corner.cmp(&b.corner))
    });

    out
}

/// Version accélérée par index spatial de collect_candidates (Roadmap 1.17).
/// Filtre les images et annotations aux seules entités proches du curseur en O(local)
/// au lieu de scanner l'intégralité du board en O(N).
pub fn collect_candidates_indexed(
    input: &PickInput,
    spatial_index: &crate::quadtree::SpatialHash,
) -> Vec<PickCandidate> {
    let slop = (pick_consts::HANDLE_SLOP_PX * 2.0) / input.scale.max(1e-6);
    let nearby_ids = spatial_index.query_rect(input.wx, input.wy, input.wx, input.wy, slop);
    if nearby_ids.is_empty() && input.arrow_id.is_none() && input.dom_hint.is_none() {
        return Vec::new();
    }

    let filtered_images: Vec<BoardImage> = input
        .images
        .iter()
        .filter(|img| nearby_ids.contains(&img.id))
        .cloned()
        .collect();

    let filtered_annotations: Vec<Annotation> = input
        .annotations
        .iter()
        .filter(|ann| nearby_ids.contains(ann.id()))
        .cloned()
        .collect();

    let filtered_input = PickInput {
        wx: input.wx,
        wy: input.wy,
        scale: input.scale,
        images: &filtered_images,
        annotations: &filtered_annotations,
        folders: input.folders,
        selected_image_ids: input.selected_image_ids,
        selected_annotation_ids: input.selected_annotation_ids,
        selected_folder_id: input.selected_folder_id,
        arrow_id: input.arrow_id,
        dom_hint: input.dom_hint,
    };

    collect_candidates(&filtered_input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct CycleState {
    pub sx: f64,
    pub sy: f64,
    pub sig: String,
    pub index: usize,
    pub t: i64,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PickOptions {
    pub alt: bool,
    pub multi: bool,
}

fn signature_of(cands: &[PickCandidate]) -> String {
    let mut parts = Vec::with_capacity(cands.len());
    for c in cands {
        parts.push(format!("{}:{}:{}", c.owner.as_str(), c.id, c.kind.as_str()));
    }
    parts.join("|")
}

fn cyclable_of(candidates: &[PickCandidate]) -> Vec<PickCandidate> {
    candidates
        .iter()
        .filter(|c| c.rank != PICK_RANK_HANDLE)
        .cloned()
        .collect()
}

pub fn pick_at_down(
    candidates: &[PickCandidate],
    prev: Option<&CycleState>,
    sx: f64,
    sy: f64,
    now: i64,
    opts: PickOptions,
) -> (Option<PickCandidate>, Option<CycleState>) {
    if candidates.is_empty() {
        return (None, None);
    }

    let handles: Vec<&PickCandidate> = candidates
        .iter()
        .filter(|c| c.rank == PICK_RANK_HANDLE)
        .collect();
    let cyclable = cyclable_of(candidates);
    let sig = signature_of(&cyclable);

    let dt = match prev {
        Some(p) => now - p.t,
        None => i64::MAX,
    };

    let same_spot = if let Some(p) = prev {
        p.sig == sig
            && dt <= pick_consts::CYCLE_TTL_MS
            && f64::hypot(sx - p.sx, sy - p.sy) <= pick_consts::CYCLE_RADIUS_PX
            && !opts.multi
    } else {
        false
    };

    let repeat = same_spot && dt >= pick_consts::DBLCLICK_MS && !opts.alt;

    let mut index = if same_spot {
        let p = prev.unwrap();
        p.index.min(cyclable.len().saturating_sub(1))
    } else {
        0
    };

    if opts.alt && same_spot && cyclable.len() > 1 {
        index = (index + 1) % cyclable.len();
    }

    let cycle = CycleState {
        sx,
        sy,
        sig,
        index,
        t: now,
        repeat,
    };

    if !handles.is_empty() && !opts.alt {
        return (Some((*handles[0]).clone()), Some(cycle));
    }

    if cyclable.is_empty() {
        let picked = handles.first().map(|&h| h.clone());
        return (picked, Some(cycle));
    }

    let picked = cyclable.get(index).or_else(|| cyclable.first()).cloned();
    (picked, Some(cycle))
}

pub fn advance_on_release(
    candidates: &[PickCandidate],
    cycle: Option<&CycleState>,
    now: i64,
) -> (Option<PickCandidate>, Option<CycleState>) {
    let c = match cycle {
        Some(c) => c,
        None => return (None, None),
    };

    let settled = CycleState {
        t: now,
        repeat: false,
        ..c.clone()
    };

    if !c.repeat {
        return (None, Some(settled));
    }

    let cyclable = cyclable_of(candidates);
    if signature_of(&cyclable) != c.sig || cyclable.len() < 2 {
        return (None, Some(settled));
    }

    if let Some(target) = cyclable.get(c.index) {
        if target.terminal {
            return (None, Some(settled));
        }
    }

    let index = (c.index + 1) % cyclable.len();
    let picked = cyclable.get(index).cloned();
    let next_cycle = CycleState {
        index,
        ..settled
    };
    (picked, Some(next_cycle))
}



#[cfg(test)]
mod tests {
    use super::*;

    fn img(id: &str, x: f64, y: f64, w: f64, h: f64, locked: bool) -> BoardImage {
        let mut img = BoardImage::new(id, x, y, w, h);
        img.locked = locked;
        img
    }

    fn membrane(id: &str, x: f64, y: f64, w: f64, h: f64, text: Option<&str>) -> Annotation {
        Annotation::Membrane {
            id: id.to_string(),
            x,
            y,
            width: w,
            height: h,
            color: None,
            text: text.map(String::from),
            mode: crate::types::MembraneMode::Classic,
            curtains: Vec::new(),
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    fn text_ann(id: &str, x: f64, y: f64, w: f64, h: f64) -> Annotation {
        Annotation::Text {
            id: id.to_string(),
            x,
            y,
            width: Some(w),
            height: Some(h),
            text: "hello".to_string(),
            font_size: None,
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    #[test]
    fn test_pick_priority_image_in_membrane() {
        let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
        let images = [img("I1", 500.0, 400.0, 200.0, 150.0, false)];
        let annotations = [memb];
        let folders = [];
        let empty: [String; 0] = [];

        let input = PickInput {
            wx: 500.0,
            wy: 400.0,
            scale: 1.0,
            images: &images,
            annotations: &annotations,
            folders: &folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };

        let cands = collect_candidates(&input);
        assert_eq!(cands[0].id, "I1");
        assert_eq!(cands[0].kind, PickKind::Image);
        assert_eq!(cands[1].id, "M1");
        assert_eq!(cands[1].kind, PickKind::MembraneBody);
    }

    #[test]
    fn test_pick_membrane_edge() {
        let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
        let images = [img("I1", 5.0, 400.0, 200.0, 150.0, false)];
        let annotations = [memb];
        let folders = [];
        let empty: [String; 0] = [];

        let input = PickInput {
            wx: 5.0,
            wy: 400.0,
            scale: 1.0,
            images: &images,
            annotations: &annotations,
            folders: &folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };

        let cands = collect_candidates(&input);
        assert_eq!(cands[0].id, "M1");
        assert_eq!(cands[0].kind, PickKind::MembraneEdge);
    }

    #[test]
    fn test_nested_membranes_smallest_wins() {
        let inner = membrane("PETITE", 400.0, 300.0, 200.0, 200.0, None);
        let outer = membrane("GRANDE", 0.0, 0.0, 1000.0, 800.0, None);
        let annotations = [outer, inner];
        let empty_images = [];
        let empty_folders = [];
        let empty: [String; 0] = [];

        let input = PickInput {
            wx: 500.0,
            wy: 400.0,
            scale: 1.0,
            images: &empty_images,
            annotations: &annotations,
            folders: &empty_folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };

        let cands = collect_candidates(&input);
        assert_eq!(cands[0].id, "PETITE");
    }

    #[test]
    fn test_text_terminal_cycle() {
        let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
        let t = text_ann("T1", 400.0, 380.0, 200.0, 60.0);
        let annotations = [memb, t];
        let empty_images = [];
        let empty_folders = [];
        let empty: [String; 0] = [];

        let input = PickInput {
            wx: 500.0,
            wy: 400.0,
            scale: 1.0,
            images: &empty_images,
            annotations: &annotations,
            folders: &empty_folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };

        let cands = collect_candidates(&input);
        assert_eq!(cands[0].id, "T1");
        assert_eq!(cands[0].terminal, true);
    }

    #[test]
    fn test_collect_candidates_indexed_matches_naive() {
        use crate::quadtree::SpatialHash;

        let mut hash = SpatialHash::new(500.0);
        let mut images = Vec::new();
        let mut annotations = Vec::new();

        for i in 0..20 {
            let id = format!("I{}", i);
            let img_obj = img(&id, i as f64 * 1000.0, 0.0, 200.0, 200.0, false);
            hash.insert_image(&img_obj);
            images.push(img_obj);
        }

        let t = text_ann("T1", 5000.0, 0.0, 100.0, 40.0);
        hash.insert_annotation(&t);
        annotations.push(t);

        let empty_folders = [];
        let empty: [String; 0] = [];

        let input = PickInput {
            wx: 5020.0,
            wy: 10.0,
            scale: 1.0,
            images: &images,
            annotations: &annotations,
            folders: &empty_folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };

        let naive = collect_candidates(&input);
        let indexed = collect_candidates_indexed(&input, &hash);

        assert_eq!(naive.len(), indexed.len());
        assert_eq!(naive[0].id, indexed[0].id);
        assert_eq!(naive[0].id, "I5");

        // Clic loin dans le vide
        let empty_input = PickInput {
            wx: 99999.0,
            wy: 99999.0,
            scale: 1.0,
            images: &images,
            annotations: &annotations,
            folders: &empty_folders,
            selected_image_ids: &empty,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };
        let empty_indexed = collect_candidates_indexed(&empty_input, &hash);
        assert!(empty_indexed.is_empty());
    }
}
