//! **TITRE-1 — le nom d'une région à la taille de la région** (fiche 95).
//!
//! La fiche 94 (principe 2) : pour qu'une vue d'ensemble se lise comme une carte, le titre d'une
//! membrane grandit avec elle — une carte écrit un pays plus gros qu'une ville. Sa parole du
//! 8/11/2026 : *« pas de système de LOD, et pas de simplification possible »* — **aucun titre ne se
//! cache**, à aucune distance : il est dessiné à sa taille, et c'est l'œil qui le lit ou non.
//!
//! # La loi : la même part de surface qu'une membrane neuve
//!
//! Une membrane posée d'un clic ([`NAISSANCE`], 320 × 240) porte son titre au corps [`CORPS`], à
//! [`RETRAIT`] de son bord gauche et [`ELEVATION`] au-dessus de son bord haut — les mesures de
//! Glucose Tauri. Une membrane de surface `A` consacre à son nom **la même part de sa surface** :
//! chacune de ces mesures est multipliée par
//!
//! ```text
//!     s = √(A / A₀),     A₀ = 320 × 240
//! ```
//!
//! La surface d'un titre croît comme le carré de son corps : c'est elle, et non sa hauteur, qui
//! garde la proportion — et la racine rend la loi indifférente à la forme (une membrane haute et
//! étroite a le titre d'une membrane carrée de même surface). **Aucune constante n'est neuve** :
//! `A₀` et le corps sont ceux d'aujourd'hui.
//!
//! Une membrane plus petite qu'une neuve garde le titre d'aujourd'hui (`s ≥ 1`) : rien de ce qui
//! existe ne rapetisse ; seules les régions plus grandes qu'une membrane neuve grandissent.
//!
//! # Ce que la loi donne sur l'article « Eau » (fiches 91, 92)
//!
//! L'article rangé en colonnes, 8 300 × 5 413 : `s` = 24, un titre de 387 unités — environ
//! 120 pixels quand l'article remplit l'écran. Une section de 2 500 × 1 500, 112 unités (35 px) ;
//! une sous-section de 1 000 × 600, 45 unités (14 px) ; le texte d'un paragraphe, 14 unités
//! (4 px). Chaque niveau s'écrit deux à trois fois plus gros que celui qu'il contient : la
//! hiérarchie des noms d'une carte, déduite de la géométrie seule.
//!
//! # L'encre d'une membrane
//!
//! Le titre se pose **au-dessus** du bord haut, hors de la boîte. Ce qui en a de l'encre se lit
//! donc sur [`Annotation::encre`] : la boîte, et la bande de son titre au-dessus. Le culling, le
//! cadrage (`F`, tout cadrer, l'export), le vol vers un nœud et la sélection élastique la lisent ;
//! la géométrie (l'appartenance, l'aimant, les poignées, ce qu'un agent lit d'une boîte) reste
//! celle de la boîte. La largeur du titre ne se mesure pas ici (le noyau n'a pas de police) : un
//! titre plus long que sa membrane déborde à droite de l'encre, comme avant cette loi.

use crate::geometry::Rect;
use crate::types::Annotation;

/// La taille d'une membrane posée d'un clic, en unités monde — celle d'un dossier aussi : ce sont
/// les deux conteneurs du canevas. Le minimum de la fiche 06 § 8.1 est 180 × 120 ; celle-ci laisse
/// de quoi poser quelque chose dedans.
pub const NAISSANCE: (f64, f64) = (320.0, 240.0);

/// Le corps du titre d'une membrane neuve, en unités monde.
pub const CORPS: f64 = 16.0;

/// Le retrait du titre depuis le bord gauche, pour une membrane neuve, en unités monde.
pub const RETRAIT: f64 = 16.0;

/// L'élévation du haut du titre au-dessus du bord haut, pour une membrane neuve, en unités monde.
pub const ELEVATION: f64 = 18.0;

/// **Le facteur de la loi** : `√(A / A₀)`, jamais moins que 1. Une taille non finie ou nulle rend
/// 1 — le titre d'aujourd'hui —, plutôt qu'un `NaN` qui atteindrait la rastérisation.
pub fn echelle(largeur: f64, hauteur: f64) -> f64 {
    let rapport = (largeur * hauteur).abs() / (NAISSANCE.0 * NAISSANCE.1);
    if rapport.is_finite() {
        rapport.sqrt().max(1.0)
    } else {
        1.0
    }
}

/// **Une membrane porte-t-elle un titre ?** Un texte non vide — la règle du dessin.
pub fn porte_un_titre(texte: Option<&str>) -> bool {
    texte.is_some_and(|t| !t.is_empty())
}

/// **Ce que le titre occupe au-dessus du bord haut**, pour une membrane de cette boîte.
pub fn elevation(largeur: f64, hauteur: f64) -> f64 {
    ELEVATION * echelle(largeur, hauteur)
}

/// **Ce que le titre prend au-dessus du bord en plus d'un titre ordinaire** — celui d'une membrane
/// neuve, que l'écart entre deux nœuds abrite déjà (30 unités pour 18 d'élévation). C'est la place
/// qu'une disposition doit lui réserver : zéro pour une membrane qui n'est pas plus grande qu'une
/// neuve, et toute disposition d'avant cette loi reste la même.
pub fn surcroit(largeur: f64, hauteur: f64) -> f64 {
    ELEVATION * (echelle(largeur, hauteur) - 1.0)
}

/// **La bande du titre** d'une membrane de cette boîte : au-dessus de son bord haut, de toute sa
/// largeur, haute de son élévation — là où le titre se pose, et où il s'écrit même vide (SAISIE-2).
pub fn bande(boite: Rect) -> Rect {
    let e = elevation(boite.width, boite.height);
    Rect::new(boite.left, boite.top - e, boite.width, e)
}

impl Annotation {
    /// **Tout ce qui a de l'encre** : la boîte englobante ([`Annotation::bounds`]), et, pour une
    /// membrane titrée, la bande de son titre au-dessus de son bord (TITRE-1).
    pub fn encre(&self) -> Rect {
        let b = self.bounds();
        match self {
            Annotation::Membrane { text, .. } if porte_un_titre(text.as_deref()) => {
                let t = bande(b);
                Rect::new(b.left, t.top, b.width, b.height + t.height)
            }
            _ => b,
        }
    }
}

#[cfg(test)]
mod tests;
