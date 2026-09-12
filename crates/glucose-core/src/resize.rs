//! RESIZE-1 — géométrie pure du redimensionnement d'un nœud par ses poignées.
//!
//! C'est le geste central de PureRef : on tire l'une des huit poignées d'un nœud sélectionné,
//! et **le côté ou le coin opposé ne bouge pas**. Tout ce qui se calcule sans écran vit ici —
//! l'ancrage, la taille minimale, le rapport d'aspect, la réconciliation avec le magnétisme.
//! Le branchement (souris, curseur, undo, document) est dans
//! `glucose-desktop::interactions::resize`.
//!
//! # Les trois règles du geste
//!
//! - **RESIZE-1 — l'ancre est le côté opposé.** Tirer la poignée droite garde le bord gauche
//!   fixe ; tirer un coin garde le coin opposé fixe. C'est ce que tout utilisateur attend,
//!   l'inverse est un bug.
//! - **RESIZE-2 — une taille minimale par type, et la poignée ne saute jamais.** Passer sous
//!   le minimum bloque la dimension à sa borne, du côté de l'ancre : le rectangle ne se
//!   retourne pas et ne devient jamais négatif.
//! - **RESIZE-3 — le rapport d'aspect se conserve sur les coins, pas sur les côtés.** Quand
//!   la règle le demande (une image, sans `Shift`), un coin glisse le long de la diagonale
//!   qui passe par l'ancre ; les côtés, eux, ne changent qu'une dimension.

use crate::smart_align::{snap_resize, AlignRect, AlignTarget, SnapGuides, SnapOptions};

/// Les huit poignées d'un rectangle, nommées par le côté qu'elles déplacent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl Handle {
    /// Les huit poignées, dans l'ordre du cadran.
    pub const ALL: [Handle; 8] = [
        Handle::TopLeft,
        Handle::Top,
        Handle::TopRight,
        Handle::Right,
        Handle::BottomRight,
        Handle::Bottom,
        Handle::BottomLeft,
        Handle::Left,
    ];

    /// Les poignées qui changent la largeur : celles d'un nœud dont la hauteur ne se tire
    /// pas (une carte de texte, dont la hauteur suit le texte — TEXT-FIT-1).
    pub const HORIZONTAL: [Handle; 6] = [
        Handle::TopLeft,
        Handle::TopRight,
        Handle::Right,
        Handle::BottomRight,
        Handle::BottomLeft,
        Handle::Left,
    ];

    /// Nom court, celui que `hit_priority` range dans `PickCandidate::corner` et que
    /// `smart_align::snap_resize` lit lettre à lettre.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "tl",
            Self::Top => "t",
            Self::TopRight => "tr",
            Self::Right => "r",
            Self::BottomRight => "br",
            Self::Bottom => "b",
            Self::BottomLeft => "bl",
            Self::Left => "l",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|h| h.as_str() == name)
    }

    pub fn moves_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::Left | Self::BottomLeft)
    }

    pub fn moves_right(self) -> bool {
        matches!(self, Self::TopRight | Self::Right | Self::BottomRight)
    }

    pub fn moves_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::Top | Self::TopRight)
    }

    pub fn moves_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::Bottom | Self::BottomRight)
    }

    pub fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    /// Où la poignée se pose sur `rect` : un coin, ou le milieu d'un côté.
    pub fn position_on(self, rect: AlignRect) -> (f64, f64) {
        let x = if self.moves_left() {
            rect.left
        } else if self.moves_right() {
            rect.left + rect.width
        } else {
            rect.left + rect.width / 2.0
        };
        let y = if self.moves_top() {
            rect.top
        } else if self.moves_bottom() {
            rect.top + rect.height
        } else {
            rect.top + rect.height / 2.0
        };
        (x, y)
    }

    /// Nom CSS du curseur qui annonce le geste : `↔`, `↕`, `⤡`, `⤢`.
    pub fn cursor(self) -> &'static str {
        match self {
            Self::TopLeft | Self::BottomRight => "nwse-resize",
            Self::TopRight | Self::BottomLeft => "nesw-resize",
            Self::Top | Self::Bottom => "ns-resize",
            Self::Left | Self::Right => "ew-resize",
        }
    }
}

// ── Tailles minimales, par type (RESIZE-2) ──────────────────────────────────

/// Côté minimal d'une image : au-dessous, l'image n'est plus qu'un point que l'on ne
/// peut plus rattraper à la souris.
pub const MIN_IMAGE_SIDE: f64 = 16.0;
/// Largeur minimale d'une carte de texte : ses deux marges de 18 et la place d'un mot.
pub const MIN_TEXT_CARD_WIDTH: f64 = 80.0;
/// Côté minimal d'un pense-bête : sa marge de 10 de chaque côté et une ligne de texte.
pub const MIN_STICKY_SIDE: f64 = 48.0;
/// Côté minimal d'une membrane : le double de son rayon de coin (60), pour qu'elle reste
/// une membrane et non un disque.
pub const MIN_MEMBRANE_SIDE: f64 = 120.0;
/// Dimensions minimales d'un dossier : son en-tête (38) plus une rangée.
pub const MIN_FOLDER_WIDTH: f64 = 120.0;
pub const MIN_FOLDER_HEIGHT: f64 = 80.0;

/// Ce qu'un nœud impose au geste.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeRule {
    pub min_width: f64,
    pub min_height: f64,
    /// Conserver le rapport largeur/hauteur de départ sur les coins (RESIZE-3).
    pub keep_aspect: bool,
}

impl ResizeRule {
    /// Une image conserve son rapport par défaut : c'est PureRef. `Shift` le libère.
    pub fn image(shift: bool) -> Self {
        Self {
            min_width: MIN_IMAGE_SIDE,
            min_height: MIN_IMAGE_SIDE,
            keep_aspect: !shift,
        }
    }

    /// Une carte de texte se tire en largeur ; sa hauteur suit son texte (TEXT-FIT-1), donc
    /// aucun minimum vertical n'a de sens ici.
    pub fn text_card() -> Self {
        Self {
            min_width: MIN_TEXT_CARD_WIDTH,
            min_height: 0.0,
            keep_aspect: false,
        }
    }

    pub fn sticky() -> Self {
        Self {
            min_width: MIN_STICKY_SIDE,
            min_height: MIN_STICKY_SIDE,
            keep_aspect: false,
        }
    }

    pub fn membrane() -> Self {
        Self {
            min_width: MIN_MEMBRANE_SIDE,
            min_height: MIN_MEMBRANE_SIDE,
            keep_aspect: false,
        }
    }

    pub fn folder() -> Self {
        Self {
            min_width: MIN_FOLDER_WIDTH,
            min_height: MIN_FOLDER_HEIGHT,
            keep_aspect: false,
        }
    }
}

// ── Le calcul ───────────────────────────────────────────────────────────────

/// Le rectangle obtenu en tirant `handle` de `delta` (unités monde) depuis `start`.
pub fn resize_rect(
    start: AlignRect,
    handle: Handle,
    delta: (f64, f64),
    rule: ResizeRule,
) -> AlignRect {
    if rule.keep_aspect && handle.is_corner() && start.width > 0.0 && start.height > 0.0 {
        resize_corner_proportional(start, handle, delta, rule)
    } else {
        resize_free(start, handle, delta, rule)
    }
}

/// RESIZE-1 et RESIZE-2 : chaque bord tiré suit le pointeur, l'autre reste, et la dimension
/// se bloque à son minimum **du côté de l'ancre**.
fn resize_free(
    start: AlignRect,
    handle: Handle,
    (dx, dy): (f64, f64),
    rule: ResizeRule,
) -> AlignRect {
    let min_w = rule.min_width.max(0.0);
    let min_h = rule.min_height.max(0.0);
    let (mut left, mut right) = (start.left, start.left + start.width);
    let (mut top, mut bottom) = (start.top, start.top + start.height);

    if handle.moves_left() {
        left = (start.left + dx).min(right - min_w);
    } else if handle.moves_right() {
        right = (right + dx).max(left + min_w);
    }
    if handle.moves_top() {
        top = (start.top + dy).min(bottom - min_h);
    } else if handle.moves_bottom() {
        bottom = (bottom + dy).max(top + min_h);
    }
    AlignRect::new(left, top, right - left, bottom - top)
}

/// RESIZE-3 : le coin tiré glisse le long de la diagonale qui passe par l'ancre. On projette
/// le pointeur sur cette diagonale, ce qui est continu — pas de saut quand le pointeur passe
/// d'un côté à l'autre de la diagonale — et rend un facteur unique appliqué aux deux côtés.
fn resize_corner_proportional(
    start: AlignRect,
    handle: Handle,
    (dx, dy): (f64, f64),
    rule: ResizeRule,
) -> AlignRect {
    let (ax, ay) = anchor_of(start, handle);
    let sx = if handle.moves_left() { -1.0 } else { 1.0 };
    let sy = if handle.moves_top() { -1.0 } else { 1.0 };
    // Diagonale orientée de l'ancre vers le coin tiré, et position du coin tiré.
    let (diag_x, diag_y) = (start.width * sx, start.height * sy);
    let (px, py) = (ax + diag_x + dx, ay + diag_y + dy);
    let factor = ((px - ax) * diag_x + (py - ay) * diag_y) / (diag_x * diag_x + diag_y * diag_y);
    let floor = (rule.min_width / start.width)
        .max(rule.min_height / start.height)
        .max(0.0);
    let factor = factor.max(floor);
    let (width, height) = (start.width * factor, start.height * factor);
    place_from_anchor((ax, ay), handle, (width, height))
}

/// Le point qui ne bouge pas : le coin ou le côté opposé à la poignée.
fn anchor_of(start: AlignRect, handle: Handle) -> (f64, f64) {
    let x = if handle.moves_left() {
        start.left + start.width
    } else {
        start.left
    };
    let y = if handle.moves_top() {
        start.top + start.height
    } else {
        start.top
    };
    (x, y)
}

/// Pose un rectangle de `size` en gardant `anchor` immobile.
fn place_from_anchor(
    (ax, ay): (f64, f64),
    handle: Handle,
    (width, height): (f64, f64),
) -> AlignRect {
    let left = if handle.moves_left() { ax - width } else { ax };
    let top = if handle.moves_top() { ay - height } else { ay };
    AlignRect::new(left, top, width, height)
}

// ── Magnétisme ──────────────────────────────────────────────────────────────

/// Le rectangle magnétisé d'un geste, et les guides à dessiner.
#[derive(Debug, Clone, PartialEq)]
pub struct SnappedResize {
    pub rect: AlignRect,
    pub guides: SnapGuides,
}

/// Aimante les bords tirés de `free` aux cibles, sans trahir la règle du geste.
///
/// `snap_resize` sait aimanter un bord ; il ne sait pas qu'un coin à rapport conservé ne
/// peut suivre qu'**un** guide à la fois. Ici, quand le rapport est tenu, l'axe X gagne
/// (choix arbitraire mais fixe), l'autre dimension se redérive du rapport de départ, et le
/// guide est abandonné s'il ferait passer le nœud sous sa taille minimale.
pub fn snap_resized_rect(
    start: AlignRect,
    handle: Handle,
    free: AlignRect,
    targets: &[AlignTarget],
    opts: SnapOptions,
    rule: ResizeRule,
) -> SnappedResize {
    let snapped = snap_resize(
        free,
        handle.as_str(),
        targets,
        opts,
        rule.min_width,
        rule.min_height,
    );
    if !(rule.keep_aspect && handle.is_corner()) || start.width <= 0.0 || start.height <= 0.0 {
        return SnappedResize {
            rect: snapped.rect,
            guides: snapped.guides,
        };
    }

    let ratio = start.height / start.width;
    let anchor = anchor_of(start, handle);
    if snapped.guides.x.is_some() {
        let width = snapped.rect.width;
        let height = width * ratio;
        if height >= rule.min_height {
            let rect = place_from_anchor(anchor, handle, (width, height));
            return SnappedResize {
                rect,
                guides: SnapGuides {
                    x: snapped.guides.x,
                    y: None,
                },
            };
        }
    }
    if snapped.guides.y.is_some() {
        let height = snapped.rect.height;
        let width = height / ratio;
        if width >= rule.min_width {
            let rect = place_from_anchor(anchor, handle, (width, height));
            return SnappedResize {
                rect,
                guides: SnapGuides {
                    x: None,
                    y: snapped.guides.y,
                },
            };
        }
    }
    SnappedResize {
        rect: free,
        guides: SnapGuides::default(),
    }
}
