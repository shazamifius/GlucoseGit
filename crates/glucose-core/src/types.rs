//! Types fondamentaux de Glucose — 100% Rust Standard Library (0 dépendance).

use std::collections::HashMap;

/// Identifiant unique (Nanoid ou chaîne hexadécimale/alphanumérique).
pub type Id = String;

// ── Domaines (Phase 3) ───────────────────────────────────────────────────────
/// Un domaine est une catégorie sémantique (Science, Art, JV, Conlang…).
#[derive(Debug, Clone, PartialEq)]
pub struct Domain {
    pub id: Id,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub created_at: i64,
}

/// Pondération d'un nœud dans un domaine (0.0..1.0).
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAssignment {
    pub domain_id: Id,
    pub weight: f64,
}

// ── Ancrage temporel (Phase 6) ───────────────────────────────────────────────
/// Date du contenu décrit par le nœud (en années entières, négatif pour av. J.-C.).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalAnchor {
    pub start: i64,
    pub end: i64,
    pub label: Option<String>,
}

// ── Assets (R-EMB-01) ────────────────────────────────────────────────────────
/// Référence à un asset binaire (image, vidéo, fichier).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetRef {
    Embed {
        sha256: String,
        mime: String,
        size_bytes: Option<u64>,
    },
    Link {
        href: String,
        sha256: Option<String>,
        size_bytes: Option<u64>,
    },
}

impl AssetRef {
    pub fn is_embed(&self) -> bool {
        matches!(self, AssetRef::Embed { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, AssetRef::Link { .. })
    }
}

// ── Images ──────────────────────────────────────────────────────────────────
/// Une image posée sur le canvas (centre ancré en x, y).
#[derive(Debug, Clone, PartialEq)]
pub struct BoardImage {
    pub id: Id,
    pub membrane_id: Option<Id>,
    pub asset: Option<AssetRef>,
    pub src: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub locked: bool,
    pub tags: Vec<String>,
    pub slot_id: Option<String>,
    pub source_url: Option<String>,
    pub original_width: f64,
    pub original_height: f64,
    pub is_video: bool,
    pub fit: Option<String>,
    pub domains: Vec<DomainAssignment>,
    pub mirror_of: Option<Id>,
    pub temporal_anchor: Option<TemporalAnchor>,
}

impl BoardImage {
    pub fn new(id: impl Into<String>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id: id.into(),
            membrane_id: None,
            asset: None,
            src: None,
            x,
            y,
            width,
            height,
            rotation: 0.0,
            locked: false,
            tags: Vec::new(),
            slot_id: None,
            source_url: None,
            original_width: width,
            original_height: height,
            is_video: false,
            fit: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }
}

// ── Annotations & Prédicats ─────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowPredicate {
    EstPrecurseur,
    Contredit,
    HeriteDe,
    Inspire,
    DependDe,
    Illustre,
}

impl ArrowPredicate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EstPrecurseur => "est_precurseur",
            Self::Contredit => "contredit",
            Self::HeriteDe => "herite_de",
            Self::Inspire => "inspire",
            Self::DependDe => "depend_de",
            Self::Illustre => "illustre",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StickyOperator {
    And,
    Or,
    But,
    Because,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MembraneMode {
    #[default]
    Classic,
    Minimized,
    Stretched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CurtainVisibility {
    #[default]
    Private,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CurtainEditable {
    #[default]
    Owner,
    Everyone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurtainNote {
    pub id: Id,
    pub text: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MembraneCurtain {
    pub id: Id,
    pub owner_id: String,
    pub owner_name: String,
    pub owner_color: String,
    pub visibility: CurtainVisibility,
    pub editable: CurtainEditable,
    pub collapsed_ratio: Option<f64>,
    pub expanded_ratio: Option<f64>,
    pub board_id: Option<Id>,
    pub notes: Vec<CurtainNote>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextAnchor {
    pub start: i64,
    pub end: i64,
    pub quote: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSelection {
    Legacy(String),
    Anchors(Vec<TextAnchor>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// Union discriminée stricte de toutes les annotations de Glucose.
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    Text {
        id: Id,
        x: f64,
        y: f64,
        width: Option<f64>,
        height: Option<f64>,
        text: String,
        font_size: Option<f64>,
        color: Option<String>,
        cursor_pos: Option<usize>,
        source_file: Option<String>,
        membrane_id: Option<Id>,
        domains: Vec<DomainAssignment>,
        mirror_of: Option<Id>,
        temporal_anchor: Option<TemporalAnchor>,
    },
    Sticky {
        id: Id,
        x: f64,
        y: f64,
        width: Option<f64>,
        height: Option<f64>,
        text: String,
        font_size: Option<f64>,
        color: Option<String>,
        bg_color: Option<String>,
        cursor_pos: Option<usize>,
        operator: Option<StickyOperator>,
        source_file: Option<String>,
        membrane_id: Option<Id>,
        domains: Vec<DomainAssignment>,
        mirror_of: Option<Id>,
        temporal_anchor: Option<TemporalAnchor>,
    },
    Arrow {
        id: Id,
        x: f64,
        y: f64,
        x2: f64,
        y2: f64,
        text: Option<String>,
        font_size: Option<f64>,
        color: Option<String>,
        arrow_type: Option<String>,
        arrow_bidirectional: bool,
        predicate: Option<ArrowPredicate>,
        stroke_width: Option<f64>,
        waypoints: Vec<Point2D>,
        source_id: Option<Id>,
        target_id: Option<Id>,
        source_block_id: Option<String>,
        target_block_id: Option<String>,
        source_text_sel: Option<TextSelection>,
        target_text_sel: Option<TextSelection>,
        long_text: Option<String>,
        target_board_id: Option<Id>,
        membrane_id: Option<Id>,
        domains: Vec<DomainAssignment>,
        mirror_of: Option<Id>,
        temporal_anchor: Option<TemporalAnchor>,
    },
    Membrane {
        id: Id,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: Option<String>,
        text: Option<String>,
        mode: MembraneMode,
        curtains: Vec<MembraneCurtain>,
        membrane_id: Option<Id>,
        domains: Vec<DomainAssignment>,
        mirror_of: Option<Id>,
        temporal_anchor: Option<TemporalAnchor>,
    },
}

impl Annotation {
    pub fn id(&self) -> &str {
        match self {
            Self::Text { id, .. }
            | Self::Sticky { id, .. }
            | Self::Arrow { id, .. }
            | Self::Membrane { id, .. } => id,
        }
    }

    pub fn x(&self) -> f64 {
        match self {
            Self::Text { x, .. }
            | Self::Sticky { x, .. }
            | Self::Arrow { x, .. }
            | Self::Membrane { x, .. } => *x,
        }
    }

    pub fn y(&self) -> f64 {
        match self {
            Self::Text { y, .. }
            | Self::Sticky { y, .. }
            | Self::Arrow { y, .. }
            | Self::Membrane { y, .. } => *y,
        }
    }

    pub fn membrane_id(&self) -> Option<&str> {
        match self {
            Self::Text { membrane_id, .. }
            | Self::Sticky { membrane_id, .. }
            | Self::Arrow { membrane_id, .. }
            | Self::Membrane { membrane_id, .. } => membrane_id.as_deref(),
        }
    }

    pub fn set_membrane_id(&mut self, mid: Option<Id>) {
        match self {
            Self::Text { membrane_id, .. }
            | Self::Sticky { membrane_id, .. }
            | Self::Arrow { membrane_id, .. }
            | Self::Membrane { membrane_id, .. } => *membrane_id = mid,
        }
    }

    pub fn domains_mut(&mut self) -> &mut Vec<DomainAssignment> {
        match self {
            Self::Text { domains, .. }
            | Self::Sticky { domains, .. }
            | Self::Arrow { domains, .. }
            | Self::Membrane { domains, .. } => domains,
        }
    }
}

// ── Dossiers (Canvas Folders) ───────────────────────────────────────────────
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
    pub fn new(id: impl Into<String>, name: impl Into<String>, child_board_id: impl Into<String>) -> Self {
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
}

// ── Viewport & Board ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardPanel {
    pub id: Id,
    pub order: i32,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl StoryboardPanel {
    pub fn new(id: impl Into<String>, order: i32, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id: id.into(),
            order,
            description: String::new(),
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FolderTreeNode {
    pub folder: CanvasFolder,
    pub annotations: Vec<Annotation>,
    pub images: Vec<BoardImage>,
    pub children: Vec<FolderTreeNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub id: Id,
    pub name: String,
    pub images: Vec<BoardImage>,
    pub annotations: Vec<Annotation>,
    pub folders: Vec<CanvasFolder>,
    pub panels: Vec<StoryboardPanel>,
    pub zones: Vec<BoardZone>,
    pub viewport: Viewport,
    pub bookmarks: HashMap<String, Viewport>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Board {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            images: Vec::new(),
            annotations: Vec::new(),
            folders: Vec::new(),
            panels: Vec::new(),
            zones: Vec::new(),
            viewport: Viewport::default(),
            bookmarks: HashMap::new(),
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new("default", "Default Board")
    }
}


// ── Presets & Zones ─────────────────────────────────────────────────────────
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

// ── Projet ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub boards: Vec<Board>,
    pub active_board_id: Id,
    pub presets: Vec<Preset>,
    pub domains: Vec<Domain>,
    pub blobs: HashMap<String, Vec<u8>>,
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
            blobs: HashMap::new(),
            collab_url: None,
            asset_channel_url: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}
