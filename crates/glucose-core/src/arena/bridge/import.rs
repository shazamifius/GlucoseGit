//! Du modèle historique vers l'arène.
//!
//! Deux passes, et c'est la seule façon d'y arriver : un nœud peut désigner un nœud qui n'a pas
//! encore été créé — une flèche vers une carte plus loin dans la liste, une membrane parente
//! déclarée après ses enfants. La première passe crée et nomme, la seconde résout.

use super::super::doc::{ArrowTrait, Doc, ImageTrait};
use super::super::{Box2, Flags, Kind, NodeId};
use super::{Bridge, ByName, DEFAULT_CARD_HEIGHT, DEFAULT_CARD_WIDTH};
use crate::fixed::Fx;
use crate::types::{Annotation, Board, BoardImage, CanvasFolder, StoryboardPanel};
use std::collections::HashMap;

/// La boîte d'un nœud, avec les drapeaux disant quelles dimensions sont calculées.
fn box_with_auto(x: f64, y: f64, w: Option<f64>, h: Option<f64>) -> (Box2, Flags) {
    let flags = Flags::default()
        .set(Flags::AUTO_WIDTH, w.is_none())
        .set(Flags::AUTO_HEIGHT, h.is_none());
    let b = Box2::new(
        Fx::from_f64(x),
        Fx::from_f64(y),
        Fx::from_f64(w.unwrap_or(DEFAULT_CARD_WIDTH)),
        Fx::from_f64(h.unwrap_or(DEFAULT_CARD_HEIGHT)),
    );
    (b, flags)
}

/// Le genre d'arène correspondant à une annotation.
fn kind_of(a: &Annotation) -> Kind {
    match a {
        Annotation::Text { .. } => Kind::Text,
        Annotation::Sticky { .. } => Kind::Sticky,
        Annotation::Arrow { .. } => Kind::Arrow,
        Annotation::Membrane { .. } => Kind::Membrane,
    }
}

impl Bridge {
    /// Porte un tableau du modèle historique dans l'arène.
    pub fn from_board(board: &Board) -> Self {
        let n = board.images.len() + board.annotations.len() + board.folders.len() + board.panels.len();
        let mut b = Self {
            doc: Doc::with_capacity(n),
            names: Vec::with_capacity(n),
        };
        let mut by_name: ByName<'_> = HashMap::with_capacity(n);
        // Les identifiants sont RETENUS, pas recalculés : les déduire de l'ordre de création
        // marcherait aujourd'hui et casserait au premier nœud sauté.
        let mut des_images = Vec::with_capacity(board.images.len());
        let mut des_annotations = Vec::with_capacity(board.annotations.len());
        let mut des_dossiers = Vec::with_capacity(board.folders.len());

        for img in &board.images {
            let id = b.spawn_image(img);
            by_name.insert(&img.id, id);
            des_images.push(id);
        }
        for ann in &board.annotations {
            let id = b.spawn_annotation(ann);
            by_name.insert(ann.id(), id);
            des_annotations.push(id);
        }
        for f in &board.folders {
            let id = b.spawn_folder(f);
            by_name.insert(&f.id, id);
            des_dossiers.push(id);
        }
        for p in &board.panels {
            let id = b.spawn_panel(p);
            by_name.insert(&p.id, id);
        }

        b.resolve_references(board, &by_name, &des_images, &des_annotations, &des_dossiers);
        b
    }

    /// Crée un nœud, retient son nom, et rend son identifiant.
    fn intern(&mut self, name: &str, kind: Kind, bx: Box2, flags: Flags) -> NodeId {
        let id = self.doc.spawn(kind, bx, NodeId::NONE);
        self.doc.nodes.set_flags(id, flags);
        self.names.push(name.into());
        id
    }

    fn spawn_image(&mut self, img: &BoardImage) -> NodeId {
        let bx = Box2::new(
            Fx::from_f64(img.x),
            Fx::from_f64(img.y),
            Fx::from_f64(img.width),
            Fx::from_f64(img.height),
        );
        let flags = Flags::default()
            .set(Flags::LOCKED, img.locked)
            .set(Flags::VIDEO, img.is_video);
        let id = self.intern(&img.id, Kind::Image, bx, flags);

        if img.rotation != 0.0 {
            self.doc.rotation.set(id, img.rotation as f32);
        }
        let propre = ImageTrait {
            src: img.src.as_deref().map(Into::into),
            source_url: img.source_url.as_deref().map(Into::into),
            slot_id: img.slot_id.as_deref().map(Into::into),
            fit: img.fit.as_deref().map(Into::into),
            tags: img.tags.iter().map(|t| t.as_str().into()).collect(),
        };
        if !propre.is_empty() {
            self.doc.image.set(id, propre);
        }
        let origine = (img.original_width as f32, img.original_height as f32);
        if origine != (0.0, 0.0) {
            self.doc.original.set(id, origine);
        }
        if let Some(a) = &img.asset {
            self.doc.asset.set(id, a.clone());
        }
        if !img.domains.is_empty() {
            self.doc.domains.set(id, img.domains.clone());
        }
        if let Some(t) = &img.temporal_anchor {
            self.doc.temporal.set(id, t.clone());
        }
        id
    }

    fn spawn_annotation(&mut self, ann: &Annotation) -> NodeId {
        let id = match ann {
            Annotation::Text { x, y, width, height, .. }
            | Annotation::Sticky { x, y, width, height, .. } => {
                let (bx, flags) = box_with_auto(*x, *y, *width, *height);
                self.intern(ann.id(), kind_of(ann), bx, flags)
            }
            Annotation::Arrow { id, x, y, x2, y2, arrow_bidirectional, .. } => {
                let bx = Box2::spanning(
                    Fx::from_f64(*x),
                    Fx::from_f64(*y),
                    Fx::from_f64(*x2),
                    Fx::from_f64(*y2),
                );
                let node = self.intern(id, Kind::Arrow, bx, Flags::default());
                self.doc.nodes.set_arrow(
                    node,
                    Fx::from_f64(*x),
                    Fx::from_f64(*y),
                    Fx::from_f64(*x2),
                    Fx::from_f64(*y2),
                );
                // `set_arrow` réécrit les drapeaux de coin : le sens double se pose après.
                let f = self.doc.nodes.flags_of(node).unwrap_or_default();
                self.doc
                    .nodes
                    .set_flags(node, f.set(Flags::ARROW_BIDIRECTIONAL, *arrow_bidirectional));
                node
            }
            Annotation::Membrane { id, x, y, width, height, .. } => {
                let bx = Box2::new(
                    Fx::from_f64(*x),
                    Fx::from_f64(*y),
                    Fx::from_f64(*width),
                    Fx::from_f64(*height),
                );
                self.intern(id, Kind::Membrane, bx, Flags::default())
            }
        };
        self.fill_annotation(id, ann);
        id
    }

    /// Les attributs communs et propres d'une annotation, une fois son nœud créé.
    fn fill_annotation(&mut self, id: NodeId, ann: &Annotation) {
        match ann {
            Annotation::Text { text, font_size, color, cursor_pos, source_file, .. } => {
                self.doc.text.set(id, text);
                self.set_common(id, *font_size, color.as_deref(), *cursor_pos, source_file.as_deref());
            }
            Annotation::Sticky {
                text, font_size, color, bg_color, cursor_pos, operator, source_file, ..
            } => {
                self.doc.text.set(id, text);
                self.set_common(id, *font_size, color.as_deref(), *cursor_pos, source_file.as_deref());
                if let Some(bg) = bg_color {
                    self.doc.set_background(id, bg);
                }
                if let Some(op) = operator {
                    self.doc.operator.set(id, *op);
                }
            }
            Annotation::Arrow {
                text, font_size, color, arrow_type, predicate, stroke_width, waypoints,
                source_block_id, target_block_id, source_text_sel, target_text_sel, long_text,
                target_board_id, ..
            } => {
                if let Some(t) = text {
                    self.doc.text.set(id, t);
                }
                self.set_common(id, *font_size, color.as_deref(), None, None);
                self.doc.arrow.set(
                    id,
                    ArrowTrait {
                        predicate: *predicate,
                        kind: arrow_type.as_deref().map(Into::into),
                        stroke_width: stroke_width.map(|w| w as f32),
                        source: NodeId::NONE,
                        target: NodeId::NONE,
                        waypoints: waypoints
                            .iter()
                            .map(|p| (Fx::from_f64(p.x), Fx::from_f64(p.y)))
                            .collect(),
                        long_text: long_text.as_deref().map(Into::into),
                        target_board: target_board_id.as_deref().map(Into::into),
                        source_block: source_block_id.as_deref().map(Into::into),
                        target_block: target_block_id.as_deref().map(Into::into),
                        source_sel: source_text_sel.clone(),
                        target_sel: target_text_sel.clone(),
                    },
                );
            }
            Annotation::Membrane { color, text, mode, curtains, .. } => {
                if let Some(t) = text {
                    self.doc.text.set(id, t);
                }
                self.set_common(id, None, color.as_deref(), None, None);
                if *mode != crate::types::MembraneMode::default() {
                    self.doc.membrane_mode.set(id, *mode);
                }
                if !curtains.is_empty() {
                    self.doc.curtains.set(id, curtains.clone());
                }
            }
        }
        if !ann.domains().is_empty() {
            self.doc.domains.set(id, ann.domains().to_vec());
        }
        if let Some(t) = temporal_of(ann) {
            self.doc.temporal.set(id, t.clone());
        }
    }

    /// Les quatre attributs que toutes les annotations partagent.
    fn set_common(
        &mut self,
        id: NodeId,
        font_size: Option<f64>,
        color: Option<&str>,
        cursor: Option<usize>,
        source_file: Option<&str>,
    ) {
        if let Some(f) = font_size {
            self.doc.font_size.set(id, f as f32);
        }
        if let Some(c) = color {
            self.doc.set_color(id, c);
        }
        if let Some(c) = cursor {
            self.doc.cursor.set(id, c as u32);
        }
        if let Some(s) = source_file {
            self.doc.source_file.set(id, s.into());
        }
    }

    fn spawn_folder(&mut self, f: &CanvasFolder) -> NodeId {
        let bx = Box2::new(
            Fx::from_f64(f.x),
            Fx::from_f64(f.y),
            Fx::from_f64(f.width),
            Fx::from_f64(f.height),
        );
        let id = self.intern(&f.id, Kind::Folder, bx, Flags::default());
        self.doc.set_color(id, &f.color);
        self.doc
            .folder
            .set(id, (f.name.as_str().into(), f.child_board_id.as_str().into()));
        if let Some(m) = &f.mirror_source {
            self.doc.folder_mirror.set(id, m.clone());
        }
        id
    }

    fn spawn_panel(&mut self, p: &StoryboardPanel) -> NodeId {
        let bx = Box2::new(
            Fx::from_f64(p.x),
            Fx::from_f64(p.y),
            Fx::from_f64(p.width),
            Fx::from_f64(p.height),
        );
        let id = self.intern(&p.id, Kind::Panel, bx, Flags::default());
        self.doc.text.set(id, &p.description);
        self.doc.order.set(id, p.order);
        id
    }

    /// La seconde passe : tout ce qui désigne un autre nœud par son nom.
    fn resolve_references(
        &mut self,
        board: &Board,
        by_name: &ByName<'_>,
        des_images: &[NodeId],
        des_annotations: &[NodeId],
        des_dossiers: &[NodeId],
    ) {
        for (img, &id) in board.images.iter().zip(des_images) {
            self.link(id, img.membrane_id.as_deref(), img.mirror_of.as_deref(), by_name);
        }
        for (ann, &id) in board.annotations.iter().zip(des_annotations) {
            self.link(id, ann.membrane_id(), mirror_of(ann), by_name);
            if let Annotation::Arrow { source_id, target_id, .. } = ann {
                if let Some(a) = self.doc.arrow.get_mut(id) {
                    a.source = super::resolve(by_name, source_id.as_deref());
                    a.target = super::resolve(by_name, target_id.as_deref());
                }
            }
        }
        for (f, &id) in board.folders.iter().zip(des_dossiers) {
            self.link(id, None, f.mirror_of.as_deref(), by_name);
        }
    }

    /// Pose le parent et le miroir d'un nœud.
    fn link(&mut self, id: NodeId, parent: Option<&str>, mirror: Option<&str>, by_name: &ByName<'_>) {
        let p = super::resolve(by_name, parent);
        if p.is_some() {
            self.doc.nodes.set_parent(id, p);
        }
        let m = super::resolve(by_name, mirror);
        if m.is_some() {
            self.doc.mirror_of.set(id, m);
        }
    }
}

/// L'ancre temporelle d'une annotation, quelle que soit sa variante.
fn temporal_of(a: &Annotation) -> Option<&crate::types::TemporalAnchor> {
    match a {
        Annotation::Text { temporal_anchor, .. }
        | Annotation::Sticky { temporal_anchor, .. }
        | Annotation::Arrow { temporal_anchor, .. }
        | Annotation::Membrane { temporal_anchor, .. } => temporal_anchor.as_ref(),
    }
}

/// Le nœud dont une annotation est le miroir, quelle que soit sa variante.
fn mirror_of(a: &Annotation) -> Option<&str> {
    match a {
        Annotation::Text { mirror_of, .. }
        | Annotation::Sticky { mirror_of, .. }
        | Annotation::Arrow { mirror_of, .. }
        | Annotation::Membrane { mirror_of, .. } => mirror_of.as_deref(),
    }
}
