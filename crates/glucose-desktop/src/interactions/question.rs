//! **Poser une question, et suivre sa réponse** (QUESTION-1, fiche 56).
//!
//! Là où le système a ses dialogues — Windows, Linux, macOS —, la question s'y pose, et sa
//! suite part aussitôt : rien ne change pour le bureau. Là où il n'en a pas — Android —, Glucose
//! la dessine ([`crate::ui::question`]) ; elle prend alors tout toucher jusqu'à sa réponse, et
//! la suite part à ce moment-là.
//!
//! Une seule écriture de chaque question, et de ce qu'elle déclenche : la voie ne change que la
//! façon de demander.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use crate::ui::question::{placer, reponse_sous, Question, Reponse, Suite};

impl GlucoseApp {
    /// **Pose cette question** ; sa réponse déclenchera `suite`.
    pub(crate) fn demander(&mut self, question: Question, suite: Suite) {
        if !self.ui.questions_dessinees {
            let reponse = self.sous_un_dialogue(|ancre| crate::dialogue::poser(ancre, &question));
            self.suivre(suite, reponse);
            return;
        }
        self.ui.question = Some((question, suite));
        self.mark_dirty();
    }

    /// **La question prend tout ce qui la touche** tant qu'elle est posée : ce qui est
    /// derrière attend. L'appui retient la réponse pressée ; elle part au relâchement
    /// ([`Self::relacher_la_question`]).
    pub(crate) fn cliquer_la_question(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        let ecran = (screen.width, screen.height);
        let placee = placer(question, &self.renderer.typography, ecran, screen.scale);
        question.sous_le_doigt = reponse_sous(&placee, pointer.x, pointer.y);
        self.mark_dirty();
        true
    }

    /// **Le bouton se relève** : la réponse part s'il se relève sur celle qu'il a pressée
    /// (APPUI-1). Rend `true` si l'appui était à elle. Sinon, le relâchement termine le geste
    /// que l'appui avait commencé — un canevas qui glissait sous le doigt quand la question a
    /// paru ne doit pas continuer de le suivre.
    pub(crate) fn relacher_la_question(&mut self) -> bool {
        let (pointer, (largeur, hauteur)) = (self.pointer(), self.taille_de_la_fenetre());
        let screen = self.screen_frame(largeur, hauteur);
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        let Some(pressee) = question.sous_le_doigt.take() else {
            return false;
        };
        let ecran = (screen.width, screen.height);
        let placee = placer(question, &self.renderer.typography, ecran, screen.scale);
        if reponse_sous(&placee, pointer.x, pointer.y) == Some(pressee) {
            if let Some((_, suite)) = self.ui.question.take() {
                self.suivre(suite, pressee);
            }
        }
        self.mark_dirty();
        true
    }

    /// **Un appui long sur une question** n'y répond rien : la réponse pressée s'oublie.
    /// Rend `true` s'il y avait une question.
    pub(crate) fn appui_long_sur_la_question(&mut self) -> bool {
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        question.sous_le_doigt = None;
        self.mark_dirty();
        true
    }

    /// Ce que la réponse déclenche.
    fn suivre(&mut self, suite: Suite, reponse: Reponse) {
        match suite {
            Suite::NouveauDocument if reponse == Reponse::Oui => self.adopter_si_on_laisse(),
            Suite::NouveauDocument => {}
            Suite::Telemetrie => self.repondre_a_la_telemetrie(reponse == Reponse::Oui),
            Suite::Ouvrir(chemins) => {
                if let Some(chemin) = reponse_choisie(reponse).and_then(|i| chemins.get(i)) {
                    if self.laisser_le_document() {
                        self.open_from(chemin.clone());
                    }
                }
            }
        }
    }
}

/// Le rang choisi dans une liste, si la réponse en est un.
fn reponse_choisie(reponse: Reponse) -> Option<usize> {
    match reponse {
        Reponse::Choix(i) => Some(i),
        _ => None,
    }
}
