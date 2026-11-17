//! **Les deux bouts d'une flèche** (BOUT-1, fiche 118) : les rebrancher, et lire ce qu'ils
//! tiennent. Glucose Tauri ne leur donnait aucune poignée ; sa capture du 17/11 : *« aucun
//! système permettant de bouger indépendamment les points A et B d'une flèche »*.

use super::Store;
use crate::arrow::Extremite;
use crate::types::Annotation;

impl Store {
    /// **Rebranche un bout de flèche** (BOUT-1) : sur un nœud, ou libre en un point du monde.
    /// Le bloc et le passage de texte qu'il désignait partent avec le nœud qu'il quitte ; le
    /// point reste ce qu'il est, la place d'un bout libre.
    pub fn rebrancher_la_fleche(
        &mut self,
        board_id: &str,
        arrow_id: &str,
        bout: Extremite,
        (noeud, point): (Option<&str>, (f64, f64)),
    ) {
        self.update_annotation(board_id, arrow_id, |ann| {
            let Annotation::Arrow {
                x,
                y,
                x2,
                y2,
                source_id,
                target_id,
                source_block_id,
                target_block_id,
                source_text_sel,
                target_text_sel,
                ..
            } = ann
            else {
                return;
            };
            let (px, py, id, bloc, passage) = match bout {
                Extremite::Depart => (x, y, source_id, source_block_id, source_text_sel),
                Extremite::Arrivee => (x2, y2, target_id, target_block_id, target_text_sel),
            };
            if id.as_deref() != noeud {
                (*bloc, *passage) = (None, None);
            }
            *id = noeud.map(str::to_string);
            (*px, *py) = point;
        });
    }

    /// **Le nœud que tient l'autre bout de cette flèche**, sur le tableau ouvert : celui qu'un
    /// bout glissé ne rejoint jamais — la flèche ne se referme pas sur elle-même (ARROW-2).
    pub fn noeud_de_l_autre_bout(&self, fleche: &str, bout: Extremite) -> Option<String> {
        self.active_board()?
            .annotations
            .iter()
            .find_map(|a| match a {
                Annotation::Arrow {
                    id,
                    source_id,
                    target_id,
                    ..
                } if id == fleche => match bout {
                    Extremite::Depart => target_id.clone(),
                    Extremite::Arrivee => source_id.clone(),
                },
                _ => None,
            })
    }
}
