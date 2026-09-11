//! PICK-1 — collecte et classement des cibles sous le curseur.

use super::handles::collect_handles;
use super::*;

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
    // R-39 : variante empruntée — aucune `String` allouée par test de clic.
    let nearby_ids = spatial_index.query_rect_refs(input.wx, input.wy, input.wx, input.wy, slop);
    if nearby_ids.is_empty() && input.arrow_id.is_none() && input.dom_hint.is_none() {
        return Vec::new();
    }

    let filtered_images: Vec<BoardImage> = input
        .images
        .iter()
        .filter(|img| nearby_ids.contains(img.id.as_str()))
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
