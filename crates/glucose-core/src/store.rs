//! State Store, Machine d'État et pile Undo/Redo infinie — 0 dépendance.
//!
//! Architecture Pure Rust std avec Invariants UNDO-1 :
//! - Navigation transparente (pan, zoom, active board, dossier ne polluent jamais l'undo)
//! - Caméra préservée à travers undo/redo (pas de téléportation)
//! - Transactions live atomiques (begin_live_edit / end_live_edit)
//! - Cascade de suppression miroirs & flèches orphelines
//!
//! # Organisation (R-41 : 500 lignes par fichier)
//!
//! Le `Store` est une seule structure, mais ses méthodes sont réparties par domaine dans des
//! sous-modules. Ce sont des blocs `impl Store` : **l'API publique est strictement identique**
//! à celle du fichier unique d'origine, aucun appelant n'a à changer d'import.
//!
//! | Module | Responsabilité |
//! |---|---|
//! | [`ids`] | génération d'identifiants (R-22) et chargement de projet |
//! | [`navigation`] | board actif, viewport, pan/zoom, dossiers (jamais d'undo) |
//! | [`selection`] | sélection d'images, d'annotations, de dossiers |
//! | [`undo`] | pile undo/redo, transactions live |
//! | [`images`] | mutations d'images, déplacement, duplication, suppression |
//! | [`annotations`] | mutations d'annotations, miroirs, panneaux storyboard |
//! | [`boards`] | création, renommage et suppression de tableaux |
//! | [`folders`] | dossiers de canevas et leurs boards enfants |
//! | [`catalog`] | presets, zones de tableau, métadonnées de projet |
//! | [`domains`] | catalogue de domaines et pondérations des nœuds (DOM-1..DOM-3) |
//! | [`resize`] | la boîte d'un nœud, posée par le geste de redimensionnement (RESIZE-1) |
//!
//! # R-35 — Les erreurs ne sont plus silencieuses
//!
//! Les opérations qui peuvent échouer exposent une variante `try_*` rendant un
//! [`crate::error::CoreResult`]. Les méthodes historiques sont conservées comme enveloppes
//! silencieuses et marquées `#[deprecated]` : le chemin de migration est explicite et aucun
//! appelant existant ne casse.

mod annotations;
mod boards;
mod catalog;
mod domains;
mod folders;
mod ids;
mod images;
pub mod journal;
mod navigation;
mod resize;
mod selection;
mod undo;

pub use domains::DomainPatch;
pub use navigation::build_folder_stack;

use crate::types::{AssetStore, Project, TemporalAnchor};

#[derive(Debug, Clone)]
pub struct Store {
    pub project: Project,
    pub assets: AssetStore,
    pub selected_image_ids: Vec<String>,
    pub selected_annotation_ids: Vec<String>,
    pub selected_folder_id: Option<String>,
    pub folder_stack: Vec<(String, String)>,
    pub temporal_filter: Option<TemporalAnchor>,

    /// Pile d'annulation. Voir [`journal`] pour la loi de coût JRN-1.
    pub journal: journal::Journal,
    /// Compteur monotone d'identifiants.
    ///
    /// INVARIANT ID-1 — il est toujours >= au plus grand suffixe numérique présent dans le
    /// projet. C'est ce qui rend `generate_id` O(1) : il n'a plus à vérifier l'absence de
    /// collision (R-22). Le seul point où l'invariant peut être rompu est l'injection d'un
    /// projet venu du disque : [`Store::load_project`] le rétablit par un unique scan.
    pub next_id: u64,
    pub version: u64,
}

impl Store {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project: Project::new(project_name),
            assets: AssetStore::new(),
            selected_image_ids: Vec::new(),
            selected_annotation_ids: Vec::new(),
            selected_folder_id: None,
            folder_stack: Vec::new(),
            temporal_filter: None,
            journal: journal::Journal::new(200),
            next_id: 1,
            version: 1,
        }
    }

    pub fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}
