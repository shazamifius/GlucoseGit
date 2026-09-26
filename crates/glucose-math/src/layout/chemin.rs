//! **Les formes de KaTeX, lues** : ses chemins SVG en commandes, et la façon dont ils se
//! posent dans leur boîte.
//!
//! # Ce qui manquait
//!
//! KaTeX dessine en SVG ce qui n'a pas de glyphe à la bonne taille : le radical d'une racine,
//! les flèches longues, les accolades horizontales, les chapeaux larges. Le pont ne transmettait
//! que le **nom** de la forme ; le dessin construisait un radical à la main — trois segments,
//! reconnaissables de loin et faux de près — et ne dessinait **aucune** des autres formes.
//!
//! Le tracé vrai est pourtant là : katex-rs porte le chemin de chaque forme, dans le vocabulaire
//! d'un attribut `d` de SVG, avec la boîte de vue (`viewBox`) et la règle d'ajustement
//! (`preserveAspectRatio`) de son élément. Ce module les lit ; celui qui dessine n'a plus qu'à
//! les suivre.
//!
//! # Le vocabulaire est fermé
//!
//! Tous les chemins de KaTeX s'écrivent avec treize commandes : `M L H V C S Z`, et leurs formes
//! relatives. Le lecteur les connaît toutes, et **refuse** tout le reste plutôt que de deviner :
//! une forme illisible ne se dessine pas, elle ne se dessine pas de travers.

/// Un pas d'un chemin, en unités de sa boîte de vue — `y` **vers le bas**, comme en SVG.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Commande {
    Aller(f64, f64),
    Ligne(f64, f64),
    /// Une cubique : deux points de contrôle, puis l'arrivée.
    Cubique(f64, f64, f64, f64, f64, f64),
    Fermer,
}

/// Où la boîte de vue se cale dans la boîte de la forme, sur un axe — `xMin`, `xMid`, `xMax`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Calage {
    #[default]
    Min,
    Mid,
    Max,
}

/// Comment la boîte de vue s'ajuste à la boîte de la forme (`preserveAspectRatio`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ajustement {
    /// `none` : chaque axe s'étire à sa mesure — un chapeau large suit sa formule.
    Etire,
    /// `meet` : l'échelle commune la plus petite, tout tient dedans.
    #[default]
    Contient,
    /// `slice` : l'échelle commune la plus grande, et ce qui déborde est rogné — c'est ainsi
    /// qu'un radical de 400 000 unités de long ne montre que la longueur de son contenu.
    Couvre,
}

/// **Une forme de KaTeX**, prête à poser : son tracé, sa boîte de vue, son ajustement.
#[derive(Debug, Clone, PartialEq)]
pub struct Forme {
    pub commandes: Vec<Commande>,
    /// `(x, y, largeur, hauteur)`, en unités du tracé.
    pub boite_de_vue: (f64, f64, f64, f64),
    pub calage: (Calage, Calage),
    pub ajustement: Ajustement,
    /// `Some(e)` : la forme est un **trait** d'épaisseur `e` (en `em` de la formule), tracé et
    /// non rempli — le `<line>` des ratures de `\cancel`. Son épaisseur ne suit pas la boîte :
    /// un SVG sans boîte de vue la compte en unités de la page.
    pub epaisseur: Option<f64>,
}

impl Forme {
    /// **La transformation du tracé vers une boîte de `(largeur, hauteur)`**, origine en haut à
    /// gauche de la boîte : `(échelle en x, échelle en y, décalage en x, décalage en y)` — la
    /// règle de `preserveAspectRatio`, telle que la norme SVG l'écrit.
    pub fn transformation(&self, (largeur, hauteur): (f64, f64)) -> (f64, f64, f64, f64) {
        let (vx, vy, vl, vh) = self.boite_de_vue;
        if vl <= 0.0 || vh <= 0.0 {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let (sx, sy) = (largeur / vl, hauteur / vh);
        let (sx, sy) = match self.ajustement {
            Ajustement::Etire => (sx, sy),
            Ajustement::Contient => (sx.min(sy), sx.min(sy)),
            Ajustement::Couvre => (sx.max(sy), sx.max(sy)),
        };
        let reste = |libre: f64, calage: Calage| match calage {
            Calage::Min => 0.0,
            Calage::Mid => libre / 2.0,
            Calage::Max => libre,
        };
        let dx = reste(largeur - vl * sx, self.calage.0) - vx * sx;
        let dy = reste(hauteur - vh * sy, self.calage.1) - vy * sy;
        (sx, sy, dx, dy)
    }

    /// **L'encre de la forme dans une boîte de `(largeur, hauteur)`**, origine en haut à
    /// gauche : `(x_min, y_min, x_max, y_max)`, rognée à la boîte comme au dessin — ou rien.
    ///
    /// Une boîte n'est pas son encre : celle d'un radical à indice commence **avant** la
    /// formule (KaTeX recule l'indice de 0,5556 em), son trait non. Les points de contrôle
    /// d'une cubique enferment la courbe : l'étendue est sûre, jamais trop courte.
    pub fn etendue(&self, (largeur, hauteur): (f64, f64)) -> Option<(f64, f64, f64, f64)> {
        let (sx, sy, dx, dy) = self.transformation((largeur, hauteur));
        let demi = self.epaisseur.unwrap_or(0.0) / 2.0;
        let mut b: Option<(f64, f64, f64, f64)> = None;
        let mut ajoute = |x: f64, y: f64| {
            let (x, y) = (dx + x * sx, dy + y * sy);
            let (x0, y0, x1, y1) = (x - demi, y - demi, x + demi, y + demi);
            b = Some(b.map_or((x0, y0, x1, y1), |(a, c, d, e)| {
                (a.min(x0), c.min(y0), d.max(x1), e.max(y1))
            }));
        };
        for c in &self.commandes {
            match *c {
                Commande::Aller(x, y) | Commande::Ligne(x, y) => ajoute(x, y),
                Commande::Cubique(a, b2, c2, d, e, f) => {
                    ajoute(a, b2);
                    ajoute(c2, d);
                    ajoute(e, f);
                }
                Commande::Fermer => {}
            }
        }
        let (x0, y0, x1, y1) = b?;
        let (x0, y0, x1, y1) = (x0.max(0.0), y0.max(0.0), x1.min(largeur), y1.min(hauteur));
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }
}

/// **Le trait d'un `<line>` de KaTeX** : ses deux bouts en fractions de la boîte (`0`, `100%`),
/// posés sur une boîte de vue unité qui s'étire à la boîte de la forme.
pub fn trait_de_ligne(bouts: [&str; 4], epaisseur: f64) -> Option<Forme> {
    let [x1, y1, x2, y2] = bouts.map(fraction);
    Some(Forme {
        commandes: vec![Commande::Aller(x1?, y1?), Commande::Ligne(x2?, y2?)],
        boite_de_vue: (0.0, 0.0, 1.0, 1.0),
        calage: (Calage::Min, Calage::Min),
        ajustement: Ajustement::Etire,
        epaisseur: Some(epaisseur),
    })
}

/// Une coordonnée de `<line>` en fraction de sa boîte : un pourcentage, ou zéro. Un nombre nu
/// serait en unités de la page ; KaTeX n'en écrit pas, et le lecteur le refuse.
fn fraction(v: &str) -> Option<f64> {
    let v = v.trim();
    match v.strip_suffix('%') {
        Some(p) => p.trim().parse::<f64>().ok().map(|p| p / 100.0),
        None => v.parse::<f64>().ok().filter(|n| *n == 0.0),
    }
}

/// Lit `preserveAspectRatio` : `none`, ou un calage `xMinYMin` suivi de `meet` ou `slice`.
pub fn aspect(valeur: Option<&str>) -> ((Calage, Calage), Ajustement) {
    let Some(v) = valeur.map(str::trim).filter(|v| !v.is_empty()) else {
        return ((Calage::Mid, Calage::Mid), Ajustement::Contient);
    };
    if v == "none" {
        return ((Calage::Min, Calage::Min), Ajustement::Etire);
    }
    let mut mots = v.split_whitespace();
    let calage = mots.next().unwrap_or("xMidYMid");
    let axe = |nom: &str| match nom {
        "Min" => Calage::Min,
        "Max" => Calage::Max,
        _ => Calage::Mid,
    };
    let (x, y) = (
        calage.get(1..4).map_or(Calage::Mid, axe),
        calage.get(5..8).map_or(Calage::Mid, axe),
    );
    let ajustement = if mots.next() == Some("slice") {
        Ajustement::Couvre
    } else {
        Ajustement::Contient
    };
    ((x, y), ajustement)
}

/// Lit une boîte de vue `"x y largeur hauteur"`.
pub fn boite_de_vue(valeur: Option<&str>) -> Option<(f64, f64, f64, f64)> {
    let n: Vec<f64> = valeur?
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match n[..] {
        [x, y, l, h] => Some((x, y, l, h)),
        _ => None,
    }
}

/// **Lit l'attribut `d` d'un chemin de KaTeX** en commandes absolues — ou rien, s'il emploie une
/// commande hors de son vocabulaire.
pub fn lire(d: &str) -> Option<Vec<Commande>> {
    let mut lecteur = Lecteur {
        octets: d.as_bytes(),
        rang: 0,
    };
    let mut sortie = Vec::new();
    let (mut x, mut y) = (0.0, 0.0);
    let (mut depart_x, mut depart_y) = (0.0, 0.0);
    // Le dernier point de contrôle d'une cubique, que `S` reflète.
    let mut controle: Option<(f64, f64)> = None;
    let mut commande = None;
    loop {
        lecteur.blancs();
        let Some(&c) = lecteur.octets.get(lecteur.rang) else {
            break;
        };
        if c.is_ascii_alphabetic() {
            lecteur.rang += 1;
            commande = Some(c);
            if c == b'Z' || c == b'z' {
                sortie.push(Commande::Fermer);
                (x, y) = (depart_x, depart_y);
                controle = None;
                continue;
            }
        }
        let c = commande?;
        let relatif = c.is_ascii_lowercase();
        let (ox, oy) = if relatif { (x, y) } else { (0.0, 0.0) };
        match c.to_ascii_uppercase() {
            b'M' => {
                let (px, py) = lecteur.paire()?;
                (x, y) = (ox + px, oy + py);
                (depart_x, depart_y) = (x, y);
                sortie.push(Commande::Aller(x, y));
                // Les paires qui suivent un `M` sont des lignes (norme SVG).
                commande = Some(if relatif { b'l' } else { b'L' });
                controle = None;
            }
            b'L' => {
                let (px, py) = lecteur.paire()?;
                (x, y) = (ox + px, oy + py);
                sortie.push(Commande::Ligne(x, y));
                controle = None;
            }
            b'H' => {
                x = ox + lecteur.nombre()?;
                sortie.push(Commande::Ligne(x, y));
                controle = None;
            }
            b'V' => {
                y = oy + lecteur.nombre()?;
                sortie.push(Commande::Ligne(x, y));
                controle = None;
            }
            b'C' => {
                let (a, b) = lecteur.paire()?;
                let (c2, d2) = lecteur.paire()?;
                let (e, f) = lecteur.paire()?;
                let (c1, c2x, fin) = ((ox + a, oy + b), (ox + c2, oy + d2), (ox + e, oy + f));
                sortie.push(Commande::Cubique(c1.0, c1.1, c2x.0, c2x.1, fin.0, fin.1));
                controle = Some(c2x);
                (x, y) = fin;
            }
            b'S' => {
                // Le premier point de contrôle est le reflet du dernier, ou le point courant.
                let c1 = controle.map_or((x, y), |(cx, cy)| (2.0 * x - cx, 2.0 * y - cy));
                let (c2, d2) = lecteur.paire()?;
                let (e, f) = lecteur.paire()?;
                let (c2x, fin) = ((ox + c2, oy + d2), (ox + e, oy + f));
                sortie.push(Commande::Cubique(c1.0, c1.1, c2x.0, c2x.1, fin.0, fin.1));
                controle = Some(c2x);
                (x, y) = fin;
            }
            _ => return None,
        }
    }
    Some(sortie)
}

/// Un curseur sur les octets d'un attribut `d`.
struct Lecteur<'a> {
    octets: &'a [u8],
    rang: usize,
}

impl Lecteur<'_> {
    fn blancs(&mut self) {
        while self
            .octets
            .get(self.rang)
            .is_some_and(|c| c.is_ascii_whitespace() || *c == b',')
        {
            self.rang += 1;
        }
    }

    /// Un nombre SVG : signe, chiffres, point, exposant. Un signe ou un second point commence le
    /// nombre suivant — `1.5.5` se lit `1.5` puis `.5`, `3-4` se lit `3` puis `-4`.
    fn nombre(&mut self) -> Option<f64> {
        self.blancs();
        let debut = self.rang;
        let mut point = false;
        let mut exposant = false;
        while let Some(&c) = self.octets.get(self.rang) {
            let signe_permis = self.rang == debut
                || (exposant && matches!(self.octets[self.rang - 1], b'e' | b'E'));
            match c {
                b'0'..=b'9' => {}
                b'+' | b'-' if signe_permis => {}
                b'.' if !point && !exposant => point = true,
                b'e' | b'E' if !exposant && self.rang > debut => exposant = true,
                _ => break,
            }
            self.rang += 1;
        }
        std::str::from_utf8(&self.octets[debut..self.rang])
            .ok()?
            .parse()
            .ok()
    }

    fn paire(&mut self) -> Option<(f64, f64)> {
        Some((self.nombre()?, self.nombre()?))
    }
}

#[cfg(test)]
mod tests;
