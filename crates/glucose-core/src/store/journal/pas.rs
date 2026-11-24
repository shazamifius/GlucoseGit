//! **Les pas d'un geste** : ce qui se fond en un seul Ctrl+Z.
//!
//! GLISSER-1 : les pas d'un glisser deviennent sa translation totale, dans le geste ouvert.
//!
//! FONDRE-145 (fiche 145) : les gestes courts d'une même saisie — la hauteur d'un texte qui gagne
//! une ligne, ce qu'elle écarte, puis la validation — se fondent en un seul **sur la pile
//! d'annulation**. Un geste resté ouvert toute la saisie aurait refusé les agents des minutes durant
//! (`Refus::GesteEnCours`) : chaque pas se ferme donc, et la pile les fond ensuite, par leur
//! étiquette — aucun délai, aucune constante. Si un autre geste s'intercale (un agent), la saisie en
//! fait deux, dans l'ordre vrai. **Ce qui est publié ne change pas** : l'histoire et les pairs
//! reçoivent chaque pas.

use super::{Edit, Journal, Transaction};

/// Les étiquettes de la pile : celle du geste qui se ferme, celle de son sommet.
#[derive(Debug, Clone, Default)]
pub(super) struct Etiquettes {
    prochaine: Option<u64>,
    sommet: Option<u64>,
}

impl Journal {
    /// Ferme le geste ouvert sous cette étiquette (FONDRE-145) : s'il suit sur la pile un geste de
    /// la même, il s'y fond. Rend `true` s'il publie.
    pub fn end_etiquete(&mut self, etiquette: u64) -> bool {
        self.etiquettes.prochaine = Some(etiquette);
        let publie = self.end();
        self.etiquettes.prochaine = None;
        publie
    }

    /// Empile un geste validé — ou le fond dans le sommet, s'il porte son étiquette.
    pub(super) fn empiler(&mut self, tx: Transaction) {
        let etiquette = self.etiquettes.prochaine.take();
        let meme = etiquette.is_some() && etiquette == self.etiquettes.sommet;
        // Défaire efface l'étiquette du sommet (`oublier_le_sommet`) : rien de défait n'est refondu.
        if let Some(sommet) = self.done.last_mut().filter(|_| meme) {
            sommet.edits.extend(tx.edits);
            sommet.fondre_les_pas();
            // Une saisie revenue à son point de départ ne laisse rien à défaire.
            if sommet.is_empty() {
                self.done.pop();
                self.etiquettes.sommet = None;
            }
            return;
        }
        self.etiquettes.sommet = etiquette;
        self.done.push(tx);
        if self.done.len() > self.max_depth {
            self.done.remove(0);
        }
        self.undone.clear();
    }

    /// Un canevas partagé n'empile pas ici : l'étiquette suit le geste publié jusqu'à sa pile
    /// (`Participation::empiler`).
    pub(super) fn etiqueter_le_dernier_ecrit(&mut self) {
        if let Some(ecrit) = self.ecrits.last_mut() {
            ecrit.etiquette = self.etiquettes.prochaine.take();
        }
    }

    /// La pile a bougé sans geste (défaire, refaire) : son sommet n'a plus d'étiquette.
    pub(super) fn oublier_le_sommet(&mut self) {
        self.etiquettes.sommet = None;
    }

    /// **Remplace ces éditions du geste ouvert** par une seule, ou par rien (GLISSER-1) : les pas
    /// d'un glisser deviennent sa translation totale. Rend `false` hors geste, ou si les rangs
    /// sortent du geste — rien n'est alors touché.
    pub(crate) fn remplacer_dans_le_geste(
        &mut self,
        rangs: std::ops::Range<usize>,
        par: Option<Edit>,
    ) -> bool {
        let Some(tx) = self.open.as_mut() else {
            return false;
        };
        if rangs.start > rangs.end || rangs.end > tx.edits.len() {
            return false;
        }
        tx.edits.splice(rangs, par);
        true
    }
}

#[cfg(test)]
#[path = "pas_tests.rs"]
mod tests;
