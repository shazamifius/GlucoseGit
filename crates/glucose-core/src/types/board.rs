//! Le tableau : sa caméra, ses nœuds, ses panneaux de storyboard, ses signets.

use super::{Annotation, BoardImage, BoardZone, CanvasFolder, Id};
use std::collections::HashMap;

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
