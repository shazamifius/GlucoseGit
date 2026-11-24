//! **La flèche en double** (sa note du 24/11, `text-pxtg-2317` ; fiche 149 § 2) : *« si la flèche
//! pointe la même chose mais existe déjà […] il faudrait indiquer à l'utilisateur et savoir s'il
//! veut réellement une duplication. »* Deux flèches aux mêmes bouts suivent le même itinéraire :
//! la seconde se cache sous la première. Au lâcher, la question ; « non » choisit l'ancienne.

use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use glucose_core::types::Annotation;

/// Le titre de la question.
pub const DEJA_RELIES: &str = "Une flèche relie déjà ces deux nœuds";

impl GlucoseApp {
    /// **La flèche qu'on vient de tracer en double une autre ?** Alors son geste s'abandonne —
    /// rien dans l'histoire — et la question se pose. Rend vrai si elle l'est.
    pub(crate) fn demander_si_en_double(&mut self, board: &str, id: &str) -> bool {
        let Some(ancienne) = self.store.fleche_qui_relie_deja(board, id) else {
            return false;
        };
        let Some(Annotation::Arrow {
            source_id: Some(source),
            target_id: Some(cible),
            ..
        }) = self.store.project.annotation(board, id).cloned()
        else {
            return false;
        };
        self.store.cancel_live_edit();
        let question = Question {
            titre: DEJA_RELIES.into(),
            texte: "Une seconde se cacherait sous la première. En tracer une seconde quand même ?"
                .into(),
            choix: vec![
                ("Tracer la seconde".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        let suite = Suite::FlecheEnDouble {
            board: board.to_string(),
            source,
            cible,
            ancienne,
        };
        self.demander(question, suite);
        true
    }

    /// **La réponse** : oui, la seconde naît, d'un geste qu'un Ctrl+Z défait ; sinon, l'ancienne
    /// est choisie — c'est elle qu'on voulait tracer.
    pub(crate) fn repondre_a_la_fleche_en_double(
        &mut self,
        (board, source, cible, ancienne): (String, String, String, String),
        reponse: Reponse,
    ) {
        if reponse != Reponse::Oui {
            self.store.clear_selection();
            self.store.select_annotation(ancienne, false);
            self.mark_dirty();
            return;
        }
        let centre = |id: &str| {
            let b = self.store.boite_d_un_noeud(&board, id).ok()?;
            Some(b.center())
        };
        let (Some(d), Some(a)) = (centre(&source), centre(&cible)) else {
            return;
        };
        let id = self.store.generate_id("arrow");
        let mut fleche = Annotation::arrow(&id, d.x, d.y, a.x, a.y);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut fleche
        {
            *source_id = Some(source);
            *target_id = Some(cible);
        }
        self.store.add_annotation(&board, fleche);
        self.store.clear_selection();
        self.store.select_annotation(id, false);
        self.mark_dirty();
    }
}

#[cfg(test)]
#[path = "fleche_en_double/tests.rs"]
mod tests;
