//! **La palette défile** : l'évidence au clavier, la fenêtre à la molette et au doigt.
//!
//! Une seule loi place la fenêtre ([`suivre_l_evidence`]) : l'évidence y reste toujours, et la
//! fenêtre ne bouge que si l'évidence en sort. La molette et les rangées « ↑ / ↓ » déplacent la
//! fenêtre, et l'évidence les suit à son bord.

use super::{trouvees_pour, Etat};
use crate::app::GlucoseApp;
use crate::persist::documents::MONTRES;
use crate::ui::question::Suite;
use winit::keyboard::{Key, NamedKey};

/// **La fenêtre qui garde l'évidence** : elle ne bouge que si l'évidence en sort, juste assez.
fn suivre_l_evidence(etat: &mut Etat, total: usize) {
    etat.choisie = etat.choisie.min(total.saturating_sub(1));
    if etat.choisie < etat.debut {
        etat.debut = etat.choisie;
    } else if etat.choisie >= etat.debut + MONTRES {
        etat.debut = etat.choisie + 1 - MONTRES;
    }
}

impl GlucoseApp {
    /// L'état de la palette posée, s'il y en a une.
    fn etat_de_la_palette(&self) -> Option<Etat> {
        match &self.ui.question {
            Some((_, Suite::Palette(etat))) => Some(etat.clone()),
            _ => None,
        }
    }

    /// **Une touche pour la palette** : les flèches et Tab déplacent l'évidence d'une
    /// commande, Page haut et Page bas d'une fenêtre. Rend `false` si la touche n'est pas à elle
    /// — Entrée, Échap et l'écriture restent à la question.
    pub(crate) fn touche_de_la_palette(&mut self, touche: &Key, arriere: bool) -> bool {
        let Some(mut etat) = self.etat_de_la_palette() else {
            return false;
        };
        let pas: isize = match touche {
            Key::Named(NamedKey::ArrowDown) => 1,
            Key::Named(NamedKey::ArrowUp) => -1,
            Key::Named(NamedKey::Tab) if arriere => -1,
            Key::Named(NamedKey::Tab) => 1,
            Key::Named(NamedKey::PageDown) => MONTRES as isize,
            Key::Named(NamedKey::PageUp) => -(MONTRES as isize),
            _ => return false,
        };
        let total = trouvees_pour(&etat.filtre, etat.but).len();
        etat.choisie = etat.choisie.saturating_add_signed(pas);
        suivre_l_evidence(&mut etat, total);
        self.poser_la_palette(etat);
        true
    }

    /// **Fait défiler la fenêtre de `rangees`** — la molette, le pavé, les rangées « ↑ / ↓ » :
    /// l'évidence reste dans la fenêtre, à son bord si elle en sortait.
    pub(crate) fn defiler_la_palette(&mut self, rangees: isize) {
        let Some(mut etat) = self.etat_de_la_palette() else {
            return;
        };
        let total = trouvees_pour(&etat.filtre, etat.but).len();
        let dernier_debut = total.saturating_sub(MONTRES);
        etat.debut = etat.debut.saturating_add_signed(rangees).min(dernier_debut);
        etat.choisie = etat
            .choisie
            .clamp(etat.debut, (etat.debut + MONTRES).saturating_sub(1));
        suivre_l_evidence(&mut etat, total);
        self.poser_la_palette(etat);
    }

    /// **La molette sur la palette** : une rangée par cran ; le pavé, une rangée par hauteur de
    /// rangée parcourue — le reste attend le geste suivant. Rend `false` sans palette.
    pub(crate) fn molette_sur_la_palette(&mut self, rangees: f32) -> bool {
        if self.etat_de_la_palette().is_none() {
            return false;
        }
        let tout = self.ui.reste_de_la_molette + rangees;
        let entier = tout.trunc();
        self.ui.reste_de_la_molette = tout - entier;
        if entier != 0.0 {
            self.defiler_la_palette(entier as isize);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **La fenêtre garde l'évidence**, et ne bouge que juste assez.
    #[test]
    fn test_p4_la_fenetre_suit_l_evidence_juste_assez() {
        let mut etat = Etat {
            filtre: String::new(),
            debut: 0,
            choisie: MONTRES,
            ..Etat::default()
        };
        suivre_l_evidence(&mut etat, 50);
        assert_eq!(
            etat.debut, 1,
            "une rangée sous la fenêtre : elle descend d'une"
        );
        etat.choisie = 0;
        suivre_l_evidence(&mut etat, 50);
        assert_eq!(etat.debut, 0);
        etat.choisie = 99;
        suivre_l_evidence(&mut etat, 50);
        assert_eq!(
            (etat.choisie, etat.debut),
            (49, 50 - MONTRES),
            "bornée à la dernière"
        );
    }
}
