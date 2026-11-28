//! **LECTEUR-1 — un lecteur ne touche pas le canevas partagé** (fiche 193 ; sa Q17 : *« lecteur »*,
//! le premier des trois grades).
//!
//! Celui qui entre par le code « regarder » voyait chacun de ses gestes s'appliquer, puis se
//! défaire au moment de partir (CLIENT-1, fiche 117 § 8 : *« un lecteur voit ses gestes se défaire,
//! au lieu d'un canevas qui ne se laisse pas toucher »*) : un nœud glissé suivait la main, puis
//! revenait ; chaque refus laissait deux gestes dans son journal. Comme un lecteur chez Figma, il
//! regarde, choisit, cherche, change d'onglet — rien de ce qui partirait ne se fait.
//!
//! **Une seule porte** : toute édition locale passe par le journal (`record_edit`,
//! `record_as_one_gesture`, les pas d'un glisser) ; c'est là qu'elle est refusée — défaite sur-le-champ,
//! avant toute image, sans trace. Un glisser est refusé avant son premier pas : rien ne bouge
//! sous la main. Changer d'onglet n'est pas une édition du journal : il passe.

use super::journal::Edit;
use super::Store;

impl Store {
    /// Ce canevas est-il partagé avec un code qui ne fait que regarder ?
    pub fn lecteur(&self) -> bool {
        self.partage.as_ref().is_some_and(|p| !p.ecrivain)
    }

    /// **Un geste refusé au lecteur**, avant qu'il ne fasse rien : rend `true` et le compte (ce
    /// que l'interface lui dit) s'il est lecteur.
    pub fn lecteur_refuse_un_geste(&mut self) -> bool {
        match self.partage.as_mut() {
            Some(p) if !p.ecrivain => {
                p.defaits_au_lecteur += 1;
                true
            }
            _ => false,
        }
    }

    /// **Les éditions d'un lecteur, défaites avant d'être vues** : le geste entier est retourné,
    /// dans l'ordre inverse, et rien n'entre au journal. Rend `true` s'il est refusé.
    pub(super) fn refuser_au_lecteur(&mut self, edits: &[Edit]) -> bool {
        if !self.lecteur() || edits.iter().all(Edit::is_noop) {
            return false;
        }
        for edit in edits.iter().rev() {
            let mut inverse = edit.clone();
            inverse.flip();
            inverse.apply(&mut self.project);
        }
        self.settle_navigation();
        self.lecteur_refuse_un_geste()
    }
}
