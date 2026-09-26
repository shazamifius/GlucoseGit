//! **La feuille de style de KaTeX, pour ce qui touche la géométrie.**
//!
//! # Pourquoi elle est ici
//!
//! KaTeX écrit une partie de sa mise en page dans son arbre — les `top` d'un empilement, les
//! marges de l'espacement mathématique — et le reste dans sa feuille de style : qu'une fraction
//! centre son numérateur, qu'un délimiteur vide occupe `0.12em`, que l'indice d'une racine
//! recule de `0.5556em`. Le navigateur de Glucose Tauri appliquait cette feuille ; le pont de
//! Glucose Rust l'ignorait. D'où, sur ses captures, un numérateur calé à gauche, des fractions
//! collées à leurs voisines, des accents décentrés.
//!
//! Le vocabulaire est **fermé** : une trentaine de règles de `katex.css` (0.16) touchent une
//! position ou une largeur, et chacune est ici, citée. Aucune n'est inventée.
//!
//! # Les classes préfixées
//!
//! katex-rs écrit `katex-accent`, `katex-stretchy`, `katex-overline`, `katex-sizing` là où la
//! feuille dit `accent`, `stretchy`… Le pont cherchait `accent` et ne le trouvait jamais : aucun
//! accent n'était centré. Toute classe est lue sous son nom de la feuille ([`nom`]).

use super::{Align, Etat};
use crate::Family;

/// Le nom d'une classe tel que la feuille de KaTeX l'écrit.
pub(super) fn nom(classe: &str) -> &str {
    classe.strip_prefix("katex-").unwrap_or(classe)
}

/// La liste porte-t-elle cette classe, sous son nom de la feuille ?
pub(super) fn porte(classes: &katex::types::ClassList, voulue: &str) -> bool {
    classes.iter().any(|c| nom(c) == voulue)
}

/// Lit une longueur CSS en `em`. Rend `0` pour tout ce qui n'est pas un nombre suivi de `em` —
/// KaTeX n'écrit que des `em` dans les propriétés qui nous intéressent.
pub(super) fn em(valeur: Option<&str>) -> f64 {
    valeur
        .and_then(|v| v.trim().strip_suffix("em"))
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// **Un nœud transparent** (`color: transparent`) : c'est ainsi que KaTeX écrit le contenu de
/// `\phantom`, qui réserve sa place sans se montrer.
pub(super) fn transparent(style: &katex::types::CssStyle) -> bool {
    style
        .get(katex::types::CssProperty::Color)
        .is_some_and(|c| c.trim() == "transparent")
}

/// Les multiplicateurs de taille de KaTeX, indexés par le numéro de `sizeN` (1 à 11).
///
/// La table est celle de sa feuille de style : `.sizing.reset-size6.size3` vaut `0.7em`, et
/// `SIZE_MULTIPLIERS[3] / SIZE_MULTIPLIERS[6] = 0.7 / 1.0`. La taille 6 est la taille normale.
const SIZE_MULTIPLIERS: [f64; 12] = [
    1.0, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.44, 1.728, 2.074, 2.488,
];

/// Une famille, une inclinaison, une graisse — chacune `None` quand la règle ne la touche pas.
type Fonte = (Option<Family>, Option<bool>, Option<bool>);

/// **Ce qu'une classe dit de la fonte**, règle par règle de la feuille (lignes 28 à 47 de
/// `katex.min.css`, 0.16).
///
/// La famille, l'inclinaison et la graisse sont trois propriétés distinctes en CSS : `.mathbf`
/// fixe la famille et la graisse, et laisse l'inclinaison à ce qui l'entoure ; `.textit` ne
/// fixe que l'inclinaison. Le pont les traitait d'un bloc, et oubliait `.mathbb` et
/// `.mathfrak` : `\mathbb{R}` sortait en « R » droit, `\mathfrak{g}` en « g ».
fn fonte(classe: &str) -> Option<Fonte> {
    use Family::*;
    let (oui, non) = (Some(true), Some(false));
    Some(match classe {
        "textrm" => (Some(Main), None, None),
        "textsf" | "mathsf" => (Some(SansSerif), None, None),
        "texttt" | "mathtt" => (Some(Typewriter), None, None),
        "mathnormal" => (Some(Math), oui, None),
        "mathit" => (Some(Main), oui, None),
        "mathrm" => (None, non, None),
        "mathbf" => (Some(Main), None, oui),
        "boldsymbol" => (Some(Math), oui, oui),
        "amsrm" | "mathbb" | "textbb" => (Some(Ams), None, None),
        "mathcal" => (Some(Caligraphic), None, None),
        "mathfrak" | "textfrak" => (Some(Fraktur), None, None),
        "mathboldfrak" | "textboldfrak" => (Some(Fraktur), None, oui),
        "mathscr" | "textscr" => (Some(Script), None, None),
        "mathboldsf" | "textboldsf" => (Some(SansSerif), None, oui),
        "mathitsf" | "mathsfit" | "textitsf" => (Some(SansSerif), oui, None),
        "mainrm" => (Some(Main), non, None),
        "textit" => (None, oui, None),
        "textbf" => (None, None, oui),
        _ => return None,
    })
}

/// Ce que les classes d'un nœud transmettent à ses enfants : la taille, la famille et le style
/// de fonte, l'alignement des empilements (`text-align`, qui s'hérite).
pub(super) fn applique_classes(classes: &katex::types::ClassList, mut etat: Etat) -> Etat {
    let (mut reset, mut taille) = (None, None);
    for classe in classes {
        let nom = nom(classe);
        if let Some((famille, italique, gras)) = fonte(nom) {
            etat.family = famille.unwrap_or(etat.family);
            etat.style.italic = italique.unwrap_or(etat.style.italic);
            etat.style.bold = gras.unwrap_or(etat.style.bold);
            continue;
        }
        match nom {
            // `text-align` : `.mfrac > span > span`, `.op-limits > .vlist-t`,
            // `.accent > .vlist-t`, `.x-arrow`, `.mover`, `.munder`, `.col-align-c > .vlist-t`
            // centrent ; `.msupsub`, `.svg-align`, `.col-align-l` calent à gauche ;
            // `.col-align-r` à droite.
            "mfrac" | "op-limits" | "accent" | "x-arrow" | "mover" | "munder" | "col-align-c" => {
                etat.align = Align::Center;
            }
            "msupsub" | "svg-align" | "col-align-l" => etat.align = Align::Left,
            "col-align-r" => etat.align = Align::Right,
            // Les grands opérateurs et les délimiteurs puisent dans les fontes de taille.
            "small-op" | "delim-size1" => etat.family = Family::Size1,
            "large-op" => etat.family = Family::Size2,
            "delim-size4" => etat.family = Family::Size4,
            autre => {
                if let Some(n) = autre
                    .strip_prefix("reset-size")
                    .and_then(|n| n.parse().ok())
                {
                    reset = Some(n);
                } else if let Some(n) = autre.strip_prefix("size").and_then(|n| n.parse().ok()) {
                    taille = Some(n);
                }
            }
        }
    }

    // `delimsizing size1..4` nomme une fonte, pas un facteur — c'est la seule collision entre
    // les deux familles de classes `sizeN`, et KaTeX la lève par la classe qui l'accompagne.
    if porte(classes, "delimsizing") {
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

/// **Un filet que la feuille étend à toute la largeur de son empilement** (`width: 100%`), et
/// l'épaisseur qu'il prend de sa bordure basse : `.mfrac .frac-line`, `.overline .overline-line`,
/// `.underline .underline-line`, `.hline`, `.hdashline`.
pub(super) fn est_un_filet(classes: &katex::types::ClassList) -> bool {
    [
        "frac-line",
        "overline-line",
        "underline-line",
        "hline",
        "hdashline",
    ]
    .iter()
    .any(|c| porte(classes, c))
}

/// **La part d'un conteneur qu'une forme occupe** : `.halfarrow-left { left: 0; width: 50.2% }`
/// et les autres. Une forme sans l'une de ces classes occupe tout son conteneur
/// (`.stretchy`, `.hide-tail`, `svg { width: 100% }`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Portion {
    /// Où elle commence, en fraction du conteneur, depuis son bord gauche — ou droit.
    pub debut: f64,
    pub largeur: f64,
    pub depuis_la_droite: bool,
}

impl Portion {
    pub const PLEINE: Self = Self {
        debut: 0.0,
        largeur: 1.0,
        depuis_la_droite: false,
    };
}

/// La portion qu'imposent les classes d'un morceau de forme.
pub(super) fn portion(classes: &katex::types::ClassList) -> Portion {
    let p = |debut, largeur, depuis_la_droite| Portion {
        debut,
        largeur,
        depuis_la_droite,
    };
    if porte(classes, "halfarrow-left") {
        p(0.0, 0.502, false)
    } else if porte(classes, "halfarrow-right") {
        p(0.0, 0.502, true)
    } else if porte(classes, "brace-left") {
        p(0.0, 0.251, false)
    } else if porte(classes, "brace-center") {
        p(0.25, 0.5, false)
    } else if porte(classes, "brace-right") {
        p(0.0, 0.251, true)
    } else {
        Portion::PLEINE
    }
}

/// **Les marges et le bourrage que la feuille donne à une classe**, en `em` du nœud :
/// `(marge gauche, marge droite, bourrage gauche, bourrage droit)`.
///
/// * `.sqrt > .root { margin-left: .2778em; margin-right: -.5556em }` — l'indice d'une racine ;
/// * `.cancel-lap { margin: 0 -.2em }`, `.angl { margin-right: .03889em }` ;
/// * `.x-arrow-pad { padding: 0 .5em }`, `.cd-arrow-pad { padding: 0 .55556em 0 .27778em }`,
///   `.boxpad { padding: 0 .3em }`, `.cancel-pad { padding: 0 .2em }`,
///   `.anglpad { padding: 0 .03889em }`.
pub(super) fn marges_de_classe(classes: &katex::types::ClassList) -> (f64, f64, f64, f64) {
    let mut m = (0.0, 0.0, 0.0, 0.0);
    for classe in classes {
        let (ml, mr, pl, pr) = match nom(classe) {
            "root" => (0.277_777_777_8, -0.555_555_555_6, 0.0, 0.0),
            "cancel-lap" => (-0.2, -0.2, 0.0, 0.0),
            "angl" => (0.0, 0.038_89, 0.0, 0.0),
            "x-arrow-pad" => (0.0, 0.0, 0.5, 0.5),
            "cd-arrow-pad" => (0.0, 0.0, 0.277_78, 0.555_56),
            "boxpad" => (0.0, 0.0, 0.3, 0.3),
            "cancel-pad" => (0.0, 0.0, 0.2, 0.2),
            "anglpad" => (0.0, 0.0, 0.038_89, 0.038_89),
            _ => continue,
        };
        m = (m.0 + ml, m.1 + mr, m.2 + pl, m.3 + pr);
    }
    m
}

/// **Les marges d'un style en ligne**, en `em` du nœud : `margin-left`, `margin-right`, ou la
/// forme abrégée `margin` (une à quatre valeurs, dans l'ordre haut, droite, bas, gauche) — celle
/// que KaTeX écrit sur le filet vertical d'un tableau (`margin: 0 -0.02em`).
pub(super) fn marges(style: &katex::types::CssStyle) -> (f64, f64) {
    use katex::types::CssProperty::{Margin, MarginLeft, MarginRight};
    let abregee: Vec<f64> = style
        .get(Margin)
        .map(|m| m.split_whitespace().map(|v| em(Some(v))).collect())
        .unwrap_or_default();
    let (g, d) = match abregee[..] {
        [tout] => (tout, tout),
        [_, cotes] | [_, cotes, _] => (cotes, cotes),
        [_, droite, _, gauche] => (gauche, droite),
        _ => (0.0, 0.0),
    };
    (
        style.get(MarginLeft).map_or(g, |v| em(Some(v))),
        style.get(MarginRight).map_or(d, |v| em(Some(v))),
    )
}

/// **Une largeur écrite dans le style**, en `em` du nœud : l'écart entre deux colonnes, un
/// morceau de grand délimiteur, le trait d'une barre verticale haute.
///
/// Le seul `calc` que KaTeX écrive — l'étage d'un accent large, `calc(100% - 2s)` avec une
/// marge gauche de `2s` — n'est pas lu : il vaut exactement la largeur qu'un bloc prend de
/// lui-même, son conteneur moins ses marges ([`super::pont`] la lui donne).
pub(super) fn largeur(style: &katex::types::CssStyle) -> Option<f64> {
    let v = style.get(katex::types::CssProperty::Width)?.trim();
    v.ends_with("em").then(|| em(Some(v)))
}

/// **Les bordures d'un nœud**, en `em` : `(haut, droite, bas, gauche)`, et si la boîte les
/// compte dans sa hauteur (`box-sizing: border-box`).
///
/// La feuille en donne à `.fbox`, `.fcolorbox` (`.04em` tout autour) et `.angl` (haut et
/// droite, `.049em`) ; le style en ligne les remplace — `border-width` pour `\boxed`,
/// `border-right-width` pour le filet vertical d'un tableau, `border-right-width` et
/// `border-top-width` pour `\rule`, qui n'est **que** bordure. Les filets du bas (`.frac-line`
/// et les autres) ne passent pas par ici : ils s'étendent à leur empilement ([`est_un_filet`]).
pub(super) fn bordures(
    classes: &katex::types::ClassList,
    style: &katex::types::CssStyle,
) -> ([f64; 4], bool) {
    use katex::types::CssProperty::{BorderRightWidth, BorderTopWidth, BorderWidth};
    let cadre = porte(classes, "fbox") || porte(classes, "fcolorbox");
    let angle = porte(classes, "angl");
    let mut b = if cadre {
        [0.04; 4]
    } else if angle {
        [0.049, 0.049, 0.0, 0.0]
    } else {
        [0.0; 4]
    };
    if let Some(v) = style.get(BorderWidth) {
        b = [em(Some(v)); 4];
    }
    if let Some(v) = style.get(BorderTopWidth) {
        b[0] = em(Some(v));
    }
    if let Some(v) = style.get(BorderRightWidth) {
        b[1] = em(Some(v));
    }
    (b, cadre || angle)
}

/// **Une largeur que la feuille impose**, quel que soit le contenu : `.nulldelimiter { width:
/// .12em }` — le vide qui borde une fraction, un binôme —, et `.thinbox`, `.llap`, `.rlap`,
/// `.clap`, `.accent-body:not(.accent-full)`, larges de zéro.
pub(super) fn largeur_imposee(classes: &katex::types::ClassList) -> Option<f64> {
    if porte(classes, "nulldelimiter") {
        Some(0.12)
    } else if ["thinbox", "llap", "rlap", "clap"]
        .iter()
        .any(|c| porte(classes, c))
        || (porte(classes, "accent-body") && !porte(classes, "accent-full"))
    {
        Some(0.0)
    } else {
        None
    }
}

/// **Où le contenu d'un recouvrement se pose**, en fraction de sa propre largeur :
/// `.llap > .inner { right: 0 }` le fait finir au point, `.rlap` le fait commencer au point,
/// `.clap > .inner > span { margin-left: -50% }` le centre dessus.
pub(super) fn recul_d_un_recouvrement(classes: &katex::types::ClassList) -> Option<f64> {
    if porte(classes, "llap") {
        Some(1.0)
    } else if porte(classes, "rlap") {
        Some(0.0)
    } else if porte(classes, "clap") {
        Some(0.5)
    } else {
        None
    }
}
