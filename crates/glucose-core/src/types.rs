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

    /// Les pondérations de domaine portées par cette annotation.
    ///
    /// Contrepartie en lecture de [`Annotation::domains_mut`]. Les deux ne devraient pas
    /// exister : `domains` est recopié dans les quatre variantes de l'énumération, ce que le
    /// standard § 2.1 interdit (« un champ présent dans plusieurs variantes remonte dans le
    /// tronc commun »). Tant que le modèle n'est pas composé, ces deux accesseurs sont le seul
    /// moyen de ne pas répéter un `match` à quatre bras à chaque lecture.
    pub fn domains(&self) -> &[DomainAssignment] {
        match self {
            Self::Text { domains, .. }
            | Self::Sticky { domains, .. }
            | Self::Arrow { domains, .. }
            | Self::Membrane { domains, .. } => domains,
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

impl Viewport {
    /// Échelle la plus petite que le **modèle** accepte : ×200 dézoomé (fiche 09 § 1).
    ///
    /// C'est la borne anti-crash du document, pas celle du geste : la molette s'arrête bien
    /// avant (fiche 07 § 7.1), mais un signet, un cadrage calculé ou un fichier venu de
    /// Glucose Tauri peuvent porter une échelle plus large, et doivent rester valides.
    pub const MIN_SCALE: f64 = 0.005;
    /// Échelle la plus grande que le modèle accepte : ×50 zoomé (fiche 09 § 1).
    pub const MAX_SCALE: f64 = 50.0;
    /// Les deux bornes du modèle, sous la forme qu'attend [`Viewport::zoom_at`].
    pub const SCALE_RANGE: (f64, f64) = (Self::MIN_SCALE, Self::MAX_SCALE);

    /// Le viewport ramené dans le domaine du modèle.
    ///
    /// Une échelle hors bornes est rabattue ; une échelle qui n'est pas un nombre — `NaN`,
    /// infini, ce qu'un fichier abîmé peut porter — devient 1, parce que `clamp` laisse
    /// passer `NaN` et qu'un `NaN` dans la caméra rend chaque pixel du canevas invisible
    /// sans faire tomber le programme. Un décalage non fini devient 0 pour la même raison.
    pub fn normalized(self) -> Self {
        let finite_or = |v: f64, fallback: f64| if v.is_finite() { v } else { fallback };
        Self {
            x: finite_or(self.x, 0.0),
            y: finite_or(self.y, 0.0),
            scale: finite_or(self.scale, 1.0).clamp(Self::MIN_SCALE, Self::MAX_SCALE),
        }
    }

    /// Zoom ancré : le point du monde sous `(cx, cy)` (pixels écran) ne bouge pas à l'écran
    /// (fiche 07 § 7.1).
    ///
    /// ```text
    ///     s' = clamp(s × facteur, lo, hi)
    ///     v' = c − (c − v) × s' / s
    /// ```
    ///
    /// `range` est la borne de l'appelant — celle du geste, plus étroite que celle du
    /// modèle — et le résultat respecte **les deux** : l'échelle finale est dans
    /// l'intersection de `range` et de [`SCALE_RANGE`](Self::SCALE_RANGE). C'est la seule
    /// écriture de cette formule dans le programme ; le noyau et l'interface l'appellent.
    pub fn zoom_at(&mut self, factor: f64, cx: f64, cy: f64, range: (f64, f64)) {
        let lo = range.0.max(Self::MIN_SCALE);
        let hi = range.1.min(Self::MAX_SCALE);
        // Partir d'un viewport valide : la formule divise par l'échelle courante.
        let base = self.normalized();
        let new_scale = (base.scale * factor).clamp(lo, hi);
        self.x = cx - (cx - base.x) * (new_scale / base.scale);
        self.y = cy - (cy - base.y) * (new_scale / base.scale);
        self.scale = new_scale;
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

#[cfg(test)]
mod viewport_tests {
    use super::Viewport;

    fn screen_to_world(sx: f64, sy: f64, vp: &Viewport) -> (f64, f64) {
        ((sx - vp.x) / vp.scale, (sy - vp.y) / vp.scale)
    }

    /// Fiche 07 § 7.1 — le point du monde sous le curseur est rigoureusement immobile.
    /// Ce test vivait dans `glucose-desktop/canvas.rs`, à côté d'une seconde copie de la
    /// formule ; il suit la formule, désormais unique.
    #[test]
    fn test_zoom_at_keeps_the_world_point_under_the_cursor_still() {
        for factor in [1.5, 0.25, 4.0, 1.0] {
            let mut vp = Viewport { x: 50.0, y: 50.0, scale: 1.0 };
            let (cx, cy) = (300.0, 200.0);
            let before = screen_to_world(cx, cy, &vp);
            vp.zoom_at(factor, cx, cy, Viewport::SCALE_RANGE);
            let after = screen_to_world(cx, cy, &vp);
            assert!((before.0 - after.0).abs() < 1e-9, "facteur {factor}: {before:?} → {after:?}");
            assert!((before.1 - after.1).abs() < 1e-9, "facteur {factor}: {before:?} → {after:?}");
            assert_eq!(vp.scale, factor);
        }
    }
}
