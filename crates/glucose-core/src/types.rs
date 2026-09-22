//! Types fondamentaux de Glucose — 100% Rust Standard Library (0 dépendance).

/// Identifiant unique (Nanoid ou chaîne hexadécimale/alphanumérique).
pub type Id = String;

// ── Les tailles de naissance ─────────────────────────────────────────────────
//
// Une carte ou un pense-bête dont le document ne fixe pas la taille en a quand même une :
// celle-ci. Elle vit **ici et nulle part ailleurs**. Douze sites la recopiaient — magnétisme,
// arbitre de clic, index spatial, disposition, export, rendu, sélection élastique, minimap —
// et ils avaient divergé : le magnétisme alignait une carte de 200 × 100 que le rendu dessinait
// en 240 × 48, et la sélection élastique cherchait un pense-bête de 180 × 130 là où il en
// dessinait un de 160 × 120. Une boîte, une source : [`Annotation::rect`].

/// Largeur d'une carte de texte que le document ne dimensionne pas — sa largeur de naissance
/// (fiche 06 § 5.1 : libre jusqu'à 600 px).
pub const DEFAULT_TEXT_CARD_WIDTH: f64 = 240.0;
/// Hauteur d'une carte de texte que le document ne dimensionne pas. Un vestige : TEXT-FIT-1
/// la remplace par la hauteur de son texte dès qu'un texte est mesuré.
pub const DEFAULT_TEXT_CARD_HEIGHT: f64 = 48.0;
/// Largeur d'un pense-bête que le document ne dimensionne pas (fiche 06 § 5.2 : 160 × 120).
pub const DEFAULT_STICKY_WIDTH: f64 = 160.0;
/// Hauteur d'un pense-bête que le document ne dimensionne pas (fiche 06 § 5.2 : 160 × 120).
pub const DEFAULT_STICKY_HEIGHT: f64 = 120.0;

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

mod annotation;
mod board;
mod folder;
mod image;
mod project;
mod recadrage;

pub use annotation::{
    Annotation, ArrowPredicate, CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain,
    MembraneMode, Point2D, StickyOperator, TextAnchor, TextSelection, DEFAULT_OPERATOR_HEIGHT,
};
pub use board::{Board, FolderTreeNode, StoryboardPanel, Viewport};
pub use folder::{CanvasFolder, FolderMirrorSource, FolderSortMode};
pub use image::{AssetRef, BoardImage};
pub use project::{AssetStore, BoardZone, Preset, PresetSlot, Project};
pub use recadrage::Recadrage;
