//! Mutations réversibles portant sur les images, et gestes portant sur la sélection entière
//! (déplacement, duplication, suppression).

use super::journal::{Edit, Slot};
use super::Store;
use crate::types::{Annotation, BoardImage};
use std::collections::HashSet;

/// Sélection figée au début d'un geste : évite de reconstruire les `HashSet` à chaque
/// événement `CursorMoved` dans les fonctions qui les consomment plusieurs fois (R-22).
struct SelectionSets {
    images: HashSet<String>,
    annotations: HashSet<String>,
    folder: Option<String>,
}

impl Store {
    fn selection_sets(&self) -> SelectionSets {
        SelectionSets {
            images: self.selected_image_ids.iter().cloned().collect(),
            annotations: self.selected_annotation_ids.iter().cloned().collect(),
            folder: self.selected_folder_id.clone(),
        }
    }

    /// Pose une image sur un board.
    ///
    /// **Site migré vers le journal** : l'entrée d'annulation porte l'image insérée et sa
    /// place, soit quelques centaines d'octets — et non plus une copie du document entier.
    pub fn add_image(&mut self, board_id: &str, img: BoardImage) {
        let id = img.id.clone();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let index = b.images.len();
        b.images.push(img.clone());

        self.record_edit(Edit::Image {
            board: board_id.to_string(),
            slot: Slot::inserted(index, img),
        });
        self.select_image(id, false);
    }

    /// Modifie une image en place.
    ///
    /// **Site migré vers le journal.** C'est le chemin le plus chaud du store : déplacer,
    /// redimensionner, verrouiller, réassigner un domaine passent tous par ici. L'entrée
    /// d'annulation porte l'image avant et après, indépendamment de la taille du document.
    pub fn update_image<F: FnOnce(&mut BoardImage)>(&mut self, board_id: &str, id: &str, f: F) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(index) = b.images.iter().position(|i| i.id == id) else {
            return;
        };
        let before = b.images[index].clone();
        f(&mut b.images[index]);
        let after = b.images[index].clone();

        self.record_edit(Edit::Image {
            board: board_id.to_string(),
            slot: Slot::changed(index, before, after),
        });
    }

    /// Ferme l'ensemble des identifiants à supprimer par la cascade des miroirs :
    /// supprimer une image supprime toutes celles qui la reflètent, transitivement.
    fn close_image_mirror_cascade(&self, to_remove: &mut HashSet<String>) {
        let mut grew = true;
        while grew {
            grew = false;
            for b in &self.project.boards {
                for img in &b.images {
                    if let Some(ref m) = img.mirror_of {
                        if to_remove.contains(m) && !to_remove.contains(&img.id) {
                            to_remove.insert(img.id.clone());
                            grew = true;
                        }
                    }
                }
            }
        }
    }

    /// Vrai si une extrémité de cette flèche pointe vers un nœud supprimé.
    fn is_orphan_arrow(ann: &Annotation, removed: &HashSet<String>) -> bool {
        match ann {
            Annotation::Arrow {
                source_id,
                target_id,
                ..
            } => {
                source_id.as_ref().is_some_and(|s| removed.contains(s))
                    || target_id.as_ref().is_some_and(|t| removed.contains(t))
            }
            _ => false,
        }
    }

    /// Supprime des images, leurs miroirs en cascade, et les flèches devenues orphelines.
    ///
    /// **Site migré vers le journal.** Le geste peut toucher plusieurs boards à la fois : il
    /// est donc enveloppé dans une transaction, pour qu'un seul Ctrl+Z le défasse en entier.
    ///
    /// Les retraits se font **de la fin vers le début** de chaque liste. C'est ce qui rend
    /// les index enregistrés valides à la réinsertion : combiné à l'inversion d'ordre de
    /// [`crate::store::journal::Transaction`], l'undo réinsère par index croissant, chacun retrouvant sa place
    /// exacte. Retirer dans l'autre sens décalerait les index suivants.
    pub fn remove_images(&mut self, _board_id: &str, ids: &[&str]) {
        let mut to_remove: HashSet<String> = ids.iter().map(|s| s.to_string()).collect();
        self.close_image_mirror_cascade(&mut to_remove);

        // Le journal n'est pas empruntable pendant qu'on tient `&mut self.project`.
        let mut edits = Vec::new();
        for b in &mut self.project.boards {
            for i in (0..b.images.len()).rev() {
                if to_remove.contains(&b.images[i].id) {
                    let img = b.images.remove(i);
                    edits.push(Edit::Image {
                        board: b.id.clone(),
                        slot: Slot::removed(i, img),
                    });
                }
            }
            for i in (0..b.annotations.len()).rev() {
                if Self::is_orphan_arrow(&b.annotations[i], &to_remove) {
                    let ann = b.annotations.remove(i);
                    edits.push(Edit::Annotation {
                        board: b.id.clone(),
                        slot: Slot::removed(i, ann),
                    });
                }
            }
        }

        self.record_as_one_gesture(edits);
        self.selected_image_ids.clear();
    }

    /// Déplace la sélection, et traîne avec elle les flèches qui y sont attachées.
    ///
    /// **Site migré vers le journal.** Le parcours des annotations est unifié : une annotation
    /// est soit sélectionnée — elle se translate en entier —, soit une flèche attachée à la
    /// sélection — seules ses extrémités concernées suivent. Les deux cas s'excluent, ce que
    /// l'ancien `drag_connected_arrows` exprimait déjà par un `continue` ; les fusionner évite
    /// un second parcours de la liste.
    ///
    /// Un élément n'est cloné **qu'une fois su qu'il est touché** : le coût d'allocation suit
    /// la taille du geste, pas celle du board (JRN-1), même si la recherche des flèches
    /// attachées reste un balayage tant qu'il n'existe pas d'index inverse nœud → flèches.
    pub fn move_selected(&mut self, board_id: &str, dx: f64, dy: f64) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let sel = self.selection_sets();
        let mut edits = Vec::new();

        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let bid = b.id.clone();

        for (i, img) in b.images.iter_mut().enumerate() {
            if sel.images.contains(&img.id) && !img.locked {
                let before = img.clone();
                img.x += dx;
                img.y += dy;
                edits.push(Edit::Image {
                    board: bid.clone(),
                    slot: Slot::changed(i, before, img.clone()),
                });
            }
        }

        for (i, ann) in b.annotations.iter_mut().enumerate() {
            let selected = sel.annotations.contains(ann.id());
            if !selected && !arrow_follows_selection(ann, &sel) {
                continue;
            }
            let before = ann.clone();
            if selected {
                translate_annotation(ann, dx, dy);
            } else {
                drag_arrow_ends(ann, &sel, dx, dy);
            }
            edits.push(Edit::Annotation {
                board: bid.clone(),
                slot: Slot::changed(i, before, ann.clone()),
            });
        }

        if let Some(fid) = sel.folder.as_deref() {
            for (i, f) in b.folders.iter_mut().enumerate() {
                if f.id == fid {
                    let before = f.clone();
                    f.x += dx;
                    f.y += dy;
                    edits.push(Edit::Folder {
                        board: bid.clone(),
                        slot: Slot::changed(i, before, f.clone()),
                    });
                }
            }
        }

        self.record_as_one_gesture(edits);
    }

    pub fn duplicate_selected(&mut self, board_id: &str) {
        if self.selected_image_ids.is_empty() && self.selected_annotation_ids.is_empty() {
            return;
        }
        let (mut new_imgs, mut new_anns) = self.clone_selection(board_id);
        const OFFSET: f64 = 20.0;

        for img in &mut new_imgs {
            img.id = self.generate_id("img");
            img.x += OFFSET;
            img.y += OFFSET;
        }
        for ann in &mut new_anns {
            let prefix = if matches!(ann, Annotation::Arrow { .. }) {
                "arrow"
            } else {
                "ann"
            };
            let fresh = self.generate_id(prefix);
            set_annotation_id(ann, fresh);
            translate_annotation(ann, OFFSET, OFFSET);
        }

        self.selected_image_ids = new_imgs.iter().map(|i| i.id.clone()).collect();
        self.selected_annotation_ids = new_anns.iter().map(|a| a.id().to_string()).collect();

        let mut edits = Vec::new();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            for img in new_imgs {
                let index = b.images.len();
                b.images.push(img.clone());
                edits.push(Edit::Image {
                    board: board_id.to_string(),
                    slot: Slot::inserted(index, img),
                });
            }
            for ann in new_anns {
                let index = b.annotations.len();
                b.annotations.push(ann.clone());
                edits.push(Edit::Annotation {
                    board: board_id.to_string(),
                    slot: Slot::inserted(index, ann),
                });
            }
        }
        self.record_as_one_gesture(edits);
    }

    /// Copies brutes des éléments sélectionnés, avant réattribution d'identifiants.
    fn clone_selection(&self, board_id: &str) -> (Vec<BoardImage>, Vec<Annotation>) {
        let mut imgs = Vec::new();
        let mut anns = Vec::new();
        if let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) {
            for id in &self.selected_image_ids {
                if let Some(img) = b.images.iter().find(|i| &i.id == id) {
                    imgs.push(img.clone());
                }
            }
            for id in &self.selected_annotation_ids {
                if let Some(ann) = b.annotations.iter().find(|a| a.id() == id) {
                    anns.push(ann.clone());
                }
            }
        }
        (imgs, anns)
    }

    pub fn delete_selected(&mut self, board_id: &str) {
        let imgs = self.selected_image_ids.clone();
        let anns = self.selected_annotation_ids.clone();
        let f_opt = self.selected_folder_id.clone();

        if !imgs.is_empty() {
            let refs: Vec<&str> = imgs.iter().map(|s| s.as_str()).collect();
            self.remove_images(board_id, &refs);
        }
        if !anns.is_empty() {
            let refs: Vec<&str> = anns.iter().map(|s| s.as_str()).collect();
            self.remove_annotations(board_id, &refs);
        }
        if let Some(ref fid) = f_opt {
            self.remove_folders(board_id, &[fid]);
        }
    }
}

/// Translation d'une annotation, extrémités et waypoints compris.
pub(super) fn translate_annotation(ann: &mut Annotation, dx: f64, dy: f64) {
    match ann {
        Annotation::Text { x, y, .. }
        | Annotation::Sticky { x, y, .. }
        | Annotation::Membrane { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Annotation::Arrow {
            x,
            y,
            x2,
            y2,
            waypoints,
            ..
        } => {
            *x += dx;
            *y += dy;
            *x2 += dx;
            *y2 += dy;
            for wp in waypoints {
                wp.x += dx;
                wp.y += dy;
            }
        }
    }
}

fn set_annotation_id(ann: &mut Annotation, new_id: String) {
    match ann {
        Annotation::Text { id, .. }
        | Annotation::Sticky { id, .. }
        | Annotation::Membrane { id, .. }
        | Annotation::Arrow { id, .. } => *id = new_id,
    }
}

/// R-12 — une flèche non sélectionnée suit l'extrémité dont le nœud, lui, bouge.
/// Vrai si cette extrémité est attachée à un nœud que le geste déplace.
fn end_follows(id: &Option<String>, sel: &SelectionSets) -> bool {
    id.as_ref()
        .is_some_and(|s| sel.annotations.contains(s) || sel.images.contains(s))
}

/// Vrai si cette annotation est une flèche dont au moins une extrémité suit la sélection.
fn arrow_follows_selection(ann: &Annotation, sel: &SelectionSets) -> bool {
    match ann {
        Annotation::Arrow {
            source_id,
            target_id,
            ..
        } => end_follows(source_id, sel) || end_follows(target_id, sel),
        _ => false,
    }
}

/// Traîne les extrémités d'une flèche attachées à la sélection. Sans effet sur autre chose.
fn drag_arrow_ends(ann: &mut Annotation, sel: &SelectionSets, dx: f64, dy: f64) {
    let Annotation::Arrow {
        source_id,
        target_id,
        x,
        y,
        x2,
        y2,
        ..
    } = ann
    else {
        return;
    };
    if end_follows(source_id, sel) {
        *x += dx;
        *y += dy;
    }
    if end_follows(target_id, sel) {
        *x2 += dx;
        *y2 += dy;
    }
}
