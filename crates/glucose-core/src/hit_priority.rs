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

mod candidates;
mod cycle;
mod handles;

pub use candidates::{collect_candidates, collect_candidates_indexed};
pub use cycle::{advance_on_release, pick_at_down, CycleState, PickOptions};
pub use handles::hit_handle;
