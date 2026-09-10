//! State Store, Machine d'État et pile Undo/Redo infinie — 0 dépendance.
//! Architecture Pure Rust std avec Invariants UNDO-1 :
//! - Navigation transparente (pan, zoom, active board, dossier ne polluent jamais l'undo)
//! - Caméra préservée à travers undo/redo (pas de téléportation)
//! - Transactions live atomiques (begin_live_edit / end_live_edit)
//! - Cascade de suppression miroirs & flèches orphelines

use crate::types::{
    Annotation, Board, BoardImage, BoardZone, CanvasFolder, Domain, FolderTreeNode,
    Preset, Project, StoryboardPanel, TemporalAnchor, Viewport,
};
use std::collections::HashSet;

/// Reconstruit la pile de dossiers UI menant au board actif.
pub fn build_folder_stack(boards: &[Board], active_board_id: &str) -> Vec<(String, String)> {
    let mut parent_map = std::collections::HashMap::new();
    for b in boards {
        for f in &b.folders {
            parent_map.insert(f.child_board_id.clone(), (b.id.clone(), f.id.clone()));
        }
    }
    let mut stack = Vec::new();
    let mut curr = active_board_id.to_string();
    let mut guard = 256;
    while let Some(parent) = parent_map.get(&curr) {
        if guard == 0 {
            break;
        }
        guard -= 1;
        stack.insert(0, parent.clone());
        curr = parent.0.clone();
    }
    stack
}

/// Préserve la caméra et le board actif lors d'un Undo ou Redo.
pub fn preserve_view(restored: &mut Project, cur: &Project) {
    if restored.boards.iter().any(|b| b.id == cur.active_board_id) {
        restored.active_board_id = cur.active_board_id.clone();
    } else if let Some(first) = restored.boards.first() {
        restored.active_board_id = first.id.clone();
    }
    for b in &mut restored.boards {
        if let Some(cb) = cur.boards.iter().find(|x| x.id == b.id) {
            b.viewport = cb.viewport;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    pub project: Project,
    pub selected_image_ids: Vec<String>,
    pub selected_annotation_ids: Vec<String>,
    pub selected_folder_id: Option<String>,
    pub folder_stack: Vec<(String, String)>,
    pub temporal_filter: Option<TemporalAnchor>,

    pub undo_stack: Vec<Project>,
    pub redo_stack: Vec<Project>,
    pub max_undo: usize,
    pub in_live_edit: bool,
    pub next_id: u64,
    pub version: u64,
}

impl Store {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project: Project::new(project_name),
            selected_image_ids: Vec::new(),
            selected_annotation_ids: Vec::new(),
            selected_folder_id: None,
            folder_stack: Vec::new(),
            temporal_filter: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 200,
            in_live_edit: false,
            next_id: 1,
            version: 1,
        }
    }

    pub fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    pub fn id_exists(&self, id: &str) -> bool {
        self.project.boards.iter().any(|b| {
            b.id == id
                || b.images.iter().any(|i| i.id == id)
                || b.annotations.iter().any(|a| a.id() == id)
                || b.folders.iter().any(|f| f.id == id || f.child_board_id == id)
        })
    }

    pub fn generate_id(&mut self, prefix: &str) -> String {
        loop {
            self.next_id += 1;
            let candidate = format!("{}-{}", prefix, self.next_id);
            if !self.id_exists(&candidate) {
                return candidate;
            }
        }
    }

    pub fn load_project(&mut self, project: Project) {
        self.project = project;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.clear_selection();
        self.folder_stack = build_folder_stack(&self.project.boards, &self.project.active_board_id);
        self.in_live_edit = false;
    }

    pub fn active_board(&self) -> Option<&Board> {
        self.project.boards.iter().find(|b| b.id == self.project.active_board_id)
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        let id = self.project.active_board_id.clone();
        self.project.boards.iter_mut().find(|b| b.id == id)
    }

    // ── Navigation (NE TOUCHE JAMAIS À L'UNDO/REDO) ──────────────────────────

    pub fn set_active_board_id(&mut self, board_id: impl Into<String>) {
        let bid = board_id.into();
        if self.project.boards.iter().any(|b| b.id == bid) {
            self.project.active_board_id = bid.clone();
            self.folder_stack = build_folder_stack(&self.project.boards, &bid);
            self.clear_selection();
        }
    }

    pub fn set_viewport(&mut self, board_id: &str, vp: Viewport) {
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.viewport = vp;
        }
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        if let Some(b) = self.active_board_mut() {
            b.viewport.x += dx;
            b.viewport.y += dy;
        }
    }

    pub fn zoom(&mut self, factor: f64, cursor_x: f64, cursor_y: f64) {
        if let Some(b) = self.active_board_mut() {
            let old_scale = b.viewport.scale;
            let new_scale = (old_scale * factor).clamp(0.01, 50.0);
            b.viewport.x = cursor_x - (cursor_x - b.viewport.x) * (new_scale / old_scale);
            b.viewport.y = cursor_y - (cursor_y - b.viewport.y) * (new_scale / old_scale);
            b.viewport.scale = new_scale;
        }
    }

    pub fn enter_folder(&mut self, folder_id: &str) {
        let current_board_id = self.project.active_board_id.clone();
        if let Some(b) = self.active_board() {
            if let Some(folder) = b.folders.iter().find(|f| f.id == folder_id) {
                let target = folder.child_board_id.clone();
                self.folder_stack.push((current_board_id, folder_id.to_string()));
                self.project.active_board_id = target;
                self.clear_selection();
            }
        }
    }

    pub fn exit_folder(&mut self) {
        if let Some((parent_board_id, _)) = self.folder_stack.pop() {
            self.project.active_board_id = parent_board_id;
            self.clear_selection();
        }
    }

    pub fn exit_to_root(&mut self) {
        if let Some((root_board_id, _)) = self.folder_stack.first().cloned() {
            self.folder_stack.clear();
            self.project.active_board_id = root_board_id;
            self.clear_selection();
        }
    }

    pub fn expand_folder(&mut self, _parent_board_id: &str, folder_id: &str, level: FolderTreeNode) {
        let child_board_id = {
            let mut found = None;
            for b in &self.project.boards {
                if let Some(f) = b.folders.iter().find(|f| f.id == folder_id) {
                    found = Some(f.child_board_id.clone());
                    break;
                }
            }
            found
        };

        if let Some(child_id) = child_board_id {
            if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == child_id) {
                b.annotations.extend(level.annotations);
                b.images.extend(level.images);
            }
        }
    }

    pub fn set_temporal_filter(&mut self, filter: Option<TemporalAnchor>) {
        self.temporal_filter = filter;
    }

    // ── Sélection ───────────────────────────────────────────────────────────

    pub fn clear_selection(&mut self) {
        self.selected_image_ids.clear();
        self.selected_annotation_ids.clear();
        self.selected_folder_id = None;
    }

    pub fn set_selected_image_ids(&mut self, ids: Vec<String>) {
        self.selected_image_ids = ids;
    }

    pub fn set_selected_annotation_ids(&mut self, ids: Vec<String>) {
        self.selected_annotation_ids = ids;
    }

    pub fn select_image(&mut self, id: String, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        if !self.selected_image_ids.contains(&id) {
            self.selected_image_ids.push(id);
        }
    }

    pub fn select_annotation(&mut self, id: String, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        if !self.selected_annotation_ids.contains(&id) {
            self.selected_annotation_ids.push(id);
        }
    }

    pub fn select_folder(&mut self, id: String) {
        self.clear_selection();
        self.selected_folder_id = Some(id);
    }

    // ── Gestion Undo / Redo ─────────────────────────────────────────────────

    pub fn push_undo(&mut self) {
        if self.in_live_edit {
            return;
        }
        self.undo_stack.push(self.project.clone());
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.bump_version();
    }

    pub fn begin_live_edit(&mut self) {
        if self.in_live_edit {
            return;
        }
        self.push_undo();
        self.in_live_edit = true;
    }

    pub fn end_live_edit(&mut self) {
        self.in_live_edit = false;
        self.bump_version();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if let Some(mut prev) = self.undo_stack.pop() {
            let current = self.project.clone();
            preserve_view(&mut prev, &current);
            self.redo_stack.push(current);
            self.project = prev;
            self.clear_selection();
            self.folder_stack = build_folder_stack(&self.project.boards, &self.project.active_board_id);
            self.in_live_edit = false;
            self.bump_version();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut next) = self.redo_stack.pop() {
            let current = self.project.clone();
            preserve_view(&mut next, &current);
            self.undo_stack.push(current);
            self.project = next;
            self.clear_selection();
            self.folder_stack = build_folder_stack(&self.project.boards, &self.project.active_board_id);
            self.in_live_edit = false;
            self.bump_version();
            true
        } else {
            false
        }
    }

    pub fn sync_annotation_size(&mut self, board_id: &str, ann_id: &str, width: f64, height: f64) {
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            for ann in &mut b.annotations {
                if ann.id() == ann_id {
                    match ann {
                        Annotation::Text { width: w, height: h, .. }
                        | Annotation::Sticky { width: w, height: h, .. } => {
                            *w = Some(width);
                            *h = Some(height);
                        }
                        Annotation::Membrane { width: w, height: h, .. } => {
                            *w = width;
                            *h = height;
                        }
                        Annotation::Arrow { .. } => {}
                    }
                    break;
                }
            }
        }
    }

    // ── Mutations Réversibles (Images) ──────────────────────────────────────

    pub fn add_image(&mut self, board_id: &str, img: BoardImage) {
        self.push_undo();
        let id = img.id.clone();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.images.push(img);
        }
        self.select_image(id, false);
    }

    pub fn update_image<F: FnOnce(&mut BoardImage)>(&mut self, board_id: &str, id: &str, f: F) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(img) = b.images.iter_mut().find(|i| i.id == id) {
                f(img);
            }
        }
    }

    pub fn remove_images(&mut self, _board_id: &str, ids: &[&str]) {
        self.push_undo();
        let mut to_remove: HashSet<String> = ids.iter().map(|s| s.to_string()).collect();

        // Cascade miroirs
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

        // Suppression dans tous les boards + suppression des flèches orphelines
        for b in &mut self.project.boards {
            b.images.retain(|img| !to_remove.contains(&img.id));
            b.annotations.retain(|a| match a {
                Annotation::Arrow { source_id, target_id, .. } => {
                    let src_orphan = source_id.as_ref().is_some_and(|s| to_remove.contains(s));
                    let tgt_orphan = target_id.as_ref().is_some_and(|t| to_remove.contains(t));
                    !src_orphan && !tgt_orphan
                }
                _ => true,
            });
        }
        self.selected_image_ids.clear();
    }

    pub fn move_selected(&mut self, board_id: &str, dx: f64, dy: f64) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.push_undo();

        let sel_img: HashSet<String> = self.selected_image_ids.iter().cloned().collect();
        let sel_ann: HashSet<String> = self.selected_annotation_ids.iter().cloned().collect();
        let sel_f = self.selected_folder_id.clone();

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            for img in &mut b.images {
                if sel_img.contains(&img.id) && !img.locked {
                    img.x += dx;
                    img.y += dy;
                }
            }
            for ann in &mut b.annotations {
                if sel_ann.contains(ann.id()) {
                    match ann {
                        Annotation::Text { x, y, .. }
                        | Annotation::Sticky { x, y, .. }
                        | Annotation::Membrane { x, y, .. } => {
                            *x += dx;
                            *y += dy;
                        }
                        Annotation::Arrow { x, y, x2, y2, waypoints, .. } => {
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
            }
            if let Some(fid) = sel_f {
                for f in &mut b.folders {
                    if f.id == fid {
                        f.x += dx;
                        f.y += dy;
                    }
                }
            }
        }
    }

    pub fn duplicate_selected(&mut self, board_id: &str) {
        if self.selected_image_ids.is_empty() && self.selected_annotation_ids.is_empty() {
            return;
        }
        self.push_undo();

        let offset = 20.0;
        let mut new_imgs = Vec::new();
        let mut new_anns = Vec::new();

        if let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) {
            for id in &self.selected_image_ids {
                if let Some(img) = b.images.iter().find(|i| &i.id == id) {
                    new_imgs.push(img.clone());
                }
            }
            for id in &self.selected_annotation_ids {
                if let Some(ann) = b.annotations.iter().find(|a| a.id() == id) {
                    new_anns.push(ann.clone());
                }
            }
        }

        for img in &mut new_imgs {
            img.id = self.generate_id("img");
            img.x += offset;
            img.y += offset;
        }

        for ann in &mut new_anns {
            match ann {
                Annotation::Text { id, x, y, .. }
                | Annotation::Sticky { id, x, y, .. }
                | Annotation::Membrane { id, x, y, .. } => {
                    *id = self.generate_id("ann");
                    *x += offset;
                    *y += offset;
                }
                Annotation::Arrow { id, x, y, x2, y2, waypoints, .. } => {
                    *id = self.generate_id("arrow");
                    *x += offset;
                    *y += offset;
                    *x2 += offset;
                    *y2 += offset;
                    for wp in waypoints {
                        wp.x += offset;
                        wp.y += offset;
                    }
                }
            }
        }

        self.selected_image_ids = new_imgs.iter().map(|i| i.id.clone()).collect();
        self.selected_annotation_ids = new_anns.iter().map(|a| a.id().to_string()).collect();

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.images.extend(new_imgs);
            b.annotations.extend(new_anns);
        }
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

    // ── Mutations Réversibles (Annotations) ──────────────────────────────────

    pub fn add_annotation(&mut self, board_id: &str, ann: Annotation) {
        self.push_undo();
        let id = ann.id().to_string();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.annotations.push(ann);
        }
        self.select_annotation(id, false);
    }

    pub fn update_annotation<F: FnOnce(&mut Annotation)>(&mut self, board_id: &str, id: &str, f: F) {
        self.push_undo();
        let mut old_pos = None;
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == id) {
                old_pos = Some((ann.x(), ann.y()));
                f(ann);
            }
            // Suivi des flèches connectées si la source a bougé
            if let Some((_ox, _oy)) = old_pos {
                if let Some(cur_ann) = b.annotations.iter().find(|a| a.id() == id) {
                    let (nx, ny) = (cur_ann.x(), cur_ann.y());
                    for a in &mut b.annotations {
                        if let Annotation::Arrow { ref source_id, ref mut x, ref mut y, .. } = a {
                            if source_id.as_deref() == Some(id) {
                                *x = nx;
                                *y = ny;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn remove_annotations(&mut self, _board_id: &str, ids: &[&str]) {
        self.push_undo();
        let id_set: HashSet<&str> = ids.iter().copied().collect();

        for b in &mut self.project.boards {
            b.annotations.retain(|a| !id_set.contains(a.id()));
            b.annotations.retain(|a| match a {
                Annotation::Arrow { source_id, target_id, .. } => {
                    let src_orphan = source_id.as_ref().is_some_and(|s| id_set.contains(s.as_str()));
                    let tgt_orphan = target_id.as_ref().is_some_and(|t| id_set.contains(t.as_str()));
                    !src_orphan && !tgt_orphan
                }
                _ => true,
            });
        }
        self.selected_annotation_ids.clear();
    }

    pub fn mirror_annotation(&mut self, board_id: &str, id: &str, x: f64, y: f64) -> Option<String> {
        self.push_undo();
        let mid = self.generate_id("mirror");
        let mut mirror = None;
        if let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) {
            if let Some(orig) = b.annotations.iter().find(|a| a.id() == id) {
                let mut m = orig.clone();
                match &mut m {
                    Annotation::Text { id: ref mut aid, x: ref mut ax, y: ref mut ay, ref mut mirror_of, .. }
                    | Annotation::Sticky { id: ref mut aid, x: ref mut ax, y: ref mut ay, ref mut mirror_of, .. }
                    | Annotation::Membrane { id: ref mut aid, x: ref mut ax, y: ref mut ay, ref mut mirror_of, .. } => {
                        *aid = mid.clone();
                        *ax = x;
                        *ay = y;
                        *mirror_of = Some(id.to_string());
                    }
                    Annotation::Arrow { id: ref mut aid, x: ref mut ax, y: ref mut ay, x2: ref mut ax2, y2: ref mut ay2, .. } => {
                        *aid = mid.clone();
                        let dx = x - *ax;
                        let dy = y - *ay;
                        *ax = x;
                        *ay = y;
                        *ax2 += dx;
                        *ay2 += dy;
                    }
                }
                mirror = Some((mid.clone(), m));
            }
        }

        if let Some((mid, m)) = mirror {
            if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
                b.annotations.push(m);
            }
            Some(mid)
        } else {
            None
        }
    }

    // ── Storyboard Panels ───────────────────────────────────────────────────

    pub fn add_panel(&mut self, board_id: &str, panel: StoryboardPanel) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.panels.push(panel);
        }
    }

    pub fn update_panel(&mut self, board_id: &str, id: &str, description: impl Into<String>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(p) = b.panels.iter_mut().find(|p| p.id == id) {
                p.description = description.into();
            }
        }
    }

    pub fn remove_panel(&mut self, board_id: &str, id: &str) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.panels.retain(|p| p.id != id);
        }
    }

    // ── Boards & Dossiers ───────────────────────────────────────────────────

    pub fn rename_board(&mut self, board_id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.name = name.into();
        }
    }

    pub fn add_board(&mut self, name: impl Into<String>) -> String {
        self.push_undo();
        let id = self.generate_id("board");
        self.project.boards.push(Board::new(&id, name));
        id
    }

    pub fn remove_board(&mut self, id: &str) -> bool {
        if self.project.boards.len() <= 1 {
            return false;
        }
        self.push_undo();
        self.project.boards.retain(|b| b.id != id);
        if self.project.active_board_id == id {
            self.project.active_board_id = self.project.boards[0].id.clone();
        }
        // Flèches portails pointant vers le board supprimé
        for b in &mut self.project.boards {
            for a in &mut b.annotations {
                if let Annotation::Arrow { ref mut target_board_id, .. } = a {
                    if target_board_id.as_deref() == Some(id) {
                        *target_board_id = None;
                    }
                }
            }
        }
        true
    }

    pub fn create_folder(&mut self, parent_board_id: &str, mut folder: CanvasFolder) {
        self.push_undo();
        if folder.id.is_empty() {
            folder.id = self.generate_id("folder");
        }
        let child_board_id = self.generate_id("board");
        folder.child_board_id = child_board_id.clone();

        let fx0 = folder.x;
        let fy0 = folder.y;
        let fx1 = folder.x + folder.width;
        let fy1 = folder.y + folder.height;
        let inside = |cx: f64, cy: f64| cx >= fx0 && cx <= fx1 && cy >= fy0 && cy <= fy1;

        let mut child_images = Vec::new();
        let mut child_annotations = Vec::new();

        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            let mut captured_img_ids = HashSet::new();
            let mut captured_ann_ids = HashSet::new();

            for img in &par.images {
                if inside(img.x, img.y) {
                    captured_img_ids.insert(img.id.clone());
                }
            }
            for ann in &par.annotations {
                if inside(ann.x(), ann.y()) {
                    captured_ann_ids.insert(ann.id().to_string());
                }
            }

            par.images.retain(|img| {
                if captured_img_ids.contains(&img.id) {
                    let mut moved = img.clone();
                    moved.x -= fx0;
                    moved.y -= fy0;
                    child_images.push(moved);
                    false
                } else {
                    true
                }
            });

            par.annotations.retain(|ann| {
                if captured_ann_ids.contains(ann.id()) {
                    let mut moved = ann.clone();
                    match &mut moved {
                        Annotation::Text { x, y, .. }
                        | Annotation::Sticky { x, y, .. }
                        | Annotation::Membrane { x, y, .. } => {
                            *x -= fx0;
                            *y -= fy0;
                        }
                        Annotation::Arrow { x, y, x2, y2, waypoints, .. } => {
                            *x -= fx0;
                            *y -= fy0;
                            *x2 -= fx0;
                            *y2 -= fy0;
                            for wp in waypoints {
                                wp.x -= fx0;
                                wp.y -= fy0;
                            }
                        }
                    }
                    child_annotations.push(moved);
                    false
                } else {
                    true
                }
            });

            par.folders.push(folder.clone());
        }

        let mut child_board = Board::new(&child_board_id, &folder.name);
        child_board.images = child_images;
        child_board.annotations = child_annotations;
        self.project.boards.push(child_board);
    }

    pub fn create_folder_with_content(
        &mut self,
        parent_board_id: &str,
        mut folder: CanvasFolder,
        seed_annotations: Vec<Annotation>,
    ) -> String {
        self.push_undo();
        let folder_id = if folder.id.is_empty() {
            self.generate_id("folder")
        } else {
            folder.id.clone()
        };
        let child_board_id = self.generate_id("board");
        folder.id = folder_id.clone();
        folder.child_board_id = child_board_id.clone();

        let mut child_board = Board::new(&child_board_id, &folder.name);
        child_board.annotations = seed_annotations;
        self.project.boards.push(child_board);

        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            par.folders.push(folder);
        }

        folder_id
    }

    pub fn update_folder<F: FnOnce(&mut CanvasFolder)>(&mut self, parent_board_id: &str, id: &str, f: F) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            if let Some(fld) = b.folders.iter_mut().find(|f| f.id == id) {
                f(fld);
            }
        }
    }

    pub fn remove_folders(&mut self, parent_board_id: &str, ids: &[&str]) {
        self.push_undo();
        let id_set: HashSet<&str> = ids.iter().copied().collect();
        let mut child_board_ids = Vec::new();

        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            for f in &par.folders {
                if id_set.contains(f.id.as_str()) {
                    child_board_ids.push(f.child_board_id.clone());
                }
            }
            par.folders.retain(|f| !id_set.contains(f.id.as_str()));
        }

        self.project.boards.retain(|b| !child_board_ids.contains(&b.id));
        if child_board_ids.contains(&self.project.active_board_id) {
            self.project.active_board_id = parent_board_id.to_string();
        }
    }

    pub fn mirror_folder(&mut self, parent_board_id: &str, folder_id: &str, x: f64, y: f64) -> Option<String> {
        self.push_undo();
        let mid = self.generate_id("mirror-folder");
        let mut mirrored = None;
        if let Some(b) = self.project.boards.iter().find(|b| b.id == parent_board_id) {
            if let Some(f) = b.folders.iter().find(|f| f.id == folder_id) {
                let mut m = f.clone();
                m.id = mid.clone();
                m.x = x;
                m.y = y;
                m.mirror_of = Some(folder_id.to_string());
                mirrored = Some((mid.clone(), m));
            }
        }

        if let Some((mid, m)) = mirrored {
            if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
                b.folders.push(m);
            }
            Some(mid)
        } else {
            None
        }
    }

    // ── Presets & Domaines ──────────────────────────────────────────────────

    pub fn add_preset(&mut self, preset: Preset) {
        self.push_undo();
        self.project.presets.push(preset);
    }

    pub fn update_preset(&mut self, id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(p) = self.project.presets.iter_mut().find(|p| p.id == id) {
            p.name = name.into();
        }
    }

    pub fn remove_preset(&mut self, id: &str) {
        self.push_undo();
        self.project.presets.retain(|p| p.id != id);
    }

    pub fn apply_preset_to_board(&mut self, board_id: &str, preset_id: Option<&str>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.zones.clear();
            if let Some(pid) = preset_id {
                if let Some(preset) = self.project.presets.iter().find(|p| p.id == pid) {
                    for (i, slot) in preset.slots.iter().enumerate() {
                        b.zones.push(BoardZone::new(
                            &slot.id,
                            (i as f64) * 250.0,
                            0.0,
                            240.0,
                            180.0,
                        ));
                    }
                }
            }
        }
    }

    pub fn add_domain(&mut self, domain: Domain) {
        self.push_undo();
        self.project.domains.push(domain);
    }

    pub fn update_domain(&mut self, id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(d) = self.project.domains.iter_mut().find(|d| d.id == id) {
            d.name = name.into();
        }
    }

    pub fn remove_domain(&mut self, id: &str) {
        self.push_undo();
        self.project.domains.retain(|d| d.id != id);
    }

    pub fn assign_domain_to_node(&mut self, board_id: &str, node_id: &str, domain_id: &str, weight: f64) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            for ann in &mut b.annotations {
                if ann.id() == node_id {
                    let dm = ann.domains_mut();
                    if let Some(existing) = dm.iter_mut().find(|d| d.domain_id == domain_id) {
                        existing.weight = weight;
                    } else {
                        dm.push(crate::types::DomainAssignment {
                            domain_id: domain_id.to_string(),
                            weight,
                        });
                    }
                    break;
                }
            }
        }
    }

    pub fn set_project_name(&mut self, name: impl Into<String>) {
        self.push_undo();
        self.project.name = name.into();
    }

    pub fn set_board_zones(&mut self, board_id: &str, zones: Vec<BoardZone>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.zones = zones;
        }
    }
}
