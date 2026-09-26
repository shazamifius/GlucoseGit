//! L'interprète : de l'arbre de KaTeX aux positions absolues.
//!
//! # Le repère
//!
//! Tout est en `em`, relatif à la taille de police du texte qui accueille la formule. L'origine
//! est le coin gauche de la ligne de base ; **`y` croît vers le haut**. Une formule occupe donc
//! `[0, width] × [−depth, height]`.
//!
//! Ce choix suit celui de KaTeX (`height` au-dessus, `depth` en dessous), qui suit celui de TeX.
//! Il ne suit pas celui d'un écran, où `y` descend — la conversion appartient à qui dessine, et
//! elle est d'une soustraction.
//!
//! # Ce que l'arbre contient, et comment on le lit
//!
//! Trois mécanismes seulement, et ils se combinent :
//!
//! * **Une suite horizontale.** Les enfants d'un `span` ordinaire se posent les uns après les
//!   autres, chacun avançant la plume de sa largeur. Les `margin-left` et `margin-right` s'y
//!   ajoutent — c'est ainsi que KaTeX écrit l'espacement mathématique et la correction d'italique.
//! * **Un empilement vertical** (`vlist`). Ses enfants ne se suivent pas : ils se superposent,
//!   chacun à la hauteur que son `top` indique. La largeur de l'ensemble est celle du plus large.
//! * **Un filet** (`frac-line`, `rule`, `overline`), dessiné plein sur toute la largeur de son
//!   parent. C'est la barre d'une fraction, le trait d'une racine, une ligne de tableau.
//!
//! # La règle verticale, et pourquoi elle est exacte
//!
//! Dans le CSS de KaTeX, chaque enfant d'une `vlist` est une boîte de hauteur nulle décalée de
//! `top`, contenant un `pstrut` de hauteur connue qui pousse le contenu vers le bas. La ligne de
//! base du contenu tombe donc à `y = −top − pstrut`.
//!
//! Ce n'est pas une reconstitution : c'est ce que `makeVList` écrit, et les deux exemples de la
//! documentation du crate le vérifient chiffre par chiffre.
//!
//! # FORMULE-1 — l'arbre, **et** la feuille de style
//!
//! Le navigateur de Glucose Tauri appliquait la feuille de KaTeX ; ce pont l'ignorait, et ses
//! captures le montraient : un numérateur calé à gauche, des fractions collées à leurs voisines,
//! les bornes d'une intégrale posées sur le signe, les lettres d'un mot qui se chevauchent. Le
//! pont applique désormais les règles géométriques de la feuille ([`feuille`]), les marges et
//! décalages que KaTeX écrit sur les symboles, un glyphe par caractère mesuré par les métriques
//! de KaTeX, et les formes SVG telles que KaTeX les trace ([`chemin`]).

mod chemin;
mod feuille;
mod pont;

pub use chemin::{Ajustement, Calage, Commande, Forme};

use crate::{Family, MathError, Mode, Style};
use katex::dom_tree::HtmlDomNode;

/// Un élément posé d'une formule.
#[derive(Debug, Clone, PartialEq)]
pub enum MathItem {
    /// Un caractère, à sa place.
    Glyph {
        /// Le caractère à dessiner. Souvent un seul, parfois une ligature.
        text: String,
        /// Abscisse du bord gauche, en `em`.
        x: f64,
        /// Ordonnée de la **ligne de base**, en `em`, positive vers le haut.
        y: f64,
        /// Taille du glyphe, en `em` — 1,0 pour le corps de la formule, moins pour un indice.
        size: f64,
        family: Family,
        style: Style,
    },
    /// Un trait plein : barre de fraction, filet de tableau.
    Rule {
        x: f64,
        /// Ordonnée du **bas** du filet.
        y: f64,
        width: f64,
        height: f64,
    },
    /// Une forme étirable de KaTeX, nommée, à dessiner dans la boîte donnée.
    ///
    /// # Pourquoi une troisième sorte
    ///
    /// Certaines formes n'ont pas de glyphe à la bonne taille : le radical d'une racine haute,
    /// une accolade qui embrasse quatre lignes, une flèche longue. KaTeX les dessine en SVG, à
    /// partir d'un jeu **fini** de chemins nommés (`sqrtMain`, `sqrtSize1`…) qu'il étire dans
    /// une boîte.
    ///
    /// Les rendre en glyphes serait faux : on ne peut pas étirer un `√` sans le déformer. Les
    /// ignorer serait pire — une racine s'afficherait sans son radical. Le crate nomme donc la
    /// forme et donne sa boîte ; celui qui dessine sait quoi y mettre.
    Path {
        /// Le nom du chemin dans le jeu de KaTeX.
        name: String,
        x: f64,
        /// Ordonnée du **bas** de la boîte.
        y: f64,
        width: f64,
        height: f64,
        /// Le tracé de KaTeX, sa boîte de vue et son ajustement — ou rien, si le chemin est
        /// inconnu ou illisible : il ne se dessine alors pas, plutôt que de travers.
        forme: Option<Forme>,
    },
}

impl MathItem {
    /// L'abscisse du bord gauche.
    pub fn x(&self) -> f64 {
        match self {
            Self::Glyph { x, .. } | Self::Rule { x, .. } | Self::Path { x, .. } => *x,
        }
    }

    /// L'ordonnée de référence — la ligne de base d'un glyphe, le bas d'un filet ou d'une forme.
    pub fn y(&self) -> f64 {
        match self {
            Self::Glyph { y, .. } | Self::Rule { y, .. } | Self::Path { y, .. } => *y,
        }
    }
}

/// Comment les étages d'un empilement se calent les uns par rapport aux autres.
///
/// Par défaut ils sont alignés à gauche ; certains contextes les centrent, et c'est visible :
/// les bornes d'une somme se posent **au milieu** du `∑`, pas contre son bord gauche. La règle
/// vient du CSS de KaTeX, qui écrit `text-align: center` sur trois contextes — les limites d'un
/// grand opérateur, les accents, et les colonnes centrées d'un tableau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Une formule mise en page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MathLayout {
    pub items: Vec<MathItem>,
    /// Largeur totale, en `em`.
    pub width: f64,
    /// Hauteur au-dessus de la ligne de base, en `em`.
    pub height: f64,
    /// Profondeur sous la ligne de base, en `em` (positive).
    pub depth: f64,
}

impl MathLayout {
    /// Hauteur totale : ce qui monte plus ce qui descend.
    pub fn total_height(&self) -> f64 {
        self.height + self.depth
    }

    /// Une formule sans aucun glyphe — le cas d'une source vide.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// La boîte réellement occupée par les éléments posés, en `em` :
    /// `(x_min, y_min, x_max, y_max)`.
    ///
    /// Elle diffère de `(0, −depth, width, height)` : KaTeX réserve de la hauteur pour aligner
    /// des formules entre elles, et une formule courte n'en occupe pas tout. C'est cette
    /// boîte-ci qu'il faut pour cadrer une formule seule dans une carte, et l'autre pour la
    /// poser dans une ligne de texte.
    pub fn ink_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut b: Option<(f64, f64, f64, f64)> = None;
        for item in &self.items {
            let (x0, y0, x1, y1) = match item {
                // Sans les métriques réelles de la fonte, un glyphe est ramené à son point
                // d'ancrage : la boîte d'encre exacte demanderait de lire la fonte, ce que ce
                // crate ne fait pas.
                MathItem::Glyph { x, y, .. } => (*x, *y, *x, *y),
                MathItem::Rule {
                    x,
                    y,
                    width,
                    height,
                } => (*x, *y, x + width, y + height),
                // Une forme : son encre dans sa boîte (`y` de la forme descend depuis le haut de
                // la boîte). Une forme sans tracé lisible ne se dessine pas : elle n'a pas d'encre.
                MathItem::Path {
                    x,
                    y,
                    width,
                    height,
                    forme,
                    ..
                } => {
                    let Some((a, b, c, d)) =
                        forme.as_ref().and_then(|f| f.etendue((*width, *height)))
                    else {
                        continue;
                    };
                    (x + a, y + height - d, x + c, y + height - b)
                }
            };
            b = Some(match b {
                None => (x0, y0, x1, y1),
                Some((ax, ay, bx, by)) => (ax.min(x0), ay.min(y0), bx.max(x1), by.max(y1)),
            });
        }
        b
    }
}

/// Met en page une formule LaTeX.
///
/// ```
/// use glucose_math::{layout, Mode};
/// let f = layout(r"\frac{a}{b}", Mode::Inline).expect("une fraction valide");
/// assert!(f.height > 0.0 && f.depth > 0.0, "une fraction monte et descend");
/// ```
///
/// Une source mal formée rend [`MathError::Parse`] avec le message de KaTeX, fait pour être
/// montré : c'est lui qui colore une formule en rouge pendant la frappe.
pub fn layout(latex: &str, mode: Mode) -> Result<MathLayout, MathError> {
    layout_avec(latex, mode, &|_, _, _| None)
}

/// **Qui mesure l'avance d'un caractère** : `(caractère, famille, style)` → sa largeur en `em`,
/// ou rien si la fonte ne le porte pas.
pub type Avance<'a> = &'a dyn Fn(char, Family, Style) -> Option<f64>;

/// Met en page une formule en mesurant chaque caractère **dans la fonte qui le dessinera**.
///
/// # Pourquoi la fonte, et pas les métriques de KaTeX
///
/// Le navigateur de Glucose Tauri avançait de la largeur que la fonte donne à chaque glyphe ;
/// les métriques de KaTeX ne lui servaient qu'aux hauteurs. Les deux s'accordent sur 2 010 des
/// 2 035 glyphes des vingt fontes — pas sur `∬` et `∭` (0,56 em dans les métriques, 1,08 et
/// 1,59 dans la fonte), ni sur `°`, ni sur les accents combinants, larges de zéro dans la fonte.
/// Mesurées par les métriques, les bornes de `\iint_D` se posaient **sur** le signe. Celui qui
/// dessine tient les fontes : il mesure. [`layout`] garde les métriques pour qui n'en a pas.
pub fn layout_avec(latex: &str, mode: Mode, avance: Avance) -> Result<MathLayout, MathError> {
    let ctx = katex::KatexContext::default();
    let options = katex::Settings {
        display_mode: mode == Mode::Display,
        ..Default::default()
    };

    let tree = katex::render_to_dom_tree(&ctx, latex, &options)
        .map_err(|e| MathError::Parse(e.to_string()))?;

    let etat = Etat {
        size: 1.0,
        family: Family::Main,
        style: Style::ROMAN,
        align: Align::Left,
    };
    let mut pont = pont::Pont::nouveau(&ctx, avance);
    // L'arbre rendu porte deux enfants : la version MathML, destinée aux lecteurs d'écran, et
    // la version HTML. Seule la seconde porte des positions.
    let mut largeur = 0.0_f64;
    for enfant in &tree.children {
        if let HtmlDomNode::DomSpan(span) = enfant {
            if span.classes.contains("katex-mathml") {
                continue;
            }
        }
        largeur = largeur.max(pont.pose(enfant, etat, (0.0, 0.0)));
    }

    Ok(MathLayout {
        items: pont.fin(),
        width: largeur,
        height: tree.height,
        depth: tree.depth,
    })
}

/// Ce qui se transmet de parent à enfant pendant le parcours.
#[derive(Debug, Clone, Copy)]
struct Etat {
    /// Facteur de taille courant, en `em`.
    size: f64,
    family: Family,
    style: Style,
    /// L'alignement que le contexte impose aux empilements qu'il contient.
    align: Align,
}
