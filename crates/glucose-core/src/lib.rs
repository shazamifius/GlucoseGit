//! Glucose Core Engine — 100% Rust Standard Library (0 dépendance).
//!
//! Contient l'intégralité du modèle de données, de la géométrie, des moteurs
//! d'alignement intelligent, de priorité de sélection, d'espace de membranes,
//! de mode focus, de modèle de rideaux, d'ancres de flèches et de textes,
//! d'index spatial, de graphe de miroirs, de déduplication d'assets, d'export
//! et de **persistance** (le format `.glucose` v2, cf. [`persist`]).

pub mod arrow_anchor;
pub mod hash;
pub mod curtain_model;
pub mod curtain_panel;
pub mod error;
pub mod export;
pub mod geometry;
pub mod hit_priority;
pub mod layout;
pub mod membrane_focus;
pub mod membrane_space;
pub mod membrane_stretch;
pub mod mirror_graph;
pub mod persist;
pub mod quadtree;
pub mod resize;
pub mod smart_align;
pub mod store;
pub mod symbiotic_hue;
pub mod text_anchors;
pub mod timeline;
pub mod types;
