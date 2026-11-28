//! **POUSSER-145, lettre à lettre** (fiche 145) : pendant qu'on écrit, le nœud grandit dans le
//! document et ce qui était dessous s'écarte — à chaque ligne gagnée ou perdue, pas à chaque lettre.
//!
//! Chaque changement de hauteur est un pas court, fermé aussitôt sous le numéro de la saisie : aucun
//! geste ne reste ouvert pendant qu'on tape, les agents et les pairs écrivent entre deux pas, et le
//! journal fond les pas d'une même saisie, validation comprise, en un seul `Ctrl+Z` (FONDRE-145).
//! La loi étant une fonction pure du bas (`Store::pousser_en_ecrivant`), effacer fait remonter.

use crate::app::GlucoseApp;
use crate::renderer::card::hauteur_poussee;
use crate::renderer::richtext::TextMode;
use glucose_core::geometry::Rect;
use glucose_core::types::Annotation;

#[cfg(test)]
mod tests;

/// Ce qu'une saisie a poussé depuis son ouverture. Il vit à côté de la session, comme l'historique
/// du texte : la session est copiée à chaque frappe (COMPOSANT-2).
#[derive(Debug, Default)]
pub struct Poussee {
    /// Le numéro de la saisie, l'étiquette de ses pas : il ne revient jamais en arrière.
    numero: u64,
    /// La boîte d'origine de ce que la saisie a touché : le nœud écrit, puis chaque voisin poussé.
    origines: Vec<(String, Rect)>,
    /// Leurs boîtes telles que le dernier pas les a laissées. En retrouver une autre, c'est qu'une
    /// autre main (un agent, un pair, la sienne) l'a prise : la saisie l'oublie.
    laissees: Vec<(String, Rect)>,
}

impl GlucoseApp {
    /// Une saisie s'ouvre : un numéro neuf, rien de touché. L'origine du nœud écrit se prend à son
    /// premier pas, telle qu'on la trouve alors.
    pub(super) fn ouvrir_la_poussee(&mut self) {
        let p = &mut self.poussee;
        p.numero += 1;
        p.origines.clear();
        p.laissees.clear();
    }

    /// Le numéro de la saisie en cours : l'étiquette de sa validation.
    pub(super) fn numero_de_la_saisie(&self) -> u64 {
        self.poussee.numero
    }

    /// À chaque changement du tampon : un pas, s'il change la hauteur.
    pub(crate) fn suivre_la_hauteur(&mut self) {
        let Some(id) = self.editing_session.as_ref().map(|s| s.ann_id.clone()) else {
            return;
        };
        // Un geste déjà ouvert (un glisser) n'est pas le nôtre : le pas suivant, ou la validation,
        // rattrapera la hauteur. Un lecteur n'écrit rien : sa hauteur est celle qu'il lit
        // (LECTEUR-1), et un refus qu'il n'a pas demandé ne doit pas lui être dit.
        if self.store.in_live_edit() || self.store.lecteur() {
            return;
        }
        self.store.begin_live_edit();
        self.pas_de_la_saisie(&id);
        self.store.end_live_edit_etiquete(self.poussee.numero);
    }

    /// **Un pas, dans le geste ouvert** : ce qu'une autre main a pris depuis le précédent est
    /// oublié (le nœud écrit, lui, reprend son origine là où on le trouve), puis la hauteur s'écrit.
    pub(super) fn pas_de_la_saisie(&mut self, id: &str) {
        let laissees = std::mem::take(&mut self.poussee.laissees);
        let mut origines = std::mem::take(&mut self.poussee.origines);
        origines.retain(|(qui, _)| {
            let laissee = laissees.iter().find(|l| &l.0 == qui).map(|l| l.1);
            laissee.is_none() || laissee == self.boite_de(qui)
        });
        self.ecrire_la_hauteur(id, &mut origines);
        let laissees = (origines.iter())
            .filter_map(|(qui, _)| Some((qui.clone(), self.boite_de(qui)?)))
            .collect();
        self.poussee = Poussee {
            numero: self.poussee.numero,
            origines,
            laissees,
        };
    }

    /// TEXT-FIT-1 hors d'une saisie : la hauteur que son texte pousse, depuis sa boîte d'aujourd'hui.
    pub fn fit_text_card_height(&mut self, ann_id: &str) {
        self.ecrire_la_hauteur(ann_id, &mut Vec::new());
    }

    /// **TEXT-FIT-1 et POUSSER-145** : écrit la hauteur que pousse le texte de `ann_id` — celui
    /// qu'on écrit s'il est en saisie, coupé comme on l'édite, comme le dessin le montre —, jamais
    /// sous sa boîte d'origine : une hauteur tirée à la main est un plancher qu'une saisie ne
    /// reprend pas ; pour la faire redescendre, on tire, et le geste s'arrête au texte. Ses
    /// membranes le suivent, ce qui était dessous s'écarte, ou revient quand il raccourcit.
    fn ecrire_la_hauteur(&mut self, ann_id: &str, origines: &mut Vec<(String, Rect)>) {
        let board = self.store.project.active_board_id.clone();
        let Some((rect, texte)) =
            (self.store.project.annotation(&board, ann_id)).and_then(|a| match a {
                Annotation::Text { text, .. } => Some((a.rect()?, text.clone())),
                _ => None,
            })
        else {
            return;
        };
        let (texte, mode) = match self.editing_session.as_ref() {
            Some(s) if s.ann_id == ann_id => (s.buffer.clone(), TextMode::Source),
            _ => (texte, TextMode::Rendered),
        };
        if !origines.iter().any(|o| o.0 == ann_id) {
            origines.push((ann_id.to_string(), rect));
        }
        let plancher =
            (origines.iter().find(|o| o.0 == ann_id)).map_or(rect.height, |o| o.1.height);
        let r = &self.renderer;
        let mesure = (&r.typography, &r.math, &r.mises_en_page);
        let height = hauteur_poussee(mesure, &texte, rect.width, mode, plancher);
        if (height - rect.height).abs() > 1e-6 {
            self.store
                .set_annotation_rect(&board, ann_id, Rect { height, ..rect });
            (self.store).pousser_en_ecrivant(&board, origines, ann_id, rect.top + height);
        }
    }

    /// La boîte d'un nœud du tableau ouvert, telle qu'elle est maintenant.
    fn boite_de(&self, id: &str) -> Option<Rect> {
        let board = &self.store.project.active_board_id;
        self.store.project.annotation(board, id)?.rect()
    }
}
