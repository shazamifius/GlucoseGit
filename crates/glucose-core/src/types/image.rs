//! Les images posées sur le canvas, et la référence vers leurs octets.

use super::{DomainAssignment, Id, Recadrage, TemporalAnchor};
use crate::geometry::Rect;

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
    /// **Ce qu'on retire des bords de cette image** (RECADRAGE-1).
    ///
    /// Non destructif : les octets du fichier ne bougent pas, et deux poses de la meme photo
    /// peuvent etre cadrees differemment. [`Recadrage::ENTIER`] est l'etat de toute image
    /// qu'on n'a jamais cadree -- il n'y a pas d'`Option`, parce que « pas de recadrage » et
    /// « recadrage qui garde tout » sont le meme etat.
    pub crop: Recadrage,
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
            crop: Recadrage::ENTIER,
            fit: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// La boîte de l'image. Une image est ancrée par son **centre** (`x`, `y`) — c'est la
    /// convention du modèle, et ce calcul est le seul endroit qui doit la connaître.
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.x - self.width / 2.0,
            self.y - self.height / 2.0,
            self.width,
            self.height,
        )
    }

    /// **La boîte qui englobe l'image tournée** — celle de [`Self::rect`] quand elle ne l'est
    /// pas. C'est elle que l'index range et qu'une flèche contourne (FLECHE-5) : le coin d'une
    /// photo penchée qui dépasse de sa boîte droite est de la photo aussi. La rotation est celle
    /// de [`crate::rotate::place`], autour du centre.
    pub fn bounds(&self) -> Rect {
        if self.rotation == 0.0 {
            return self.rect();
        }
        let (c, s) = (self.rotation.cos().abs(), self.rotation.sin().abs());
        let (hx, hy) = (
            (self.width * c + self.height * s) / 2.0,
            (self.width * s + self.height * c) / 2.0,
        );
        Rect::new(self.x - hx, self.y - hy, 2.0 * hx, 2.0 * hy)
    }
}
