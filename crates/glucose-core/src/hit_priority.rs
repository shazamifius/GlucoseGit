//! PICK-1 — Arbitre de sélection au clic et priorité des cibles.
//! 100% Rust Standard Library (0 dépendance).

use crate::geometry::{in_rect, in_rotated_box, on_rect_edge};
use crate::types::{Annotation, BoardImage, CanvasFolder};

pub const PICK_RANK_HANDLE: i32 = 0;
pub const PICK_RANK_MEMBRANE_EDGE: i32 = 10;
pub const PICK_RANK_FOLDER_EDGE: i32 = 10;
pub const PICK_RANK_ARROW: i32 = 20;
// **PICK-2 — les contenus dans l'ordre où ils sont peints** (registre de Tauri, n° 8).
//
// Tauri classait les contenus par « intention » — l'image avant la note, la note avant le
// texte — et mettait le texte toujours dernier, pour que le cycle de profondeur s'y arrête et
// laisse le double-clic d'édition intact. Sa plainte : *« une image SOUS un texte : même la
// souris sur le texte sélectionne l'image — c'est faux »*, et le cycle ne passait plus rien
// dessous. Ce qu'il demande, c'est **ce qu'on voit sous la souris** : les affordances fines
// d'abord (poignée, bord d'un conteneur, trait d'une flèche), puis le contenu peint au-dessus
// à cet endroit, puis le corps des conteneurs. Glucose peint les photos, puis les cartes, puis
// les pense-bêtes par-dessus : c'est cet ordre-là.
//
// Et le texte n'a plus à être le fond de la pile : ici, l'édition s'ouvre par un vrai
// double-clic (deux clics en moins de `DBLCLICK_MS`), et le cycle ne descend qu'au relâchement
// d'un re-clic plus lent que cette fenêtre. Le temps sépare les deux gestes.
pub const PICK_RANK_STICKY: i32 = 30;
pub const PICK_RANK_TEXT: i32 = 40;
pub const PICK_RANK_IMAGE: i32 = 50;
pub const PICK_RANK_MEMBRANE_BODY: i32 = 60;
pub const PICK_RANK_FOLDER_BODY: i32 = 60;

pub mod pick_consts {
    pub const HANDLE_SLOP_PX: f64 = 24.0;
    pub const HANDLE_SLOP_MAX_RATIO: f64 = 0.35;
    /// Côté du carré d'une poignée sur un nœud qui a toute la place (fiche 06 § 4.2 : 9 px).
    pub const HANDLE_SIDE_PX: f64 = 9.0;
    /// Le plus petit carré qui soit encore une poignée : un pixel de fond blanc entre deux
    /// pixels de liseré. En dessous, il ne reste qu'une tache.
    pub const HANDLE_SMALLEST_SQUARE_PX: f64 = 3.0;
    pub const EDGE_BAND_PX: f64 = 14.0;
    pub const FOLDER_HEADER: f64 = 38.0;
    pub const FOLDER_HANDLE_INSET: f64 = 8.0;
    pub const MEMBRANE_LABEL_BAND: f64 = 30.0;
    pub const CYCLE_RADIUS_PX: f64 = 8.0;
    pub const CYCLE_TTL_MS: i64 = 2500;
    /// Fenêtre du double-clic (fiche 07 § 1, `DOUBLE_CLICK_WINDOW`). Une seule notion, une
    /// seule constante : en deçà, deux clics sont un double-clic (édition, dossier) ; au-delà,
    /// un re-clic qui avance le cycle de profondeur. La référence en avait deux — 350 pour
    /// l'une, 400 pour l'autre — avec 50 ms entre les deux où un clic n'était ni l'un ni
    /// l'autre.
    pub const DBLCLICK_MS: i64 = 350;
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
}

/// **POIGNEE-1 — la prise d'une poignée**, en pixels logiques, pour un nœud dont le petit côté
/// mesure `petit_cote` pixels logiques à l'écran ; `None` quand il est trop petit pour en porter.
///
/// La prise vaut 24 px, sans dépasser 35 % du petit côté : quatre prises pleines couvriraient
/// un petit nœud, et on ne pourrait plus le déplacer (fiche 08 § 3.2).
///
/// Le carré **dessiné** est le cœur visible de cette prise, toujours dans la proportion 9 : 24
/// ([`handle_side_px`]) : quand le nœud rapetisse à l'écran, prise et carré rapetissent
/// ensemble, et ce qui se voit est ce qui s'attrape. Ils gardaient 9 px et 6 px de plancher
/// quel que soit le zoom : au dézoom, huit carrés blancs plus gros que la photo qu'ils
/// entouraient (son retour du 07/10), et un nœud minuscule tout entier couvert de prises, qu'on
/// redimensionnait en voulant le déplacer. Quand le carré n'a plus la place d'être un carré, la
/// poignée disparaît — du dessin et du clic à la fois ; le cadre de la sélection reste.
pub fn handle_reach_px(petit_cote: f64) -> Option<f64> {
    use pick_consts::*;
    let prise = HANDLE_SLOP_PX.min(petit_cote.abs() * HANDLE_SLOP_MAX_RATIO);
    (prise * HANDLE_SIDE_PX / HANDLE_SLOP_PX >= HANDLE_SMALLEST_SQUARE_PX).then_some(prise)
}

/// Le côté du carré d'une poignée, en pixels logiques, sur un nœud dont le petit côté mesure
/// `petit_cote` pixels logiques à l'écran ; `None` quand il n'en porte pas (POIGNEE-1).
pub fn handle_side_px(petit_cote: f64) -> Option<f64> {
    use pick_consts::*;
    handle_reach_px(petit_cote).map(|prise| prise * HANDLE_SIDE_PX / HANDLE_SLOP_PX)
}

/// La prise d'une poignée en unités du monde, sur une boîte du monde vue à `scale` ; `None`
/// quand la boîte est trop petite à l'écran pour porter des poignées (POIGNEE-1).
pub fn handle_slop_world(scale: f64, box_w: f64, box_h: f64) -> Option<f64> {
    let s = scale.max(1e-6);
    handle_reach_px(box_w.abs().min(box_h.abs()) * s).map(|prise| prise / s)
}

/// Nom CSS du curseur d'une poignee (`PickCandidate::corner`), `default` si le nom est inconnu.
pub fn handle_cursor(corner: &str) -> &'static str {
    crate::resize::Handle::parse(corner).map_or("default", |h| h.cursor())
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
    /// **Ce qui mesure les nœuds pour les flèches** (FLECHE-4) : la boîte d'un nœud et la
    /// hauteur d'un passage de texte. Sans lui, une flèche ancrée à un passage se vise depuis
    /// le milieu de la carte ; avec lui, là où le dessin la pose — le même interlocuteur pour
    /// les deux (loi L4).
    pub noeuds: Option<&'a dyn crate::arrow::Noeuds>,
}

mod candidates;
mod cycle;
mod handles;

pub use candidates::{collect_candidates, collect_candidates_indexed};
pub use cycle::{advance_on_release, pick_at_down, CycleState, PickOptions};
pub use handles::hit_handle;
