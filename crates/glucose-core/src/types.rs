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
/// Un domaine est un sujet qui **traverse** les nœuds, les membranes et les fichiers (sa définition,
/// `docs/VISION.md` ; fiche 88). Son nom est unique : deux sujets différents portent deux noms
/// (« Python code », « Python serpent »).
#[derive(Debug, Clone, PartialEq)]
pub struct Domain {
    /// Son identité d'un fichier à l'autre : née une fois (`eau@1762500000000`), jamais changée,
    /// même quand on le renomme.
    pub id: Id,
    pub name: String,
    pub color: String,
    pub icon: String,
    /// Quand ce nom, cette couleur et ce sigle ont été écrits, en millisecondes : d'un fichier à
    /// l'autre, la définition la plus récente l'emporte. Le format garde ici la date de création
    /// des anciens domaines, qui se lit de la même façon.
    pub defini_a: i64,
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
mod operateur;
mod project;
mod recadrage;
mod storyboard;

pub use annotation::{
    Annotation, ArrowPredicate, CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain,
    MembraneMode, Point2D, TextAnchor, TextSelection,
};
pub use board::{Board, FolderTreeNode, Viewport};
pub use folder::{CanvasFolder, FolderMirrorSource, FolderSortMode};
pub use image::{AssetRef, BoardImage};
pub use operateur::{StickyOperator, DEFAULT_OPERATOR_HEIGHT};
pub use project::{AssetStore, BoardZone, Preset, PresetSlot, Project};
pub use recadrage::Recadrage;
pub use storyboard::{
    champs_de_naissance, rattacher, Format, Grille, Plan, Storyboard, StoryboardPanel, DESCRIPTION,
    DUREE_DE_NAISSANCE_MS, STORYBOARD_D_AVANT,
};
