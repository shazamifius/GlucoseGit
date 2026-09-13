//! Les annotations — cartes, pense-bêtes, flèches, membranes — et ce qui les qualifie :
//! prédicats, opérateurs, modes, rideaux, ancres de texte.

use super::{
    DomainAssignment, Id, TemporalAnchor, DEFAULT_STICKY_HEIGHT, DEFAULT_STICKY_WIDTH,
    DEFAULT_TEXT_CARD_HEIGHT, DEFAULT_TEXT_CARD_WIDTH,
};
use crate::geometry::Rect;

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

/// Hauteur de naissance d'un pense-bête opérateur : une pilule (fiche 06 § 5.3).
pub const DEFAULT_OPERATOR_HEIGHT: f64 = 44.0;

impl StickyOperator {
    /// Le mot que la pilule affiche, tel que la référence l'écrit.
    pub fn label(self) -> &'static str {
        match self {
            Self::And => "ET",
            Self::Or => "OU",
            Self::But => "MAIS",
            Self::Because => "PARCE QUE",
        }
    }

    /// Largeur de naissance de la pilule : « PARCE QUE » est plus long que les trois autres.
    pub fn default_width(self) -> f64 {
        match self {
            Self::Because => 130.0,
            Self::And | Self::Or | Self::But => 80.0,
        }
    }
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
    /// Une carte de texte, avec ses seuls champs obligatoires.
    ///
    /// # Pourquoi ces constructeurs existent
    ///
    /// Une `Annotation::Text` a quatorze champs dont douze sont vides à la création. Sans
    /// constructeur, chaque site qui en crée une recopie les douze — et le projet en compte
    /// plus de cent cinquante. C'est la « répétition structurelle » que la fiche 05 nomme, et
    /// elle a un coût réel : ajouter un champ au modèle oblige à visiter tous ces sites, et la
    /// moindre valeur par défaut oubliée passe inaperçue.
    ///
    /// Les champs facultatifs se posent ensuite, nommément, sur la valeur rendue. Le code neuf
    /// passe par ici ; les sites existants restent à convertir.
    pub fn text(id: impl Into<Id>, x: f64, y: f64, text: impl Into<String>) -> Self {
        Self::Text {
            id: id.into(),
            x,
            y,
            width: None,
            height: None,
            text: text.into(),
            font_size: None,
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// Un pense-bête, avec ses seuls champs obligatoires. Voir [`Annotation::text`].
    pub fn sticky(id: impl Into<Id>, x: f64, y: f64, text: impl Into<String>) -> Self {
        Self::Sticky {
            id: id.into(),
            x,
            y,
            width: None,
            height: None,
            text: text.into(),
            font_size: None,
            color: None,
            bg_color: None,
            cursor_pos: None,
            operator: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// Une flèche d'un point à un autre. Voir [`Annotation::text`].
    pub fn arrow(id: impl Into<Id>, x: f64, y: f64, x2: f64, y2: f64) -> Self {
        Self::Arrow {
            id: id.into(),
            x,
            y,
            x2,
            y2,
            text: None,
            font_size: None,
            color: None,
            arrow_type: None,
            arrow_bidirectional: false,
            predicate: None,
            stroke_width: None,
            waypoints: Vec::new(),
            source_id: None,
            target_id: None,
            source_block_id: None,
            target_block_id: None,
            source_text_sel: None,
            target_text_sel: None,
            long_text: None,
            target_board_id: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// Une membrane classique, sans rideau. Voir [`Annotation::text`].
    pub fn membrane(id: impl Into<Id>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Membrane {
            id: id.into(),
            x,
            y,
            width,
            height,
            color: None,
            text: None,
            mode: MembraneMode::Classic,
            curtains: Vec::new(),
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Text { id, .. }
            | Self::Sticky { id, .. }
            | Self::Arrow { id, .. }
            | Self::Membrane { id, .. } => id,
        }
    }

    /// La taille d'une annotation, tailles de naissance comprises. `None` pour une flèche :
    /// elle a deux extrémités, pas une boîte.
    pub fn size(&self) -> Option<(f64, f64)> {
        match self {
            Self::Text { width, height, .. } => Some((
                width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH),
                height.unwrap_or(DEFAULT_TEXT_CARD_HEIGHT),
            )),
            // Un pense-bête qui porte un opérateur est une pilule, pas un papier : il naît
            // à la taille de son mot.
            Self::Sticky {
                width,
                height,
                operator: Some(op),
                ..
            } => Some((
                width.unwrap_or(op.default_width()),
                height.unwrap_or(DEFAULT_OPERATOR_HEIGHT),
            )),
            Self::Sticky { width, height, .. } => Some((
                width.unwrap_or(DEFAULT_STICKY_WIDTH),
                height.unwrap_or(DEFAULT_STICKY_HEIGHT),
            )),
            Self::Membrane { width, height, .. } => Some((*width, *height)),
            Self::Arrow { .. } => None,
        }
    }

    /// La boîte d'une annotation, tailles de naissance comprises — **la seule** façon d'obtenir
    /// la boîte d'une carte ou d'un pense-bête dont le document ne fixe pas la taille. `None`
    /// pour une flèche.
    pub fn rect(&self) -> Option<Rect> {
        let (width, height) = self.size()?;
        Some(Rect::new(self.x(), self.y(), width, height))
    }

    /// La boîte englobante de l'annotation : sa boîte ([`Annotation::rect`]) — ou, pour une
    /// flèche, l'enveloppe de ses extrémités et de ses points de passage. C'est ce que le
    /// culling et la sélection élastique regardent : tout ce qui a de l'encre est dedans.
    pub fn bounds(&self) -> Rect {
        if let Some(rect) = self.rect() {
            return rect;
        }
        let Self::Arrow {
            x,
            y,
            x2,
            y2,
            waypoints,
            ..
        } = self
        else {
            unreachable!("une annotation sans boîte est une flèche");
        };
        let (mut left, mut top, mut right, mut bottom) =
            (x.min(*x2), y.min(*y2), x.max(*x2), y.max(*y2));
        for wp in waypoints {
            left = left.min(wp.x);
            top = top.min(wp.y);
            right = right.max(wp.x);
            bottom = bottom.max(wp.y);
        }
        Rect::new(left, top, right - left, bottom - top)
    }

    /// Déplace l'annotation de `(dx, dy)` — pour une flèche, ses deux extrémités et ses points
    /// de passage avec elle. C'est **la** translation ; toute autre écriture de `x` et `y` est
    /// une façon d'oublier un point de passage.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Self::Text { x, y, .. } | Self::Sticky { x, y, .. } | Self::Membrane { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
            Self::Arrow {
                x,
                y,
                x2,
                y2,
                waypoints,
                ..
            } => {
                *x += dx;
                *y += dy;
                *x2 += dx;
                *y2 += dy;
                for wp in waypoints {
                    wp.x += dx;
                    wp.y += dy;
                }
            }
        }
    }

    /// Pose l'origine de l'annotation en `(x, y)` sans la déformer — une flèche garde son
    /// vecteur et ses points de passage.
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.translate(x - self.x(), y - self.y());
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
