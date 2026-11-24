//! **TEXT-FIT-2 — un texte qui grandit pousse sa membrane** (fiche 124).
//!
//! TEXT-FIT-1 fait du texte le plancher de sa carte : quand il demande plus de hauteur — on y écrit,
//! ou une mise en page neuve le coupe autrement —, la carte grandit vers le bas. Sa membrane, elle,
//! restait où elle était. Le 19/11, TABLE-3 a coupé en lignes les tableaux que Claude.ai avait
//! posés dans l'onglet « Textes optimisés » : les textes ont grandi, et sont sortis par le bas de
//! leur membrane — celui du co-working en dépassait déjà de vingt unités.
//!
//! La loi : **le bas d'une membrane suit le bas de ce qui grandit en elle**, avec l'écart du cadre
//! (GROUPER-1) ; et celle qui la contient la suit à son tour, comme MEMB-7 le veut d'une membrane
//! qui grandit. Seul ce qui grandit pousse — un texte qui ne change pas ne touche à rien, même s'il
//! dépasse —, et seul le bas bouge : ni le titre, ni les côtés, ni les voisins (MEMB-7 ne les
//! pousse pas). L'appartenance est celle que le document écrit (`membrane_id`) : un texte qu'on
//! tire hors de sa membrane n'en est plus membre (MEMB-1, au dépôt), il ne l'étire donc jamais.
//!
//! Une loi, deux appelants : le chargement, qui ajuste chaque carte à son texte hors du journal
//! (`ajuster_les_cartes`), et la saisie, dans le geste qu'elle a ouvert ([`Store::pousser_les_membranes`]).

use super::zones::ECART;
use super::Store;
use crate::geometry::Rect;
use crate::types::{Annotation, Board};

/// **Les membranes que pousse le nœud `id`, son bas descendu à `bas`**, et le bas neuf de chacune :
/// la sienne d'abord, puis chaque membrane qui la contient, jusqu'à la première qui le contient
/// déjà. Le rang est celui de la membrane dans `annotations`.
fn membranes_poussees(annotations: &[Annotation], id: &str, bas: f64) -> Vec<(usize, f64)> {
    let mut poussees: Vec<(usize, f64)> = Vec::new();
    let mut parente = annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(Annotation::membrane_id);
    let mut bas = bas;
    while let Some(m) = parente {
        let Some(rang) = annotations
            .iter()
            .position(|a| a.id() == m && matches!(a, Annotation::Membrane { .. }))
        else {
            break;
        };
        // Un cycle de membranes ne doit pas exister ; s'il existait, il ne pousserait pas sans fin.
        if poussees.iter().any(|&(r, _)| r == rang) {
            break;
        }
        let Some(cadre) = annotations[rang].rect() else {
            break;
        };
        let voulu = bas + ECART;
        if voulu <= cadre.bottom() {
            break;
        }
        poussees.push((rang, voulu));
        bas = voulu;
        parente = annotations[rang].membrane_id();
    }
    poussees
}

/// **Les voisins que pousse le nœud `id` pendant qu'on l'écrit** (fiche 145 ; sa réponse sur
/// `relais-rep-327`) : `depart` est son tableau au début de l'écriture, `bas` son bas courant. À son
/// niveau (même membrane), ce qui était entièrement dessous et que le bas neuf atteint descend du
/// strict nécessaire — l'écart d'origine gardé, au plus `ECART` —, et pousse à son tour : un
/// balayage par haut croissant (Force Scan, vers le bas seulement), qui garde l'ordre. Une
/// fonction de l'état de départ, jamais une suite de poussées : effacer fait remonter, sans dérive.
/// Rend le rang de chaque voisin qui descend et son haut neuf.
pub fn voisins_pousses(depart: &[Annotation], id: &str, bas: f64) -> Vec<(usize, f64)> {
    let Some((cadre, niveau)) =
        (depart.iter().find(|a| a.id() == id)).and_then(|a| Some((a.rect()?, a.membrane_id())))
    else {
        return Vec::new();
    };
    let mut dessous: Vec<(usize, Rect)> = (depart.iter().enumerate())
        .filter(|(_, a)| a.id() != id && a.membrane_id() == niveau)
        .filter_map(|(rang, a)| Some((rang, a.rect()?)))
        .collect();
    dessous.sort_by(|a, b| a.1.top.total_cmp(&b.1.top));
    // Chaque pousseur : sa boîte de départ, et son bas neuf.
    let mut pousseurs: Vec<(Rect, f64)> = vec![(cadre, bas)];
    let mut pousses = Vec::new();
    for (rang, c) in dessous {
        let haut = (pousseurs.iter())
            .filter(|(p, _)| p.left < c.right() && c.left < p.right() && p.bottom() <= c.top)
            .map(|(p, bas_neuf)| bas_neuf + ECART.min(c.top - p.bottom()))
            .fold(c.top, f64::max);
        if haut > c.top {
            pousses.push((rang, haut));
        }
        pousseurs.push((c, haut + c.height));
    }
    pousses
}

/// **Ce que pousse le nœud `id` pendant qu'on l'écrit, à chaque étage** (fiche 145) : ses voisins
/// ([`voisins_pousses`]), puis sa membrane, qui grandit jusqu'au plus bas de ce qu'elle contient
/// désormais (+ `ECART`) et pousse à son tour les siens — jusqu'à la première qui le contient déjà.
/// Rend le haut neuf de chaque voisin poussé et le bas neuf de chaque membrane qui grandit, par rang.
pub fn pousser_en_ecrivant(depart: &[Annotation], id: &str, bas: f64) -> Vec<(usize, Bord, f64)> {
    let mut faits: Vec<(usize, Bord, f64)> = Vec::new();
    let (mut qui, mut bas) = (id.to_string(), bas);
    while faits.len() <= depart.len() {
        let pousses = voisins_pousses(depart, &qui, bas);
        let plus_bas = (pousses.iter())
            .filter_map(|&(r, haut)| Some(haut + depart[r].rect()?.height))
            .fold(bas, f64::max);
        faits.extend(pousses.into_iter().map(|(r, haut)| (r, Bord::Haut, haut)));
        let Some((rang, cadre)) = (depart.iter().find(|a| a.id() == qui))
            .and_then(Annotation::membrane_id)
            .and_then(|m| depart.iter().position(|a| a.id() == m))
            .and_then(|r| Some((r, depart[r].rect()?)))
        else {
            break;
        };
        let voulu = plus_bas + ECART;
        if voulu <= cadre.bottom() {
            break;
        }
        faits.push((rang, Bord::Bas, voulu));
        (qui, bas) = (depart[rang].id().to_string(), voulu);
    }
    faits
}

/// Le bord qu'un geste de [`pousser_en_ecrivant`] déplace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bord {
    /// Le nœud descend, sa taille gardée.
    Haut,
    /// La membrane grandit, son haut gardé.
    Bas,
}

/// Le bas d'une membrane posé à `bas`, son haut gardé.
fn poser_le_bas(membrane: &mut Annotation, bas: f64) {
    if let Annotation::Membrane { y, height, .. } = membrane {
        *height = bas - *y;
    }
}

/// **TEXT-FIT-2, hors du journal** : ce que fait un document qu'on lit, avant qu'il ne devienne un
/// geste — le nœud `id` de ce tableau a descendu son bas à `bas`, ses membranes le suivent.
pub fn pousser_dans_le_tableau(board: &mut Board, id: &str, bas: f64) {
    for (rang, bas) in membranes_poussees(&board.annotations, id, bas) {
        poser_le_bas(&mut board.annotations[rang], bas);
    }
}

impl Store {
    /// **TEXT-FIT-2, par le journal** : le nœud `id` vient de descendre son bas à `bas` — ses
    /// membranes le suivent, dans le geste ouvert, et le même `Ctrl+Z` les défait avec lui.
    pub fn pousser_les_membranes(&mut self, board_id: &str, id: &str, bas: f64) {
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return;
        };
        let poussees: Vec<(String, Rect)> = membranes_poussees(&b.annotations, id, bas)
            .into_iter()
            .filter_map(|(rang, bas)| {
                let m = &b.annotations[rang];
                let cadre = m.rect()?;
                Some((
                    m.id().to_string(),
                    Rect::new(cadre.left, cadre.top, cadre.width, bas - cadre.top),
                ))
            })
            .collect();
        for (m, cadre) in poussees {
            self.set_annotation_rect(board_id, &m, cadre);
        }
    }
}

#[cfg(test)]
mod tests;
