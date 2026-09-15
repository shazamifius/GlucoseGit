//! Les coudes d'une flèche : les poser, les déplacer, les retirer (ARROW-3).
//!
//! Glucose Tauri donne à une flèche sélectionnée deux sortes de poignées
//! (`ArrowSvgLayer.tsx`) : un disque sur chaque **coude**, qu'on glisse et qu'un double-clic
//! retire, et un losange au **milieu de chaque tronçon**, dont l'appui insère un coude.
//!
//! # Poser et déplacer sont ici **un seul** geste
//!
//! Glucose Tauri insère le coude à l'appui, et il faut relâcher puis le reprendre pour le
//! bouger. Ici, l'appui sur un milieu insère le coude **et ouvre le glisser dessus** : on
//! tire le tronçon là où on le veut d'un seul mouvement.
//!
//! Ce n'est pas une divergence gratuite, c'est DRAW-1 appliqué à un coude : l'objet naît
//! sous la main, il suit la main, et le relâchement décide. Un clic sans mouvement laisse
//! donc un coude au milieu du tronçon — ce que Glucose Tauri fait — et un glisser le pose
//! où on voulait. Aucun mode, aucun modificateur.

use crate::app::{GlucoseApp, LastClickInfo};
use crate::interactions::pick::DOUBLE_CLICK_SLOP_PX;
use glucose_core::arrow::{self, ArrowHandle, HandleKind};
use glucose_core::hit_priority::pick_consts;

/// Le coude qu'on tient sous la main.
#[derive(Debug, Clone)]
pub struct BendSession {
    /// La flèche à laquelle il appartient.
    pub arrow_id: String,
    /// Son rang parmi les coudes de cette flèche.
    pub bend: usize,
}

impl GlucoseApp {
    /// Le rang de ce clic sur une cible **qui n'est pas un nœud**, comme une poignée.
    ///
    /// Le comptage des clics rapprochés sert déjà à ouvrir un nœud ([`super::pick`]) ; une
    /// poignée de coude n'en est pas un, mais la question posée est exactement la même —
    /// « ce clic suit-il le précédent, au même endroit ? ». La clé est donc une chaîne
    /// libre, et le compte se partage : deux horloges pour une question, c'est une
    /// divergence qui attend son bug.
    pub(crate) fn click_count_at(&self, cle: &str) -> u32 {
        let Some(last) = &self.last_click else {
            return 1;
        };
        let elapsed = self.now_ms() - last.at_ms;
        let moved = (last.pos.0 - self.mouse_pos.0).hypot(last.pos.1 - self.mouse_pos.1);
        if last.id == cle && elapsed < pick_consts::DBLCLICK_MS && moved < DOUBLE_CLICK_SLOP_PX {
            last.count + 1
        } else {
            1
        }
    }

    /// Retient ce clic et son rang, pour que le suivant puisse se compter.
    pub(crate) fn remember_click(&mut self, cle: String, count: u32) {
        self.last_click = Some(LastClickInfo {
            at_ms: self.now_ms(),
            pos: self.mouse_pos,
            id: cle,
            count,
        });
    }

    /// La poignée de flèche sous ce point du monde, s'il y en a une.
    ///
    /// Seules les flèches **sélectionnées** en ont : une poignée est une affordance de ce
    /// qu'on manipule, et les faire toutes apparaître couvrirait le canevas de disques.
    fn arrow_handle_at(&self, wx: f64, wy: f64) -> Option<(String, ArrowHandle)> {
        let board = self.store.active_board()?;
        let scale = board.viewport.scale;
        self.store.selected_arrows().into_iter().find_map(|ann| {
            let handle =
                arrow::handle_at(ann, |node| arrow::node_rect(board, node), (wx, wy), scale)?;
            Some((ann.id().to_string(), handle))
        })
    }

    /// L'appui a-t-il attrapé une poignée de flèche ? Si oui, le geste commence.
    ///
    /// Rend `true` quand le clic a été consommé — l'appelant ne doit alors ni sélectionner
    /// ni commencer un glisser de nœud.
    pub fn begin_arrow_bend(&mut self, wx: f64, wy: f64) -> bool {
        let Some((arrow_id, handle)) = self.arrow_handle_at(wx, wy) else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();

        // **Tout** clic sur une poignée se retient, milieu compris : le rang d'un clic se
        // compte contre le précédent, et un clic oublié laisserait le suivant se comparer à
        // un clic bien plus ancien, sur tout autre chose.
        let cle = match handle.kind {
            HandleKind::Bend(index) => format!("bend:{arrow_id}:{index}"),
            HandleKind::Midpoint(segment) => format!("mid:{arrow_id}:{segment}"),
        };
        let rang = self.click_count_at(&cle);
        self.remember_click(cle, rang);

        // Un double-clic sur un coude le retire : une flèche qu'on a trop pliée se déplie du
        // même geste qui l'a pliée, et au même endroit.
        if let HandleKind::Bend(index) = handle.kind {
            if rang > 1 {
                self.store.remove_arrow_bend(&board, &arrow_id, index);
                self.mark_dirty();
                return true;
            }
        }

        // Tout le geste — l'insertion éventuelle et le glisser qui suit — tient dans une
        // seule entrée d'annulation.
        self.store.begin_live_edit();
        let bend = match handle.kind {
            HandleKind::Bend(index) => index,
            HandleKind::Midpoint(segment) => {
                self.store
                    .insert_arrow_bend(&board, &arrow_id, segment, handle.at);
                segment
            }
        };
        self.bend_session = Some(BendSession { arrow_id, bend });
        self.mark_dirty();
        true
    }

    /// Le glisser en cours amène le coude sous le curseur.
    pub fn update_bend(&mut self, wx: f64, wy: f64) {
        let Some(session) = self.bend_session.clone() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        self.store
            .move_arrow_bend(&board, &session.arrow_id, session.bend, (wx, wy));
        self.mark_dirty();
    }

    /// Referme le geste du coude sur une seule entrée d'annulation.
    pub fn finish_bend(&mut self) {
        if self.bend_session.take().is_some() {
            self.store.end_live_edit();
            self.mark_dirty();
        }
    }
}

#[cfg(test)]
mod tests;
