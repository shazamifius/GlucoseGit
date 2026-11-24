//! **DEPLACER-1 — une IA déplace un nœud comme la main le déplace** (fiche 139).
//!
//! Le 24/11, en rangeant son archive, `glucose_deplacer` sur une membrane ne bougeait qu'elle : les
//! nœuds d'un « Groupe » sont restés à 4 000 unités de leur membrane. Sous sa main, une membrane
//! emporte ce qu'elle porte (MEMB-1). Un Glucose se lit et se manie *« par un humain ET par une IA,
//! de quasi la même façon »* (VISION § 1) : le geste de l'agent passe donc par la translation même
//! du glisser, et rien d'autre.

use super::Refus;
use crate::geometry::Rect;
use crate::store::Store;

impl Store {
    /// **Déplace un nœud comme la main le déplace** (DEPLACER-1, fiche 139) : le coin haut gauche
    /// de sa boîte va en `(x, y)`, une membrane emporte ce qu'elle porte, les flèches accrochées
    /// suivent, et il appartient ensuite à la membrane où il tombe (MEMB-1). Un dossier se
    /// déplace aussi ; une photo verrouillée, non.
    ///
    /// Le nœud devient la sélection le temps du geste — `geste_d_agent` rend ensuite celle de
    /// l'utilisateur (AGENT-2) —, et c'est la translation même du glisser qui bouge
    /// (`move_selected`) : une seule loi pour les deux mains. La première version ne translatait
    /// que le nœud nommé — déplacée par une IA, une membrane laissait son contenu derrière elle
    /// (le 24/11, en rangeant son archive) — et ne connaissait pas les dossiers.
    pub fn deplacer_pour_un_agent(
        &mut self,
        board_id: &str,
        id: &str,
        (x, y): (f64, f64),
    ) -> Result<(), Refus> {
        let photo = self.image(board_id, id).map(|i| (i.locked, i.rect()));
        let boite = match (photo, self.boite_d_un_dossier(board_id, id)) {
            (Some((true, _)), _) => return Err(Refus::Verrouillee(id.to_string())),
            (Some((false, boite)), _) => {
                self.select_image(id.to_string(), false);
                boite
            }
            (None, Some(boite)) => {
                self.select_folder(id.to_string(), false);
                boite
            }
            (None, None) => {
                let boite = self.boite_d_un_noeud(board_id, id)?;
                self.select_annotation(id.to_string(), false);
                boite
            }
        };
        self.move_selected(board_id, x - boite.left, y - boite.top);
        self.rattacher_la_selection(board_id);
        Ok(())
    }

    /// La boîte d'un dossier de ce tableau, s'il en est un.
    fn boite_d_un_dossier(&self, board_id: &str, id: &str) -> Option<Rect> {
        let b = self.project.boards.iter().find(|b| b.id == board_id)?;
        b.folders.iter().find(|f| f.id == id).map(|f| f.rect())
    }
}
