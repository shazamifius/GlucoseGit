//! **Le contrôleur du tableau** (VERIFIER-1, fiche 184 ; le point 4 de `relais-rep-tableau-aveugle`,
//! 26/11) : ce qu'une IA aveugle ne voit pas, mesuré sur ce que l'œil voit. Le 26/11, la mesure se
//! faisait à la main, sur des rectangles : 33 flèches sur 89 traversaient un nœud, 4 textes se
//! chevauchaient, 2 membranes étaient « dedans à l'œil, dehors pour l'IA » — tous de Claude. Le
//! 27/11, trois réponses de Claude ont encore fait déborder leurs membranes, vues seulement par un
//! script écrit pour l'occasion.
//!
//! Trois constats, aucun seuil :
//! * **les chevauchements** : deux nœuds qui se recouvrent sans que l'un contienne l'autre — un
//!   nœud « à cheval » sur une membrane en est un ;
//! * **les appartenances contredites** : la plus petite membrane qui contient un nœud à l'œil n'est
//!   pas celle où l'IA le lit ;
//! * **les flèches qui traversent** un nœud qu'elles ne relient pas, sur leur tracé dessiné
//!   ([`crate::arrow::morceaux_in`], le contournement compris), pas sur la droite de leurs bouts.

use super::{Decrit, Refus, Sorte};
use crate::arrow::{self, trace, Noeuds};
use crate::geometry::Rect;
use crate::quadtree::Noeud;
use crate::store::Store;
use crate::types::Board;

/// Ce que le contrôleur a trouvé sur un tableau ; vide, le tableau est propre.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Constat {
    /// Deux nœuds qui se recouvrent sans que l'un contienne l'autre.
    pub chevauchements: Vec<(String, String)>,
    /// Un nœud que l'œil voit dans une membrane, et l'IA dans une autre (ou dans aucune).
    pub appartenances: Vec<Appartenance>,
    /// Une flèche, et les nœuds qu'elle traverse sans les relier.
    pub traversees: Vec<(String, Vec<String>)>,
}

impl Constat {
    pub fn est_propre(&self) -> bool {
        self.chevauchements.is_empty()
            && self.appartenances.is_empty()
            && self.traversees.is_empty()
    }
}

/// Où un nœud est, pour l'œil et pour l'IA.
#[derive(Debug, Clone, PartialEq)]
pub struct Appartenance {
    pub id: String,
    /// La plus petite membrane qui contient sa boîte entière.
    pub a_l_oeil: Option<String>,
    /// Celle que le document lui donne.
    pub pour_l_ia: Option<String>,
}

/// La précision du tracé d'une courbe remplacée par une ligne brisée, en unités du monde : bien
/// sous l'écart que tout ce qu'une IA pose garde avec ses voisins.
const TOLERANCE: f64 = 1.0;

impl Store {
    /// **Contrôle le tableau `board_id`** : ses chevauchements, ses appartenances contredites, ses
    /// flèches qui traversent.
    pub fn verifier_le_tableau(&self, board_id: &str) -> Result<Constat, Refus> {
        let decrits = self.decrire_pour_un_agent(board_id)?;
        let noeuds: Vec<&Decrit> = (decrits.iter())
            .filter(|d| d.sorte != Sorte::Fleche)
            .collect();
        let board = (self.project.boards.iter())
            .find(|b| b.id == board_id)
            .ok_or_else(|| Refus::TableauInconnu(board_id.to_string()))?;
        let mut constat = Constat {
            chevauchements: chevauchements(&noeuds),
            appartenances: appartenances(&noeuds),
            traversees: Vec::new(),
        };
        let regard = Regard::de(board);
        for fleche in (decrits.iter()).filter(|d| d.sorte == Sorte::Fleche) {
            let Some(ann) = board.annotations.iter().find(|a| a.id() == fleche.id) else {
                continue;
            };
            let Some(morceaux) = arrow::morceaux_with(ann, &regard as &dyn Noeuds) else {
                continue;
            };
            let brisee = trace::aplatir(&morceaux, TOLERANCE);
            let traverses = traverses(&brisee, &noeuds);
            if !traverses.is_empty() {
                constat.traversees.push((fleche.id.clone(), traverses));
            }
        }
        Ok(constat)
    }
}

/// **Le tableau tel que le dessin le voit** pour tracer une flèche : les boîtes de ses nœuds, et
/// ce qu'elle contourne ([`arrow::obstacle`], la règle du dessin). Sans obstacles, le tracé du
/// noyau allait droit là où l'écran contourne, et le contrôleur aurait compté des traversées que
/// personne ne voit. Une flèche ancrée à un passage d'un texte part, ici, de son bord : la
/// hauteur du passage demande la typographie, qui vit au bureau.
struct Regard<'a> {
    board: &'a Board,
    obstacles: Vec<Rect>,
}

impl<'a> Regard<'a> {
    fn de(board: &'a Board) -> Self {
        let noeuds = (board.images.iter().map(Noeud::Image))
            .chain(board.annotations.iter().map(Noeud::Annotation))
            .chain(board.folders.iter().map(Noeud::Dossier));
        Self {
            board,
            obstacles: noeuds.filter_map(arrow::obstacle).collect(),
        }
    }
}

impl Noeuds for Regard<'_> {
    fn boite(&self, id: &str) -> Option<Rect> {
        arrow::node_rect(self.board, id)
    }

    fn obstacles(&self, zone: Rect, sortie: &mut Vec<Rect>) {
        // Bords compris, comme le dessin (`renderer/arrow/noeuds.rs`).
        let dans = |b: &&Rect| {
            b.left <= zone.right()
                && zone.left <= b.right()
                && b.top <= zone.bottom()
                && zone.top <= b.bottom()
        };
        sortie.extend(self.obstacles.iter().filter(dans));
    }
}

/// Les paires qui se recouvrent sans que l'une contienne l'autre, chacune une fois.
fn chevauchements(noeuds: &[&Decrit]) -> Vec<(String, String)> {
    let mut paires = Vec::new();
    for (i, a) in noeuds.iter().enumerate() {
        for b in &noeuds[i + 1..] {
            let (p, q) = (a.boite, b.boite);
            if p.overlaps(q) && !p.contains_rect(q) && !q.contains_rect(p) {
                paires.push((a.id.clone(), b.id.clone()));
            }
        }
    }
    paires
}

/// Les nœuds dont la plus petite membrane qui les contient n'est pas celle du document.
fn appartenances(noeuds: &[&Decrit]) -> Vec<Appartenance> {
    let membranes: Vec<&&Decrit> = (noeuds.iter())
        .filter(|d| d.sorte == Sorte::Membrane)
        .collect();
    let mut contredites = Vec::new();
    for n in noeuds {
        let a_l_oeil = (membranes.iter())
            .filter(|m| {
                m.id != n.id && m.boite.contains_rect(n.boite) && !n.boite.contains_rect(m.boite)
            })
            .min_by(|a, b| a.boite.area().total_cmp(&b.boite.area()))
            .map(|m| m.id.clone());
        if a_l_oeil != n.membrane {
            contredites.push(Appartenance {
                id: n.id.clone(),
                a_l_oeil,
                pour_l_ia: n.membrane.clone(),
            });
        }
    }
    contredites
}

/// Les nœuds où entre la ligne brisée d'une flèche — ceux qu'elle aurait dû contourner : un
/// texte, un post-it, une photo, un dossier ; une membrane se traverse pour relier deux domaines.
/// **Aucune exclusion** : une flèche part du bord de ses bouts et ne les traverse donc jamais…
/// sauf quand son tracé revient sur son propre texte, ce que l'œil voit et qu'il faut dire (la
/// boucle de « URGENT IMPORTANT », fiche 152).
fn traverses(brisee: &[trace::Point], noeuds: &[&Decrit]) -> Vec<String> {
    let contourne = |n: &Decrit| {
        matches!(
            n.sorte,
            Sorte::Texte | Sorte::PostIt | Sorte::Photo | Sorte::Dossier
        )
    };
    (noeuds.iter())
        .filter(|n| contourne(n))
        .filter(|n| {
            brisee
                .windows(2)
                .any(|s| segment_traverse(s[0], s[1], n.boite))
        })
        .map(|n| n.id.clone())
        .collect()
}

/// **Le segment `a`–`b` entre-t-il dans l'intérieur de `r` ?** L'algorithme de Liang et Barsky :
/// le segment paramétré `a + t·(b − a)`, `t` dans `[0, 1]`, rogné par les quatre bords ; il reste
/// un morceau de longueur non nulle s'il traverse. Longer un bord n'est pas traverser.
fn segment_traverse(a: trace::Point, b: trace::Point, r: Rect) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, a.0 - r.left),
        (dx, r.right() - a.0),
        (-dy, a.1 - r.top),
        (dy, r.bottom() - a.1),
    ] {
        if p == 0.0 {
            if q <= 0.0 {
                return false; // parallèle à ce bord, et dehors ou dessus
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    t0 < t1
}

#[cfg(test)]
#[path = "verifier_tests.rs"]
mod tests;
