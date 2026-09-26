//! **Le parcours de l'arbre de KaTeX** : chaque nœud posé à sa place, sous les règles de la
//! feuille ([`super::feuille`]).
//!
//! Trois sortes de nœuds seulement : le symbole, le `span`, l'empilement (`vlist`). Ce qu'un
//! `span` fait de ses enfants dépend de ses classes — une suite, un filet, une forme, un
//! recouvrement — et la largeur qu'il rend est celle qu'un navigateur lui donnerait.

use super::chemin::{self, Forme};
use super::feuille::{self, em, porte, Portion};
use super::{Align, Avance, Etat, MathItem};
use katex::dom_tree::{HtmlDomNode, Span, SvgChildNode, SvgNode, SymbolNode};
use katex::types::CssProperty;

/// Ce qui attend la largeur de son empilement pour être posé : un filet, ou une forme qui
/// occupe (une part de) son conteneur — `width: 100%` n'a de sens qu'une fois le conteneur connu.
struct Attente {
    rang: usize,
    portion: Portion,
    /// La largeur que la feuille garantit quoi qu'il arrive (`min-width`).
    minimum: f64,
}

/// Un étage posé d'un empilement : ses éléments, sa largeur, et les marges de sa boîte.
struct Etage {
    elements: std::ops::Range<usize>,
    largeur: f64,
    marges: (f64, f64),
}

/// L'état du parcours : ce qui a été posé, et ce qui attend.
pub(super) struct Pont<'a> {
    ctx: &'a katex::KatexContext,
    avance: Avance<'a>,
    items: Vec<MathItem>,
    /// Pour chaque élément posé : est-il transparent ? Il occupe sa place, il ne se dessine pas.
    transparents: Vec<bool>,
    /// Le parcours est-il sous un nœud transparent (`\phantom`) ?
    sous_transparent: bool,
    attentes: Vec<Attente>,
}

impl<'a> Pont<'a> {
    pub(super) fn nouveau(ctx: &'a katex::KatexContext, avance: Avance<'a>) -> Self {
        Self {
            ctx,
            avance,
            items: Vec::new(),
            transparents: Vec::new(),
            sous_transparent: false,
            attentes: Vec::new(),
        }
    }

    /// Ce qui a été posé et se dessine. Ce qui attendait encore un conteneur — une forme hors de
    /// tout empilement — prend sa largeur garantie.
    pub(super) fn fin(mut self) -> Vec<MathItem> {
        for a in std::mem::take(&mut self.attentes) {
            let x = self.items[a.rang].x();
            self.resoudre(&a, (x, a.minimum));
        }
        self.items
            .into_iter()
            .zip(self.transparents)
            .filter_map(|(item, transparent)| (!transparent).then_some(item))
            .collect()
    }

    /// Pose un élément, transparent si le parcours l'est.
    fn pousse(&mut self, item: MathItem) {
        self.items.push(item);
        self.transparents.push(self.sous_transparent);
    }

    /// Pose un nœud à `(x, y)` et rend la largeur qu'il occupe, en `em`.
    ///
    /// Un nœud `color: transparent` — tout ce que `\phantom` contient — occupe sa place et ne se
    /// dessine pas : la couleur s'hérite en CSS, la transparence donc aussi.
    pub(super) fn pose(&mut self, node: &HtmlDomNode, etat: Etat, (x, y): (f64, f64)) -> f64 {
        let style = match node {
            HtmlDomNode::Symbol(s) => Some(&s.style),
            HtmlDomNode::DomSpan(span) => Some(&span.style),
            _ => None,
        };
        let avant = self.sous_transparent;
        self.sous_transparent |= style.is_some_and(feuille::transparent);
        let largeur = self.pose_sans_couleur(node, etat, (x, y));
        self.sous_transparent = avant;
        largeur
    }

    fn pose_sans_couleur(&mut self, node: &HtmlDomNode, etat: Etat, (x, y): (f64, f64)) -> f64 {
        match node {
            HtmlDomNode::Symbol(s) => self.symbole(s, etat, (x, y)),
            HtmlDomNode::DomSpan(span) => self.span(span, etat, (x, y)),
            HtmlDomNode::Fragment(f) => self.suite(&f.children, etat, (x, y)),
            HtmlDomNode::Anchor(a) => self.suite(&a.children, etat, (x, y)),
            // MathML ne porte aucune position ; un SVG se pose par le `span` qui le contient,
            // seul à connaître sa boîte ; une image n'existe pas dans une formule de Glucose.
            HtmlDomNode::MathML(_) | HtmlDomNode::Img(_) | HtmlDomNode::SvgNode(_) => 0.0,
        }
    }

    /// Des enfants posés les uns après les autres.
    fn suite(&mut self, enfants: &[HtmlDomNode], etat: Etat, (x, y): (f64, f64)) -> f64 {
        let mut avance = 0.0;
        for enfant in enfants {
            avance += self.pose(enfant, etat, (x + avance, y));
        }
        avance
    }

    /// **Un symbole : un glyphe par caractère**, chacun à la largeur que les métriques de KaTeX
    /// lui donnent.
    ///
    /// KaTeX fusionne les lettres voisines d'un même style en un seul symbole — « nj », « or » —
    /// et ne garde que la largeur **de la première** : le navigateur, qui mesure lui-même le
    /// texte, ne la lit jamais. Le pont la lisait : « bonjours » posait son « o » sous le « j »,
    /// « fjord » son « d » sur le « r » (ses captures du 26/09). La largeur se mesure donc
    /// caractère par caractère, dans la fonte que le dessin emploiera ([`super::layout_avec`]) ;
    /// à défaut, par les métriques de KaTeX.
    ///
    /// La marge d'un symbole est la correction d'italique que KaTeX écrit sur lui (`margin-right`
    /// d'un `j`, d'un `∫`) : sans elle, les bornes d'une intégrale se posaient sur le signe.
    fn symbole(&mut self, s: &SymbolNode, etat: Etat, (x, y): (f64, f64)) -> f64 {
        if s.text.is_empty() || s.text == "\u{200b}" {
            // L'espace de largeur nulle sert au calage d'une `vlist` en CSS.
            return 0.0;
        }
        let etat = feuille::applique_classes(&s.classes, etat);
        let (gauche, droite) = feuille::marges(&s.style);
        let (gauche, droite) = (gauche * etat.size, droite * etat.size);
        // La correction d'italique devient une marge droite au moment où KaTeX écrit le symbole
        // (`SymbolNode::toNode`) : elle est dans son champ, pas dans son style.
        let droite = droite + s.italic.max(0.0) * etat.size;
        let (dx, dy) = decalage(&s.style, etat.size);
        let fonte = etat.family.nom_de_fonte(etat.style);
        let largeurs: Option<Vec<f64>> = s
            .text
            .chars()
            .map(|c| {
                (self.avance)(c, etat.family, etat.style).or_else(|| self.largeur_de(c, &fonte))
            })
            .collect();
        let n = s.text.chars().count() as f64;
        let mut avance = 0.0;
        for (rang, c) in s.text.chars().enumerate() {
            self.pousse(MathItem::Glyph {
                text: c.to_string(),
                x: x + gauche + dx + avance,
                y: y + dy,
                size: etat.size,
                family: etat.family,
                style: etat.style,
            });
            // Sans métriques pour l'un d'eux, la largeur de KaTeX se partage : c'est la seule
            // qu'on ait, et elle est juste pour un symbole d'un seul caractère.
            let w = largeurs.as_ref().map_or(s.width / n, |l| l[rang]);
            avance += w * etat.size;
        }
        gauche + avance + droite
    }

    /// La largeur d'un caractère dans cette fonte, en `em`, selon les métriques de KaTeX.
    fn largeur_de(&self, c: char, fonte: &str) -> Option<f64> {
        katex::get_character_metrics(self.ctx, c, fonte, katex::symbols::Mode::Math)
            .ok()
            .flatten()
            .map(|m| m.width)
    }

    /// **Un `span`**, selon ce que ses classes et son style en font.
    fn span(&mut self, span: &Span<HtmlDomNode>, etat: Etat, (x, y): (f64, f64)) -> f64 {
        let etat = feuille::applique_classes(&span.classes, etat);
        let size = etat.size;
        let (mg, md, _, _) = feuille::marges_de_classe(&span.classes);
        let (sg, sd) = feuille::marges(&span.style);
        let (gauche, droite) = ((sg + mg) * size, (sd + md) * size);
        let (dx, dy) = decalage(&span.style, size);
        let (x0, y0) = (x + gauche + dx, y + dy);

        if feuille::est_un_filet(&span.classes) {
            let epaisseur = em(span.style.get(CssProperty::BorderBottomWidth)) * size;
            self.attend(
                MathItem::Rule {
                    x: x0,
                    y: y0,
                    width: 0.0,
                    height: epaisseur.max(f64::MIN_POSITIVE),
                },
                (Portion::PLEINE, 0.0),
            );
            return gauche + droite;
        }
        if porte(&span.classes, "vlist") {
            return gauche + self.empile(span, etat, (x0, y0)) + droite;
        }
        // Les rangées d'une `vlist-t` sont un tableau : elles s'empilent, elles ne se suivent
        // pas — la plus large fait la largeur.
        if porte(&span.classes, "vlist-t") {
            let mut largeur = 0.0_f64;
            for rangee in &span.children {
                largeur = largeur.max(self.pose(rangee, etat, (x0, y0)));
            }
            return gauche + largeur + droite;
        }
        let minimum = em(span.style.get(CssProperty::MinWidth)) * size;
        if let Some(largeur) = self.formes(span, size, (x0, y0), minimum) {
            return gauche + largeur + droite;
        }
        // Une forme en plusieurs morceaux, ou un cadre : chacun se pose dans sa part du
        // conteneur, à la même origine, et l'ensemble occupe au moins sa largeur garantie.
        if porte(&span.classes, "stretchy") {
            for morceau in &span.children {
                self.pose(morceau, etat, (x0, y0));
            }
            self.bordures(span, size, (x0, y0), None);
            return gauche + minimum + droite;
        }
        if let Some(recul) = feuille::recul_d_un_recouvrement(&span.classes) {
            let debut = self.items.len();
            let w = self.suite(&span.children, etat, (x0, y0));
            for item in &mut self.items[debut..] {
                decale(item, -recul * w);
            }
            return gauche + droite;
        }
        gauche + self.boite(span, etat, (x0, y0)) + droite
    }

    /// **Une boîte ordinaire** : son bourrage, ses enfants à la suite, ses bordures — et la
    /// largeur que le style ou la feuille lui imposent, s'ils en imposent une.
    fn boite(&mut self, span: &Span<HtmlDomNode>, etat: Etat, (x, y): (f64, f64)) -> f64 {
        let size = etat.size;
        let (_, _, bg, bd) = feuille::marges_de_classe(&span.classes);
        let (bourrage_g, bourrage_d) = (
            bg * size + em(span.style.get(CssProperty::PaddingLeft)) * size,
            // KaTeX n'écrit jamais de bourrage droit en ligne : seule la feuille en donne.
            bd * size,
        );
        let ([_, _, _, bord_g], _) = feuille::bordures(&span.classes, &span.style);
        let debut = x + bord_g * size + bourrage_g;
        let contenu = self.suite(&span.children, etat, (debut, y));
        let largeur = feuille::largeur(&span.style)
            .or_else(|| feuille::largeur_imposee(&span.classes))
            .map_or(bourrage_g + contenu + bourrage_d, |l| l * size);
        largeur + self.bordures(span, size, (x, y), Some(largeur))
    }

    /// **Dessine les bordures d'une boîte** dont le bas-gauche est en `(x, y)` : de `largeur`
    /// connue (son contenu), ou de toute la largeur de son conteneur (`None`). Rend la largeur
    /// que les bordures ajoutent à la boîte.
    ///
    /// Le haut et le bas couvrent toute la largeur ; les côtés, ce qui reste entre eux — les
    /// bordures ne se recouvrent pas, un coin ne s'assombrit pas. Un `\rule` n'est que bordure :
    /// sa bordure du haut **est** le trait.
    fn bordures(
        &mut self,
        span: &Span<HtmlDomNode>,
        size: f64,
        (x, y): (f64, f64),
        largeur: Option<f64>,
    ) -> f64 {
        let ([haut, droite, bas, gauche], boite_de_bordure) =
            feuille::bordures(&span.classes, &span.style);
        let [haut, droite, bas, gauche] = [haut, droite, bas, gauche].map(|b| b * size);
        let contenu = em(span.style.get(CssProperty::Height)) * size;
        let hauteur = if boite_de_bordure {
            contenu
        } else {
            contenu + haut + bas
        };
        let cote = |depuis_la_droite| Portion {
            debut: 0.0,
            largeur: 0.0,
            depuis_la_droite,
        };
        let entre = hauteur - haut - bas;
        let cotes = [
            (Portion::PLEINE, 0.0, bas, 0.0),
            (Portion::PLEINE, hauteur - haut, haut, 0.0),
            (cote(false), bas, entre, gauche),
            (cote(true), bas, entre, droite),
        ];
        let boite = largeur.map(|l| l + gauche + droite);
        for (portion, dy, h, minimum) in cotes {
            if h <= 0.0 || (portion != Portion::PLEINE && minimum <= 0.0) {
                continue;
            }
            let filet = MathItem::Rule {
                x,
                y: y + dy,
                width: 0.0,
                height: h,
            };
            match boite {
                None => self.attend(filet, (portion, minimum)),
                Some(l) => {
                    let a = Attente {
                        rang: self.items.len(),
                        portion,
                        minimum,
                    };
                    self.pousse(filet);
                    self.resoudre(&a, (x, l));
                }
            }
        }
        if boite_de_bordure {
            0.0
        } else {
            gauche + droite
        }
    }

    /// **Les formes SVG qu'un `span` contient**, s'il en contient : chacune se pose dans la
    /// boîte du `span` (`svg { width: 100%; height: inherit }`), dans la part que ses classes
    /// lui donnent. Le `span` rend la largeur que son style écrit — un morceau de grand
    /// délimiteur —, ou sa largeur garantie, et la forme attend alors son conteneur.
    fn formes(
        &mut self,
        span: &Span<HtmlDomNode>,
        size: f64,
        (x, y): (f64, f64),
        minimum: f64,
    ) -> Option<f64> {
        let svgs: Vec<&SvgNode> = span
            .children
            .iter()
            .filter_map(|c| match c {
                HtmlDomNode::SvgNode(s) => Some(s),
                _ => None,
            })
            .collect();
        if svgs.is_empty() {
            return None;
        }
        let hauteur = em(span.style.get(CssProperty::Height)) * size;
        let portion = feuille::portion(&span.classes);
        let ecrite = feuille::largeur(&span.style).map(|l| l * size);
        for svg in svgs {
            for (nom, forme) in formes_du_svg(svg, size) {
                let item = MathItem::Path {
                    name: nom,
                    x,
                    y,
                    width: ecrite.unwrap_or(minimum),
                    height: hauteur,
                    forme,
                };
                match ecrite {
                    Some(_) => self.pousse(item),
                    None => self.attend(item, (portion, minimum)),
                }
            }
        }
        Some(ecrite.unwrap_or(minimum))
    }

    /// Pose un élément dont la largeur attend celle de son conteneur.
    fn attend(&mut self, item: MathItem, (portion, minimum): (Portion, f64)) {
        self.attentes.push(Attente {
            rang: self.items.len(),
            portion,
            minimum,
        });
        self.pousse(item);
    }

    /// Donne à un élément en attente sa place dans un conteneur `(x, largeur)`.
    fn resoudre(&mut self, a: &Attente, (x, largeur): (f64, f64)) {
        let p = a.portion;
        let w = (p.largeur * largeur).max(a.minimum);
        let debut = if p.depuis_la_droite {
            x + largeur - w - p.debut * largeur
        } else {
            x + p.debut * largeur
        };
        match &mut self.items[a.rang] {
            MathItem::Rule { x, width, .. } | MathItem::Path { x, width, .. } => {
                *x = debut;
                *width = w;
            }
            MathItem::Glyph { .. } => {}
        }
    }

    /// **Pose les étages d'une `vlist`** : ils se superposent au lieu de se suivre.
    ///
    /// Chaque étage est un `span` de hauteur nulle portant `top`, dont le premier enfant est un
    /// `pstrut` de hauteur connue. La ligne de base du contenu tombe à `y = −top − pstrut`. La
    /// largeur de l'empilement est celle de son étage le plus large — marges comprises, dont la
    /// marge droite d'un exposant (`\scriptspace`, 0,05 em), que le pont oubliait : le signe qui
    /// suit `e^{i\pi}` collait au π.
    fn empile(&mut self, span: &Span<HtmlDomNode>, etat: Etat, (x, y): (f64, f64)) -> f64 {
        let mut etages: Vec<Etage> = Vec::new();
        for enfant in &span.children {
            let debut = self.items.len();
            let HtmlDomNode::DomSpan(boite) = enfant else {
                let w = self.pose(enfant, etat, (x, y));
                etages.push(Etage {
                    elements: debut..self.items.len(),
                    largeur: w,
                    marges: (0.0, 0.0),
                });
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
            let etat_boite = feuille::applique_classes(&boite.classes, etat);
            let (gauche, droite) = feuille::marges(&boite.style);
            let (gauche, droite) = (gauche * etat_boite.size, droite * etat_boite.size);
            let mut avance = 0.0;
            for petit in &boite.children {
                if matches!(petit, HtmlDomNode::DomSpan(s) if s.classes.contains("pstrut")) {
                    continue;
                }
                avance += self.pose(petit, etat_boite, (x + gauche + avance, ligne));
            }
            etages.push(Etage {
                elements: debut..self.items.len(),
                largeur: gauche + avance + droite,
                marges: (gauche, droite),
            });
        }
        let largeur = etages.iter().map(|e| e.largeur).fold(0.0, f64::max);
        self.caler(&etages, etat.align, (x, largeur));
        largeur
    }

    /// Cale les étages d'un empilement de `largeur` posé en `x`, et donne leur place à ce qui
    /// les attendait. Un étage qui porte un élément de toute la largeur — un filet, une forme — est
    /// **plein** (`width: 100%`) : le centrer ne le déplace pas.
    ///
    /// Un étage est un bloc : sa boîte commence après sa marge gauche et va jusqu'à sa marge
    /// droite. C'est dans cette boîte que ses formes se posent — l'étage d'un accent large porte
    /// l'inclinaison de sa lettre en marge gauche, et le chapeau de `\widehat{A}` se décale
    /// d'autant.
    fn caler(&mut self, etages: &[Etage], align: Align, (x, largeur): (f64, f64)) {
        for etage in etages {
            let attentes: Vec<Attente> = {
                let (dans, hors): (Vec<Attente>, Vec<Attente>) = std::mem::take(&mut self.attentes)
                    .into_iter()
                    .partition(|a| etage.elements.contains(&a.rang));
                self.attentes = hors;
                dans
            };
            let plein = !attentes.is_empty();
            let decalage = match align {
                _ if plein => 0.0,
                Align::Left => 0.0,
                Align::Center => (largeur - etage.largeur) / 2.0,
                Align::Right => largeur - etage.largeur,
            };
            for item in &mut self.items[etage.elements.clone()] {
                decale(item, decalage);
            }
            let (gauche, droite) = etage.marges;
            for a in &attentes {
                self.resoudre(a, (x + gauche, largeur - gauche - droite));
            }
        }
    }
}

/// Déplace un élément posé de `dx`, horizontalement.
fn decale(item: &mut MathItem, dx: f64) {
    match item {
        MathItem::Glyph { x, .. } | MathItem::Rule { x, .. } | MathItem::Path { x, .. } => *x += dx,
    }
}

/// **Le décalage qu'un positionnement relatif impose**, sans rien changer à la place que le
/// nœud occupe : `left` le pousse à droite, `top` le descend et `bottom` le monte (`y` monte
/// ici), `vertical-align` le monte. C'est ainsi que KaTeX recentre un grand opérateur
/// (`top: -0.0011em`), décale l'accent d'une lettre penchée (`left: -0.2222em`) et lève un
/// `\rule[0.5ex]` (`bottom`).
fn decalage(style: &katex::types::CssStyle, size: f64) -> (f64, f64) {
    (
        em(style.get(CssProperty::Left)) * size,
        (em(style.get(CssProperty::VerticalAlign)) - em(style.get(CssProperty::Top))
            + em(style.get(CssProperty::Bottom)))
            * size,
    )
}

/// Les formes qu'un SVG de KaTeX porte : le nom de chacun de ses tracés, et sa forme lue — un
/// `<path>` se remplit, un `<line>` se trace à l'épaisseur que son attribut écrit.
fn formes_du_svg(svg: &SvgNode, size: f64) -> Vec<(String, Option<Forme>)> {
    let vue = chemin::boite_de_vue(svg.attributes.get("viewBox").map(String::as_str));
    let (calage, ajustement) = chemin::aspect(
        svg.attributes
            .get("preserveAspectRatio")
            .map(String::as_str),
    );
    svg.children
        .iter()
        .map(|c| match c {
            SvgChildNode::Path(p) => {
                let d = p.alternate.clone().or_else(|| {
                    katex::svg_geometry::PATH_MAP
                        .get(p.path_name.as_str())
                        .map(|d| (*d).to_string())
                });
                let forme = match (d.as_deref().and_then(chemin::lire), vue) {
                    (Some(commandes), Some(boite_de_vue)) => Some(Forme {
                        commandes,
                        boite_de_vue,
                        calage,
                        ajustement,
                        epaisseur: None,
                    }),
                    _ => None,
                };
                (p.path_name.clone(), forme)
            }
            SvgChildNode::Line(l) => {
                let a = |nom: &str| l.attributes.get(nom).map_or("", String::as_str);
                let epaisseur = em(Some(a("stroke-width"))) * size;
                let forme = chemin::trait_de_ligne([a("x1"), a("y1"), a("x2"), a("y2")], epaisseur);
                ("line".to_string(), forme)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
