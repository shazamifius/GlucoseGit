//! Le projet : ses tableaux, ses domaines, ses presets, ses zones — et le magasin des actifs.

use super::{Board, Domain, Id};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PresetSlot {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub order: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slots: Vec<PresetSlot>,
    pub is_builtin: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardZone {
    pub slot_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoardZone {
    pub fn new(slot_id: impl Into<String>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            slot_id: slot_id.into(),
            x,
            y,
            width,
            height,
        }
    }
}

/// Magasin d'actifs binaires indépendant du document (Roadmap 1.20, R-04).
/// Les octets bruts des images ne polluent plus jamais les snapshots de la pile d'undo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetStore {
    pub blobs: HashMap<String, Vec<u8>>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            blobs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: impl Into<String>, bytes: Vec<u8>) {
        self.blobs.insert(id.into(), bytes);
    }

    pub fn get(&self, id: &str) -> Option<&[u8]> {
        self.blobs.get(id).map(|v| v.as_slice())
    }

    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }
}

// ── Projet ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub boards: Vec<Board>,
    pub active_board_id: Id,
    pub presets: Vec<Preset>,
    pub domains: Vec<Domain>,
    pub collab_url: Option<String>,
    pub asset_channel_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        let board = Board::new("main", "Canvas Principal");
        let active_id = board.id.clone();
        Self {
            version: "1.0.0".into(),
            name: name.into(),
            boards: vec![board],
            active_board_id: active_id,
            presets: Vec::new(),
            domains: Vec::new(),
            collab_url: None,
            asset_channel_url: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}
