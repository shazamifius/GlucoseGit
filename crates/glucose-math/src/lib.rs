//! Les formules LaTeX de Glucose, mises en page en **géométrie pure**.
//!
//! # Ce que ce crate fait, et ce qu'il ne fait pas
//!
//! Il prend du LaTeX et rend une liste de glyphes et de filets **positionnés**, exprimés en
//! `em` par rapport à une ligne de base. Il ne connaît ni pixel, ni rastériseur, ni fichier de
//! fonte : il dit *quel caractère, dans quelle famille, à quelle taille, à quel endroit*. Le
//! dessin appartient à celui qui a déjà un moteur typographique.
//!
//! La conséquence pratique est que **toute formule est vérifiable sans écran** : que `\frac{a}{b}`
//! pose bien `a` au-dessus de `b` avec un filet entre les deux est une assertion sur des nombres.
//!
//! # Pourquoi une dépendance, et pourquoi celle-là
//!
//! La charte du projet dit qu'une dépendance se défend par une impossibilité ou un coût
//! démesuré. Réécrire la typographie mathématique de TeX est ce coût : KaTeX couvre environ six
//! cents commandes, et un sous-ensemble maison serait une version moins bonne de ce qui existe.
//!
//! Deux bibliothèques ont été essayées sur les mêmes formules. `latex-rust` aplatit les matrices
//! (`\begin{pmatrix} a & b \\ c & d \end{pmatrix}` sort en `(a bc d)`), superpose les bornes
//! d'une intégrale, dessine `\binom` avec une barre de fraction et tronque ses propres boîtes.
//! [`katex-rs`](https://docs.rs/katex-rs) est le portage du vrai KaTeX — même lexer, mêmes
//! métriques, même mise en page — et ses dépendances sont six crates légères, sans JavaScript.
//!
//! # Le pont : de l'arbre HTML aux positions
//!
//! KaTeX produit un arbre de `span` destiné à un navigateur. Ce n'est pas un obstacle : **toute
//! la typographie y est déjà faite**, et le CSS ne fait que l'appliquer. Le vocabulaire est
//! fermé — des boîtes posées horizontalement, des `vlist` empilées avec un décalage explicite,
//! des marges, et des symboles portant leurs métriques.
//!
//! La règle de position verticale se lit dans le CSS de KaTeX et se vérifie sur ses sorties :
//! dans une `vlist`, chaque enfant est une boîte de hauteur nulle décalée de `top`, qui contient
//! un `pstrut` de hauteur connue. La ligne de base du contenu est donc à
//!
//! ```text
//! y = −top − hauteur_du_pstrut        (en em, positif vers le haut)
//! ```
//!
//! Sur `\frac{a}{b}` : le dénominateur a `top: −2.655em` et un `pstrut` de `3em`, soit
//! `y = −0,345` — sous la ligne de base ; le numérateur a `top: −3.394em`, soit `y = +0,394`.
//! Sur `\int_0^\infty` : la borne basse tombe à `−0,356`, la haute à `+0,558`.

use std::fmt;

pub mod layout;

pub use layout::{layout, Align, MathItem, MathLayout};

/// Les douze familles de fontes de KaTeX.
///
/// Le choix vient des classes CSS que KaTeX pose sur chaque symbole (`mathnormal`, `mord`,
/// `op-symbol small-op`…), et la correspondance est celle de sa feuille de style — elle n'est
/// pas devinée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Family {
    /// `KaTeX_Main` — la fonte du texte droit, des chiffres et de la ponctuation.
    #[default]
    Main,
    /// `KaTeX_Math`, italique — les variables (`x`, `y`, `n`).
    Math,
    /// `KaTeX_AMS` — les symboles de l'American Mathematical Society.
    Ams,
    /// `KaTeX_Caligraphic` — `\mathcal`.
    Caligraphic,
    /// `KaTeX_Fraktur` — `\mathfrak`.
    Fraktur,
    /// `KaTeX_SansSerif` — `\mathsf`.
    SansSerif,
    /// `KaTeX_Script` — `\mathscr`.
    Script,
    /// `KaTeX_Typewriter` — `\mathtt`.
    Typewriter,
    /// `KaTeX_Size1` à `KaTeX_Size4` — les grands opérateurs et les délimiteurs extensibles.
    /// Un `\sum` en `\displaystyle` vient de `Size2`, une parenthèse haute de `Size3` ou `Size4`.
    Size1,
    Size2,
    Size3,
    Size4,
}

impl Family {
    /// Le nom de la famille tel qu'il apparaît dans les fichiers de fonte de KaTeX.
    pub fn font_name(self) -> &'static str {
        match self {
            Self::Main => "KaTeX_Main",
            Self::Math => "KaTeX_Math",
            Self::Ams => "KaTeX_AMS",
            Self::Caligraphic => "KaTeX_Caligraphic",
            Self::Fraktur => "KaTeX_Fraktur",
            Self::SansSerif => "KaTeX_SansSerif",
            Self::Script => "KaTeX_Script",
            Self::Typewriter => "KaTeX_Typewriter",
            Self::Size1 => "KaTeX_Size1",
            Self::Size2 => "KaTeX_Size2",
            Self::Size3 => "KaTeX_Size3",
            Self::Size4 => "KaTeX_Size4",
        }
    }
}

/// La graisse et l'inclinaison d'un glyphe, indépendantes de sa famille.
///
/// Les familles `Size1` à `Size4` n'ont qu'une variante ; les autres en ont jusqu'à quatre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub const ROMAN: Self = Self { bold: false, italic: false };
    pub const ITALIC: Self = Self { bold: false, italic: true };
    pub const BOLD: Self = Self { bold: true, italic: false };

    /// Le suffixe du fichier de fonte : `KaTeX_Main` + `-Regular`, `-Bold`, `-Italic`,
    /// `-BoldItalic`.
    pub fn suffix(self) -> &'static str {
        match (self.bold, self.italic) {
            (false, false) => "Regular",
            (true, false) => "Bold",
            (false, true) => "Italic",
            (true, true) => "BoldItalic",
        }
    }
}

/// Comment une formule se pose dans le texte qui l'entoure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// `$...$` — dans le fil du texte. Les indices se serrent contre leur opérateur, les
    /// fractions se resserrent.
    #[default]
    Inline,
    /// `$$...$$` — sur sa propre ligne, en grand. Les bornes d'une somme passent au-dessus et
    /// en dessous plutôt qu'à côté.
    Display,
}

/// Ce qui peut échouer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    /// Le LaTeX est mal formé — accolade non fermée, commande inconnue. Le message vient de
    /// KaTeX et est fait pour être montré : c'est lui qui colore une formule en rouge dans
    /// l'éditeur.
    Parse(String),
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for MathError {}

#[cfg(test)]
mod tests;
