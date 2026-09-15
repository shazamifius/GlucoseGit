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
    let band = pick_consts::EDGE_BAND_PX / input.scale.max(1e-6);
    let mut out = Vec::new();

    collect_handles(input, &mut out);
    for (z, img) in input.images.iter().enumerate() {
        push_image(input, z, img, &mut out);
    }
    for (z, ann) in input.annotations.iter().enumerate() {
        push_annotation(input, band, z, ann, &mut out);
    }
    for (z, f) in input.folders.iter().enumerate() {
        push_folder(input, band, z, f, &mut out);
    }
    // Une flèche se désigne par sa **géométrie** (ARROW-1) et non par un nom reçu tout
    // fait. Tauri le tenait du DOM — une bande invisible que le navigateur savait toucher —
    // et le portage avait gardé le champ sans le navigateur : personne ne le remplissait,
    // donc aucune flèche n'était jamais sélectionnable.
    if let Some((fleche, dist)) = crate::arrow::at(
        input.annotations,
        // Une flèche ancrée se vise **là où elle se dessine**, sur le bord du nœud
        // qu'elle touche et non sur son centre. Le résolveur donne la boîte d'un nœud ;
        // l'arbitre n'a pas à savoir comment une ancre se calcule.
        |id| node_rect_of(input, id),
        (input.wx, input.wy),
        input.scale,
    ) {
        out.push(PickCandidate {
            owner: PickOwner::Arrow,
            id: fleche.id().to_string(),
            kind: PickKind::Arrow,
            rank: PICK_RANK_ARROW,
            z: 0,
            corner: None,
            // La distance au trait départage deux flèches qui se croisent, comme elle
            // départage deux poignées voisines.
            dist,
            area: 0.0,
            terminal: false,
        });
    }

    ensure_dom_hint(input, &mut out);
    sort_candidates(&mut out);
    out
}

/// Une image sous le curseur.
///
/// Une image **verrouillée reste désignable**. Ce que le verrou retire, c'est le geste : pas
/// de poignées ([`super::handles`]), pas de déplacement (`Store::move_selected`). La retirer
/// aussi de la pile des candidats la rendait inatteignable — donc impossible à déverrouiller
/// autrement que par `Ctrl+Z`, ce que la fiche 08 § 1.3 ne demande nulle part.
fn push_image(input: &PickInput, z: usize, img: &BoardImage, out: &mut Vec<PickCandidate>) {
    if !in_rotated_box(
        input.wx,
        input.wy,
        img.x,
        img.y,
        img.width,
        img.height,
        img.rotation,
    ) {
        return;
    }
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

/// Une annotation sous le curseur : le bord ou le corps d'une membrane, une carte, une note.
///
/// Une carte et un pense-bête sont **terminaux** : le cycle de profondeur s'arrête sur eux,
/// pour laisser le double-clic d'édition intact. Leur boîte est celle du modèle, taille de
/// naissance comprise. Une flèche ne passe pas par ici : sa géométrie la désigne (ARROW-1).
fn push_annotation(
    input: &PickInput,
    band: f64,
    z: usize,
    ann: &Annotation,
    out: &mut Vec<PickCandidate>,
) {
    let (wx, wy) = (input.wx, input.wy);
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
            let (w, h) = (*width, *height);
            let area = (w * h).abs();
            let on_label = text.is_some()
                && wx >= *x
                && wx <= *x + w
                && wy >= *y - pick_consts::MEMBRANE_LABEL_BAND
                && wy <= *y;

            let (kind, rank) = if on_label || on_rect_edge(wx, wy, *x, *y, w, h, band) {
                (PickKind::MembraneEdge, PICK_RANK_MEMBRANE_EDGE)
            } else if in_rect(wx, wy, *x, *y, w, h) {
                (PickKind::MembraneBody, PICK_RANK_MEMBRANE_BODY)
            } else {
                return;
            };
            out.push(PickCandidate {
                owner: PickOwner::Membrane,
                id: id.clone(),
                kind,
                rank,
                z,
                corner: None,
                dist: 0.0,
                area,
                terminal: false,
            });
        }
        Annotation::Text { id, .. } | Annotation::Sticky { id, .. } => {
            let (kind, rank) = if matches!(ann, Annotation::Text { .. }) {
                (PickKind::Text, PICK_RANK_TEXT)
            } else {
                (PickKind::Sticky, PICK_RANK_STICKY)
            };
            let inside = ann
                .rect()
                .is_some_and(|r| in_rect(wx, wy, r.left, r.top, r.width, r.height));
            if inside {
                out.push(PickCandidate {
                    owner: PickOwner::Annotation,
                    id: id.clone(),
                    kind,
                    rank,
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

/// Un dossier sous le curseur : son en-tête et son bord prennent le pas sur son corps.
fn push_folder(
    input: &PickInput,
    band: f64,
    z: usize,
    f: &CanvasFolder,
    out: &mut Vec<PickCandidate>,
) {
    let (wx, wy) = (input.wx, input.wy);
    let on_header =
        wx >= f.x && wx <= f.x + f.width && wy >= f.y && wy <= f.y + pick_consts::FOLDER_HEADER;

    let (kind, rank) = if on_header || on_rect_edge(wx, wy, f.x, f.y, f.width, f.height, band) {
        (PickKind::FolderEdge, PICK_RANK_FOLDER_EDGE)
    } else if in_rect(wx, wy, f.x, f.y, f.width, f.height) {
        (PickKind::FolderBody, PICK_RANK_FOLDER_BODY)
    } else {
        return;
    };
    out.push(PickCandidate {
        owner: PickOwner::Folder,
        id: f.id.clone(),
        kind,
        rank,
        z,
        corner: None,
        dist: 0.0,
        area: (f.width * f.height).abs(),
        terminal: false,
    });
}

/// Le tri total et stable de l'arbitre : rang, puis distance, puis aire, puis profondeur,
/// puis identité. Aucun ex æquo ne subsiste, donc deux appels rendent le même ordre.
fn sort_candidates(out: &mut [PickCandidate]) {
    out.sort_by(|a, b| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| {
                a.dist
                    .partial_cmp(&b.dist)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.area
                    .partial_cmp(&b.area)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| b.z.cmp(&a.z))
            .then_with(|| a.id.cmp(&b.id))
            .then_with(|| a.corner.cmp(&b.corner))
    });
}

/// Version accélérée par index spatial de collect_candidates (Roadmap 1.17).
/// Filtre les images et annotations aux seules entités proches du curseur en O(local)
/// au lieu de scanner l'intégralité du board en O(N).
pub fn collect_candidates_indexed(
    input: &PickInput,
    spatial_index: &crate::quadtree::SpatialHash,
) -> Vec<PickCandidate> {
    let slop = (pick_consts::HANDLE_SLOP_PX * 2.0) / input.scale.max(1e-6);
    // R-39 : la requête d'index n'alloue aucune `String`. Les nœuds retenus, eux, sont
    // **clonés** ci-dessous — quelques annotations par clic, en O(local), mais des clones
    // tout de même : `collect_candidates` veut des tranches contiguës. À faire disparaître
    // en le faisant travailler sur des références (fiche 07 § 3).
    let nearby_ids = spatial_index.query_rect_refs(input.wx, input.wy, input.wx, input.wy, slop);
    if nearby_ids.is_empty() && input.dom_hint.is_none() {
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
        dom_hint: input.dom_hint,
    };

    collect_candidates(&filtered_input)
}

/// La boîte d'un nœud désigné par son identifiant, dans ce qu'on donne à l'arbitre.
///
/// L'arbitre reçoit des tranches et non un tableau : c'est ce qui lui permet de ne voir que
/// le voisinage du curseur. Le résolveur travaille donc sur ces mêmes tranches.
fn node_rect_of(input: &PickInput<'_>, id: &str) -> Option<crate::geometry::Rect> {
    if let Some(img) = input.images.iter().find(|i| i.id == id) {
        return Some(img.rect());
    }
    if let Some(ann) = input.annotations.iter().find(|a| a.id() == id) {
        return ann.rect();
    }
    input
        .folders
        .iter()
        .find(|f| f.id == id)
        .map(crate::types::CanvasFolder::rect)
}
