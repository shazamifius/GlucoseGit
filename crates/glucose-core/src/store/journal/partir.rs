//! **NOMMER-1 — un geste part nommé dans l'état qui le suit** (CLIENT-1, fiche 117).
//!
//! Un geste partagé désigne ce qu'il touche par le nom (PARTAGE-1). Presque toutes les éditions
//! portent déjà leurs noms ; **le déplacement** ne garde que des rangs (JRN-4), qui ne disent un
//! nom que dans l'état où ils ont été écrits — celui qui suit le geste. Nommer chaque geste de la
//! file dans l'état d'**après tous** serait faux dès qu'un geste suivant décale ces rangs (un
//! déplacement, puis un retrait dans la même image : le déplacement nommerait le voisin).
//!
//! La file se remonte donc à rebours : chaque geste se nomme dans l'état courant, puis se défait,
//! jusqu'au plus ancien déplacement — et tout se refait dans l'ordre. Le coût est celui des gestes
//! (JRN-1) ; sans déplacement, rien ne se défait.

use super::partage::GestePartage;
use super::{Edit, Journal, Main};
use crate::types::Project;

impl Journal {
    /// **Ce qui part aux autres** depuis la dernière fois : chaque geste de ce Glucose, dans
    /// l'ordre, nommé dans l'état qui le suivait, avec sa main. Chacun n'est nommé qu'une fois.
    pub(crate) fn nommer_ce_qui_part(
        &mut self,
        projet: &mut Project,
    ) -> Vec<(GestePartage, Main, Option<u64>)> {
        let deplace = |e: &super::Ecrit| {
            e.part
                && e.transaction
                    .edits
                    .iter()
                    .any(|x| matches!(x, Edit::Translation { .. }))
        };
        let plus_ancien = self.ecrits.iter().position(deplace);
        let mut noms = Vec::new();
        let mut defaits = Vec::new();
        for j in (0..self.ecrits.len()).rev() {
            let e = &mut self.ecrits[j];
            if e.part {
                e.part = false;
                noms.push((
                    e.transaction.a_partager(projet),
                    e.main.clone(),
                    e.etiquette,
                ));
            }
            if plus_ancien.is_some_and(|k| j > k) {
                let defait = e.transaction.clone().inverse().apply(projet);
                debug_assert!(defait, "un geste appliqué se défait (JRN-2)");
                defaits.push(j);
            }
        }
        for j in defaits.into_iter().rev() {
            let refait = self.ecrits[j].transaction.apply(projet);
            debug_assert!(refait, "un geste défait se refait (JRN-2)");
        }
        noms.reverse();
        noms
    }
}
