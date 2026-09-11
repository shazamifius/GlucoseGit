//! PICK-1 — poignees de redimensionnement : leur geometrie et leur collecte.
//!
//! Une poignee prime sur tout le reste (`PICK_RANK_HANDLE` = 0) : c'est la cible la plus
//! petite a l'ecran, donc la plus couteuse a rater.

use super::*;

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

/// Collecte les poignees sous le curseur. Utilisee aussi par `candidates`.
pub(super) fn collect_handles(input: &PickInput, out: &mut Vec<PickCandidate>) {
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
