//! De l'arène vers le modèle historique.
//!
//! Le chemin inverse de `import`, et le seul qui puisse prouver que le premier n'a rien perdu.
//! Il rend les nœuds dans l'ordre où le modèle les rangeait — images, annotations, dossiers,
//! panneaux — parce qu'un aller-retour qui réordonne un document n'est pas un aller-retour.

use super::super::{Flags, Kind, NodeId};
use super::Bridge;
use crate::fixed::Fx;
use crate::types::{
    Annotation, Board, BoardImage, CanvasFolder, MembraneMode, Point2D, StoryboardPanel, Viewport,
};

/// Un texte optionnel : l'arène ne distingue pas le vide de l'absent, et à l'écran ils sont la
/// même chose. Voir la table des normalisations, en tête du module `bridge`.
fn some_if_filled(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

/// La largeur d'un nœud, ou `None` si elle est calculée par la mise en page.
fn declared(value: Fx, auto: bool) -> Option<f64> {
    (!auto).then(|| value.to_f64())
}

impl Bridge {
    /// Reconstruit un tableau du modèle historique, sous l'identifiant et le nom donnés.
    ///
    /// Le viewport, les signets, les zones et les dates ne traversent pas l'arène : ce sont des
    /// propriétés du tableau, pas de ses nœuds. [`Bridge::to_board_like`] les reprend d'un
    /// tableau d'origine ; cette méthode-ci les laisse à leur valeur par défaut.
    pub fn to_board(&self, id: &str, name: &str) -> Board {
        let mut b = Board::new(id, name);
        self.fill(&mut b);
        b
    }

    /// Comme [`Bridge::to_board`], mais en reprenant du tableau `modele` tout ce qui n'est pas
    /// porté par les nœuds : identité, viewport, signets, zones, dates.
    pub fn to_board_like(&self, modele: &Board) -> Board {
        let mut b = Board {
            images: Vec::new(),
            annotations: Vec::new(),
            folders: Vec::new(),
            panels: Vec::new(),
            ..modele.clone()
        };
        self.fill(&mut b);
        b
    }

    /// Les nœuds, répartis dans les quatre listes du modèle.
    fn fill(&self, b: &mut Board) {
        for id in self.doc.nodes.iter_alive() {
            match self.doc.nodes.kind_of(id) {
                Some(Kind::Image) => b.images.push(self.image(id)),
                Some(Kind::Folder) => b.folders.push(self.folder(id)),
                Some(Kind::Panel) => b.panels.push(self.panel(id)),
                Some(_) => b.annotations.push(self.annotation(id)),
                None => {}
            }
        }
    }

    /// Le nom du nœud qu'une table de références désigne, ou `None`.
    fn name_ref(&self, target: NodeId) -> Option<String> {
        target.is_some().then(|| self.name_of(target).to_string())
    }

    /// La membrane parente d'un nœud, par son nom.
    fn parent_name(&self, id: NodeId) -> Option<String> {
        self.doc.nodes.parent_of(id).and_then(|p| self.name_ref(p))
    }

    /// Le nœud dont celui-ci est le miroir, par son nom.
    fn mirror_name(&self, id: NodeId) -> Option<String> {
        self.doc.mirror_of.get(id).and_then(|&m| self.name_ref(m))
    }

    fn image(&self, id: NodeId) -> BoardImage {
        let bx = self.doc.nodes.box_of(id).unwrap_or(super::super::Box2::new(
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
        ));
        let f = self.doc.nodes.flags_of(id).unwrap_or_default();
        let t = self.doc.image.get(id);
        BoardImage {
            id: self.name_of(id).to_string(),
            membrane_id: self.parent_name(id),
            asset: self.doc.asset.get(id).cloned(),
            src: t.and_then(|t| t.src.as_deref()).map(str::to_string),
            x: bx.x.to_f64(),
            y: bx.y.to_f64(),
            width: bx.w.to_f64(),
            height: bx.h.to_f64(),
            rotation: self.doc.rotation.get(id).copied().unwrap_or(0.0) as f64,
            locked: f.has(Flags::LOCKED),
            tags: t
                .map(|t| t.tags.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            slot_id: t.and_then(|t| t.slot_id.as_deref()).map(str::to_string),
            source_url: t.and_then(|t| t.source_url.as_deref()).map(str::to_string),
            original_width: self.doc.original.get(id).map(|o| o.0 as f64).unwrap_or_default(),
            original_height: self.doc.original.get(id).map(|o| o.1 as f64).unwrap_or_default(),
            is_video: f.has(Flags::VIDEO),
            fit: t.and_then(|t| t.fit.as_deref()).map(str::to_string),
            domains: self.doc.domains.get(id).cloned().unwrap_or_default(),
            mirror_of: self.mirror_name(id),
            temporal_anchor: self.doc.temporal.get(id).cloned(),
        }
    }

    fn annotation(&self, id: NodeId) -> Annotation {
        let d = &self.doc;
        let bx = d.nodes.box_of(id).unwrap_or(super::super::Box2::new(
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
        ));
        let f = d.nodes.flags_of(id).unwrap_or_default();
        let (width, height) = (
            declared(bx.w, f.has(Flags::AUTO_WIDTH)),
            declared(bx.h, f.has(Flags::AUTO_HEIGHT)),
        );
        let commun = Commun {
            id: self.name_of(id).to_string(),
            font_size: d.font_size.get(id).map(|&v| v as f64),
            color: d.color_of(id),
            cursor_pos: d.cursor.get(id).map(|&v| v as usize),
            source_file: d.source_file.get(id).map(|s| s.to_string()),
            membrane_id: self.parent_name(id),
            domains: d.domains.get(id).cloned().unwrap_or_default(),
            mirror_of: self.mirror_name(id),
            temporal_anchor: d.temporal.get(id).cloned(),
        };

        match d.nodes.kind_of(id) {
            Some(Kind::Sticky) => Annotation::Sticky {
                id: commun.id,
                x: bx.x.to_f64(),
                y: bx.y.to_f64(),
                width,
                height,
                text: d.text.get(id).to_string(),
                font_size: commun.font_size,
                color: commun.color,
                bg_color: d.background_of(id),
                cursor_pos: commun.cursor_pos,
                operator: d.operator.get(id).copied(),
                source_file: commun.source_file,
                membrane_id: commun.membrane_id,
                domains: commun.domains,
                mirror_of: commun.mirror_of,
                temporal_anchor: commun.temporal_anchor,
            },
            Some(Kind::Arrow) => self.arrow(id, commun),
            Some(Kind::Membrane) => Annotation::Membrane {
                id: commun.id,
                x: bx.x.to_f64(),
                y: bx.y.to_f64(),
                width: bx.w.to_f64(),
                height: bx.h.to_f64(),
                color: commun.color,
                text: some_if_filled(d.text.get(id)),
                mode: d.membrane_mode.get(id).copied().unwrap_or(MembraneMode::Classic),
                curtains: d.curtains.get(id).cloned().unwrap_or_default(),
                membrane_id: commun.membrane_id,
                domains: commun.domains,
                mirror_of: commun.mirror_of,
                temporal_anchor: commun.temporal_anchor,
            },
            _ => Annotation::Text {
                id: commun.id,
                x: bx.x.to_f64(),
                y: bx.y.to_f64(),
                width,
                height,
                text: d.text.get(id).to_string(),
                font_size: commun.font_size,
                color: commun.color,
                cursor_pos: commun.cursor_pos,
                source_file: commun.source_file,
                membrane_id: commun.membrane_id,
                domains: commun.domains,
                mirror_of: commun.mirror_of,
                temporal_anchor: commun.temporal_anchor,
            },
        }
    }

    fn arrow(&self, id: NodeId, commun: Commun) -> Annotation {
        let d = &self.doc;
        let (x, y, x2, y2) = d
            .nodes
            .arrow_endpoints(id)
            .unwrap_or((Fx::ZERO, Fx::ZERO, Fx::ZERO, Fx::ZERO));
        let f = d.nodes.flags_of(id).unwrap_or_default();
        let t = d.arrow.get(id);
        Annotation::Arrow {
            id: commun.id,
            x: x.to_f64(),
            y: y.to_f64(),
            x2: x2.to_f64(),
            y2: y2.to_f64(),
            text: some_if_filled(d.text.get(id)),
            font_size: commun.font_size,
            color: commun.color,
            arrow_type: t.and_then(|t| t.kind.as_deref()).map(str::to_string),
            arrow_bidirectional: f.has(Flags::ARROW_BIDIRECTIONAL),
            predicate: t.and_then(|t| t.predicate),
            stroke_width: t.and_then(|t| t.stroke_width).map(|w| w as f64),
            waypoints: t
                .map(|t| {
                    t.waypoints
                        .iter()
                        .map(|&(px, py)| Point2D { x: px.to_f64(), y: py.to_f64() })
                        .collect()
                })
                .unwrap_or_default(),
            source_id: t.and_then(|t| self.name_ref(t.source)),
            target_id: t.and_then(|t| self.name_ref(t.target)),
            source_block_id: t.and_then(|t| t.source_block.as_deref()).map(str::to_string),
            target_block_id: t.and_then(|t| t.target_block.as_deref()).map(str::to_string),
            source_text_sel: t.and_then(|t| t.source_sel.clone()),
            target_text_sel: t.and_then(|t| t.target_sel.clone()),
            long_text: t.and_then(|t| t.long_text.as_deref()).map(str::to_string),
            target_board_id: t.and_then(|t| t.target_board.as_deref()).map(str::to_string),
            membrane_id: commun.membrane_id,
            domains: commun.domains,
            mirror_of: commun.mirror_of,
            temporal_anchor: commun.temporal_anchor,
        }
    }

    fn folder(&self, id: NodeId) -> CanvasFolder {
        let bx = self.doc.nodes.box_of(id).unwrap_or(super::super::Box2::new(
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
        ));
        let (name, child) = self
            .doc
            .folder
            .get(id)
            .map(|(n, c)| (n.to_string(), c.to_string()))
            .unwrap_or_default();
        CanvasFolder {
            id: self.name_of(id).to_string(),
            name,
            color: self.doc.color_of(id).unwrap_or_default(),
            x: bx.x.to_f64(),
            y: bx.y.to_f64(),
            width: bx.w.to_f64(),
            height: bx.h.to_f64(),
            child_board_id: child,
            mirror_of: self.mirror_name(id),
            mirror_source: self.doc.folder_mirror.get(id).cloned(),
        }
    }

    fn panel(&self, id: NodeId) -> StoryboardPanel {
        let bx = self.doc.nodes.box_of(id).unwrap_or(super::super::Box2::new(
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
            Fx::ZERO,
        ));
        StoryboardPanel {
            id: self.name_of(id).to_string(),
            order: self.doc.order.get(id).copied().unwrap_or(0),
            description: self.doc.text.get(id).to_string(),
            x: bx.x.to_f64(),
            y: bx.y.to_f64(),
            width: bx.w.to_f64(),
            height: bx.h.to_f64(),
        }
    }
}

/// Les champs que les quatre variantes d'annotation partagent, rassemblés une fois plutôt que
/// répétés quatre fois. C'est le tronc commun que le modèle historique n'a jamais eu.
struct Commun {
    id: String,
    font_size: Option<f64>,
    color: Option<String>,
    cursor_pos: Option<usize>,
    source_file: Option<String>,
    membrane_id: Option<String>,
    domains: Vec<crate::types::DomainAssignment>,
    mirror_of: Option<String>,
    temporal_anchor: Option<crate::types::TemporalAnchor>,
}

/// Le viewport ne traverse pas l'arène : il décrit la caméra, pas les nœuds. Rappelé ici pour
/// que la liste des champs non portés soit lisible au même endroit que l'export.
const _: Option<Viewport> = None;

/// Ce que [`Doc`] ne porte pas, et qui reste au tableau : `viewport`, `bookmarks`, `zones`,
/// `created_at`, `updated_at`, `id`, `name`. [`Bridge::to_board_like`] les reprend tels quels.
const _NON_PORTES: &[&str] = &[
    "viewport",
    "bookmarks",
    "zones",
    "created_at",
    "updated_at",
    "id",
    "name",
];
