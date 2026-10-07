//! PICK-1 — poignees de redimensionnement : leur geometrie et leur collecte.
//!
//! Une poignee prime sur tout le reste (`PICK_RANK_HANDLE` = 0) : c'est la cible la plus
//! petite a l'ecran, donc la plus couteuse a rater.
//!
//! Chaque noeud selectionne expose les huit poignees de [`Handle::ALL`] — quatre coins,
//! quatre milieux de cote. La carte de texte n'en offrait que six : sa hauteur suivait
//! son texte, donc ses bords haut et bas ne se tiraient pas. Ils se tirent desormais, et
//! le texte en est le plancher. Le nom range dans `PickCandidate::corner` est celui de
//! [`Handle::as_str`].

use super::*;
use crate::resize::Handle;
use crate::smart_align::AlignRect;

/// Pousse dans `out` les poignees de `handles` posees sur `rect` qui sont a portee du curseur.
fn push_rect_handles(
    out: &mut Vec<PickCandidate>,
    owner: PickOwner,
    id: &str,
    z: usize,
    rect: AlignRect,
    handles: &[Handle],
    input: &PickInput,
) {
    let Some(slop) = handle_slop_world(input.scale, rect.width, rect.height) else {
        return;
    };
    let positions = handles.iter().map(|h| (*h, h.position_on(rect)));
    push_handles(out, owner, id, z, positions, (input.wx, input.wy), slop);
}

fn push_handles(
    out: &mut Vec<PickCandidate>,
    owner: PickOwner,
    id: &str,
    z: usize,
    positions: impl Iterator<Item = (Handle, (f64, f64))>,
    (wx, wy): (f64, f64),
    slop: f64,
) {
    for (handle, (cx, cy)) in positions {
        let d = f64::hypot(wx - cx, wy - cy);
        if d <= slop {
            out.push(PickCandidate {
                owner,
                id: id.to_string(),
                kind: PickKind::Handle,
                rank: PICK_RANK_HANDLE,
                z,
                corner: Some(handle.as_str().to_string()),
                dist: d,
                area: 0.0,
            });
        }
    }
}

/// Les poignees d'une image tournent avec elle : posees sur sa boite locale, centree,
/// puis ramenees dans le monde par sa rotation.
fn push_image_handles(out: &mut Vec<PickCandidate>, img: &BoardImage, z: usize, input: &PickInput) {
    let Some(slop) = handle_slop_world(input.scale, img.width, img.height) else {
        return;
    };
    let local = AlignRect::new(-img.width / 2.0, -img.height / 2.0, img.width, img.height);
    let positions = Handle::ALL.iter().map(|h| {
        let offset = h.position_on(local);
        (
            *h,
            crate::rotate::place((img.x, img.y), offset, img.rotation),
        )
    });
    push_handles(
        out,
        PickOwner::Image,
        &img.id,
        z,
        positions,
        (input.wx, input.wy),
        slop,
    );
}

/// Boîte et poignées d'une annotation sélectionnée ; `None` pour une flèche, qui n'a pas de
/// boîte. Une carte de texte n'a que des poignées horizontales : sa hauteur suit son texte
/// (TEXT-FIT-1).
fn annotation_handles(ann: &Annotation) -> Option<(PickOwner, AlignRect, &'static [Handle])> {
    let rect = ann.rect()?;
    let (owner, handles) = match ann {
        Annotation::Membrane { .. } => (PickOwner::Membrane, &Handle::ALL[..]),
        Annotation::Text { .. } => (PickOwner::Annotation, &Handle::ALL[..]),
        Annotation::Sticky { .. } => (PickOwner::Annotation, &Handle::ALL[..]),
        Annotation::Arrow { .. } => return None,
    };
    Some((owner, rect, handles))
}

/// Les poignées d'un groupe : ses quatre coins. Un groupe garde toujours son rapport — l'étirer
/// déformerait chaque nœud différemment selon sa place —, donc un côté n'aurait rien à faire
/// qu'un coin ne fasse.
pub const POIGNEES_DU_GROUPE: [Handle; 4] = [
    Handle::TopLeft,
    Handle::TopRight,
    Handle::BottomRight,
    Handle::BottomLeft,
];

/// **Le cadre du groupe** : l'emprise de ce que la sélection transforme, quand elle compte
/// **deux nœuds ou plus** — sinon rien, et chaque nœud garde ses poignées (fiche 53 § 10).
///
/// Une image verrouillée n'en fait pas partie, comme elle ne porte pas de poignée. Le dessin
/// lit la même boîte que le clic : un cadre dessiné est un cadre qu'on attrape (loi L4).
pub fn emprise_du_groupe(input: &PickInput) -> Option<AlignRect> {
    let images = input
        .images
        .iter()
        .filter(|i| !i.locked && input.selected_image_ids.contains(&i.id))
        .map(BoardImage::bounds);
    let annotations = input
        .annotations
        .iter()
        .filter(|a| input.selected_annotation_ids.iter().any(|s| s == a.id()))
        .map(Annotation::bounds);
    let dossier = input
        .folders
        .iter()
        .filter(|f| input.selected_folder_id == Some(f.id.as_str()))
        .map(|f| f.rect());
    let boites: Vec<AlignRect> = images.chain(annotations).chain(dossier).collect();
    if boites.len() < 2 {
        return None;
    }
    crate::smart_align::union_rect(&boites)
}

/// **Les poignées du groupe sous le curseur**, si la sélection forme un groupe — `None` sinon.
///
/// Le cadre se calcule sur **toute** la sélection : la collecte par l'index ne voit que les
/// nœuds proches du curseur, et un coin du groupe peut n'en toucher aucun.
pub(super) fn poignees_du_groupe(input: &PickInput) -> Option<Vec<PickCandidate>> {
    let groupe = emprise_du_groupe(input)?;
    let mut out = Vec::new();
    push_rect_handles(
        &mut out,
        PickOwner::Groupe,
        "groupe",
        usize::MAX,
        groupe,
        &POIGNEES_DU_GROUPE,
        input,
    );
    Some(out)
}

/// Collecte les poignees sous le curseur. Utilisee aussi par `candidates`.
///
/// Une sélection de deux nœuds ou plus n'a que les poignées de son groupe.
pub(super) fn collect_handles(input: &PickInput, out: &mut Vec<PickCandidate>) {
    if let Some(du_groupe) = poignees_du_groupe(input) {
        out.extend(du_groupe);
        return;
    }
    if !input.selected_image_ids.is_empty() {
        for (z, img) in input.images.iter().enumerate() {
            if img.locked || !input.selected_image_ids.contains(&img.id) {
                continue;
            }
            push_image_handles(out, img, z, input);
        }
    }

    if !input.selected_annotation_ids.is_empty() {
        for (z, ann) in input.annotations.iter().enumerate() {
            if !input.selected_annotation_ids.iter().any(|s| s == ann.id()) {
                continue;
            }
            if let Some((owner, rect, handles)) = annotation_handles(ann) {
                push_rect_handles(out, owner, ann.id(), z, rect, handles, input);
            }
        }
    }

    if let Some(sel_folder_id) = input.selected_folder_id {
        for (z, f) in input.folders.iter().enumerate() {
            if f.id != sel_folder_id {
                continue;
            }
            let rect = AlignRect::new(f.x, f.y, f.width, f.height);
            push_rect_handles(out, PickOwner::Folder, &f.id, z, rect, &Handle::ALL, input);
        }
    }
}

pub fn hit_handle(input: &PickInput) -> Option<PickCandidate> {
    let mut out = Vec::new();
    collect_handles(input, &mut out);
    if out.is_empty() {
        return None;
    }
    out.sort_by(|a, b| {
        a.dist
            .partial_cmp(&b.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.into_iter().next()
}
