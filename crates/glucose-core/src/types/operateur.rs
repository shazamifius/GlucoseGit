//! **Les opérateurs des post-it** — ET, OU, MAIS, PARCE QUE : un post-it qui relie deux idées
//! devient une pilule (fiche 06 § 5.3). Sortis d'`annotation.rs` quand leurs noms pour le clavier
//! et pour une IA (OPERATEUR-1 et 2, fiche 98) l'ont fait passer les six cents lignes.

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
    /// Les quatre, dans l'ordre de leurs touches (`Alt+1` à `Alt+4`).
    pub const TOUS: [Self; 4] = [Self::And, Self::Or, Self::But, Self::Because];

    /// **Son identifiant**, pour une IA et pour la ligne de commande comme pour le clavier
    /// (`operateur.et`) : un seul nom partout (OPERATEUR-2, fiche 98).
    pub fn id(self) -> &'static str {
        match self {
            Self::And => "et",
            Self::Or => "ou",
            Self::But => "mais",
            Self::Because => "parce-que",
        }
    }

    /// L'opérateur de cet identifiant, s'il en est un.
    pub fn de_l_id(id: &str) -> Option<Self> {
        Self::TOUS.into_iter().find(|o| o.id() == id)
    }

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
