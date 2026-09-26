//! **Les obstacles connus, rangés par cases** : un test ne regarde que ce que le segment longe.
//!
//! Sans elle, chaque test de visibilité parcourait tous les obstacles connus. Une flèche qui
//! longe une colonne de deux cents cartes en connaît deux cents ; chacun des milliers de tests
//! de sa recherche les parcourait tous, et elle épuisait son budget pour finir en ligne droite.
//! Rangés par cases, un test marche de case en case le long du segment (Amanatides et Woo,
//! 1987) et s'arrête au premier obstacle qu'il traverse : un segment bloqué tout près de son
//! départ coûte une case.
//!
//! La maille est la taille moyenne des obstacles connus : une boîte tient dans quelques cases,
//! une case en porte quelques-unes. Une boîte qui couvrirait plus de cases qu'il n'y a de boîtes
//! coûterait plus à ranger qu'à tester : elle reste à part, testée à chaque fois.

use super::{Boite, Point};
use std::collections::HashMap;

pub(super) struct Grille {
    cote: f64,
    cases: HashMap<(i64, i64), Vec<usize>>,
    /// Les boîtes trop grandes pour être rangées.
    grandes: Vec<usize>,
    /// Le tour auquel chaque boîte a été testée : une boîte à cheval sur plusieurs cases ne se
    /// teste qu'une fois par segment.
    vues: Vec<u32>,
    tour: u32,
}

impl Grille {
    pub(super) fn de(boites: &[Boite]) -> Self {
        let n = boites.len();
        let cote = if n == 0 {
            1.0
        } else {
            boites
                .iter()
                .map(|b| (b.x1 - b.x0).max(b.y1 - b.y0))
                .sum::<f64>()
                / n as f64
        };
        let mut grille = Self {
            cote,
            cases: HashMap::new(),
            grandes: Vec::new(),
            vues: vec![0; n],
            tour: 0,
        };
        for (k, b) in boites.iter().enumerate() {
            let (i0, j0) = grille.case((b.x0, b.y0));
            let (i1, j1) = grille.case((b.x1, b.y1));
            let couvertes = (i1 - i0 + 1).saturating_mul(j1 - j0 + 1);
            if couvertes > n as i64 {
                grille.grandes.push(k);
                continue;
            }
            for i in i0..=i1 {
                for j in j0..=j1 {
                    grille.cases.entry((i, j)).or_default().push(k);
                }
            }
        }
        grille
    }

    fn case(&self, p: Point) -> (i64, i64) {
        (
            (p.0 / self.cote).floor() as i64,
            (p.1 / self.cote).floor() as i64,
        )
    }

    /// Le point est-il strictement dans l'une des boîtes ?
    pub(super) fn contient(&self, boites: &[Boite], p: Point) -> bool {
        let dans_la_case = self.cases.get(&self.case(p)).into_iter().flatten();
        self.grandes
            .iter()
            .chain(dans_la_case)
            .any(|&k| boites[k].contient(p))
    }

    /// **Le segment traverse-t-il l'une des boîtes ?** Case par case, du départ vers
    /// l'arrivée ; `travail` compte les boîtes testées.
    pub(super) fn traversee(
        &mut self,
        boites: &[Boite],
        (p, q): (Point, Point),
        travail: &mut usize,
    ) -> bool {
        self.tour = self.tour.wrapping_add(1);
        *travail += self.grandes.len() + 1;
        if self.grandes.iter().any(|&k| boites[k].traversee(p, q)) {
            return true;
        }
        let c = self.cote;
        let (dx, dy) = (q.0 - p.0, q.1 - p.1);
        let pas = |d: f64| i64::from(d > 0.0) - i64::from(d < 0.0);
        let (si, sj) = (pas(dx), pas(dy));
        let (mut i, mut j) = self.case(p);
        // Le paramètre auquel le segment quitte la case courante, sur chaque axe, et ce qu'il
        // faut pour en traverser une entière.
        let sortie = |cellule: i64, s: i64, depart: f64, d: f64| {
            if s == 0 {
                f64::INFINITY
            } else {
                ((cellule + i64::from(s > 0)) as f64 * c - depart) / d
            }
        };
        let (mut tx, mut ty) = (sortie(i, si, p.0, dx), sortie(j, sj, p.1, dy));
        let traversee_entiere = |s: i64, d: f64| if s == 0 { f64::INFINITY } else { c / d.abs() };
        let (px, py) = (traversee_entiere(si, dx), traversee_entiere(sj, dy));
        loop {
            if self.case_traversee(boites, (i, j), (p, q), travail) {
                return true;
            }
            // Le segment finit dans cette case : il n'en quitte aucune autre.
            if tx.min(ty) > 1.0 {
                return false;
            }
            if tx < ty {
                i += si;
                tx += px;
            } else {
                j += sj;
                ty += py;
            }
        }
    }

    fn case_traversee(
        &mut self,
        boites: &[Boite],
        case: (i64, i64),
        (p, q): (Point, Point),
        travail: &mut usize,
    ) -> bool {
        let Some(dedans) = self.cases.get(&case) else {
            return false;
        };
        for &k in dedans {
            if self.vues[k] == self.tour {
                continue;
            }
            self.vues[k] = self.tour;
            *travail += 1;
            if boites[k].traversee(p, q) {
                return true;
            }
        }
        false
    }
}
