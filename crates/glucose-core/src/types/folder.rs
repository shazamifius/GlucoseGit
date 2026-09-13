//! Les dossiers-sous-canvas et leur miroir éventuel d'un dossier du disque.

use super::Id;
use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderSortMode {
    NameAsc,
    NameDesc,
    Type,
    SizeDesc,
    SizeAsc,
    ModifiedDesc,
    ModifiedAsc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FolderMirrorSource {
    pub root_path: String,
    pub mode: String, // "snapshot" | "live"
    pub last_scanned_at: i64,
    pub pattern: Option<String>,
    pub recursive: bool,
    pub sort_by: Option<FolderSortMode>,
    pub pending_scan: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasFolder {
    pub id: Id,
    pub name: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub child_board_id: Id,
    pub mirror_of: Option<Id>,
    pub mirror_source: Option<FolderMirrorSource>,
}

impl CanvasFolder {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        child_board_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color: "#888888".into(),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 150.0,
            child_board_id: child_board_id.into(),
            mirror_of: None,
            mirror_source: None,
        }
    }

    /// La boîte du dossier, ancrée en haut à gauche comme les annotations.
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}
