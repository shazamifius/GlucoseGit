//! **Ce qui a changé dans un texte** (MCP-CHANGE, fiche 180 ; `relais-rep-mcp`, point 6 :
//! *« relire seulement ce qui a changé dans un texte, pas le texte entier »*).
//!
//! Les lignes communes aux deux versions se trouvent par le **« patience diff »** de Bram Cohen
//! (`git diff --patience` ; blog.jcoglan.com/2017/09/19/the-patience-diff-algorithm, lu le 26/11) :
//! les débuts et les fins communs s'apparient ; puis les lignes qui ne paraissent qu'une fois dans
//! chaque version, dans le plus long ordre qu'elles gardent toutes deux ; puis la même chose entre
//! ces ancres. Une région sans ligne unique reste un seul changement (git y repasse par Myers) :
//! un texte de Glucose y perd peu, et le chemin reste en O(n log n), sans table quadratique — il
//! tourne dans la boucle qui dessine.

use std::collections::HashMap;
use std::ops::Range;

/// **Un changement** : à la ligne `ligne` (comptée de 1) du texte d'après, ces lignes d'avant
/// sont devenues celles-ci. `None` : aucune ligne — `Some("")`, une ligne vide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changement {
    pub ligne: usize,
    pub avant: Option<String>,
    pub apres: Option<String>,
}

/// **Les changements d'`avant` à `apres`**, dans l'ordre ; `None` quand ils ne sont pas plus
/// courts que le texte d'après : mieux vaut alors le relire entier.
pub fn changements(avant: &str, apres: &str) -> Option<Vec<Changement>> {
    let rendus = tous(avant, apres);
    let taille = |t: &Option<String>| t.as_ref().map_or(0, String::len);
    let poids: usize = rendus
        .iter()
        .map(|c| taille(&c.avant) + taille(&c.apres))
        .sum();
    (poids < apres.len()).then_some(rendus)
}

/// **Tous les changements**, si longs soient-ils.
fn tous(avant: &str, apres: &str) -> Vec<Changement> {
    let a: Vec<&str> = avant.split('\n').collect();
    let b: Vec<&str> = apres.split('\n').collect();
    let mut paires = apparier(&a, &b);
    paires.sort_unstable();
    let mut rendus = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (x, y) in paires.into_iter().chain([(a.len(), b.len())]) {
        if x > i || y > j {
            let lignes = |l: &[&str]| (!l.is_empty()).then(|| l.join("\n"));
            rendus.push(Changement {
                ligne: j + 1,
                avant: lignes(&a[i..x]),
                apres: lignes(&b[j..y]),
            });
        }
        (i, j) = (x + 1, y + 1);
    }
    rendus
}

/// **Les lignes communes**, en paires d'indices (dans `a`, dans `b`), sans ordre. Une pile de
/// régions plutôt qu'une récursion : un long texte ne creuse pas la pile du fil.
fn apparier(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let mut paires = Vec::new();
    let mut regions: Vec<(Range<usize>, Range<usize>)> = vec![(0..a.len(), 0..b.len())];
    while let Some((mut ra, mut rb)) = regions.pop() {
        while !ra.is_empty() && !rb.is_empty() && a[ra.start] == b[rb.start] {
            paires.push((ra.start, rb.start));
            (ra.start, rb.start) = (ra.start + 1, rb.start + 1);
        }
        while !ra.is_empty() && !rb.is_empty() && a[ra.end - 1] == b[rb.end - 1] {
            (ra.end, rb.end) = (ra.end - 1, rb.end - 1);
            paires.push((ra.end, rb.end));
        }
        let (mut i, mut j) = (ra.start, rb.start);
        for (x, y) in ancres(&a[ra.clone()], &b[rb.clone()]) {
            let (x, y) = (ra.start + x, rb.start + y);
            paires.push((x, y));
            regions.push((i..x, j..y));
            (i, j) = (x + 1, y + 1);
        }
        // Sans ancre, la région est un seul changement ; avec, ce qui suit la dernière.
        if i != ra.start {
            regions.push((i..ra.end, j..rb.end));
        }
    }
    paires
}

/// **Les ancres** : les lignes qui ne paraissent qu'une fois dans `a` et une fois dans `b`, dans le
/// plus long ordre qu'elles gardent toutes deux ; leurs indices.
fn ancres(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let mut vues: HashMap<&str, [(u32, usize); 2]> = HashMap::new();
    for (cote, lignes) in [a, b].into_iter().enumerate() {
        for (k, &ligne) in lignes.iter().enumerate() {
            let v = &mut vues.entry(ligne).or_default()[cote];
            *v = (v.0 + 1, k);
        }
    }
    let mut uniques: Vec<(usize, usize)> = (vues.into_values())
        .filter(|[x, y]| x.0 == 1 && y.0 == 1)
        .map(|[x, y]| (x.1, y.1))
        .collect();
    uniques.sort_unstable();
    plus_longue_croissante(&uniques)
}

/// **La plus longue sous-suite dont les seconds indices croissent**, les premiers croissant déjà :
/// le tri de patience, chaque paire posée sur la première pile dont le sommet la dépasse.
fn plus_longue_croissante(paires: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut sommets: Vec<usize> = Vec::new();
    let mut dessous = vec![None; paires.len()];
    for (k, &(_, y)) in paires.iter().enumerate() {
        let p = sommets.partition_point(|&s| paires[s].1 < y);
        dessous[k] = p.checked_sub(1).map(|q| sommets[q]);
        if p == sommets.len() {
            sommets.push(k);
        } else {
            sommets[p] = k;
        }
    }
    let mut suite = Vec::with_capacity(sommets.len());
    let mut k = sommets.last().copied();
    while let Some(i) = k {
        suite.push(paires[i]);
        k = dessous[i];
    }
    suite.reverse();
    suite
}

#[cfg(test)]
mod tests;
