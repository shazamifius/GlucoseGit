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

use crate::{Family, MathError, Mode, Style};
use katex::dom_tree::HtmlDomNode;
use katex::types::CssProperty;

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
                MathItem::Rule { x, y, width, height }
                | MathItem::Path { x, y, width, height, .. } => (*x, *y, x + width, y + height),
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
    let ctx = katex::KatexContext::default();
    let mut options = katex::Settings::default();
    options.display_mode = mode == Mode::Display;

    let tree = katex::render_to_dom_tree(&ctx, latex, &options)
        .map_err(|e| MathError::Parse(e.to_string()))?;

    let mut sortie = Vec::new();
    let etat = Etat { size: 1.0, family: Family::Main, style: Style::ROMAN, align: Align::Left };
    // L'arbre rendu porte deux enfants : la version MathML, destinée aux lecteurs d'écran, et
    // la version HTML. Seule la seconde porte des positions.
    let mut largeur = 0.0_f64;
    for enfant in &tree.children {
        if let HtmlDomNode::DomSpan(span) = enfant {
            if span.classes.contains("katex-mathml") {
                continue;
            }
        }
        largeur = largeur.max(pose(enfant, etat, 0.0, 0.0, &mut sortie));
    }

    Ok(MathLayout { items: sortie, width: largeur, height: tree.height, depth: tree.depth })
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

/// Les multiplicateurs de taille de KaTeX, indexés par le numéro de `sizeN` (1 à 11).
///
/// La table est celle de sa feuille de style : `.sizing.reset-size6.size3` vaut `0.7em`, et
/// `SIZE_MULTIPLIERS[3] / SIZE_MULTIPLIERS[6] = 0.7 / 1.0`. La taille 6 est la taille normale.
const SIZE_MULTIPLIERS: [f64; 12] =
    [1.0, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.44, 1.728, 2.074, 2.488];

/// Lit une longueur CSS en `em`. Rend `0` pour tout ce qui n'est pas un nombre suivi de `em` —
/// KaTeX n'écrit que des `em` dans les propriétés qui nous intéressent.
fn em(valeur: Option<&str>) -> f64 {
    valeur
        .and_then(|v| v.trim().strip_suffix("em"))
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Le facteur de taille qu'un jeu de classes `sizing reset-sizeN sizeM` applique, et la famille
/// ou le style qu'il impose, s'il en impose.
fn applique_classes(classes: &katex::types::ClassList, mut etat: Etat) -> Etat {
    let (mut reset, mut taille) = (None, None);
    for classe in classes {
        match classe {
            "mathnormal" => {
                etat.family = Family::Math;
                etat.style = Style::ITALIC;
            }
            "mathit" => {
                etat.family = Family::Main;
                etat.style = Style::ITALIC;
            }
            "mathbf" => {
                etat.family = Family::Main;
                etat.style = Style::BOLD;
            }
            "boldsymbol" => {
                etat.family = Family::Math;
                etat.style = Style { bold: true, italic: true };
            }
            "mainrm" | "textrm" => {
                etat.family = Family::Main;
                etat.style = Style::ROMAN;
            }
            "amsrm" | "textbb" => etat.family = Family::Ams,
            "mathcal" => etat.family = Family::Caligraphic,
            "textfrak" => etat.family = Family::Fraktur,
            "textboldfrak" => {
                etat.family = Family::Fraktur;
                etat.style = Style::BOLD;
            }
            "textsf" | "mathsf" => etat.family = Family::SansSerif,
            "textboldsf" => {
                etat.family = Family::SansSerif;
                etat.style = Style::BOLD;
            }
            "textitsf" => {
                etat.family = Family::SansSerif;
                etat.style = Style::ITALIC;
            }
            "textscr" => etat.family = Family::Script,
            "texttt" | "mathtt" => etat.family = Family::Typewriter,
            // Les trois contextes que le CSS de KaTeX centre, et les deux qu'il aligne.
            "op-limits" | "accent" | "col-align-c" => etat.align = Align::Center,
            "col-align-l" => etat.align = Align::Left,
            "col-align-r" => etat.align = Align::Right,
            // Les grands opérateurs et les délimiteurs puisent dans les fontes de taille.
            "small-op" | "delim-size1" => etat.family = Family::Size1,
            "large-op" => etat.family = Family::Size2,
            _ => {
                if let Some(n) = classe.strip_prefix("reset-size").and_then(|n| n.parse().ok()) {
                    reset = Some(n);
                } else if let Some(n) = classe.strip_prefix("size").and_then(|n| n.parse().ok()) {
                    taille = Some(n);
                }
            }
        }
    }

    // `delimsizing size1..4` nomme une fonte, pas un facteur — c'est la seule collision entre
    // les deux familles de classes `sizeN`, et KaTeX la lève par la classe qui l'accompagne.
    if classes.contains("delimsizing") {
        if let Some(n) = taille {
            etat.family = match n {
                1 => Family::Size1,
                2 => Family::Size2,
                3 => Family::Size3,
                _ => Family::Size4,
            };
        }
        return etat;
    }

    if let Some(m) = taille {
        let de = SIZE_MULTIPLIERS[reset.unwrap_or(6usize).min(11)];
        let vers = SIZE_MULTIPLIERS[m.min(11usize)];
        if de > 0.0 {
            etat.size *= vers / de;
        }
    }
    etat
}

/// Pose un nœud à `(x, y)` et rend la largeur qu'il occupe, en `em`.
fn pose(node: &HtmlDomNode, etat: Etat, x: f64, y: f64, out: &mut Vec<MathItem>) -> f64 {
    match node {
        HtmlDomNode::Symbol(s) => {
            if s.text.is_empty() || s.text == "\u{200b}" {
                // L'espace de largeur nulle sert au calage d'une `vlist` en CSS ; il ne se
                // dessine pas.
                return 0.0;
            }
            let etat = applique_classes(&s.classes, etat);
            out.push(MathItem::Glyph {
                text: s.text.clone(),
                x,
                y,
                size: etat.size,
                family: etat.family,
                style: etat.style,
            });
            // La correction d'italique appartient au glyphe suivant, pas à celui-ci : c'est
            // ainsi que KaTeX la compte.
            s.width * etat.size
        }

        HtmlDomNode::DomSpan(span) => {
            let etat = applique_classes(&span.classes, etat);
            let gauche = em(span.style.get(CssProperty::MarginLeft)) * etat.size;
            let droite = em(span.style.get(CssProperty::MarginRight)) * etat.size;
            let x = x + gauche;

            // Un filet : il se dessine plein, sur la largeur que son parent lui donne. Sa
            // largeur réelle est posée par l'empilement qui le contient, qui seul la connaît ;
            // ici on note son épaisseur et sa place.
            if span.classes.contains("frac-line") {
                let epaisseur = em(span.style.get(CssProperty::BorderBottomWidth)) * etat.size;
                out.push(MathItem::Rule { x, y, width: 0.0, height: epaisseur.max(f64::MIN_POSITIVE) });
                return gauche + droite;
            }

            if span.classes.contains("vlist") {
                return gauche + empile(span, etat, x, y, out) + droite;
            }

            // Une forme étirable : `hide-tail` (radical, queue d'accolade) ou `stretchy`
            // (flèches longues). Le span porte la boîte, le SVG qu'il contient porte le nom du
            // chemin. Le SVG lui-même est large de 400 em et volontairement rogné par son
            // parent : c'est le `min-width` du parent qui dit la largeur vraie, pas le SVG.
            if span.classes.contains("hide-tail") || span.classes.contains("stretchy") {
                let largeur = em(span.style.get(CssProperty::MinWidth)).max(em(span.style.get(CssProperty::Width)));
                let hauteur = em(span.style.get(CssProperty::Height));
                if let Some(nom) = nom_du_chemin(span) {
                    out.push(MathItem::Path {
                        name: nom,
                        x,
                        y,
                        width: largeur * etat.size,
                        height: hauteur * etat.size,
                    });
                }
                return gauche + largeur * etat.size + droite;
            }

            // Une suite horizontale ordinaire.
            let mut avance = 0.0;
            for enfant in &span.children {
                avance += pose(enfant, etat, x + avance, y, out);
            }
            // Un `mspace` n'a pas d'enfant : toute sa largeur est dans ses marges.
            gauche + avance + droite
        }

        HtmlDomNode::Fragment(f) => {
            let mut avance = 0.0;
            for enfant in &f.children {
                avance += pose(enfant, etat, x + avance, y, out);
            }
            avance
        }

        HtmlDomNode::Anchor(a) => {
            let mut avance = 0.0;
            for enfant in &a.children {
                avance += pose(enfant, etat, x + avance, y, out);
            }
            avance
        }

        // MathML ne porte aucune position — c'est la version destinée aux lecteurs d'écran.
        // Les images et les SVG de délimiteurs extensibles ne sont pas encore traités : ils
        // concernent les très grands délimiteurs, où KaTeX assemble un trait à partir de
        // morceaux.
        HtmlDomNode::MathML(_) | HtmlDomNode::Img(_) | HtmlDomNode::SvgNode(_) => 0.0,
    }
}

/// Le nom du chemin qu'un span de forme étirable contient, s'il en contient un.
fn nom_du_chemin(span: &katex::dom_tree::Span<HtmlDomNode>) -> Option<String> {
    use katex::dom_tree::SvgChildNode;
    for enfant in &span.children {
        if let HtmlDomNode::SvgNode(svg) = enfant {
            for forme in &svg.children {
                if let SvgChildNode::Path(chemin) = forme {
                    return Some(chemin.path_name.clone());
                }
            }
        }
    }
    None
}

/// Pose les enfants d'une `vlist` : ils se superposent au lieu de se suivre.
///
/// Chaque enfant est un `span` de hauteur nulle portant `top`, dont le premier enfant est un
/// `pstrut` de hauteur connue. La ligne de base du contenu tombe à `y = −top − pstrut`.
///
/// Les filets qu'un empilement contient reçoivent ici leur largeur : c'est lui qui la connaît,
/// puisqu'elle vaut celle du plus large de ses enfants.
fn empile(span: &katex::dom_tree::Span<HtmlDomNode>, etat: Etat, x: f64, y: f64, out: &mut Vec<MathItem>) -> f64 {
    // Ce que chaque étage a produit : sa tranche d'éléments, et la largeur qu'il occupe. La
    // largeur de l'empilement n'est connue qu'après le dernier, et deux choses en dépendent —
    // l'alignement des étages, et la longueur des filets. D'où les deux passes.
    let mut etages: Vec<(usize, usize, f64)> = Vec::new();
    let mut largeur = 0.0_f64;

    for enfant in &span.children {
        let debut = out.len();
        let HtmlDomNode::DomSpan(boite) = enfant else {
            // Un enfant qui n'est pas une boîte décalée est posé tel quel — c'est le cas de
            // l'espace de largeur nulle qui sert au calage de la ligne de base en CSS.
            let w = pose(enfant, etat, x, y, out);
            etages.push((debut, out.len(), w));
            largeur = largeur.max(w);
            continue;
        };

        let top = em(boite.style.get(CssProperty::Top));
        let pstrut = boite
            .children
            .iter()
            .find_map(|c| match c {
                HtmlDomNode::DomSpan(s) if s.classes.contains("pstrut") => {
                    Some(em(s.style.get(CssProperty::Height)))
                }
                _ => None,
            })
            .unwrap_or(0.0);
        // Le décalage est exprimé dans l'échelle du parent de l'empilement.
        let ligne = y + (-top - pstrut) * etat.size;

        let etat_boite = applique_classes(&boite.classes, etat);
        let marge = em(boite.style.get(CssProperty::MarginLeft)) * etat_boite.size;
        let mut avance = 0.0;
        for petit in &boite.children {
            if matches!(petit, HtmlDomNode::DomSpan(s) if s.classes.contains("pstrut")) {
                continue;
            }
            avance += pose(petit, etat_boite, x + marge + avance, ligne, out);
        }
        etages.push((debut, out.len(), marge + avance));
        largeur = largeur.max(marge + avance);
    }

    for (debut, fin, w) in etages {
        let decalage = match etat.align {
            Align::Left => 0.0,
            Align::Center => (largeur - w) / 2.0,
            Align::Right => largeur - w,
        };
        for item in &mut out[debut..fin] {
            match item {
                MathItem::Glyph { x: gx, .. } | MathItem::Path { x: gx, .. } => *gx += decalage,
                MathItem::Rule { x: rx, width, .. } => {
                    *rx += decalage;
                    // Un filet occupe toute la largeur de l'empilement, quel que soit
                    // l'alignement : c'est la barre d'une fraction ou le trait d'une racine.
                    if *width == 0.0 {
                        *width = largeur - (*rx - x);
                    }
                }
            }
        }
    }
    largeur
}
