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
